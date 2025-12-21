//! WebSocket teleop server for browser-based operators.
//!
//! Parallel to the UDP server, this handles commands/telemetry over WebSocket
//! for the web-based Operator app.

use crate::{Telemetry, TeleopError};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
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
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Some(cmd) = parse_command(&data) {
                    debug!(?cmd, "WebSocket command received");
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

/// Parse a command from raw bytes (same format as UDP).
fn parse_command(data: &[u8]) -> Option<Command> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        // Twist command (with optional boost byte)
        0x01 if data.len() >= 17 => {
            let linear = f64::from_le_bytes(data[1..9].try_into().ok()?);
            let angular = f64::from_le_bytes(data[9..17].try_into().ok()?);
            let boost = data.get(17).map(|&b| b != 0).unwrap_or(false);
            Some(Command::Twist(Twist { linear, angular, boost }))
        }
        // E-Stop
        0x02 => Some(Command::EStop),
        // Heartbeat
        0x03 => Some(Command::Heartbeat),
        // E-Stop Release
        0x06 => Some(Command::EStopRelease),
        // Set mode
        0x04 if data.len() >= 2 => {
            let mode = match data[1] {
                0 => Mode::Disabled,
                1 => Mode::Idle,
                2 => Mode::Teleop,
                3 => Mode::Autonomous,
                _ => return None,
            };
            Some(Command::SetMode(mode))
        }
        // Tool command
        0x05 if data.len() >= 11 => {
            let axis = f32::from_le_bytes(data[1..5].try_into().ok()?);
            let motor = f32::from_le_bytes(data[5..9].try_into().ok()?);
            let action_a = data.get(9).map(|&b| b != 0).unwrap_or(false);
            let action_b = data.get(10).map(|&b| b != 0).unwrap_or(false);
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

/// Serialize telemetry for transmission (same format as UDP).
fn serialize_telemetry(telemetry: &Telemetry) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(128);

    buf.push(0x10); // Telemetry message type
    buf.push(telemetry.mode as u8);

    // Pose (x, y, theta) - 24 bytes
    buf.extend_from_slice(&telemetry.pose.x.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.y.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.theta.to_le_bytes());

    // Power
    buf.extend_from_slice(&telemetry.power.battery_voltage.to_le_bytes());

    // Timestamp
    buf.extend_from_slice(&telemetry.timestamp_ms.to_le_bytes());

    // Velocity
    buf.extend_from_slice(&telemetry.velocity.linear.to_le_bytes());
    buf.extend_from_slice(&telemetry.velocity.angular.to_le_bytes());

    // Motor temps
    for temp in &telemetry.motor_temps {
        buf.extend_from_slice(&temp.to_le_bytes());
    }

    // Motor currents
    for current in &telemetry.motor_currents {
        buf.extend_from_slice(&current.to_le_bytes());
    }

    Some(buf)
}
