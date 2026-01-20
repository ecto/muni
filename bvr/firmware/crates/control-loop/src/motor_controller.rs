//! Motor Controller
//!
//! Handles rate limiting, duty cycle computation, and CAN transmission
//! to VESC motor controllers.

use control::{DiffDriveMixer, RateLimiter};
use types::{Twist, WheelVelocities};

/// Maximum wheel velocity at 5 m/s with 0.08m radius = 62.5 rad/s
const MAX_WHEEL_VEL: f64 = 62.5;

/// Normal mode duty cycle limit (~50% power, ~3 m/s)
const NORMAL_DUTY: f64 = 0.5;

/// Boost mode duty cycle limit (full power)
const BOOST_DUTY: f64 = 0.95;

/// Angular velocity multiplier for skid steering (requires more torque)
const ANGULAR_BOOST: f64 = 2.5;

/// Motor controller output after processing
#[derive(Debug, Clone)]
pub struct MotorOutput {
    /// Wheel duty cycles (-1.0 to 1.0) for [FL, FR, RL, RR]
    pub duties: [f64; 4],
    /// Whether boost mode is active
    pub boost_active: bool,
    /// The rate-limited twist that was applied
    pub applied_twist: Twist,
}

/// Controls motor outputs with rate limiting and mixing
pub struct MotorController {
    mixer: DiffDriveMixer,
    rate_limiter: RateLimiter,
}

impl MotorController {
    /// Create a new motor controller with the given chassis config and limits
    pub fn new(mixer: DiffDriveMixer, rate_limiter: RateLimiter) -> Self {
        Self {
            mixer,
            rate_limiter,
        }
    }

    /// Compute motor outputs from a target twist
    ///
    /// Applies rate limiting for safety, mixes to wheel velocities,
    /// and converts to duty cycles with optional boost.
    pub fn compute(&mut self, target_twist: Twist, boost: bool) -> MotorOutput {
        // Apply rate limiting (acceleration limits only)
        let mut twist = self.rate_limiter.limit(target_twist);

        // Boost angular for skid steering
        twist.angular *= ANGULAR_BOOST;

        // Mix to wheel velocities
        let wheel_vels = self.mixer.mix(twist);

        // Convert to duty cycles
        let max_duty = if boost { BOOST_DUTY } else { NORMAL_DUTY };
        let duties = Self::velocities_to_duties(&wheel_vels, max_duty);

        MotorOutput {
            duties,
            boost_active: boost,
            applied_twist: twist,
        }
    }

    /// Reset the rate limiter (call on e-stop or mode change)
    pub fn reset(&mut self) {
        self.rate_limiter.reset();
    }

    /// Convert wheel velocities (rad/s) to duty cycles (-1.0 to 1.0)
    fn velocities_to_duties(vels: &WheelVelocities, max_duty: f64) -> [f64; 4] {
        [
            (vels.front_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (vels.front_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (vels.rear_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (vels.rear_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
        ]
    }

    /// Check if wheels are turning (for logging)
    pub fn is_turning(duties: &[f64; 4]) -> bool {
        (duties[0] - duties[1]).abs() > 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use control::{ChassisParams, Limits};

    fn test_controller() -> MotorController {
        let chassis = ChassisParams {
            wheelbase: 0.5,
            track_width: 0.4,
            wheel_radius: 0.08,
        };
        let limits = Limits {
            max_linear: 2.0,
            max_angular: 4.0,
            max_accel: 2.0,
            max_decel: 4.0,
            max_wheel_rpm: 1000.0,
        };
        MotorController::new(
            DiffDriveMixer::new(chassis),
            RateLimiter::new(limits),
        )
    }

    #[test]
    fn test_zero_twist() {
        let mut controller = test_controller();
        let output = controller.compute(Twist::default(), false);

        for duty in output.duties {
            assert!(duty.abs() < 0.001);
        }
        assert!(!output.boost_active);
    }

    #[test]
    fn test_forward_motion() {
        let mut controller = test_controller();
        let twist = Twist {
            linear: 1.0,
            angular: 0.0,
            boost: false,
        };
        let output = controller.compute(twist, false);

        // All wheels should have same positive duty
        assert!(output.duties[0] > 0.0);
        assert!((output.duties[0] - output.duties[1]).abs() < 0.001);
        assert!((output.duties[2] - output.duties[3]).abs() < 0.001);
    }

    #[test]
    fn test_boost_mode() {
        let mut controller = test_controller();
        let twist = Twist {
            linear: 2.0,
            angular: 0.0,
            boost: true,
        };

        let normal = controller.compute(twist, false);
        controller.reset();
        let boosted = controller.compute(twist, true);

        // Boosted should have higher duty (up to 0.95 vs 0.5)
        assert!(boosted.duties[0] > normal.duties[0]);
        assert!(boosted.boost_active);
    }

    #[test]
    fn test_turning_detection() {
        // Turning: left != right
        assert!(MotorController::is_turning(&[0.3, 0.5, 0.3, 0.5]));
        // Straight: left == right
        assert!(!MotorController::is_turning(&[0.5, 0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_duty_clamping() {
        let mut controller = test_controller();
        // Very high velocity should be clamped
        let twist = Twist {
            linear: 100.0, // Way over max
            angular: 0.0,
            boost: false,
        };
        let output = controller.compute(twist, false);

        for duty in output.duties {
            assert!(duty.abs() <= NORMAL_DUTY + 0.001);
        }
    }
}
