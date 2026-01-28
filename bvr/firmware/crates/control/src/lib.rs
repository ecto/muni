//! Motor mixing and velocity control for bvr.

use std::time::{Duration, Instant};
use types::{Twist, WheelVelocities};

/// Chassis geometry parameters.
#[derive(Debug, Clone)]
pub struct ChassisParams {
    /// Wheel radius in meters
    pub wheel_radius: f64,
    /// Track width (distance between left and right wheels) in meters
    pub track_width: f64,
    /// Wheelbase (distance between front and rear axles) in meters
    pub wheelbase: f64,
}

impl ChassisParams {
    pub fn new(wheel_diameter: f64, track_width: f64, wheelbase: f64) -> Self {
        Self {
            wheel_radius: wheel_diameter / 2.0,
            track_width,
            wheelbase,
        }
    }
}

/// Differential drive mixer for skid-steer configuration.
///
/// Converts body-frame velocity commands (linear, angular) to individual wheel velocities.
pub struct DiffDriveMixer {
    params: ChassisParams,
}

impl DiffDriveMixer {
    pub fn new(params: ChassisParams) -> Self {
        Self { params }
    }

    /// Convert a twist command to wheel velocities (rad/s).
    ///
    /// For a skid-steer robot:
    /// - Left wheels: (v - ω × L/2) / r
    /// - Right wheels: (v + ω × L/2) / r
    ///
    /// Where:
    /// - v = linear velocity (m/s)
    /// - ω = angular velocity (rad/s)
    /// - L = track width (m)
    /// - r = wheel radius (m)
    pub fn mix(&self, twist: Twist) -> WheelVelocities {
        let v = twist.linear;
        let w = twist.angular;
        let half_track = self.params.track_width / 2.0;
        let r = self.params.wheel_radius;

        let left = (v - w * half_track) / r;
        let right = (v + w * half_track) / r;

        WheelVelocities {
            front_left: left,
            front_right: right,
            rear_left: left,
            rear_right: right,
        }
    }

    /// Convert wheel velocities (rad/s) to wheel RPM.
    pub fn to_rpm(&self, velocities: &WheelVelocities) -> [f64; 4] {
        let to_rpm = 60.0 / (2.0 * std::f64::consts::PI);
        [
            velocities.front_left * to_rpm,
            velocities.front_right * to_rpm,
            velocities.rear_left * to_rpm,
            velocities.rear_right * to_rpm,
        ]
    }
}

/// Velocity limits for safety.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_linear: f64,    // m/s
    pub max_angular: f64,   // rad/s
    pub max_accel: f64,     // m/s² (acceleration)
    pub max_decel: f64,     // m/s² (deceleration - can be higher for quicker stops)
    pub max_wheel_rpm: f64, // RPM
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_linear: 5.0,
            max_angular: 2.5,
            max_accel: 50.0, // Effectively unlimited — teleop is direct control
            max_decel: 15.0, // Quick stop for safety when releasing stick
            max_wheel_rpm: 650.0, // ~5.5 m/s with 160mm wheels (headroom)
        }
    }
}

/// Rate limiter for smooth acceleration.
pub struct RateLimiter {
    limits: Limits,
    last_twist: Twist,
    last_time: Option<Instant>,
}

impl RateLimiter {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            last_twist: Twist::default(),
            last_time: None,
        }
    }

    /// Apply rate limiting to a twist command.
    pub fn limit(&mut self, mut twist: Twist) -> Twist {
        // Clamp to absolute limits
        twist.linear = twist
            .linear
            .clamp(-self.limits.max_linear, self.limits.max_linear);
        twist.angular = twist
            .angular
            .clamp(-self.limits.max_angular, self.limits.max_angular);

        // Apply acceleration/deceleration limit
        let now = Instant::now();
        if let Some(last) = self.last_time {
            let dt = now.duration_since(last).as_secs_f64();

            let delta = twist.linear - self.last_twist.linear;

            // Use higher decel limit when slowing down (toward zero or reversing)
            let is_decelerating = delta.signum() != self.last_twist.linear.signum()
                || twist.linear.abs() < self.last_twist.linear.abs();
            let max_rate = if is_decelerating {
                self.limits.max_decel
            } else {
                self.limits.max_accel
            };
            let max_delta = max_rate * dt;

            if delta.abs() > max_delta {
                twist.linear = self.last_twist.linear + delta.signum() * max_delta;
            }
        }

        self.last_twist = twist;
        self.last_time = Some(now);

        twist
    }

    /// Reset the limiter (e.g., after e-stop).
    pub fn reset(&mut self) {
        self.last_twist = Twist::default();
        self.last_time = None;
    }
}

