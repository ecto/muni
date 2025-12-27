//! Map API Service
//!
//! Serves processed maps to the operator app and other clients.
//!
//! Endpoints:
//! - GET /maps - List all maps
//! - GET /maps/:id - Get map manifest
//! - GET /maps/:id/splat.ply - Download splat file
//! - GET /maps/:id/pointcloud.laz - Download point cloud
//! - GET /maps/:id/thumbnail.jpg - Get map thumbnail
//! - GET /sessions - List all sessions
//! - GET /sessions/:id - Get session details
//! - GET /health - Health check

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use uuid::Uuid;

// =============================================================================
// Types (shared with mapper, ideally in a common crate)
// =============================================================================

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Json(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpsBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct MapAssets {
    pub splat: Option<String>,
    pub pointcloud: Option<String>,
    pub mesh: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSessionRef {
    pub session_id: Uuid,
    pub rover_id: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MapStats {
    pub total_points: u64,
    pub total_splats: u64,
    pub coverage_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapIndex {
    pub maps: Vec<MapIndexEntry>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapIndexEntry {
    pub id: Uuid,
    pub name: String,
    pub bounds: GpsBounds,
    pub version: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Pending,
    Queued,
    Processing,
    Processed,
    Failed,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

// =============================================================================
// API Response Types
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MapListResponse {
    maps: Vec<MapSummary>,
    total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MapSummary {
    id: Uuid,
    name: String,
    bounds: GpsBounds,
    version: u32,
    updated_at: DateTime<Utc>,
    session_count: usize,
    has_splat: bool,
    thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionListResponse {
    sessions: Vec<Session>,
    total: usize,
}

// =============================================================================
// State
// =============================================================================

struct AppState {
    maps_dir: PathBuf,
    maps: RwLock<HashMap<Uuid, MapManifest>>,
    sessions: RwLock<HashMap<Uuid, Session>>,
}

impl AppState {
    fn new(maps_dir: PathBuf) -> Self {
        Self {
            maps_dir,
            maps: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    async fn reload(&self) -> Result<(), ApiError> {
        // Load map index
        let index_path = self.maps_dir.join("index.json");
        if index_path.exists() {
            let json = tokio::fs::read_to_string(&index_path).await?;
            let index: MapIndex = serde_json::from_str(&json)?;

            let mut maps = self.maps.write().await;
            maps.clear();

            for entry in index.maps {
                let manifest_path = self.maps_dir.join(&entry.name).join("manifest.json");
                if manifest_path.exists() {
                    let json = tokio::fs::read_to_string(&manifest_path).await?;
                    let manifest: MapManifest = serde_json::from_str(&json)?;
                    maps.insert(manifest.id, manifest);
                }
            }

            info!("Loaded {} maps", maps.len());
        }

        // Load sessions
        let sessions_path = self.maps_dir.join("sessions.json");
        if sessions_path.exists() {
            let json = tokio::fs::read_to_string(&sessions_path).await?;
            let sessions_vec: Vec<Session> = serde_json::from_str(&json)?;

            let mut sessions = self.sessions.write().await;
            sessions.clear();

            for session in sessions_vec {
                sessions.insert(session.id, session);
            }

            info!("Loaded {} sessions", sessions.len());
        }

        Ok(())
    }

    fn get_map_dir(&self, map_name: &str) -> PathBuf {
        self.maps_dir.join(map_name)
    }
}

type SharedState = Arc<AppState>;

// =============================================================================
// Handlers
// =============================================================================

async fn health(State(state): State<SharedState>) -> impl IntoResponse {
    let map_count = state.maps.read().await.len();
    let session_count = state.sessions.read().await.len();

    Json(serde_json::json!({
        "status": "ok",
        "maps": map_count,
        "sessions": session_count
    }))
}

async fn list_maps(State(state): State<SharedState>) -> Result<impl IntoResponse, ApiError> {
    // Reload to get latest data
    if let Err(e) = state.reload().await {
        warn!(error = %e, "Failed to reload state");
    }

    let maps = state.maps.read().await;

    let summaries: Vec<MapSummary> = maps
        .values()
        .map(|m| MapSummary {
            id: m.id,
            name: m.name.clone(),
            bounds: m.bounds.clone(),
            version: m.version,
            updated_at: m.updated_at,
            session_count: m.sessions.len(),
            has_splat: m.assets.splat.is_some(),
            thumbnail_url: m.assets.thumbnail.as_ref().map(|_| {
                format!("/maps/{}/thumbnail.jpg", m.id)
            }),
        })
        .collect();

    let total = summaries.len();

    Ok(Json(MapListResponse {
        maps: summaries,
        total,
    }))
}

async fn get_map(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let maps = state.maps.read().await;

    let map = maps
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Map {} not found", id)))?;

    Ok(Json(map.clone()))
}

async fn get_map_asset(
    State(state): State<SharedState>,
    Path((id, asset)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let maps = state.maps.read().await;

    let map = maps
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Map {} not found", id)))?;

    // Validate asset exists in manifest
    let asset_file = match asset.as_str() {
        "splat.ply" => map.assets.splat.as_ref(),
        "pointcloud.laz" => map.assets.pointcloud.as_ref(),
        "mesh.glb" => map.assets.mesh.as_ref(),
        "thumbnail.jpg" => map.assets.thumbnail.as_ref(),
        _ => return Err(ApiError::NotFound(format!("Unknown asset: {}", asset))),
    };

    let asset_file = asset_file
        .ok_or_else(|| ApiError::NotFound(format!("Asset {} not available", asset)))?;

    let asset_path = state.get_map_dir(&map.name).join(asset_file);

    if !asset_path.exists() {
        return Err(ApiError::NotFound(format!("Asset file not found: {}", asset)));
    }

    let contents = tokio::fs::read(&asset_path).await?;

    // Determine content type
    let content_type = match asset.as_str() {
        "splat.ply" => "application/octet-stream",
        "pointcloud.laz" => "application/octet-stream",
        "mesh.glb" => "model/gltf-binary",
        "thumbnail.jpg" => "image/jpeg",
        _ => "application/octet-stream",
    };

    Ok((
        [(header::CONTENT_TYPE, content_type)],
        contents,
    ))
}

async fn list_sessions(State(state): State<SharedState>) -> Result<impl IntoResponse, ApiError> {
    // Reload to get latest data
    if let Err(e) = state.reload().await {
        warn!(error = %e, "Failed to reload state");
    }

    let sessions = state.sessions.read().await;

    let mut sessions_vec: Vec<Session> = sessions.values().cloned().collect();
    sessions_vec.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    let total = sessions_vec.len();

    Ok(Json(SessionListResponse {
        sessions: sessions_vec,
        total,
    }))
}

async fn get_session(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let sessions = state.sessions.read().await;

    let session = sessions
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Session {} not found", id)))?;

    Ok(Json(session.clone()))
}

async fn get_map_sessions(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let maps = state.maps.read().await;

    let map = maps
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Map {} not found", id)))?;

    Ok(Json(map.sessions.clone()))
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
                .unwrap_or_else(|_| "map_api=info".into()),
        )
        .init();

    let maps_dir =
        PathBuf::from(std::env::var("MAPS_DIR").unwrap_or_else(|_| "/data/maps".to_string()));

    // Ensure directory exists
    tokio::fs::create_dir_all(&maps_dir).await?;

    info!(maps_dir = %maps_dir.display(), "Starting map-api service");

    let state = Arc::new(AppState::new(maps_dir));

    // Initial load
    if let Err(e) = state.reload().await {
        warn!(error = %e, "Failed to load initial state");
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/maps", get(list_maps))
        .route("/maps/{id}", get(get_map))
        .route("/maps/{id}/sessions", get(get_map_sessions))
        .route("/maps/{id}/{asset}", get(get_map_asset))
        .route("/sessions", get(list_sessions))
        .route("/sessions/{id}", get(get_session))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4870);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Map API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
