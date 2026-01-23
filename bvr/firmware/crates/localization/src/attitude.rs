//! IMU attitude estimation using an Extended Kalman Filter.
//!
//! Fuses gyroscope rate measurements with accelerometer gravity reference
//! to estimate roll and pitch angles with gyro bias correction.

use nalgebra::{Matrix2x4, Matrix4, Vector2, Vector4};

/// Estimated attitude (roll and pitch).
#[derive(Debug, Clone, Copy, Default)]
pub struct Attitude {
    /// Roll angle in radians (rotation about X/forward axis).
    pub roll: f32,
    /// Pitch angle in radians (rotation about Y/lateral axis).
    pub pitch: f32,
}

/// Configuration parameters for the attitude filter.
#[derive(Debug, Clone)]
pub struct AttitudeConfig {
    /// Process noise for angle states (rad²/s).
    pub q_angle: f64,
    /// Process noise for gyro bias states (rad²/s³).
    pub q_bias: f64,
    /// Base measurement noise for accelerometer (rad²).
    pub r_accel: f64,
    /// Threshold for accelerometer magnitude deviation to increase R (m/s²).
    pub accel_threshold: f64,
    /// Gravity magnitude (m/s²).
    pub gravity: f64,
}

impl Default for AttitudeConfig {
    fn default() -> Self {
        Self {
            q_angle: 0.001,
            q_bias: 1e-7,
            r_accel: 0.01,
            accel_threshold: 0.5,
            gravity: 9.81,
        }
    }
}

/// Extended Kalman Filter for attitude estimation.
///
/// State vector: `[roll, pitch, gyro_bias_x, gyro_bias_y]`
/// - Gyroscope provides fast angular rate updates
/// - Accelerometer provides drift correction via gravity reference
/// - Bias estimation improves long-term stability
pub struct AttitudeFilter {
    /// State vector: [roll, pitch, bias_gx, bias_gy]
    x: Vector4<f64>,
    /// State covariance matrix
    p: Matrix4<f64>,
    /// Filter configuration
    config: AttitudeConfig,
    /// Whether the filter has been initialized with a measurement
    initialized: bool,
}

impl Default for AttitudeFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl AttitudeFilter {
    /// Create a new attitude filter with default configuration.
    pub fn new() -> Self {
        Self::with_config(AttitudeConfig::default())
    }

    /// Create a new attitude filter with custom configuration.
    pub fn with_config(config: AttitudeConfig) -> Self {
        Self {
            x: Vector4::zeros(),
            p: Matrix4::identity() * 0.1, // Initial uncertainty
            config,
            initialized: false,
        }
    }

