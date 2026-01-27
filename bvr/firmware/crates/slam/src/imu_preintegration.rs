//! IMU pre-integration between LiDAR scans.
//!
//! Accumulates gyro-Z samples at ~200Hz between ~10Hz LiDAR scans,
//! producing a pre-integrated yaw delta with variance estimate.
//! This is combined with wheel odometry translation to form a hybrid
//! initial guess for GICP that is more robust during turns.

/// Configuration for IMU pre-integration.
#[derive(Debug, Clone)]
pub struct ImuPreintegrationConfig {
    /// Gyro noise density (rad/s/sqrt(Hz)). BMI088 typical: 0.014
    pub gyro_noise_density: f64,
    /// Gyro bias random walk (rad/s²/sqrt(Hz)). BMI088 typical: 0.0003
    pub gyro_bias_rw: f64,
    /// Enable IMU pre-integration
    pub enabled: bool,
}

impl Default for ImuPreintegrationConfig {
    fn default() -> Self {
        Self {
            gyro_noise_density: 0.014,
            gyro_bias_rw: 0.0003,
            enabled: true,
        }
    }
}

/// Result of consuming accumulated IMU samples between scans.
#[derive(Debug, Clone, Copy)]
pub struct PreintegratedDelta {
    /// Accumulated yaw rotation (radians)
    pub delta_yaw: f64,
    /// Variance of the accumulated yaw (rad²)
    pub yaw_variance: f64,
    /// Number of samples integrated
    pub sample_count: u32,
}

/// Accumulates gyro-Z samples between LiDAR scans.
///
/// Call [`integrate`] at the IMU rate (~200Hz) with each compensated
/// body-frame gyro-Z sample, then [`consume`] when a LiDAR scan arrives
/// to retrieve and reset the accumulated delta.
pub struct ImuPreintegrator {
    config: ImuPreintegrationConfig,
    /// Accumulated yaw delta (radians)
    delta_yaw: f64,
    /// Accumulated yaw variance (rad²)
    yaw_variance: f64,
    /// Number of samples accumulated
    sample_count: u32,
    /// Estimated gyro bias (rad/s), slowly updated
    gyro_bias: f64,
    /// Motion threshold: skip integration when gyro rate is below this (rad/s)
    stationary_threshold: f64,
}

impl ImuPreintegrator {
    /// Create a new pre-integrator.
    pub fn new(config: ImuPreintegrationConfig) -> Self {
        Self {
            config,
            delta_yaw: 0.0,
            yaw_variance: 0.0,
            sample_count: 0,
            gyro_bias: 0.0,
            stationary_threshold: 0.01, // ~0.6 deg/s
        }
    }

    /// Integrate a single gyro-Z sample.
    ///
    /// `gyro_z_body` should already be compensated for the IMU mounting pitch
    /// (i.e. `-gyro_x * sin(pitch) + gyro_z * cos(pitch)`).
    /// `dt` is the time since the last sample (seconds).
    pub fn integrate(&mut self, gyro_z_body: f64, dt: f64) {
        if !self.config.enabled || dt <= 0.0 || !dt.is_finite() {
            return;
        }

        let rate = gyro_z_body - self.gyro_bias;

        // Skip integration when stationary to prevent bias drift
        if rate.abs() < self.stationary_threshold {
            // Update bias estimate toward current reading when stationary
            // Simple exponential moving average
            self.gyro_bias += (gyro_z_body - self.gyro_bias) * 0.001;
            self.sample_count += 1;
            return;
        }

        self.delta_yaw += rate * dt;

        // Variance propagation:
        //   σ² += (noise_density² / dt) * dt² = noise_density² * dt
        // Plus bias random walk contribution:
        //   σ² += bias_rw² * dt
        let nd = self.config.gyro_noise_density;
        let brw = self.config.gyro_bias_rw;
        self.yaw_variance += nd * nd * dt + brw * brw * dt;

        self.sample_count += 1;
    }

    /// Consume the accumulated pre-integrated delta, resetting for the next interval.
    ///
    /// Returns `None` if no samples were integrated or pre-integration is disabled.
    pub fn consume(&mut self) -> Option<PreintegratedDelta> {
        if !self.config.enabled || self.sample_count == 0 {
            return None;
        }

        let delta = PreintegratedDelta {
            delta_yaw: self.delta_yaw,
            yaw_variance: self.yaw_variance,
            sample_count: self.sample_count,
        };

        // Reset accumulators
        self.delta_yaw = 0.0;
        self.yaw_variance = 0.0;
        self.sample_count = 0;

        Some(delta)
    }

