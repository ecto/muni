//! Action and observation space definitions.
//!
//! Provides Gymnasium-compatible space specifications.

/// Specification for action space.
#[derive(Debug, Clone)]
pub struct ActionSpace {
    /// Dimensionality of the action
    pub shape: Vec<usize>,
    /// Lower bounds for each dimension
    pub low: Vec<f32>,
    /// Upper bounds for each dimension
    pub high: Vec<f32>,
}

impl ActionSpace {
    /// Create a Box action space with given bounds.
    pub fn box_space(shape: Vec<usize>, low: f32, high: f32) -> Self {
        let size: usize = shape.iter().product();
        Self {
            shape,
            low: vec![low; size],
            high: vec![high; size],
        }
    }

    /// Standard BVR action space: [linear_vel, angular_vel] in [-1, 1].
    pub fn bvr_default() -> Self {
        Self::box_space(vec![2], -1.0, 1.0)
    }

    /// Sample a random action uniformly within bounds.
    pub fn sample(&self, rng: &mut impl rand::Rng) -> Vec<f32> {
        self.low
            .iter()
            .zip(&self.high)
            .map(|(&lo, &hi)| rng.r#gen_range(lo..=hi))
            .collect()
    }

    /// Check if an action is within bounds.
    pub fn contains(&self, action: &[f32]) -> bool {
        if action.len() != self.low.len() {
            return false;
        }
        action
            .iter()
            .zip(&self.low)
            .zip(&self.high)
            .all(|((&a, &lo), &hi)| a >= lo && a <= hi)
    }
}

/// Specification for observation space.
#[derive(Debug, Clone)]
pub struct ObservationSpace {
    /// Named components of the observation
    pub components: Vec<ObservationComponent>,
    /// Total flattened size
    pub flat_size: usize,
}

/// A single component of the observation.
#[derive(Debug, Clone)]
pub struct ObservationComponent {
    /// Name of this component
    pub name: String,
    /// Shape of this component
    pub shape: Vec<usize>,
    /// Lower bound
    pub low: f32,
    /// Upper bound
    pub high: f32,
}

impl ObservationSpace {
    /// Create an observation space from components.
    pub fn from_components(components: Vec<ObservationComponent>) -> Self {
        let flat_size = components
            .iter()
            .map(|c| c.shape.iter().product::<usize>())
            .sum();
        Self {
            components,
            flat_size,
        }
    }

    /// Default BVR observation space.
    pub fn bvr_default(lidar_points: usize) -> Self {
        Self::from_components(vec![
            // Pose: [x, y, theta]
            ObservationComponent {
                name: "pose".into(),
                shape: vec![3],
                low: -100.0,
                high: 100.0,
            },
            // Velocity: [linear, angular]
            ObservationComponent {
                name: "velocity".into(),
                shape: vec![2],
                low: -2.0,
                high: 2.0,
            },
            // Goal (relative): [dx, dy]
            ObservationComponent {
                name: "goal".into(),
                shape: vec![2],
                low: -100.0,
                high: 100.0,
            },
            // LiDAR ranges (1D for simplicity)
            ObservationComponent {
                name: "lidar".into(),
                shape: vec![lidar_points],
                low: 0.0,
                high: 40.0,
            },
        ])
    }

    /// Get a component by name.
    pub fn get_component(&self, name: &str) -> Option<&ObservationComponent> {
        self.components.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_space() {
        let space = ActionSpace::bvr_default();
        assert_eq!(space.shape, vec![2]);
        assert!(space.contains(&[0.0, 0.0]));
        assert!(space.contains(&[-1.0, 1.0]));
        assert!(!space.contains(&[1.5, 0.0]));
    }

    #[test]
    fn test_observation_space() {
        let space = ObservationSpace::bvr_default(36);
        // 3 (pose) + 2 (vel) + 2 (goal) + 36 (lidar) = 43
        assert_eq!(space.flat_size, 43);
    }
}
