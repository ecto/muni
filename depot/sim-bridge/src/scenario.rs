//! Parse scenario.toml: rovers, world configuration, spawn points.

use crate::world::World;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub world: WorldConfig,
    #[serde(rename = "rover")]
    pub rovers: Vec<RoverConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    #[serde(rename = "type")]
    pub world_type: String,
    #[serde(default = "default_size")]
    pub size: f32,
    #[serde(default = "default_wall_height")]
    pub wall_height: f32,
    #[serde(default)]
    pub num_obstacles: usize,
    #[serde(default = "default_seed")]
    pub seed: u64,
}

fn default_size() -> f32 {
    20.0
}
fn default_wall_height() -> f32 {
    2.0
}
fn default_seed() -> u64 {
    42
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoverConfig {
    pub id: String,
    pub spawn: SpawnPoint,
    pub can_port: u16,
    pub point_cloud_port: u16,
    pub imu_port: u16,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct SpawnPoint {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

impl Scenario {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let scenario: Scenario = toml::from_str(&content)?;
        Ok(scenario)
    }

    pub fn build_world(&self) -> World {
        match self.world.world_type.as_str() {
            "empty_room" => World::empty_room(self.world.size, self.world.wall_height),
            "random_obstacles" => World::random_obstacles(
                self.world.size,
                self.world.wall_height,
                self.world.num_obstacles,
                self.world.seed,
            ),
            _ => {
                tracing::warn!(
                    world_type = %self.world.world_type,
                    "Unknown world type, using empty room"
                );
                World::empty_room(self.world.size, self.world.wall_height)
            }
        }
    }
}