/// Configuration for the teleop collision guard.
#[derive(Debug, Clone)]
pub struct CollisionGuardConfig {
    /// Whether the collision guard is active.
    pub enabled: bool,
    /// How far ahead to project the arc (seconds).
    pub lookahead_time: f64,
    /// Number of sample points along the projected arc.
    pub num_samples: usize,
    /// Distance beyond which no scaling is applied (meters).
    pub full_speed_distance: f64,
    /// Distance below which linear velocity is zeroed (meters).
    pub stop_distance: f64,
    /// Costmap cost that counts as blocked (253 = INSCRIBED).
    pub cost_threshold: u8,
}

impl Default for CollisionGuardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookahead_time: 1.0,
            num_samples: 10,
            full_speed_distance: 1.0,
            stop_distance: 0.15,
            cost_threshold: 253,
        }
    }
}

/// Scales teleop linear velocity based on obstacle proximity along the
/// commanded arc. Angular velocity is never modified — the operator can
/// always steer away.
pub struct CollisionGuard {
    config: CollisionGuardConfig,
}

impl CollisionGuard {
    pub fn new(config: CollisionGuardConfig) -> Self {
        Self { config }
    }

    /// Scale `twist.linear` down when obstacles lie on the projected arc.
    /// `cost_at` returns the costmap cost at a world-frame (x, y).
    pub fn scale(
        &self,
        twist: Twist,
        robot_x: f64,
        robot_y: f64,
        robot_theta: f64,
        cost_at: impl Fn(f64, f64) -> u8,
    ) -> Twist {
        if !self.config.enabled || twist.linear.abs() < 1e-4 {
            return twist;
        }

        let v = twist.linear;
        let w = twist.angular;
        let dt = self.config.lookahead_time / self.config.num_samples as f64;
        let cos_th = robot_theta.cos();
        let sin_th = robot_theta.sin();

        for i in 1..=self.config.num_samples {
            let t = dt * i as f64;

            // Body-frame position along the arc
            let (bx, by) = if w.abs() < 1e-3 {
                // Straight line
                (v * t, 0.0)
            } else {
                // Circular arc: R = v/w
                let r = v / w;
                (r * (w * t).sin(), r * (1.0 - (w * t).cos()))
            };

            // Transform to world frame
            let wx = robot_x + cos_th * bx - sin_th * by;
            let wy = robot_y + sin_th * bx + cos_th * by;

            // Arc distance from robot center to this sample
            let arc_dist = if w.abs() < 1e-3 {
                (v * t).abs()
            } else {
                let r = (v / w).abs();
                r * (w * t).abs()
            };

            if cost_at(wx, wy) >= self.config.cost_threshold {
                // Obstacle hit — compute linear scale
                let range = self.config.full_speed_distance - self.config.stop_distance;
                if range <= 0.0 {
                    return Twist { linear: 0.0, ..twist };
                }
                let scale = ((arc_dist - self.config.stop_distance) / range).clamp(0.0, 1.0);
                return Twist {
                    linear: v * scale,
                    ..twist
                };
            }
        }

        // No obstacle detected along arc — pass through
        twist
    }
}

/// Command watchdog — triggers safe stop if commands stop arriving.
pub struct Watchdog {
    timeout: Duration,
    last_command: Option<Instant>,
}