    /// Check if pre-integration is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Compute a hybrid initial guess combining odometry translation with IMU yaw.
///
/// Uses odometry for (x, y) translation and the pre-integrated IMU delta for yaw.
/// Falls back to pure odometry when no IMU delta is available.
pub fn compute_initial_guess(
    odom_delta: &transforms::Transform2D,
    imu_delta: Option<&PreintegratedDelta>,
) -> transforms::Transform2D {
    match imu_delta {
        Some(delta) if delta.sample_count > 0 => {
            // Use odom translation but IMU yaw
            transforms::Transform2D::new(
                odom_delta.translation().x,
                odom_delta.translation().y,
                delta.delta_yaw,
            )
        }
        _ => *odom_delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_accumulation() {
        let config = ImuPreintegrationConfig::default();
        let mut pre = ImuPreintegrator::new(config);

        // Simulate 100ms of rotation at 1 rad/s (200Hz = 20 samples)
        let dt = 0.005; // 200Hz
        let rate = 1.0_f64; // rad/s
        for _ in 0..20 {
            pre.integrate(rate, dt);
        }

        let delta = pre.consume().unwrap();
        assert!((delta.delta_yaw - 0.1).abs() < 0.01, "Expected ~0.1 rad, got {}", delta.delta_yaw);
        assert!(delta.yaw_variance > 0.0);
        assert_eq!(delta.sample_count, 20);
    }

    #[test]
    fn test_stationary_skip() {
        let config = ImuPreintegrationConfig::default();
        let mut pre = ImuPreintegrator::new(config);

        // Below stationary threshold (0.01 rad/s)
        let dt = 0.005;
        for _ in 0..20 {
            pre.integrate(0.005, dt);
        }

        let delta = pre.consume().unwrap();
        // Should have accumulated nearly zero yaw since rate is below threshold
        assert!(delta.delta_yaw.abs() < 0.001, "Expected ~0 rad, got {}", delta.delta_yaw);
    }

    #[test]
    fn test_consume_resets() {
        let config = ImuPreintegrationConfig::default();
        let mut pre = ImuPreintegrator::new(config);

        pre.integrate(1.0, 0.005);
        let first = pre.consume().unwrap();
        assert!(first.delta_yaw.abs() > 0.0);

        // Second consume without new data returns None
        assert!(pre.consume().is_none());
    }

    #[test]
    fn test_disabled() {
        let config = ImuPreintegrationConfig {
            enabled: false,
            ..Default::default()
        };
        let mut pre = ImuPreintegrator::new(config);

        pre.integrate(1.0, 0.005);
        assert!(pre.consume().is_none());
    }

    #[test]
    fn test_mounting_pitch_transform() {
        // Verify the same transform used in main.rs works correctly
        let mounting_pitch_deg: f64 = 30.0;
        let pitch_rad = mounting_pitch_deg.to_radians();
        let (sin_p, cos_p) = pitch_rad.sin_cos();

        // Pure yaw rotation in sensor frame: gyro_x=0, gyro_z=1.0
        let gyro_z_body = -0.0 * sin_p + 1.0 * cos_p;
        assert!((gyro_z_body - cos_p).abs() < 1e-10);

        // Pure roll in sensor frame: gyro_x=1.0, gyro_z=0
        let gyro_z_body = -1.0 * sin_p + 0.0 * cos_p;
        assert!((gyro_z_body - (-sin_p)).abs() < 1e-10);
    }

    #[test]
    fn test_compute_initial_guess_with_imu() {
        let odom = transforms::Transform2D::new(0.5, 0.1, 0.3);
        let imu = PreintegratedDelta {
            delta_yaw: 0.5,
            yaw_variance: 0.001,
            sample_count: 20,
        };

        let guess = compute_initial_guess(&odom, Some(&imu));
        // Translation from odom
        assert!((guess.translation().x - 0.5).abs() < 1e-10);
        assert!((guess.translation().y - 0.1).abs() < 1e-10);
        // Yaw from IMU
        assert!((guess.rotation() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_compute_initial_guess_fallback() {
        let odom = transforms::Transform2D::new(0.5, 0.1, 0.3);
        let guess = compute_initial_guess(&odom, None);
        assert!((guess.rotation() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_variance_grows_with_time() {
        let config = ImuPreintegrationConfig::default();
        let mut pre = ImuPreintegrator::new(config);

        let dt = 0.005;
        let rate = 1.0;

        pre.integrate(rate, dt);
        let v1 = pre.yaw_variance;

        pre.integrate(rate, dt);
        let v2 = pre.yaw_variance;

        assert!(v2 > v1, "Variance should grow monotonically");
    }
}