    /// Update the filter with new IMU measurements.
    ///
    /// # Arguments
    /// * `gyro` - Gyroscope readings [x, y, z] in rad/s (body frame)
    /// * `accel` - Accelerometer readings [x, y, z] in m/s² (body frame)
    /// * `dt` - Time step in seconds
    ///
    /// # Returns
    /// Current attitude estimate
    pub fn update(&mut self, gyro: [f32; 3], accel: [f32; 3], dt: f64) -> Attitude {
        let gyro = [gyro[0] as f64, gyro[1] as f64, gyro[2] as f64];
        let accel = [accel[0] as f64, accel[1] as f64, accel[2] as f64];

        // Compute accelerometer-derived angles for initialization or measurement update
        let accel_roll = accel[1].atan2(accel[2]);
        let accel_pitch = (-accel[0]).atan2((accel[1].powi(2) + accel[2].powi(2)).sqrt());

        // Initialize on first measurement
        if !self.initialized {
            self.x[0] = accel_roll;
            self.x[1] = accel_pitch;
            self.x[2] = 0.0; // Initial gyro bias X
            self.x[3] = 0.0; // Initial gyro bias Y
            self.initialized = true;
            return self.attitude();
        }

        // === PREDICT STEP ===
        // State transition: integrate gyro rates (corrected for bias)
        let gyro_x_corrected = gyro[0] - self.x[2];
        let gyro_y_corrected = gyro[1] - self.x[3];

        // Predict new state
        // Roll rate = gyro_x + sin(roll)*tan(pitch)*gyro_y + cos(roll)*tan(pitch)*gyro_z
        // Pitch rate = cos(roll)*gyro_y - sin(roll)*gyro_z
        // For small angles, simplify to direct integration
        let (sin_roll, cos_roll) = self.x[0].sin_cos();
        let tan_pitch = self.x[1].tan();

        let roll_rate = gyro_x_corrected
            + sin_roll * tan_pitch * gyro_y_corrected
            + cos_roll * tan_pitch * (gyro[2] - 0.0); // No z-bias estimation
        let pitch_rate = cos_roll * gyro_y_corrected - sin_roll * (gyro[2] - 0.0);

        self.x[0] += roll_rate * dt;
        self.x[1] += pitch_rate * dt;
        // Biases evolve as random walk (no change in predict)

        // State transition Jacobian (linearized around current state)
        // For simplicity, use identity for angles and approximate derivatives
        let mut f = Matrix4::identity();
        f[(0, 2)] = -dt; // d(roll)/d(bias_x)
        f[(1, 3)] = -dt; // d(pitch)/d(bias_y)

        // Process noise covariance
        let mut q = Matrix4::zeros();
        q[(0, 0)] = self.config.q_angle * dt;
        q[(1, 1)] = self.config.q_angle * dt;
        q[(2, 2)] = self.config.q_bias * dt;
        q[(3, 3)] = self.config.q_bias * dt;

        // Propagate covariance: P = F * P * F' + Q
        self.p = f * self.p * f.transpose() + q;

        // === UPDATE STEP ===
        // Measurement: accelerometer-derived roll and pitch
        let z = Vector2::new(accel_roll, accel_pitch);

        // Measurement matrix (H): maps state to measurement
        // z = H * x, where we measure roll and pitch directly
        let mut h = Matrix2x4::zeros();
        h[(0, 0)] = 1.0; // roll
        h[(1, 1)] = 1.0; // pitch

        // Adaptive measurement noise based on acceleration magnitude
        let accel_mag = (accel[0].powi(2) + accel[1].powi(2) + accel[2].powi(2)).sqrt();
        let accel_deviation = (accel_mag - self.config.gravity).abs();

        // Increase R when accelerometer deviates from gravity (motion detected)
        let r_scale = if accel_deviation > self.config.accel_threshold {
            // Quadratic scaling: larger deviations = much higher noise
            1.0 + (accel_deviation / self.config.accel_threshold).powi(2) * 10.0
        } else {
            1.0
        };

        let r = nalgebra::Matrix2::identity() * self.config.r_accel * r_scale;

        // Innovation (measurement residual)
        let y = z - h * self.x;

        // Innovation covariance: S = H * P * H' + R
        let s = h * self.p * h.transpose() + r;

        // Kalman gain: K = P * H' * S^-1
        let s_inv = match s.try_inverse() {
            Some(inv) => inv,
            None => {
                // Fallback: return current estimate if S is singular
                return self.attitude();
            }
        };
        let k = self.p * h.transpose() * s_inv;

        // Update state: x = x + K * y
        self.x += k * y;

        // Update covariance: P = (I - K * H) * P
        // Use Joseph form for numerical stability: P = (I - KH) * P * (I - KH)' + K * R * K'
        let i_kh = Matrix4::identity() - k * h;
        self.p = i_kh * self.p * i_kh.transpose() + k * r * k.transpose();

        // Normalize angles to [-π, π]
        self.x[0] = normalize_angle(self.x[0]);
        self.x[1] = normalize_angle(self.x[1]);

        self.attitude()
    }

    /// Get the current attitude estimate without updating.
    pub fn attitude(&self) -> Attitude {
        Attitude {
            roll: self.x[0] as f32,
            pitch: self.x[1] as f32,
        }
    }

    /// Get the estimated gyroscope biases [bias_x, bias_y] in rad/s.
    pub fn gyro_bias(&self) -> [f64; 2] {
        [self.x[2], self.x[3]]
    }

    /// Reset the filter to uninitialized state.
    pub fn reset(&mut self) {
        self.x = Vector4::zeros();
        self.p = Matrix4::identity() * 0.1;
        self.initialized = false;
    }
}

