//! Telemetry recording for bvr using Rerun.
//!
//! This crate provides session-based recording of robot telemetry for later playback
//! and analysis. Data is stored in `.rrd` files that can be viewed with the Rerun Viewer.
//!
//! # Architecture
//!
//! ```text
//! bvrd control loop (100Hz)
//!         │
//!         ▼
//!    Recorder::log_*()
//!         │
//!         ▼
//!    RecordingStream (async, buffered)
//!         │
//!         ▼
//!    /var/log/bvr/sessions/{rover_id}_{timestamp}.rrd
//! ```
//!
//! # Usage
//!
//! ```ignore
//! let config = Config::default();
//! let recorder = Recorder::new(&config)?;
//!
//! // In control loop:
//! recorder.set_time(timestamp_secs);
//! recorder.log_pose(&pose)?;
//! recorder.log_velocity(commanded, actual)?;
//! recorder.log_motors(&currents, &temps)?;
//! ```

use rerun::{RecordingStream, RecordingStreamBuilder};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info};
use types::{Mode, Pose, Twist};

#[derive(Error, Debug)]
pub enum RecordingError {
    #[error("Rerun error: {0}")]
    Rerun(#[from] rerun::RecordingStreamError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Recording not active")]
    NotActive,
}

/// Configuration for the recorder.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory to store session recordings.
    pub session_dir: PathBuf,
    /// Unique identifier for this rover.
    pub rover_id: String,
    /// Maximum storage in bytes before rotating old sessions.
    pub max_storage_bytes: u64,
    /// Whether to include camera frames (higher bandwidth).
    pub include_camera: bool,
    /// Whether recording is enabled.
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            session_dir: PathBuf::from("/var/log/bvr/sessions"),
            rover_id: "bvr-01".to_string(),
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            include_camera: false,
            enabled: true,
        }
    }
}

/// Telemetry recorder using Rerun.
///
/// Handles session lifecycle, data logging, and storage management.
pub struct Recorder {
    config: Config,
    stream: Option<RecordingStream>,
    session_path: Option<PathBuf>,
}

impl Recorder {
    /// Create a new recorder and start a session.
    pub fn new(config: &Config) -> Result<Self, RecordingError> {
        if !config.enabled {
            info!("Recording disabled by config");
            return Ok(Self {
                config: config.clone(),
                stream: None,
                session_path: None,
            });
        }

        // Ensure session directory exists
        std::fs::create_dir_all(&config.session_dir)?;

        // Rotate old sessions if needed
        Self::rotate_sessions_if_needed(&config.session_dir, config.max_storage_bytes)?;

        // Generate session filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let session_name = format!("{}_{}", config.rover_id, timestamp);
        let session_path = config.session_dir.join(format!("{}.rrd", session_name));

        info!(path = %session_path.display(), "Starting recording session");

        // Create recording stream
        let stream = RecordingStreamBuilder::new(session_name.clone())
            .save(&session_path)?;

        // Log initial metadata
        Self::log_session_metadata(&stream, config)?;

        Ok(Self {
            config: config.clone(),
            stream: Some(stream),
            session_path: Some(session_path),
        })
    }

    /// Create a disabled recorder (no-op for all operations).
    pub fn disabled() -> Self {
        Self {
            config: Config {
                enabled: false,
                ..Default::default()
            },
            stream: None,
            session_path: None,
        }
    }

