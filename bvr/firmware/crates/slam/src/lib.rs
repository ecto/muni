//! 2D LiDAR SLAM for BVR rover.
//!
//! Provides:
//! - Correlative scan matching for robust pose estimation
//! - Pose graph with sequential odometry edges
//! - Loop closure detection and graph optimization
//!
//! The SLAM system runs at ~10Hz (scan rate) and outputs pose corrections
//! that are applied to the odometry estimate.

use lidar::LaserScan;
use nalgebra::{DMatrix, DVector, Matrix3, Vector3};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, warn};
use transforms::Transform2D;
use types::Pose;

mod scan_matcher;

pub use scan_matcher::{CorrelativeScanMatcher, ScanMatchConfig, ScanMatchResult};

#[derive(Error, Debug)]
pub enum SlamError {
    #[error("Scan matching failed: {0}")]
    ScanMatchFailed(String),
    #[error("Not enough keyframes for loop closure")]
    NotEnoughKeyframes,
    #[error("Graph optimization failed: {0}")]
    OptimizationFailed(String),
}

/// SLAM configuration.
#[derive(Debug, Clone)]
pub struct SlamConfig {
    /// Resolution for scan matching correlation grid (meters)
    pub scan_match_resolution: f64,
    /// Linear search range for scan matching (meters)
    pub scan_match_range: f64,
    /// Angular search range for scan matching (radians)
    pub scan_match_angular_range: f64,
    /// Distance threshold for inserting new keyframe (meters)
    pub keyframe_distance: f64,
    /// Rotation threshold for inserting new keyframe (radians)
    pub keyframe_rotation: f64,
    /// Minimum node count before checking for loop closures
    pub loop_closure_min_nodes: usize,
    /// Score threshold for accepting loop closure match
    pub loop_closure_threshold: f64,
    /// Maximum distance to consider for loop closure (meters)
    pub loop_closure_search_radius: f64,
}

impl Default for SlamConfig {
    fn default() -> Self {
        Self {
            scan_match_resolution: 0.05,
            scan_match_range: 0.5,
            scan_match_angular_range: 0.26, // ~15 degrees
            keyframe_distance: 1.0,
            keyframe_rotation: 0.5,
            loop_closure_min_nodes: 10,
            loop_closure_threshold: 0.7,
            loop_closure_search_radius: 5.0,
        }
    }
}

/// A keyframe in the pose graph.
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Unique identifier
    pub id: usize,
    /// Pose in world frame (optimized)
    pub pose: Transform2D,
    /// Associated LiDAR scan
    pub scan: Arc<LaserScan>,
    /// Timestamp when keyframe was created
    pub timestamp: Instant,
}

/// An edge (constraint) in the pose graph.
#[derive(Debug, Clone)]
pub struct PoseGraphEdge {
    /// Source keyframe ID
    pub from_id: usize,
    /// Target keyframe ID
    pub to_id: usize,
    /// Relative pose measurement (from -> to)
    pub measurement: Transform2D,
    /// Information matrix (inverse covariance)
    pub information: Matrix3<f64>,
    /// Whether this is a loop closure edge
    pub is_loop_closure: bool,
}

/// Result from SLAM update.
#[derive(Debug, Clone)]
pub struct SlamUpdate {
    /// Current pose in world frame
    pub world_pose: Pose,
    /// Correction transform (odom -> world)
    pub odom_correction: Transform2D,
    /// Whether a keyframe was added
    pub keyframe_added: bool,
    /// Whether a loop closure was detected
    pub loop_closure_detected: bool,
    /// Current keyframe count
    pub keyframe_count: usize,
    /// Total loop closures detected
    pub loop_closure_count: usize,
}

/// Main SLAM processor.
pub struct SlamProcessor {
    config: SlamConfig,
    scan_matcher: CorrelativeScanMatcher,
    /// All keyframes
    keyframes: Vec<Keyframe>,
    /// Pose graph edges
    edges: Vec<PoseGraphEdge>,
    /// Current pose in world frame
    current_pose: Transform2D,
    /// Odom -> world correction (updated by SLAM)
    odom_correction: Transform2D,
    /// Last odom pose received
    last_odom_pose: Transform2D,
    /// Pose at last keyframe insertion
    last_keyframe_pose: Transform2D,
    /// Reference scan for scan-to-scan matching
    reference_scan: Option<Arc<LaserScan>>,
    /// Total loop closures detected
    loop_closure_count: usize,
}