/// Normalize angle to [-π, π].
fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle % (2.0 * std::f64::consts::PI);
    if a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    } else if a < -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAVITY: f32 = 9.81;
    const DT: f64 = 0.01; // 100 Hz

    #[test]
    fn test_level_imu() {
        let mut filter = AttitudeFilter::new();

        // Level IMU: gravity pointing down (-Z)
        let gyro = [0.0, 0.0, 0.0];
        let accel = [0.0, 0.0, GRAVITY];

        // Run a few iterations to stabilize
        for _ in 0..100 {
            filter.update(gyro, accel, DT);
        }

        let att = filter.attitude();
        assert!(
            att.roll.abs() < 0.01,
            "Roll should be ~0 for level IMU, got {}",
            att.roll
        );
        assert!(
            att.pitch.abs() < 0.01,
            "Pitch should be ~0 for level IMU, got {}",
            att.pitch
        );
    }

    #[test]
    fn test_known_tilt_roll() {
        let mut filter = AttitudeFilter::new();

        // 30° roll: gravity has Y and Z components
        let roll_rad = 30.0_f32.to_radians();
        let gyro = [0.0, 0.0, 0.0];
        let accel = [0.0, GRAVITY * roll_rad.sin(), GRAVITY * roll_rad.cos()];

        for _ in 0..100 {
            filter.update(gyro, accel, DT);
        }

        let att = filter.attitude();
        let error = (att.roll - roll_rad).abs();
        assert!(
            error < 0.02,
            "Roll error {} rad exceeds threshold for 30° tilt",
            error
        );
    }

    #[test]
    fn test_known_tilt_pitch() {
        let mut filter = AttitudeFilter::new();

        // 20° pitch (nose up): gravity has -X and Z components
        let pitch_rad = 20.0_f32.to_radians();
        let gyro = [0.0, 0.0, 0.0];
        let accel = [
            -GRAVITY * pitch_rad.sin(),
            0.0,
            GRAVITY * pitch_rad.cos(),
        ];

        for _ in 0..100 {
            filter.update(gyro, accel, DT);
        }

        let att = filter.attitude();
        let error = (att.pitch - pitch_rad).abs();
        assert!(
            error < 0.02,
            "Pitch error {} rad exceeds threshold for 20° tilt",
            error
        );
    }

    #[test]
    fn test_noise_rejection() {
        use std::f32::consts::PI;

        let mut filter = AttitudeFilter::new();
        let mut rng_state: u32 = 12345;

        // Simple LCG for deterministic "random" noise
        let mut next_noise = |scale: f32| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            ((rng_state >> 16) as f32 / 32768.0 - 1.0) * scale
        };

        // Collect samples with noisy measurements
        let mut roll_samples = Vec::new();
        let mut iteration = 0;
        for _ in 0..500 {
            // Gyro noise: ±0.01 rad/s (typical MEMS noise)
            let gyro = [next_noise(0.01), next_noise(0.01), next_noise(0.01)];
            // Accel noise: ±0.05 m/s² around gravity
            let accel = [next_noise(0.05), next_noise(0.05), GRAVITY + next_noise(0.05)];

            let att = filter.update(gyro, accel, DT);
            iteration += 1;
            if iteration > 100 {
                // Skip initial transient
                roll_samples.push(att.roll);
            }
        }

        // Compute standard deviation
        let mean: f32 = roll_samples.iter().sum::<f32>() / roll_samples.len() as f32;
        let variance: f32 =
            roll_samples.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / roll_samples.len() as f32;
        let std_dev = variance.sqrt();

        // Should be significantly less than raw accelerometer noise
        let std_dev_deg = std_dev * 180.0 / PI;
        assert!(
            std_dev_deg < 0.5,
            "Filtered noise std dev {} deg exceeds 0.5 deg threshold",
            std_dev_deg
        );
    }

    #[test]
    fn test_bias_estimation() {
        let mut filter = AttitudeFilter::new();

        // Simulate gyro with constant bias
        let true_bias_x = 0.01; // 0.01 rad/s bias
        let gyro = [true_bias_x as f32, 0.0, 0.0];
        let accel = [0.0, 0.0, GRAVITY]; // Level

        // Run long enough for bias to converge
        for _ in 0..1000 {
            filter.update(gyro, accel, DT);
        }

        let [bias_x, _] = filter.gyro_bias();

        // Bias estimate should approach true bias
        let bias_error = (bias_x - true_bias_x).abs();
        assert!(
            bias_error < 0.005,
            "Bias estimation error {} rad/s exceeds threshold",
            bias_error
        );
    }

    #[test]
    fn test_reset() {
        let mut filter = AttitudeFilter::new();

        // Initialize with some data
        filter.update([0.0, 0.0, 0.0], [0.0, 0.0, GRAVITY], DT);
        assert!(filter.initialized);

        // Reset
        filter.reset();
        assert!(!filter.initialized);

        // Should re-initialize on next update
        filter.update([0.0, 0.0, 0.0], [0.0, 0.0, GRAVITY], DT);
        assert!(filter.initialized);
    }
}
