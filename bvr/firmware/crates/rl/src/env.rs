//! BVR navigation environment for reinforcement learning.

use crate::observation::{Observation, ObservationConfig};
use crate::reward::{RewardCalculator, RewardConfig};
use crate::spaces::{ActionSpace, ObservationSpace};
use crate::{Action, Environment, StepInfo, StepResult};

use sim::lidar::{LidarConfig, LidarScan, LidarSim};
use sim::physics::Physics;
use sim::world::World;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Configuration for the BVR environment.
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// World size (square room side length in meters)
    pub world_size: f32,
    /// Wall height
    pub wall_height: f32,
    /// Number of random obstacles (0 for empty room)
    pub num_obstacles: usize,
    /// Maximum episode length (steps)
    pub max_steps: usize,
    /// Simulation timestep (seconds)
    pub dt: f64,
    /// Maximum linear velocity (m/s)
    pub max_linear_vel: f64,
    /// Maximum angular velocity (rad/s)
    pub max_angular_vel: f64,
    /// Minimum spawn distance from goal
    pub min_spawn_distance: f32,
    /// Maximum spawn distance from goal
    pub max_spawn_distance: f32,
    /// Fixed goal position (None for random)
    pub fixed_goal: Option<(f64, f64)>,
    /// Fixed spawn position (None for random)
    pub fixed_spawn: Option<(f64, f64, f64)>,
    /// Observation configuration
    pub observation: ObservationConfig,
    /// Reward configuration
    pub reward: RewardConfig,
    /// LiDAR configuration
    pub lidar: LidarConfig,
    /// Whether to use LiDAR observations
    pub use_lidar: bool,
    /// Random seed for reproducibility (None for random)
    pub seed: Option<u64>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            world_size: 20.0,
            wall_height: 2.0,
            num_obstacles: 0,
            max_steps: 500,
            dt: 0.02, // 50Hz control
            max_linear_vel: 1.5,
            max_angular_vel: 2.0,
            min_spawn_distance: 3.0,
            max_spawn_distance: 8.0,
            fixed_goal: None,
            fixed_spawn: None,
            observation: ObservationConfig::default(),
            reward: RewardConfig::default(),
            lidar: LidarConfig::low_res(),
            use_lidar: true,
            seed: None,
        }
    }
}

impl EnvConfig {
    /// Create a simple config for initial training (no obstacles, no LiDAR).
    pub fn simple() -> Self {
        Self {
            num_obstacles: 0,
            use_lidar: false,
            max_steps: 300,
            observation: ObservationConfig {
                lidar_samples: 0,
                ..Default::default()
            },
            reward: RewardConfig::dense(),
            ..Default::default()
        }
    }

    /// Create a config with obstacles for more challenging training.
    pub fn with_obstacles(num_obstacles: usize) -> Self {
        Self {
            num_obstacles,
            use_lidar: true,
            reward: RewardConfig::dense(),
            ..Default::default()
        }
    }
}

/// Statistics for an episode.
#[derive(Debug, Clone, Default)]
pub struct EpisodeStats {
    /// Total reward accumulated
    pub total_reward: f32,
    /// Number of steps taken
    pub steps: usize,
    /// Whether goal was reached
    pub success: bool,
    /// Whether episode ended in collision
    pub collision: bool,
    /// Whether episode timed out
    pub timeout: bool,
    /// Final distance to goal
    pub final_distance: f32,
    /// Average speed during episode
    pub avg_speed: f32,
}

/// BVR navigation environment.
pub struct BVREnv {
    config: EnvConfig,
    world: World,
    physics: Physics,
    lidar: Option<LidarSim>,
    reward_calc: RewardCalculator,
    rng: StdRng,
    
    // Episode state
    goal: (f64, f64),
    steps: usize,
    total_reward: f32,
    total_speed: f32,
    last_scan: LidarScan,
    
    // Spaces
    observation_space: ObservationSpace,
    action_space: ActionSpace,
}

impl BVREnv {
    /// Create a new environment with the given configuration.
    pub fn new(config: EnvConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let world = if config.num_obstacles > 0 {
            World::random_obstacles(
                config.world_size,
                config.wall_height,
                config.num_obstacles,
                rng.clone().r#gen(),
            )
        } else {
            World::empty_room(config.world_size, config.wall_height)
        };

        let lidar = if config.use_lidar {
            Some(LidarSim::new(config.lidar.clone()))
        } else {
            None
        };

        let observation_space = if config.use_lidar {
            ObservationSpace::bvr_default(config.observation.lidar_samples)
        } else {
            ObservationSpace::bvr_default(0)
        };

        Self {
            world,
            physics: Physics::new(),
            lidar,
            reward_calc: RewardCalculator::new(config.reward.clone()),
            rng,
            goal: (0.0, 0.0),
            steps: 0,
            total_reward: 0.0,
            total_speed: 0.0,
            last_scan: LidarScan::default(),
            observation_space,
            action_space: ActionSpace::bvr_default(),
            config,
        }
    }

