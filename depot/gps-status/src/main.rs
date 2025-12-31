//! GPS Base Station Status Service
//!
//! Connects to a ZED-F9P GPS module via serial and exposes status information
//! via HTTP and WebSocket endpoints.
//!
//! Endpoints:
//! - GET /status - Current GPS status as JSON
//! - GET /ws - WebSocket for live status updates
//! - GET /health - Health check

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, RwLock};
use tokio_serial::SerialPortBuilderExt;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

/// GPS fix quality from NMEA GGA
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FixQuality {
    #[default]
    NoFix = 0,
    Gps = 1,
    Dgps = 2,
    Pps = 3,
    RtkFixed = 4,
    RtkFloat = 5,
    Estimated = 6,
    Manual = 7,
    Simulation = 8,
}

impl From<u8> for FixQuality {
    fn from(value: u8) -> Self {
        match value {
            1 => FixQuality::Gps,
            2 => FixQuality::Dgps,
            3 => FixQuality::Pps,
            4 => FixQuality::RtkFixed,
            5 => FixQuality::RtkFloat,
            6 => FixQuality::Estimated,
            7 => FixQuality::Manual,
            8 => FixQuality::Simulation,
            _ => FixQuality::NoFix,
        }
    }
}

/// GPS status information
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpsStatus {
    /// Whether we have a connection to the GPS module
    pub connected: bool,
    /// Operating mode: "base" or "rover" or "unknown"
    pub mode: String,
    /// Fix quality
    pub fix_quality: FixQuality,
    /// Number of satellites in use
    pub satellites: u8,
    /// Latitude in degrees (WGS84)
    pub latitude: Option<f64>,
    /// Longitude in degrees (WGS84)
    pub longitude: Option<f64>,
    /// Altitude above mean sea level in meters
    pub altitude: Option<f64>,
    /// Horizontal dilution of precision
    pub hdop: Option<f64>,
    /// Survey-in status (for base station mode)
    pub survey_in: Option<SurveyInStatus>,
    /// RTCM message statistics (for base station mode)
    pub rtcm_messages: Vec<RtcmMessageStats>,
    /// Number of connected NTRIP clients
    pub clients: Option<u32>,
    /// Last update timestamp (unix ms)
    pub last_update: u64,
}

/// Survey-in progress for base station
#[derive(Debug, Clone, Serialize, Default)]
pub struct SurveyInStatus {
    pub active: bool,
    pub valid: bool,
    pub duration: u32, // seconds
    pub accuracy: f64, // meters
}

/// RTCM message statistics
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcmMessageStats {
    #[serde(rename = "type")]
    pub msg_type: u16,
    pub count: u32,
    pub last_seen: u64, // unix ms
}

/// WebSocket message to clients
#[derive(Debug, Serialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: GpsStatus,
}

/// Shared application state
struct AppState {
    status: RwLock<GpsStatus>,
    broadcast_tx: broadcast::Sender<()>,
}

