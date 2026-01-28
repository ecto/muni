//! Livox Mid360 LiDAR driver for bvr.
//!
//! Provides 3D point cloud data from a Livox Mid360 sensor over UDP.
//! The driver parses the Livox SDK2 protocol and produces point clouds
//! as `PointCloud` structs.
//!
//! The lidar can be controlled at runtime via SDK2 commands to start/stop
//! sampling. This allows power savings when the rover is idle.

use std::net::Ipv4Addr;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::{mpsc, watch};

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

/// Commands to control lidar sampling at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidarCommand {
    /// Start sampling (motor spins, data flows).
    Start,
    /// Stop sampling (motor stops, saves power).
    Stop,
    /// Trigger forced lens heating (3 min max, clears fog/ice).
    ForceHeat,
    /// Set detection sensitivity at runtime (0=normal, 1=sensitive).
    SetDetectMode(u8),
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
    /// Mounting pitch angle in degrees (positive = tilted down/forward)
    /// Used to transform points from LiDAR frame to rover body frame.
    pub mounting_pitch_deg: f32,
    /// Mounting height above ground in meters.
    /// Translates Z so ground ≈ 0 after pitch rotation.
    pub mounting_height_m: f32,
    /// Detection sensitivity: 0=normal, 1=sensitive (better for dark surfaces).
    pub detect_mode: u8,
    /// Enable lens heating (prevents fog/ice in winter).
    pub glass_heat: bool,
    /// Minimum detection range in cm (50-200, 0 = device default).
    pub blind_spot_cm: u16,
    /// Offload mounting pitch to lidar firmware (instead of software transform).
    pub use_hardware_transform: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lidar_ip: Ipv4Addr::new(192, 168, 1, 100),
            host_ip: Ipv4Addr::new(0, 0, 0, 0),
            point_cloud_port: 56301,
            imu_port: 56401,
            cmd_port: 56101,
            mounting_pitch_deg: 0.0,
            mounting_height_m: 0.0,
            detect_mode: 0,
            glass_heat: false,
            blind_spot_cm: 0,
            use_hardware_transform: false,
        }
    }
}

/// Runtime status from the LiDAR device (polled periodically).
#[derive(Debug, Clone, Copy, Default)]
pub struct LidarStatus {
    /// Internal core temperature (°C).
    pub core_temp_c: f32,
    /// Current work state (1=sampling, 2=idle, etc.).
    pub work_state: u8,
    /// Device replied to last query.
    pub responsive: bool,
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
    pub tag: u8,
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

    /// Spawn with IMU and runtime control.
    ///
    /// The command receiver allows starting/stopping sampling at runtime.
    /// When stopped, the lidar motor stops spinning (saving power and heat).
    /// The task keeps running and will resume when a Start command is received.
    ///
    /// If `start_immediately` is false, the lidar will wait for a Start command
    /// before beginning to sample.
    pub fn spawn_with_control(
        self,
        point_tx: watch::Sender<Option<PointCloud>>,
        imu_tx: watch::Sender<Option<ImuData>>,
        cmd_rx: mpsc::Receiver<LidarCommand>,
        start_immediately: bool,
        status_tx: watch::Sender<LidarStatus>,
    ) -> tokio::task::JoinHandle<Result<(), LidarError>> {
        let config = self.config;
        tokio::spawn(async move {
            driver::run_reader_with_control(
                config,
                point_tx,
                imu_tx,
                cmd_rx,
                start_immediately,
                status_tx,
            )
            .await
        })
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
}
