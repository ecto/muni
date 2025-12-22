//! Synthetic LiDAR simulation for the Livox Mid-360.
//!
//! Generates point clouds by ray-casting against the simulated world.
//! Mimics the Mid-360's scan pattern: 360° horizontal, ~59° vertical FOV.

use crate::world::World;
use nalgebra::{Point3, Vector3};
use rand::Rng;
use std::f32::consts::PI;

/// A single LiDAR point.
#[derive(Debug, Clone, Copy, Default)]
pub struct LidarPoint {
    /// X coordinate in rover frame (forward)
    pub x: f32,
    /// Y coordinate in rover frame (left)
    pub y: f32,
    /// Z coordinate in rover frame (up)
    pub z: f32,
    /// Intensity (0-255)
    pub intensity: u8,
}

impl LidarPoint {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            intensity: 255,
        }
    }

    pub fn with_intensity(mut self, intensity: u8) -> Self {
        self.intensity = intensity;
        self
    }

    /// Distance from origin.
    pub fn range(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Convert to nalgebra Point3.
    pub fn to_point3(&self) -> Point3<f32> {
        Point3::new(self.x, self.y, self.z)
    }
}

/// A complete LiDAR scan (one rotation).
#[derive(Debug, Clone, Default)]
pub struct LidarScan {
    /// Points in rover frame
    pub points: Vec<LidarPoint>,
    /// Timestamp (simulation time in seconds)
    pub timestamp: f64,
}

impl LidarScan {
    /// Get points within a distance range.
    pub fn filter_range(&self, min: f32, max: f32) -> Vec<&LidarPoint> {
        self.points
            .iter()
            .filter(|p| {
                let r = p.range();
                r >= min && r <= max
            })
            .collect()
    }

    /// Get the minimum distance to any point.
    pub fn min_range(&self) -> Option<f32> {
        self.points.iter().map(|p| p.range()).reduce(f32::min)
    }

    /// Get points in a horizontal sector (for safety zones).
    pub fn sector(&self, angle_min: f32, angle_max: f32) -> Vec<&LidarPoint> {
        self.points
            .iter()
            .filter(|p| {
                let angle = p.y.atan2(p.x);
                angle >= angle_min && angle <= angle_max
            })
            .collect()
    }

    /// Downsample to fixed number of points (for RL observation).
    pub fn downsample(&self, target_count: usize) -> Vec<LidarPoint> {
        if self.points.len() <= target_count {
            return self.points.clone();
        }

        let step = self.points.len() / target_count;
        self.points
            .iter()
            .step_by(step)
            .take(target_count)
            .copied()
            .collect()
    }

    /// Convert to flat array for RL observation: [x0, y0, z0, x1, y1, z1, ...]
    pub fn to_flat_array(&self) -> Vec<f32> {
        self.points
            .iter()
            .flat_map(|p| [p.x, p.y, p.z])
            .collect()
    }

    /// Convert to range-only array (simpler observation).
    pub fn to_range_array(&self) -> Vec<f32> {
        self.points.iter().map(|p| p.range()).collect()
    }
}

/// Configuration for the simulated LiDAR.
#[derive(Debug, Clone)]
pub struct LidarConfig {
    /// Number of horizontal rays per scan
    pub horizontal_rays: usize,
    /// Number of vertical layers
    pub vertical_layers: usize,
    /// Minimum vertical angle (radians, negative = down)
    pub vertical_min: f32,
    /// Maximum vertical angle (radians, positive = up)
    pub vertical_max: f32,
    /// Maximum detection range (meters)
    pub max_range: f32,
    /// Minimum detection range (meters)
    pub min_range: f32,
    /// LiDAR mount height above ground (meters)
    pub mount_height: f32,
    /// Range noise standard deviation (meters)
    pub range_noise: f32,
    /// Dropout probability (0.0 to 1.0)
    pub dropout_rate: f32,
}

impl Default for LidarConfig {
    fn default() -> Self {
        // Approximates Livox Mid-360 characteristics
        Self {
            horizontal_rays: 360,    // 1 degree resolution
            vertical_layers: 8,      // Simplified from real non-repetitive pattern
            vertical_min: -0.52,     // -30 degrees
            vertical_max: 0.52,      // +30 degrees (59° total FOV)
            max_range: 40.0,         // 40m range
            min_range: 0.1,          // 10cm minimum
            mount_height: 0.4,       // Mounted 40cm above ground
            range_noise: 0.02,       // 2cm noise std dev
            dropout_rate: 0.01,      // 1% dropout
        }
    }
}

