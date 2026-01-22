//! Mapper Service
//!
//! Watches for new sessions uploaded by rovers and processes them into maps.
//!
//! Responsibilities:
//! - Monitor sessions directory for new uploads (both metadata.json and .rrd files)
//! - Extract data from .rrd files (poses, LiDAR, camera, GPS)
//! - Parse session metadata and validate completeness
//! - Queue sessions for processing
//! - Run Gaussian splatting pipeline (or invoke external processor)
//! - Update map registry with results
//! - Merge new sessions into existing maps when regions overlap

use mapper::rrd_extractor;

use axum::{
    body::Body,
    extract::Path as AxumPath,
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use tokio_util::io::ReaderStream;
use chrono::Utc;
use depot_types::{
    GpsBounds, MapAssets, MapIndex, MapIndexEntry, MapManifest, MapSessionRef, MapStats, Session,
    SessionMetadata, SessionStatus,
};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
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
// Local Types (not shared with other services)
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
    #[error("Extraction error: {0}")]
    Extraction(#[from] rrd_extractor::ExtractionError),
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

/// Minimum requirements for a session to be processable
const MIN_LIDAR_FRAMES: u32 = 10;
const MIN_POSE_SAMPLES: u32 = 5;

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

    // Validate LiDAR data quality
    if metadata.lidar_frames > 0 {
        // Check LiDAR directory exists
        let lidar_dir = session_path.join("lidar");
        if !lidar_dir.exists() {
            return Err(MapperError::IncompleteSession(
                "LiDAR frames indicated but lidar/ directory missing".into(),
            ));
        }

        // Check poses file exists for LiDAR alignment
        let poses_path = session_path.join("poses.csv");
        if !poses_path.exists() {
            warn!(
                session = %metadata.session_id,
                "No poses.csv found, LiDAR frames will use identity poses"
            );
        }

        // Warn if not enough data for quality map
        if metadata.lidar_frames < MIN_LIDAR_FRAMES {
            warn!(
                session = %metadata.session_id,
                lidar_frames = metadata.lidar_frames,
                "Low LiDAR frame count may result in sparse map"
            );
        }

        if metadata.pose_samples < MIN_POSE_SAMPLES && metadata.pose_samples > 0 {
            warn!(
                session = %metadata.session_id,
                pose_samples = metadata.pose_samples,
                "Low pose sample count may result in poor LiDAR alignment"
            );
        }
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

/// Scan sessions directory for .rrd files that haven't been extracted yet.
///
/// Returns paths to .rrd files that need processing.
fn scan_rrd_sessions(sessions_dir: &Path) -> Vec<PathBuf> {
    let mut rrd_files = Vec::new();

    for entry in WalkDir::new(sessions_dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip partial uploads
        if path.to_string_lossy().contains(".partial") {
            continue;
        }

        // Handle flat .rrd files (e.g., sessions/frog-0-2024-01-15.rrd)
        // Skip session.rrd files here - they're handled via the directory check below
        if path.is_file()
            && path.extension().map(|e| e == "rrd").unwrap_or(false)
            && path.file_name().map(|n| n != "session.rrd").unwrap_or(true)
        {
            let extracted_dir = path.with_extension("extracted");
            let skipped_marker = path.with_extension("skipped");
            if !extracted_dir.exists() && !skipped_marker.exists() {
                rrd_files.push(path.to_path_buf());
            }
        }

        // Handle directory-based sessions (e.g., sessions/2024-01-15T12-00-00/session.rrd)
        if path.is_dir() {
            let rrd_path = path.join("session.rrd");
            let extracted_dir = path.join("extracted");
            let skipped_marker = path.join("skipped");
            if rrd_path.exists() && !extracted_dir.exists() && !skipped_marker.exists() {
                rrd_files.push(rrd_path);
            }
        }
    }

    debug!(count = rrd_files.len(), "Found unprocessed RRD files");
    rrd_files
}

/// Process an RRD session file by extracting its contents.
///
/// Extracts poses, LiDAR, camera, and GPS data from the .rrd file
/// and writes them to an extracted/ directory alongside it.
async fn process_rrd_session(
    state: SharedState,
    rrd_path: PathBuf,
) -> Result<(), MapperError> {
    info!(path = %rrd_path.display(), "Processing RRD session");

    // Determine output directory and skipped marker path
    let (extract_dir, skipped_marker) = if rrd_path.file_name().map(|n| n == "session.rrd").unwrap_or(false) {
        // Directory-based: /sessions/timestamp/session.rrd -> /sessions/timestamp/extracted/
        let parent = rrd_path.parent().unwrap();
        (parent.join("extracted"), parent.join("skipped"))
    } else {
        // Flat file: /sessions/name.rrd -> /sessions/name.extracted/ and /sessions/name.skipped
        (rrd_path.with_extension("extracted"), rrd_path.with_extension("skipped"))
    };

    // Check if already processed (either extracted or marked as skipped)
    if extract_dir.exists() {
        debug!(path = %rrd_path.display(), "Already extracted, skipping");
        return Ok(());
    }
    if skipped_marker.exists() {
        debug!(path = %rrd_path.display(), "Previously skipped, not reprocessing");
        return Ok(());
    }

    // Check if file is still being written (wait for stable mtime)
    let metadata = tokio::fs::metadata(&rrd_path).await?;
    let mtime = metadata.modified()?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let new_metadata = tokio::fs::metadata(&rrd_path).await?;
    if new_metadata.modified()? != mtime {
        debug!(path = %rrd_path.display(), "File still being written, skipping");
        return Ok(());
    }

    // Extract data from RRD file
    let rrd_path_clone = rrd_path.clone();
    let extract_dir_clone = extract_dir.clone();
    let skipped_marker_clone = skipped_marker.clone();
    let extraction_result = tokio::task::spawn_blocking(move || {
        let result = rrd_extractor::extract_from_rrd(&rrd_path_clone)?;

        // Validate we have LiDAR data (required for mapping)
        if result.lidar_frames.is_empty() {
            warn!(path = %rrd_path_clone.display(), "No LiDAR data in RRD, marking as skipped");
            // Create marker file so we don't reprocess this file
            if let Err(e) = std::fs::write(&skipped_marker_clone, "no_lidar_data") {
                warn!(error = %e, "Failed to write skipped marker");
            }
            return Err(MapperError::IncompleteSession("No LiDAR data".into()));
        }

        // Write extracted data
        rrd_extractor::write_extracted_data(&result, &extract_dir_clone)
            .map_err(|e| MapperError::ProcessingFailed(e.to_string()))?;

        Ok::<_, MapperError>(result)
    })
    .await
    .map_err(|e| MapperError::ProcessingFailed(format!("Task join error: {}", e)))??;

    info!(
        path = %rrd_path.display(),
        poses = extraction_result.poses.len(),
        lidar_frames = extraction_result.lidar_frames.len(),
        camera_frames = extraction_result.camera_frames.len(),
        "RRD extraction complete"
    );

    // Queue splat job for the extracted data
    let maps_dir = {
        let state = state.read().await;
        state.maps_dir.clone()
    };

    let jobs_dir = std::env::var("JOBS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/jobs"));

    // Generate map name from RRD filename or GPS bounds
    let map_name = if let Some(bounds) = &extraction_result.gps_bounds {
        let (lat, lon) = ((bounds.min_lat + bounds.max_lat) / 2.0, (bounds.min_lon + bounds.max_lon) / 2.0);
        format!("map_{:.4}_{:.4}", lat, lon)
    } else {
        let stem = rrd_path.file_stem().and_then(|s| s.to_str()).unwrap_or("session");
        format!("map_{}", stem)
    };

    let map_dir = maps_dir.join(&map_name);
    tokio::fs::create_dir_all(&map_dir).await?;

    // Queue splat job
    if let Err(e) = queue_splat_job(&extract_dir, &map_dir, &jobs_dir).await {
        warn!(error = %e, "Failed to queue splat job, but extraction succeeded");
    }

    Ok(())
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
        pose_samples: metadata.pose_samples,
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
        pose_samples = session.pose_samples,
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
                // Look for new metadata.json or .rrd files being created
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in event.paths {
                        // Handle metadata.json (legacy format)
                        if path.file_name().map(|n| n == "metadata.json").unwrap_or(false) {
                            if let Some(session_dir) = path.parent() {
                                // Debounce: wait a bit for the session to finish writing
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                let _ = process_new_session(state.clone(), session_dir.to_path_buf()).await;
                            }
                        }

                        // Handle .rrd files (new format)
                        if path.extension().map(|e| e == "rrd").unwrap_or(false) {
                            // Skip partial uploads
                            if path.to_string_lossy().contains(".partial") {
                                continue;
                            }

                            // Debounce: wait for upload to complete
                            tokio::time::sleep(Duration::from_secs(5)).await;

                            // Check if already extracted
                            let extract_dir = if path.file_name().map(|n| n == "session.rrd").unwrap_or(false) {
                                path.parent().map(|p| p.join("extracted"))
                            } else {
                                Some(path.with_extension("extracted"))
                            };

                            if let Some(ref dir) = extract_dir {
                                if !dir.exists() {
                                    let _ = process_rrd_session(state.clone(), path.clone()).await;
                                }
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

    // Spawn health server
    let health_state = state.clone();
    let sessions_dir_for_api = sessions_dir.clone();
    let health_handle = tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(|| async { Json(serde_json::json!({"status": "ok"})) }))
            .route("/status", get({
                let state = health_state.clone();
                let sessions_dir_status = sessions_dir_for_api.clone();
                move || {
                    let state = state.clone();
                    let sessions_dir = sessions_dir_status.clone();
                    async move {
                        let s = state.read().await;

                        // Count pending extractions (RRD files without .extracted dir)
                        let mut pending_extractions = 0;
                        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                let name = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");

                                if name.ends_with(".rrd") && !name.contains(".partial") {
                                    let extracted_dir = path.with_extension("extracted");
                                    if !extracted_dir.exists() {
                                        pending_extractions += 1;
                                    }
                                }
                            }
                        }

                        // Count splat jobs
                        let jobs_dir = std::env::var("JOBS_DIR")
                            .unwrap_or_else(|_| "/data/jobs".to_string());
                        let mut queued_jobs = 0;
                        let mut processing_jobs = 0;
                        let mut completed_jobs = 0;
                        let mut failed_jobs = 0;

                        if let Ok(entries) = std::fs::read_dir(&jobs_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.extension().map(|e| e == "json").unwrap_or(false) {
                                    // Check if there's a corresponding result
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        if let Ok(job) = serde_json::from_str::<serde_json::Value>(&content) {
                                            if let Some(output_path) = job.get("output_path").and_then(|v| v.as_str()) {
                                                let result_path = std::path::Path::new(output_path).join("result.json");
                                                if result_path.exists() {
                                                    if let Ok(result_content) = std::fs::read_to_string(&result_path) {
                                                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&result_content) {
                                                            match result.get("status").and_then(|v| v.as_str()) {
                                                                Some("success") | Some("point_cloud_only") => completed_jobs += 1,
                                                                Some("failed") => failed_jobs += 1,
                                                                Some("processing") => processing_jobs += 1,
                                                                _ => queued_jobs += 1,
                                                            }
                                                        } else {
                                                            queued_jobs += 1;
                                                        }
                                                    } else {
                                                        queued_jobs += 1;
                                                    }
                                                } else {
                                                    queued_jobs += 1;
                                                }
                                            } else {
                                                queued_jobs += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Json(serde_json::json!({
                            "status": "ok",
                            "sessions": s.sessions.len(),
                            "maps": s.maps.len(),
                            "pending_extractions": pending_extractions,
                            "splat_queue": {
                                "queued": queued_jobs,
                                "processing": processing_jobs,
                                "completed": completed_jobs,
                                "failed": failed_jobs,
                            }
                        }))
                    }
                }
            }))
            .route("/sessions", get({
                let sessions_dir = sessions_dir_for_api.clone();
                move || async move {
                    // Scan for all .rrd files
                    let mut rrd_files = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();

                            // Skip partial uploads
                            if name.contains(".partial") {
                                continue;
                            }

                            if name.ends_with(".rrd") {
                                // Flat .rrd file
                                if let Ok(meta) = std::fs::metadata(&path) {
                                    let mut session_info = serde_json::json!({
                                        "name": name,
                                        "size": meta.len(),
                                        "modified": meta.modified()
                                            .ok()
                                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                            .map(|d| d.as_secs()),
                                    });

                                    // Try to parse rover_id and timestamp from filename
                                    // Format: rover-id_timestamp.rrd (e.g., frog-0_1768854113.rrd)
                                    let base_name = name.strip_suffix(".rrd").unwrap_or(&name);
                                    if let Some((rover_id, ts_str)) = base_name.rsplit_once('_') {
                                        session_info["rover_id"] = serde_json::json!(rover_id);
                                        if let Ok(ts) = ts_str.parse::<i64>() {
                                            // Timestamps are in nanoseconds, convert to seconds
                                            let duration_ns = meta.modified()
                                                .ok()
                                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                                .map(|d| d.as_nanos() as i64)
                                                .unwrap_or(0) - ts;
                                            if duration_ns > 0 {
                                                session_info["duration_secs"] = serde_json::json!(duration_ns / 1_000_000_000);
                                            }
                                        }
                                    }

                                    // Check for extraction metadata
                                    let extracted_dir = path.with_extension("extracted");
                                    let metadata_path = extracted_dir.join("extraction_metadata.json");
                                    if metadata_path.exists() {
                                        if let Ok(json_str) = std::fs::read_to_string(&metadata_path) {
                                            if let Ok(extraction_meta) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                                session_info["extracted"] = serde_json::json!(true);
                                                if let Some(poses) = extraction_meta.get("pose_count") {
                                                    session_info["pose_count"] = poses.clone();
                                                }
                                                if let Some(lidar) = extraction_meta.get("lidar_frame_count") {
                                                    session_info["lidar_frame_count"] = lidar.clone();
                                                }
                                                if let Some(camera) = extraction_meta.get("camera_frame_count") {
                                                    session_info["camera_frame_count"] = camera.clone();
                                                }
                                                if let Some(rover) = extraction_meta.get("rover_id") {
                                                    session_info["rover_id"] = rover.clone();
                                                }
                                                if let Some(gps) = extraction_meta.get("gps_bounds") {
                                                    if !gps.is_null() {
                                                        session_info["has_gps"] = serde_json::json!(true);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    rrd_files.push(session_info);
                                }
                            } else if path.is_dir() {
                                // Check for session.rrd inside directory
                                let rrd_path = path.join("session.rrd");
                                if rrd_path.exists() {
                                    if let Ok(meta) = std::fs::metadata(&rrd_path) {
                                        let mut session_info = serde_json::json!({
                                            "name": name,
                                            "size": meta.len(),
                                            "modified": meta.modified()
                                                .ok()
                                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                                .map(|d| d.as_secs()),
                                        });

                                        // Check for extraction metadata
                                        let extracted_dir = path.join("extracted");
                                        let metadata_path = extracted_dir.join("extraction_metadata.json");
                                        if metadata_path.exists() {
                                            if let Ok(json_str) = std::fs::read_to_string(&metadata_path) {
                                                if let Ok(extraction_meta) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                                    session_info["extracted"] = serde_json::json!(true);
                                                    if let Some(poses) = extraction_meta.get("pose_count") {
                                                        session_info["pose_count"] = poses.clone();
                                                    }
                                                    if let Some(lidar) = extraction_meta.get("lidar_frame_count") {
                                                        session_info["lidar_frame_count"] = lidar.clone();
                                                    }
                                                    if let Some(camera) = extraction_meta.get("camera_frame_count") {
                                                        session_info["camera_frame_count"] = camera.clone();
                                                    }
                                                    if let Some(rover) = extraction_meta.get("rover_id") {
                                                        session_info["rover_id"] = rover.clone();
                                                    }
                                                    if let Some(gps) = extraction_meta.get("gps_bounds") {
                                                        if !gps.is_null() {
                                                            session_info["has_gps"] = serde_json::json!(true);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        rrd_files.push(session_info);
                                    }
                                }
                            }
                        }
                    }

                    // Sort by modified time, newest first
                    rrd_files.sort_by(|a, b| {
                        let a_time = a.get("modified").and_then(|v| v.as_u64()).unwrap_or(0);
                        let b_time = b.get("modified").and_then(|v| v.as_u64()).unwrap_or(0);
                        b_time.cmp(&a_time)
                    });

                    Json(serde_json::json!({
                        "sessions": rrd_files,
                        "count": rrd_files.len(),
                    }))
                }
            }))
            .route("/sessions/{name}", get({
                let sessions_dir = sessions_dir_for_api;
                move |AxumPath(name): AxumPath<String>| {
                    let sessions_dir = sessions_dir.clone();
                    async move {
                        // Sanitize name - prevent path traversal
                        if name.contains("..") || name.contains('/') || name.contains('\\') {
                            return Err((StatusCode::BAD_REQUEST, "Invalid session name"));
                        }

                        // Strip .rrd suffix if present (for Rerun viewer compatibility)
                        let base_name = name.strip_suffix(".rrd").unwrap_or(&name);

                        // Try flat .rrd file first (e.g., sessions/name.rrd)
                        let flat_rrd_path = sessions_dir.join(format!("{}.rrd", base_name));
                        let file_path = if flat_rrd_path.exists() {
                            flat_rrd_path
                        } else {
                            // Try directory with session.rrd inside (e.g., sessions/name/session.rrd)
                            let dir_path = sessions_dir.join(base_name).join("session.rrd");
                            if dir_path.exists() {
                                dir_path
                            } else {
                                return Err((StatusCode::NOT_FOUND, "Session not found"));
                            }
                        };

                        // Open file and stream it
                        let file = match tokio::fs::File::open(&file_path).await {
                            Ok(f) => f,
                            Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file")),
                        };

                        let metadata = match file.metadata().await {
                            Ok(m) => m,
                            Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read metadata")),
                        };

                        let stream = ReaderStream::new(file);
                        let body = Body::from_stream(stream);

                        // Set filename for download (always use .rrd extension)
                        let filename = format!("{}.rrd", base_name);

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/octet-stream")
                            .header(header::CONTENT_LENGTH, metadata.len())
                            .header(
                                header::CONTENT_DISPOSITION,
                                format!("attachment; filename=\"{}\"", filename),
                            )
                            .body(body)
                            .unwrap())
                    }
                }
            }));

        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4895);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!(port = port, "Health server listening");

        if let Err(e) = axum::serve(
            tokio::net::TcpListener::bind(addr).await.unwrap(),
            app.into_make_service(),
        )
        .await
        {
            error!(error = %e, "Health server error");
        }
    });

    // Spawn initial scan task (runs in background so HTTP server is available immediately)
    let initial_scan_state = state.clone();
    let initial_scan_sessions_dir = sessions_dir.clone();
    let initial_scan_handle = tokio::spawn(async move {
        info!("Starting initial session scan in background...");

        // Scan for existing metadata-based sessions
        let existing = scan_sessions(&initial_scan_sessions_dir);
        for session_path in existing {
            let _ = process_new_session(initial_scan_state.clone(), session_path).await;
        }

        // Scan for .rrd files that need extraction
        info!("Scanning for RRD files...");
        let rrd_files = scan_rrd_sessions(&initial_scan_sessions_dir);
        info!(count = rrd_files.len(), "Found unextracted RRD files");

        for rrd_path in rrd_files {
            if let Err(e) = process_rrd_session(initial_scan_state.clone(), rrd_path.clone()).await {
                warn!(path = %rrd_path.display(), error = %e, "Failed to process RRD session");
            }
        }

        // Process any queued sessions
        if let Err(e) = process_queued_sessions(initial_scan_state.clone()).await {
            error!(error = %e, "Error processing queued sessions");
        }

        info!("Initial session scan complete");
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");

    // Clean shutdown
    let _ = shutdown_tx.send(()).await;
    watcher_handle.abort();
    processor_handle.abort();
    splat_checker_handle.abort();
    health_handle.abort();
    initial_scan_handle.abort();

    info!("Mapper service stopped");
    Ok(())
}
