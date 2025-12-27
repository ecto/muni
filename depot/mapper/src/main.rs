//! Mapper Service
//!
//! Watches for new sessions uploaded by rovers and processes them into maps.
//!
//! Responsibilities:
//! - Monitor sessions directory for new uploads
//! - Parse session metadata and validate completeness
//! - Queue sessions for processing
//! - Run Gaussian splatting pipeline (or invoke external processor)
//! - Update map registry with results
//! - Merge new sessions into existing maps when regions overlap

use chrono::{DateTime, Utc};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

// =============================================================================
// Types
// =============================================================================

#[derive(Error, Debug)]
pub enum MapperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),
    #[error("Session incomplete: {0}")]
    IncompleteSession(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

/// GPS bounding box
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpsBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GpsBounds {
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
        )
    }

    pub fn overlaps(&self, other: &GpsBounds) -> bool {
        self.min_lat <= other.max_lat
            && self.max_lat >= other.min_lat
            && self.min_lon <= other.max_lon
            && self.max_lon >= other.min_lon
    }

    pub fn expand(&mut self, other: &GpsBounds) {
        self.min_lat = self.min_lat.min(other.min_lat);
        self.max_lat = self.max_lat.max(other.max_lat);
        self.min_lon = self.min_lon.min(other.min_lon);
        self.max_lon = self.max_lon.max(other.max_lon);
    }
}

/// Session metadata (written by rover)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: Uuid,
    pub rover_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub gps_bounds: Option<GpsBounds>,
    pub lidar_frames: u32,
    pub camera_frames: u32,
    pub telemetry_file: String,
}

/// Session processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Just discovered, not yet validated
    Pending,
    /// Validated and queued for processing
    Queued,
    /// Currently being processed
    Processing,
    /// Successfully processed and merged into a map
    Processed,
    /// Processing failed
    Failed,
    /// Incomplete or invalid session
    Invalid,
}

