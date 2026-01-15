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
use types::{Command, Mode, Pose, PowerStatus, SlamStatus, ToolCommand, Twist};

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
            heartbeat_interval: Duration::from_millis(20), // 50Hz telemetry
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
    /// SLAM status (optional, only when SLAM is enabled)
    #[serde(default)]
    pub slam_status: Option<SlamStatus>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use types::{Mode, Pose, PowerStatus, Twist};

    #[test]
    fn test_parse_twist_command() {
        let mut buf = vec![0x01]; // Twist message type
        buf.extend_from_slice(&1.5_f64.to_le_bytes()); // linear
        buf.extend_from_slice(&0.5_f64.to_le_bytes()); // angular

        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_some());
        if let Some(Command::Twist(twist)) = cmd {
            assert!((twist.linear - 1.5).abs() < 0.001);
            assert!((twist.angular - 0.5).abs() < 0.001);
            assert!(!twist.boost);
        } else {
            panic!("Expected Twist command");
        }
    }

    #[test]
    fn test_parse_twist_command_with_boost() {
        let mut buf = vec![0x01];
        buf.extend_from_slice(&2.0_f64.to_le_bytes());
        buf.extend_from_slice(&1.0_f64.to_le_bytes());
        buf.push(1); // boost = true

        let cmd = Server::parse_command(&buf);
        if let Some(Command::Twist(twist)) = cmd {
            assert!(twist.boost);
        } else {
            panic!("Expected Twist command with boost");
        }
    }

    #[test]
    fn test_parse_estop_command() {
        let buf = [0x02];
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::EStop)));
    }

    #[test]
    fn test_parse_heartbeat_command() {
        let buf = [0x03];
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::Heartbeat)));
    }

    #[test]
    fn test_parse_estop_release_command() {
        let buf = [0x06];
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::EStopRelease)));
    }

    #[test]
    fn test_parse_set_mode_command() {
        // Test each mode
        for (mode_byte, expected_mode) in [
            (0u8, Mode::Disabled),
            (1, Mode::Idle),
            (2, Mode::Teleop),
            (3, Mode::Autonomous),
        ] {
            let buf = [0x04, mode_byte];
            let cmd = Server::parse_command(&buf);
            if let Some(Command::SetMode(mode)) = cmd {
                assert_eq!(mode, expected_mode);
            } else {
                panic!("Expected SetMode command for mode byte {}", mode_byte);
            }
        }
    }

    #[test]
    fn test_parse_set_mode_invalid() {
        let buf = [0x04, 99]; // Invalid mode
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_tool_command() {
        let mut buf = vec![0x05];
        buf.extend_from_slice(&0.75_f32.to_le_bytes()); // axis
        buf.extend_from_slice(&(-0.5_f32).to_le_bytes()); // motor
        buf.push(1); // action_a = true
        buf.push(0); // action_b = false

        let cmd = Server::parse_command(&buf);
        if let Some(Command::Tool(tool_cmd)) = cmd {
            assert!((tool_cmd.axis - 0.75).abs() < 0.001);
            assert!((tool_cmd.motor - (-0.5)).abs() < 0.001);
            assert!(tool_cmd.action_a);
            assert!(!tool_cmd.action_b);
        } else {
            panic!("Expected Tool command");
        }
    }

    #[test]
    fn test_parse_empty_data() {
        let buf: [u8; 0] = [];
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_unknown_command_type() {
        let buf = [0xFF];
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_twist_too_short() {
        // Only 10 bytes instead of 17
        let buf = [0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_serialize_telemetry() {
        let telemetry = Telemetry {
            timestamp_ms: 12345,
            mode: Mode::Teleop,
            pose: Pose { x: 1.0, y: 2.0, theta: 0.5 },
            power: PowerStatus {
                battery_voltage: 48.0,
                system_current: 10.0,
            },
            velocity: Twist {
                linear: 0.5,
                angular: 0.1,
                boost: false,
            },
            motor_temps: [30.0, 31.0, 32.0, 33.0],
            motor_currents: [5.0, 5.5, 6.0, 6.5],
            active_tool: None,
            tool_status: None,
            slam_status: None,
        };

        let data = Server::serialize_telemetry(&telemetry);
        assert!(data.is_some());
        let data = data.unwrap();

        // Verify message type
        assert_eq!(data[0], 0x10);
        // Verify mode
        assert_eq!(data[1], Mode::Teleop as u8);

        // Verify pose (x, y, theta) - each f64 is 8 bytes
        let x = f64::from_le_bytes(data[2..10].try_into().unwrap());
        let y = f64::from_le_bytes(data[10..18].try_into().unwrap());
        let theta = f64::from_le_bytes(data[18..26].try_into().unwrap());
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 2.0).abs() < 0.001);
        assert!((theta - 0.5).abs() < 0.001);

        // Verify battery voltage
        let voltage = f64::from_le_bytes(data[26..34].try_into().unwrap());
        assert!((voltage - 48.0).abs() < 0.001);

        // Verify timestamp
        let ts = u64::from_le_bytes(data[34..42].try_into().unwrap());
        assert_eq!(ts, 12345);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.listen_port, 4840);
        assert_eq!(config.heartbeat_interval.as_millis(), 20);
        assert_eq!(config.connection_timeout.as_secs(), 1);
    }
}



