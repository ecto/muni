//! Telemetry recording for bvr using Rerun.
//!
//! This crate provides session-based recording of robot telemetry for later playback
//! and analysis. All data is stored in a single `.rrd` file that can be viewed with
//! the Rerun Viewer or embedded in the operator web app.
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
//!    /var/log/bvr/sessions/{timestamp}/
//!         ├── metadata.json     <- Session metadata for quick lookups
//!         └── session.rrd       <- All data: telemetry, camera, LiDAR
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
//! recorder.log_camera_frame(jpeg_data)?;
//! recorder.log_lidar_points(&points)?;
//!
//! // At end of session:
//! recorder.end_session();  // Writes metadata.json
//! ```

use chrono::{DateTime, Utc};
use rerun::{RecordingStream, RecordingStreamBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info, warn};
use types::{Mode, Pose, Twist};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RecordingError {
    #[error("Rerun error: {0}")]
    Rerun(#[from] rerun::RecordingStreamError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Recording not active")]
    NotActive,
}

// =============================================================================
// Session Metadata (shared with depot services)
// =============================================================================

/// GPS bounding box for the session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpsBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GpsBounds {
    /// Expand bounds to include a new point
    pub fn expand(&mut self, lat: f64, lon: f64) {
        if self.min_lat == 0.0 && self.max_lat == 0.0 {
            // First point
            self.min_lat = lat;
            self.max_lat = lat;
            self.min_lon = lon;
            self.max_lon = lon;
        } else {
            self.min_lat = self.min_lat.min(lat);
            self.max_lat = self.max_lat.max(lat);
            self.min_lon = self.min_lon.min(lon);
            self.max_lon = self.max_lon.max(lon);
        }
    }

    /// Check if bounds are valid (have been expanded at least once)
    pub fn is_valid(&self) -> bool {
        self.min_lat != 0.0 || self.max_lat != 0.0 || self.min_lon != 0.0 || self.max_lon != 0.0
    }
}

/// Session metadata written to metadata.json
///
/// This provides quick lookups for the session list without parsing the RRD file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Unique session identifier
    pub session_id: Uuid,
    /// Rover identifier
    pub rover_id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session end time (None if still recording)
    pub ended_at: Option<DateTime<Utc>>,
    /// Duration in seconds (computed from start/end)
    #[serde(default)]
    pub duration_secs: f64,
    /// GPS bounds covered by this session
    pub gps_bounds: Option<GpsBounds>,
    /// Number of LiDAR frames recorded
    pub lidar_frames: u32,
    /// Number of camera frames recorded
    pub camera_frames: u32,
    /// Number of pose samples recorded
    #[serde(default)]
    pub pose_samples: u32,
    /// Name of the session file (always "session.rrd")
    pub session_file: String,
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
    /// Whether to include camera frames.
    pub include_camera: bool,
    /// Whether to include LiDAR point clouds.
    pub include_lidar: bool,
    /// LiDAR recording rate in Hz (point clouds are large, typically 2-10 Hz).
    pub lidar_hz: f32,
    /// Camera recording rate in Hz (typically 2-5 Hz for mapping).
    pub camera_hz: f32,
    /// Whether recording is enabled.
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            session_dir: PathBuf::from("/var/log/bvr/sessions"),
            rover_id: "bvr-01".to_string(),
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            include_camera: true,
            include_lidar: true,
            lidar_hz: 10.0,
            camera_hz: 2.0,
            enabled: true,
        }
    }
}

/// Telemetry recorder using Rerun.
///
/// All session data is recorded to a single .rrd file for easy playback.
pub struct Recorder {
    config: Config,
    stream: Option<RecordingStream>,
    /// Root directory for this session
    session_dir: Option<PathBuf>,
    /// Path to session.rrd file
    session_path: Option<PathBuf>,
    /// Session metadata (written at end)
    metadata: Option<SessionMetadata>,
    /// GPS bounds accumulator
    gps_bounds: GpsBounds,
    /// LiDAR frame counter
    lidar_frame_count: AtomicU32,
    /// Camera frame counter
    camera_frame_count: AtomicU32,
    /// Pose sample counter
    pose_count: AtomicU32,
    /// Last LiDAR frame time (for rate limiting)
    last_lidar_time: f64,
    /// Last camera frame time (for rate limiting)
    last_camera_time: f64,
}