/// Session record in our database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub rover_id: String,
    pub path: PathBuf,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub gps_bounds: Option<GpsBounds>,
    pub lidar_frames: u32,
    pub camera_frames: u32,
    pub status: SessionStatus,
    pub map_id: Option<Uuid>,
    pub discovered_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Map metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapManifest {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub bounds: GpsBounds,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assets: MapAssets,
    pub sessions: Vec<MapSessionRef>,
    pub stats: MapStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapAssets {
    pub splat: Option<String>,
    pub pointcloud: Option<String>,
    pub mesh: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSessionRef {
    pub session_id: Uuid,
    pub rover_id: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapStats {
    pub total_points: u64,
    pub total_splats: u64,
    pub coverage_pct: f32,
}

/// Map index (list of all maps)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapIndex {
    pub maps: Vec<MapIndexEntry>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapIndexEntry {
    pub id: Uuid,
    pub name: String,
    pub bounds: GpsBounds,
    pub version: u32,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// State
// =============================================================================

struct MapperState {
    sessions: HashMap<Uuid, Session>,
    maps: HashMap<Uuid, MapManifest>,
    sessions_dir: PathBuf,
    maps_dir: PathBuf,
}

impl MapperState {
    fn new(sessions_dir: PathBuf, maps_dir: PathBuf) -> Self {
        Self {
            sessions: HashMap::new(),
            maps: HashMap::new(),
            sessions_dir,
            maps_dir,
        }
    }

    /// Find a map that overlaps with the given bounds
    fn find_overlapping_map(&self, bounds: &GpsBounds) -> Option<&MapManifest> {
        self.maps.values().find(|m| m.bounds.overlaps(bounds))
    }

    /// Save the map index to disk
    async fn save_index(&self) -> Result<(), MapperError> {
        let index = MapIndex {
            maps: self
                .maps
                .values()
                .map(|m| MapIndexEntry {
                    id: m.id,
                    name: m.name.clone(),
                    bounds: m.bounds.clone(),
                    version: m.version,
                    updated_at: m.updated_at,
                })
                .collect(),
            updated_at: Utc::now(),
        };

        let index_path = self.maps_dir.join("index.json");
        let json = serde_json::to_string_pretty(&index)?;
        tokio::fs::write(&index_path, json).await?;
        debug!(path = %index_path.display(), "Saved map index");
        Ok(())
    }

    /// Save a map manifest to disk
    async fn save_manifest(&self, map: &MapManifest) -> Result<(), MapperError> {
        let map_dir = self.maps_dir.join(&map.name);
        tokio::fs::create_dir_all(&map_dir).await?;

        let manifest_path = map_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(map)?;
        tokio::fs::write(&manifest_path, json).await?;
        debug!(path = %manifest_path.display(), "Saved map manifest");
        Ok(())
    }

    /// Save sessions database to disk
    async fn save_sessions(&self) -> Result<(), MapperError> {
        let sessions_db_path = self.maps_dir.join("sessions.json");
        let sessions: Vec<_> = self.sessions.values().collect();
        let json = serde_json::to_string_pretty(&sessions)?;
        tokio::fs::write(&sessions_db_path, json).await?;
        debug!("Saved sessions database ({} sessions)", sessions.len());
        Ok(())
    }

    /// Load state from disk
    async fn load(&mut self) -> Result<(), MapperError> {
        // Load sessions database
        let sessions_db_path = self.maps_dir.join("sessions.json");
        if sessions_db_path.exists() {
            let json = tokio::fs::read_to_string(&sessions_db_path).await?;
            let sessions: Vec<Session> = serde_json::from_str(&json)?;
            for session in sessions {
                self.sessions.insert(session.id, session);
            }
            info!("Loaded {} sessions from database", self.sessions.len());
        }

        // Load map manifests
        let index_path = self.maps_dir.join("index.json");
        if index_path.exists() {
            let json = tokio::fs::read_to_string(&index_path).await?;
            let index: MapIndex = serde_json::from_str(&json)?;

            for entry in index.maps {
                let manifest_path = self.maps_dir.join(&entry.name).join("manifest.json");
                if manifest_path.exists() {
                    let json = tokio::fs::read_to_string(&manifest_path).await?;
                    let manifest: MapManifest = serde_json::from_str(&json)?;
                    self.maps.insert(manifest.id, manifest);
                }
            }
            info!("Loaded {} maps from index", self.maps.len());
        }

        Ok(())
    }
}

type SharedState = Arc<RwLock<MapperState>>;

// =============================================================================
// Session Discovery
// =============================================================================

/// Check if a session directory is complete and ready for processing
fn validate_session(session_path: &Path) -> Result<SessionMetadata, MapperError> {
    let metadata_path = session_path.join("metadata.json");

    if !metadata_path.exists() {
        return Err(MapperError::IncompleteSession(
            "metadata.json not found".into(),
        ));
    }

    let json = std::fs::read_to_string(&metadata_path)?;
    let metadata: SessionMetadata = serde_json::from_str(&json)?;

    // Check for required files
    let telemetry_path = session_path.join(&metadata.telemetry_file);
    if !telemetry_path.exists() {
        return Err(MapperError::IncompleteSession(format!(
            "Telemetry file {} not found",
            metadata.telemetry_file
        )));
    }

    // Session must have ended (not still recording)
    if metadata.ended_at.is_none() {
        return Err(MapperError::IncompleteSession(
            "Session still in progress".into(),
        ));
    }

    Ok(metadata)
}

/// Scan sessions directory for all session directories
fn scan_sessions(sessions_dir: &Path) -> Vec<PathBuf> {
    let mut sessions = Vec::new();

    // Sessions are organized as: sessions/{rover_id}/sessions/{timestamp}/
    for entry in WalkDir::new(sessions_dir)
        .min_depth(1)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() && path.join("metadata.json").exists() {
            sessions.push(path.to_path_buf());
        }
    }

    sessions
}

/// Process a newly discovered session
async fn process_new_session(
    state: SharedState,
    session_path: PathBuf,
) -> Result<(), MapperError> {
    info!(path = %session_path.display(), "Processing new session");

    // Validate session
    let metadata = match validate_session(&session_path) {
        Ok(m) => m,
        Err(e) => {
            warn!(path = %session_path.display(), error = %e, "Invalid session");
            return Ok(()); // Not an error, just skip
        }
    };

    let mut state = state.write().await;

    // Check if we already know about this session
    if state.sessions.contains_key(&metadata.session_id) {
        debug!(id = %metadata.session_id, "Session already known");
        return Ok(());
    }

    // Create session record
    let session = Session {
        id: metadata.session_id,
        rover_id: metadata.rover_id.clone(),
        path: session_path.clone(),
        started_at: metadata.started_at,
        ended_at: metadata.ended_at,
        gps_bounds: metadata.gps_bounds.clone(),
        lidar_frames: metadata.lidar_frames,
        camera_frames: metadata.camera_frames,
        status: SessionStatus::Queued,
        map_id: None,
        discovered_at: Utc::now(),
        processed_at: None,
        error: None,
    };

    info!(
        id = %session.id,
        rover = %session.rover_id,
        lidar_frames = session.lidar_frames,
        camera_frames = session.camera_frames,
        "Session queued for processing"
    );

    state.sessions.insert(session.id, session);
    state.save_sessions().await?;

    Ok(())
}

// =============================================================================
// Map Processing
// =============================================================================

/// Process queued sessions into maps
async fn process_queued_sessions(state: SharedState) -> Result<(), MapperError> {
    // Get queued sessions
    let queued: Vec<Session> = {
        let state = state.read().await;
        state
            .sessions
            .values()
            .filter(|s| s.status == SessionStatus::Queued)
            .cloned()
            .collect()
    };

    for session in queued {
        if let Err(e) = process_session(state.clone(), session.id).await {
            error!(id = %session.id, error = %e, "Failed to process session");

            // Mark as failed
            let mut state = state.write().await;
            if let Some(s) = state.sessions.get_mut(&session.id) {
                s.status = SessionStatus::Failed;
                s.error = Some(e.to_string());
            }
            state.save_sessions().await?;
        }
    }

    Ok(())
}

/// Process a single session
async fn process_session(state: SharedState, session_id: Uuid) -> Result<(), MapperError> {
    // Mark as processing
    {
        let mut state = state.write().await;
        if let Some(session) = state.sessions.get_mut(&session_id) {
            session.status = SessionStatus::Processing;
        }
        state.save_sessions().await?;
    }

    let session = {
        let state = state.read().await;
        state.sessions.get(&session_id).cloned()
    };

    let session = session.ok_or_else(|| MapperError::ProcessingFailed("Session not found".into()))?;

    info!(id = %session_id, path = %session.path.display(), "Starting session processing");

    // Check if this session overlaps with an existing map
    let existing_map_id = {
        let state = state.read().await;
        session
            .gps_bounds
            .as_ref()
            .and_then(|bounds| state.find_overlapping_map(bounds))
            .map(|m| m.id)
    };

    let map_id = if let Some(map_id) = existing_map_id {
        // Merge into existing map
        info!(session = %session_id, map = %map_id, "Merging session into existing map");
        merge_session_into_map(state.clone(), &session, map_id).await?;
        map_id
    } else {
        // Create new map
        info!(session = %session_id, "Creating new map from session");
        create_map_from_session(state.clone(), &session).await?
    };

    // Mark session as processed
    {
        let mut state = state.write().await;
        if let Some(s) = state.sessions.get_mut(&session_id) {
            s.status = SessionStatus::Processed;
            s.map_id = Some(map_id);
            s.processed_at = Some(Utc::now());
        }
        state.save_sessions().await?;
        state.save_index().await?;
    }

    info!(session = %session_id, map = %map_id, "Session processing complete");
    Ok(())
}

/// Create a new map from a session
async fn create_map_from_session(
    state: SharedState,
    session: &Session,
) -> Result<Uuid, MapperError> {
    let map_id = Uuid::new_v4();
    let now = Utc::now();

    // Generate map name from GPS center or rover ID + date
    let name = if let Some(ref bounds) = session.gps_bounds {
        let (lat, lon) = bounds.center();
        format!("map_{:.4}_{:.4}", lat, lon)
    } else {
        format!("map_{}_{}", session.rover_id, now.format("%Y%m%d"))
    };

    let bounds = session.gps_bounds.clone().unwrap_or_default();

    // Create map directory
    let maps_dir = {
        let state = state.read().await;
        state.maps_dir.clone()
    };
    let map_dir = maps_dir.join(&name);
    tokio::fs::create_dir_all(&map_dir).await?;

    // Run splatting pipeline (placeholder: just copy/reference source data)
    // TODO: Invoke actual Gaussian splatting when implemented
    let splat_path = run_splat_pipeline(&session.path, &map_dir).await?;

    let manifest = MapManifest {
        id: map_id,
        name: name.clone(),
        description: Some(format!("Generated from session {}", session.id)),
        bounds,
        version: 1,
        created_at: now,
        updated_at: now,
        assets: MapAssets {
            splat: splat_path,
            pointcloud: None,
            mesh: None,
            thumbnail: None,
        },
        sessions: vec![MapSessionRef {
            session_id: session.id,
            rover_id: session.rover_id.clone(),
            date: session.started_at,
        }],
        stats: MapStats {
            total_points: session.lidar_frames as u64 * 200_000, // Estimate
            total_splats: 0,
            coverage_pct: 0.0,
        },
    };

    // Save manifest and update state
    {
        let mut state = state.write().await;
        state.save_manifest(&manifest).await?;
        state.maps.insert(map_id, manifest);
    }

    info!(id = %map_id, name = %name, "Created new map");
    Ok(map_id)
}

/// Merge a session into an existing map
async fn merge_session_into_map(
    state: SharedState,
    session: &Session,
    map_id: Uuid,
) -> Result<(), MapperError> {
    let (manifest_clone, new_version) = {
        let mut state = state.write().await;

        let map = state
            .maps
            .get_mut(&map_id)
            .ok_or_else(|| MapperError::ProcessingFailed("Map not found".into()))?;

        // Expand bounds
        if let Some(ref bounds) = session.gps_bounds {
            map.bounds.expand(bounds);
        }

        // Add session reference
        map.sessions.push(MapSessionRef {
            session_id: session.id,
            rover_id: session.rover_id.clone(),
            date: session.started_at,
        });

        // Update stats
        map.stats.total_points += session.lidar_frames as u64 * 200_000;
        map.version += 1;
        map.updated_at = Utc::now();

        // TODO: Actually re-run splatting with merged data
        // For now, just update metadata

        let new_version = map.version;
        (map.clone(), new_version)
    };

    // Save manifest outside of the mutable borrow
    {
        let state = state.read().await;
        state.save_manifest(&manifest_clone).await?;
    }

    info!(map = %map_id, version = new_version, "Updated map with new session");
    Ok(())
}

/// Job request for the splat-worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplatJob {
    pub id: Uuid,
    pub session_path: PathBuf,
    pub output_path: PathBuf,
    pub config: SplatConfig,
}

/// Configuration for Gaussian splatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplatConfig {
    pub iterations: u32,
    pub resolution: u32,
}

impl Default for SplatConfig {
    fn default() -> Self {
        Self {
            iterations: 30000,
            resolution: 1024,
        }
    }
}

/// Result from splat-worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplatResult {
    pub job_id: Uuid,
    pub status: String,
    pub completed_at: Option<String>,
    pub stats: Option<serde_json::Value>,
}