impl LidarConfig {
    /// Create a lower-resolution config for faster simulation.
    pub fn low_res() -> Self {
        Self {
            horizontal_rays: 90,
            vertical_layers: 4,
            ..Default::default()
        }
    }

    /// Create a higher-resolution config for detailed simulation.
    pub fn high_res() -> Self {
        Self {
            horizontal_rays: 720,
            vertical_layers: 16,
            ..Default::default()
        }
    }
}

/// Simulated LiDAR sensor.
pub struct LidarSim {
    config: LidarConfig,
    /// Random number generator for noise
    rng: rand::rngs::ThreadRng,
}

impl LidarSim {
    pub fn new(config: LidarConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a LiDAR scan from the given pose.
    ///
    /// # Arguments
    /// * `world` - The simulated world to ray-cast against
    /// * `x` - Rover X position in world frame
    /// * `y` - Rover Y position in world frame  
    /// * `theta` - Rover heading (radians)
    /// * `time` - Simulation timestamp
    pub fn scan(
        &mut self,
        world: &World,
        x: f64,
        y: f64,
        theta: f64,
        time: f64,
    ) -> LidarScan {
        let mut points = Vec::with_capacity(
            self.config.horizontal_rays * self.config.vertical_layers,
        );

        let origin = Point3::new(
            x as f32,
            y as f32,
            self.config.mount_height,
        );

        let cos_theta = theta.cos() as f32;
        let sin_theta = theta.sin() as f32;

        for h in 0..self.config.horizontal_rays {
            // Horizontal angle (0 = forward, positive = left/CCW)
            let h_angle = (h as f32 / self.config.horizontal_rays as f32) * 2.0 * PI;

            for v in 0..self.config.vertical_layers {
                // Vertical angle
                let v_frac = if self.config.vertical_layers > 1 {
                    v as f32 / (self.config.vertical_layers - 1) as f32
                } else {
                    0.5
                };
                let v_angle = self.config.vertical_min
                    + v_frac * (self.config.vertical_max - self.config.vertical_min);

                // Random dropout
                if self.rng.r#gen::<f32>() < self.config.dropout_rate {
                    continue;
                }

                // Direction in rover frame
                let cos_v = v_angle.cos();
                let sin_v = v_angle.sin();
                let cos_h = h_angle.cos();
                let sin_h = h_angle.sin();

                // Local direction (rover frame)
                let local_dir = Vector3::new(cos_h * cos_v, sin_h * cos_v, sin_v);

                // Rotate to world frame
                let world_dir = Vector3::new(
                    local_dir.x * cos_theta - local_dir.y * sin_theta,
                    local_dir.x * sin_theta + local_dir.y * cos_theta,
                    local_dir.z,
                );

                // Cast ray
                if let Some(distance) = world.ray_cast(origin, world_dir, self.config.max_range) {
                    // Add noise
                    let noisy_dist = distance + self.rng.r#gen::<f32>() * self.config.range_noise;

                    if noisy_dist >= self.config.min_range && noisy_dist <= self.config.max_range {
                        // Hit point in rover frame (not world frame)
                        let point = LidarPoint::new(
                            local_dir.x * noisy_dist,
                            local_dir.y * noisy_dist,
                            local_dir.z * noisy_dist,
                        );

                        // Intensity based on distance (closer = brighter)
                        let intensity = ((1.0 - noisy_dist / self.config.max_range) * 255.0) as u8;
                        points.push(point.with_intensity(intensity));
                    }
                }
            }
        }

        LidarScan {
            points,
            timestamp: time,
        }
    }

    /// Get config reference.
    pub fn config(&self) -> &LidarConfig {
        &self.config
    }
}

/// Safety zone checker using LiDAR data.
#[derive(Debug, Clone)]
pub struct SafetyZone {
    /// Inner radius for immediate stop (meters)
    pub stop_radius: f32,
    /// Outer radius for slowdown (meters)
    pub slow_radius: f32,
    /// Minimum height to consider (ignore ground, meters)
    pub height_min: f32,
    /// Maximum height to consider (ignore overhead, meters)
    pub height_max: f32,
}

