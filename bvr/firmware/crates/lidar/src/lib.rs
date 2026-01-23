//! Livox Mid360 LiDAR driver for bvr.
//!
//! Provides 3D point cloud data from a Livox Mid360 sensor over UDP.
//! The driver parses the Livox SDK2 protocol and produces point clouds
//! as `PointCloud` structs.

use std::net::Ipv4Addr;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::watch;

mod driver;

#[derive(Error, Debug)]
pub enum LidarError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Timeout waiting for data")]
    Timeout,
    #[error("Device not found")]
    NotFound,
}

/// Point cloud data format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PointDataType {
    /// 32-bit Cartesian coordinates (14 bytes/point, mm precision)
    #[default]
    Cartesian32 = 1,
    /// 16-bit Cartesian coordinates (8 bytes/point, 10mm precision)
    Cartesian16 = 2,
    /// Spherical coordinates (10 bytes/point)
    Spherical = 3,
}

/// Time synchronization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TimeSyncMode {
    /// No time synchronization (use device internal clock)
    #[default]
    None = 0,
    /// PTP time synchronization (IEEE 1588v2.0)
    Ptp = 1,
    /// GPS time synchronization
    Gps = 2,
}

/// Field of View configuration for a single scan region.
#[derive(Debug, Clone, Copy)]
pub struct FovRegion {
    /// Yaw start angle in degrees (-180 to 180)
    pub yaw_start: f32,
    /// Yaw stop angle in degrees (-180 to 180)
    pub yaw_stop: f32,
    /// Pitch start angle in degrees (-25 to 34 for Mid-360)
    pub pitch_start: f32,
    /// Pitch stop angle in degrees (-25 to 34 for Mid-360)
    pub pitch_stop: f32,
}

impl Default for FovRegion {
    fn default() -> Self {
        Self {
            yaw_start: -180.0,
            yaw_stop: 180.0,
            pitch_start: -25.0,
            pitch_stop: 34.0,
        }
    }
}

/// Livox Mid360 configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// LiDAR IP address (default: 192.168.1.1xx based on SN)
    pub lidar_ip: Ipv4Addr,
    /// Host IP address to bind (default: 0.0.0.0)
    pub host_ip: Ipv4Addr,
    /// Point cloud data port (default: 56301)
    pub point_cloud_port: u16,
    /// IMU data port (default: 56401, 0 to disable)
    pub imu_port: u16,
    /// Command port for control messages (default: 56101)
    pub cmd_port: u16,
    /// Point cloud data format (default: Cartesian32)
    pub data_type: PointDataType,
    /// Time synchronization mode (default: None)
    pub time_sync: TimeSyncMode,
    /// Heartbeat interval in milliseconds (0 to disable, default: 1000)
    pub heartbeat_interval_ms: u32,
    /// FOV region 1 (optional, None = full FOV)
    pub fov_region1: Option<FovRegion>,
    /// FOV region 2 (optional, None = disabled)
    pub fov_region2: Option<FovRegion>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lidar_ip: Ipv4Addr::new(192, 168, 1, 100),
            host_ip: Ipv4Addr::new(0, 0, 0, 0),
            point_cloud_port: 56301,
            imu_port: 56401,
            cmd_port: 56101,
            data_type: PointDataType::default(),
            time_sync: TimeSyncMode::default(),
            heartbeat_interval_ms: 1000,
            fov_region1: None,
            fov_region2: None,
        }
    }
}

/// Point confidence level extracted from tag field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PointConfidence {
    /// Normal detection, high confidence
    High = 0,
    /// Reduced confidence
    Medium = 1,
    /// Low confidence (possible interference)
    Low = 2,
    /// Very low confidence (likely noise)
    VeryLow = 3,
}

impl From<u8> for PointConfidence {
    fn from(val: u8) -> Self {
        match val & 0b11 {
            0 => PointConfidence::High,
            1 => PointConfidence::Medium,
            2 => PointConfidence::Low,
            _ => PointConfidence::VeryLow,
        }
    }
}

