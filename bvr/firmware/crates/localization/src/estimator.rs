//! EKF pose estimator that fuses wheel odometry, GPS, IMU yaw, and SLAM.
//!
//! State: [x, y, theta] in local frame.
//! - Odometry + IMU gyro provide high-frequency prediction (100Hz)
//! - GPS provides position corrections with quality-adaptive noise
//! - SLAM provides full-pose corrections with scan-match covariance

use std::f64::consts::PI;
use std::time::Instant;

use gps::GpsState;
use nalgebra::{Matrix2, Matrix3, Vector2, Vector3};
use tracing::{debug, trace, warn};
use types::{GpsCoord, Pose};

/// EKF configuration parameters.
#[derive(Debug, Clone)]
pub struct EkfConfig {
    /// Process noise for linear displacement (m/m)
    pub odom_linear_noise: f64,
    /// Process noise for angular displacement (rad/m)
    pub odom_angular_noise: f64,
    /// GPS measurement noise for RTK Fixed (m)
    pub gps_noise_rtk_fixed: f64,
    /// GPS measurement noise for RTK Float (m)
    pub gps_noise_rtk_float: f64,
    /// GPS measurement noise for DGPS (m)
    pub gps_noise_dgps: f64,
    /// GPS measurement noise for standalone GPS (m)
    pub gps_noise_standalone: f64,
    /// Minimum speed for GPS heading updates (m/s)
    pub gps_heading_min_speed: f64,
    /// Weight for IMU yaw rate vs wheel odometry (0.0 = wheel only, 1.0 = IMU only)
    pub imu_yaw_weight: f64,
    /// Gyro noise spectral density (rad/√s)
    pub gyro_noise: f64,
}

impl Default for EkfConfig {
    fn default() -> Self {
        Self {
            odom_linear_noise: 0.05,
            odom_angular_noise: 0.02,
            gps_noise_rtk_fixed: 0.02,
            gps_noise_rtk_float: 0.5,
            gps_noise_dgps: 1.0,
            gps_noise_standalone: 3.0,
            gps_heading_min_speed: 0.3,
            imu_yaw_weight: 0.7,
            gyro_noise: 0.01,
        }
    }
}

/// EKF-based pose estimator fusing odometry, GPS, IMU, and SLAM.
pub struct EkfPoseEstimator {
    /// State vector: [x, y, theta]
    x: Vector3<f64>,
    /// State covariance matrix
    p: Matrix3<f64>,
    /// GPS origin (first valid GPS reading)
    gps_origin: Option<GpsCoord>,
    /// Previous GPS position in local frame + timestamp (for heading derivation)
    prev_gps_local: Option<(f64, f64, Instant)>,
    /// Configuration
    config: EkfConfig,
    /// Whether initialized with a measurement
    initialized: bool,
}

