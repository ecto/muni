# lidar

RPLidar A1 driver for the BVR rover.

## Overview

This crate provides a Rust driver for the SLAMTEC RPLidar A1 sensor. It connects via serial port (USB) and produces 360-degree laser scans for SLAM and obstacle detection.

## Features

- Non-blocking serial I/O using a dedicated thread
- Continuous 360-degree scans at ~5-10 Hz
- Standard laser scan format with range and intensity data
- Automatic packet parsing and scan assembly
- Robust error handling and recovery

## Hardware Specifications

**RPLidar A1:**
- Range: 0.2m - 12m
- Sample rate: 8000 samples/second
- Scan rate: 5-10 Hz
- Angular resolution: ~1°
- Interface: USB (serial UART, 115200 baud)

## Usage

```rust
use lidar::{Config, LidarReader, LaserScan};
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let config = Config {
        port: "/dev/ttyUSB0".into(),
        baud_rate: 115200,
    };

    let (tx, mut rx) = watch::channel(None);
    let reader = LidarReader::new(config);
    let _handle = reader.spawn(tx).expect("Failed to spawn reader");

    // Process scans
    while rx.changed().await.is_ok() {
        if let Some(scan) = &*rx.borrow() {
            println!("Received scan with {} points", scan.ranges.len());
        }
    }
}
```

## Example

Run the interactive scan viewer:

```bash
# Default port (/dev/ttyUSB0)
cargo run --example read_scans

# Custom port
cargo run --example read_scans -- /dev/ttyUSB1
```

The example displays:
- Scan statistics (point count, range, quality)
- ASCII visualization every 10 scans

## LaserScan Format

Each `LaserScan` contains:
- `timestamp`: When the scan was captured
- `angle_increment`: Radians between consecutive measurements
- `range_min`, `range_max`: Valid range limits (0.2m - 12m)
- `ranges`: 360-element array of distances in meters (index = degrees)
- `intensities`: 360-element array of signal quality (0-63)

## Protocol Details

The driver implements the RPLidar A1 binary protocol:

**Commands:**
- `0xA5 0x20`: Start scan
- `0xA5 0x25`: Stop scan
- `0xA5 0x40`: Reset device

**Packet Format (5 bytes):**
```
Byte 0: Quality (6 bits) + Start flag (1 bit)
Byte 1-2: Angle (1/64 degree increments)
Byte 3-4: Distance (1/4 mm increments)
```

## Integration

Add to your `Cargo.toml`:

```toml
[dependencies]
lidar = { path = "../lidar" }
tokio = { version = "1", features = ["sync"] }
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

- `gps` - GPS receiver driver (similar architecture)
- `localization` - Sensor fusion for pose estimation
- `recording` - Rerun integration for data capture
