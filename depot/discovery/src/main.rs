//! Rover Discovery Service
//!
//! A lightweight service that tracks rover presence and provides
//! a WebSocket API for operators to discover available rovers.
//!
//! Endpoints:
//! - POST /register - Rovers register with their info
//! - POST /heartbeat/:id - Rovers send heartbeats with telemetry
//! - GET /rovers - List all known rovers (HTTP fallback)
//! - GET /ws - WebSocket for live rover updates
//! - GET /health - Health check
//! - GET /api/sessions - List all recorded sessions
//! - GET /api/sessions/:rover_id/:session_dir/session.rrd - Serve session file

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;
use tracing::{debug, info, warn};

/// Rover timeout - rovers go offline after this duration without heartbeat
const ROVER_TIMEOUT: Duration = Duration::from_secs(10);

/// Pose in 2D space
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Pose {
    x: f64,
    y: f64,
    theta: f64,
}

/// Rover information stored in registry
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoverInfo {
    id: String,
    name: String,
    address: String,
    video_address: String,
    online: bool,
    battery_voltage: f64,
    last_pose: Pose,
    mode: u8,
    last_seen: u64, // Unix timestamp in milliseconds
    #[serde(skip)]
    last_seen_instant: Instant,
}

/// Registration payload from rovers
#[derive(Debug, Deserialize)]
struct RegisterPayload {
    id: String,
    name: String,
    address: String,
    video_address: Option<String>,
    battery_voltage: Option<f64>,
    mode: Option<u8>,
    pose: Option<Pose>,
}

/// Heartbeat payload from rovers
#[derive(Debug, Deserialize)]
struct HeartbeatPayload {
    battery_voltage: Option<f64>,
    mode: Option<u8>,
    pose: Option<Pose>,
}

/// WebSocket message to operators
#[derive(Debug, Serialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: Vec<RoverInfo>,
}

// =============================================================================
// Session Types (matching recording crate's SessionMetadata)
// =============================================================================

/// GPS bounding box for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GpsSessionBounds {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
}

/// Session metadata (read from metadata.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionMetadata {
    session_id: String,
    rover_id: String,
    started_at: String,
    ended_at: Option<String>,
    #[serde(default)]
    duration_secs: f64,
    gps_bounds: Option<GpsSessionBounds>,
    lidar_frames: u32,
    camera_frames: u32,
    #[serde(default)]
    pose_samples: u32,
    session_file: String,
    /// Added by API: the directory name
    #[serde(default)]
    session_dir: String,
}

/// Query params for sessions list
#[derive(Debug, Deserialize)]
struct SessionsQuery {
    rover_id: Option<String>,
}

/// Response for sessions list
#[derive(Debug, Serialize)]
struct SessionsResponse {
    sessions: Vec<SessionMetadata>,
}

/// Shared application state
struct AppState {
    rovers: RwLock<HashMap<String, RoverInfo>>,
    broadcast_tx: broadcast::Sender<()>,
    /// Base directory for session data (contains rover subdirectories)
    sessions_dir: PathBuf,
}

impl AppState {
    fn new(sessions_dir: PathBuf) -> Self {
        let (broadcast_tx, _) = broadcast::channel(16);
        Self {
            rovers: RwLock::new(HashMap::new()),
            broadcast_tx,
            sessions_dir,
        }
    }

    /// Get current rover list with online status updated
    async fn get_rovers(&self) -> Vec<RoverInfo> {
        let rovers = self.rovers.read().await;
        let now = Instant::now();

        rovers
            .values()
            .map(|r| {
                let mut rover = r.clone();
                rover.online = now.duration_since(r.last_seen_instant) < ROVER_TIMEOUT;
                rover
            })
            .collect()
    }