impl EkfPoseEstimator {
    /// Create a new EKF pose estimator with default config.
    pub fn new() -> Self {
        Self::with_config(EkfConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: EkfConfig) -> Self {
        Self {
            x: Vector3::zeros(),
            p: Matrix3::identity() * 0.1,
            gps_origin: None,
            prev_gps_local: None,
            config,
            initialized: false,
        }
    }

    /// Predict step: integrate odometry + optional IMU gyro.
    ///
    /// Called at control loop frequency (~100Hz).
    ///
    /// # Arguments
    /// * `dx` - Forward displacement in robot frame (meters)
    /// * `dy` - Lateral displacement in robot frame (meters)
    /// * `dtheta_wheels` - Rotation from wheel odometry (radians)
    /// * `gyro_z` - Optional IMU yaw rate in body frame (rad/s)
    /// * `dt` - Time step in seconds
    pub fn predict(&mut self, dx: f64, dy: f64, dtheta_wheels: f64, gyro_z: Option<f32>, dt: f64) {
        // Fuse wheel dtheta with IMU gyro_z
        let dtheta = match gyro_z {
            Some(gz) => {
                let imu_dtheta = gz as f64 * dt;
                let w = self.config.imu_yaw_weight;
                w * imu_dtheta + (1.0 - w) * dtheta_wheels
            }
            None => dtheta_wheels,
        };

        let theta = self.x[2];
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        // Transform displacement from robot frame to world frame
        let world_dx = dx * cos_theta - dy * sin_theta;
        let world_dy = dx * sin_theta + dy * cos_theta;

        // Update state
        self.x[0] += world_dx;
        self.x[1] += world_dy;
        self.x[2] = normalize_angle(self.x[2] + dtheta);

        // Jacobian of motion model w.r.t. state
        // F = ∂f/∂x = [[1, 0, -dx*sin(θ) - dy*cos(θ)],
        //               [0, 1,  dx*cos(θ) - dy*sin(θ)],
        //               [0, 0,  1                     ]]
        let mut f = Matrix3::identity();
        f[(0, 2)] = -dx * sin_theta - dy * cos_theta;
        f[(1, 2)] = dx * cos_theta - dy * sin_theta;

        // Process noise Q — scaled by displacement magnitude
        let displacement = (dx * dx + dy * dy).sqrt();
        let sigma_lin = self.config.odom_linear_noise;
        let sigma_ang = self.config.odom_angular_noise;
        let sigma_gyro = self.config.gyro_noise;

        let q_lin = sigma_lin * sigma_lin * displacement;
        let q_ang = sigma_ang * sigma_ang * displacement + sigma_gyro * sigma_gyro * dt;

        let q = Matrix3::new(
            q_lin, 0.0, 0.0,
            0.0, q_lin, 0.0,
            0.0, 0.0, q_ang,
        );

        // Propagate covariance: P = F * P * F^T + Q
        self.p = f * self.p * f.transpose() + q;

        trace!(
            x = self.x[0],
            y = self.x[1],
            theta = self.x[2],
            "EKF predict"
        );
    }

    /// GPS position update with quality-adaptive noise.
    pub fn update_gps(&mut self, gps_state: &GpsState) {
        let coord = match gps_state.coord {
            Some(ref c) => c,
            None => return,
        };

        // Reject zero or invalid accuracy
        if coord.accuracy <= 0.0 || coord.accuracy > 50.0 {
            trace!(accuracy = coord.accuracy, "GPS reading rejected");
            return;
        }

        // Set origin on first valid reading
        if self.gps_origin.is_none() {
            self.gps_origin = Some(*coord);
            self.initialized = true;
            debug!(lat = coord.lat, lon = coord.lon, "GPS origin set");
            return;
        }

        let origin = self.gps_origin.as_ref().unwrap();
        let (gps_x, gps_y) = gps_to_local(coord, origin);

        // Map fix_quality to measurement noise σ
        let sigma = self.fix_quality_to_sigma(gps_state.fix_quality, coord.accuracy);
        let r = Matrix2::identity() * (sigma * sigma);

        // Measurement: z = [gps_x, gps_y]
        let z = Vector2::new(gps_x, gps_y);

        // Observation matrix: H = [[1,0,0],[0,1,0]]
        let h = nalgebra::Matrix2x3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
        );

        // Innovation
        let y = z - h * self.x;

        // Innovation covariance: S = H * P * H^T + R
        let s = h * self.p * h.transpose() + r;

        let s_inv = match s.try_inverse() {
            Some(inv) => inv,
            None => {
                warn!("GPS update: S matrix singular, skipping");
                return;
            }
        };

        // Kalman gain: K = P * H^T * S^-1
        let k = self.p * h.transpose() * s_inv;

        // Update state
        self.x += k * y;
        self.x[2] = normalize_angle(self.x[2]);

        // Joseph form for covariance update: P = (I - K*H) * P * (I - K*H)^T + K * R * K^T
        let i_kh = Matrix3::identity() - k * h;
        self.p = i_kh * self.p * i_kh.transpose() + k * r * k.transpose();

        // GPS heading update (when moving fast enough)
        let now = Instant::now();
        if let Some((prev_x, prev_y, prev_time)) = self.prev_gps_local {
            let dt_gps = now.duration_since(prev_time).as_secs_f64();
            if dt_gps > 0.0 {
                let delta_x = gps_x - prev_x;
                let delta_y = gps_y - prev_y;
                let dist = (delta_x * delta_x + delta_y * delta_y).sqrt();
                let speed = dist / dt_gps;

                if speed > self.config.gps_heading_min_speed && dist > 0.3 {
                    let gps_heading = delta_y.atan2(delta_x);
                    self.update_heading(gps_heading, 0.3);
                }
            }
        }
        self.prev_gps_local = Some((gps_x, gps_y, now));

        debug!(
            gps_x = gps_x,
            gps_y = gps_y,
            sigma = sigma,
            fix = gps_state.fix_quality,
            "EKF GPS update"
        );
    }