/// Return type information extracted from tag field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReturnType {
    /// Single return (normal)
    Single = 0,
    /// First of multiple returns
    FirstMulti = 1,
    /// Second of multiple returns
    SecondMulti = 2,
    /// Third or later return
    ThirdMulti = 3,
}

impl From<u8> for ReturnType {
    fn from(val: u8) -> Self {
        match val & 0b11 {
            0 => ReturnType::Single,
            1 => ReturnType::FirstMulti,
            2 => ReturnType::SecondMulti,
            _ => ReturnType::ThirdMulti,
        }
    }
}

/// A 3D point from the LiDAR.
#[derive(Debug, Clone, Copy, Default)]
pub struct Point3D {
    /// X coordinate in meters (forward)
    pub x: f32,
    /// Y coordinate in meters (left)
    pub y: f32,
    /// Z coordinate in meters (up)
    pub z: f32,
    /// Reflectivity (0-255)
    pub reflectivity: u8,
    /// Point tag (classification flags)
    /// - Bits 0-1: Confidence (0=high, 3=very low)
    /// - Bits 2-3: Return type (0=single, 1-3=multi-return index)
    /// - Bits 4-5: Interference flags (rain/fog/dust/crosstalk)
    /// - Bits 6-7: Reserved
    pub tag: u8,
}

impl Point3D {
    /// Get confidence level of this point detection.
    #[inline]
    pub fn confidence(&self) -> PointConfidence {
        PointConfidence::from(self.tag & 0b11)
    }

    /// Get return type (single vs multi-return).
    #[inline]
    pub fn return_type(&self) -> ReturnType {
        ReturnType::from((self.tag >> 2) & 0b11)
    }

    /// Check if interference was detected (rain, fog, dust, or crosstalk).
    #[inline]
    pub fn has_interference(&self) -> bool {
        (self.tag >> 4) & 0b11 != 0
    }

    /// Check if this is a high-quality point (high confidence, no interference).
    #[inline]
    pub fn is_high_quality(&self) -> bool {
        self.confidence() == PointConfidence::High && !self.has_interference()
    }

    /// Check if this point should be used for navigation (medium+ confidence, no interference).
    #[inline]
    pub fn is_navigation_quality(&self) -> bool {
        matches!(self.confidence(), PointConfidence::High | PointConfidence::Medium)
            && !self.has_interference()
    }
}

/// A complete point cloud frame from the LiDAR.
#[derive(Debug, Clone)]
pub struct PointCloud {
    /// Timestamp when frame was captured
    pub timestamp: Instant,
    /// Nanosecond timestamp from LiDAR (GPS synced if available)
    pub timestamp_ns: u64,
    /// Frame sequence number
    pub frame_id: u32,
    /// Point cloud data
    pub points: Vec<Point3D>,
}

impl Default for PointCloud {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            timestamp_ns: 0,
            frame_id: 0,
            points: Vec::new(),
        }
    }
}

/// IMU data from the Mid360.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImuData {
    /// Timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// Gyroscope X (rad/s)
    pub gyro_x: f32,
    /// Gyroscope Y (rad/s)
    pub gyro_y: f32,
    /// Gyroscope Z (rad/s)
    pub gyro_z: f32,
    /// Accelerometer X (m/s²)
    pub accel_x: f32,
    /// Accelerometer Y (m/s²)
    pub accel_y: f32,
    /// Accelerometer Z (m/s²)
    pub accel_z: f32,
}

/// LiDAR reader that receives Livox Mid360 point clouds over UDP.
pub struct LidarReader {
    config: Config,
}

