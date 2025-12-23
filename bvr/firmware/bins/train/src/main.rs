//! RL training for BVR navigation policies.
//!
//! This binary provides tools for training navigation policies using
//! the simulated BVR environment.

use anyhow::Result;
use clap::{Parser, Subcommand};
use policy::{PolicyBuilder, PolicyMetrics};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rl::{Action, BVREnv, EnvConfig, Environment, EpisodeStats};
use std::time::Instant;
use tracing::info;

#[derive(Parser)]
#[command(name = "train")]
#[command(about = "Train and evaluate BVR navigation policies")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run random rollouts to test the environment
    Random {
        /// Number of episodes to run
        #[arg(short, long, default_value = "100")]
        episodes: usize,

        /// Random seed
        #[arg(short, long)]
        seed: Option<u64>,

        /// Use obstacles in the environment
        #[arg(long)]
        obstacles: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run a simple heuristic policy (always face goal)
    Heuristic {
        /// Number of episodes to run
        #[arg(short, long, default_value = "100")]
        episodes: usize,

        /// Random seed
        #[arg(short, long)]
        seed: Option<u64>,

        /// Use obstacles
        #[arg(long)]
        obstacles: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Benchmark environment performance
    Bench {
        /// Number of steps to run
        #[arg(short, long, default_value = "100000")]
        steps: usize,

        /// Use LiDAR
        #[arg(long)]
        lidar: bool,
    },

    /// Train using REINFORCE (policy gradient)
    Train {
        /// Number of training iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Episodes per iteration
        #[arg(long, default_value = "10")]
        episodes_per_iter: usize,

        /// Learning rate
        #[arg(long, default_value = "0.01")]
        lr: f32,

        /// Discount factor
        #[arg(long, default_value = "0.99")]
        gamma: f32,

        /// Random seed
        #[arg(short, long)]
        seed: Option<u64>,

        /// Path to save policy (directory or full path)
        #[arg(short, long)]
        output: Option<String>,

        /// Policy name (used in versioned output)
        #[arg(long, default_value = "nav")]
        name: String,

        /// Policy version (semver format)
        #[arg(long, default_value = "0.1.0")]
        version: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("train=info".parse()?)
                .add_directive("rl=warn".parse()?)
                .add_directive("sim=warn".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Random {
            episodes,
            seed,
            obstacles,
            verbose,
        } => run_random(episodes, seed, obstacles, verbose),

        Commands::Heuristic {
            episodes,
            seed,
            obstacles,
            verbose,
        } => run_heuristic(episodes, seed, obstacles, verbose),

        Commands::Bench { steps, lidar } => run_benchmark(steps, lidar),

        Commands::Train {
            iterations,
            episodes_per_iter,
            lr,
            gamma,
            seed,
            output,
            name,
            version,
        } => run_training(iterations, episodes_per_iter, lr, gamma, seed, output, name, version),
    }
}

/// Run episodes with random actions.
fn run_random(episodes: usize, seed: Option<u64>, obstacles: bool, verbose: bool) -> Result<()> {
    info!("Running {} random episodes", episodes);

    let config = if obstacles {
        EnvConfig::with_obstacles(5)
    } else {
        EnvConfig::simple()
    };

    let mut env = BVREnv::new(config);
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let mut stats = AggregateStats::new();

    for ep in 0..episodes {
        let _obs = env.reset(Some(rng.r#gen()));
        let mut episode_reward = 0.0;
        let mut steps = 0;

        loop {
            let action = Action::new(
                rng.r#gen_range(-1.0..1.0),
                rng.r#gen_range(-1.0..1.0),
            );

            let result = env.step(&action);
            episode_reward += result.reward;
            steps += 1;

            if result.terminated || result.truncated {
                let ep_stats = env.episode_stats();
                stats.add(&ep_stats);

                if verbose {
                    info!(
                        "Episode {}: reward={:.2}, steps={}, success={}, collision={}",
                        ep, episode_reward, steps, ep_stats.success, ep_stats.collision
                    );
                }
                break;
            }
        }
    }

    stats.print_summary("Random Policy");
    Ok(())
}

/// Run episodes with a simple heuristic policy.
fn run_heuristic(episodes: usize, seed: Option<u64>, obstacles: bool, verbose: bool) -> Result<()> {
    info!("Running {} heuristic episodes", episodes);

    let config = if obstacles {
        EnvConfig::with_obstacles(5)
    } else {
        EnvConfig::simple()
    };

    let mut env = BVREnv::new(config);
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let mut stats = AggregateStats::new();

    for ep in 0..episodes {
        let _obs = env.reset(Some(rng.r#gen()));
        let mut episode_reward = 0.0;
        let mut steps = 0;

        loop {
            // Simple heuristic: turn toward goal, then drive forward
            let (x, y, theta) = env.physics().position();
            let (gx, gy) = env.goal();

            let dx = gx - x;
            let dy = gy - y;
            let goal_angle = dy.atan2(dx);
            let angle_error = normalize_angle(goal_angle - theta);

            // P-controller for steering
            let angular = (angle_error * 2.0).clamp(-1.0, 1.0) as f32;

            // Slow down when turning, speed up when aligned
            let linear = (1.0 - angle_error.abs() / std::f64::consts::PI).max(0.2) as f32;

            let action = Action::new(linear, angular);
            let result = env.step(&action);
            episode_reward += result.reward;
            steps += 1;

            if result.terminated || result.truncated {
                let ep_stats = env.episode_stats();
                stats.add(&ep_stats);

                if verbose {
                    info!(
                        "Episode {}: reward={:.2}, steps={}, success={}, collision={}",
                        ep, episode_reward, steps, ep_stats.success, ep_stats.collision
                    );
                }
                break;
            }
        }
    }

    stats.print_summary("Heuristic Policy");
    Ok(())
}

/// Benchmark environment performance.
fn run_benchmark(steps: usize, use_lidar: bool) -> Result<()> {
    info!("Benchmarking {} steps (lidar={})", steps, use_lidar);

    let config = EnvConfig {
        use_lidar,
        num_obstacles: if use_lidar { 5 } else { 0 },
        ..EnvConfig::simple()
    };

    let mut env = BVREnv::new(config);
    let mut rng = StdRng::seed_from_u64(42);

    let start = Instant::now();
    let mut total_steps = 0;

    env.reset(Some(42));

    while total_steps < steps {
        let action = Action::new(rng.r#gen_range(-1.0..1.0), rng.r#gen_range(-1.0..1.0));

        let result = env.step(&action);
        total_steps += 1;

        if result.terminated || result.truncated {
            env.reset(Some(rng.r#gen()));
        }
    }

    let elapsed = start.elapsed();
    let steps_per_sec = total_steps as f64 / elapsed.as_secs_f64();

    info!(
        "Completed {} steps in {:.2}s ({:.0} steps/sec)",
        total_steps,
        elapsed.as_secs_f64(),
        steps_per_sec
    );

    // At 50Hz control, how many real-time environments can we simulate?
    let realtime_ratio = steps_per_sec / 50.0;
    info!(
        "Real-time ratio: {:.1}x (can simulate {:.0} envs in parallel)",
        realtime_ratio, realtime_ratio
    );

    Ok(())
}

/// Train using REINFORCE (simple policy gradient).
fn run_training(
    iterations: usize,
    episodes_per_iter: usize,
    lr: f32,
    gamma: f32,
    seed: Option<u64>,
    output: Option<String>,
    policy_name: String,
    policy_version: String,
) -> Result<()> {
    info!(
        "Training for {} iterations ({} episodes/iter)",
        iterations, episodes_per_iter
    );

    // Simple linear policy: action = W * obs + b
    // Observation size: 7 (pose + vel + goal) for simple env
    // Action size: 2 (linear, angular)
    let obs_size = 7;
    let act_size = 2;

    let mut policy = LinearPolicy::new(obs_size, act_size, seed.unwrap_or(42));
    let config = EnvConfig::simple();
    let mut env = BVREnv::new(config);

    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let mut best_reward = f32::NEG_INFINITY;
    let mut best_success_rate = 0.0f32;

    for iter in 0..iterations {
        let mut all_rewards = Vec::new();
        let mut all_log_probs = Vec::new();
        let mut iter_returns = Vec::new();

        // Collect episodes
        for _ in 0..episodes_per_iter {
            let obs = env.reset(Some(rng.r#gen()));
            let mut episode_rewards = Vec::new();
            let mut episode_log_probs = Vec::new();
            let mut obs = obs;

            loop {
                // Get action from policy
                let obs_vec: Vec<f32> = vec![
                    obs.pose[0],
                    obs.pose[1],
                    obs.pose[2],
                    obs.velocity[0],
                    obs.velocity[1],
                    obs.goal_relative[0],
                    obs.goal_relative[1],
                ];

                let (action, log_prob) = policy.sample(&obs_vec, &mut rng);
                let result = env.step(&action);

                episode_rewards.push(result.reward);
                episode_log_probs.push(log_prob);

                if result.terminated || result.truncated {
                    break;
                }

                obs = result.observation;
            }

            // Compute returns (discounted cumulative reward)
            let returns = compute_returns(&episode_rewards, gamma);
            let episode_return = returns.first().copied().unwrap_or(0.0);
            iter_returns.push(episode_return);

            all_rewards.extend(returns);
            all_log_probs.extend(episode_log_probs);
        }

        // Compute baseline (mean return)
        let baseline: f32 = all_rewards.iter().sum::<f32>() / all_rewards.len() as f32;

        // Note: This is a simplified version. Full REINFORCE would need
        // the observation at each step to compute gradients properly.
        // For now, we just adjust based on returns.

        // Update policy using returns - baseline as advantage
        // This is a very simplified update; real implementation would
        // compute gradients through the policy network

        let mean_return: f32 = iter_returns.iter().sum::<f32>() / iter_returns.len() as f32;
        let success_rate = iter_returns
            .iter()
            .filter(|&&r| r > 50.0) // Rough success threshold
            .count() as f32
            / iter_returns.len() as f32;

        if mean_return > best_reward {
            best_reward = mean_return;
            best_success_rate = success_rate;

            if let Some(ref path) = output {
                policy.save_versioned(
                    path,
                    &policy_name,
                    &policy_version,
                    iterations,
                    iter * episodes_per_iter,
                    mean_return,
                    success_rate,
                )?;
                info!("Saved best policy (reward={:.2})", mean_return);
            }
        }

        if iter % 10 == 0 || iter == iterations - 1 {
            info!(
                "Iter {}: mean_return={:.2}, best={:.2}, success_rate={:.1}%",
                iter,
                mean_return,
                best_reward,
                success_rate * 100.0
            );
        }

        // Simple parameter perturbation for learning
        // (Real implementation would use backprop through policy)
        policy.perturb(lr, mean_return - baseline, &mut rng);
    }

    info!("Training complete. Best reward: {:.2}", best_reward);

    if let Some(path) = output {
        policy.save_versioned(
            &path,
            &policy_name,
            &policy_version,
            iterations,
            iterations * episodes_per_iter,
            best_reward,
            best_success_rate,
        )?;
        info!("Saved final policy to {}", path);
    }

    Ok(())
}

/// Compute discounted returns.
fn compute_returns(rewards: &[f32], gamma: f32) -> Vec<f32> {
    let mut returns = vec![0.0; rewards.len()];
    let mut running_return = 0.0;

    for i in (0..rewards.len()).rev() {
        running_return = rewards[i] + gamma * running_return;
        returns[i] = running_return;
    }

    returns
}

/// Normalize angle to [-pi, pi].
fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle;
    while a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    }
    while a < -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    a
}

/// Simple linear policy with Gaussian noise.
struct LinearPolicy {
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
    log_std: Vec<f32>,
}

impl LinearPolicy {
    fn new(obs_size: usize, act_size: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Xavier initialization
        let scale = (2.0 / (obs_size + act_size) as f32).sqrt();

        let weights: Vec<Vec<f32>> = (0..act_size)
            .map(|_| {
                (0..obs_size)
                    .map(|_| rng.r#gen_range(-scale..scale))
                    .collect()
            })
            .collect();

        let biases = vec![0.0; act_size];
        let log_std = vec![-0.5; act_size]; // std = ~0.6

        Self {
            weights,
            biases,
            log_std,
        }
    }

    fn forward(&self, obs: &[f32]) -> Vec<f32> {
        self.weights
            .iter()
            .zip(&self.biases)
            .map(|(w, b)| {
                w.iter().zip(obs).map(|(wi, oi)| wi * oi).sum::<f32>() + b
            })
            .collect()
    }

    fn sample(&self, obs: &[f32], rng: &mut impl Rng) -> (Action, f32) {
        let mean = self.forward(obs);
        let std: Vec<f32> = self.log_std.iter().map(|ls| ls.exp()).collect();

        // Sample from Gaussian
        let actions: Vec<f32> = mean
            .iter()
            .zip(&std)
            .map(|(m, s)| {
                let noise: f32 = rng.r#gen_range(-3.0..3.0) * s;
                (m + noise).clamp(-1.0, 1.0)
            })
            .collect();

        // Compute log probability (simplified)
        let log_prob: f32 = mean
            .iter()
            .zip(&actions)
            .zip(&std)
            .map(|((m, a), s)| {
                let diff = a - m;
                -0.5 * (diff / s).powi(2) - s.ln()
            })
            .sum();

        (Action::new(actions[0], actions[1]), log_prob)
    }

    fn perturb(&mut self, lr: f32, advantage: f32, rng: &mut impl Rng) {
        // Simple evolution strategy-style update
        let noise_scale = (lr * advantage.abs().min(1.0)).max(0.001);
        let direction = advantage.signum();

        for row in &mut self.weights {
            for w in row {
                let noise: f32 = rng.r#gen_range(-1.0..1.0) * noise_scale;
                *w += noise * direction;
            }
        }

        for b in &mut self.biases {
            let noise: f32 = rng.r#gen_range(-1.0..1.0) * noise_scale * 0.1;
            *b += noise * direction;
        }
    }

    /// Save policy in versioned format using PolicyBuilder.
    fn save_versioned(
        &self,
        path: &str,
        name: &str,
        version: &str,
        training_iterations: usize,
        training_episodes: usize,
        avg_reward: f32,
        success_rate: f32,
    ) -> Result<()> {
        let builder = PolicyBuilder::new(name, version, self.weights.clone(), self.biases.clone())
            .description("Navigation policy trained with REINFORCE")
            .log_std(self.log_std.clone())
            .metrics(PolicyMetrics {
                success_rate,
                avg_reward,
                training_iterations,
                training_episodes,
            });

        // Determine output path
        let output_path = if path.ends_with(".json") {
            path.to_string()
        } else {
            // Create filename from name and version
            let filename = format!("{}-v{}.json", name, version);
            if std::path::Path::new(path).is_dir() {
                format!("{}/{}", path, filename)
            } else {
                // Create directory if needed
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                path.to_string()
            }
        };

        builder.save(&output_path)?;
        info!(path = %output_path, "Policy saved");
        Ok(())
    }

    /// Save policy in legacy format (for backwards compatibility).
    #[allow(dead_code)]
    fn save(&self, path: &str) -> Result<()> {
        let data = serde_json::json!({
            "weights": self.weights,
            "biases": self.biases,
            "log_std": self.log_std,
        });
        std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn load(path: &str) -> Result<Self> {
        let data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        
        let weights: Vec<Vec<f32>> = serde_json::from_value(data["weights"].clone())?;
        let biases: Vec<f32> = serde_json::from_value(data["biases"].clone())?;
        let log_std: Vec<f32> = serde_json::from_value(data["log_std"].clone())?;

        Ok(Self { weights, biases, log_std })
    }
}

/// Aggregate statistics across episodes.
struct AggregateStats {
    total_episodes: usize,
    total_reward: f32,
    total_steps: usize,
    successes: usize,
    collisions: usize,
    timeouts: usize,
}

impl AggregateStats {
    fn new() -> Self {
        Self {
            total_episodes: 0,
            total_reward: 0.0,
            total_steps: 0,
            successes: 0,
            collisions: 0,
            timeouts: 0,
        }
    }

    fn add(&mut self, stats: &EpisodeStats) {
        self.total_episodes += 1;
        self.total_reward += stats.total_reward;
        self.total_steps += stats.steps;

        if stats.success {
            self.successes += 1;
        }
        if stats.collision {
            self.collisions += 1;
        }
        if stats.timeout {
            self.timeouts += 1;
        }
    }

    fn print_summary(&self, policy_name: &str) {
        let avg_reward = self.total_reward / self.total_episodes as f32;
        let avg_steps = self.total_steps as f32 / self.total_episodes as f32;
        let success_rate = self.successes as f32 / self.total_episodes as f32 * 100.0;
        let collision_rate = self.collisions as f32 / self.total_episodes as f32 * 100.0;
        let timeout_rate = self.timeouts as f32 / self.total_episodes as f32 * 100.0;

        info!("=== {} Results ===", policy_name);
        info!("Episodes:       {}", self.total_episodes);
        info!("Avg Reward:     {:.2}", avg_reward);
        info!("Avg Steps:      {:.1}", avg_steps);
        info!("Success Rate:   {:.1}%", success_rate);
        info!("Collision Rate: {:.1}%", collision_rate);
        info!("Timeout Rate:   {:.1}%", timeout_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_returns() {
        let rewards = vec![1.0, 1.0, 1.0];
        let returns = compute_returns(&rewards, 0.99);
        
        assert!(returns[0] > returns[1]);
        assert!(returns[1] > returns[2]);
    }

    #[test]
    fn test_linear_policy() {
        let policy = LinearPolicy::new(7, 2, 42);
        let obs = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let mean = policy.forward(&obs);
        assert_eq!(mean.len(), 2);
    }
}
