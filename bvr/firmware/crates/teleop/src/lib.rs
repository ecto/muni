//! LTE teleop communications for bvr.
//!
//! Handles command reception and telemetry transmission over unreliable links.

pub mod video;
pub mod video_ws;
pub mod ws;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, trace, warn};
use types::{Command, Mode, Pose, PowerStatus, ToolCommand, Twist};

#[derive(Error, Debug)]
pub enum TeleopError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection timeout")]
    Timeout,
}

/// Configuration for teleop.
#[derive(Debug, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_port: 4840,
            heartbeat_interval: Duration::from_millis(100),
            connection_timeout: Duration::from_secs(1),
        }
    }
}

/// Telemetry sent from rover to operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub timestamp_ms: u64,
    pub mode: Mode,
    pub pose: Pose,
    pub power: PowerStatus,
    pub velocity: Twist,
    pub motor_temps: [f32; 4],
    pub motor_currents: [f32; 4],
    pub active_tool: Option<String>,
    pub tool_status: Option<ToolStatus>,
}

/// Tool status for telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub position: Option<f32>,
    pub active: bool,
    pub current: Option<f32>,
}

/// Teleop server — runs in async context.
pub struct Server {
    config: Config,
    command_tx: mpsc::Sender<Command>,
    telemetry_rx: watch::Receiver<Telemetry>,
}

impl Server {
    pub fn new(
        config: Config,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
        }
    }

    /// Run the teleop server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.listen_port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        info!(addr, "Teleop server listening on UDP");

        let mut buf = [0u8; 1024];
        let mut operator_addr: Option<SocketAddr> = None;
        let mut last_recv = std::time::Instant::now();

        // Use interval instead of sleep - interval tracks time across iterations
        let mut telemetry_interval = tokio::time::interval(self.config.heartbeat_interval);

        loop {
            tokio::select! {
                // Receive commands
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            operator_addr = Some(addr);
                            last_recv = std::time::Instant::now();

                            if let Some(cmd) = Self::parse_command(&buf[..len]) {
                                debug!(?cmd, "Received command");
                                let _ = self.command_tx.send(cmd).await;
                            }
                        }
                        Err(e) => {
                            error!(?e, "Socket receive error");
                        }
                    }
                }

                // Send telemetry at regular intervals (not reset by recv)
                _ = telemetry_interval.tick() => {
                    if let Some(addr) = operator_addr {
                        let telemetry = self.telemetry_rx.borrow().clone();
                        if let Some(data) = Self::serialize_telemetry(&telemetry) {
                            match socket.send_to(&data, addr).await {
                                Ok(n) => trace!(%addr, bytes = n, mode = ?telemetry.mode, "Sent telemetry"),
                                Err(e) => error!(?e, "Failed to send telemetry"),
                            }
                        }

                        // Check connection timeout - only warn once per disconnect
                        if last_recv.elapsed() > self.config.connection_timeout {
                            // Don't spam the control loop - just log once and clear operator
                            if operator_addr.is_some() {
                                warn!("Operator connection timeout");
                                operator_addr = None;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parse a command from raw bytes.
    fn parse_command(data: &[u8]) -> Option<Command> {
        if data.is_empty() {
            return None;
        }

        // Simple binary protocol:
        // [0] = message type
        // [1..] = payload
        match data[0] {
            // Twist command
            0x01 if data.len() >= 17 => {
                let linear = f64::from_le_bytes(data[1..9].try_into().ok()?);
                let angular = f64::from_le_bytes(data[9..17].try_into().ok()?);
                Some(Command::Twist(Twist { linear, angular }))
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
            0x05 if data.len() >= 7 => {
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

    /// Serialize telemetry for transmission.
    fn serialize_telemetry(telemetry: &Telemetry) -> Option<Vec<u8>> {
        // Simple binary encoding (in production, use protobuf)
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
}

/// Helper to send commands from operator station (for testing/CLI).
pub async fn send_twist(addr: &str, twist: Twist) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let mut buf = Vec::with_capacity(17);
    buf.push(0x01);
    buf.extend_from_slice(&twist.linear.to_le_bytes());
    buf.extend_from_slice(&twist.angular.to_le_bytes());

    socket.send_to(&buf, addr).await?;
    Ok(())
}

/// Helper to send e-stop.
pub async fn send_estop(addr: &str) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(&[0x02], addr).await?;
    Ok(())
}



