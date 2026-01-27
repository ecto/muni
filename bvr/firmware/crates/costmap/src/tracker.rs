//! Object tracker with stable IDs and alpha-beta position filtering.
//!
//! Greedy nearest-centroid tracker that sits between `extract_obstacles()`
//! and the watch channel. Provides:
//! - Stable track IDs across frames (monotonic u32, never recycled)
//! - Alpha-beta filter for smooth position and velocity estimation
//! - Classification hysteresis (require N consistent frames before switching)
//! - Coasting (dead-reckoning with velocity when detection is lost)

use crate::clustering::{Obstacle, ObstacleClass};

/// A tracked obstacle with stable identity and velocity.
#[derive(Debug, Clone)]
pub struct TrackedObstacle {
    /// Monotonic track ID (never recycled).
    pub track_id: u32,
    /// Smoothed classification (after hysteresis).
    pub class: ObstacleClass,
    /// Confidence: 0 = tentative, 255 = fully confirmed.
    pub confidence: u8,
    /// Age in frames since track was created.
    pub age: u16,
    /// Centroid X in world coordinates (meters).
    pub centroid_x: f32,
    /// Centroid Y in world coordinates (meters).
    pub centroid_y: f32,
    /// Bounding box minimum X.
    pub bbox_min_x: f32,
    /// Bounding box minimum Y.
    pub bbox_min_y: f32,
    /// Bounding box maximum X.
    pub bbox_max_x: f32,
    /// Bounding box maximum Y.
    pub bbox_max_y: f32,
    /// Number of cells in the cluster.
    pub cell_count: u32,
    /// Area in square meters.
    pub area: f32,
    /// Minimum observed Z (height).
    pub min_z: f32,
    /// Maximum observed Z (height).
    pub max_z: f32,
    /// PCA principal axis angle in radians.
    pub rotation: f32,
    /// Estimated velocity X (m/s).
    pub velocity_x: f32,
    /// Estimated velocity Y (m/s).
    pub velocity_y: f32,
}

/// Tracker configuration.
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Maximum distance (meters) for associating a detection to a track.
    pub max_association_distance: f32,
    /// Number of frames a track can coast without detection before deletion.
    pub coast_frames: u16,
    /// Alpha-beta filter: position correction gain (0-1, higher = more responsive).
    pub position_alpha: f32,
    /// Alpha-beta filter: velocity correction gain (0-1, higher = more responsive).
    pub velocity_beta: f32,
    /// Number of consistent raw-class frames before switching classification.
    pub class_hysteresis_frames: u8,
    /// Number of frames before a tentative track is confirmed.
    pub confirm_age: u16,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            max_association_distance: 1.0,
            coast_frames: 5,
            position_alpha: 0.3,
            velocity_beta: 0.05,
            class_hysteresis_frames: 3,
            confirm_age: 3,
        }
    }
}

/// Internal track state.
#[derive(Debug, Clone)]
struct Track {
    id: u32,
    centroid_x: f32,
    centroid_y: f32,
    velocity_x: f32,
    velocity_y: f32,
    /// Predicted position (after velocity extrapolation).
    predicted_x: f32,
    predicted_y: f32,
    /// Current smoothed classification.
    class: ObstacleClass,
    /// Raw class seen in recent frames (for hysteresis).
    pending_class: ObstacleClass,
    /// Number of consecutive frames with pending_class.
    pending_class_count: u8,
    age: u16,
    coast_remaining: u16,
    /// Last matched obstacle data (for output).
    bbox_min_x: f32,
    bbox_min_y: f32,
    bbox_max_x: f32,
    bbox_max_y: f32,
    cell_count: u32,
    area: f32,
    min_z: f32,
    max_z: f32,
    rotation: f32,
}

/// Object tracker with greedy nearest-centroid matching.
pub struct ObstacleTracker {
    config: TrackerConfig,
    tracks: Vec<Track>,
    next_id: u32,
    /// Pre-allocated buffer for distance pairs during association.
    pairs: Vec<(f32, usize, usize)>,
}

impl std::fmt::Debug for ObstacleTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObstacleTracker")
            .field("config", &self.config)
            .field("active_tracks", &self.tracks.len())
            .field("next_id", &self.next_id)
            .finish()
    }
}