impl SlamProcessor {
    /// Create a new SLAM processor.
    pub fn new(config: SlamConfig) -> Self {
        let scan_config = ScanMatchConfig {
            resolution: config.scan_match_resolution,
            linear_range: config.scan_match_range,
            angular_range: config.scan_match_angular_range,
            angular_resolution: 0.02, // ~1 degree
        };

        Self {
            config,
            scan_matcher: CorrelativeScanMatcher::new(scan_config),
            keyframes: Vec::new(),
            edges: Vec::new(),
            current_pose: Transform2D::identity(),
            odom_correction: Transform2D::identity(),
            last_odom_pose: Transform2D::identity(),
            last_keyframe_pose: Transform2D::identity(),
            reference_scan: None,
            loop_closure_count: 0,
        }
    }

    /// Update with new odometry pose (high frequency, ~100Hz).
    /// Call this every control loop iteration.
    pub fn update_odometry(&mut self, odom_pose: &Pose) {
        let odom_tf = Transform2D::from_pose(odom_pose);

        // Compute odom delta since last update
        let delta = self.last_odom_pose.relative_to(&odom_tf);

        // Apply delta to current world pose
        self.current_pose = &self.current_pose * &delta;

        self.last_odom_pose = odom_tf;
    }

    /// Process a new LiDAR scan (lower frequency, ~10Hz).
    /// Returns update info if pose was refined or keyframe added.
    pub fn process_scan(&mut self, scan: &LaserScan) -> Option<SlamUpdate> {
        let scan = Arc::new(scan.clone());

        // Check if we should add a keyframe
        let should_add_keyframe = self.should_add_keyframe();

        // Perform scan matching
        let matched = if let Some(ref reference) = self.reference_scan {
            // Scan-to-scan matching
            match self.scan_matcher.match_scans(reference, &scan, Transform2D::identity()) {
                Ok(result) => {
                    if result.score > 0.5 {
                        // Apply scan match correction
                        let correction = result.transform;
                        self.current_pose = &self.current_pose * &correction;
                        true
                    } else {
                        debug!(score = result.score, "Scan match score too low, skipping correction");
                        false
                    }
                }
                Err(e) => {
                    warn!(?e, "Scan matching failed");
                    false
                }
            }
        } else {
            // First scan, no matching yet
            false
        };

        let mut keyframe_added = false;
        let mut loop_closure_detected = false;

        if should_add_keyframe {
            keyframe_added = true;
            let keyframe_id = self.keyframes.len();

            // Create keyframe
            let keyframe = Keyframe {
                id: keyframe_id,
                pose: self.current_pose,
                scan: scan.clone(),
                timestamp: Instant::now(),
            };

            // Add odometry edge from previous keyframe
            if let Some(prev) = self.keyframes.last() {
                let relative_pose = prev.pose.relative_to(&self.current_pose);
                let edge = PoseGraphEdge {
                    from_id: prev.id,
                    to_id: keyframe_id,
                    measurement: relative_pose,
                    information: Matrix3::identity() * 100.0, // Tune based on odometry quality
                    is_loop_closure: false,
                };
                self.edges.push(edge);
            }

            // Check for loop closures
            if keyframe_id >= self.config.loop_closure_min_nodes {
                if let Some(closure) = self.detect_loop_closure(&keyframe) {
                    loop_closure_detected = true;
                    self.loop_closure_count += 1;
                    self.edges.push(closure);

                    // Optimize pose graph
                    if let Err(e) = self.optimize() {
                        warn!(?e, "Pose graph optimization failed");
                    }
                }
            }

            self.keyframes.push(keyframe);
            self.last_keyframe_pose = self.current_pose;

            info!(
                id = keyframe_id,
                x = self.current_pose.translation().x,
                y = self.current_pose.translation().y,
                "Added keyframe"
            );
        }

        // Update reference scan
        self.reference_scan = Some(scan);

        // Update odom correction based on current pose vs odom pose
        self.odom_correction = self.last_odom_pose.relative_to(&self.current_pose);

        if keyframe_added || matched {
            Some(SlamUpdate {
                world_pose: self.current_pose.to_pose(),
                odom_correction: self.odom_correction,
                keyframe_added,
                loop_closure_detected,
                keyframe_count: self.keyframes.len(),
                loop_closure_count: self.loop_closure_count,
            })
        } else {
            None
        }
    }

    /// Get current pose in world frame.
    pub fn pose(&self) -> Pose {
        self.current_pose.to_pose()
    }

    /// Get the odom->world correction transform.
    pub fn odom_correction(&self) -> Transform2D {
        self.odom_correction
    }

    /// Get all keyframes.
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Get all edges.
    pub fn edges(&self) -> &[PoseGraphEdge] {
        &self.edges
    }