/// Queue a splatting job for the splat-worker.
///
/// Creates a job.json file in the jobs directory that the splat-worker
/// will pick up and process.
async fn queue_splat_job(
    session_path: &Path,
    map_dir: &Path,
    jobs_dir: &Path,
) -> Result<Uuid, MapperError> {
    // Check if we have the required data
    let lidar_dir = session_path.join("lidar");
    let camera_dir = session_path.join("camera");

    if !lidar_dir.exists() && !camera_dir.exists() {
        warn!(
            session = %session_path.display(),
            "No LiDAR or camera data, skipping splatting"
        );
        return Err(MapperError::ProcessingFailed(
            "No LiDAR or camera data available".into(),
        ));
    }

    // Create job
    let job_id = Uuid::new_v4();
    let job = SplatJob {
        id: job_id,
        session_path: session_path.to_path_buf(),
        output_path: map_dir.to_path_buf(),
        config: SplatConfig::default(),
    };

    // Ensure jobs directory exists
    tokio::fs::create_dir_all(jobs_dir).await?;

    // Write job file
    let job_path = jobs_dir.join(format!("{}.json", job_id));
    let json = serde_json::to_string_pretty(&job)?;
    tokio::fs::write(&job_path, json).await?;

    info!(
        job_id = %job_id,
        session = %session_path.display(),
        output = %map_dir.display(),
        "Queued splatting job"
    );

    Ok(job_id)
}