    /// Notify all WebSocket clients of an update
    fn notify(&self) {
        let _ = self.broadcast_tx.send(());
    }
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "discovery=info".into()),
        )
        .init();

    // Sessions directory (where synced rover data lives)
    let sessions_dir = std::env::var("SESSIONS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/sessions"));

    info!(path = %sessions_dir.display(), "Sessions directory");

    let state = Arc::new(AppState::new(sessions_dir));

    // Spawn background task to check for stale rovers
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            let mut changed = false;
            {
                let rovers = state_clone.rovers.read().await;
                let now = Instant::now();
                for rover in rovers.values() {
                    let was_online = now.duration_since(rover.last_seen_instant) < ROVER_TIMEOUT;
                    // We just check; actual update happens on next get_rovers() call
                    if !was_online && rover.online {
                        changed = true;
                    }
                }
            }

            if changed {
                state_clone.notify();
            }
        }
    });

    let app = Router::new()
        .route("/register", post(register))
        .route("/heartbeat/{id}", post(heartbeat))
        .route("/rovers", get(list_rovers))
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        // Sessions API
        .route("/api/sessions", get(list_sessions))
        .route(
            "/api/sessions/{rover_id}/{session_dir}/session.rrd",
            get(serve_session_rrd),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4860);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Discovery service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Health check endpoint
async fn health(State(state): State<SharedState>) -> impl IntoResponse {
    let count = state.rovers.read().await.len();
    Json(serde_json::json!({
        "status": "ok",
        "rovers": count
    }))
}

