//! 3D LiDAR SLAM for BVR rover.
//!
//! Provides:
//! - 3D GICP scan matching for robust pose estimation
//! - Pose graph with sequential odometry edges
//! - Loop closure detection and graph optimization
//!
//! The SLAM system runs at ~10Hz (scan rate) and outputs pose corrections
//! that are applied to the odometry estimate.

use lidar::PointCloud;
use nalgebra::{DMatrix, DVector, Matrix3, Vector3};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, warn};
use transforms::Transform2D;
use types::Pose;

mod imu_preintegration;
mod persistence;
mod scan_context;
mod scan_matcher;

pub use imu_preintegration::{
    compute_initial_guess, ImuPreintegrationConfig, ImuPreintegrator, PreintegratedDelta,
};
pub use persistence::{MapError, PersistenceConfig};
pub use scan_context::{
    ScanContextCandidate, ScanContextConfig, ScanContextDatabase, ScanContextDescriptor,
};
pub use scan_matcher::{CorrelativeScanMatcher, ScanMatchConfig, ScanMatchResult};

/// State from SLAM task for use by the main control loop.
/// This is updated whenever SLAM processes a scan and sent via a watch channel.
#[derive(Debug, Clone)]
pub struct SlamState {
    /// Current pose in world frame
    pub pose: Pose,
    /// Number of keyframes
    pub keyframe_count: u32,
    /// Number of loop closures detected
    pub loop_closure_count: u32,
    /// Keyframe poses for visualization (x, y, theta)
    pub keyframe_poses: Vec<(f64, f64, f64)>,
    /// Pose covariance from EKF (row-major 3x3).
    /// Uses plain array to avoid nalgebra in the watch channel type.
    pub pose_covariance: [[f64; 3]; 3],
}

impl Default for SlamState {
    fn default() -> Self {
        Self {
            pose: Pose::default(),
            keyframe_count: 0,
            loop_closure_count: 0,
            keyframe_poses: vec![],
            pose_covariance: [[1e-4, 0.0, 0.0], [0.0, 1e-4, 0.0], [0.0, 0.0, 1e-4]],
        }
    }
}

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
    /// Voxel size for GICP downsampling (meters)
    pub voxel_size: f64,
    /// Maximum GICP iterations
    pub max_iterations: usize,
    /// GICP convergence threshold (pose delta norm)
    pub convergence_threshold: f64,
    /// Maximum correspondence distance for outlier rejection (meters)
    pub max_correspondence_dist: f64,
    /// Minimum fraction of points with valid correspondences
    pub min_overlap: f64,
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
    /// Huber loss threshold for robust loop closure weighting (meters)
    pub huber_threshold: f64,
    /// Maximum disagreement (in σ) between loop closure and accumulated odometry
    pub loop_closure_max_disagreement_sigma: f64,
    /// Minimum odometry motion (meters) before scan matching is applied.
    /// Prevents yaw drift from GICP noise while stationary.
    pub min_scan_match_distance: f64,
    /// Minimum odometry rotation (radians) before scan matching is applied.
    pub min_scan_match_rotation: f64,
    /// EKF process noise: linear noise as fraction of distance traveled.
    pub odom_linear_noise: f64,
    /// EKF process noise: angular noise as fraction of rotation.
    pub odom_angular_noise: f64,
    /// EKF process noise: angular noise per meter of linear motion (rad/m).
    pub odom_linear_to_angular_noise: f64,
    /// EKF process noise: linear noise per radian of rotation (m/rad).
    pub odom_angular_to_linear_noise: f64,
    /// IMU pre-integration configuration
    pub imu: ImuPreintegrationConfig,
    /// Scan context descriptor configuration for loop closure
    pub scan_context: ScanContextConfig,
}

