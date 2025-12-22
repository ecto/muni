//! Policy loading and inference for autonomous navigation.
//!
//! This crate provides:
//! - Versioned policy file format with metadata
//! - Policy loading from disk
//! - Inference for generating actions from observations
//!
//! # Policy Format
//!
//! Policies are stored as JSON files with the following structure:
//!
//! ```json
//! {
//!   "version": "1.0.0",
//!   "name": "nav-simple",
//!   "description": "Basic navigation policy",
//!   "created_at": "2025-12-22T00:00:00Z",
//!   "observation_size": 7,
//!   "action_size": 2,
//!   "architecture": "linear",
//!   "weights": [[...], [...]],
//!   "biases": [...],
//!   "log_std": [...]
//! }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use policy::{Policy, PolicyObservation};
//!
//! let policy = Policy::load("/var/lib/bvr/policies/nav-v1.0.0.json")?;
//! let obs = PolicyObservation::new(pose, velocity, goal_relative);
//! let action = policy.infer(&obs);
//! let twist = action.to_twist(1.5, 2.0);
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info};
use types::Twist;

/// Errors that can occur when loading or using policies.
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("failed to read policy file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("failed to parse policy: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("unsupported policy architecture: {0}")]
    UnsupportedArchitecture(String),

    #[error("observation size mismatch: expected {expected}, got {actual}")]
    ObservationSizeMismatch { expected: usize, actual: usize },

    #[error("policy file not found: {0}")]
    NotFound(String),
}

/// Policy metadata for versioning and identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    /// Semantic version (e.g., "1.0.0")
    pub version: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this policy does
    #[serde(default)]
    pub description: String,
    /// ISO 8601 timestamp when the policy was created
    #[serde(default)]
    pub created_at: String,
    /// Git commit or training run ID
    #[serde(default)]
    pub training_id: Option<String>,
    /// Training metrics (success rate, avg reward, etc.)
    #[serde(default)]
    pub metrics: Option<PolicyMetrics>,
}

/// Training metrics recorded with the policy.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyMetrics {
    /// Success rate during evaluation (0.0 to 1.0)
    #[serde(default)]
    pub success_rate: f32,
    /// Average episode reward
    #[serde(default)]
    pub avg_reward: f32,
    /// Number of training iterations
    #[serde(default)]
    pub training_iterations: usize,
    /// Number of episodes used for training
    #[serde(default)]
    pub training_episodes: usize,
}

/// Neural network architecture type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    /// Simple linear policy: action = W * obs + b
    Linear,
    /// Multi-layer perceptron (future)
    Mlp,
}

impl Default for Architecture {
    fn default() -> Self {
        Self::Linear
    }
}

/// Serializable policy file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyFile {
    /// Semantic version
    pub version: String,
    /// Policy name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Creation timestamp
    #[serde(default)]
    pub created_at: String,
    /// Training run identifier
    #[serde(default)]
    pub training_id: Option<String>,
    /// Training metrics
    #[serde(default)]
    pub metrics: Option<PolicyMetrics>,
    /// Observation vector size
    pub observation_size: usize,
    /// Action vector size
    pub action_size: usize,
    /// Network architecture
    #[serde(default)]
    pub architecture: Architecture,
    /// Weight matrix (action_size x observation_size)
    pub weights: Vec<Vec<f32>>,
    /// Bias vector (action_size)
    pub biases: Vec<f32>,
    /// Log standard deviation for stochastic policies
    #[serde(default)]
    pub log_std: Option<Vec<f32>>,
}

/// Observation input for policy inference.
#[derive(Debug, Clone, Default)]
pub struct PolicyObservation {
    /// Robot pose [x, y, theta] (normalized)
    pub pose: [f32; 3],
    /// Robot velocity [linear, angular] (normalized)
    pub velocity: [f32; 2],
    /// Goal position relative to robot [dx, dy] (normalized)
    pub goal_relative: [f32; 2],
}

impl PolicyObservation {
    /// Create a new observation from raw values.
    ///
    /// Values should already be normalized to approximately [-1, 1].
    pub fn new(pose: [f32; 3], velocity: [f32; 2], goal_relative: [f32; 2]) -> Self {
        Self {
            pose,
            velocity,
            goal_relative,
        }
    }

