//! Example: Read point clouds from a Livox Mid360.
//!
//! Usage:
//!   cargo run --example read_points
//!   cargo run --example read_points -- --ip 192.168.1.100

use lidar::{Config, LidarReader, PointCloud};
use std::net::Ipv4Addr;
use tokio::sync::watch;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Parse optional IP argument
    let args: Vec<String> = std::env::args().collect();
    let lidar_ip = if args.len() > 2 && args[1] == "--ip" {
        args[2].parse().expect("Invalid IP address")
    } else {
        Ipv4Addr::new(192, 168, 1, 100)
    };

    let config = Config {
        lidar_ip,
        ..Default::default()
    };

    info!(ip = %config.lidar_ip, port = config.point_cloud_port, "Starting LiDAR reader");
    info!("Ensure the Mid360 is configured to send point clouds to this host");

    let (tx, mut rx) = watch::channel(None::<PointCloud>);
    let reader = LidarReader::new(config);
    let _handle = reader.spawn(tx);

    let mut frame_count = 0u64;
    let mut total_points = 0u64;

    println!("\nWaiting for point clouds from Livox Mid360...\n");
    println!("Frame | Points | Min X | Max X | Min Z | Max Z");
    println!("------|--------|-------|-------|-------|------");

    while rx.changed().await.is_ok() {
        if let Some(cloud) = &*rx.borrow() {
            frame_count += 1;
            total_points += cloud.points.len() as u64;

            // Calculate bounds
            let (min_x, max_x, min_z, max_z) = if cloud.points.is_empty() {
                (0.0, 0.0, 0.0, 0.0)
            } else {
                let mut min_x = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut min_z = f32::INFINITY;
                let mut max_z = f32::NEG_INFINITY;

                for p in &cloud.points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_z = min_z.min(p.z);
                    max_z = max_z.max(p.z);
                }

                (min_x, max_x, min_z, max_z)
            };

            println!(
                "{:5} | {:6} | {:5.1} | {:5.1} | {:5.1} | {:5.1}",
                frame_count,
                cloud.points.len(),
                min_x,
                max_x,
                min_z,
                max_z
            );

            // Print detailed stats every 10 frames
            if frame_count % 10 == 0 {
                println!("\n--- Summary: {} frames, {} total points ---\n", frame_count, total_points);
            }
        }
    }
}
