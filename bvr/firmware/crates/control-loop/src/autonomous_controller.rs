//! Autonomous Controller
//!
//! Handles policy inference, waypoint management, and goal tracking
//! for autonomous navigation mode.

use std::time::{Duration, Instant};
use tracing::{debug, error, warn};
use types::{Pose, Twist};

/// Threshold for considering a waypoint reached (meters)
pub const WAYPOINT_REACHED_THRESHOLD: f64 = 0.5;

/// Warning threshold for slow policy inference
pub const POLICY_WARN_THRESHOLD: Duration = Duration::from_millis(20);

/// Error threshold for critically slow policy inference
pub const POLICY_ERROR_THRESHOLD: Duration = Duration::from_millis(50);

/// Result of autonomous control computation
#[derive(Debug, Clone)]
pub struct AutonomousOutput {
    /// Computed twist command
    pub twist: Twist,
    /// Distance to current goal
    pub distance_to_goal: f64,
    /// Whether within waypoint threshold
    pub waypoint_reached: bool,
    /// Policy inference time
    pub inference_time: Duration,
    /// Whether policy succeeded
    pub policy_ok: bool,
}

impl Default for AutonomousOutput {
    fn default() -> Self {
        Self {
            twist: Twist::default(),
            distance_to_goal: f64::MAX,
            waypoint_reached: false,
            inference_time: Duration::ZERO,
            policy_ok: false,
        }
    }
}

/// Policy observation for inference
#[derive(Debug, Clone)]
pub struct PolicyObservation {
    pub pose_x: f64,
    pub pose_y: f64,
    pub pose_theta: f64,
    pub vel_linear: f64,
    pub vel_angular: f64,
    pub goal_x: f64,
    pub goal_y: f64,
}

/// Trait for policy inference (allows mocking in tests)
pub trait Policy: Send + Sync {
    fn infer(&self, obs: &PolicyObservation) -> Result<PolicyAction, PolicyError>;
}

/// Policy action output (normalized -1 to 1)
#[derive(Debug, Clone)]
pub struct PolicyAction {
    pub linear: f64,
    pub angular: f64,
}

impl PolicyAction {
    /// Convert to twist with velocity limits
    pub fn to_twist(&self, max_linear: f64, max_angular: f64) -> Twist {
        Twist {
            linear: (self.linear * max_linear).clamp(-max_linear, max_linear),
            angular: (self.angular * max_angular).clamp(-max_angular, max_angular),
            boost: false,
        }
    }
}

/// Policy inference error
#[derive(Debug, Clone)]
pub struct PolicyError(pub String);

/// Controls autonomous navigation
pub struct AutonomousController {
    /// Maximum linear velocity (m/s)
    max_linear: f64,
    /// Maximum angular velocity (rad/s)
    max_angular: f64,
    /// Count of consecutive slow policy inferences
    slow_count: u32,
}

impl AutonomousController {
    /// Create a new autonomous controller
    pub fn new(max_linear: f64, max_angular: f64) -> Self {
        Self {
            max_linear,
            max_angular,
            slow_count: 0,
        }
    }

    /// Compute autonomous control output given current state and goal
    ///
    /// Returns the twist command and metadata about the computation.
    pub fn compute<P: Policy>(
        &mut self,
        policy: &P,
        current_pose: Pose,
        current_velocity: Twist,
        goal: [f64; 2],
    ) -> AutonomousOutput {
        let mut output = AutonomousOutput::default();

        // Calculate distance to goal
        let dx = goal[0] - current_pose.x;
        let dy = goal[1] - current_pose.y;
        output.distance_to_goal = (dx * dx + dy * dy).sqrt();
        output.waypoint_reached = output.distance_to_goal < WAYPOINT_REACHED_THRESHOLD;

        // Build observation
        let obs = PolicyObservation {
            pose_x: current_pose.x,
            pose_y: current_pose.y,
            pose_theta: current_pose.theta,
            vel_linear: current_velocity.linear,
            vel_angular: current_velocity.angular,
            goal_x: goal[0],
            goal_y: goal[1],
        };

        // Run policy inference with timing
        let start = Instant::now();
        let result = policy.infer(&obs);
        output.inference_time = start.elapsed();

        // Track slow inferences
        self.track_inference_time(output.inference_time);

        match result {
            Ok(action) => {
                output.twist = action.to_twist(self.max_linear, self.max_angular);
                output.policy_ok = true;

                debug!(
                    linear = output.twist.linear,
                    angular = output.twist.angular,
                    goal_x = goal[0],
                    goal_y = goal[1],
                    distance = output.distance_to_goal,
                    inference_ms = output.inference_time.as_micros() as f64 / 1000.0,
                    "Autonomous policy output"
                );
            }
            Err(e) => {
                warn!(error = ?e.0, "Policy inference failed, stopping");
                output.twist = Twist::default();
                output.policy_ok = false;
            }
        }

        output
    }