impl Default for SafetyZone {
    fn default() -> Self {
        Self {
            stop_radius: 0.8,
            slow_radius: 2.0,
            height_min: 0.1,
            height_max: 1.5,
        }
    }
}

/// Result of safety zone check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyStatus {
    /// No obstacles in safety zone
    Clear,
    /// Obstacle in slow-down zone
    Slow,
    /// Obstacle in stop zone
    Stop,
}

impl SafetyZone {
    /// Check a LiDAR scan against the safety zones.
    pub fn check(&self, scan: &LidarScan) -> SafetyStatus {
        let mut status = SafetyStatus::Clear;

        for point in &scan.points {
            // Filter by height
            if point.z < self.height_min || point.z > self.height_max {
                continue;
            }

            // Horizontal distance only
            let horiz_dist = (point.x * point.x + point.y * point.y).sqrt();

            if horiz_dist < self.stop_radius {
                return SafetyStatus::Stop;
            }

            if horiz_dist < self.slow_radius {
                status = SafetyStatus::Slow;
            }
        }

        status
    }

    /// Get the minimum distance to any point in the height band.
    pub fn min_distance(&self, scan: &LidarScan) -> Option<f32> {
        scan.points
            .iter()
            .filter(|p| p.z >= self.height_min && p.z <= self.height_max)
            .map(|p| (p.x * p.x + p.y * p.y).sqrt())
            .reduce(f32::min)
    }

    /// Get minimum distance in front sector (for forward safety).
    pub fn min_forward_distance(&self, scan: &LidarScan, half_angle: f32) -> Option<f32> {
        scan.points
            .iter()
            .filter(|p| {
                p.z >= self.height_min
                    && p.z <= self.height_max
                    && p.x > 0.0  // Forward only
                    && p.y.atan2(p.x).abs() < half_angle
            })
            .map(|p| (p.x * p.x + p.y * p.y).sqrt())
            .reduce(f32::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lidar_scan_empty_world() {
        let world = World::new();
        let mut lidar = LidarSim::new(LidarConfig::low_res());

        let scan = lidar.scan(&world, 0.0, 0.0, 0.0, 0.0);

        // Should have some ground hits
        assert!(!scan.points.is_empty());
    }

    #[test]
    fn test_lidar_scan_with_obstacles() {
        let world = World::empty_room(10.0, 2.0);
        let mut lidar = LidarSim::new(LidarConfig::low_res());

        let scan = lidar.scan(&world, 0.0, 0.0, 0.0, 0.0);

        // Should have wall and ground hits
        assert!(scan.points.len() > 100);

        // Check we can find walls at reasonable distance
        let min_horiz = scan
            .points
            .iter()
            .filter(|p| p.z > 0.3)
            .map(|p| (p.x * p.x + p.y * p.y).sqrt())
            .reduce(f32::min);

        assert!(min_horiz.is_some());
        assert!(min_horiz.unwrap() < 6.0); // Walls are at ~5m
    }

    #[test]
    fn test_safety_zone() {
        let zone = SafetyZone::default();

        // Create a scan with a close point
        let scan = LidarScan {
            points: vec![
                LidarPoint::new(0.5, 0.0, 0.5), // Close, in height band
            ],
            timestamp: 0.0,
        };

        assert_eq!(zone.check(&scan), SafetyStatus::Stop);

        // Point in slow zone
        let scan2 = LidarScan {
            points: vec![LidarPoint::new(1.5, 0.0, 0.5)],
            timestamp: 0.0,
        };

        assert_eq!(zone.check(&scan2), SafetyStatus::Slow);

        // Point outside zones
        let scan3 = LidarScan {
            points: vec![LidarPoint::new(3.0, 0.0, 0.5)],
            timestamp: 0.0,
        };

        assert_eq!(zone.check(&scan3), SafetyStatus::Clear);
    }

    #[test]
    fn test_downsample() {
        let scan = LidarScan {
            points: (0..1000)
                .map(|i| LidarPoint::new(i as f32 * 0.01, 0.0, 0.0))
                .collect(),
            timestamp: 0.0,
        };

        let downsampled = scan.downsample(100);
        assert_eq!(downsampled.len(), 100);
    }
}
