//! Synthetic LiDAR simulation for the Livox Mid-360.
//!
//! Generates point clouds by ray-casting against the simulated world.
//! Copied from bvr/firmware/crates/sim/src/lidar.rs (standalone, no firmware deps).

use crate::world::World;
use nalgebra::{Point3, Vector3};
use rand::Rng;
use std::f32::consts::PI;

/// A single LiDAR point in rover frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct LidarPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
}

/// Configuration for the simulated LiDAR.
#[derive(Debug, Clone)]
pub struct LidarConfig {
    pub horizontal_rays: usize,
    pub vertical_layers: usize,
    pub vertical_min: f32,
    pub vertical_max: f32,
    pub max_range: f32,
    pub min_range: f32,
    pub mount_height: f32,
    pub range_noise: f32,
    pub dropout_rate: f32,
}

impl Default for LidarConfig {
    fn default() -> Self {
        Self {
            horizontal_rays: 360,
            vertical_layers: 8,
            vertical_min: -0.52,
            vertical_max: 0.52,
            max_range: 40.0,
            min_range: 0.1,
            mount_height: 0.4,
            range_noise: 0.02,
            dropout_rate: 0.01,
        }
    }
}

/// Simulated LiDAR sensor.
pub struct LidarSim {
    config: LidarConfig,
    rng: rand::rngs::StdRng,
}

impl LidarSim {
    pub fn new(config: LidarConfig) -> Self {
        use rand::SeedableRng;
        Self {
            config,
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    /// Generate a LiDAR scan from the given pose.
    pub fn scan(
        &mut self,
        world: &World,
        x: f64,
        y: f64,
        theta: f64,
    ) -> Vec<LidarPoint> {
        let mut points = Vec::with_capacity(
            self.config.horizontal_rays * self.config.vertical_layers,
        );

        let origin = Point3::new(x as f32, y as f32, self.config.mount_height);
        let cos_theta = theta.cos() as f32;
        let sin_theta = theta.sin() as f32;

        for h in 0..self.config.horizontal_rays {
            let h_angle = (h as f32 / self.config.horizontal_rays as f32) * 2.0 * PI;

            for v in 0..self.config.vertical_layers {
                let v_frac = if self.config.vertical_layers > 1 {
                    v as f32 / (self.config.vertical_layers - 1) as f32
                } else {
                    0.5
                };
                let v_angle = self.config.vertical_min
                    + v_frac * (self.config.vertical_max - self.config.vertical_min);

                if self.rng.gen::<f32>() < self.config.dropout_rate {
                    continue;
                }

                let cos_v = v_angle.cos();
                let sin_v = v_angle.sin();
                let cos_h = h_angle.cos();
                let sin_h = h_angle.sin();

                let local_dir = Vector3::new(cos_h * cos_v, sin_h * cos_v, sin_v);

                let world_dir = Vector3::new(
                    local_dir.x * cos_theta - local_dir.y * sin_theta,
                    local_dir.x * sin_theta + local_dir.y * cos_theta,
                    local_dir.z,
                );

                if let Some(distance) = world.ray_cast(origin, world_dir, self.config.max_range) {
                    let noisy_dist = distance + self.rng.gen::<f32>() * self.config.range_noise;

                    if noisy_dist >= self.config.min_range && noisy_dist <= self.config.max_range {
                        let point = LidarPoint::new(
                            local_dir.x * noisy_dist,
                            local_dir.y * noisy_dist,
                            local_dir.z * noisy_dist,
                        );

                        let intensity =
                            ((1.0 - noisy_dist / self.config.max_range) * 255.0) as u8;
                        points.push(point.with_intensity(intensity));
                    }
                }
            }
        }

        points
    }
}