    fn track_inference_time(&mut self, duration: Duration) {
        if duration > POLICY_ERROR_THRESHOLD {
            self.slow_count += 1;
            error!(
                duration_ms = duration.as_millis(),
                slow_count = self.slow_count,
                "Policy inference critically slow - may cause control issues"
            );
        } else if duration > POLICY_WARN_THRESHOLD {
            self.slow_count += 1;
            warn!(
                duration_ms = duration.as_millis(),
                slow_count = self.slow_count,
                "Policy inference slow"
            );
        } else {
            self.slow_count = 0;
        }
    }

    /// Get count of consecutive slow inferences
    pub fn slow_inference_count(&self) -> u32 {
        self.slow_count
    }

    /// Reset slow inference counter
    pub fn reset_slow_count(&mut self) {
        self.slow_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPolicy {
        action: PolicyAction,
    }

    impl Policy for MockPolicy {
        fn infer(&self, _obs: &PolicyObservation) -> Result<PolicyAction, PolicyError> {
            Ok(self.action.clone())
        }
    }

    struct FailingPolicy;

    impl Policy for FailingPolicy {
        fn infer(&self, _obs: &PolicyObservation) -> Result<PolicyAction, PolicyError> {
            Err(PolicyError("test error".to_string()))
        }
    }

    #[test]
    fn test_policy_output() {
        let mut controller = AutonomousController::new(2.0, 1.5);

        let policy = MockPolicy {
            action: PolicyAction {
                linear: 0.5,
                angular: 0.3,
            },
        };

        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let goal = [5.0, 5.0];

        let output = controller.compute(&policy, pose, vel, goal);

        // linear: 0.5 * 2.0 = 1.0
        assert!((output.twist.linear - 1.0).abs() < 0.001);
        // angular: 0.3 * 1.5 = 0.45
        assert!((output.twist.angular - 0.45).abs() < 0.001);
        assert!(output.policy_ok);
    }

    #[test]
    fn test_policy_failure() {
        let mut controller = AutonomousController::new(1.0, 1.0);

        let policy = FailingPolicy;
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let goal = [5.0, 5.0];

        let output = controller.compute(&policy, pose, vel, goal);

        // Should return zero twist on failure
        assert!(output.twist.linear.abs() < 0.001);
        assert!(!output.policy_ok);
    }

    #[test]
    fn test_waypoint_reached() {
        let mut controller = AutonomousController::new(1.0, 1.0);

        let policy = MockPolicy {
            action: PolicyAction {
                linear: 0.1,
                angular: 0.0,
            },
        };

        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let goal = [0.1, 0.1]; // Very close goal (~0.14m away)

        let output = controller.compute(&policy, pose, vel, goal);

        // Within 0.5m threshold, so waypoint should be reached
        assert!(output.waypoint_reached);
        assert!(output.distance_to_goal < WAYPOINT_REACHED_THRESHOLD);
    }

    #[test]
    fn test_policy_action_clamping() {
        let action = PolicyAction {
            linear: 2.0,  // Over max
            angular: -2.0, // Over max (negative)
        };

        let twist = action.to_twist(1.0, 0.5);

        assert!((twist.linear - 1.0).abs() < 0.001);
        assert!((twist.angular - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_distance_calculation() {
        let mut controller = AutonomousController::new(1.0, 1.0);

        let policy = MockPolicy {
            action: PolicyAction { linear: 0.0, angular: 0.0 },
        };

        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let goal = [3.0, 4.0]; // 3-4-5 triangle, distance = 5

        let output = controller.compute(&policy, pose, vel, goal);

        assert!((output.distance_to_goal - 5.0).abs() < 0.001);
    }
}
