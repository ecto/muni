//! Teleop communications for bvr.
//!
//! Handles command reception and telemetry transmission over unreliable links.
//!
//! # Protocol (V2)
//!
//! ## Commands (Console → Rover)
//! All commands have 8-byte header: [type, flags, seq_lo, seq_hi, ts_0, ts_1, ts_2, ts_3]
//! - type: Command type (0x01=Twist, 0x02=EStop, 0x03=Heartbeat, etc.)
//! - flags: MUST_ACK (0x01), RETRANSMIT (0x02)
//! - seq: u16 LE sequence number
//! - ts: u32 LE timestamp in milliseconds
//!
//! ## Telemetry (Rover → Console)
//! 128 bytes with pose, velocity, measured velocity, acceleration, motor data, ACK fields, IMU orientation.

pub mod costmap_stream;
pub mod pointcloud;
pub mod rtc;
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
use types::{Command, Mode, Pose, PowerStatus, SlamStatus, SubsystemHealth, ToolCommand, Twist};

// =============================================================================
// Protocol Constants
// =============================================================================

/// Command types
pub const MSG_TWIST: u8 = 0x01;
pub const MSG_ESTOP: u8 = 0x02;
pub const MSG_HEARTBEAT: u8 = 0x03;
pub const MSG_SET_MODE: u8 = 0x04;
pub const MSG_TOOL: u8 = 0x05;
pub const MSG_ESTOP_RELEASE: u8 = 0x06;
pub const MSG_LIDAR_TOGGLE: u8 = 0x07;

/// Telemetry message type
pub const MSG_TELEMETRY: u8 = 0x11;

/// Point cloud message type
pub const MSG_POINT_CLOUD: u8 = 0x21;

/// Costmap message type
pub const MSG_COSTMAP: u8 = 0x22;

/// Command flags
pub mod cmd_flags {
    /// Require firmware ACK (SetMode, EStop, EStopRelease)
    pub const MUST_ACK: u8 = 1 << 0;
    /// This is a retransmission
    pub const RETRANSMIT: u8 = 1 << 1;
}

/// Command header size (8 bytes)
pub const CMD_HEADER_SIZE: usize = 8;

/// Parsed command header.
#[derive(Debug, Clone, Copy)]
pub struct CommandHeader {
    pub msg_type: u8,
    pub flags: u8,
    pub sequence: u16,
    pub timestamp_ms: u32,
}

impl CommandHeader {
    /// Parse header from bytes. Returns None if buffer too short.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < CMD_HEADER_SIZE {
            return None;
        }
        Some(Self {
            msg_type: data[0],
            flags: data[1],
            sequence: u16::from_le_bytes([data[2], data[3]]),
            timestamp_ms: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        })
    }
}

/// Sliding window for tracking received command sequences.
#[derive(Debug, Default)]
pub struct SequenceTracker {
    last_seq: u16,
    received_bits: u16,
}

impl SequenceTracker {
    const MAX_AGE_MS: u32 = 500;

    pub fn new() -> Self {
        Self::default()
    }

    /// Process incoming command. Returns true if should be processed.
    pub fn accept(&mut self, seq: u16, timestamp_ms: u32, now_ms: u32) -> bool {
        let age = now_ms.wrapping_sub(timestamp_ms);
        if age > Self::MAX_AGE_MS {
            trace!(seq, age, "Rejecting stale command");
            return false;
        }

        let dist = seq.wrapping_sub(self.last_seq) as i16;

        if dist > 0 {
            if dist < 16 {
                self.received_bits = (self.received_bits << dist) | 1;
            } else {
                self.received_bits = 1;
            }
            self.last_seq = seq;
            true
        } else if dist >= -15 {
            let bit_pos = (-dist) as u32;
            let mask = 1u16 << bit_pos;
            if self.received_bits & mask != 0 {
                trace!(seq, "Rejecting duplicate command");
                false
            } else {
                self.received_bits |= mask;
                true
            }
        } else {
            trace!(seq, dist, "Rejecting command before window");
            false
        }
    }