impl Recorder {
    /// Create a new recorder and start a session.
    pub fn new(config: &Config) -> Result<Self, RecordingError> {
        if !config.enabled {
            info!("Recording disabled by config");
            return Ok(Self::disabled());
        }

        // Ensure session directory exists
        std::fs::create_dir_all(&config.session_dir)?;

        // Rotate old sessions if needed
        Self::rotate_sessions_if_needed(&config.session_dir, config.max_storage_bytes)?;

        // Generate session directory name using ISO timestamp
        let now = Utc::now();
        let session_name = now.format("%Y-%m-%dT%H-%M-%S").to_string();
        let session_dir = config.session_dir.join(&session_name);
        std::fs::create_dir_all(&session_dir)?;

        let session_id = Uuid::new_v4();
        info!(
            path = %session_dir.display(),
            id = %session_id,
            "Starting recording session"
        );

        // Create session.rrd file (single unified file)
        let session_file = "session.rrd".to_string();
        let session_path = session_dir.join(&session_file);
        let stream = RecordingStreamBuilder::new(format!("{}-{}", config.rover_id, session_name))
            .save(&session_path)?;

        // Log initial metadata to Rerun
        Self::log_session_metadata_rerun(&stream, config, &session_id)?;

        // Initialize session metadata
        let metadata = SessionMetadata {
            session_id,
            rover_id: config.rover_id.clone(),
            started_at: now,
            ended_at: None,
            duration_secs: 0.0,
            gps_bounds: None,
            lidar_frames: 0,
            camera_frames: 0,
            pose_samples: 0,
            session_file,
        };

        Ok(Self {
            config: config.clone(),
            stream: Some(stream),
            session_dir: Some(session_dir),
            session_path: Some(session_path),
            metadata: Some(metadata),
            gps_bounds: GpsBounds::default(),
            lidar_frame_count: AtomicU32::new(0),
            camera_frame_count: AtomicU32::new(0),
            pose_count: AtomicU32::new(0),
            last_lidar_time: 0.0,
            last_camera_time: 0.0,
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
            session_dir: None,
            session_path: None,
            metadata: None,
            gps_bounds: GpsBounds::default(),
            lidar_frame_count: AtomicU32::new(0),
            camera_frame_count: AtomicU32::new(0),
            pose_count: AtomicU32::new(0),
            last_lidar_time: 0.0,
            last_camera_time: 0.0,
        }
    }

    /// Get the session directory path.
    pub fn session_dir(&self) -> Option<&Path> {
        self.session_dir.as_deref()
    }

    /// Get the session ID.
    pub fn session_id(&self) -> Option<Uuid> {
        self.metadata.as_ref().map(|m| m.session_id)
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

        // Log individual components as scalars for time-series
        stream.log("robot/x", &rerun::Scalar::new(pose.x))?;
        stream.log("robot/y", &rerun::Scalar::new(pose.y))?;
        stream.log("robot/heading", &rerun::Scalar::new(pose.theta))?;

        self.pose_count.fetch_add(1, Ordering::Relaxed);
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
    pub fn log_gps(&mut self, lat: f64, lon: f64, accuracy: f32) -> Result<(), RecordingError> {
        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Log as 2D point (lon, lat for standard map orientation)
        stream.log("gps/position", &rerun::Points2D::new([(lon as f32, lat as f32)]))?;

        stream.log("gps/latitude", &rerun::Scalar::new(lat))?;
        stream.log("gps/longitude", &rerun::Scalar::new(lon))?;
        stream.log("gps/accuracy", &rerun::Scalar::new(accuracy as f64))?;

        // Update GPS bounds for session metadata
        if lat.abs() > 0.001 && lon.abs() > 0.001 {
            self.gps_bounds.expand(lat, lon);
        }

        Ok(())
    }

    /// Log a LiDAR point cloud frame.
    ///
    /// Points should be in rover frame coordinates.
    /// Rate limited based on config.lidar_hz.
    pub fn log_lidar_points(
        &mut self,
        time_secs: f64,
        points: &[[f32; 3]],
    ) -> Result<(), RecordingError> {
        if !self.config.include_lidar {
            return Ok(());
        }

        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Rate limiting
        let interval = 1.0 / self.config.lidar_hz as f64;
        if time_secs - self.last_lidar_time < interval {
            return Ok(());
        }
        self.last_lidar_time = time_secs;

        let frame_num = self.lidar_frame_count.fetch_add(1, Ordering::Relaxed);

        // Log to Rerun
        stream.log("lidar/points", &rerun::Points3D::new(points.to_vec()))?;

        debug!(frame = frame_num, points = points.len(), "Logged LiDAR frame");
        Ok(())
    }

    /// Log a camera frame (JPEG encoded).
    ///
    /// Rate limited based on config.camera_hz.
    pub fn log_camera_frame(
        &mut self,
        time_secs: f64,
        jpeg_data: &[u8],
    ) -> Result<(), RecordingError> {
        if !self.config.include_camera {
            return Ok(());
        }

        let Some(ref stream) = self.stream else {
            return Ok(());
        };

        // Rate limiting
        let interval = 1.0 / self.config.camera_hz as f64;
        if time_secs - self.last_camera_time < interval {
            return Ok(());
        }
        self.last_camera_time = time_secs;

        let frame_num = self.camera_frame_count.fetch_add(1, Ordering::Relaxed);

        // Log to Rerun as encoded image
        stream.log(
            "camera/image",
            &rerun::EncodedImage::from_file_contents(jpeg_data.to_vec()),
        )?;

        debug!(frame = frame_num, size = jpeg_data.len(), "Logged camera frame");
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
    ///
    /// Writes the session metadata.json file for quick lookups.
    pub fn end_session(&mut self) {
        // Write session metadata
        if let (Some(ref session_dir), Some(ref mut metadata)) = (&self.session_dir, &mut self.metadata) {
            let end_time = Utc::now();
            metadata.ended_at = Some(end_time);
            metadata.duration_secs = (end_time - metadata.started_at).num_milliseconds() as f64 / 1000.0;
            metadata.lidar_frames = self.lidar_frame_count.load(Ordering::Relaxed);
            metadata.camera_frames = self.camera_frame_count.load(Ordering::Relaxed);
            metadata.pose_samples = self.pose_count.load(Ordering::Relaxed);

            if self.gps_bounds.is_valid() {
                metadata.gps_bounds = Some(self.gps_bounds.clone());
            }

            let metadata_path = session_dir.join("metadata.json");
            match serde_json::to_string_pretty(metadata) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&metadata_path, json) {
                        warn!(error = %e, "Failed to write session metadata");
                    } else {
                        info!(
                            path = %metadata_path.display(),
                            duration = format!("{:.1}s", metadata.duration_secs),
                            lidar_frames = metadata.lidar_frames,
                            camera_frames = metadata.camera_frames,
                            pose_samples = metadata.pose_samples,
                            "Session complete"
                        );
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to serialize session metadata");
                }
            }
        }

        if let Some(ref path) = self.session_dir {
            info!(path = %path.display(), "Ending recording session");
        }

        self.stream = None;
        self.session_dir = None;
        self.session_path = None;
    }

    /// Log session metadata to Rerun at start (static data).
    fn log_session_metadata_rerun(
        stream: &RecordingStream,
        config: &Config,
        session_id: &Uuid,
    ) -> Result<(), RecordingError> {
        stream.log_static(
            "session/rover_id",
            &rerun::TextLog::new(config.rover_id.clone()),
        )?;
        stream.log_static(
            "session/id",
            &rerun::TextLog::new(session_id.to_string()),
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
        // Find all session directories (they contain session.rrd)
        let mut sessions: Vec<_> = std::fs::read_dir(session_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.path().join("session.rrd").exists())
            .collect();

        // Sort by name (ISO timestamp format, so alphabetical = chronological)
        sessions.sort_by_key(|e| e.file_name());

        // Calculate total size
        let total_size: u64 = sessions
            .iter()
            .map(|e| dir_size(&e.path()))
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
            let size = dir_size(&path);
            if std::fs::remove_dir_all(&path).is_ok() {
                debug!(path = %path.display(), size, "Rotated old session");
                current_size -= size;
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

/// Calculate total size of a directory.
fn dir_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                .sum()
        })
        .unwrap_or(0)
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

    #[test]
    fn test_gps_bounds() {
        let mut bounds = GpsBounds::default();
        assert!(!bounds.is_valid());

        bounds.expand(44.9778, -93.2650);
        assert!(bounds.is_valid());
        assert_eq!(bounds.min_lat, 44.9778);
        assert_eq!(bounds.max_lat, 44.9778);

        bounds.expand(44.9812, -93.2580);
        assert_eq!(bounds.min_lat, 44.9778);
        assert_eq!(bounds.max_lat, 44.9812);
        assert_eq!(bounds.min_lon, -93.2650);
        assert_eq!(bounds.max_lon, -93.2580);
    }
}
