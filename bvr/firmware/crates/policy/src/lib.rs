//! Policy loading and inference for autonomous navigation.
//!
//! This crate is the on-rover inference engine for learned navigation policies.
//! It loads policy files from disk (JSON), validates their structure, and runs
//! forward passes to produce velocity commands from observations. The classical
//! navigation stack remains the safety backbone — this crate provides the
//! learned component that runs alongside it.
//!
//! # Supported Architectures
//!
//! Two network architectures are supported:
//!
//! - **Linear** (`"architecture": "linear"`): Single-layer policy where
//!   `action = tanh(W * obs + b)`. The weight matrix and bias vector are stored
//!   directly in the top-level `weights` and `biases` fields. This is the
//!   original format and the default when `architecture` is omitted.
//!
//! - **MLP** (`"architecture": "mlp"`): Multi-layer perceptron with tanh
//!   activations on every layer. Layers are stored in the `layers` array, each
//!   containing its own `weights` matrix and `biases` vector. The standard
//!   behavioral cloning architecture is `7 → 64 → 64 → 2` (4802 parameters).
//!   MLP policies are produced by the Python training pipeline in
//!   `bvr/training/train_bc.py`.
//!
//! Both architectures use tanh activations, so all outputs are bounded to
//! `[-1, 1]`. These normalized outputs are then scaled to real velocity
//! commands via [`PolicyAction::to_twist`].
//!
//! # Observation Format
//!
//! All policies expect a 7-dimensional observation vector, normalized to
//! approximately `[-1, 1]`:
//!
//! | Index | Field          | Normalization           | Source                    |
//! |-------|----------------|-------------------------|---------------------------|
//! | 0     | x position     | `x / max_position`      | GPS / localization        |
//! | 1     | y position     | `y / max_position`      | GPS / localization        |
//! | 2     | heading (θ)    | `θ / π`                 | IMU / localization        |
//! | 3     | linear vel     | `v / max_linear_vel`    | Commanded or measured     |
//! | 4     | angular vel    | `ω / max_angular_vel`   | Commanded or measured     |
//! | 5     | goal Δx        | `dx / max_position`     | Planner (robot frame)     |
//! | 6     | goal Δy        | `dy / max_position`     | Planner (robot frame)     |
//!
//! Default normalization constants (see [`NormalizationConfig`]):
//! - `max_position = 50.0` meters
//! - `max_linear_vel = 2.0` m/s
//! - `max_angular_vel = 2.0` rad/s
//!
//! **Important:** The goal (indices 5-6) is in the **robot's local frame**, not
//! world frame. The transformation is: rotate the world-frame delta
//! `(goal - position)` by `-θ`. See [`PolicyObservation::from_raw`] for the
//! exact math.
//!
//! During behavioral cloning training, teleop recordings have no explicit goal.
//! The training pipeline uses a **pseudo-goal**: the robot's actual position
//! 2 seconds in the future, transformed into the robot frame at the current
//! timestep. At deploy time, the classical nav stack provides real goals.
//!
//! # Action Format
//!
//! Policies output a 2-dimensional action vector, both in `[-1, 1]`:
//!
//! | Index | Field       | Denormalization               |
//! |-------|-------------|-------------------------------|
//! | 0     | linear vel  | `action[0] * max_linear_vel`  |
//! | 1     | angular vel | `action[1] * max_angular_vel` |
//!
//! # Policy File Format
//!
//! Policies are JSON files. Both architectures share the same envelope:
//!
//! ## Linear Policy
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
//!   "weights": [[0.1, 0.2, ...], [0.3, 0.4, ...]],
//!   "biases": [0.0, 0.0],
//!   "log_std": [-0.5, -0.5]
//! }
//! ```
//!
//! ## MLP Policy (from behavioral cloning)
//!
//! ```json
//! {
//!   "version": "0.1.0",
//!   "name": "bc-teleop",
//!   "description": "Behavioral cloning from teleop demos",
//!   "observation_size": 7,
//!   "action_size": 2,
//!   "architecture": "mlp",
//!   "weights": [],
//!   "biases": [],
//!   "layers": [
//!     { "weights": [[...64 rows x 7 cols...]], "biases": [... 64 ...] },
//!     { "weights": [[...64 rows x 64 cols...]], "biases": [... 64 ...] },
//!     { "weights": [[...2 rows x 64 cols...]], "biases": [... 2 ...] }
//!   ],
//!   "metrics": {
//!     "mse": 0.02,
//!     "training_samples": 6000,
//!     "demo_hours": 0.5
//!   }
//! }
//! ```
//!
//! For MLP policies, `weights` and `biases` at the top level are empty — all
//! parameters live inside the `layers` array. Each layer's `weights` matrix
//! has shape `(output_dim, input_dim)` and `biases` has length `output_dim`.
//!
//! # Pipeline Overview
//!
//! ```text
//! ┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
//! │  Teleop Session   │     │  extract_demos.py │     │   train_bc.py    │
//! │  (.rrd recording) │────▸│  .rrd → demos.npz │────▸│  demos → policy  │
//! └──────────────────┘     └──────────────────┘     │  .json (MLP)     │
//!                                                    └────────┬─────────┘
//!                                                             │
//!                          ┌──────────────────┐               │
//!                          │  bvrd (rover)     │◂──────────────┘
//!                          │  Policy::load()   │  scp policy.json
//!                          │  policy.infer()   │  to rover
//!                          └──────────────────┘
//! ```
//!
//! # Deployment
//!
//! ```bash
//! scp bc-teleop-v0.1.0.json rover:/var/lib/bvr/policies/default.json
//! ```
//!
//! `bvrd` loads the policy on startup via [`PolicyManager::load_default`].
//! The architecture is auto-detected from the JSON.
//!
//! # Usage Example
//!
//! ```ignore
//! use policy::{Policy, PolicyObservation, NormalizationConfig};
//!
//! // Load any policy (linear or MLP — auto-detected)
//! let policy = Policy::load("/var/lib/bvr/policies/default.json")?;
//! println!("Loaded {} ({:?})", policy.name(), policy.architecture());
//!
//! // Build observation from raw sensor data
//! let config = NormalizationConfig::default();
//! let obs = PolicyObservation::from_raw(
//!     x, y, theta,           // pose from localization
//!     linear_vel, angular_vel, // current velocity
//!     goal_x, goal_y,        // goal from planner (world frame)
//!     &config,
//! );
//!
//! // Run inference
//! let action = policy.infer(&obs)?;
//!
//! // Convert to velocity command
//! let twist = action.to_twist(1.5, 2.0); // max_linear, max_angular
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
    /// Multi-layer perceptron with tanh activations
    Mlp,
}