    /// Get keyframe poses as (x, y, theta) tuples for visualization.
    pub fn keyframe_poses(&self) -> Vec<(f64, f64, f64)> {
        self.keyframes
            .iter()
            .map(|kf| {
                let t = kf.pose.translation();
                (t.x, t.y, kf.pose.rotation())
            })
            .collect()
    }

    /// Get loop closure count.
    pub fn loop_closure_count(&self) -> usize {
        self.loop_closure_count
    }

    /// Get keyframe count.
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if we should add a new keyframe.
    fn should_add_keyframe(&self) -> bool {
        // Always add first keyframe
        if self.keyframes.is_empty() {
            return true;
        }

        let delta = self.last_keyframe_pose.relative_to(&self.current_pose);
        let distance = delta.translation().norm();
        let rotation = delta.rotation().abs();

        distance >= self.config.keyframe_distance || rotation >= self.config.keyframe_rotation
    }

    /// Detect loop closure for a new keyframe.
    fn detect_loop_closure(&self, new_keyframe: &Keyframe) -> Option<PoseGraphEdge> {
        let current_pos = new_keyframe.pose.translation();

        // Search through old keyframes (skip recent ones)
        let skip_recent = self.config.loop_closure_min_nodes;

        for candidate in self.keyframes.iter().take(self.keyframes.len().saturating_sub(skip_recent)) {
            let candidate_pos = candidate.pose.translation();
            let distance = (current_pos - candidate_pos).norm();

            // Check if within search radius
            if distance > self.config.loop_closure_search_radius {
                continue;
            }

            // Compute initial guess for scan matching
            let initial_guess = candidate.pose.relative_to(&new_keyframe.pose);

            // Try scan matching
            match self.scan_matcher.match_scans(&candidate.scan, &new_keyframe.scan, initial_guess) {
                Ok(result) => {
                    if result.score >= self.config.loop_closure_threshold {
                        info!(
                            from = candidate.id,
                            to = new_keyframe.id,
                            score = result.score,
                            "Loop closure detected"
                        );

                        return Some(PoseGraphEdge {
                            from_id: candidate.id,
                            to_id: new_keyframe.id,
                            measurement: result.transform,
                            information: self.compute_information(&result),
                            is_loop_closure: true,
                        });
                    }
                }
                Err(_) => continue,
            }
        }

        None
    }

    /// Compute information matrix from scan match result.
    fn compute_information(&self, result: &ScanMatchResult) -> Matrix3<f64> {
        // Simple approach: scale identity by match score
        // Better approach would use covariance estimation from scan matching
        let weight = result.score * result.score * 1000.0;
        Matrix3::identity() * weight
    }

    /// Optimize the pose graph using Gauss-Newton.
    fn optimize(&mut self) -> Result<(), SlamError> {
        if self.keyframes.len() < 2 {
            return Ok(());
        }

        const MAX_ITERATIONS: usize = 10;
        const CONVERGENCE_THRESHOLD: f64 = 1e-4;

        for iteration in 0..MAX_ITERATIONS {
            let (h, b) = self.build_linear_system();

            // Solve H * dx = -b
            // Add damping for stability
            let n = h.nrows();
            let mut h_damped = h.clone();
            for i in 0..n {
                h_damped[(i, i)] += 1e-3;
            }

            // Fix first node (gauge freedom)
            for i in 0..3 {
                h_damped[(i, i)] += 1e10;
            }

            let dx = match h_damped.lu().solve(&(-&b)) {
                Some(x) => x,
                None => {
                    return Err(SlamError::OptimizationFailed("LU decomposition failed".into()));
                }
            };

            // Check convergence
            let delta_norm = dx.norm();
            if delta_norm < CONVERGENCE_THRESHOLD {
                debug!(iterations = iteration + 1, "Pose graph optimization converged");
                break;
            }

            // Apply update
            self.apply_update(&dx);
        }

        // Update current pose if it changed
        if let Some(last) = self.keyframes.last() {
            self.current_pose = last.pose;
        }

        Ok(())
    }