    pub fn last_seq(&self) -> u16 {
        self.last_seq
    }

    pub fn ack_bits(&self) -> u16 {
        self.received_bits
    }
}

/// Velocity tracker for computing measured velocity from odometry.
#[derive(Debug, Default)]
pub struct VelocityTracker {
    linear: f64,
    angular: f64,
    linear_accel: f64,
    angular_accel: f64,
    last_update_us: u64,
    quality: u16,
}

impl VelocityTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update with odometry displacement.
    pub fn update(&mut self, dx: f64, dtheta: f64, now_us: u64) {
        if self.last_update_us == 0 {
            self.last_update_us = now_us;
            return;
        }

        let dt_us = now_us.saturating_sub(self.last_update_us);
        let dt = (dt_us as f64) / 1_000_000.0;

        if dt < 0.001 {
            return;
        }

        let new_linear = dx / dt;
        let new_angular = dtheta / dt;

        self.linear_accel = (new_linear - self.linear) / dt;
        self.angular_accel = (new_angular - self.angular) / dt;

        self.linear = new_linear;
        self.angular = new_angular;
        self.last_update_us = now_us;

        self.quality = if self.linear.abs() < 10.0 && self.angular.abs() < 5.0 {
            100
        } else {
            50
        };
    }

    pub fn linear(&self) -> f64 {
        self.linear
    }

    pub fn angular(&self) -> f64 {
        self.angular
    }

    pub fn linear_accel(&self) -> f64 {
        self.linear_accel
    }

    pub fn angular_accel(&self) -> f64 {
        self.angular_accel
    }

    pub fn quality(&self) -> u16 {
        self.quality
    }

    pub fn dt_ms(&self, now_us: u64) -> f32 {
        let dt_us = now_us.saturating_sub(self.last_update_us);
        (dt_us as f32) / 1000.0
    }
}

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

