//! Model Predictive Path Integral (MPPI) controller.
//!
//! Sampling-based trajectory optimizer that proactively steers around obstacles
//! instead of stopping. Generates candidate trajectories, scores them against
//! multiple critics, and returns a soft-max weighted optimal control.
//!
//! # Algorithm
//!
//! 1. Sample N candidate trajectories (Gaussian perturbations of previous best)
//! 2. Forward-simulate each with differential-drive kinematics
//! 3. Score against 5 critics: PathAlign, Obstacle, Goal, PreferForward, AngularVelocity
//! 4. Soft-max weighted average → new control sequence
//! 5. Return first action, warm-start shift for next cycle

use costmap::{costs, Costmap};
use planner::{Path, Waypoint};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use tracing::debug;
use types::{Pose, Twist};

/// MPPI controller configuration.
#[derive(Debug, Clone)]
pub struct MppiConfig {
    /// Number of candidate trajectories to sample.
    pub num_samples: usize,
    /// Prediction horizon steps.
    pub horizon_steps: usize,
    /// Time step per horizon step (seconds).
    pub dt: f64,
    /// Temperature parameter for soft-max weighting (lower = greedier).
    pub temperature: f64,
    /// Standard deviation for linear velocity noise (m/s).
    pub linear_noise_std: f64,
    /// Standard deviation for angular velocity noise (rad/s).
    pub angular_noise_std: f64,
    /// Maximum linear velocity (m/s).
    pub max_linear_velocity: f64,
    /// Maximum angular velocity (rad/s).
    pub max_angular_velocity: f64,
    /// Critic weights.
    pub path_align_weight: f64,
    pub obstacle_weight: f64,
    pub goal_weight: f64,
    pub prefer_forward_weight: f64,
    pub angular_velocity_weight: f64,
}

impl Default for MppiConfig {
    fn default() -> Self {
        Self {
            num_samples: 512,
            horizon_steps: 30,
            dt: 0.1,
            temperature: 0.5,
            linear_noise_std: 0.3,
            angular_noise_std: 0.5,
            max_linear_velocity: 1.0,
            max_angular_velocity: 0.6,
            path_align_weight: 5.0,
            obstacle_weight: 20.0,
            goal_weight: 3.0,
            prefer_forward_weight: 2.0,
            angular_velocity_weight: 1.0,
        }
    }
}

/// Output from MPPI controller.
#[derive(Debug, Clone)]
pub struct MppiOutput {
    /// Optimal twist command for this timestep.
    pub twist: Twist,
    /// Best trajectory cost (lower is better).
    pub best_cost: f64,
    /// Mean trajectory cost across all samples.
    pub mean_cost: f64,
    /// Best-scoring trajectory as (x, y) positions in world frame.
    pub best_trajectory: Vec<[f64; 2]>,
}

/// State at a single point in a simulated trajectory.
#[derive(Debug, Clone, Copy)]
struct TrajState {
    x: f64,
    y: f64,
    theta: f64,
}

/// MPPI trajectory optimizer.
///
/// Pre-allocates all buffers to achieve zero-allocation control computation.
pub struct MppiController {
    config: MppiConfig,
    rng: SmallRng,

    // Pre-allocated buffers
    /// Control sequence: (linear, angular) per horizon step — warm-started.
    control_seq: Vec<[f64; 2]>,
    /// Per-sample noise perturbations: [num_samples][horizon_steps][2]
    noise: Vec<f64>,
    /// Per-sample trajectory states: [num_samples][horizon_steps+1]
    trajectories: Vec<TrajState>,
    /// Per-sample costs.
    costs: Vec<f64>,
    /// Per-sample weights (soft-max).
    weights: Vec<f64>,
}

impl MppiController {
    /// Create a new MPPI controller with pre-allocated buffers.
    pub fn new(config: MppiConfig) -> Self {
        let n = config.num_samples;
        let h = config.horizon_steps;

        let control_seq = vec![[0.0; 2]; h];
        let noise = vec![0.0; n * h * 2];
        let trajectories = vec![TrajState { x: 0.0, y: 0.0, theta: 0.0 }; n * (h + 1)];
        let costs = vec![0.0; n];
        let weights = vec![0.0; n];

        Self {
            config,
            rng: SmallRng::seed_from_u64(42),
            control_seq,
            noise,
            trajectories,
            costs,
            weights,
        }
    }

    /// Reset warm-start (call when switching to a new path).
    pub fn reset(&mut self) {
        for ctrl in &mut self.control_seq {
            *ctrl = [0.0; 2];
        }
    }