    /// Get current goal position.
    pub fn goal(&self) -> (f64, f64) {
        self.goal
    }

    /// Get current episode statistics.
    pub fn episode_stats(&self) -> EpisodeStats {
        let (x, y, _) = self.physics.position();
        let distance = ((x - self.goal.0).powi(2) + (y - self.goal.1).powi(2)).sqrt() as f32;
        
        EpisodeStats {
            total_reward: self.total_reward,
            steps: self.steps,
            success: self.reward_calc.is_goal_reached(distance),
            collision: self.physics.last_collision().is_collision(),
            timeout: self.steps >= self.config.max_steps,
            final_distance: distance,
            avg_speed: if self.steps > 0 {
                self.total_speed / self.steps as f32
            } else {
                0.0
            },
        }
    }

    /// Get reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get reference to physics.
    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    /// Get the last LiDAR scan.
    pub fn last_scan(&self) -> &LidarScan {
        &self.last_scan
    }

    /// Generate observation from current state.
    fn get_observation(&mut self) -> Observation {
        let (x, y, theta) = self.physics.position();
        let (linear_vel, angular_vel) = self.physics.velocity();

        // Get LiDAR scan if enabled
        if let Some(ref mut lidar) = self.lidar {
            self.last_scan = lidar.scan(
                &self.world,
                x,
                y,
                theta,
                self.steps as f64 * self.config.dt,
            );
        }

        Observation::from_state(
            x,
            y,
            theta,
            linear_vel,
            angular_vel,
            self.goal.0,
            self.goal.1,
            &self.last_scan,
            &self.config.observation,
        )
    }

    /// Spawn robot at a random valid position.
    fn spawn_random(&mut self) -> (f64, f64, f64) {
        let half = self.config.world_size as f64 / 2.0 - 1.0;
        let margin = 2.0;

        for _ in 0..100 {
            let x = self.rng.r#gen_range(-half + margin..half - margin);
            let y = self.rng.r#gen_range(-half + margin..half - margin);
            let theta = self.rng.r#gen_range(-std::f64::consts::PI..std::f64::consts::PI);

            // Check distance from goal
            let dist = ((x - self.goal.0).powi(2) + (y - self.goal.1).powi(2)).sqrt() as f32;
            if dist < self.config.min_spawn_distance || dist > self.config.max_spawn_distance {
                continue;
            }

            // Check collision
            let center = nalgebra::Point3::new(x as f32, y as f32, 0.25);
            if !self.world.circle_collides(center, self.physics.collision_radius() as f32) {
                return (x, y, theta);
            }
        }

        // Fallback to center
        (0.0, 0.0, 0.0)
    }

    /// Choose a random goal position.
    fn choose_goal(&mut self) -> (f64, f64) {
        if let Some(goal) = self.config.fixed_goal {
            return goal;
        }

        let half = self.config.world_size as f64 / 2.0 - 1.5;
        let margin = 1.5;

        for _ in 0..100 {
            let x = self.rng.r#gen_range(-half + margin..half - margin);
            let y = self.rng.r#gen_range(-half + margin..half - margin);

            // Check collision
            let center = nalgebra::Point3::new(x as f32, y as f32, 0.25);
            if !self.world.circle_collides(center, 0.5) {
                return (x, y);
            }
        }

        (0.0, 0.0)
    }

    /// Regenerate world with new random obstacles.
    pub fn regenerate_world(&mut self, seed: u64) {
        if self.config.num_obstacles > 0 {
            self.world = World::random_obstacles(
                self.config.world_size,
                self.config.wall_height,
                self.config.num_obstacles,
                seed,
            );
        }
    }
}

impl Environment for BVREnv {
    type Observation = Observation;
    type Action = Action;

    fn reset(&mut self, seed: Option<u64>) -> Observation {
        // Reseed RNG if provided
        if let Some(s) = seed {
            self.rng = StdRng::seed_from_u64(s);
            // Optionally regenerate world
            if self.config.num_obstacles > 0 {
                self.regenerate_world(s);
            }
        }

        // Reset episode state
        self.steps = 0;
        self.total_reward = 0.0;
        self.total_speed = 0.0;
        self.reward_calc.reset();
        self.physics.reset();
        self.last_scan = LidarScan::default();

        // Choose goal first
        self.goal = self.choose_goal();

        // Spawn robot
        let (x, y, theta) = if let Some(spawn) = self.config.fixed_spawn {
            spawn
        } else {
            self.spawn_random()
        };
        self.physics.set_position(x, y, theta);

        self.get_observation()
    }