    /// Build the linear system H * dx = b for pose graph optimization.
    fn build_linear_system(&self) -> (DMatrix<f64>, DVector<f64>) {
        let n = self.keyframes.len() * 3;
        let mut h = DMatrix::zeros(n, n);
        let mut b = DVector::zeros(n);

        for edge in &self.edges {
            let i = edge.from_id * 3;
            let j = edge.to_id * 3;

            // Get current poses
            let pose_i = &self.keyframes[edge.from_id].pose;
            let pose_j = &self.keyframes[edge.to_id].pose;

            // Compute error
            let predicted = pose_i.relative_to(pose_j);
            let error = self.compute_edge_error(&predicted, &edge.measurement);

            // Compute Jacobians (simplified: identity for small angles)
            let (j_i, j_j) = self.compute_jacobians(pose_i, pose_j);

            // Add to H and b
            let omega = &edge.information;

            // H += J^T * Omega * J
            let h_ii = j_i.transpose() * omega * &j_i;
            let h_ij = j_i.transpose() * omega * &j_j;
            let h_jj = j_j.transpose() * omega * &j_j;

            for (di, dj, val) in [(0, 0, &h_ii), (0, 1, &h_ij), (1, 1, &h_jj)] {
                let row = if di == 0 { i } else { j };
                let col = if dj == 0 { i } else { j };
                for r in 0..3 {
                    for c in 0..3 {
                        h[(row + r, col + c)] += val[(r, c)];
                        if di != dj {
                            h[(col + c, row + r)] += val[(r, c)];
                        }
                    }
                }
            }

            // b += J^T * Omega * e
            let b_i = j_i.transpose() * omega * &error;
            let b_j = j_j.transpose() * omega * &error;
            for r in 0..3 {
                b[i + r] += b_i[r];
                b[j + r] += b_j[r];
            }
        }

        (h, b)
    }

    /// Compute error between predicted and measured relative pose.
    fn compute_edge_error(&self, predicted: &Transform2D, measured: &Transform2D) -> Vector3<f64> {
        let diff = predicted.relative_to(measured);
        Vector3::new(
            diff.translation().x,
            diff.translation().y,
            transforms::normalize_angle(diff.rotation()),
        )
    }

    /// Compute Jacobians for pose graph edge.
    fn compute_jacobians(
        &self,
        _pose_i: &Transform2D,
        _pose_j: &Transform2D,
    ) -> (Matrix3<f64>, Matrix3<f64>) {
        // Simplified Jacobians (exact for small errors)
        // J_i = -I, J_j = I
        (-Matrix3::identity(), Matrix3::identity())
    }

    /// Apply optimization update to keyframe poses.
    fn apply_update(&mut self, dx: &DVector<f64>) {
        for (i, keyframe) in self.keyframes.iter_mut().enumerate() {
            let idx = i * 3;
            let delta = Transform2D::new(dx[idx], dx[idx + 1], dx[idx + 2]);
            keyframe.pose = &keyframe.pose * &delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_scan(angle_offset: f32) -> LaserScan {
        // Create a simple scan with a few ranges
        let mut scan = LaserScan::default();
        scan.angle_increment = std::f32::consts::PI * 2.0 / 360.0;
        scan.ranges = (0..360)
            .map(|i| {
                let angle = (i as f32) * scan.angle_increment + angle_offset;
                // Simple box-like environment
                let x = angle.cos();
                let y = angle.sin();
                let range = if x.abs() > y.abs() {
                    5.0 / x.abs()
                } else {
                    5.0 / y.abs()
                };
                range.min(scan.range_max)
            })
            .collect();
        scan.intensities = vec![128; 360];
        scan
    }

    #[test]
    fn test_slam_processor_creation() {
        let config = SlamConfig::default();
        let processor = SlamProcessor::new(config);
        assert_eq!(processor.keyframes.len(), 0);
        assert_eq!(processor.edges.len(), 0);
    }

    #[test]
    fn test_odometry_update() {
        let config = SlamConfig::default();
        let mut processor = SlamProcessor::new(config);

        // Update odometry
        processor.update_odometry(&Pose {
            x: 1.0,
            y: 0.5,
            theta: 0.1,
        });

        let pose = processor.pose();
        assert!((pose.x - 1.0).abs() < 0.01);
        assert!((pose.y - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_keyframe_insertion() {
        let mut config = SlamConfig::default();
        config.keyframe_distance = 0.5; // Lower threshold for testing
        let mut processor = SlamProcessor::new(config);

        // First scan should create a keyframe
        let scan1 = make_test_scan(0.0);
        let result = processor.process_scan(&scan1);
        assert!(result.is_some());
        assert!(result.unwrap().keyframe_added);
        assert_eq!(processor.keyframes.len(), 1);

        // Move robot and process another scan
        processor.update_odometry(&Pose {
            x: 1.0,
            y: 0.0,
            theta: 0.0,
        });
        let scan2 = make_test_scan(0.0);
        let result = processor.process_scan(&scan2);
        assert!(result.is_some());
        assert!(result.unwrap().keyframe_added);
        assert_eq!(processor.keyframes.len(), 2);
        assert_eq!(processor.edges.len(), 1); // One odometry edge
    }
}