    /// Compute optimal control given current state and reference path.
    ///
    /// This is the main entry point, called every control cycle.
    /// Target: < 5ms on Jetson Orin NX.
    pub fn compute(
        &mut self,
        path: &Path,
        pose: &Pose,
        _velocity: &Twist,
        costmap: &Costmap,
    ) -> MppiOutput {
        if path.is_empty() {
            return MppiOutput {
                twist: Twist::default(),
                best_cost: f64::MAX,
                mean_cost: f64::MAX,
                best_trajectory: vec![],
            };
        }

        let n = self.config.num_samples;
        let h = self.config.horizon_steps;

        // Step 1: Generate noise perturbations
        self.generate_noise();

        // Step 2: Forward-simulate all trajectories
        self.simulate_trajectories(pose);

        // Step 3: Score all trajectories
        self.score_trajectories(path, costmap);

        // Step 4: Compute soft-max weights
        self.compute_weights();

        // Step 5: Weighted average of control perturbations → update control sequence
        self.update_controls();

        // Extract first action
        let twist = Twist {
            linear: self.control_seq[0][0].clamp(
                -self.config.max_linear_velocity,
                self.config.max_linear_velocity,
            ),
            angular: self.control_seq[0][1].clamp(
                -self.config.max_angular_velocity,
                self.config.max_angular_velocity,
            ),
            boost: false,
        };

        let (best_idx, best_cost) = self
            .costs
            .iter()
            .enumerate()
            .fold((0, f64::INFINITY), |(bi, bc), (i, &c)| {
                if c < bc { (i, c) } else { (bi, bc) }
            });
        let mean_cost = self.costs.iter().sum::<f64>() / n as f64;

        // Extract best trajectory positions (skip initial state at step 0)
        let best_trajectory: Vec<[f64; 2]> = {
            let base = best_idx * (h + 1);
            (1..=h)
                .map(|t| {
                    let s = &self.trajectories[base + t];
                    [s.x, s.y]
                })
                .collect()
        };

        debug!(
            linear = format!("{:.3}", twist.linear),
            angular = format!("{:.3}", twist.angular),
            best_cost = format!("{:.1}", best_cost),
            mean_cost = format!("{:.1}", mean_cost),
            "MPPI output"
        );

        // Warm-start: shift control sequence left by 1
        for i in 0..h - 1 {
            self.control_seq[i] = self.control_seq[i + 1];
        }
        self.control_seq[h - 1] = [0.0, 0.0];

        MppiOutput {
            twist,
            best_cost,
            mean_cost,
            best_trajectory,
        }
    }

    /// Generate Gaussian noise for all samples and horizon steps.
    fn generate_noise(&mut self) {
        let n = self.config.num_samples;
        let h = self.config.horizon_steps;
        let linear_dist = Normal::new(0.0, self.config.linear_noise_std).unwrap();
        let angular_dist = Normal::new(0.0, self.config.angular_noise_std).unwrap();

        for s in 0..n {
            for t in 0..h {
                let idx = (s * h + t) * 2;
                self.noise[idx] = linear_dist.sample(&mut self.rng);
                self.noise[idx + 1] = angular_dist.sample(&mut self.rng);
            }
        }
    }

    /// Forward-simulate all trajectories using diff-drive kinematics.
    fn simulate_trajectories(&mut self, pose: &Pose) {
        let n = self.config.num_samples;
        let h = self.config.horizon_steps;
        let dt = self.config.dt;

        for s in 0..n {
            // Initialize trajectory at current pose
            let base = s * (h + 1);
            self.trajectories[base] = TrajState {
                x: pose.x,
                y: pose.y,
                theta: pose.theta,
            };

            for t in 0..h {
                let prev = self.trajectories[base + t];

                // Perturbed control
                let noise_idx = (s * h + t) * 2;
                let v = (self.control_seq[t][0] + self.noise[noise_idx])
                    .clamp(-self.config.max_linear_velocity, self.config.max_linear_velocity);
                let w = (self.control_seq[t][1] + self.noise[noise_idx + 1])
                    .clamp(-self.config.max_angular_velocity, self.config.max_angular_velocity);

                // Diff-drive forward kinematics
                let new_theta = prev.theta + w * dt;
                let new_x = prev.x + v * ((prev.theta + new_theta) / 2.0).cos() * dt;
                let new_y = prev.y + v * ((prev.theta + new_theta) / 2.0).sin() * dt;

                self.trajectories[base + t + 1] = TrajState {
                    x: new_x,
                    y: new_y,
                    theta: new_theta,
                };
            }
        }
    }

