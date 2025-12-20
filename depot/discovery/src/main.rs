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

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

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

/// Shared application state
struct AppState {
    rovers: RwLock<HashMap<String, RoverInfo>>,
    broadcast_tx: broadcast::Sender<()>,
}

impl AppState {
    fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(16);
        Self {
            rovers: RwLock::new(HashMap::new()),
            broadcast_tx,
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

    let state = Arc::new(AppState::new());

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