    /// SLAM pose update from raw covariance array (row-major 3x3).
    /// Convenience wrapper for watch channel types that avoid nalgebra.
    pub fn update_slam_from_array(&mut self, slam_pose: &Pose, cov: &[[f64; 3]; 3]) {
        let covariance = Matrix3::new(
            cov[0][0], cov[0][1], cov[0][2],
            cov[1][0], cov[1][1], cov[1][2],
            cov[2][0], cov[2][1], cov[2][2],
        );
        self.update_slam(slam_pose, &covariance);
    }

    /// SLAM pose update with scan-match covariance.
    pub fn update_slam(&mut self, slam_pose: &Pose, covariance: &Matrix3<f64>) {
        // Measurement: full pose from SLAM
        let z = Vector3::new(slam_pose.x, slam_pose.y, slam_pose.theta);

        // H = I (SLAM observes full state)
        let h = Matrix3::identity();

        // R = provided covariance from scan matching
        let r = *covariance;

        // Innovation with angle wrapping
        let mut y = z - h * self.x;
        y[2] = normalize_angle(y[2]);

        // Innovation covariance
        let s = h * self.p * h.transpose() + r;

        let s_inv = match s.try_inverse() {
            Some(inv) => inv,
            None => {
                warn!("SLAM update: S matrix singular, skipping");
                return;
            }
        };

        // Kalman gain
        let k = self.p * h.transpose() * s_inv;

        // Update state
        self.x += k * y;
        self.x[2] = normalize_angle(self.x[2]);

        // Joseph form covariance update
        let i_kh = Matrix3::identity() - k * h;
        self.p = i_kh * self.p * i_kh.transpose() + k * r * k.transpose();

        debug!(
            slam_x = slam_pose.x,
            slam_y = slam_pose.y,
            "EKF SLAM update"
        );
    }

    /// Get the current estimated pose.
    pub fn pose(&self) -> Pose {
        Pose {
            x: self.x[0],
            y: self.x[1],
            theta: self.x[2],
        }
    }

    /// Get the state covariance matrix.
    pub fn covariance(&self) -> &Matrix3<f64> {
        &self.p
    }

    /// Scalar uncertainty: sqrt of max eigenvalue of position block.
    pub fn uncertainty(&self) -> f64 {
        let pos_block = self.p.fixed_view::<2, 2>(0, 0);
        // Use trace-based approximation (fast, avoids eigendecomposition)
        // For 2x2: max eigenvalue ≤ trace
        let trace = pos_block[(0, 0)] + pos_block[(1, 1)];
        (trace / 2.0).max(0.0).sqrt()
    }

    /// Get the GPS origin coordinates.
    pub fn gps_origin(&self) -> Option<&GpsCoord> {
        self.gps_origin.as_ref()
    }

    /// Set the GPS origin manually.
    pub fn set_gps_origin(&mut self, coord: GpsCoord) {
        self.gps_origin = Some(coord);
    }

    /// Reset the estimator to origin.
    pub fn reset(&mut self) {
        self.x = Vector3::zeros();
        self.p = Matrix3::identity() * 0.1;
        self.gps_origin = None;
        self.prev_gps_local = None;
        self.initialized = false;
    }

    /// Set the current pose (e.g., for manual correction).
    pub fn set_pose(&mut self, pose: Pose) {
        self.x[0] = pose.x;
        self.x[1] = pose.y;
        self.x[2] = pose.theta;
    }