/// Register a new rover
async fn register(
    State(state): State<SharedState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let now = Instant::now();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let video_address = payload
        .video_address
        .unwrap_or_else(|| payload.address.replace(":4850", ":4851"));

    let rover = RoverInfo {
        id: payload.id.clone(),
        name: payload.name.clone(),
        address: payload.address.clone(),
        video_address,
        online: true,
        battery_voltage: payload.battery_voltage.unwrap_or(0.0),
        last_pose: payload.pose.unwrap_or_default(),
        mode: payload.mode.unwrap_or(0),
        last_seen: now_ms,
        last_seen_instant: now,
    };

    let is_new = {
        let mut rovers = state.rovers.write().await;
        let is_new = !rovers.contains_key(&payload.id);
        rovers.insert(payload.id.clone(), rover);
        is_new
    };

    info!(
        id = %payload.id,
        name = %payload.name,
        address = %payload.address,
        new = is_new,
        "Rover registered"
    );

    state.notify();

    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

/// Heartbeat from a rover
async fn heartbeat(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<HeartbeatPayload>,
) -> impl IntoResponse {
    let now = Instant::now();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut rovers = state.rovers.write().await;

    if let Some(rover) = rovers.get_mut(&id) {
        rover.last_seen = now_ms;
        rover.last_seen_instant = now;
        rover.online = true;

        if let Some(v) = payload.battery_voltage {
            rover.battery_voltage = v;
        }
        if let Some(m) = payload.mode {
            rover.mode = m;
        }
        if let Some(p) = payload.pose {
            rover.last_pose = p;
        }

        drop(rovers);
        state.notify();

        (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
    } else {
        warn!(id = %id, "Heartbeat for unknown rover");
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Rover not found" })),
        )
    }
}

/// List all rovers (HTTP fallback)
async fn list_rovers(State(state): State<SharedState>) -> impl IntoResponse {
    let rovers = state.get_rovers().await;
    Json(rovers)
}

/// WebSocket handler for live updates
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to updates
    let mut rx = state.broadcast_tx.subscribe();

    // Send initial rover list
    let rovers = state.get_rovers().await;
    let msg = WsMessage {
        msg_type: "rovers".to_string(),
        data: rovers,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    info!("Operator connected via WebSocket");

    loop {
        tokio::select! {
            // Handle broadcast updates
            Ok(()) = rx.recv() => {
                let rovers = state.get_rovers().await;
                let msg = WsMessage {
                    msg_type: "rovers".to_string(),
                    data: rovers,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Handle incoming messages (for ping/pong, etc.)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("Operator disconnected");
}

// =============================================================================
// Sessions API
// =============================================================================

/// List all recorded sessions
///
/// Scans the sessions directory for all rovers and returns session metadata.
/// Sessions are read from metadata.json files in each session directory.
async fn list_sessions(
    State(state): State<SharedState>,
    Query(query): Query<SessionsQuery>,
) -> impl IntoResponse {
    let mut sessions: Vec<SessionMetadata> = Vec::new();

    // Check if sessions directory exists
    if !state.sessions_dir.exists() {
        debug!(
            path = %state.sessions_dir.display(),
            "Sessions directory does not exist"
        );
        return Json(SessionsResponse { sessions });
    }

    // Iterate over rover directories
    let rover_dirs = match std::fs::read_dir(&state.sessions_dir) {
        Ok(dirs) => dirs,
        Err(e) => {
            warn!(error = %e, "Failed to read sessions directory");
            return Json(SessionsResponse { sessions });
        }
    };

    for rover_entry in rover_dirs.filter_map(|e| e.ok()) {
        let rover_path = rover_entry.path();
        if !rover_path.is_dir() {
            continue;
        }

        let rover_id = rover_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Filter by rover_id if specified
        if let Some(ref filter_id) = query.rover_id {
            if &rover_id != filter_id {
                continue;
            }
        }

        // Look for sessions directory under rover (synced structure: /data/sessions/{rover_id}/sessions/...)
        let sessions_path = rover_path.join("sessions");
        let search_path = if sessions_path.exists() {
            sessions_path
        } else {
            rover_path.clone()
        };

        // Iterate over session directories
        let session_dirs = match std::fs::read_dir(&search_path) {
            Ok(dirs) => dirs,
            Err(_) => continue,
        };

        for session_entry in session_dirs.filter_map(|e| e.ok()) {
            let session_path = session_entry.path();
            if !session_path.is_dir() {
                continue;
            }

            let session_dir = session_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Read metadata.json
            let metadata_path = session_path.join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }

            match std::fs::read_to_string(&metadata_path) {
                Ok(content) => match serde_json::from_str::<SessionMetadata>(&content) {
                    Ok(mut metadata) => {
                        metadata.session_dir = session_dir;
                        sessions.push(metadata);
                    }
                    Err(e) => {
                        debug!(
                            path = %metadata_path.display(),
                            error = %e,
                            "Failed to parse session metadata"
                        );
                    }
                },
                Err(e) => {
                    debug!(
                        path = %metadata_path.display(),
                        error = %e,
                        "Failed to read session metadata"
                    );
                }
            }
        }
    }

    // Sort by start time, newest first
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    info!(count = sessions.len(), "Listed sessions");
    Json(SessionsResponse { sessions })
}

/// Serve a session's RRD file
///
/// Streams the session.rrd file for viewing in Rerun.
async fn serve_session_rrd(
    State(state): State<SharedState>,
    Path((rover_id, session_dir)): Path<(String, String)>,
) -> impl IntoResponse {
    // Build the path to the session file
    // Try both direct and nested structures:
    // - /data/sessions/{rover_id}/{session_dir}/session.rrd
    // - /data/sessions/{rover_id}/sessions/{session_dir}/session.rrd
    let direct_path = state
        .sessions_dir
        .join(&rover_id)
        .join(&session_dir)
        .join("session.rrd");
    let nested_path = state
        .sessions_dir
        .join(&rover_id)
        .join("sessions")
        .join(&session_dir)
        .join("session.rrd");

    let rrd_path = if direct_path.exists() {
        direct_path
    } else if nested_path.exists() {
        nested_path
    } else {
        warn!(
            rover_id = %rover_id,
            session_dir = %session_dir,
            "Session RRD file not found"
        );
        return Err((StatusCode::NOT_FOUND, "Session not found"));
    };

    // Open the file
    let file = match tokio::fs::File::open(&rrd_path).await {
        Ok(f) => f,
        Err(e) => {
            warn!(
                path = %rrd_path.display(),
                error = %e,
                "Failed to open session file"
            );
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read session"));
        }
    };

    // Get file size for Content-Length
    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file metadata"));
        }
    };

    let file_size = metadata.len();
    info!(
        path = %rrd_path.display(),
        size = file_size,
        "Serving session RRD"
    );

    // Stream the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"session.rrd\"",
            ),
            // Allow cross-origin requests for Rerun viewer
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        ],
        body,
    ))
}