    /// Score all trajectories using 5 critics.
    fn score_trajectories(&mut self, path: &Path, costmap: &Costmap) {
        let n = self.config.num_samples;
        let h = self.config.horizon_steps;

        let goal = path.goal().unwrap_or(Waypoint::new(0.0, 0.0));

        for s in 0..n {
            let base = s * (h + 1);
            let mut total_cost = 0.0;

            for t in 1..=h {
                let state = self.trajectories[base + t];
                let noise_idx = (s * h + (t - 1)) * 2;
                let v = self.control_seq[t - 1][0] + self.noise[noise_idx];
                let w = self.control_seq[t - 1][1] + self.noise[noise_idx + 1];

                // Critic 1: Path alignment — distance to nearest path point
                let path_dist = self.nearest_path_distance(path, state.x, state.y);
                total_cost += self.config.path_align_weight * path_dist;

                // Critic 2: Obstacle cost — high penalty for costmap obstacles
                let cell_cost = costmap.get_cost(state.x, state.y);
                if cell_cost >= costs::LETHAL {
                    total_cost += self.config.obstacle_weight * 100.0; // Very high penalty
                } else if cell_cost >= costs::INSCRIBED {
                    total_cost += self.config.obstacle_weight * 10.0;
                } else if cell_cost > 0 {
                    total_cost += self.config.obstacle_weight
                        * (cell_cost as f64 / costs::INSCRIBED as f64);
                }

                // Critic 3: Goal distance (only at final step)
                if t == h {
                    let dx = goal.x - state.x;
                    let dy = goal.y - state.y;
                    let goal_dist = (dx * dx + dy * dy).sqrt();
                    total_cost += self.config.goal_weight * goal_dist;
                }

                // Critic 4: Prefer forward motion — penalize backward, reward forward
                total_cost += self.config.prefer_forward_weight * (-v).max(0.0);
                // Small reward for forward motion (negative cost)
                total_cost -= self.config.prefer_forward_weight * 0.1 * v.max(0.0);

                // Critic 5: Angular velocity penalty (smoother trajectories)
                total_cost += self.config.angular_velocity_weight * w.abs();
            }

            self.costs[s] = total_cost;
        }
    }

    /// Find minimum distance from (x, y) to any point on the path.
    fn nearest_path_distance(&self, path: &Path, x: f64, y: f64) -> f64 {
        let mut min_dist = f64::MAX;
        for wp in &path.waypoints {
            let dx = x - wp.x;
            let dy = y - wp.y;
            let dist = (dx * dx + dy * dy).sqrt();
            min_dist = min_dist.min(dist);
        }
        min_dist
    }

    /// Compute soft-max weights from costs.
    fn compute_weights(&mut self) {
        let n = self.config.num_samples;
        let lambda = self.config.temperature;

        // Find minimum cost for numerical stability
        let min_cost = self.costs.iter().cloned().fold(f64::INFINITY, f64::min);

        // Compute unnormalized weights: exp(-(cost - min_cost) / lambda)
        let mut total_weight = 0.0;
        for s in 0..n {
            self.weights[s] = (-(self.costs[s] - min_cost) / lambda).exp();
            total_weight += self.weights[s];
        }

        // Normalize
        if total_weight > 1e-10 {
            for s in 0..n {
                self.weights[s] /= total_weight;
            }
        } else {
            // Uniform fallback
            let uniform = 1.0 / n as f64;
            for s in 0..n {
                self.weights[s] = uniform;
            }
        }
    }

