//! Shared types and message definitions for bvr.

use serde::{Deserialize, Serialize};

/// Velocity command: linear (m/s) and angular (rad/s).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Twist {
    /// Linear velocity in m/s (positive = forward)
    pub linear: f64,
    /// Angular velocity in rad/s (positive = counter-clockwise)
    pub angular: f64,
}

/// Individual wheel velocity command (rad/s).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WheelVelocities {
    pub front_left: f64,
    pub front_right: f64,
    pub rear_left: f64,
    pub rear_right: f64,
}

impl WheelVelocities {
    pub fn as_array(&self) -> [f64; 4] {
        [
            self.front_left,
            self.front_right,
            self.rear_left,
            self.rear_right,
        ]
    }

    pub fn from_array(arr: [f64; 4]) -> Self {
        Self {
            front_left: arr[0],
            front_right: arr[1],
            rear_left: arr[2],
            rear_right: arr[3],
        }
    }
}

/// ESC status report.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EscStatus {
    /// Node ID (1-4)
    pub id: u8,
    /// Current velocity (rad/s)
    pub velocity: f64,
    /// Motor current (A)
    pub current: f64,
    /// Temperature (°C)
    pub temperature: f64,
    /// Fault flags
    pub fault: EscFault,
}

/// ESC fault flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EscFault {
    pub over_current: bool,
    pub over_temperature: bool,
    pub under_voltage: bool,
    pub encoder_error: bool,
}

impl EscFault {
    pub fn any(&self) -> bool {
        self.over_current || self.over_temperature || self.under_voltage || self.encoder_error
    }
}

/// System operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Mode {
    /// Powered off / safe state
    #[default]
    Disabled,
    /// Ready to receive commands but not moving
    Idle,
    /// Actively executing velocity commands
    Teleop,
    /// Autonomous operation (future)
    Autonomous,
    /// Emergency stop triggered
    EStop,
    /// Fault condition
    Fault,
}

/// Power system status.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PowerStatus {
    /// Main battery voltage (V)
    pub battery_voltage: f64,
    /// 12V rail voltage (V)
    pub rail_12v: f64,
    /// Total system current draw (A)
    pub system_current: f64,
}

/// Full rover state for telemetry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoverState {
    pub mode: Mode,
    pub power: PowerStatus,
    pub escs: [EscStatus; 4],
    pub commanded_twist: Twist,
    pub timestamp_ms: u64,
}

/// Command from operator to rover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Set velocity
    Twist(Twist),
    /// Change mode
    SetMode(Mode),
    /// Emergency stop
    EStop,
    /// Heartbeat (keep-alive)
    Heartbeat,
}

/// Wheel position in the chassis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WheelPosition {
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
}

impl WheelPosition {
    pub fn index(&self) -> usize {
        match self {
            Self::FrontLeft => 0,
            Self::FrontRight => 1,
            Self::RearLeft => 2,
            Self::RearRight => 3,
        }
    }

    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::FrontLeft),
            1 => Some(Self::FrontRight),
            2 => Some(Self::RearLeft),
            3 => Some(Self::RearRight),
            _ => None,
        }
    }
}