impl Default for SlamConfig {
    fn default() -> Self {
        Self {
            voxel_size: 0.2,
            max_iterations: 30,
            convergence_threshold: 1e-4,
            max_correspondence_dist: 1.0,
            min_overlap: 0.3,
            keyframe_distance: 1.0,
            keyframe_rotation: 0.5,
            loop_closure_min_nodes: 10,
            loop_closure_threshold: 0.7,
            loop_closure_search_radius: 5.0,
            huber_threshold: 1.0,
            loop_closure_max_disagreement_sigma: 3.0,
            min_scan_match_distance: 0.05,
            min_scan_match_rotation: 0.02,
            odom_linear_noise: 0.05,
            odom_angular_noise: 0.05,
            odom_linear_to_angular_noise: 0.02,
            odom_angular_to_linear_noise: 0.01,
            imu: ImuPreintegrationConfig::default(),
            scan_context: ScanContextConfig::default(),
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
    pub scan: Arc<PointCloud>,
    /// Timestamp when keyframe was created
    pub timestamp: Instant,
    /// Scan context descriptor for fast loop closure retrieval
    pub descriptor: Option<ScanContextDescriptor>,
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

/// Extended Kalman Filter for 2D pose estimation.
///
/// State vector: [x, y, θ] in world frame.
/// Predicts with body-frame odometry deltas, updates with scan match measurements.
pub struct PoseEkf {
    /// State: [x, y, θ]
    state: Vector3<f64>,
    /// 3x3 covariance matrix
    covariance: Matrix3<f64>,
    /// Process noise scaling
    linear_noise: f64,
    angular_noise: f64,
    linear_to_angular_noise: f64,
    angular_to_linear_noise: f64,
}

impl PoseEkf {
    /// Create a new EKF at the origin with small initial covariance.
    fn new(config: &SlamConfig) -> Self {
        Self {
            state: Vector3::zeros(),
            covariance: Matrix3::from_diagonal(&Vector3::new(1e-4, 1e-4, 1e-4)),
            linear_noise: config.odom_linear_noise,
            angular_noise: config.odom_angular_noise,
            linear_to_angular_noise: config.odom_linear_to_angular_noise,
            angular_to_linear_noise: config.odom_angular_to_linear_noise,
        }
    }

    /// Predict step: propagate state with body-frame odometry delta (dx, dy, dθ).
    fn predict(&mut self, body_dx: f64, body_dy: f64, dtheta: f64) {
        let theta = self.state[2];
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // Rotate body-frame delta into world frame
        let world_dx = cos_t * body_dx - sin_t * body_dy;
        let world_dy = sin_t * body_dx + cos_t * body_dy;

        // State prediction
        self.state[0] += world_dx;
        self.state[1] += world_dy;
        self.state[2] = transforms::normalize_angle(self.state[2] + dtheta);

        // Jacobian of state transition w.r.t. state
        // F = d(f)/d(state) where f projects body delta through current heading
        let f = Matrix3::new(
            1.0, 0.0, -sin_t * body_dx - cos_t * body_dy,
            0.0, 1.0,  cos_t * body_dx - sin_t * body_dy,
            0.0, 0.0, 1.0,
        );

        // Process noise Q scales with motion magnitude
        let dist = (body_dx * body_dx + body_dy * body_dy).sqrt();
        let rot = dtheta.abs();

        let q_x = self.linear_noise * dist + self.angular_to_linear_noise * rot;
        let q_y = self.linear_noise * dist + self.angular_to_linear_noise * rot;
        let q_t = self.angular_noise * rot + self.linear_to_angular_noise * dist;

        // Minimum floor to prevent covariance from collapsing
        let q = Matrix3::from_diagonal(&Vector3::new(
            (q_x * q_x).max(1e-8),
            (q_y * q_y).max(1e-8),
            (q_t * q_t).max(1e-8),
        ));

        // Covariance prediction: P = F * P * F^T + Q
        self.covariance = f * self.covariance * f.transpose() + q;
    }

    /// Update step: incorporate an absolute pose measurement with covariance R.
    /// H = I (direct observation of full state).
    fn update(&mut self, measurement: &Vector3<f64>, r: &Matrix3<f64>) {
        // Innovation: y = z - H*x (with angle wrapping)
        let mut innovation = measurement - self.state;
        innovation[2] = transforms::normalize_angle(innovation[2]);

        // Innovation covariance: S = H*P*H^T + R = P + R
        let s = self.covariance + r;

        // Kalman gain: K = P * H^T * S^{-1} = P * S^{-1}
        let s_inv = match s.try_inverse() {
            Some(inv) => inv,
            None => {
                warn!("EKF update: innovation covariance singular, skipping");
                return;
            }
        };
        let k = self.covariance * s_inv;

        // State update: x = x + K * y
        self.state += k * innovation;
        self.state[2] = transforms::normalize_angle(self.state[2]);

        // Joseph-form covariance update: P = (I - K*H) * P * (I - K*H)^T + K*R*K^T
        // More numerically stable than P = (I - K*H) * P
        let i_kh = Matrix3::identity() - k;
        self.covariance = i_kh * self.covariance * i_kh.transpose() + k * r * k.transpose();
    }

    /// Get current pose as Transform2D.
    fn pose(&self) -> Transform2D {
        Transform2D::new(self.state[0], self.state[1], self.state[2])
    }

    /// Get current covariance matrix.
    fn covariance(&self) -> Matrix3<f64> {
        self.covariance
    }

    /// Reset pose (e.g. after graph optimization) preserving covariance.
    fn reset_pose(&mut self, tf: &Transform2D) {
        self.state[0] = tf.translation().x;
        self.state[1] = tf.translation().y;
        self.state[2] = tf.rotation();
    }
}

/// Main SLAM processor.
pub struct SlamProcessor {
    config: SlamConfig,
    scan_matcher: CorrelativeScanMatcher,
    /// All keyframes
    keyframes: Vec<Keyframe>,
    /// Pose graph edges
    edges: Vec<PoseGraphEdge>,
    /// EKF for probabilistic pose estimation
    ekf: PoseEkf,
    /// Odom -> world correction (updated by SLAM)
    odom_correction: Transform2D,
    /// Last odom pose received
    last_odom_pose: Transform2D,
    /// Pose at last keyframe insertion
    last_keyframe_pose: Transform2D,
    /// Odom pose when the last scan was processed (for computing delta)
    last_scan_odom_pose: Transform2D,
    /// Total loop closures detected
    loop_closure_count: usize,
    /// Pending pre-integrated IMU delta for next scan
    pending_imu_delta: Option<PreintegratedDelta>,
    /// Scan context database for efficient loop closure candidate retrieval
    scan_context_db: ScanContextDatabase,
}

impl SlamProcessor {
    /// Create a new SLAM processor.
    pub fn new(config: SlamConfig) -> Self {
        let scan_config = ScanMatchConfig {
            resolution: config.voxel_size,
            linear_range: 0.5,  // unused in GICP, kept for compat
            angular_range: 0.26, // unused in GICP, kept for compat
            max_iterations: config.max_iterations,
            convergence_threshold: config.convergence_threshold,
            max_correspondence_dist: config.max_correspondence_dist,
            min_overlap: config.min_overlap,
        };

        let ekf = PoseEkf::new(&config);

        let scan_context_db = ScanContextDatabase::new(config.scan_context.clone());

        Self {
            config,
            scan_matcher: CorrelativeScanMatcher::new(scan_config),
            keyframes: Vec::new(),
            edges: Vec::new(),
            ekf,
            odom_correction: Transform2D::identity(),
            last_odom_pose: Transform2D::identity(),
            last_keyframe_pose: Transform2D::identity(),
            last_scan_odom_pose: Transform2D::identity(),
            loop_closure_count: 0,
            pending_imu_delta: None,
            scan_context_db,
        }
    }

    /// Update with new odometry pose (high frequency, ~100Hz).
    /// Call this every control loop iteration.
    pub fn update_odometry(&mut self, odom_pose: &Pose) {
        let odom_tf = Transform2D::from_pose(odom_pose);

        // Compute body-frame delta since last update
        let delta = self.last_odom_pose.relative_to(&odom_tf);
        let body_dx = delta.translation().x;
        let body_dy = delta.translation().y;
        let dtheta = delta.rotation();

        // EKF prediction with body-frame odometry
        self.ekf.predict(body_dx, body_dy, dtheta);

        self.last_odom_pose = odom_tf;
    }

    /// Set the pre-integrated IMU delta for the next scan.
    /// Call this before `process_scan()` with the consumed delta from the preintegrator.
    pub fn set_imu_delta(&mut self, delta: Option<PreintegratedDelta>) {
        self.pending_imu_delta = delta;
    }

    /// Process a new LiDAR scan (lower frequency, ~10Hz).
    /// Returns update info if pose was refined or keyframe added.
    pub fn process_scan(&mut self, scan: &PointCloud) -> Option<SlamUpdate> {
        let scan = Arc::new(scan.clone());

        // Check if we should add a keyframe
        let should_add_keyframe = self.should_add_keyframe();

        // Compute odometry delta since last processed scan
        let odom_delta = self.last_scan_odom_pose.relative_to(&self.last_odom_pose);
        self.last_scan_odom_pose = self.last_odom_pose;

        let odom_distance = odom_delta.translation().norm();
        let odom_rotation = odom_delta.rotation().abs();

        // Motion gate: skip scan matching when stationary to prevent
        // yaw drift from GICP noise on nearly-identical scans.
        let has_moved = odom_distance >= self.config.min_scan_match_distance
            || odom_rotation >= self.config.min_scan_match_rotation;

        // Match against the latest keyframe scan (scan-to-map) instead of
        // the previous scan (scan-to-scan). This prevents accumulation of
        // small per-frame errors that cause drift.
        let reference = self.keyframes.last().map(|kf| kf.scan.clone());

        // Take the pending IMU delta (consumed once per scan)
        let imu_delta = self.pending_imu_delta.take();

        let matched = if has_moved {
            if let Some(ref ref_scan) = reference {
                // Build initial guess for GICP.
                // Start with EKF-based keyframe-relative pose, then optionally
                // override yaw with IMU pre-integrated delta for better turn handling.
                let keyframe_pose = &self.keyframes.last().unwrap().pose;
                let ekf_guess = keyframe_pose.relative_to(&self.ekf.pose());
                let initial_guess = compute_initial_guess(&ekf_guess, imu_delta.as_ref());

                match self.scan_matcher.match_scans(ref_scan, &scan, initial_guess) {
                    Ok(result) => {
                        if result.score > 0.5 {
                            // Compute absolute measurement: keyframe_pose * scan_match_result
                            let measurement_tf = keyframe_pose * &result.transform;
                            let measurement = Vector3::new(
                                measurement_tf.translation().x,
                                measurement_tf.translation().y,
                                measurement_tf.rotation(),
                            );

                            // EKF update with scan match measurement and covariance
                            self.ekf.update(&measurement, &result.covariance);
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
                // No keyframes yet, no reference to match against
                false
            }
        } else {
            debug!(
                odom_dist = odom_distance,
                odom_rot = odom_rotation,
                "Skipping scan match (below motion threshold)"
            );
            false
        };

        let mut keyframe_added = false;
        let mut loop_closure_detected = false;

        if should_add_keyframe {
            keyframe_added = true;
            let keyframe_id = self.keyframes.len();

            // Create keyframe with scan context descriptor
            let current_pose = self.ekf.pose();
            let descriptor = Some(self.compute_descriptor(&scan));
            let keyframe = Keyframe {
                id: keyframe_id,
                pose: current_pose,
                scan: scan.clone(),
                timestamp: Instant::now(),
                descriptor: descriptor.clone(),
            };

            // Insert descriptor into scan context database
            if let Some(ref desc) = descriptor {
                self.scan_context_db.insert(keyframe_id, desc.clone());
            }

            // Add odometry edge from previous keyframe
            if let Some(prev) = self.keyframes.last() {
                let relative_pose = prev.pose.relative_to(&current_pose);
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
            self.last_keyframe_pose = current_pose;

            info!(
                id = keyframe_id,
                x = current_pose.translation().x,
                y = current_pose.translation().y,
                "Added keyframe"
            );
        }

        // Update odom correction based on current pose vs odom pose
        let ekf_pose = self.ekf.pose();
        self.odom_correction = self.last_odom_pose.relative_to(&ekf_pose);

        if keyframe_added || matched {
            Some(SlamUpdate {
                world_pose: ekf_pose.to_pose(),
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
        self.ekf.pose().to_pose()
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

    /// Get the EKF pose covariance as a plain 3x3 array (for watch channels).
    pub fn pose_covariance_array(&self) -> [[f64; 3]; 3] {
        let m = self.ekf.covariance();
        [
            [m[(0, 0)], m[(0, 1)], m[(0, 2)]],
            [m[(1, 0)], m[(1, 1)], m[(1, 2)]],
            [m[(2, 0)], m[(2, 1)], m[(2, 2)]],
        ]
    }

    /// Check if we should add a new keyframe.
    fn should_add_keyframe(&self) -> bool {
        // Always add first keyframe
        if self.keyframes.is_empty() {
            return true;
        }

        let delta = self.last_keyframe_pose.relative_to(&self.ekf.pose());
        let distance = delta.translation().norm();
        let rotation = delta.rotation().abs();

        distance >= self.config.keyframe_distance || rotation >= self.config.keyframe_rotation
    }

    /// Detect loop closure for a new keyframe using scan context descriptors.
    ///
    /// Uses the scan context database for fast top-K candidate retrieval,
    /// then verifies each candidate with spatial proximity, GICP scan matching,
    /// and odometry consistency checks.
    fn detect_loop_closure(&self, new_keyframe: &Keyframe) -> Option<PoseGraphEdge> {
        let descriptor = new_keyframe.descriptor.as_ref()?;
        let current_pos = new_keyframe.pose.translation();
        let skip_recent = self.config.loop_closure_min_nodes;

        // Query scan context database for top-K candidates
        let candidates = self.scan_context_db.find_candidates(descriptor);
        if candidates.is_empty() {
            return None;
        }

        debug!(
            num_candidates = candidates.len(),
            "Scan context loop closure candidates"
        );

        for sc_candidate in &candidates {
            let kid = sc_candidate.keyframe_id;

            // Skip recent keyframes
            if kid + skip_recent >= self.keyframes.len() {
                continue;
            }

            let candidate = &self.keyframes[kid];
            let candidate_pos = candidate.pose.translation();
            let distance = (current_pos - candidate_pos).norm();

            // Spatial proximity check
            if distance > self.config.loop_closure_search_radius {
                continue;
            }

            // Compute initial guess for scan matching
            let initial_guess = candidate.pose.relative_to(&new_keyframe.pose);

            // GICP verification
            match self
                .scan_matcher
                .match_scans(&candidate.scan, &new_keyframe.scan, initial_guess)
            {
                Ok(result) => {
                    if result.score >= self.config.loop_closure_threshold {
                        // Odometry consistency check
                        let match_translation = result.transform.translation().norm();
                        let odom_distance =
                            self.accumulated_odom_distance(candidate.id, new_keyframe.id);
                        let pos_sigma =
                            (result.covariance[(0, 0)] + result.covariance[(1, 1)]).sqrt();
                        let disagreement = (match_translation - odom_distance).abs();
                        let max_sigma = self.config.loop_closure_max_disagreement_sigma;

                        if pos_sigma > 0.0 && disagreement > max_sigma * pos_sigma {
                            warn!(
                                from = candidate.id,
                                to = new_keyframe.id,
                                score = result.score,
                                sc_dist = sc_candidate.distance,
                                disagreement,
                                sigma = pos_sigma,
                                "Loop closure rejected: inconsistent with odometry"
                            );
                            continue;
                        }

                        info!(
                            from = candidate.id,
                            to = new_keyframe.id,
                            score = result.score,
                            sc_dist = sc_candidate.distance,
                            "Loop closure detected (scan context)"
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

    /// Compute a scan context descriptor from a point cloud.
    ///
    /// Uses voxel-downsampled points for consistency across scans
    /// (matches the GICP voxel resolution).
    fn compute_descriptor(&self, scan: &PointCloud) -> ScanContextDescriptor {
        let points: Vec<Vector3<f64>> = scan
            .points
            .iter()
            .filter_map(|p| {
                let range_sq = p.x * p.x + p.y * p.y + p.z * p.z;
                if range_sq.is_finite() && range_sq > 0.01 && range_sq < 2500.0 {
                    Some(Vector3::new(p.x as f64, p.y as f64, p.z as f64))
                } else {
                    None
                }
            })
            .collect();
        ScanContextDescriptor::from_points(&points, &self.config.scan_context)
    }

    /// Get a reference to the scan context database.
    pub fn scan_context_db(&self) -> &ScanContextDatabase {
        &self.scan_context_db
    }

    /// Compute information matrix from scan match result.
    fn compute_information(&self, result: &ScanMatchResult) -> Matrix3<f64> {
        // Use actual scan match covariance when available, fall back to score heuristic
        result
            .covariance
            .try_inverse()
            .unwrap_or_else(|| Matrix3::identity() * (result.score * result.score * 1000.0))
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

        // Update EKF pose if it changed (preserves covariance)
        if let Some(last) = self.keyframes.last() {
            self.ekf.reset_pose(&last.pose);
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

            // Apply Huber robust weighting to loop closure edges
            let omega = if edge.is_loop_closure {
                let residual_norm = error.norm();
                let weight = huber_weight(residual_norm, self.config.huber_threshold);
                edge.information * weight
            } else {
                edge.information
            };

            // H += J^T * Omega * J
            let h_ii = j_i.transpose() * &omega * &j_i;
            let h_ij = j_i.transpose() * &omega * &j_j;
            let h_jj = j_j.transpose() * &omega * &j_j;

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
            let b_i = j_i.transpose() * &omega * &error;
            let b_j = j_j.transpose() * &omega * &error;
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

    /// Compute accumulated odometry distance between two keyframe IDs.
    /// Walks sequential (non-loop-closure) edges between from_id and to_id.
    fn accumulated_odom_distance(&self, from_id: usize, to_id: usize) -> f64 {
        let (lo, hi) = if from_id < to_id {
            (from_id, to_id)
        } else {
            (to_id, from_id)
        };

        let mut total = 0.0;
        for edge in &self.edges {
            if !edge.is_loop_closure && edge.from_id >= lo && edge.to_id <= hi {
                total += edge.measurement.translation().norm();
            }
        }
        total
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

/// Huber robust loss weight function.
///
/// Returns 1.0 for residuals within threshold (quadratic region),
/// decreasing weight for larger residuals (linear region).
fn huber_weight(residual_norm: f64, threshold: f64) -> f64 {
    if residual_norm <= threshold {
        1.0
    } else {
        threshold / residual_norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_scan(angle_offset: f32) -> PointCloud {
        // Create a simple point cloud simulating a box-like environment
        use lidar::Point3D;

        let angle_increment = std::f32::consts::PI * 2.0 / 360.0;
        let points: Vec<Point3D> = (0..360)
            .map(|i| {
                let angle = (i as f32) * angle_increment + angle_offset;
                // Simple box-like environment at z=0
                let dir_x = angle.cos();
                let dir_y = angle.sin();
                let range = if dir_x.abs() > dir_y.abs() {
                    5.0 / dir_x.abs()
                } else {
                    5.0 / dir_y.abs()
                }
                .min(50.0);

                Point3D {
                    x: range * dir_x,
                    y: range * dir_y,
                    z: 0.5, // Mid-height
                    reflectivity: 128,
                    tag: 0,
                }
            })
            .collect();

        PointCloud {
            points,
            ..Default::default()
        }
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

    #[test]
    fn test_huber_weight_regions() {
        let threshold = 1.0;

        // Within threshold: weight = 1.0 (quadratic region)
        assert!((huber_weight(0.0, threshold) - 1.0).abs() < 1e-10);
        assert!((huber_weight(0.5, threshold) - 1.0).abs() < 1e-10);
        assert!((huber_weight(1.0, threshold) - 1.0).abs() < 1e-10);

        // Beyond threshold: weight = threshold / residual (linear region)
        assert!((huber_weight(2.0, threshold) - 0.5).abs() < 1e-10);
        assert!((huber_weight(4.0, threshold) - 0.25).abs() < 1e-10);
        assert!((huber_weight(10.0, threshold) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_pose_covariance_array() {
        let config = SlamConfig::default();
        let processor = SlamProcessor::new(config);

        // EKF always has covariance (small initial values)
        let cov = processor.pose_covariance_array();
        assert!(cov[0][0] > 0.0);
        assert!(cov[1][1] > 0.0);
        assert!(cov[2][2] > 0.0);
    }

    #[test]
    fn test_slam_state_has_covariance_field() {
        let state = SlamState::default();
        // Default has small positive diagonal
        assert!(state.pose_covariance[0][0] > 0.0);
        assert!(state.pose_covariance[1][1] > 0.0);
        assert!(state.pose_covariance[2][2] > 0.0);

        let state = SlamState {
            pose: Pose::default(),
            keyframe_count: 0,
            loop_closure_count: 0,
            keyframe_poses: vec![],
            pose_covariance: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        assert!((state.pose_covariance[0][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ekf_covariance_grows_with_motion() {
        let config = SlamConfig::default();
        let mut processor = SlamProcessor::new(config);

        let initial_cov = processor.pose_covariance_array();

        // Drive forward 1m
        processor.update_odometry(&Pose {
            x: 1.0,
            y: 0.0,
            theta: 0.0,
        });

        let after_motion_cov = processor.pose_covariance_array();

        // Covariance should grow in all dimensions after motion
        assert!(after_motion_cov[0][0] > initial_cov[0][0]);
        assert!(after_motion_cov[1][1] > initial_cov[1][1]);
        assert!(after_motion_cov[2][2] > initial_cov[2][2]);
    }
}
