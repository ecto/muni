//! Reward computation for the RL environment.

use sim::lidar::{LidarScan, SafetyStatus, SafetyZone};
use sim::physics::CollisionResult;

/// Configuration for reward computation.
#[derive(Debug, Clone)]
pub struct RewardConfig {
    /// Reward for reaching the goal
    pub goal_reward: f32,
    /// Penalty for collision
    pub collision_penalty: f32,
    /// Penalty for timeout
    pub timeout_penalty: f32,
    /// Reward scale for progress toward goal (per meter)
    pub progress_scale: f32,
    /// Penalty for being too close to obstacles
    pub proximity_penalty_scale: f32,
    /// Penalty for angular velocity (encourages smooth paths)
    pub angular_penalty_scale: f32,
    /// Penalty for reversing
    pub reverse_penalty: f32,
    /// Bonus for smooth control (low jerk)
    pub smoothness_bonus_scale: f32,
    /// Small per-step penalty (encourages efficiency)
    pub step_penalty: f32,
    /// Distance threshold to consider goal reached
    pub goal_threshold: f32,
    /// Safety zone for proximity penalty
    pub safety_zone: SafetyZone,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            goal_reward: 100.0,
            collision_penalty: -50.0,
            timeout_penalty: -10.0,
            progress_scale: 1.0,
            proximity_penalty_scale: -0.5,
            angular_penalty_scale: -0.1,
            reverse_penalty: -0.2,
            smoothness_bonus_scale: 0.05,
            step_penalty: -0.01,
            goal_threshold: 0.5,
            safety_zone: SafetyZone::default(),
        }
    }
}

impl RewardConfig {
    /// Create a sparse reward config (only goal/collision/timeout).
    pub fn sparse() -> Self {
        Self {
            progress_scale: 0.0,
            proximity_penalty_scale: 0.0,
            angular_penalty_scale: 0.0,
            reverse_penalty: 0.0,
            smoothness_bonus_scale: 0.0,
            step_penalty: 0.0,
            ..Default::default()
        }
    }

    /// Create a dense reward config (more shaping).
    pub fn dense() -> Self {
        Self {
            progress_scale: 2.0,
            proximity_penalty_scale: -1.0,
            angular_penalty_scale: -0.2,
            reverse_penalty: -0.5,
            smoothness_bonus_scale: 0.1,
            step_penalty: -0.05,
            ..Default::default()
        }
    }
}

/// Breakdown of reward components (for debugging/analysis).
#[derive(Debug, Clone, Default)]
pub struct RewardComponents {
    /// Reward for reaching goal
    pub goal: f32,
    /// Penalty for collision
    pub collision: f32,
    /// Reward for progress toward goal
    pub progress: f32,
    /// Penalty for proximity to obstacles
    pub proximity: f32,
    /// Penalty for angular velocity
    pub angular: f32,
    /// Penalty for reversing
    pub reverse: f32,
    /// Bonus for smooth control
    pub smoothness: f32,
    /// Per-step penalty
    pub step: f32,
    /// Total reward
    pub total: f32,
}

impl RewardComponents {
    /// Sum all components.
    pub fn compute_total(&mut self) {
        self.total = self.goal
            + self.collision
            + self.progress
            + self.proximity
            + self.angular
            + self.reverse
            + self.smoothness
            + self.step;
    }
}

/// Reward calculator.
pub struct RewardCalculator {
    config: RewardConfig,
    prev_distance: Option<f32>,
    prev_linear_vel: f32,
    prev_angular_vel: f32,
}

impl RewardCalculator {
    pub fn new(config: RewardConfig) -> Self {
        Self {
            config,
            prev_distance: None,
            prev_linear_vel: 0.0,
            prev_angular_vel: 0.0,
        }
    }

    /// Reset the calculator for a new episode.
    pub fn reset(&mut self) {
        self.prev_distance = None;
        self.prev_linear_vel = 0.0;
        self.prev_angular_vel = 0.0;
    }

    /// Compute reward for a step.
    pub fn compute(
        &mut self,
        goal_reached: bool,
        collision: CollisionResult,
        timeout: bool,
        distance_to_goal: f32,
        linear_vel: f32,
        angular_vel: f32,
        lidar_scan: Option<&LidarScan>,
    ) -> RewardComponents {
        let mut components = RewardComponents::default();

        // Terminal rewards
        if goal_reached {
            components.goal = self.config.goal_reward;
        }

        if collision.is_collision() {
            components.collision = self.config.collision_penalty;
        }

        // Progress reward (change in distance to goal)
        if let Some(prev_dist) = self.prev_distance {
            let progress = prev_dist - distance_to_goal; // Positive if getting closer
            components.progress = progress * self.config.progress_scale;
        }
        self.prev_distance = Some(distance_to_goal);

        // Proximity penalty (based on LiDAR)
        if let Some(scan) = lidar_scan {
            let status = self.config.safety_zone.check(scan);
            match status {
                SafetyStatus::Stop => {
                    components.proximity = self.config.proximity_penalty_scale * 2.0;
                }
                SafetyStatus::Slow => {
                    components.proximity = self.config.proximity_penalty_scale;
                }
                SafetyStatus::Clear => {}
            }
        }

        // Angular velocity penalty
        components.angular = angular_vel.abs() * self.config.angular_penalty_scale;

        // Reverse penalty
        if linear_vel < 0.0 {
            components.reverse = self.config.reverse_penalty;
        }

        // Smoothness bonus (low jerk)
        let linear_jerk = (linear_vel - self.prev_linear_vel).abs();
        let angular_jerk = (angular_vel - self.prev_angular_vel).abs();
        let jerk = linear_jerk + angular_jerk;
        if jerk < 0.1 {
            components.smoothness = self.config.smoothness_bonus_scale;
        }
        self.prev_linear_vel = linear_vel;
        self.prev_angular_vel = angular_vel;

        // Step penalty
        components.step = self.config.step_penalty;

        // If timeout, apply penalty
        if timeout && !goal_reached {
            components.step = self.config.timeout_penalty;
        }

        components.compute_total();
        components
    }

    /// Get config reference.
    pub fn config(&self) -> &RewardConfig {
        &self.config
    }

    /// Check if distance is within goal threshold.
    pub fn is_goal_reached(&self, distance: f32) -> bool {
        distance <= self.config.goal_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_reward() {
        let config = RewardConfig::sparse();
        let mut calc = RewardCalculator::new(config);

        let components = calc.compute(
            true,
            CollisionResult::None,
            false,
            0.0,
            0.0,
            0.0,
            None,
        );

        assert!(components.total > 50.0); // Should get goal reward
    }

    #[test]
    fn test_collision_penalty() {
        let config = RewardConfig::sparse();
        let mut calc = RewardCalculator::new(config);

        let components = calc.compute(
            false,
            CollisionResult::Obstacle,
            false,
            5.0,
            0.0,
            0.0,
            None,
        );

        assert!(components.total < -10.0); // Should get collision penalty
    }

    #[test]
    fn test_progress_reward() {
        let config = RewardConfig::default();
        let mut calc = RewardCalculator::new(config);

        // First step: establish baseline
        calc.compute(false, CollisionResult::None, false, 10.0, 1.0, 0.0, None);

        // Second step: got closer
        let components = calc.compute(
            false,
            CollisionResult::None,
            false,
            8.0, // 2m closer
            1.0,
            0.0,
            None,
        );

        assert!(components.progress > 0.0); // Progress reward
    }
}