/// Telemetry sent from rover to operator (120 bytes binary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    /// Sequence number for loss detection
    pub sequence: u16,
    /// Timestamp in microseconds (monotonic)
    pub timestamp_us: u64,
    pub mode: Mode,
    pub pose: Pose,
    pub power: PowerStatus,
    /// Commanded velocity
    pub cmd_velocity: Twist,
    /// Measured velocity from wheel odometry
    pub meas_velocity: (f32, f32), // (linear, angular)
    /// Acceleration (for Hermite interpolation)
    pub acceleration: (f32, f32), // (linear, angular)
    pub motor_temps: [f32; 4],
    pub motor_currents: [f32; 4],
    pub health: SubsystemHealth,
    /// Odometry quality (0-100)
    pub odometry_quality: u16,
    /// Actual update interval in milliseconds
    pub dt_ms: f32,
    /// Last command sequence received (for ACK)
    pub last_cmd_seq: u16,
    /// Bitfield: which of last 16 command sequences were received
    pub ack_bits: u16,
    /// Roll angle from IMU (radians, 0 = level)
    pub roll: f32,
    /// Pitch angle from IMU (radians, 0 = level)
    pub pitch: f32,
    /// System CPU usage (0-100%)
    pub cpu_percent: u8,
    /// System memory usage (0-100%)
    pub mem_percent: u8,
    /// System disk usage (0-100%)
    pub disk_percent: u8,
    /// Active tool name (not sent over binary, only JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_tool: Option<String>,
    /// Tool status (not sent over binary, only JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_status: Option<ToolStatus>,
    /// SLAM status (not sent over binary, only JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slam_status: Option<SlamStatus>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            sequence: 0,
            timestamp_us: 0,
            mode: Mode::Disabled,
            pose: Pose::default(),
            power: PowerStatus::default(),
            cmd_velocity: Twist::default(),
            meas_velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            motor_temps: [0.0; 4],
            motor_currents: [0.0; 4],
            health: SubsystemHealth::default(),
            odometry_quality: 0,
            dt_ms: 0.0,
            last_cmd_seq: 0,
            ack_bits: 0,
            roll: 0.0,
            pitch: 0.0,
            cpu_percent: 0,
            mem_percent: 0,
            disk_percent: 0,
            active_tool: None,
            tool_status: None,
            slam_status: None,
        }
    }
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
                                // Log SetMode at info level for visibility
                                match &cmd {
                                    Command::SetMode(mode) => info!(?mode, %addr, "UDP SetMode received"),
                                    _ => debug!(?cmd, %addr, "UDP command received"),
                                }
                                // Use try_send to avoid blocking if channel is full.
                                // This prevents cascading stalls when main loop is slow.
                                if let Err(e) = self.command_tx.try_send(cmd) {
                                    warn!(?e, "Command channel full, dropping UDP command");
                                }
                            } else {
                                // Log failed parses for debugging
                                let hex: String = buf[..len.min(16)].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                                warn!(len, hex, %addr, "UDP failed to parse command");
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
                        if let Some(data) = serialize_telemetry(&telemetry) {
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
    ///
    /// Binary format:
    /// [0]     u8      msg_type
    /// [1]     u8      flags
    /// [2-3]   u16 LE  sequence
    /// [4-7]   u32 LE  timestamp_ms
    /// [8...]          payload
    fn parse_command(data: &[u8]) -> Option<Command> {
        let header = CommandHeader::parse(data)?;
        let payload = &data[CMD_HEADER_SIZE..];

        match header.msg_type {
            // Twist: [linear:f32] [angular:f32] [boost:u8]
            MSG_TWIST if payload.len() >= 9 => {
                let linear = f32::from_le_bytes(payload[0..4].try_into().ok()?) as f64;
                let angular = f32::from_le_bytes(payload[4..8].try_into().ok()?) as f64;
                let boost = payload[8] != 0;
                Some(Command::Twist(Twist { linear, angular, boost }))
            }
            // E-Stop (no payload)
            MSG_ESTOP => Some(Command::EStop),
            // Heartbeat (no payload)
            MSG_HEARTBEAT => Some(Command::Heartbeat),
            // E-Stop Release (no payload)
            MSG_ESTOP_RELEASE => Some(Command::EStopRelease),
            // SetMode: [mode:u8]
            MSG_SET_MODE if !payload.is_empty() => {
                let mode = match payload[0] {
                    0 => Mode::Disabled,
                    1 => Mode::Idle,
                    2 => Mode::Teleop,
                    3 => Mode::Autonomous,
                    6 => Mode::Sleep,
                    _ => return None,
                };
                debug!(?mode, seq = header.sequence, "SetMode command");
                Some(Command::SetMode(mode))
            }
            // Tool: [axis:f32] [motor:f32] [action_a:u8] [action_b:u8]
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

}

/// Serialize telemetry for transmission (128 bytes).
///
/// This is the single source of truth for telemetry serialization.
/// Used by UDP, WebSocket, and WebRTC transports.
///
/// Binary format:
/// ```text
/// [0]       u8      msg_type (0x11)
/// [1]       u8      mode
/// [2-3]     u16 LE  sequence
/// [4-6]     u8×3    system metrics (cpu%, mem%, disk%)
/// [7]       u8      reserved
/// [8-31]    f64×3   pose (x, y, theta)
/// [32-39]   f64     battery_voltage
/// [40-47]   u64 LE  timestamp_us (monotonic)
/// [48-55]   f32×2   cmd_velocity (linear, angular)
/// [56-63]   f32×2   meas_velocity (linear, angular)
/// [64-71]   f32×2   acceleration (linear, angular)
/// [72-87]   f32×4   motor_temps
/// [88-103]  f32×4   motor_currents
/// [104-105] u16 LE  health_bits
/// [106-107] u16 LE  odometry_quality
/// [108-111] f32 LE  dt_ms
/// [112-113] u16 LE  last_cmd_seq (ACK)
/// [114-115] u16 LE  ack_bits
/// [116-119] f32 LE  roll (radians, from IMU)
/// [120-123] f32 LE  pitch (radians, from IMU)
/// [124-127] u32     crc32
/// ```
pub fn serialize_telemetry(telemetry: &Telemetry) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(128);

    // Header
    buf.push(MSG_TELEMETRY);        // [0]
    buf.push(telemetry.mode as u8); // [1]
    buf.extend_from_slice(&telemetry.sequence.to_le_bytes()); // [2-3]
    // System metrics [4-6] + padding [7]
    buf.push(telemetry.cpu_percent);  // [4]
    buf.push(telemetry.mem_percent);  // [5]
    buf.push(telemetry.disk_percent); // [6]
    buf.push(0);                      // [7] reserved

    // Pose (x, y, theta) - 24 bytes [8-31]
    buf.extend_from_slice(&telemetry.pose.x.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.y.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.theta.to_le_bytes());

    // Battery voltage [32-39]
    buf.extend_from_slice(&telemetry.power.battery_voltage.to_le_bytes());

    // Timestamp [40-47]
    buf.extend_from_slice(&telemetry.timestamp_us.to_le_bytes());

    // Commanded velocity [48-55]
    buf.extend_from_slice(&(telemetry.cmd_velocity.linear as f32).to_le_bytes());
    buf.extend_from_slice(&(telemetry.cmd_velocity.angular as f32).to_le_bytes());

    // Measured velocity [56-63]
    buf.extend_from_slice(&telemetry.meas_velocity.0.to_le_bytes());
    buf.extend_from_slice(&telemetry.meas_velocity.1.to_le_bytes());

    // Acceleration [64-71]
    buf.extend_from_slice(&telemetry.acceleration.0.to_le_bytes());
    buf.extend_from_slice(&telemetry.acceleration.1.to_le_bytes());

    // Motor temps [72-87]
    for temp in &telemetry.motor_temps {
        buf.extend_from_slice(&temp.to_le_bytes());
    }

    // Motor currents [88-103]
    for current in &telemetry.motor_currents {
        buf.extend_from_slice(&current.to_le_bytes());
    }

    // Health bits [104-105]
    buf.extend_from_slice(&telemetry.health.to_bits().to_le_bytes());

    // Odometry quality [106-107]
    buf.extend_from_slice(&telemetry.odometry_quality.to_le_bytes());

    // dt_ms [108-111]
    buf.extend_from_slice(&telemetry.dt_ms.to_le_bytes());

    // ACK fields [112-115]
    buf.extend_from_slice(&telemetry.last_cmd_seq.to_le_bytes());
    buf.extend_from_slice(&telemetry.ack_bits.to_le_bytes());

    // IMU orientation [116-123]
    buf.extend_from_slice(&telemetry.roll.to_le_bytes());
    buf.extend_from_slice(&telemetry.pitch.to_le_bytes());

    // CRC32 [124-127]
    let crc = crc32fast::hash(&buf);
    buf.extend_from_slice(&crc.to_le_bytes());

    debug_assert_eq!(buf.len(), 128);
    Some(buf)
}

/// Helper to send commands from operator station (for testing/CLI).
/// Uses full 8-byte header format required by parse_command.
pub async fn send_twist(addr: &str, twist: Twist) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Build full command: 8-byte header + 9-byte payload
    let mut buf = Vec::with_capacity(17);
    buf.push(MSG_TWIST); // type
    buf.push(0); // flags
    buf.extend_from_slice(&0u16.to_le_bytes()); // sequence
    buf.extend_from_slice(&0u32.to_le_bytes()); // timestamp_ms
    buf.extend_from_slice(&(twist.linear as f32).to_le_bytes()); // linear (f32)
    buf.extend_from_slice(&(twist.angular as f32).to_le_bytes()); // angular (f32)
    buf.push(if twist.boost { 1 } else { 0 }); // boost

    socket.send_to(&buf, addr).await?;
    Ok(())
}

/// Helper to send e-stop.
/// Uses full 8-byte header format required by parse_command.
pub async fn send_estop(addr: &str) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Build full command: 8-byte header only (no payload)
    let mut buf = Vec::with_capacity(8);
    buf.push(MSG_ESTOP); // type
    buf.push(0x01); // flags (MUST_ACK)
    buf.extend_from_slice(&0u16.to_le_bytes()); // sequence
    buf.extend_from_slice(&0u32.to_le_bytes()); // timestamp_ms

    socket.send_to(&buf, addr).await?;
    Ok(())
}