    /// Create from unnormalized values with default scaling.
    pub fn from_raw(
        x: f64,
        y: f64,
        theta: f64,
        linear_vel: f64,
        angular_vel: f64,
        goal_x: f64,
        goal_y: f64,
        config: &NormalizationConfig,
    ) -> Self {
        // Compute goal in robot frame
        let dx = goal_x - x;
        let dy = goal_y - y;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let goal_robot_x = dx * cos_t + dy * sin_t;
        let goal_robot_y = -dx * sin_t + dy * cos_t;

        Self {
            pose: [
                (x as f32 / config.max_position).clamp(-1.0, 1.0),
                (y as f32 / config.max_position).clamp(-1.0, 1.0),
                (theta as f32 / std::f32::consts::PI).clamp(-1.0, 1.0),
            ],
            velocity: [
                (linear_vel as f32 / config.max_linear_vel).clamp(-1.0, 1.0),
                (angular_vel as f32 / config.max_angular_vel).clamp(-1.0, 1.0),
            ],
            goal_relative: [
                (goal_robot_x as f32 / config.max_position).clamp(-1.0, 1.0),
                (goal_robot_y as f32 / config.max_position).clamp(-1.0, 1.0),
            ],
        }
    }

    /// Convert to flat vector for neural network input.
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.pose[0],
            self.pose[1],
            self.pose[2],
            self.velocity[0],
            self.velocity[1],
            self.goal_relative[0],
            self.goal_relative[1],
        ]
    }

    /// Size of the flattened observation.
    pub const fn size() -> usize {
        7 // 3 pose + 2 velocity + 2 goal
    }
}

/// Configuration for normalizing observations.
#[derive(Debug, Clone)]
pub struct NormalizationConfig {
    /// Maximum position for normalization
    pub max_position: f32,
    /// Maximum linear velocity
    pub max_linear_vel: f32,
    /// Maximum angular velocity
    pub max_angular_vel: f32,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            max_position: 50.0,
            max_linear_vel: 2.0,
            max_angular_vel: 2.0,
        }
    }
}

/// Action output from policy inference.
#[derive(Debug, Clone, Copy, Default)]
pub struct PolicyAction {
    /// Linear velocity command (-1 to 1)
    pub linear: f32,
    /// Angular velocity command (-1 to 1)
    pub angular: f32,
}

impl PolicyAction {
    /// Create a new action.
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
}

/// A loaded navigation policy ready for inference.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Expected observation size
    observation_size: usize,
    /// Action output size (unused but kept for metadata)
    #[allow(dead_code)]
    action_size: usize,
    /// Weight matrix (action_size x observation_size)
    weights: Vec<Vec<f32>>,
    /// Bias vector
    biases: Vec<f32>,
}

impl Policy {
    /// Load a policy from a JSON file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PolicyError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(PolicyError::NotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)?;
        let file: PolicyFile = serde_json::from_str(&content)?;

        if file.architecture != Architecture::Linear {
            return Err(PolicyError::UnsupportedArchitecture(format!(
                "{:?}",
                file.architecture
            )));
        }

        info!(
            name = %file.name,
            version = %file.version,
            obs_size = file.observation_size,
            act_size = file.action_size,
            "Loaded policy"
        );