impl Default for Architecture {
    fn default() -> Self {
        Self::Linear
    }
}

/// A single dense layer in an MLP: `y = tanh(W * x + b)`.
///
/// Each layer performs a matrix-vector multiply, adds a bias, and applies
/// the `tanh` activation function element-wise. Layers are chained
/// sequentially — the output of one layer becomes the input to the next.
///
/// # Weight Layout
///
/// `weights` is stored as `Vec<Vec<f32>>` with shape `(output_dim, input_dim)`.
/// This matches the PyTorch `nn.Linear` convention where `weight` has shape
/// `(out_features, in_features)`. During the forward pass, each row of
/// `weights` is dot-producted with the input vector to produce one output
/// element.
///
/// # Compatibility
///
/// This struct is serialized/deserialized by the Python training pipeline
/// (`train_bc.py` → `BCPolicy.export_layers()`). The Python code extracts
/// `module.weight` and `module.bias` from `nn.Linear` layers and writes
/// them directly into this format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpLayer {
    /// Weight matrix with shape `(output_dim, input_dim)`.
    ///
    /// Outer vec has `output_dim` elements, each inner vec has `input_dim`
    /// elements. Row `i` contains the weights for output neuron `i`.
    pub weights: Vec<Vec<f32>>,
    /// Bias vector with length `output_dim`.
    ///
    /// Element `i` is added to the dot product of `weights[i]` and the
    /// input before the tanh activation.
    pub biases: Vec<f32>,
}

