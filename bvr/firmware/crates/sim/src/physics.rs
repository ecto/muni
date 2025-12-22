//! Simple 2D physics simulation.

use crate::world::World;
use nalgebra::Point3;

/// Collision result from physics update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionResult {
    /// No collision occurred
    None,
    /// Collided with an obstacle
    Obstacle,
    /// Went out of bounds
    OutOfBounds,
}

/// 2D physics for the rover.
pub struct Physics {
    /// Position (x, y) in meters
    x: f64,
    y: f64,
    /// Heading in radians
    theta: f64,
    /// Linear velocity (m/s)
    linear_vel: f64,
    /// Angular velocity (rad/s)
    angular_vel: f64,
    /// Chassis parameters
    wheel_radius: f64,
    track_width: f64,
    /// Rover collision radius (meters)
    collision_radius: f64,
    /// Whether collision detection is enabled
    collision_enabled: bool,
    /// Last collision result
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
            wheel_radius: 0.0825, // 165mm diameter
            track_width: 0.55,
            collision_radius: 0.4, // 40cm collision radius
            collision_enabled: true,
            last_collision: CollisionResult::None,
        }
    }

    /// Create with custom collision radius.
    pub fn with_collision_radius(mut self, radius: f64) -> Self {
        self.collision_radius = radius;
        self
    }

    /// Enable or disable collision detection.
    pub fn set_collision_enabled(&mut self, enabled: bool) {
        self.collision_enabled = enabled;
    }

    /// Update physics based on wheel RPMs (without world collision).
    ///
    /// wheel_rpms: [FL, FR, RL, RR] in mechanical RPM
    pub fn update(&mut self, wheel_rpms: [f64; 4], dt: f64) {
        self.update_with_world(wheel_rpms, dt, None);
    }

    /// Update physics based on wheel RPMs with optional world collision.
    ///
    /// wheel_rpms: [FL, FR, RL, RR] in mechanical RPM
    /// Returns the collision result.
    pub fn update_with_world(
        &mut self,
        wheel_rpms: [f64; 4],
        dt: f64,
        world: Option<&World>,
    ) -> CollisionResult {
        // Store previous position for collision rollback
        let prev_x = self.x;
        let prev_y = self.y;

        // Convert RPM to rad/s
        let rpm_to_rads = std::f64::consts::PI / 30.0;

        // Average left and right sides
        let left_vel = ((wheel_rpms[0] + wheel_rpms[2]) / 2.0) * rpm_to_rads * self.wheel_radius;
        let right_vel = ((wheel_rpms[1] + wheel_rpms[3]) / 2.0) * rpm_to_rads * self.wheel_radius;

        // Differential drive kinematics
        self.linear_vel = (left_vel + right_vel) / 2.0;
        self.angular_vel = (right_vel - left_vel) / self.track_width;

        // Update pose
        if self.angular_vel.abs() < 0.001 {
            // Straight line motion
            self.x += self.linear_vel * self.theta.cos() * dt;
            self.y += self.linear_vel * self.theta.sin() * dt;
        } else {
            // Arc motion
            let r = self.linear_vel / self.angular_vel;
            let dtheta = self.angular_vel * dt;
            self.x += r * (self.theta.sin() - (self.theta - dtheta).sin());
            self.y += r * ((self.theta - dtheta).cos() - self.theta.cos());
            self.theta += dtheta;
        }

        // Normalize theta to [-pi, pi]
        while self.theta > std::f64::consts::PI {
            self.theta -= 2.0 * std::f64::consts::PI;
        }
        while self.theta < -std::f64::consts::PI {
            self.theta += 2.0 * std::f64::consts::PI;
        }

        // Check collision if world is provided
        self.last_collision = CollisionResult::None;

        if let Some(world) = world {
            if self.collision_enabled {
                // Check bounds
                if !world.in_bounds(self.x, self.y) {
                    self.x = prev_x;
                    self.y = prev_y;
                    self.linear_vel = 0.0;
                    self.last_collision = CollisionResult::OutOfBounds;
                    return self.last_collision;
                }

                // Check obstacle collision
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

    /// Get current position (x, y, theta).
    pub fn position(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.theta)
    }

    /// Get current velocity (linear, angular).
    pub fn velocity(&self) -> (f64, f64) {
        (self.linear_vel, self.angular_vel)
    }

    /// Get last collision result.
    pub fn last_collision(&self) -> CollisionResult {
        self.last_collision
    }

    /// Get collision radius.
    pub fn collision_radius(&self) -> f64 {
        self.collision_radius
    }

    /// Reset to origin.
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.theta = 0.0;
        self.linear_vel = 0.0;
        self.angular_vel = 0.0;
        self.last_collision = CollisionResult::None;
    }

    /// Set position directly (for spawning/resetting).
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

impl CollisionResult {
    /// Returns true if any collision occurred.
    pub fn is_collision(&self) -> bool {
        !matches!(self, CollisionResult::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_line() {
        let mut physics = Physics::new();

        // 100 RPM on all wheels (about 0.86 m/s)
        let rpms = [100.0, 100.0, 100.0, 100.0];
        physics.update(rpms, 1.0);

        // Should move forward in x
        assert!(physics.x > 0.8);
        assert!(physics.x < 0.9);
        assert!(physics.y.abs() < 0.01);
    }

    #[test]
    fn test_rotate_in_place() {
        let mut physics = Physics::new();

        // Left wheels backward, right forward
        let rpms = [-100.0, 100.0, -100.0, 100.0];
        physics.update(rpms, 0.1);

        // Should rotate without moving
        assert!(physics.x.abs() < 0.01);
        assert!(physics.y.abs() < 0.01);
        assert!(physics.theta.abs() > 0.01);
    }
}



