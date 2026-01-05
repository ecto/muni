//! Shared types and message definitions for bvr.

use serde::{Deserialize, Serialize};

/// Velocity command: linear (m/s) and angular (rad/s).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Twist {
    /// Linear velocity in m/s (positive = forward)
    pub linear: f64,
    /// Angular velocity in rad/s (positive = counter-clockwise)
    pub angular: f64,
    /// Boost mode (shift/L3) - removes speed limiter for full power
    #[serde(default)]
    pub boost: bool,
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
    /// Release emergency stop (return to Idle)
    EStopRelease,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twist_serde_roundtrip() {
        let twist = Twist {
            linear: 1.5,
            angular: -0.5,
            boost: true,
        };
        let json = serde_json::to_string(&twist).unwrap();
        let decoded: Twist = serde_json::from_str(&json).unwrap();
        assert!((decoded.linear - twist.linear).abs() < 0.001);
        assert!((decoded.angular - twist.angular).abs() < 0.001);
        assert_eq!(decoded.boost, twist.boost);
    }

    #[test]
    fn test_twist_default() {
        let twist = Twist::default();
        assert_eq!(twist.linear, 0.0);
        assert_eq!(twist.angular, 0.0);
        assert!(!twist.boost);
    }

    #[test]
    fn test_wheel_velocities_array_conversion() {
        let wv = WheelVelocities {
            front_left: 1.0,
            front_right: 2.0,
            rear_left: 3.0,
            rear_right: 4.0,
        };
        let arr = wv.as_array();
        assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);

        let wv2 = WheelVelocities::from_array(arr);
        assert_eq!(wv2.front_left, 1.0);
        assert_eq!(wv2.front_right, 2.0);
        assert_eq!(wv2.rear_left, 3.0);
        assert_eq!(wv2.rear_right, 4.0);
    }

    #[test]
    fn test_wheel_velocities_serde_roundtrip() {
        let wv = WheelVelocities {
            front_left: 10.5,
            front_right: -10.5,
            rear_left: 5.0,
            rear_right: -5.0,
        };
        let json = serde_json::to_string(&wv).unwrap();
        let decoded: WheelVelocities = serde_json::from_str(&json).unwrap();
        assert!((decoded.front_left - wv.front_left).abs() < 0.001);
        assert!((decoded.front_right - wv.front_right).abs() < 0.001);
        assert!((decoded.rear_left - wv.rear_left).abs() < 0.001);
        assert!((decoded.rear_right - wv.rear_right).abs() < 0.001);
    }

    #[test]
    fn test_mode_serde_roundtrip() {
        for mode in [
            Mode::Disabled,
            Mode::Idle,
            Mode::Teleop,
            Mode::Autonomous,
            Mode::EStop,
            Mode::Fault,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let decoded: Mode = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, mode);
        }
    }

    #[test]
    fn test_mode_default() {
        assert_eq!(Mode::default(), Mode::Disabled);
    }

    #[test]
    fn test_pose_serde_roundtrip() {
        let pose = Pose {
            x: 100.5,
            y: -50.25,
            theta: std::f64::consts::PI / 4.0,
        };
        let json = serde_json::to_string(&pose).unwrap();
        let decoded: Pose = serde_json::from_str(&json).unwrap();
        assert!((decoded.x - pose.x).abs() < 0.001);
        assert!((decoded.y - pose.y).abs() < 0.001);
        assert!((decoded.theta - pose.theta).abs() < 0.001);
    }

    #[test]
    fn test_gps_coord_serde_roundtrip() {
        let coord = GpsCoord {
            lat: 42.3601,
            lon: -71.0589,
            alt: 10.5,
            accuracy: 2.5,
        };
        let json = serde_json::to_string(&coord).unwrap();
        let decoded: GpsCoord = serde_json::from_str(&json).unwrap();
        assert!((decoded.lat - coord.lat).abs() < 0.0001);
        assert!((decoded.lon - coord.lon).abs() < 0.0001);
        assert!((decoded.alt - coord.alt).abs() < 0.001);
        assert!((decoded.accuracy - coord.accuracy).abs() < 0.001);
    }

    #[test]
    fn test_power_status_serde_roundtrip() {
        let status = PowerStatus {
            battery_voltage: 48.5,
            system_current: 15.2,
        };
        let json = serde_json::to_string(&status).unwrap();
        let decoded: PowerStatus = serde_json::from_str(&json).unwrap();
        assert!((decoded.battery_voltage - status.battery_voltage).abs() < 0.001);
        assert!((decoded.system_current - status.system_current).abs() < 0.001);
    }

    #[test]
    fn test_command_variants_serde() {
        // Test Twist command
        let cmd = Command::Twist(Twist {
            linear: 1.0,
            angular: 0.5,
            boost: false,
        });
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::Twist(_)));

        // Test SetMode
        let cmd = Command::SetMode(Mode::Teleop);
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::SetMode(Mode::Teleop)));

        // Test EStop
        let cmd = Command::EStop;
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::EStop));

        // Test EStopRelease
        let cmd = Command::EStopRelease;
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::EStopRelease));

        // Test Heartbeat
        let cmd = Command::Heartbeat;
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::Heartbeat));

        // Test Tool command
        let cmd = Command::Tool(ToolCommand {
            axis: 0.5,
            motor: -0.25,
            action_a: true,
            action_b: false,
        });
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Command::Tool(_)));
    }

    #[test]
    fn test_tool_command_default() {
        let tc = ToolCommand::default();
        assert_eq!(tc.axis, 0.0);
        assert_eq!(tc.motor, 0.0);
        assert!(!tc.action_a);
        assert!(!tc.action_b);
    }

    #[test]
    fn test_wheel_position_index() {
        assert_eq!(WheelPosition::FrontLeft.index(), 0);
        assert_eq!(WheelPosition::FrontRight.index(), 1);
        assert_eq!(WheelPosition::RearLeft.index(), 2);
        assert_eq!(WheelPosition::RearRight.index(), 3);
    }

    #[test]
    fn test_wheel_position_from_index() {
        assert_eq!(WheelPosition::from_index(0), Some(WheelPosition::FrontLeft));
        assert_eq!(WheelPosition::from_index(1), Some(WheelPosition::FrontRight));
        assert_eq!(WheelPosition::from_index(2), Some(WheelPosition::RearLeft));
        assert_eq!(WheelPosition::from_index(3), Some(WheelPosition::RearRight));
        assert_eq!(WheelPosition::from_index(4), None);
        assert_eq!(WheelPosition::from_index(100), None);
    }

    #[test]
    fn test_wheel_position_index_roundtrip() {
        for pos in [
            WheelPosition::FrontLeft,
            WheelPosition::FrontRight,
            WheelPosition::RearLeft,
            WheelPosition::RearRight,
        ] {
            let idx = pos.index();
            let recovered = WheelPosition::from_index(idx).unwrap();
            assert_eq!(recovered, pos);
        }
    }
}



