//! Observation representation for the RL environment.

use sim::lidar::LidarScan;

/// Configuration for observation generation.
#[derive(Debug, Clone)]
pub struct ObservationConfig {
    /// Number of LiDAR range samples in observation
    pub lidar_samples: usize,
    /// Whether to normalize observations to [-1, 1]
    pub normalize: bool,
    /// Max range for LiDAR normalization
    pub max_lidar_range: f32,
    /// Max position for pose normalization
    pub max_position: f32,
    /// Max velocity for normalization
    pub max_linear_vel: f32,
    /// Max angular velocity for normalization
    pub max_angular_vel: f32,
}

impl Default for ObservationConfig {
    fn default() -> Self {
        Self {
            lidar_samples: 36, // 10-degree resolution
            normalize: true,
            max_lidar_range: 40.0,
            max_position: 50.0,
            max_linear_vel: 2.0,
            max_angular_vel: 2.0,
        }
    }
}

/// Full observation from the environment.
#[derive(Debug, Clone)]
pub struct Observation {
    /// Robot pose [x, y, theta]
    pub pose: [f32; 3],
    /// Robot velocity [linear, angular]
    pub velocity: [f32; 2],
    /// Goal position relative to robot [dx, dy]
    pub goal_relative: [f32; 2],
    /// LiDAR range measurements (downsampled)
    pub lidar_ranges: Vec<f32>,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            pose: [0.0; 3],
            velocity: [0.0; 2],
            goal_relative: [0.0; 2],
            lidar_ranges: Vec::new(),
        }
    }
}

impl Observation {
    /// Create observation from environment state.
    pub fn from_state(
        x: f64,
        y: f64,
        theta: f64,
        linear_vel: f64,
        angular_vel: f64,
        goal_x: f64,
        goal_y: f64,
        lidar_scan: &LidarScan,
        config: &ObservationConfig,
    ) -> Self {
        // Compute goal in robot frame
        let dx = goal_x - x;
        let dy = goal_y - y;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let goal_robot_x = dx * cos_t + dy * sin_t;
        let goal_robot_y = -dx * sin_t + dy * cos_t;

        // Downsample LiDAR to fixed size
        let lidar_ranges = downsample_lidar(lidar_scan, config.lidar_samples, config.max_lidar_range);

        let mut obs = Self {
            pose: [x as f32, y as f32, theta as f32],
            velocity: [linear_vel as f32, angular_vel as f32],
            goal_relative: [goal_robot_x as f32, goal_robot_y as f32],
            lidar_ranges,
        };

        if config.normalize {
            obs.normalize(config);
        }

        obs
    }

    /// Normalize all values to approximately [-1, 1].
    pub fn normalize(&mut self, config: &ObservationConfig) {
        // Pose
        self.pose[0] /= config.max_position;
        self.pose[1] /= config.max_position;
        self.pose[2] /= std::f32::consts::PI;

        // Velocity
        self.velocity[0] /= config.max_linear_vel;
        self.velocity[1] /= config.max_angular_vel;

        // Goal (normalized by position scale)
        self.goal_relative[0] /= config.max_position;
        self.goal_relative[1] /= config.max_position;

        // LiDAR (0 to 1, where 1 = max range)
        for r in &mut self.lidar_ranges {
            *r /= config.max_lidar_range;
        }
    }

    /// Flatten to a single vector for neural network input.
    pub fn to_vec(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(3 + 2 + 2 + self.lidar_ranges.len());
        v.extend_from_slice(&self.pose);
        v.extend_from_slice(&self.velocity);
        v.extend_from_slice(&self.goal_relative);
        v.extend_from_slice(&self.lidar_ranges);
        v
    }

    /// Get the size of the flattened observation.
    pub fn flat_size(&self) -> usize {
        3 + 2 + 2 + self.lidar_ranges.len()
    }

    /// Create from a flat vector (for testing/reconstruction).
    pub fn from_vec(v: &[f32], lidar_samples: usize) -> Self {
        assert!(v.len() >= 7 + lidar_samples);
        Self {
            pose: [v[0], v[1], v[2]],
            velocity: [v[3], v[4]],
            goal_relative: [v[5], v[6]],
            lidar_ranges: v[7..7 + lidar_samples].to_vec(),
        }
    }
}

