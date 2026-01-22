//! WebSocket teleop server for browser-based operators.
//!
//! Parallel to the UDP server, this handles commands/telemetry over WebSocket
//! for the web-based Operator app.

use crate::{
    serialize_telemetry, CommandHeader, Telemetry, TeleopError, CMD_HEADER_SIZE, MSG_ESTOP,
    MSG_ESTOP_RELEASE, MSG_HEARTBEAT, MSG_SET_MODE, MSG_TOOL, MSG_TWIST,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use types::{Command, Mode, ToolCommand, Twist};

/// WebSocket server configuration.
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub port: u16,
    pub heartbeat_interval: Duration,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            port: 4850,
            heartbeat_interval: Duration::from_millis(20), // 50Hz telemetry
        }
    }
}

/// WebSocket teleop server.
pub struct WsServer {
    config: WsConfig,
    command_tx: mpsc::Sender<Command>,
    telemetry_rx: watch::Receiver<Telemetry>,
}

impl WsServer {
    pub fn new(
        config: WsConfig,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
        }
    }

    /// Run the WebSocket server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!(addr, "WebSocket teleop server listening");

        let command_tx = Arc::new(self.command_tx);
        let telemetry_rx = self.telemetry_rx;
        let heartbeat_interval = self.config.heartbeat_interval;

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!(%addr, "WebSocket client connected");
                    let cmd_tx = command_tx.clone();
                    let telem_rx = telemetry_rx.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_connection(stream, cmd_tx, telem_rx, heartbeat_interval).await
                        {
                            error!(?e, "WebSocket connection error");
                        }
                        info!(%addr, "WebSocket client disconnected");
                    });
                }
                Err(e) => {
                    error!(?e, "Failed to accept WebSocket connection");
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    command_tx: Arc<mpsc::Sender<Command>>,
    telemetry_rx: watch::Receiver<Telemetry>,
    heartbeat_interval: Duration,
) -> Result<(), TeleopError> {
    // Disable Nagle's algorithm for lower latency
    let _ = stream.set_nodelay(true);

    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| TeleopError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Spawn telemetry sender task
    let telemetry_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(heartbeat_interval);
        loop {
            interval.tick().await;
            let telemetry = telemetry_rx.borrow().clone();
            if let Some(data) = serialize_telemetry(&telemetry) {
                if ws_sender.send(Message::Binary(data.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Process incoming commands
    let mut last_cmd_time = Instant::now();
    let mut cmd_count: u64 = 0;
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Some(cmd) = parse_command(&data) {
                    // Log timing gaps to debug jerky controls
                    let now = Instant::now();
                    let gap_ms = now.duration_since(last_cmd_time).as_millis();
                    cmd_count += 1;

                    // Parse header timestamp for latency calculation
                    let latency_info = if let Some(header) = CommandHeader::parse(&data) {
                        let server_ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u32)
                            .unwrap_or(0);
                        let latency = server_ts.wrapping_sub(header.timestamp_ms);
                        Some(latency)
                    } else {
                        None
                    };

                    if gap_ms > 100 {
                        warn!(gap_ms, cmd_count, latency_ms = ?latency_info, "Large gap between WebSocket commands");
                    } else if let Some(lat) = latency_info {
                        if lat > 100 {
                            warn!(latency_ms = lat, gap_ms, "High command latency");
                        }
                    }

                    last_cmd_time = now;
                    debug!(?cmd, gap_ms, latency_ms = ?latency_info, "WebSocket command received");
                    let _ = command_tx.send(cmd).await;
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by tungstenite
                debug!("Ping received: {:?}", data);
            }
            Err(e) => {
                warn!(?e, "WebSocket receive error");
                break;
            }
            _ => {}
        }
    }

    telemetry_task.abort();
    Ok(())
}

/// Parse a command from raw bytes.
fn parse_command(data: &[u8]) -> Option<Command> {
    let header = CommandHeader::parse(data)?;
    let payload = &data[CMD_HEADER_SIZE..];

    match header.msg_type {
        MSG_TWIST if payload.len() >= 9 => {
            let linear = f32::from_le_bytes(payload[0..4].try_into().ok()?) as f64;
            let angular = f32::from_le_bytes(payload[4..8].try_into().ok()?) as f64;
            let boost = payload[8] != 0;
            Some(Command::Twist(Twist {
                linear,
                angular,
                boost,
            }))
        }
        MSG_ESTOP => Some(Command::EStop),
        MSG_HEARTBEAT => Some(Command::Heartbeat),
        MSG_ESTOP_RELEASE => Some(Command::EStopRelease),
        MSG_SET_MODE if !payload.is_empty() => {
            let mode = match payload[0] {
                0 => Mode::Disabled,
                1 => Mode::Idle,
                2 => Mode::Teleop,
                3 => Mode::Autonomous,
                _ => return None,
            };
            debug!(?mode, seq = header.sequence, "SetMode command");
            Some(Command::SetMode(mode))
        }
        MSG_TOOL if payload.len() >= 10 => {
            let axis = f32::from_le_bytes(payload[0..4].try_into().ok()?);
            let motor = f32::from_le_bytes(payload[4..8].try_into().ok()?);
            let action_a = payload[8] != 0;
            let action_b = payload[9] != 0;
            Some(Command::Tool(ToolCommand {
                axis,
                motor,
                action_a,
                action_b,
            }))
        }
        _ => None,
    }
}