        Ok(Self {
            metadata: PolicyMetadata {
                version: file.version,
                name: file.name,
                description: file.description,
                created_at: file.created_at,
                training_id: file.training_id,
                metrics: file.metrics,
            },
            observation_size: file.observation_size,
            action_size: file.action_size,
            weights: file.weights,
            biases: file.biases,
        })
    }

    /// Load a policy from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, PolicyError> {
        let file: PolicyFile = serde_json::from_str(json)?;

        if file.architecture != Architecture::Linear {
            return Err(PolicyError::UnsupportedArchitecture(format!(
                "{:?}",
                file.architecture
            )));
        }

        Ok(Self {
            metadata: PolicyMetadata {
                version: file.version,
                name: file.name,
                description: file.description,
                created_at: file.created_at,
                training_id: file.training_id,
                metrics: file.metrics,
            },
            observation_size: file.observation_size,
            action_size: file.action_size,
            weights: file.weights,
            biases: file.biases,
        })
    }

    /// Get the expected observation size.
    pub fn observation_size(&self) -> usize {
        self.observation_size
    }

    /// Run inference to get an action from an observation.
    pub fn infer(&self, obs: &PolicyObservation) -> Result<PolicyAction, PolicyError> {
        let obs_vec = obs.to_vec();

        if obs_vec.len() != self.observation_size {
            return Err(PolicyError::ObservationSizeMismatch {
                expected: self.observation_size,
                actual: obs_vec.len(),
            });
        }

        let output = self.forward(&obs_vec);

        debug!(
            linear = output[0],
            angular = output[1],
            "Policy inference"
        );

        Ok(PolicyAction::new(output[0], output[1]))
    }

    /// Run inference from a raw observation vector.
    pub fn infer_raw(&self, obs: &[f32]) -> Result<PolicyAction, PolicyError> {
        if obs.len() != self.observation_size {
            return Err(PolicyError::ObservationSizeMismatch {
                expected: self.observation_size,
                actual: obs.len(),
            });
        }

        let output = self.forward(obs);
        Ok(PolicyAction::new(output[0], output[1]))
    }

    /// Forward pass through the policy network.
    fn forward(&self, obs: &[f32]) -> Vec<f32> {
        self.weights
            .iter()
            .zip(&self.biases)
            .map(|(w, b)| {
                let sum: f32 = w.iter().zip(obs).map(|(wi, oi)| wi * oi).sum();
                // Tanh activation to bound output to [-1, 1]
                (sum + b).tanh()
            })
            .collect()
    }

    /// Get a reference to the policy name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the policy version.
    pub fn version(&self) -> &str {
        &self.metadata.version
    }
}

/// Policy manager for loading and managing policies from a directory.
#[derive(Debug)]
pub struct PolicyManager {
    /// Directory containing policy files
    policy_dir: std::path::PathBuf,
    /// Currently loaded policy
    current: Option<Policy>,
}

impl PolicyManager {
    /// Create a new policy manager with the given policy directory.
    pub fn new<P: AsRef<Path>>(policy_dir: P) -> Self {
        Self {
            policy_dir: policy_dir.as_ref().to_path_buf(),
            current: None,
        }
    }

    /// Load the default policy (latest version or specified by name).
    pub fn load_default(&mut self) -> Result<&Policy, PolicyError> {
        // Look for "default.json" or the latest version
        let default_path = self.policy_dir.join("default.json");
        if default_path.exists() {
            let policy = Policy::load(&default_path)?;
            self.current = Some(policy);
            return Ok(self.current.as_ref().unwrap());
        }

        // Find the first .json policy file
        let entries = std::fs::read_dir(&self.policy_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                let policy = Policy::load(&path)?;
                self.current = Some(policy);
                return Ok(self.current.as_ref().unwrap());
            }
        }

        Err(PolicyError::NotFound(format!(
            "no policies found in {}",
            self.policy_dir.display()
        )))
    }

    /// Load a specific policy by name.
    pub fn load(&mut self, name: &str) -> Result<&Policy, PolicyError> {
        let path = self.policy_dir.join(format!("{}.json", name));
        let policy = Policy::load(&path)?;
        self.current = Some(policy);
        Ok(self.current.as_ref().unwrap())
    }

    /// Load a policy from an absolute path.
    pub fn load_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<&Policy, PolicyError> {
        let policy = Policy::load(path)?;
        self.current = Some(policy);
        Ok(self.current.as_ref().unwrap())
    }

    /// Get the currently loaded policy.
    pub fn current(&self) -> Option<&Policy> {
        self.current.as_ref()
    }

    /// List available policies in the directory.
    pub fn list_policies(&self) -> Result<Vec<PolicyMetadata>, PolicyError> {
        let mut policies = Vec::new();

        if !self.policy_dir.exists() {
            return Ok(policies);
        }

        let entries = std::fs::read_dir(&self.policy_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(policy) = Policy::load(&path) {
                    policies.push(policy.metadata);
                }
            }
        }

        Ok(policies)
    }
}

/// Builder for creating policy files (used by training).
#[derive(Debug, Clone)]
pub struct PolicyBuilder {
    file: PolicyFile,
}