/// Downsample LiDAR scan to fixed angular bins.
fn downsample_lidar(scan: &LidarScan, num_bins: usize, max_range: f32) -> Vec<f32> {
    use std::f32::consts::PI;

    // Handle zero bins case
    if num_bins == 0 {
        return Vec::new();
    }

    // Initialize bins with max range (no obstacle = far)
    let mut bins = vec![max_range; num_bins];

    for point in &scan.points {
        // Only consider points at reasonable height
        if point.z < 0.1 || point.z > 1.5 {
            continue;
        }

        // Compute horizontal angle
        let angle = point.y.atan2(point.x);

        // Convert to bin index [0, num_bins)
        let normalized_angle = (angle + PI) / (2.0 * PI); // [0, 1]
        let bin_idx = ((normalized_angle * num_bins as f32) as usize) % num_bins;

        // Take minimum range for each bin
        let range = (point.x * point.x + point.y * point.y).sqrt();
        bins[bin_idx] = bins[bin_idx].min(range);
    }

    bins
}

/// Simplified observation without LiDAR (for initial training).
#[derive(Debug, Clone, Default)]
pub struct SimpleObservation {
    /// Robot pose [x, y, theta]
    pub pose: [f32; 3],
    /// Robot velocity [linear, angular]
    pub velocity: [f32; 2],
    /// Goal position relative to robot [distance, bearing]
    pub goal_polar: [f32; 2],
}

impl SimpleObservation {
    /// Create from state without LiDAR.
    pub fn from_state(
        x: f64,
        y: f64,
        theta: f64,
        linear_vel: f64,
        angular_vel: f64,
        goal_x: f64,
        goal_y: f64,
        config: &ObservationConfig,
    ) -> Self {
        // Compute goal in polar coordinates relative to robot
        let dx = goal_x - x;
        let dy = goal_y - y;
        let distance = (dx * dx + dy * dy).sqrt();
        let world_angle = dy.atan2(dx);
        let bearing = world_angle - theta; // Relative bearing

        let mut obs = Self {
            pose: [x as f32, y as f32, theta as f32],
            velocity: [linear_vel as f32, angular_vel as f32],
            goal_polar: [distance as f32, bearing as f32],
        };

        if config.normalize {
            obs.pose[0] /= config.max_position;
            obs.pose[1] /= config.max_position;
            obs.pose[2] /= std::f32::consts::PI;
            obs.velocity[0] /= config.max_linear_vel;
            obs.velocity[1] /= config.max_angular_vel;
            obs.goal_polar[0] /= config.max_position;
            obs.goal_polar[1] /= std::f32::consts::PI;
        }

        obs
    }

    /// Flatten to vector.
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.pose[0],
            self.pose[1],
            self.pose[2],
            self.velocity[0],
            self.velocity[1],
            self.goal_polar[0],
            self.goal_polar[1],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_size() {
        let config = ObservationConfig::default();
        let scan = LidarScan::default();
        let obs = Observation::from_state(
            0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0, &scan, &config,
        );
        assert_eq!(obs.lidar_ranges.len(), config.lidar_samples);
        assert_eq!(obs.flat_size(), 7 + config.lidar_samples);
    }

    #[test]
    fn test_goal_relative() {
        let config = ObservationConfig {
            normalize: false,
            ..Default::default()
        };
        let scan = LidarScan::default();

        // Robot at origin facing +X, goal at (5, 0)
        let obs = Observation::from_state(
            0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0, &scan, &config,
        );
        assert!((obs.goal_relative[0] - 5.0).abs() < 0.01); // Goal is 5m ahead
        assert!(obs.goal_relative[1].abs() < 0.01); // Goal is straight ahead
    }

    #[test]
    fn test_simple_observation() {
        let config = ObservationConfig {
            normalize: false,
            ..Default::default()
        };

        let obs = SimpleObservation::from_state(
            0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 4.0, &config,
        );
        assert!((obs.goal_polar[0] - 5.0).abs() < 0.01); // Distance is 5
    }
}
