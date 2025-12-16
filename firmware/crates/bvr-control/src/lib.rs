//! Motor mixing and velocity control for bvr.

use bvr_types::{Twist, WheelVelocities};
use std::time::{Duration, Instant};

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
    /// - Left wheels: (v - ω * L/2) / r
    /// - Right wheels: (v + ω * L/2) / r
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
}

/// Velocity limits for safety.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_linear: f64,    // m/s
    pub max_angular: f64,   // rad/s
    pub max_accel: f64,     // m/s²
    pub max_wheel_vel: f64, // rad/s
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_linear: 3.0,
            max_angular: 2.0,
            max_accel: 2.0,
            max_wheel_vel: 30.0, // ~300 RPM
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
        twist.linear = twist.linear.clamp(-self.limits.max_linear, self.limits.max_linear);
        twist.angular = twist.angular.clamp(-self.limits.max_angular, self.limits.max_angular);

        // Apply acceleration limit
        let now = Instant::now();
        if let Some(last) = self.last_time {
            let dt = now.duration_since(last).as_secs_f64();
            let max_delta = self.limits.max_accel * dt;

            let delta = twist.linear - self.last_twist.linear;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_drive_forward() {
        let params = ChassisParams::new(0.165, 0.55, 0.55);
        let mixer = DiffDriveMixer::new(params);

        let twist = Twist {
            linear: 1.0,
            angular: 0.0,
        };
        let wheels = mixer.mix(twist);

        // All wheels should spin the same for straight motion
        assert!((wheels.front_left - wheels.front_right).abs() < 0.001);
        assert!((wheels.rear_left - wheels.rear_right).abs() < 0.001);
    }

    #[test]
    fn test_diff_drive_rotate() {
        let params = ChassisParams::new(0.165, 0.55, 0.55);
        let mixer = DiffDriveMixer::new(params);

        let twist = Twist {
            linear: 0.0,
            angular: 1.0,
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
}

