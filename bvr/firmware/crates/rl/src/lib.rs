//! Reinforcement learning environment for BVR rover simulation.
//!
//! Provides a Gymnasium-style interface for training navigation policies.
//!
//! # Example
//!
//! ```ignore
//! use rl::{BVREnv, EnvConfig};
//!
//! let mut env = BVREnv::new(EnvConfig::default());
//! let obs = env.reset(None);
//!
//! loop {
//!     let action = policy.act(&obs);
//!     let step = env.step(&action);
//!     
//!     if step.terminated || step.truncated {
//!         break;
//!     }
//!     obs = step.observation;
//! }
//! ```

mod env;
mod observation;
mod reward;
mod spaces;

pub use env::{BVREnv, EnvConfig, EpisodeStats};
pub use observation::{Observation, ObservationConfig};
pub use reward::{RewardConfig, RewardComponents};
pub use spaces::{ActionSpace, ObservationSpace};

use types::Twist;

/// Action for the BVR environment (velocity command).
#[derive(Debug, Clone, Copy, Default)]
pub struct Action {
    /// Linear velocity command (-1 to 1, scaled to max_linear_vel)
    pub linear: f32,
    /// Angular velocity command (-1 to 1, scaled to max_angular_vel)
    pub angular: f32,
}

impl Action {
    pub fn new(linear: f32, angular: f32) -> Self {
        Self {
            linear: linear.clamp(-1.0, 1.0),
            angular: angular.clamp(-1.0, 1.0),
        }
    }

    /// Convert to Twist with given velocity limits.
    pub fn to_twist(&self, max_linear: f64, max_angular: f64) -> Twist {
        Twist {
            linear: self.linear as f64 * max_linear,
            angular: self.angular as f64 * max_angular,
            boost: false,
        }
    }

    /// Create from raw array [linear, angular].
    pub fn from_array(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    /// Convert to array.
    pub fn to_array(&self) -> [f32; 2] {
        [self.linear, self.angular]
    }
}

/// Result of taking a step in the environment.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// New observation after the step
    pub observation: Observation,
    /// Reward received
    pub reward: f32,
    /// Whether the episode ended (goal reached, collision, etc.)
    pub terminated: bool,
    /// Whether the episode was cut short (timeout, out of bounds)
    pub truncated: bool,
    /// Additional information
    pub info: StepInfo,
}

/// Additional information from a step.
#[derive(Debug, Clone, Default)]
pub struct StepInfo {
    /// Distance to goal
    pub distance_to_goal: f32,
    /// Whether goal was reached
    pub goal_reached: bool,
    /// Whether collision occurred
    pub collision: bool,
    /// Current simulation time
    pub time: f64,
    /// Reward breakdown
    pub reward_components: RewardComponents,
}

/// Gymnasium-style environment trait.
pub trait Environment {
    type Observation;
    type Action;

    /// Reset the environment and return initial observation.
    fn reset(&mut self, seed: Option<u64>) -> Self::Observation;

    /// Take a step with the given action.
    fn step(&mut self, action: &Self::Action) -> StepResult;

    /// Get the observation space specification.
    fn observation_space(&self) -> ObservationSpace;

    /// Get the action space specification.
    fn action_space(&self) -> ActionSpace;

    /// Render the environment (optional, for debugging).
    fn render(&self) {}

    /// Close the environment and clean up resources.
    fn close(&mut self) {}
}
