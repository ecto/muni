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

### Network Setup (BVR Rover)

The Mid360 uses the HAP (High-speed API Protocol) which broadcasts its configuration on startup. The LiDAR sends data to a pre-configured host IP that may differ from the rover's primary interface.

**Discovering the LiDAR:**
```bash
# Capture HAP broadcast packets to find the LiDAR
sudo tcpdump -i eth0 udp port 10001 -c 5

# Look for packets from 192.168.1.1xx containing DevType:Mid-360
# The packet also shows what host IP the LiDAR expects to send data to
```

**Adding a secondary IP (if LiDAR expects a different host):**
```bash
# Temporary (lost on reboot)
sudo ip addr add 192.168.1.5/24 dev eth0

# Permanent (NetworkManager)
sudo nmcli connection modify eth0 +ipv4.addresses '192.168.1.5/24'
sudo nmcli connection up eth0
```

**Verify data is flowing:**
```bash
# Should see UDP packets on port 56301 (point cloud) and 56401 (IMU)
sudo tcpdump -i eth0 host 192.168.1.177 -c 20
```

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
lidar_ip = "192.168.1.177"      # LiDAR IP (find via HAP broadcast)
host_ip = "192.168.1.5"         # Host IP the LiDAR sends data to
point_cloud_port = 56301
imu_port = 56401                # Set to 0 to disable IMU
mounting_pitch_deg = 30.0       # LiDAR tilt angle (positive = tilted down)
```

### Mounting Angle

The `mounting_pitch_deg` parameter compensates for the physical mounting angle of the LiDAR. When the LiDAR is tilted forward (pitched down), set this to a positive value. The driver applies a rotation to transform points from the LiDAR frame to the rover body frame.

For BVR rovers, the Mid360 is typically mounted at 30° pitch to provide better ground coverage for obstacle detection.

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