/// Check if a splat job has completed
async fn check_splat_result(map_dir: &Path) -> Option<SplatResult> {
    let result_path = map_dir.join("result.json");
    
    if !result_path.exists() {
        return None;
    }

    match tokio::fs::read_to_string(&result_path).await {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(result) => Some(result),
            Err(e) => {
                warn!(error = %e, "Failed to parse splat result");
                None
            }
        },
        Err(e) => {
            warn!(error = %e, "Failed to read splat result");
            None
        }
    }
}

/// Run the Gaussian splatting pipeline.
///
/// Queues a job for the splat-worker and optionally waits for completion.
async fn run_splat_pipeline(
    session_path: &Path,
    map_dir: &Path,
) -> Result<Option<String>, MapperError> {
    // Get jobs directory from environment or use default
    let jobs_dir = std::env::var("JOBS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/jobs"));

    // Check if we have the required data
    let lidar_dir = session_path.join("lidar");
    let camera_dir = session_path.join("camera");

    if !lidar_dir.exists() && !camera_dir.exists() {
        warn!(
            session = %session_path.display(),
            "No LiDAR or camera data, skipping splatting"
        );
        return Ok(None);
    }

    // Queue the job
    let job_id = queue_splat_job(session_path, map_dir, &jobs_dir).await?;
    
    // For now, we don't wait for completion (async processing)
    // The splat-worker will write result.json when done
    // A periodic check or webhook could update the map manifest
    
    info!(
        job_id = %job_id,
        "Splatting job queued, will be processed asynchronously"
    );

    // Check if there's already a result (from a previous run)
    if let Some(result) = check_splat_result(map_dir).await {
        if result.status == "success" || result.status == "point_cloud_only" {
            info!(job_id = %result.job_id, status = %result.status, "Found existing splat result");
            return Ok(Some("splat.ply".to_string()));
        }
    }

    // Return None for now (processing is async)
    // The map manifest will be updated when we detect the result
    Ok(None)
}