impl PolicyBuilder {
    /// Create a new policy builder with required fields.
    pub fn new(name: &str, version: &str, weights: Vec<Vec<f32>>, biases: Vec<f32>) -> Self {
        let observation_size = weights.first().map(|w| w.len()).unwrap_or(0);
        let action_size = weights.len();

        Self {
            file: PolicyFile {
                version: version.to_string(),
                name: name.to_string(),
                description: String::new(),
                created_at: chrono_now(),
                training_id: None,
                metrics: None,
                observation_size,
                action_size,
                architecture: Architecture::Linear,
                weights,
                biases,
                log_std: None,
            },
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: &str) -> Self {
        self.file.description = desc.to_string();
        self
    }

    /// Set the training ID.
    pub fn training_id(mut self, id: &str) -> Self {
        self.file.training_id = Some(id.to_string());
        self
    }

    /// Set training metrics.
    pub fn metrics(mut self, metrics: PolicyMetrics) -> Self {
        self.file.metrics = Some(metrics);
        self
    }

    /// Set log standard deviation for stochastic inference.
    pub fn log_std(mut self, log_std: Vec<f32>) -> Self {
        self.file.log_std = Some(log_std);
        self
    }

    /// Build and serialize to JSON string.
    pub fn to_json(&self) -> Result<String, PolicyError> {
        Ok(serde_json::to_string_pretty(&self.file)?)
    }

    /// Build and save to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), PolicyError> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Get current timestamp in ISO 8601 format (simplified, no external deps).
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple conversion (approximate, ignores leap seconds)
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Days since 1970-01-01 to approximate date
    let years = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30).min(11) + 1;
    let day = (day_of_year % 30) + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        years, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policy_json() -> &'static str {
        r#"{
            "version": "1.0.0",
            "name": "test-policy",
            "description": "Test navigation policy",
            "observation_size": 7,
            "action_size": 2,
            "architecture": "linear",
            "weights": [
                [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7],
                [-0.1, -0.2, -0.3, -0.4, -0.5, 0.8, 0.9]
            ],
            "biases": [0.0, 0.0]
        }"#
    }

    #[test]
    fn test_policy_load_from_json() {
        let policy = Policy::from_json(sample_policy_json()).unwrap();
        assert_eq!(policy.name(), "test-policy");
        assert_eq!(policy.version(), "1.0.0");
        assert_eq!(policy.observation_size(), 7);
    }

    #[test]
    fn test_policy_inference() {
        let policy = Policy::from_json(sample_policy_json()).unwrap();
        let obs = PolicyObservation::new([0.0, 0.0, 0.0], [0.0, 0.0], [1.0, 0.0]);

        let action = policy.infer(&obs).unwrap();
        assert!(action.linear >= -1.0 && action.linear <= 1.0);
        assert!(action.angular >= -1.0 && action.angular <= 1.0);
    }

    #[test]
    fn test_policy_observation_size() {
        assert_eq!(PolicyObservation::size(), 7);
    }

    #[test]
    fn test_policy_action_to_twist() {
        let action = PolicyAction::new(0.5, -0.3);
        let twist = action.to_twist(2.0, 1.5);

        assert!((twist.linear - 1.0).abs() < 0.01);
        assert!((twist.angular - (-0.45)).abs() < 0.01);
    }

    #[test]
    fn test_policy_builder() {
        let weights = vec![vec![0.1, 0.2, 0.3], vec![0.4, 0.5, 0.6]];
        let biases = vec![0.0, 0.0];

        let builder = PolicyBuilder::new("my-policy", "0.1.0", weights, biases)
            .description("A test policy")
            .metrics(PolicyMetrics {
                success_rate: 0.85,
                avg_reward: 123.4,
                training_iterations: 1000,
                training_episodes: 10000,
            });

        let json = builder.to_json().unwrap();
        assert!(json.contains("my-policy"));
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn test_observation_from_raw() {
        let config = NormalizationConfig::default();
        let obs = PolicyObservation::from_raw(10.0, 5.0, 0.5, 1.0, 0.5, 15.0, 5.0, &config);

        // Check normalization happened
        assert!(obs.pose[0].abs() <= 1.0);
        assert!(obs.pose[1].abs() <= 1.0);
        assert!(obs.velocity[0].abs() <= 1.0);
    }
}
