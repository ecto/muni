//! Shared types and message definitions for bvr.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

/// Velocity command: linear (m/s) and angular (rad/s).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
pub enum Mode {
    /// Legacy wire-compat value (byte 0). New firmware never enters this state;
    /// kept so old consoles that send `SetMode(0)` still deserialise correctly.
    Disabled,
    /// Ready to receive commands but not moving (boot state)
    #[default]
    Idle,
    /// Actively executing velocity commands from teleop
    Teleop,
    /// Autonomous operation
    Autonomous,
    /// Emergency stop triggered
    EStop,
    /// Fault condition
    Fault,
    /// Sleep mode: sensors powered down but rover stays reachable
    Sleep,
    /// Dance demo mode: choreographed motion sequence
    Dance,
}

/// Power system status.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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

/// SLAM system status for telemetry.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
pub struct SlamStatus {
    /// Current pose from SLAM (world frame)
    pub pose: Pose,
    /// Match confidence (0.0-1.0)
    pub confidence: f32,
    /// Number of keyframes in pose graph
    pub keyframe_count: u32,
    /// Number of loop closures detected
    pub loop_closure_count: u32,
    /// Whether SLAM is actively mapping
    pub mapping_active: bool,
}

/// Wheel/VESC status values.
pub mod wheel_status {
    /// VESC is offline (not responding on CAN)
    pub const OFFLINE: u8 = 0;
    /// VESC is online but degraded (e.g., motor not connected)
    pub const DEGRADED: u8 = 1;
    /// VESC is fully online and operational
    pub const ONLINE: u8 = 2;
}

/// Subsystem health flags for telemetry.
///
/// Each flag indicates whether a subsystem is operating normally.
/// These are encoded as 2 bytes (16 bits) in the binary telemetry protocol.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
pub struct SubsystemHealth {
    /// CAN bus communication is healthy (heartbeats received from all VESCs)
    pub can_healthy: bool,
    /// Recording subsystem is active and writing data
    pub recording_active: bool,
    /// GPS has a valid fix (RTK fixed or float)
    pub gps_fix: bool,
    /// Camera is streaming frames
    pub camera_active: bool,
    /// Connected to dispatch service
    pub dispatch_connected: bool,
    /// Registered with discovery service
    pub discovery_connected: bool,
    /// LiDAR sensor is active and providing data
    pub lidar_active: bool,
    /// SLAM/localization is running
    pub slam_running: bool,
    /// Wheel status: 0=offline, 1=degraded, 2=online (2 bits each in bits 8-15)
    pub wheel_status: [u8; 4],
}

impl SubsystemHealth {
    /// Encode health flags as a u16 bitmask.
    pub fn to_bits(&self) -> u16 {
        let mut bits = 0u16;
        if self.can_healthy {
            bits |= 1 << 0;
        }
        if self.recording_active {
            bits |= 1 << 1;
        }
        if self.gps_fix {
            bits |= 1 << 2;
        }
        if self.camera_active {
            bits |= 1 << 3;
        }
        if self.dispatch_connected {
            bits |= 1 << 4;
        }
        if self.discovery_connected {
            bits |= 1 << 5;
        }
        if self.lidar_active {
            bits |= 1 << 6;
        }
        if self.slam_running {
            bits |= 1 << 7;
        }
        // Pack wheel status: 2 bits each for FL, FR, RL, RR (bits 8-15)
        bits |= (self.wheel_status[0] as u16 & 0x3) << 8;
        bits |= (self.wheel_status[1] as u16 & 0x3) << 10;
        bits |= (self.wheel_status[2] as u16 & 0x3) << 12;
        bits |= (self.wheel_status[3] as u16 & 0x3) << 14;
        bits
    }

    /// Decode health flags from a u16 bitmask.
    pub fn from_bits(bits: u16) -> Self {
        Self {
            can_healthy: bits & (1 << 0) != 0,
            recording_active: bits & (1 << 1) != 0,
            gps_fix: bits & (1 << 2) != 0,
            camera_active: bits & (1 << 3) != 0,
            dispatch_connected: bits & (1 << 4) != 0,
            discovery_connected: bits & (1 << 5) != 0,
            lidar_active: bits & (1 << 6) != 0,
            slam_running: bits & (1 << 7) != 0,
            wheel_status: [
                ((bits >> 8) & 0x3) as u8,
                ((bits >> 10) & 0x3) as u8,
                ((bits >> 12) & 0x3) as u8,
                ((bits >> 14) & 0x3) as u8,
            ],
        }
    }

    /// Check if all critical subsystems are healthy.
    pub fn is_operational(&self) -> bool {
        self.can_healthy
    }

    /// Count how many subsystems are healthy.
    pub fn healthy_count(&self) -> u8 {
        self.to_bits().count_ones() as u8
    }
}

/// Command from operator/autonomy to rover.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
    /// Toggle LiDAR streaming (true = enable, false = disable)
    LidarToggle(bool),
    /// Set navigation goal (click-to-navigate from console)
    SetGoal { x: f64, y: f64 },
}

