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

/// System operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Mode {
    /// Powered off / safe state
    #[default]
    Disabled,
    /// Ready to receive commands but not moving
    Idle,
    /// Actively executing velocity commands from teleop
    Teleop,
    /// Autonomous operation
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
    /// Total system current draw (A)
    pub system_current: f64,
}

/// 2D pose in local frame (meters, radians).
/// Origin is where the rover was powered on (or last reset).
/// In production, this would be augmented with GPS coordinates.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Pose {
    /// X position in meters (positive = forward at theta=0)
    pub x: f64,
    /// Y position in meters (positive = left at theta=0)
    pub y: f64,
    /// Heading in radians (positive = counter-clockwise from X axis)
    pub theta: f64,
}

/// GPS coordinates (WGS84).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GpsCoord {
    /// Latitude in degrees
    pub lat: f64,
    /// Longitude in degrees  
    pub lon: f64,
    /// Altitude in meters (above WGS84 ellipsoid)
    pub alt: f64,
    /// Horizontal accuracy in meters (0 = unknown)
    pub accuracy: f32,
}

/// Command from operator/autonomy to rover.
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
    /// Tool command
    Tool(ToolCommand),
}

/// Command for the active tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCommand {
    /// Axis input (-1.0 to 1.0, e.g., lift up/down)
    pub axis: f32,
    /// Motor input (-1.0 to 1.0, e.g., auger speed)
    pub motor: f32,
    /// Action button states
    pub action_a: bool,
    pub action_b: bool,
}

impl Default for ToolCommand {
    fn default() -> Self {
        Self {
            axis: 0.0,
            motor: 0.0,
            action_a: false,
            action_b: false,
        }
    }
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