impl Watchdog {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last_command: None,
        }
    }

    /// Mark that a command was received.
    pub fn feed(&mut self) {
        self.last_command = Some(Instant::now());
    }

    /// Check if the watchdog has timed out.
    pub fn is_timed_out(&self) -> bool {
        match self.last_command {
            Some(t) => t.elapsed() > self.timeout,
            None => true, // Never received a command
        }
    }

    /// Reset the watchdog.
    pub fn reset(&mut self) {
        self.last_command = None;
    }

    /// Get the time elapsed since the last command (for debugging).
    pub fn elapsed_since_last_command(&self) -> Option<Duration> {
        self.last_command.map(|t| t.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_drive_forward() {
        let params = ChassisParams::new(0.160, 0.55, 0.55);
        let mixer = DiffDriveMixer::new(params);

        let twist = Twist {
            linear: 1.0,
            angular: 0.0,
            boost: false,
        };
        let wheels = mixer.mix(twist);

        // All wheels should spin the same for straight motion
        assert!((wheels.front_left - wheels.front_right).abs() < 0.001);
        assert!((wheels.rear_left - wheels.rear_right).abs() < 0.001);
    }

    #[test]
    fn test_diff_drive_rotate() {
        let params = ChassisParams::new(0.160, 0.55, 0.55);
        let mixer = DiffDriveMixer::new(params);

        let twist = Twist {
            linear: 0.0,
            angular: 1.0,
            boost: false,
        };
        let wheels = mixer.mix(twist);

        // Left and right should be opposite for pure rotation
        assert!((wheels.front_left + wheels.front_right).abs() < 0.001);
    }

    #[test]
    fn test_watchdog() {
        let mut wd = Watchdog::new(Duration::from_millis(100));
        assert!(wd.is_timed_out()); // No command yet

        wd.feed();
        assert!(!wd.is_timed_out());

        std::thread::sleep(Duration::from_millis(150));
        assert!(wd.is_timed_out());
    }

    fn guard() -> CollisionGuard {
        CollisionGuard::new(CollisionGuardConfig {
            enabled: true,
            lookahead_time: 1.0,
            num_samples: 10,
            full_speed_distance: 1.0,
            stop_distance: 0.15,
            cost_threshold: 253,
        })
    }

    fn fwd(v: f64) -> Twist {
        Twist { linear: v, angular: 0.0, boost: false }
    }

    #[test]
    fn collision_guard_no_obstacle() {
        let g = guard();
        let t = fwd(1.0);
        let out = g.scale(t, 0.0, 0.0, 0.0, |_, _| 0);
        assert!((out.linear - 1.0).abs() < 1e-6);
    }

    #[test]
    fn collision_guard_obstacle_at_stop_distance() {
        let g = guard();
        let t = fwd(1.0);
        // Everything is blocked
        let out = g.scale(t, 0.0, 0.0, 0.0, |_, _| 253);
        // First sample is at ~0.1m (v*dt = 1.0*0.1), which is < stop_distance (0.15)
        assert!(out.linear.abs() < 1e-6, "expected ~0, got {}", out.linear);
    }

    #[test]
    fn collision_guard_obstacle_at_midpoint() {
        let g = guard();
        let t = fwd(1.0);
        // Block at ~0.5m (sample 5 at t=0.5s, dist=0.5m for v=1.0)
        let out = g.scale(t, 0.0, 0.0, 0.0, |x, _| {
            if x >= 0.45 { 253 } else { 0 }
        });
        // scale = (0.5 - 0.15) / (1.0 - 0.15) ≈ 0.41
        assert!(out.linear > 0.3 && out.linear < 0.55,
            "expected ~0.41, got {}", out.linear);
    }

    #[test]
    fn collision_guard_near_zero_linear() {
        let g = guard();
        let t = Twist { linear: 0.00005, angular: 1.0, boost: false };
        let out = g.scale(t, 0.0, 0.0, 0.0, |_, _| 253);
        // Near-zero linear is passed through (below 1e-4 threshold)
        assert!((out.linear - 0.00005).abs() < 1e-8);
    }

    #[test]
    fn collision_guard_reverse() {
        let g = guard();
        let t = fwd(-1.0);
        // Block behind at x <= -0.45
        let out = g.scale(t, 0.0, 0.0, 0.0, |x, _| {
            if x <= -0.45 { 253 } else { 0 }
        });
        // Should scale reverse velocity (negative)
        assert!(out.linear > -1.0 && out.linear < 0.0,
            "expected scaled reverse, got {}", out.linear);
    }

    #[test]
    fn collision_guard_disabled() {
        let mut cfg = CollisionGuardConfig::default();
        cfg.enabled = false;
        let g = CollisionGuard::new(cfg);
        let t = fwd(1.0);
        let out = g.scale(t, 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.linear - 1.0).abs() < 1e-6);
    }

    #[test]
    fn collision_guard_preserves_angular() {
        let g = guard();
        let t = Twist { linear: 1.0, angular: 0.5, boost: false };
        let out = g.scale(t, 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.angular - 0.5).abs() < 1e-6, "angular must not be modified");
    }
}

