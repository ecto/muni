//! sim-bridge — Multi-rover simulation service.
//!
//! Manages N simulated rovers with physics, CAN simulation, and geometric LiDAR.
//! Single source of truth for all rover state.

mod can_protocol;
mod imu_sim;
mod lidar_sim;
mod livox_protocol;
mod physics;
mod rover;
mod scenario;
mod tool;
mod vesc;
mod world;

use axum::{extract::State, routing::get, Json, Router};
use clap::Parser;
use rover::{RoverStatus, PoseStatus};
use scenario::Scenario;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "sim-bridge", about = "Multi-rover simulation service")]
struct Args {
    /// Path to scenario configuration file
    #[arg(short, long, default_value = "bvr/firmware/config/sim/scenario.toml")]
    scenario: String,

    /// HTTP status API port
    #[arg(short, long, default_value = "4900")]
    port: u16,
}

type SharedStatus = Arc<Vec<Arc<tokio::sync::Mutex<RoverStatus>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!(scenario = %args.scenario, "Loading scenario");
    let scenario = Scenario::load(&args.scenario)?;

    info!(
        world_type = %scenario.world.world_type,
        size = scenario.world.size,
        rovers = scenario.rovers.len(),
        "Building world"
    );
    let world = Arc::new(scenario.build_world());

    // Create per-rover status handles
    let mut statuses: Vec<Arc<tokio::sync::Mutex<RoverStatus>>> = Vec::new();

    for config in &scenario.rovers {
        let status = Arc::new(tokio::sync::Mutex::new(RoverStatus {
            id: config.id.clone(),
            connected: false,
            pose: PoseStatus {
                x: config.spawn.x,
                y: config.spawn.y,
                theta: config.spawn.theta,
            },
        }));

        statuses.push(status.clone());

        let rover_config = config.clone();
        let rover_world = world.clone();
        tokio::spawn(async move {
            rover::run_rover(rover_config, rover_world, status).await;
        });
    }

    let shared: SharedStatus = Arc::new(statuses);

    // HTTP status API
    let app = Router::new()
        .route("/status", get(status_handler))
        .with_state(shared);

    let addr = format!("0.0.0.0:{}", args.port);
    info!(addr = %addr, "Starting status API");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        port = args.port,
        rovers = scenario.rovers.len(),
        "sim-bridge ready"
    );

    axum::serve(listener, app).await?;

    Ok(())
}

async fn status_handler(
    State(statuses): State<SharedStatus>,
) -> Json<serde_json::Value> {
    let mut rovers = Vec::new();
    for status in statuses.iter() {
        let s = status.lock().await;
        rovers.push(serde_json::json!({
            "id": s.id,
            "connected": s.connected,
            "pose": {
                "x": s.pose.x,
                "y": s.pose.y,
                "theta": s.pose.theta,
            }
        }));
    }
    Json(serde_json::json!({ "rovers": rovers }))
}