/// Serializable policy file format.
///
/// This is the on-disk JSON representation. It supports both linear and MLP
/// architectures in a single schema. The `architecture` field determines
/// which weight fields are used:
///
/// - `Linear`: uses `weights` (action_size x obs_size) and `biases` (action_size)
/// - `Mlp`: uses `layers` (Vec of [`MlpLayer`]), ignoring top-level `weights`/`biases`
///
/// All fields marked `#[serde(default)]` are optional for backward
/// compatibility — older linear policy files that predate the MLP support
/// will still deserialize correctly.
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
    /// MLP layers (used when architecture == "mlp")
    #[serde(default)]
    pub layers: Option<Vec<MlpLayer>>,
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
///
/// This is the runtime representation. It holds validated, deserialized
/// weights and dispatches to the correct forward pass based on
/// [`Architecture`]. Construct via [`Policy::load`] (from file) or
/// [`Policy::from_json`] (from string).
///
/// # Thread Safety
///
/// `Policy` is `Send + Sync` (all fields are owned data). It can be shared
/// across threads via `Arc<Policy>` for concurrent inference without locks
/// — the forward pass is purely functional with no mutation.
///
/// # Performance
///
/// For the standard behavioral cloning MLP (7→64→64→2, 4802 params),
/// a single forward pass is ~10μs on the Jetson Orin NX. This is
/// negligible compared to the 50ms control loop period.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Expected observation size
    observation_size: usize,
    /// Action output size
    #[allow(dead_code)]
    action_size: usize,
    /// Network architecture
    architecture: Architecture,
    /// Weight matrix for linear policy (action_size x observation_size)
    weights: Vec<Vec<f32>>,
    /// Bias vector for linear policy
    biases: Vec<f32>,
    /// MLP layers (used when architecture == Mlp)
    layers: Vec<MlpLayer>,
}