    fn step(&mut self, action: &Action) -> StepResult {
        self.steps += 1;

        // Convert action to twist command
        let twist = action.to_twist(self.config.max_linear_vel, self.config.max_angular_vel);

        // Convert twist to wheel RPMs (simplified: assume direct velocity control)
        let track_width = 0.55;
        let wheel_radius = 0.0825;
        let rpm_per_vel = 30.0 / (std::f64::consts::PI * wheel_radius);

        let left_vel = twist.linear - twist.angular * track_width / 2.0;
        let right_vel = twist.linear + twist.angular * track_width / 2.0;

        let wheel_rpms = [
            left_vel * rpm_per_vel,
            right_vel * rpm_per_vel,
            left_vel * rpm_per_vel,
            right_vel * rpm_per_vel,
        ];

        // Update physics with collision detection
        let collision = self.physics.update_with_world(
            wheel_rpms,
            self.config.dt,
            Some(&self.world),
        );

        // Get new state
        let (x, y, _theta) = self.physics.position();
        let (linear_vel, angular_vel) = self.physics.velocity();

        // Track speed
        self.total_speed += linear_vel.abs() as f32;

        // Compute distance to goal
        let distance = ((x - self.goal.0).powi(2) + (y - self.goal.1).powi(2)).sqrt() as f32;
        let goal_reached = self.reward_calc.is_goal_reached(distance);
        let timeout = self.steps >= self.config.max_steps;

        // Get LiDAR scan
        let scan_ref = if self.lidar.is_some() {
            let (x, y, theta) = self.physics.position();
            if let Some(ref mut lidar) = self.lidar {
                self.last_scan = lidar.scan(
                    &self.world,
                    x,
                    y,
                    theta,
                    self.steps as f64 * self.config.dt,
                );
            }
            Some(&self.last_scan)
        } else {
            None
        };

        // Compute reward
        let reward_components = self.reward_calc.compute(
            goal_reached,
            collision,
            timeout,
            distance,
            linear_vel as f32,
            angular_vel as f32,
            scan_ref,
        );

        self.total_reward += reward_components.total;

        // Determine termination
        let terminated = goal_reached || collision.is_collision();
        let truncated = timeout && !terminated;

        // Build observation
        let observation = self.get_observation();

        StepResult {
            observation,
            reward: reward_components.total,
            terminated,
            truncated,
            info: StepInfo {
                distance_to_goal: distance,
                goal_reached,
                collision: collision.is_collision(),
                time: self.steps as f64 * self.config.dt,
                reward_components,
            },
        }
    }

    fn observation_space(&self) -> ObservationSpace {
        self.observation_space.clone()
    }

    fn action_space(&self) -> ActionSpace {
        self.action_space.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_creation() {
        let env = BVREnv::new(EnvConfig::simple());
        assert_eq!(env.action_space().shape, vec![2]);
    }

    #[test]
    fn test_env_reset() {
        let mut env = BVREnv::new(EnvConfig::simple());
        let obs = env.reset(Some(42));
        assert_eq!(obs.pose.len(), 3);
        assert_eq!(obs.velocity.len(), 2);
    }

    #[test]
    fn test_env_step() {
        let mut env = BVREnv::new(EnvConfig::simple());
        env.reset(Some(42));

        let action = Action::new(0.5, 0.0); // Move forward
        let result = env.step(&action);

        assert!(!result.terminated);
        assert!(!result.truncated);
    }

    #[test]
    fn test_goal_reaching() {
        let mut env = BVREnv::new(EnvConfig {
            fixed_spawn: Some((0.0, 0.0, 0.0)),
            fixed_goal: Some((1.0, 0.0)),
            ..EnvConfig::simple()
        });

        env.reset(None);

        // Move toward goal
        for _ in 0..100 {
            let action = Action::new(1.0, 0.0);
            let result = env.step(&action);
            if result.info.goal_reached {
                assert!(result.terminated);
                assert!(result.reward > 50.0);
                return;
            }
        }

        panic!("Should have reached goal");
    }

    #[test]
    fn test_collision_detection() {
        let mut env = BVREnv::new(EnvConfig {
            world_size: 10.0,
            fixed_spawn: Some((0.0, 0.0, 0.0)),
            fixed_goal: Some((0.0, 5.0)),
            ..EnvConfig::simple()
        });

        env.reset(None);

        // Drive into wall
        for _ in 0..500 {
            let action = Action::new(1.0, 0.0);
            let result = env.step(&action);
            if result.info.collision {
                assert!(result.terminated);
                return;
            }
        }

        // Should have hit wall or boundary
    }
}