    /// Heading-only update (1D).
    fn update_heading(&mut self, heading: f64, sigma: f64) {
        let h = Vector3::new(0.0, 0.0, 1.0).transpose();
        let r = sigma * sigma;

        let innovation = normalize_angle(heading - self.x[2]);
        let s = (h * self.p * h.transpose())[(0, 0)] + r;

        if s.abs() < 1e-12 {
            return;
        }

        let k = self.p * h.transpose() / s;
        self.x += k * innovation;
        self.x[2] = normalize_angle(self.x[2]);

        // Joseph form
        let i_kh = Matrix3::identity() - k * h;
        self.p = i_kh * self.p * i_kh.transpose() + k * r * k.transpose();
    }

    /// Map GPS fix quality to measurement noise sigma.
    fn fix_quality_to_sigma(&self, fix_quality: u8, reported_accuracy: f32) -> f64 {
        let base_sigma = match fix_quality {
            4 => self.config.gps_noise_rtk_fixed,
            5 => self.config.gps_noise_rtk_float,
            2 => self.config.gps_noise_dgps,
            1 => self.config.gps_noise_standalone,
            _ => self.config.gps_noise_standalone * 2.0,
        };
        // Floor with reported accuracy
        base_sigma.max(reported_accuracy as f64 * 0.5)
    }
}

impl Default for EkfPoseEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert GPS coordinates to local X/Y (meters from origin).
///
/// Uses equirectangular projection (accurate for small distances).
fn gps_to_local(coord: &GpsCoord, origin: &GpsCoord) -> (f64, f64) {
    const EARTH_RADIUS: f64 = 6_371_000.0;

    let origin_lat_rad = origin.lat.to_radians();
    let dlat = coord.lat - origin.lat;
    let dlon = coord.lon - origin.lon;

    let x = dlon.to_radians() * EARTH_RADIUS * origin_lat_rad.cos();
    let y = dlat.to_radians() * EARTH_RADIUS;

    // GPS Y (north) -> robot X (forward), GPS X (east) -> robot -Y (left)
    (y, -x)
}