    /// Check if recording is active.
    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }

    /// Get the current session file path.
    pub fn session_path(&self) -> Option<&Path> {
        self.session_path.as_deref()
    }

    /// Set the current timestamp for subsequent logs.
    ///
    /// Call this once per control loop iteration.
    pub fn set_time(&self, time_secs: f64) {
        if let Some(ref stream) = self.stream {
            stream.set_time_seconds("time", time_secs);
        }
    }

    /// Log robot pose (position and heading).
    pub fn log_pose(&self, pose: &Pose) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Log as 3D transform for visualization
        stream.log(
            "robot/pose",
            &rerun::Transform3D::from_translation_rotation(
                [pose.x as f32, pose.y as f32, 0.0],
                rerun::RotationAxisAngle::new(
                    [0.0, 0.0, 1.0],
                    rerun::Angle::from_radians(pose.theta as f32),
                ),
            ),
        )?;

        // Also log as 2D point for trajectory visualization
        stream.log(
            "robot/trajectory",
            &rerun::Points2D::new([(pose.x as f32, pose.y as f32)]),
        )?;

        // Log heading as scalar for time-series
        stream.log("robot/heading", &rerun::Scalar::new(pose.theta))?;

        Ok(())
    }

    /// Log commanded and actual velocity.
    pub fn log_velocity(&self, commanded: &Twist, actual: &Twist) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Linear velocity comparison
        stream.log(
            "velocity/linear/commanded",
            &rerun::Scalar::new(commanded.linear),
        )?;
        stream.log("velocity/linear/actual", &rerun::Scalar::new(actual.linear))?;

        // Angular velocity comparison
        stream.log(
            "velocity/angular/commanded",
            &rerun::Scalar::new(commanded.angular),
        )?;
        stream.log(
            "velocity/angular/actual",
            &rerun::Scalar::new(actual.angular),
        )?;

        Ok(())
    }

    /// Log motor telemetry (currents and temperatures).
    pub fn log_motors(&self, currents: &[f32; 4], temps: &[f32; 4]) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Per-motor currents
        stream.log("motors/fl/current", &rerun::Scalar::new(currents[0] as f64))?;
        stream.log("motors/fr/current", &rerun::Scalar::new(currents[1] as f64))?;
        stream.log("motors/rl/current", &rerun::Scalar::new(currents[2] as f64))?;
        stream.log("motors/rr/current", &rerun::Scalar::new(currents[3] as f64))?;

        // Per-motor temperatures
        stream.log("motors/fl/temp", &rerun::Scalar::new(temps[0] as f64))?;
        stream.log("motors/fr/temp", &rerun::Scalar::new(temps[1] as f64))?;
        stream.log("motors/rl/temp", &rerun::Scalar::new(temps[2] as f64))?;
        stream.log("motors/rr/temp", &rerun::Scalar::new(temps[3] as f64))?;

        // Total current
        let total_current: f32 = currents.iter().sum();
        stream.log("motors/total_current", &rerun::Scalar::new(total_current as f64))?;

        Ok(())
    }

    /// Log power status.
    pub fn log_power(&self, voltage: f64, current: f64) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        stream.log("power/battery_voltage", &rerun::Scalar::new(voltage))?;
        stream.log("power/system_current", &rerun::Scalar::new(current))?;
        stream.log("power/power_watts", &rerun::Scalar::new(voltage * current))?;

        Ok(())
    }

    /// Log operating mode change.
    pub fn log_mode(&self, mode: Mode) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        let mode_str = match mode {
            Mode::Disabled => "Disabled",
            Mode::Idle => "Idle",
            Mode::Teleop => "Teleop",
            Mode::Autonomous => "Autonomous",
            Mode::EStop => "EStop",
            Mode::Fault => "Fault",
        };

        stream.log(
            "state/mode",
            &rerun::TextLog::new(mode_str).with_level(rerun::TextLogLevel::INFO),
        )?;

        Ok(())
    }

    /// Log GPS coordinates.
    pub fn log_gps(&self, lat: f64, lon: f64, accuracy: f32) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Log as 2D point (lon, lat for standard map orientation)
        stream.log("gps/position", &rerun::Points2D::new([(lon as f32, lat as f32)]))?;

        stream.log("gps/latitude", &rerun::Scalar::new(lat))?;
        stream.log("gps/longitude", &rerun::Scalar::new(lon))?;
        stream.log("gps/accuracy", &rerun::Scalar::new(accuracy as f64))?;

        Ok(())
    }

    /// Log a camera frame (JPEG encoded).
    ///
    /// Only logs if `include_camera` is enabled in config.
    pub fn log_camera_jpeg(
        &self,
        entity: &str,
        data: &[u8],
        _width: u32,
        _height: u32,
    ) -> Result<(), RecordingError> {
        if !self.config.include_camera {
            return Ok(());
        }

        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        stream.log(
            entity,
            &rerun::EncodedImage::from_file_contents(data.to_vec()),
        )?;

        Ok(())
    }

    /// Log a text annotation (for events, warnings, etc.).
    pub fn log_event(&self, message: &str, level: EventLevel) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        let rerun_level = match level {
            EventLevel::Debug => rerun::TextLogLevel::DEBUG,
            EventLevel::Info => rerun::TextLogLevel::INFO,
            EventLevel::Warn => rerun::TextLogLevel::WARN,
            EventLevel::Error => rerun::TextLogLevel::ERROR,
        };

        stream.log("events", &rerun::TextLog::new(message).with_level(rerun_level))?;

        Ok(())
    }

    /// Log wheel odometry deltas.
    pub fn log_odometry(&self, dx: f64, dy: f64, dtheta: f64) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        stream.log("odometry/dx", &rerun::Scalar::new(dx))?;
        stream.log("odometry/dy", &rerun::Scalar::new(dy))?;
        stream.log("odometry/dtheta", &rerun::Scalar::new(dtheta))?;

        Ok(())
    }

    /// Log tool state.
    pub fn log_tool(
        &self,
        name: &str,
        position: Option<f32>,
        current: Option<f32>,
    ) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        if let Some(pos) = position {
            stream.log(
                format!("tools/{}/position", name),
                &rerun::Scalar::new(pos as f64),
            )?;
        }

        if let Some(cur) = current {
            stream.log(
                format!("tools/{}/current", name),
                &rerun::Scalar::new(cur as f64),
            )?;
        }

        Ok(())
    }

    /// End the current session.
    pub fn end_session(&mut self) {
        if let Some(path) = self.session_path.take() {
            info!(path = %path.display(), "Ending recording session");
        }
        self.stream = None;
    }

    /// Log session metadata at start.
    fn log_session_metadata(stream: &RecordingStream, config: &Config) -> Result<(), RecordingError> {
        stream.log_static(
            "session/rover_id",
            &rerun::TextLog::new(config.rover_id.clone()),
        )?;

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        stream.log_static(
            "session/start_time",
            &rerun::TextLog::new(format!("{}", start_time)),
        )?;

        Ok(())
    }

    /// Rotate old sessions if storage limit exceeded.
    fn rotate_sessions_if_needed(session_dir: &Path, max_bytes: u64) -> Result<(), RecordingError> {
        let mut sessions: Vec<_> = std::fs::read_dir(session_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "rrd")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time (oldest first)
        sessions.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

        // Calculate total size
        let total_size: u64 = sessions
            .iter()
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();

        if total_size <= max_bytes {
            return Ok(());
        }

        // Delete oldest sessions until under limit
        let mut current_size = total_size;
        for entry in sessions {
            if current_size <= max_bytes {
                break;
            }

            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                if std::fs::remove_file(&path).is_ok() {
                    debug!(path = %path.display(), size, "Rotated old session");
                    current_size -= size;
                }
            }
        }

        Ok(())
    }
}

/// Event severity level.
#[derive(Debug, Clone, Copy)]
pub enum EventLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_recorder() {
        let recorder = Recorder::disabled();
        assert!(!recorder.is_active());

        // All operations should be no-ops
        recorder.set_time(0.0);
        assert!(recorder.log_pose(&Pose::default()).is_ok());
    }
}