// =============================================================================
// File Watcher
// =============================================================================

async fn watch_sessions(
    state: SharedState,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), MapperError> {
    let sessions_dir = {
        let state = state.read().await;
        state.sessions_dir.clone()
    };

    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_secs(5)),
    )?;

    watcher.watch(&sessions_dir, RecursiveMode::Recursive)?;
    info!(path = %sessions_dir.display(), "Watching for new sessions");

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                // Look for new metadata.json files being created
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in event.paths {
                        if path.file_name().map(|n| n == "metadata.json").unwrap_or(false) {
                            if let Some(session_dir) = path.parent() {
                                // Debounce: wait a bit for the session to finish writing
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                let _ = process_new_session(state.clone(), session_dir.to_path_buf()).await;
                            }
                        }
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Shutting down file watcher");
                break;
            }
        }
    }

    Ok(())
}

// =============================================================================
// Splat Job Monitoring
// =============================================================================

/// Check for completed splat jobs and update map manifests.
async fn check_completed_splat_jobs(state: SharedState) -> Result<(), MapperError> {
    let maps_to_check: Vec<(Uuid, PathBuf)> = {
        let state = state.read().await;
        state
            .maps
            .values()
            .filter(|m| m.assets.splat.is_none()) // Only check maps without splats
            .map(|m| (m.id, state.maps_dir.join(&m.name)))
            .collect()
    };

    for (map_id, map_dir) in maps_to_check {
        if let Some(result) = check_splat_result(&map_dir).await {
            if result.status == "success" || result.status == "point_cloud_only" {
                // Update map manifest with splat asset
                let splat_path = map_dir.join("splat.ply");
                if splat_path.exists() {
                    info!(
                        map_id = %map_id,
                        status = %result.status,
                        "Splat job completed, updating manifest"
                    );

                    let mut state = state.write().await;
                    if let Some(map) = state.maps.get_mut(&map_id) {
                        map.assets.splat = Some("splat.ply".to_string());
                        map.updated_at = Utc::now();

                        // Update stats if available
                        if let Some(stats) = &result.stats {
                            if let Some(points) = stats.get("output_points").and_then(|v| v.as_u64()) {
                                map.stats.total_points = points;
                            }
                            if let Some(gaussians) = stats.get("output_gaussians").and_then(|v| v.as_u64()) {
                                map.stats.total_splats = gaussians;
                            }
                        }

                        // Save updated manifest
                        let manifest = map.clone();
                        if let Err(e) = state.save_manifest(&manifest).await {
                            warn!(error = %e, "Failed to save updated manifest");
                        }
                        if let Err(e) = state.save_index().await {
                            warn!(error = %e, "Failed to save index");
                        }
                    }
                }
            } else if result.status == "failed" {
                warn!(
                    map_id = %map_id,
                    status = %result.status,
                    "Splat job failed"
                );
            }
        }
    }

    Ok(())
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mapper=info".into()),
        )
        .init();

    let sessions_dir = PathBuf::from(
        std::env::var("SESSIONS_DIR").unwrap_or_else(|_| "/data/sessions".to_string()),
    );
    let maps_dir =
        PathBuf::from(std::env::var("MAPS_DIR").unwrap_or_else(|_| "/data/maps".to_string()));

    // Ensure directories exist
    tokio::fs::create_dir_all(&sessions_dir).await?;
    tokio::fs::create_dir_all(&maps_dir).await?;

    info!(
        sessions = %sessions_dir.display(),
        maps = %maps_dir.display(),
        "Starting mapper service"
    );

    let state = Arc::new(RwLock::new(MapperState::new(
        sessions_dir.clone(),
        maps_dir,
    )));

    // Load existing state
    {
        let mut s = state.write().await;
        if let Err(e) = s.load().await {
            warn!(error = %e, "Failed to load state, starting fresh");
        }
    }

    // Scan for existing sessions that need processing
    info!("Scanning for existing sessions...");
    let existing = scan_sessions(&sessions_dir);
    for session_path in existing {
        let _ = process_new_session(state.clone(), session_path).await;
    }

    // Process any queued sessions
    if let Err(e) = process_queued_sessions(state.clone()).await {
        error!(error = %e, "Error processing queued sessions");
    }

    // Set up shutdown channel
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

    // Spawn file watcher
    let watcher_state = state.clone();
    let watcher_handle = tokio::spawn(async move {
        if let Err(e) = watch_sessions(watcher_state, shutdown_rx).await {
            error!(error = %e, "File watcher error");
        }
    });

    // Spawn periodic processing task
    let processor_state = state.clone();
    let processor_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = process_queued_sessions(processor_state.clone()).await {
                error!(error = %e, "Error in periodic processing");
            }
        }
    });

    // Spawn task to check for completed splat jobs
    let splat_checker_state = state.clone();
    let splat_checker_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = check_completed_splat_jobs(splat_checker_state.clone()).await {
                error!(error = %e, "Error checking splat jobs");
            }
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");

    // Clean shutdown
    let _ = shutdown_tx.send(()).await;
    watcher_handle.abort();
    processor_handle.abort();
    splat_checker_handle.abort();

    info!("Mapper service stopped");
    Ok(())
}