/// Helper to send SetMode command.
/// Uses full 8-byte header format required by parse_command.
pub async fn send_set_mode(addr: &str, mode: types::Mode) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Build full command with 8-byte header + 1 byte payload
    let mut buf = Vec::with_capacity(9);
    buf.push(MSG_SET_MODE); // type
    buf.push(0x01); // flags (MUST_ACK)
    buf.extend_from_slice(&0u16.to_le_bytes()); // sequence
    buf.extend_from_slice(&0u32.to_le_bytes()); // timestamp_ms
    buf.push(mode as u8); // payload: mode

    socket.send_to(&buf, addr).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{Mode, Pose, PowerStatus, SubsystemHealth, Twist};

    /// Build a command with header for testing.
    fn build_command(msg_type: u8, seq: u16, payload: &[u8]) -> Vec<u8> {
        let mut buf = vec![msg_type, 0]; // type, flags
        buf.extend_from_slice(&seq.to_le_bytes()); // sequence
        buf.extend_from_slice(&0u32.to_le_bytes()); // timestamp_ms
        buf.extend_from_slice(payload);
        buf
    }

    #[test]
    fn test_parse_twist_command() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1.5_f32.to_le_bytes()); // linear
        payload.extend_from_slice(&0.5_f32.to_le_bytes()); // angular
        payload.push(0); // boost = false

        let buf = build_command(MSG_TWIST, 1, &payload);
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
        let mut payload = Vec::new();
        payload.extend_from_slice(&2.0_f32.to_le_bytes());
        payload.extend_from_slice(&1.0_f32.to_le_bytes());
        payload.push(1); // boost = true

        let buf = build_command(MSG_TWIST, 2, &payload);
        let cmd = Server::parse_command(&buf);

        if let Some(Command::Twist(twist)) = cmd {
            assert!(twist.boost);
        } else {
            panic!("Expected Twist command with boost");
        }
    }

    #[test]
    fn test_parse_estop_command() {
        let buf = build_command(MSG_ESTOP, 3, &[]);
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::EStop)));
    }

    #[test]
    fn test_parse_heartbeat_command() {
        let buf = build_command(MSG_HEARTBEAT, 4, &[]);
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::Heartbeat)));
    }

    #[test]
    fn test_parse_estop_release_command() {
        let buf = build_command(MSG_ESTOP_RELEASE, 5, &[]);
        let cmd = Server::parse_command(&buf);
        assert!(matches!(cmd, Some(Command::EStopRelease)));
    }

    #[test]
    fn test_parse_set_mode_command() {
        for (mode_byte, expected_mode) in [
            (0u8, Mode::Disabled),
            (1, Mode::Idle),
            (2, Mode::Teleop),
            (3, Mode::Autonomous),
        ] {
            let buf = build_command(MSG_SET_MODE, 6, &[mode_byte]);
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
        let buf = build_command(MSG_SET_MODE, 7, &[99]); // Invalid mode
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_tool_command() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0.75_f32.to_le_bytes()); // axis
        payload.extend_from_slice(&(-0.5_f32).to_le_bytes()); // motor
        payload.push(1); // action_a = true
        payload.push(0); // action_b = false

        let buf = build_command(MSG_TOOL, 8, &payload);
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
    fn test_parse_header_too_short() {
        let buf = [0x01, 0, 0]; // Only 3 bytes, need 8
        let cmd = Server::parse_command(&buf);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_serialize_telemetry() {
        let telemetry = Telemetry {
            sequence: 42,
            timestamp_us: 12345000,
            mode: Mode::Teleop,
            pose: Pose { x: 1.0, y: 2.0, theta: 0.5 },
            power: PowerStatus {
                battery_voltage: 48.0,
                system_current: 10.0,
            },
            cmd_velocity: Twist {
                linear: 0.5,
                angular: 0.1,
                boost: false,
            },
            meas_velocity: (0.48, 0.09),
            acceleration: (0.1, 0.05),
            motor_temps: [30.0, 31.0, 32.0, 33.0],
            motor_currents: [5.0, 5.5, 6.0, 6.5],
            health: SubsystemHealth::default(),
            odometry_quality: 95,
            dt_ms: 20.0,
            last_cmd_seq: 100,
            ack_bits: 0xFFFF,
            roll: 0.1,
            pitch: -0.05,
            cpu_percent: 25,
            mem_percent: 50,
            disk_percent: 30,
            active_tool: None,
            tool_status: None,
            slam_status: None,
        };

        let data = serialize_telemetry(&telemetry);
        assert!(data.is_some());
        let data = data.unwrap();

        // Verify size (128 bytes)
        assert_eq!(data.len(), 128);

        // Verify message type
        assert_eq!(data[0], MSG_TELEMETRY);
        // Verify mode
        assert_eq!(data[1], Mode::Teleop as u8);
        // Verify sequence
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 42);

        // Verify pose (x, y, theta) at offset 8
        let x = f64::from_le_bytes(data[8..16].try_into().unwrap());
        let y = f64::from_le_bytes(data[16..24].try_into().unwrap());
        let theta = f64::from_le_bytes(data[24..32].try_into().unwrap());
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 2.0).abs() < 0.001);
        assert!((theta - 0.5).abs() < 0.001);

        // Verify battery voltage at offset 32
        let voltage = f64::from_le_bytes(data[32..40].try_into().unwrap());
        assert!((voltage - 48.0).abs() < 0.001);

        // Verify ACK fields at offset 112
        let last_cmd_seq = u16::from_le_bytes([data[112], data[113]]);
        let ack_bits = u16::from_le_bytes([data[114], data[115]]);
        assert_eq!(last_cmd_seq, 100);
        assert_eq!(ack_bits, 0xFFFF);
    }

    #[test]
    fn test_sequence_tracker() {
        let mut tracker = SequenceTracker::new();
        let now_ms = 1000u32;

        // Sequence 0 is initial state, so seq 1 is dist=1 (ahead)
        assert!(tracker.accept(1, now_ms, now_ms));
        assert_eq!(tracker.last_seq(), 1);

        // Sequential command should be accepted
        assert!(tracker.accept(2, now_ms, now_ms));
        assert_eq!(tracker.last_seq(), 2);

        // Duplicate should be rejected
        assert!(!tracker.accept(2, now_ms, now_ms));

        // Already seen seq 1, should be rejected as duplicate
        assert!(!tracker.accept(1, now_ms, now_ms));

        // New sequence 3 should be accepted
        assert!(tracker.accept(3, now_ms, now_ms));
        assert_eq!(tracker.last_seq(), 3);

        // Stale command should be rejected
        assert!(!tracker.accept(4, now_ms - 1000, now_ms)); // 1000ms old
    }

    #[test]
    fn test_velocity_tracker() {
        let mut tracker = VelocityTracker::new();

        // First update just records timestamp (initial last_update_us is 0)
        tracker.update(0.0, 0.0, 1000); // Start at 1ms
        assert_eq!(tracker.linear(), 0.0);

        // Second update computes velocity
        // 0.1m forward in 100ms = 1.0 m/s
        tracker.update(0.1, 0.0, 101_000); // 100ms later
        assert!((tracker.linear() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.listen_port, 4840);
        assert_eq!(config.heartbeat_interval.as_millis(), 20);
        assert_eq!(config.connection_timeout.as_secs(), 1);
    }
}