impl AppState {
    fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(16);
        Self {
            status: RwLock::new(GpsStatus::default()),
            broadcast_tx,
        }
    }

    async fn update_status<F>(&self, f: F)
    where
        F: FnOnce(&mut GpsStatus),
    {
        let mut status = self.status.write().await;
        f(&mut status);
        status.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        drop(status);
        let _ = self.broadcast_tx.send(());
    }

    async fn get_status(&self) -> GpsStatus {
        self.status.read().await.clone()
    }
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gps_status=info".into()),
        )
        .init();

    let state = Arc::new(AppState::new());

    // Serial port configuration
    let serial_port = std::env::var("GPS_SERIAL_PORT").unwrap_or_else(|_| "/dev/ttyUSB0".into());
    let baud_rate: u32 = std::env::var("GPS_BAUD_RATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(115200);

    // Spawn serial reader task
    let state_clone = state.clone();
    let serial_port_clone = serial_port.clone();
    tokio::spawn(async move {
        loop {
            match run_serial_reader(&serial_port_clone, baud_rate, state_clone.clone()).await {
                Ok(()) => {
                    info!("Serial reader exited, reconnecting...");
                }
                Err(e) => {
                    warn!(error = %e, port = %serial_port_clone, "Serial connection failed");
                    state_clone
                        .update_status(|s| {
                            s.connected = false;
                        })
                        .await;
                }
            }
            // Wait before reconnecting
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    let app = Router::new()
        .route("/status", get(get_status))
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4880);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(port = port, serial = %serial_port, baud = baud_rate, "GPS status service starting");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Run the serial reader loop
async fn run_serial_reader(port: &str, baud_rate: u32, state: SharedState) -> Result<(), String> {
    info!(port = %port, baud = baud_rate, "Opening serial port");

    let serial = tokio_serial::new(port, baud_rate)
        .open_native_async()
        .map_err(|e| format!("Failed to open serial port: {}", e))?;

    info!("Serial port opened successfully");

    state
        .update_status(|s| {
            s.connected = true;
            s.mode = "base".to_string(); // Assume base station mode
        })
        .await;

    let reader = BufReader::new(serial);
    let mut lines = reader.lines();
    let mut last_gga = Instant::now();

    while let Ok(Some(line)) = lines.next_line().await {
        // Parse NMEA sentences
        if line.starts_with("$GNGGA") || line.starts_with("$GPGGA") {
            if let Some(gga) = parse_gga(&line) {
                state
                    .update_status(|s| {
                        s.connected = true;
                        s.fix_quality = gga.fix_quality;
                        s.satellites = gga.satellites;
                        s.latitude = gga.latitude;
                        s.longitude = gga.longitude;
                        s.altitude = gga.altitude;
                        s.hdop = gga.hdop;
                    })
                    .await;
                last_gga = Instant::now();
                debug!(
                    fix = ?gga.fix_quality,
                    sats = gga.satellites,
                    "GGA update"
                );
            }
        }

        // Check for stale data
        if last_gga.elapsed() > Duration::from_secs(5) {
            state
                .update_status(|s| {
                    s.fix_quality = FixQuality::NoFix;
                })
                .await;
        }
    }

    Ok(())
}

/// Parsed GGA data
struct GgaData {
    fix_quality: FixQuality,
    satellites: u8,
    latitude: Option<f64>,
    longitude: Option<f64>,
    altitude: Option<f64>,
    hdop: Option<f64>,
}

/// Parse a GGA NMEA sentence
fn parse_gga(sentence: &str) -> Option<GgaData> {
    // Remove checksum and split by comma
    let parts: Vec<&str> = sentence
        .split('*')
        .next()?
        .split(',')
        .collect();

    if parts.len() < 15 {
        return None;
    }

    // Parse fix quality (field 6)
    let fix_quality: FixQuality = parts.get(6)?.parse::<u8>().ok()?.into();

    // Parse satellites (field 7)
    let satellites: u8 = parts.get(7)?.parse().unwrap_or(0);

    // Parse HDOP (field 8)
    let hdop: Option<f64> = parts.get(8).and_then(|s| s.parse().ok());

    // Parse latitude (fields 2,3)
    let latitude = parse_nmea_coord(parts.get(2).copied(), parts.get(3).copied());

    // Parse longitude (fields 4,5)
    let longitude = parse_nmea_coord(parts.get(4).copied(), parts.get(5).copied());

    // Parse altitude (field 9)
    let altitude: Option<f64> = parts.get(9).and_then(|s| s.parse().ok());

    Some(GgaData {
        fix_quality,
        satellites,
        latitude,
        longitude,
        altitude,
        hdop,
    })
}

/// Parse NMEA coordinate (DDMM.MMMM format) to decimal degrees
fn parse_nmea_coord(coord: Option<&str>, direction: Option<&str>) -> Option<f64> {
    let coord_str = coord?;
    let dir = direction?;

    if coord_str.is_empty() {
        return None;
    }

    // Find the decimal point to split degrees and minutes
    let dot_pos = coord_str.find('.')?;
    if dot_pos < 2 {
        return None;
    }

    let deg_len = dot_pos - 2;
    let degrees: f64 = coord_str[..deg_len].parse().ok()?;
    let minutes: f64 = coord_str[deg_len..].parse().ok()?;

    let mut result = degrees + minutes / 60.0;

    if dir == "S" || dir == "W" {
        result = -result;
    }

    Some(result)
}

/// Health check endpoint
async fn health(State(state): State<SharedState>) -> impl IntoResponse {
    let status = state.get_status().await;
    Json(serde_json::json!({
        "status": "ok",
        "connected": status.connected,
        "fix": status.fix_quality
    }))
}

/// Get current GPS status
async fn get_status(State(state): State<SharedState>) -> impl IntoResponse {
    let status = state.get_status().await;
    Json(status)
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

    // Send initial status
    let status = state.get_status().await;
    let msg = WsMessage {
        msg_type: "gps_status".to_string(),
        data: status,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    info!("Client connected to GPS status WebSocket");

    loop {
        tokio::select! {
            // Handle broadcast updates
            Ok(()) = rx.recv() => {
                let status = state.get_status().await;
                let msg = WsMessage {
                    msg_type: "gps_status".to_string(),
                    data: status,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Handle incoming messages
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

    info!("Client disconnected from GPS status WebSocket");
}
