//! Example: Read and display laser scans from RPLidar A1.
//!
//! Usage:
//!   cargo run --example read_scans
//!   cargo run --example read_scans -- /dev/ttyUSB1

use lidar::{Config, LidarReader};
use tokio::sync::watch;
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Parse port from command line arguments
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/ttyUSB0".to_string());

    let config = Config {
        port,
        baud_rate: 115200,
    };

    info!("Starting LiDAR reader on {}", config.port);

    // Create watch channel for scans
    let (tx, mut rx) = watch::channel(None);

    // Spawn reader thread
    let reader = LidarReader::new(config);
    let _handle = reader.spawn(tx).expect("Failed to spawn LiDAR reader");

    info!("Waiting for scans... (Ctrl+C to stop)");

    // Process scans as they arrive
    let mut scan_count = 0u64;
    loop {
        if rx.changed().await.is_ok() {
            if let Some(scan) = &*rx.borrow() {
                scan_count += 1;

                // Count valid measurements
                let valid_points = scan
                    .ranges
                    .iter()
                    .filter(|&&r| r > scan.range_min && r < scan.range_max)
                    .count();

                // Find min/max ranges
                let min_range = scan
                    .ranges
                    .iter()
                    .filter(|&&r| r > 0.0)
                    .copied()
                    .fold(f32::INFINITY, f32::min);
                let max_range = scan.ranges.iter().copied().fold(0.0f32, f32::max);

                // Average intensity
                let avg_intensity = if valid_points > 0 {
                    scan.intensities
                        .iter()
                        .filter(|&&i| i > 0)
                        .map(|&i| i as u32)
                        .sum::<u32>() as f32
                        / valid_points as f32
                } else {
                    0.0
                };

                info!(
                    scan = scan_count,
                    points = valid_points,
                    angle_inc = format!("{:.2}°", scan.angle_increment.to_degrees()),
                    range = format!("[{:.2}m - {:.2}m]", min_range, max_range),
                    avg_quality = format!("{:.0}", avg_intensity),
                    "Scan received"
                );

                // Display a simple ASCII visualization every 10 scans
                if scan_count % 10 == 0 {
                    print_scan_visualization(&scan);
                }
            }
        }
    }
}

/// Print a simple ASCII visualization of the scan.
fn print_scan_visualization(scan: &lidar::LaserScan) {
    const WIDTH: usize = 80;
    const HEIGHT: usize = 24;
    const MAX_DISPLAY_RANGE: f32 = 8.0; // meters

    let mut grid = vec![vec![' '; WIDTH]; HEIGHT];

    // Mark center (rover position)
    let center_x = WIDTH / 2;
    let center_y = HEIGHT / 2;
    grid[center_y][center_x] = '+';

    // Plot scan points
    for (i, &distance) in scan.ranges.iter().enumerate() {
        if distance > scan.range_min && distance < MAX_DISPLAY_RANGE {
            let angle = (i as f32).to_radians();

            // Convert polar to cartesian (with Y inverted for display)
            let x = distance * angle.cos();
            let y = distance * angle.sin();

            // Scale to grid
            let grid_x = center_x as isize + (x / MAX_DISPLAY_RANGE * (WIDTH / 2) as f32) as isize;
            let grid_y = center_y as isize - (y / MAX_DISPLAY_RANGE * (HEIGHT / 2) as f32) as isize;

            if grid_x >= 0
                && grid_x < WIDTH as isize
                && grid_y >= 0
                && grid_y < HEIGHT as isize
            {
                grid[grid_y as usize][grid_x as usize] = '*';
            }
        }
    }

    // Print grid
    println!("\n{}", "=".repeat(WIDTH));
    for row in &grid {
        let line: String = row.iter().collect();
        println!("{}", line);
    }
    println!("{}", "=".repeat(WIDTH));
    println!("Rover at '+', obstacles at '*', range: 0-{}m\n", MAX_DISPLAY_RANGE);
}