impl ObstacleTracker {
    /// Create a new tracker with the given configuration.
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            config,
            tracks: Vec::with_capacity(64),
            next_id: 0,
            pairs: Vec::with_capacity(256),
        }
    }

    /// Update the tracker with new detections.
    ///
    /// Returns tracked obstacles with stable IDs and velocity estimates.
    pub fn update(&mut self, detections: &[Obstacle], dt: f32) -> Vec<TrackedObstacle> {
        let dt = dt.max(0.001); // avoid division by zero

        // 1. Predict each track's position using velocity
        for track in &mut self.tracks {
            track.predicted_x = track.centroid_x + track.velocity_x * dt;
            track.predicted_y = track.centroid_y + track.velocity_y * dt;
        }

        // 2. Compute distances from each detection to each predicted track
        self.pairs.clear();
        let max_dist = self.config.max_association_distance;
        for (di, det) in detections.iter().enumerate() {
            for (ti, track) in self.tracks.iter().enumerate() {
                let dx = det.centroid_x - track.predicted_x;
                let dy = det.centroid_y - track.predicted_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= max_dist {
                    self.pairs.push((dist, di, ti));
                }
            }
        }

        // 3. Sort pairs by distance, greedily assign
        self.pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut det_matched = vec![false; detections.len()];
        let mut track_matched = vec![false; self.tracks.len()];

        for &(_, di, ti) in &self.pairs {
            if det_matched[di] || track_matched[ti] {
                continue;
            }
            det_matched[di] = true;
            track_matched[ti] = true;

            let det = &detections[di];
            let track = &mut self.tracks[ti];
            let alpha = self.config.position_alpha;
            let beta = self.config.velocity_beta;

            // Alpha-beta filter: correct predicted position and velocity
            // using the residual between detection and prediction.
            let residual_x = det.centroid_x - track.predicted_x;
            let residual_y = det.centroid_y - track.predicted_y;
            track.centroid_x = track.predicted_x + alpha * residual_x;
            track.centroid_y = track.predicted_y + alpha * residual_y;
            track.velocity_x += (beta / dt) * residual_x;
            track.velocity_y += (beta / dt) * residual_y;

            // Update shape data
            track.bbox_min_x = det.bbox_min_x;
            track.bbox_min_y = det.bbox_min_y;
            track.bbox_max_x = det.bbox_max_x;
            track.bbox_max_y = det.bbox_max_y;
            track.cell_count = det.cell_count;
            track.area = det.area;
            track.min_z = det.min_z;
            track.max_z = det.max_z;
            track.rotation = det.rotation;

            // Classification hysteresis
            if det.class != track.pending_class {
                track.pending_class = det.class;
                track.pending_class_count = 1;
            } else {
                track.pending_class_count =
                    track.pending_class_count.saturating_add(1);
            }
            if track.pending_class_count >= self.config.class_hysteresis_frames {
                track.class = track.pending_class;
            }

            track.age = track.age.saturating_add(1);
            track.coast_remaining = self.config.coast_frames;
        }

        // 4. Unmatched detections -> new tracks
        for (di, det) in detections.iter().enumerate() {
            if det_matched[di] {
                continue;
            }
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            self.tracks.push(Track {
                id,
                centroid_x: det.centroid_x,
                centroid_y: det.centroid_y,
                velocity_x: 0.0,
                velocity_y: 0.0,
                predicted_x: det.centroid_x,
                predicted_y: det.centroid_y,
                class: det.class,
                pending_class: det.class,
                pending_class_count: 1,
                age: 1, // first detection counts as one observation
                coast_remaining: self.config.coast_frames,
                bbox_min_x: det.bbox_min_x,
                bbox_min_y: det.bbox_min_y,
                bbox_max_x: det.bbox_max_x,
                bbox_max_y: det.bbox_max_y,
                cell_count: det.cell_count,
                area: det.area,
                min_z: det.min_z,
                max_z: det.max_z,
                rotation: det.rotation,
            });
        }

        // 5. Unmatched tracks -> coast or delete
        let coast_frames = self.config.coast_frames;
        let mut i = 0;
        while i < self.tracks.len() {
            let ti_orig = i; // track index in original order
            if ti_orig < track_matched.len() && !track_matched[ti_orig] {
                let track = &mut self.tracks[i];
                // Dead-reckon with velocity
                track.centroid_x = track.predicted_x;
                track.centroid_y = track.predicted_y;
                // Shift bbox by velocity
                let dx = track.velocity_x * dt;
                let dy = track.velocity_y * dt;
                track.bbox_min_x += dx;
                track.bbox_min_y += dy;
                track.bbox_max_x += dx;
                track.bbox_max_y += dy;
                track.age = track.age.saturating_add(1);
                track.coast_remaining = track.coast_remaining.saturating_sub(1);
                if track.coast_remaining == 0 {
                    self.tracks.swap_remove(i);
                    // Also adjust track_matched if needed
                    if i < track_matched.len() {
                        track_matched.swap_remove(i);
                    }
                    continue;
                }
            }
            i += 1;
        }

        // 6. Build output
        let confirm_age = self.config.confirm_age;
        self.tracks
            .iter()
            .filter(|t| t.age >= confirm_age || t.coast_remaining < coast_frames)
            .map(|t| TrackedObstacle {
                track_id: t.id,
                class: t.class,
                confidence: if t.age >= confirm_age {
                    // Scale confidence: confirmed tracks get 128-255 based on age
                    128 + (t.age.min(127) as u8)
                } else {
                    // Tentative: scale 0-127
                    ((t.age as f32 / confirm_age as f32) * 127.0) as u8
                },
                age: t.age,
                centroid_x: t.centroid_x,
                centroid_y: t.centroid_y,
                bbox_min_x: t.bbox_min_x,
                bbox_min_y: t.bbox_min_y,
                bbox_max_x: t.bbox_max_x,
                bbox_max_y: t.bbox_max_y,
                cell_count: t.cell_count,
                area: t.area,
                min_z: t.min_z,
                max_z: t.max_z,
                rotation: t.rotation,
                velocity_x: t.velocity_x,
                velocity_y: t.velocity_y,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obstacle(id: u16, cx: f32, cy: f32) -> Obstacle {
        Obstacle {
            id,
            class: ObstacleClass::Pedestrian,
            centroid_x: cx,
            centroid_y: cy,
            bbox_min_x: cx - 0.3,
            bbox_min_y: cy - 0.3,
            bbox_max_x: cx + 0.3,
            bbox_max_y: cy + 0.3,
            cell_count: 25,
            area: 0.36,
            elongation: 1.2,
            rotation: 0.0,
            min_z: 0.0,
            max_z: 1.7,
        }
    }

    #[test]
    fn test_new_detections_get_unique_ids() {
        let mut tracker = ObstacleTracker::new(TrackerConfig::default());
        let dets = vec![make_obstacle(0, 1.0, 2.0), make_obstacle(1, 5.0, 6.0)];
        let tracked = tracker.update(&dets, 0.1);

        // First frame: age=1, confirm_age=3 -> tentative, not yet confirmed
        assert!(tracked.is_empty());

        // Second frame: age=2, still tentative
        let tracked = tracker.update(&dets, 0.1);
        assert!(tracked.is_empty());

        // Third frame: matched, age=3 >= confirm_age=3 -> confirmed
        let tracked = tracker.update(&dets, 0.1);
        assert_eq!(tracked.len(), 2);
        assert_ne!(tracked[0].track_id, tracked[1].track_id);
    }

    #[test]
    fn test_stable_ids_across_frames() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            ..Default::default()
        });

        let dets1 = vec![make_obstacle(0, 1.0, 2.0)];
        let tracked1 = tracker.update(&dets1, 0.1);
        assert_eq!(tracked1.len(), 1);
        let id = tracked1[0].track_id;

        // Same obstacle moved slightly
        let dets2 = vec![make_obstacle(0, 1.05, 2.02)];
        let tracked2 = tracker.update(&dets2, 0.1);
        assert_eq!(tracked2.len(), 1);
        assert_eq!(tracked2[0].track_id, id); // same ID
    }

    #[test]
    fn test_velocity_estimation() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            position_alpha: 1.0, // snap to detection for test
            velocity_beta: 1.0,  // full velocity correction for test
            ..Default::default()
        });

        let dets1 = vec![make_obstacle(0, 0.0, 0.0)];
        tracker.update(&dets1, 0.1);

        // Move 1m in X over 0.1s -> 10 m/s
        let dets2 = vec![make_obstacle(0, 1.0, 0.0)];
        let tracked = tracker.update(&dets2, 0.1);
        assert_eq!(tracked.len(), 1);
        assert!((tracked[0].velocity_x - 10.0).abs() < 0.01);
        assert!(tracked[0].velocity_y.abs() < 0.01);
    }

    #[test]
    fn test_coast_and_delete() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            coast_frames: 3,
            ..Default::default()
        });

        let dets = vec![make_obstacle(0, 1.0, 2.0)];
        tracker.update(&dets, 0.1);

        // Detection disappears
        let empty: Vec<Obstacle> = vec![];
        let t1 = tracker.update(&empty, 0.1);
        assert_eq!(t1.len(), 1); // coasting

        let t2 = tracker.update(&empty, 0.1);
        assert_eq!(t2.len(), 1); // still coasting

        let t3 = tracker.update(&empty, 0.1);
        assert!(t3.is_empty()); // coast_remaining hit 0, deleted
    }

    #[test]
    fn test_classification_hysteresis() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            class_hysteresis_frames: 3,
            ..Default::default()
        });

        let mut det = make_obstacle(0, 1.0, 2.0);
        det.class = ObstacleClass::Pedestrian;
        tracker.update(&[det.clone()], 0.1);
        let tracked = tracker.update(&[det.clone()], 0.1);
        assert_eq!(tracked[0].class, ObstacleClass::Pedestrian);

        // Change raw class to Vehicle - should not switch immediately
        det.class = ObstacleClass::Vehicle;
        let tracked = tracker.update(&[det.clone()], 0.1);
        assert_eq!(tracked[0].class, ObstacleClass::Pedestrian); // still Pedestrian

        let tracked = tracker.update(&[det.clone()], 0.1);
        assert_eq!(tracked[0].class, ObstacleClass::Pedestrian); // still (2 frames)

        let tracked = tracker.update(&[det.clone()], 0.1);
        assert_eq!(tracked[0].class, ObstacleClass::Vehicle); // switched after 3 frames
    }

    #[test]
    fn test_no_id_reuse() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            coast_frames: 1,
            ..Default::default()
        });

        let dets = vec![make_obstacle(0, 1.0, 2.0)];
        let t1 = tracker.update(&dets, 0.1);
        let id1 = t1[0].track_id;

        // Remove and re-add at same position
        let empty: Vec<Obstacle> = vec![];
        tracker.update(&empty, 0.1); // coast
        tracker.update(&empty, 0.1); // deleted

        let t2 = tracker.update(&dets, 0.1);
        assert_eq!(t2.len(), 1);
        assert_ne!(t2[0].track_id, id1); // new ID, not recycled
    }

    #[test]
    fn test_position_smoothing() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            position_alpha: 0.5,
            velocity_beta: 0.1,
            ..Default::default()
        });

        // Frame 1: obstacle at (0, 0)
        let dets1 = vec![make_obstacle(0, 0.0, 0.0)];
        let t1 = tracker.update(&dets1, 0.1);
        assert_eq!(t1.len(), 1);
        assert!((t1[0].centroid_x).abs() < 0.01);

        // Frame 2: detection jumps to (1, 0). With alpha=0.5, position should
        // NOT snap to 1.0 — it should be predicted(0) + 0.5 * residual(1) = 0.5
        let dets2 = vec![make_obstacle(0, 1.0, 0.0)];
        let t2 = tracker.update(&dets2, 0.1);
        assert_eq!(t2.len(), 1);
        assert!(
            (t2[0].centroid_x - 0.5).abs() < 0.01,
            "expected ~0.5, got {}",
            t2[0].centroid_x
        );
    }

    #[test]
    fn test_empty_detections() {
        let mut tracker = ObstacleTracker::new(TrackerConfig::default());
        let tracked = tracker.update(&[], 0.1);
        assert!(tracked.is_empty());
    }

    #[test]
    fn test_association_distance_threshold() {
        let mut tracker = ObstacleTracker::new(TrackerConfig {
            confirm_age: 1,
            max_association_distance: 0.5,
            ..Default::default()
        });

        let dets1 = vec![make_obstacle(0, 0.0, 0.0)];
        let t1 = tracker.update(&dets1, 0.1);
        let id1 = t1[0].track_id;

        // Move 2m away — beyond association distance, should create new track
        let dets2 = vec![make_obstacle(0, 2.0, 0.0)];
        let t2 = tracker.update(&dets2, 0.1);
        // The old track is coasting and the new detection creates a new track
        assert!(t2.len() >= 1);
        // The new detection should have a different ID
        let new_ids: Vec<u32> = t2.iter().map(|t| t.track_id).collect();
        // id1 might still be coasting; the new detection should be a different ID
        assert!(new_ids.iter().any(|&id| id != id1));
    }
}