/// Command for the active tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "generated/"))]
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
            Mode::Sleep,
            Mode::Dance,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let decoded: Mode = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, mode);
        }
    }

    #[test]
    fn test_mode_default() {
        assert_eq!(Mode::default(), Mode::Idle);
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

    #[test]
    fn test_subsystem_health_default() {
        let health = SubsystemHealth::default();
        assert!(!health.can_healthy);
        assert!(!health.recording_active);
        assert!(!health.gps_fix);
        assert!(!health.camera_active);
        assert!(!health.dispatch_connected);
        assert!(!health.discovery_connected);
        assert!(!health.lidar_active);
        assert!(!health.slam_running);
        assert_eq!(health.wheel_status, [0, 0, 0, 0]);
        assert_eq!(health.to_bits(), 0);
        assert_eq!(health.healthy_count(), 0);
    }

    #[test]
    fn test_subsystem_health_to_bits() {
        let health = SubsystemHealth {
            can_healthy: true,
            recording_active: false,
            gps_fix: true,
            camera_active: false,
            dispatch_connected: true,
            discovery_connected: false,
            lidar_active: true,
            slam_running: false,
            // FL=online(2), FR=offline(0), RL=degraded(1), RR=online(2)
            wheel_status: [2, 0, 1, 2],
        };
        // Bits 0-7: 0b01010101 = 0x55
        // Bits 8-9: FL=2 (0b10), Bits 10-11: FR=0 (0b00), Bits 12-13: RL=1 (0b01), Bits 14-15: RR=2 (0b10)
        // Bits 8-15: 0b10_01_00_10 = 0x9200
        // Combined: 0x9255
        assert_eq!(health.to_bits(), 0x9255);
        // healthy_count only counts set bits (not wheel status values)
        // Bits 0-7 have 4 set, plus wheel bits: 10_01_00_10 has 3 more = 7
        assert_eq!(health.healthy_count(), 7);
    }

    #[test]
    fn test_subsystem_health_from_bits() {
        // All bool flags set, all wheels online (status=2=0b10 each)
        // Bits 8-15: 0b10_10_10_10 = 0xAA
        let bits = 0xAAFFu16;
        let health = SubsystemHealth::from_bits(bits);
        assert!(health.can_healthy);
        assert!(health.recording_active);
        assert!(health.gps_fix);
        assert!(health.camera_active);
        assert!(health.dispatch_connected);
        assert!(health.discovery_connected);
        assert!(health.lidar_active);
        assert!(health.slam_running);
        assert_eq!(health.wheel_status, [2, 2, 2, 2]); // All online
        // healthy_count counts set bits: 0xAAFF = 0b1010_1010_1111_1111 = 12 bits
        assert_eq!(health.healthy_count(), 12);
    }

    #[test]
    fn test_subsystem_health_roundtrip() {
        let original = SubsystemHealth {
            can_healthy: true,
            recording_active: true,
            gps_fix: false,
            camera_active: true,
            dispatch_connected: false,
            discovery_connected: true,
            lidar_active: false,
            slam_running: true,
            wheel_status: [2, 0, 1, 2], // online, offline, degraded, online
        };
        let bits = original.to_bits();
        let decoded = SubsystemHealth::from_bits(bits);
        assert_eq!(decoded.can_healthy, original.can_healthy);
        assert_eq!(decoded.recording_active, original.recording_active);
        assert_eq!(decoded.gps_fix, original.gps_fix);
        assert_eq!(decoded.camera_active, original.camera_active);
        assert_eq!(decoded.dispatch_connected, original.dispatch_connected);
        assert_eq!(decoded.discovery_connected, original.discovery_connected);
        assert_eq!(decoded.lidar_active, original.lidar_active);
        assert_eq!(decoded.slam_running, original.slam_running);
        assert_eq!(decoded.wheel_status, original.wheel_status);
    }

    #[test]
    fn test_subsystem_health_is_operational() {
        let mut health = SubsystemHealth::default();
        assert!(!health.is_operational());

        health.can_healthy = true;
        assert!(health.is_operational());
    }

    #[test]
    fn test_subsystem_health_serde() {
        let health = SubsystemHealth {
            can_healthy: true,
            recording_active: true,
            gps_fix: true,
            camera_active: false,
            dispatch_connected: true,
            discovery_connected: true,
            lidar_active: false,
            slam_running: true,
            wheel_status: [2, 1, 0, 2],
        };
        let json = serde_json::to_string(&health).unwrap();
        let decoded: SubsystemHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.can_healthy, health.can_healthy);
        assert_eq!(decoded.recording_active, health.recording_active);
        assert_eq!(decoded.gps_fix, health.gps_fix);
        assert_eq!(decoded.slam_running, health.slam_running);
        assert_eq!(decoded.wheel_status, health.wheel_status);
    }
}



