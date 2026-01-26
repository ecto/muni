//! Simple 2D physics simulation.
//!
//! Copied from bvr/firmware/crates/sim/src/physics.rs with minimal changes:
//! - Removed `use crate::world::World` (uses local world module)

use crate::world::World;
use nalgebra::Point3;

/// Collision result from physics update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionResult {
    None,
    Obstacle,
    OutOfBounds,
}

/// 2D physics for the rover.
pub struct Physics {
    x: f64,
    y: f64,
    theta: f64,
    linear_vel: f64,
    angular_vel: f64,
    wheel_radius: f64,
    track_width: f64,
    collision_radius: f64,
    collision_enabled: bool,
    last_collision: CollisionResult,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            linear_vel: 0.0,
            angular_vel: 0.0,
            wheel_radius: 0.0825,
            track_width: 0.55,
            collision_radius: 0.4,
            collision_enabled: true,
            last_collision: CollisionResult::None,
        }
    }

    pub fn update_with_world(
        &mut self,
        wheel_rpms: [f64; 4],
        dt: f64,
        world: Option<&World>,
    ) -> CollisionResult {
        let prev_x = self.x;
        let prev_y = self.y;

        let rpm_to_rads = std::f64::consts::PI / 30.0;

        let left_vel = ((wheel_rpms[0] + wheel_rpms[2]) / 2.0) * rpm_to_rads * self.wheel_radius;
        let right_vel = ((wheel_rpms[1] + wheel_rpms[3]) / 2.0) * rpm_to_rads * self.wheel_radius;

        self.linear_vel = (left_vel + right_vel) / 2.0;
        self.angular_vel = (right_vel - left_vel) / self.track_width;

        if self.angular_vel.abs() < 0.001 {
            self.x += self.linear_vel * self.theta.cos() * dt;
            self.y += self.linear_vel * self.theta.sin() * dt;
        } else {
            let r = self.linear_vel / self.angular_vel;
            let dtheta = self.angular_vel * dt;
            self.x += r * (self.theta.sin() - (self.theta - dtheta).sin());
            self.y += r * ((self.theta - dtheta).cos() - self.theta.cos());
            self.theta += dtheta;
        }

        while self.theta > std::f64::consts::PI {
            self.theta -= 2.0 * std::f64::consts::PI;
        }
        while self.theta < -std::f64::consts::PI {
            self.theta += 2.0 * std::f64::consts::PI;
        }

        self.last_collision = CollisionResult::None;

        if let Some(world) = world {
            if self.collision_enabled {
                if !world.in_bounds(self.x, self.y) {
                    self.x = prev_x;
                    self.y = prev_y;
                    self.linear_vel = 0.0;
                    self.last_collision = CollisionResult::OutOfBounds;
                    return self.last_collision;
                }

                let center = Point3::new(self.x as f32, self.y as f32, 0.25);
                if world.circle_collides(center, self.collision_radius as f32) {
                    self.x = prev_x;
                    self.y = prev_y;
                    self.linear_vel = 0.0;
                    self.last_collision = CollisionResult::Obstacle;
                    return self.last_collision;
                }
            }
        }

        self.last_collision
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.theta)
    }

    pub fn velocity(&self) -> (f64, f64) {
        (self.linear_vel, self.angular_vel)
    }

    pub fn set_position(&mut self, x: f64, y: f64, theta: f64) {
        self.x = x;
        self.y = y;
        self.theta = theta;
        self.last_collision = CollisionResult::None;
    }
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}
