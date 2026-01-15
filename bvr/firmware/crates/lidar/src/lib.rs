//! RPLidar A1 driver for bvr.
//!
//! Provides laser scan data from a serial-connected RPLidar A1 sensor.
//! The driver parses the binary protocol and produces 360-degree scans
//! as `LaserScan` structs.

use std::time::Instant;
use thiserror::Error;
use tokio::sync::watch;
use tracing::error;

mod driver;

#[derive(Error, Debug)]
pub enum LidarError {
    #[error("Serial port error: {0}")]
    Serial(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Timeout waiting for data")]
    Timeout,
}

/// RPLidar A1 configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Serial port path (e.g., "/dev/ttyUSB0", "/dev/ttyUSB1")
    pub port: String,
    /// Baud rate (typically 115200 for RPLidar A1)
    pub baud_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: "/dev/ttyUSB0".into(),
            baud_rate: 115200,
        }
    }
}

/// A complete 360-degree laser scan.
#[derive(Debug, Clone)]
pub struct LaserScan {
    /// Timestamp when scan was captured
    pub timestamp: Instant,
    /// Angular resolution (radians between points)
    pub angle_increment: f32,
    /// Minimum range (meters, typically 0.2m for RPLidar A1)
    pub range_min: f32,
    /// Maximum range (meters, typically 12m for RPLidar A1)
    pub range_max: f32,
    /// Range measurements (meters), indexed by angle
    /// Index 0 = 0°, increases counter-clockwise
    pub ranges: Vec<f32>,
    /// Intensity values (0-255), same length as ranges
    pub intensities: Vec<u8>,
}

impl Default for LaserScan {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            angle_increment: 0.0,
            range_min: 0.2,
            range_max: 12.0,
            ranges: Vec::new(),
            intensities: Vec::new(),
        }
    }
}

/// LiDAR reader that parses RPLidar A1 packets from a serial port.
pub struct LidarReader {
    config: Config,
}

impl LidarReader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the LiDAR reader, sending scan updates to the provided channel.
    ///
    /// This spawns a blocking thread for serial I/O.
    pub fn spawn(
        self,
        tx: watch::Sender<Option<LaserScan>>,
    ) -> Result<std::thread::JoinHandle<()>, LidarError> {
        let config = self.config.clone();

        let handle = std::thread::spawn(move || {
            if let Err(e) = driver::run_reader(config, tx) {
                error!(?e, "LiDAR reader error");
            }
        });

        Ok(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.port, "/dev/ttyUSB0");
        assert_eq!(config.baud_rate, 115200);
    }

    #[test]
    fn test_laser_scan_default() {
        let scan = LaserScan::default();
        assert_eq!(scan.range_min, 0.2);
        assert_eq!(scan.range_max, 12.0);
        assert_eq!(scan.ranges.len(), 0);
        assert_eq!(scan.intensities.len(), 0);
    }
}