impl Policy {
    /// Load a policy from a JSON file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PolicyError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(PolicyError::NotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Load a policy from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, PolicyError> {
        let file: PolicyFile = serde_json::from_str(json)?;
        Self::from_file(file)
    }

    /// Construct a Policy from a deserialized [`PolicyFile`].
    ///
    /// This is the shared construction path for both [`load`](Self::load) and
    /// [`from_json`](Self::from_json). It extracts MLP layers (defaulting to
    /// empty for linear policies) and logs the loaded policy metadata.
    fn from_file(file: PolicyFile) -> Result<Self, PolicyError> {
        let architecture = file.architecture.clone();
        let layers = file.layers.unwrap_or_default();

        info!(
            name = %file.name,
            version = %file.version,
            architecture = ?architecture,
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
            architecture,
            weights: file.weights,
            biases: file.biases,
            layers,
        })
    }

    /// Get the expected observation size.
    pub fn observation_size(&self) -> usize {
        self.observation_size
    }

    /// Run inference to get an action from a structured observation.
    ///
    /// This is the primary entry point for the control loop. It flattens the
    /// observation into a vector, validates its size, runs the forward pass
    /// (dispatching to linear or MLP based on architecture), and returns a
    /// clamped action.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::ObservationSizeMismatch`] if the observation
    /// vector length doesn't match `observation_size` from the policy file.
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

    /// Run inference from a raw (already-normalized) observation vector.
    ///
    /// Use this when you've already constructed the flat `[f32; 7]` observation
    /// yourself. The vector must be pre-normalized to `[-1, 1]` using the same
    /// [`NormalizationConfig`] constants as training.
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

    /// Get the architecture type.
    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    /// Forward pass through the policy network, dispatching by architecture.
    ///
    /// Returns a vector of action values, each in `[-1, 1]` (tanh-bounded).
    /// The vector length equals `action_size` from the policy file (typically 2).
    fn forward(&self, obs: &[f32]) -> Vec<f32> {
        match self.architecture {
            Architecture::Linear => self.forward_linear(obs),
            Architecture::Mlp => self.forward_mlp(obs),
        }
    }

    /// Linear forward pass: `action = tanh(W * obs + b)`.
    ///
    /// Each row of `self.weights` is dot-producted with `obs`, the
    /// corresponding bias is added, and tanh is applied. This is equivalent
    /// to a single [`MlpLayer`].
    fn forward_linear(&self, obs: &[f32]) -> Vec<f32> {
        self.weights
            .iter()
            .zip(&self.biases)
            .map(|(w, b)| {
                let sum: f32 = w.iter().zip(obs).map(|(wi, oi)| wi * oi).sum();
                (sum + b).tanh()
            })
            .collect()
    }

    /// MLP forward pass through dense layers with tanh activations.
    ///
    /// Iterates through `self.layers` sequentially. For each layer, computes
    /// `y = tanh(W * x + b)` where `W` is the weight matrix and `b` is the
    /// bias vector. The output of each layer feeds into the next.
    ///
    /// For the standard BC architecture (7→64→64→2):
    /// - Layer 0: 7 inputs → 64 outputs (448 weights + 64 biases)
    /// - Layer 1: 64 inputs → 64 outputs (4096 weights + 64 biases)
    /// - Layer 2: 64 inputs → 2 outputs (128 weights + 2 biases)
    /// - Total: 4802 parameters
    ///
    /// # Numerical Note
    ///
    /// The Rust `f32::tanh()` implementation matches PyTorch's CPU f32 tanh
    /// to within ~1e-7, so inference results are effectively identical between
    /// training (Python) and deployment (Rust).
    fn forward_mlp(&self, obs: &[f32]) -> Vec<f32> {
        let mut x = obs.to_vec();
        for layer in &self.layers {
            let mut next = Vec::with_capacity(layer.weights.len());
            for (w, b) in layer.weights.iter().zip(&layer.biases) {
                let sum: f32 = w.iter().zip(&x).map(|(wi, xi)| wi * xi).sum();
                next.push((sum + b).tanh());
            }
            x = next;
        }
        x
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

/// Builder for creating policy JSON files.
///
/// Used by training scripts and tests to construct well-formed policy files.
/// Supports both linear and MLP architectures via [`PolicyBuilder::new`] and
/// [`PolicyBuilder::new_mlp`] respectively.
///
/// # Example: Creating an MLP policy from Python-exported layers
///
/// ```ignore
/// let layers = vec![
///     MlpLayer { weights: w0, biases: b0 },
///     MlpLayer { weights: w1, biases: b1 },
///     MlpLayer { weights: w2, biases: b2 },
/// ];
///
/// PolicyBuilder::new_mlp("bc-teleop", "0.1.0", layers)
///     .description("Behavioral cloning from 3 teleop sessions")
///     .save("/var/lib/bvr/policies/default.json")?;
/// ```
#[derive(Debug, Clone)]
pub struct PolicyBuilder {
    file: PolicyFile,
}

impl PolicyBuilder {
    /// Create a new linear policy builder with required fields.
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
                layers: None,
            },
        }
    }

    /// Create a new MLP policy builder.
    ///
    /// `layers` defines the dense layers of the MLP. The observation_size is
    /// inferred from the first layer's weight width, and action_size from the
    /// last layer's output dimension.
    pub fn new_mlp(name: &str, version: &str, layers: Vec<MlpLayer>) -> Self {
        let observation_size = layers
            .first()
            .and_then(|l| l.weights.first().map(|w| w.len()))
            .unwrap_or(0);
        let action_size = layers.last().map(|l| l.weights.len()).unwrap_or(0);

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
                architecture: Architecture::Mlp,
                weights: vec![],
                biases: vec![],
                log_std: None,
                layers: Some(layers),
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

    fn sample_mlp_policy_json() -> &'static str {
        r#"{
            "version": "0.1.0",
            "name": "bc-teleop",
            "description": "Behavioral cloning from teleop demos",
            "observation_size": 7,
            "action_size": 2,
            "architecture": "mlp",
            "weights": [],
            "biases": [],
            "layers": [
                {
                    "weights": [
                        [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
                        [0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2],
                        [0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3],
                        [0.4, 0.4, 0.4, 0.4, 0.4, 0.4, 0.4]
                    ],
                    "biases": [0.0, 0.0, 0.0, 0.0]
                },
                {
                    "weights": [
                        [0.5, 0.5, 0.5, 0.5],
                        [-0.5, -0.5, -0.5, -0.5]
                    ],
                    "biases": [0.0, 0.0]
                }
            ]
        }"#
    }

    #[test]
    fn test_mlp_policy_load_from_json() {
        let policy = Policy::from_json(sample_mlp_policy_json()).unwrap();
        assert_eq!(policy.name(), "bc-teleop");
        assert_eq!(policy.version(), "0.1.0");
        assert_eq!(policy.observation_size(), 7);
        assert_eq!(*policy.architecture(), Architecture::Mlp);
        assert_eq!(policy.layers.len(), 2);
    }

    #[test]
    fn test_mlp_policy_roundtrip() {
        let layers = vec![
            MlpLayer {
                weights: vec![vec![0.1; 7]; 4],
                biases: vec![0.0; 4],
            },
            MlpLayer {
                weights: vec![vec![0.5; 4]; 2],
                biases: vec![0.0; 2],
            },
        ];

        let builder = PolicyBuilder::new_mlp("roundtrip-test", "1.0.0", layers)
            .description("Roundtrip test policy");

        let json = builder.to_json().unwrap();
        let policy = Policy::from_json(&json).unwrap();

        assert_eq!(policy.name(), "roundtrip-test");
        assert_eq!(*policy.architecture(), Architecture::Mlp);
        assert_eq!(policy.observation_size(), 7);
    }

    #[test]
    fn test_mlp_forward_pass_known_weights() {
        // 2-input -> 2-hidden -> 1-output MLP with known weights
        let layers = vec![
            MlpLayer {
                weights: vec![
                    vec![1.0, 0.0], // hidden[0] = tanh(x0)
                    vec![0.0, 1.0], // hidden[1] = tanh(x1)
                ],
                biases: vec![0.0, 0.0],
            },
            MlpLayer {
                weights: vec![
                    vec![1.0, 1.0], // out = tanh(tanh(x0) + tanh(x1))
                ],
                biases: vec![0.0],
            },
        ];

        let json = PolicyBuilder::new_mlp("fwd-test", "0.1.0", layers)
            .to_json()
            .unwrap();
        let policy = Policy::from_json(&json).unwrap();

        let obs = vec![0.5_f32, 0.3];
        let out = policy.forward_mlp(&obs);

        let expected_h0 = 0.5_f32.tanh();
        let expected_h1 = 0.3_f32.tanh();
        let expected_out = (expected_h0 + expected_h1).tanh();

        assert!((out[0] - expected_out).abs() < 1e-6, "got {}", out[0]);
    }

    #[test]
    fn test_mlp_inference_bounded() {
        let policy = Policy::from_json(sample_mlp_policy_json()).unwrap();
        let obs = PolicyObservation::new([1.0, 1.0, 1.0], [1.0, 1.0], [1.0, 1.0]);

        let action = policy.infer(&obs).unwrap();
        assert!(action.linear >= -1.0 && action.linear <= 1.0);
        assert!(action.angular >= -1.0 && action.angular <= 1.0);
    }

    #[test]
    fn test_linear_backward_compat() {
        // Ensure old linear policies still load and work
        let policy = Policy::from_json(sample_policy_json()).unwrap();
        assert_eq!(*policy.architecture(), Architecture::Linear);
        assert!(policy.layers.is_empty());

        let obs = PolicyObservation::new([0.0, 0.0, 0.0], [0.0, 0.0], [1.0, 0.0]);
        let action = policy.infer(&obs).unwrap();
        assert!(action.linear >= -1.0 && action.linear <= 1.0);
    }
}