    /// Update control sequence using weighted average of noise perturbations.
    fn update_controls(&mut self) {
        let n = self.config.num_samples;
        let h = self.config.horizon_steps;

        for t in 0..h {
            let mut weighted_v = 0.0;
            let mut weighted_w = 0.0;

            for s in 0..n {
                let noise_idx = (s * h + t) * 2;
                weighted_v += self.weights[s] * self.noise[noise_idx];
                weighted_w += self.weights[s] * self.noise[noise_idx + 1];
            }

            self.control_seq[t][0] += weighted_v;
            self.control_seq[t][1] += weighted_w;

            // Clamp
            self.control_seq[t][0] = self.control_seq[t][0].clamp(
                -self.config.max_linear_velocity,
                self.config.max_linear_velocity,
            );
            self.control_seq[t][1] = self.control_seq[t][1].clamp(
                -self.config.max_angular_velocity,
                self.config.max_angular_velocity,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> MppiConfig {
        // Smaller config for tests
        MppiConfig {
            num_samples: 64,
            horizon_steps: 10,
            ..Default::default()
        }
    }

    fn make_straight_path() -> Path {
        Path {
            waypoints: vec![
                Waypoint::new(0.0, 0.0),
                Waypoint::new(1.0, 0.0),
                Waypoint::new(2.0, 0.0),
                Waypoint::new(3.0, 0.0),
            ],
            length: 3.0,
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = MppiConfig::default();
        assert_eq!(config.num_samples, 512);
        assert_eq!(config.horizon_steps, 30);
        assert!((config.dt - 0.1).abs() < 1e-6);
        assert!((config.obstacle_weight - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_buffer_sizes() {
        let config = MppiConfig::default();
        let controller = MppiController::new(config.clone());

        assert_eq!(controller.control_seq.len(), config.horizon_steps);
        assert_eq!(controller.costs.len(), config.num_samples);
        assert_eq!(controller.weights.len(), config.num_samples);
        assert_eq!(
            controller.noise.len(),
            config.num_samples * config.horizon_steps * 2
        );
        assert_eq!(
            controller.trajectories.len(),
            config.num_samples * (config.horizon_steps + 1)
        );
    }

    #[test]
    fn test_compute_empty_path() {
        let mut controller = MppiController::new(default_config());
        let path = Path::empty();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);

        let output = controller.compute(&path, &pose, &vel, &costmap);
        assert!(output.twist.linear.abs() < 1e-6);
        assert!(output.twist.angular.abs() < 1e-6);
    }

    #[test]
    fn test_compute_straight_path() {
        let config = MppiConfig {
            num_samples: 512,
            horizon_steps: 20,
            prefer_forward_weight: 5.0,
            ..Default::default()
        };
        let mut controller = MppiController::new(config);
        let path = make_straight_path();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);

        // Run iterations to let warm-start converge
        let mut output = controller.compute(&path, &pose, &vel, &costmap);
        for _ in 0..9 {
            output = controller.compute(&path, &pose, &vel, &costmap);
        }

        // After convergence, should produce forward motion for a straight-ahead path
        assert!(output.twist.linear > 0.0, "expected forward velocity, got {}", output.twist.linear);
        assert!(output.best_cost.is_finite());
        assert!(output.mean_cost.is_finite());
    }

    #[test]
    fn test_velocity_limits() {
        let config = MppiConfig {
            max_linear_velocity: 0.5,
            max_angular_velocity: 0.3,
            ..default_config()
        };
        let mut controller = MppiController::new(config.clone());
        let path = make_straight_path();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);

        let output = controller.compute(&path, &pose, &vel, &costmap);

        assert!(output.twist.linear.abs() <= config.max_linear_velocity + 1e-6);
        assert!(output.twist.angular.abs() <= config.max_angular_velocity + 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut controller = MppiController::new(default_config());
        let path = make_straight_path();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);

        // Run one iteration to populate control sequence
        let _ = controller.compute(&path, &pose, &vel, &costmap);

        // Reset should zero the sequence
        controller.reset();
        for ctrl in &controller.control_seq {
            assert!((ctrl[0]).abs() < 1e-10);
            assert!((ctrl[1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_nearest_path_distance() {
        let controller = MppiController::new(default_config());
        let path = make_straight_path();

        // Point on path
        let dist = controller.nearest_path_distance(&path, 1.0, 0.0);
        assert!(dist < 1e-6);

        // Point 1m off path
        let dist = controller.nearest_path_distance(&path, 1.0, 1.0);
        assert!((dist - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_soft_max_weights_sum_to_one() {
        let mut controller = MppiController::new(default_config());
        // Set some varied costs
        for (i, cost) in controller.costs.iter_mut().enumerate() {
            *cost = (i as f64) * 0.5;
        }
        controller.compute_weights();

        let sum: f64 = controller.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_soft_max_prefers_lower_costs() {
        let config = MppiConfig {
            num_samples: 4,
            ..default_config()
        };
        let mut controller = MppiController::new(config);
        controller.costs = vec![1.0, 10.0, 100.0, 1000.0];
        controller.compute_weights();

        // Lower cost should have higher weight
        assert!(controller.weights[0] > controller.weights[1]);
        assert!(controller.weights[1] > controller.weights[2]);
        assert!(controller.weights[2] > controller.weights[3]);
    }

    #[test]
    fn test_warm_start_shift() {
        let config = MppiConfig {
            horizon_steps: 5,
            ..default_config()
        };
        let mut controller = MppiController::new(config);

        // Set control sequence to known values
        for (i, ctrl) in controller.control_seq.iter_mut().enumerate() {
            ctrl[0] = (i + 1) as f64 * 0.1;
            ctrl[1] = 0.0;
        }

        let path = make_straight_path();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let vel = Twist::default();
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);

        // Compute will shift the sequence after returning
        let _ = controller.compute(&path, &pose, &vel, &costmap);

        // Last element should be zero (shifted in)
        let last = controller.control_seq.last().unwrap();
        assert!((last[0]).abs() < 1e-10);
        assert!((last[1]).abs() < 1e-10);
    }
}
