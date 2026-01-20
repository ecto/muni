//! Session types for depot services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::geo::GpsBounds;

/// Session metadata (written by rover during recording).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: Uuid,
    pub rover_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub gps_bounds: Option<GpsBounds>,
    pub lidar_frames: u32,
    pub camera_frames: u32,
    /// Number of pose samples recorded (for LiDAR alignment)
    #[serde(default)]
    pub pose_samples: u32,
    pub telemetry_file: String,
}

impl SessionMetadata {
    /// Calculate session duration in seconds.
    pub fn duration_secs(&self) -> Option<i64> {
        self.ended_at
            .map(|end| (end - self.started_at).num_seconds())
    }

    /// Check if the session has GPS data.
    pub fn has_gps(&self) -> bool {
        self.gps_bounds.as_ref().map_or(false, |b| b.is_valid())
    }

    /// Check if the session has enough data for processing.
    pub fn is_processable(&self) -> bool {
        self.lidar_frames > 0 || self.camera_frames > 0
    }
}

/// Session processing status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Just discovered, not yet validated
    Pending,
    /// Validated and queued for processing
    Queued,
    /// Currently being processed
    Processing,
    /// Successfully processed and merged into a map
    Processed,
    /// Processing failed
    Failed,
    /// Incomplete or invalid session
    Invalid,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Session record in the mapper database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: Uuid,
    pub rover_id: String,
    pub path: PathBuf,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub gps_bounds: Option<GpsBounds>,
    pub lidar_frames: u32,
    pub camera_frames: u32,
    /// Number of pose samples for LiDAR alignment
    #[serde(default)]
    pub pose_samples: u32,
    pub status: SessionStatus,
    pub map_id: Option<Uuid>,
    pub discovered_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Session {
    /// Create a new session from metadata.
    pub fn from_metadata(metadata: SessionMetadata, path: PathBuf) -> Self {
        Self {
            id: metadata.session_id,
            rover_id: metadata.rover_id,
            path,
            started_at: metadata.started_at,
            ended_at: metadata.ended_at,
            gps_bounds: metadata.gps_bounds,
            lidar_frames: metadata.lidar_frames,
            camera_frames: metadata.camera_frames,
            pose_samples: metadata.pose_samples,
            status: SessionStatus::Pending,
            map_id: None,
            discovered_at: Utc::now(),
            processed_at: None,
            error: None,
        }
    }

    /// Calculate session duration in seconds.
    pub fn duration_secs(&self) -> Option<i64> {
        self.ended_at
            .map(|end| (end - self.started_at).num_seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serde() {
        let statuses = [
            SessionStatus::Pending,
            SessionStatus::Queued,
            SessionStatus::Processing,
            SessionStatus::Processed,
            SessionStatus::Failed,
            SessionStatus::Invalid,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let decoded: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, status);
        }
    }

    #[test]
    fn test_session_status_snake_case() {
        // Verify snake_case serialization
        assert_eq!(
            serde_json::to_string(&SessionStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Processing).unwrap(),
            "\"processing\""
        );
    }

    #[test]
    fn test_session_metadata_duration() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(3600);

        let metadata = SessionMetadata {
            session_id: Uuid::new_v4(),
            rover_id: "test".to_string(),
            started_at: start,
            ended_at: Some(end),
            gps_bounds: None,
            lidar_frames: 100,
            camera_frames: 50,
            pose_samples: 1000,
            telemetry_file: "telemetry.rrd".to_string(),
        };

        assert_eq!(metadata.duration_secs(), Some(3600));
        assert!(metadata.is_processable());
    }
}