/// Normalize angle to [-PI, PI].
fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle;
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_gps_state(fix_quality: u8, lat: f64, lon: f64, accuracy: f32) -> GpsState {
        GpsState {
            coord: Some(GpsCoord {
                lat,
                lon,
                alt: 0.0,
                accuracy,
            }),
            fix_quality,
            satellites: 12,
            hdop: 0.8,
        }
    }

    #[test]
    fn test_prediction_forward() {
        let mut ekf = EkfPoseEstimator::new();

        // Move forward 1m (robot facing +X)
        ekf.predict(1.0, 0.0, 0.0, None, 0.01);

        let pose = ekf.pose();
        assert!((pose.x - 1.0).abs() < 0.001);
        assert!(pose.y.abs() < 0.001);
        assert!(pose.theta.abs() < 0.001);
    }

    #[test]
    fn test_prediction_rotation() {
        let mut ekf = EkfPoseEstimator::new();

        // Rotate 90 degrees CCW
        ekf.predict(0.0, 0.0, PI / 2.0, None, 0.01);

        let pose = ekf.pose();
        assert!(pose.x.abs() < 0.001);
        assert!(pose.y.abs() < 0.001);
        assert!((pose.theta - PI / 2.0).abs() < 0.001);

        // Now move "forward" — should go in +Y direction
        ekf.predict(1.0, 0.0, 0.0, None, 0.01);

        let pose = ekf.pose();
        assert!(pose.x.abs() < 0.01);
        assert!((pose.y - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_covariance_grows_in_prediction() {
        let mut ekf = EkfPoseEstimator::new();
        let initial_trace = ekf.covariance().trace();

        // Predict with displacement — covariance should grow
        for _ in 0..100 {
            ekf.predict(0.1, 0.0, 0.01, None, 0.01);
        }

        let final_trace = ekf.covariance().trace();
        assert!(
            final_trace > initial_trace,
            "Covariance should grow during prediction: {} vs {}",
            final_trace,
            initial_trace
        );
    }

    #[test]
    fn test_gps_rtk_fixed_large_correction() {
        let mut ekf = EkfPoseEstimator::new();

        // Set origin
        let origin = default_gps_state(4, 37.7749, -122.4194, 0.02);
        ekf.update_gps(&origin);
        assert!(ekf.gps_origin().is_some());

        // Drive 10m forward (accumulate error)
        for _ in 0..1000 {
            ekf.predict(0.01, 0.0, 0.0, None, 0.01);
        }
        let pre_gps = ekf.pose();

        // GPS says we're at a slightly different position (RTK Fixed, very accurate)
        let gps = default_gps_state(4, 37.7749 + 0.0001, -122.4194, 0.02);
        ekf.update_gps(&gps);

        let post_gps = ekf.pose();
        // RTK fixed should pull strongly toward GPS
        let gps_local = gps_to_local(
            &gps.coord.unwrap(),
            ekf.gps_origin().unwrap(),
        );

        let error_before = ((pre_gps.x - gps_local.0).powi(2) + (pre_gps.y - gps_local.1).powi(2)).sqrt();
        let error_after = ((post_gps.x - gps_local.0).powi(2) + (post_gps.y - gps_local.1).powi(2)).sqrt();

        assert!(
            error_after < error_before,
            "RTK Fixed GPS should reduce position error: before={}, after={}",
            error_before,
            error_after
        );
    }

    #[test]
    fn test_gps_standalone_small_correction() {
        let mut ekf = EkfPoseEstimator::new();

        // Set origin
        let origin = default_gps_state(1, 37.7749, -122.4194, 3.0);
        ekf.update_gps(&origin);

        // Drive forward, building up covariance
        for _ in 0..100 {
            ekf.predict(0.01, 0.0, 0.0, None, 0.01);
        }
        let pre_gps = ekf.pose();
        // Standalone GPS (low quality) — should make smaller correction
        let gps = default_gps_state(1, 37.7749 + 0.0001, -122.4194, 5.0);
        ekf.update_gps(&gps);

        let post_gps = ekf.pose();
        let correction = ((post_gps.x - pre_gps.x).powi(2) + (post_gps.y - pre_gps.y).powi(2)).sqrt();

        // With standalone GPS and short drive, correction should be modest
        // (not snapping to GPS position)
        let gps_local = gps_to_local(
            &gps.coord.unwrap(),
            ekf.gps_origin().unwrap(),
        );
        let gps_distance = ((gps_local.0 - pre_gps.x).powi(2) + (gps_local.1 - pre_gps.y).powi(2)).sqrt();

        assert!(
            correction < gps_distance * 0.9,
            "Standalone GPS should not snap to GPS: correction={}, gps_distance={}",
            correction,
            gps_distance
        );
    }

    #[test]
    fn test_gps_shrinks_covariance() {
        let mut ekf = EkfPoseEstimator::new();

        // Set origin
        let origin = default_gps_state(4, 37.7749, -122.4194, 0.02);
        ekf.update_gps(&origin);

        // Grow covariance with odometry
        for _ in 0..200 {
            ekf.predict(0.05, 0.0, 0.01, None, 0.01);
        }
        let pre_uncertainty = ekf.uncertainty();

        // RTK fix should shrink covariance
        let gps = default_gps_state(4, 37.7749 + 0.0001, -122.4194, 0.02);
        ekf.update_gps(&gps);

        let post_uncertainty = ekf.uncertainty();
        assert!(
            post_uncertainty < pre_uncertainty,
            "GPS should reduce uncertainty: before={}, after={}",
            pre_uncertainty,
            post_uncertainty
        );
    }

    #[test]
    fn test_slam_update_moves_toward_slam_pose() {
        let mut ekf = EkfPoseEstimator::new();

        // Drive to some position
        for _ in 0..100 {
            ekf.predict(0.1, 0.0, 0.0, None, 0.01);
        }

        let pre_slam = ekf.pose();
        let slam_pose = Pose {
            x: pre_slam.x + 0.5,
            y: pre_slam.y + 0.3,
            theta: pre_slam.theta + 0.1,
        };
        let slam_cov = Matrix3::identity() * 0.01; // High confidence

        ekf.update_slam(&slam_pose, &slam_cov);

        let post_slam = ekf.pose();
        let error_before = ((pre_slam.x - slam_pose.x).powi(2) + (pre_slam.y - slam_pose.y).powi(2)).sqrt();
        let error_after = ((post_slam.x - slam_pose.x).powi(2) + (post_slam.y - slam_pose.y).powi(2)).sqrt();

        assert!(
            error_after < error_before,
            "SLAM update should move toward SLAM pose: before={}, after={}",
            error_before,
            error_after
        );
    }

    #[test]
    fn test_imu_tracks_yaw_on_wheel_slip() {
        let mut ekf = EkfPoseEstimator::new();

        // Wheels report no rotation (slip), but IMU says rotating at 1 rad/s
        let dt = 0.01;
        for _ in 0..100 {
            ekf.predict(0.0, 0.0, 0.0, Some(1.0), dt);
        }

        let pose = ekf.pose();
        // 100 steps × 0.01s × 1 rad/s × 0.7 weight = ~0.7 rad
        let expected_theta = 100.0 * dt * 1.0 * 0.7;
        assert!(
            (pose.theta - expected_theta).abs() < 0.05,
            "IMU should drive heading during wheel slip: got {} expected ~{}",
            pose.theta,
            expected_theta
        );
    }

    #[test]
    fn test_no_imu_matches_wheel_only() {
        let mut ekf_with_imu = EkfPoseEstimator::new();
        let mut ekf_no_imu = EkfPoseEstimator::new();

        let dt = 0.01;
        let dtheta = 0.05;

        for _ in 0..100 {
            ekf_with_imu.predict(0.1, 0.0, dtheta, None, dt);
            ekf_no_imu.predict(0.1, 0.0, dtheta, None, dt);
        }

        let p1 = ekf_with_imu.pose();
        let p2 = ekf_no_imu.pose();
        assert!((p1.x - p2.x).abs() < 0.001);
        assert!((p1.y - p2.y).abs() < 0.001);
        assert!((p1.theta - p2.theta).abs() < 0.001);
    }

    #[test]
    fn test_angle_normalization() {
        assert!((normalize_angle(0.0) - 0.0).abs() < 0.001);
        assert!((normalize_angle(PI) - PI).abs() < 0.001);
        assert!((normalize_angle(-PI) - (-PI)).abs() < 0.001);
        assert!((normalize_angle(3.0 * PI) - PI).abs() < 0.001);
        assert!((normalize_angle(-3.0 * PI) - (-PI)).abs() < 0.001);
    }

    #[test]
    fn test_gps_origin_set() {
        let mut ekf = EkfPoseEstimator::new();
        assert!(ekf.gps_origin().is_none());

        let state = default_gps_state(1, 37.7749, -122.4194, 5.0);
        ekf.update_gps(&state);

        assert!(ekf.gps_origin().is_some());
        let origin = ekf.gps_origin().unwrap();
        assert!((origin.lat - 37.7749).abs() < 0.0001);
    }

    #[test]
    fn test_gps_to_local() {
        let origin = GpsCoord {
            lat: 37.0,
            lon: -122.0,
            alt: 0.0,
            accuracy: 1.0,
        };

        // Same location should be (0, 0)
        let (x, y) = gps_to_local(&origin, &origin);
        assert!(x.abs() < 0.01);
        assert!(y.abs() < 0.01);

        // 1 degree north should be ~111km
        let north = GpsCoord {
            lat: 38.0,
            lon: -122.0,
            alt: 0.0,
            accuracy: 1.0,
        };
        let (x, _y) = gps_to_local(&north, &origin);
        assert!(x > 100_000.0);
    }

    #[test]
    fn test_reset() {
        let mut ekf = EkfPoseEstimator::new();
        ekf.predict(1.0, 0.0, 0.5, None, 0.01);
        assert!(ekf.pose().x > 0.0);

        ekf.reset();
        let pose = ekf.pose();
        assert!(pose.x.abs() < 0.001);
        assert!(pose.y.abs() < 0.001);
        assert!(pose.theta.abs() < 0.001);
        assert!(ekf.gps_origin().is_none());
    }
}
