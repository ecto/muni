# lidar

Livox Mid360 LiDAR driver for the BVR rover.

## Overview

This crate provides a Rust driver for the Livox Mid360 3D LiDAR sensor. It receives point cloud data over UDP using the Livox SDK2 protocol and produces 3D point clouds for SLAM, obstacle detection, and mapping.

## Features

- Async UDP receiver using Tokio
- 3D point cloud frames (~10k points per frame at 10Hz)
- Multiple point data formats (Cartesian 32-bit, 16-bit, Spherical)
- Optional IMU data streaming
- Frame assembly from multiple UDP packets

## Hardware Specifications

**Livox Mid360:**
- Range: 0.1m - 70m
- FOV: 360° × 59° (non-repetitive scanning)
- Point rate: 200,000 points/second
- Scan rate: ~10 Hz (full coverage)
- Interface: Ethernet (100BASE-TX)
- Power: PoE or 9-27V DC

## Network Configuration

The Mid360 uses fixed UDP ports:

| Data Type | LiDAR Port | Host Port |
|-----------|-----------|-----------|
| Point Cloud | 56300 | 56301 |
| IMU Data | 56400 | 56401 |
| Commands | 56100 | 56101 |

Default LiDAR IP: `192.168.1.1xx` (last two digits from serial number)

## Usage

```rust
use lidar::{Config, LidarReader, PointCloud};
use std::net::Ipv4Addr;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let config = Config {
        lidar_ip: Ipv4Addr::new(192, 168, 1, 100),
        point_cloud_port: 56301,
        imu_port: 56401,
        ..Default::default()
    };

    let (tx, mut rx) = watch::channel(None);
    let reader = LidarReader::new(config);
    let _handle = reader.spawn(tx);

    // Process point clouds
    while rx.changed().await.is_ok() {
        if let Some(cloud) = &*rx.borrow() {
            println!("Frame {}: {} points", cloud.frame_id, cloud.points.len());
            for p in &cloud.points {
                println!("  ({:.2}, {:.2}, {:.2})", p.x, p.y, p.z);
            }
        }
    }
}
```

## Example

Run the interactive point cloud viewer:

```bash
# Default IP (192.168.1.100)
cargo run --example read_points

# Custom IP
cargo run --example read_points -- --ip 192.168.1.101
```

## PointCloud Format

Each `PointCloud` contains:
- `timestamp`: Local time when frame was received
- `timestamp_ns`: LiDAR nanosecond timestamp (GPS synced if available)
- `frame_id`: Monotonic frame counter
- `points`: Vector of `Point3D`:
  - `x`, `y`, `z`: Coordinates in meters (LiDAR frame)
  - `reflectivity`: Surface reflectivity (0-255)
  - `tag`: Classification flags

## Coordinate System

The Mid360 uses a right-handed coordinate system:
- X: Forward (along the connector direction)
- Y: Left
- Z: Up

## Configuration

In `bvr.toml`:

```toml
[lidar]
enabled = true
lidar_ip = "192.168.1.100"
point_cloud_port = 56301
imu_port = 56401  # Set to 0 to disable IMU
```

## Integration

Add to your `Cargo.toml`:

```toml
[dependencies]
lidar = { path = "../lidar" }
tokio = { version = "1", features = ["net", "sync", "rt"] }
```

Or use workspace dependencies:

```toml
[dependencies]
lidar.workspace = true
```

## Testing

```bash
# Run unit tests
cargo test -p lidar

# Build all targets
cargo check -p lidar --all-targets
```

## Related Crates

- `gps` - GPS receiver driver (similar watch channel pattern)
- `localization` - Sensor fusion for pose estimation
- `recording` - Rerun integration for data capture
- `slam` - SLAM processing using point clouds

## Protocol Reference

Based on [Livox Mid360 Ethernet Protocol](https://livox-wiki-en.readthedocs.io/en/latest/tutorials/new_product/mid360/livox_eth_protocol_mid360.html).