impl LidarReader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Spawn the LiDAR reader as an async task.
    ///
    /// Returns a handle that can be used to abort the task.
    pub fn spawn(
        self,
        tx: watch::Sender<Option<PointCloud>>,
    ) -> tokio::task::JoinHandle<Result<(), LidarError>> {
        let config = self.config;
        tokio::spawn(async move { driver::run_reader(config, tx).await })
    }

    /// Spawn with IMU data channel.
    pub fn spawn_with_imu(
        self,
        point_tx: watch::Sender<Option<PointCloud>>,
        imu_tx: watch::Sender<Option<ImuData>>,
    ) -> tokio::task::JoinHandle<Result<(), LidarError>> {
        let config = self.config;
        tokio::spawn(async move { driver::run_reader_with_imu(config, point_tx, imu_tx).await })
    }
}

// Re-export for backwards compatibility with code expecting LaserScan
// (the recording crate uses this)
pub use PointCloud as LaserScan;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.lidar_ip, Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(config.point_cloud_port, 56301);
    }

    #[test]
    fn test_point_cloud_default() {
        let cloud = PointCloud::default();
        assert_eq!(cloud.points.len(), 0);
        assert_eq!(cloud.frame_id, 0);
    }

    #[test]
    fn test_point3d_default() {
        let point = Point3D::default();
        assert_eq!(point.x, 0.0);
        assert_eq!(point.y, 0.0);
        assert_eq!(point.z, 0.0);
        assert_eq!(point.reflectivity, 0);
    }

    #[test]
    fn test_point_confidence() {
        // Tag = 0b00 -> High confidence
        let point = Point3D { tag: 0b00, ..Default::default() };
        assert_eq!(point.confidence(), PointConfidence::High);

        // Tag = 0b01 -> Medium confidence
        let point = Point3D { tag: 0b01, ..Default::default() };
        assert_eq!(point.confidence(), PointConfidence::Medium);

        // Tag = 0b10 -> Low confidence
        let point = Point3D { tag: 0b10, ..Default::default() };
        assert_eq!(point.confidence(), PointConfidence::Low);

        // Tag = 0b11 -> Very low confidence
        let point = Point3D { tag: 0b11, ..Default::default() };
        assert_eq!(point.confidence(), PointConfidence::VeryLow);
    }

    #[test]
    fn test_point_return_type() {
        // Bits 2-3 = 0b00 -> Single return
        let point = Point3D { tag: 0b0000, ..Default::default() };
        assert_eq!(point.return_type(), ReturnType::Single);

        // Bits 2-3 = 0b01 -> First multi
        let point = Point3D { tag: 0b0100, ..Default::default() };
        assert_eq!(point.return_type(), ReturnType::FirstMulti);

        // Bits 2-3 = 0b10 -> Second multi
        let point = Point3D { tag: 0b1000, ..Default::default() };
        assert_eq!(point.return_type(), ReturnType::SecondMulti);
    }

    #[test]
    fn test_point_interference() {
        // No interference (bits 4-5 = 0)
        let point = Point3D { tag: 0b00_00_00, ..Default::default() };
        assert!(!point.has_interference());

        // Interference detected (bits 4-5 != 0)
        let point = Point3D { tag: 0b01_00_00, ..Default::default() };
        assert!(point.has_interference());

        let point = Point3D { tag: 0b10_00_00, ..Default::default() };
        assert!(point.has_interference());
    }

    #[test]
    fn test_point_quality() {
        // High quality: confidence=high, no interference
        let point = Point3D { tag: 0b00_00_00, ..Default::default() };
        assert!(point.is_high_quality());
        assert!(point.is_navigation_quality());

        // Medium confidence, no interference -> nav quality but not high quality
        let point = Point3D { tag: 0b00_00_01, ..Default::default() };
        assert!(!point.is_high_quality());
        assert!(point.is_navigation_quality());

        // High confidence but interference -> not high quality, not nav quality
        let point = Point3D { tag: 0b01_00_00, ..Default::default() };
        assert!(!point.is_high_quality());
        assert!(!point.is_navigation_quality());

        // Low confidence -> not nav quality
        let point = Point3D { tag: 0b00_00_10, ..Default::default() };
        assert!(!point.is_navigation_quality());
    }
}
