//! RPLidar A1 serial protocol implementation.

use crate::{Config, LaserScan, LidarError};
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_serial::SerialPort;
use tracing::{debug, error, info, trace};

/// RPLidar A1 commands
const CMD_SCAN: [u8; 2] = [0xA5, 0x20];
const CMD_STOP: [u8; 2] = [0xA5, 0x25];
const CMD_RESET: [u8; 2] = [0xA5, 0x40];

/// Packet size (5 bytes per measurement point)
const PACKET_SIZE: usize = 5;

/// A single measurement point from the LiDAR
#[derive(Debug, Clone, Copy)]
struct MeasurementPoint {
    /// Start of new scan flag
    start: bool,
    /// Quality (0-63)
    quality: u8,
    /// Angle in degrees (0-360)
    angle: f32,
    /// Distance in meters
    distance: f32,
}

/// Internal reader loop that runs in a blocking thread.
pub(crate) fn run_reader(
    config: Config,
    tx: watch::Sender<Option<LaserScan>>,
) -> Result<(), LidarError> {
    info!(port = %config.port, baud = config.baud_rate, "Opening LiDAR serial port");

    // Open serial port
    let mut port = tokio_serial::new(&config.port, config.baud_rate)
        .timeout(Duration::from_secs(2))
        .open_native()
        .map_err(|e| LidarError::Serial(e.to_string()))?;

    // Send reset command to clear any previous state
    port.write_all(&CMD_RESET)
        .map_err(|e| LidarError::Serial(e.to_string()))?;
    std::thread::sleep(Duration::from_millis(100));

    // Flush any pending data
    let _ = port.clear(tokio_serial::ClearBuffer::All);

    // Send start scan command
    port.write_all(&CMD_SCAN)
        .map_err(|e| LidarError::Serial(e.to_string()))?;
    port.flush()
        .map_err(|e| LidarError::Serial(e.to_string()))?;

    info!("LiDAR reader started");

    let mut packet_buf = [0u8; PACKET_SIZE];
    let mut scan_points = Vec::with_capacity(360);
    let mut scan_start_time = Instant::now();

    loop {
        // Read one packet (5 bytes)
        match port.read_exact(&mut packet_buf) {
            Ok(()) => {
                trace!(bytes = ?packet_buf, "Received packet");

                // Parse the packet
                match parse_packet(&packet_buf) {
                    Ok(point) => {
                        // Check if this is the start of a new scan
                        if point.start {
                            // If we have accumulated points, send the completed scan
                            if !scan_points.is_empty() {
                                let scan = build_scan(scan_points, scan_start_time);
                                debug!(
                                    points = scan.ranges.len(),
                                    min_range = scan.ranges.iter().copied().fold(f32::INFINITY, f32::min),
                                    max_range = scan.ranges.iter().copied().fold(0.0f32, f32::max),
                                    "Completed scan"
                                );

                                if tx.send(Some(scan)).is_err() {
                                    // Receiver dropped
                                    info!("LiDAR receiver dropped, stopping");
                                    break;
                                }

                                // Start a new scan
                                scan_points = Vec::with_capacity(360);
                                scan_start_time = Instant::now();
                            }
                        }

                        // Add point to current scan (only if distance is valid)
                        if point.distance > 0.0 && point.quality > 0 {
                            scan_points.push(point);
                        }
                    }
                    Err(e) => {
                        trace!(?e, "Failed to parse packet");
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is OK, just continue
                trace!("Read timeout, continuing");
                continue;
            }
            Err(e) => {
                error!(?e, "LiDAR read error");
                break;
            }
        }
    }

    // Send stop command on exit
    let _ = port.write_all(&CMD_STOP);

    info!("LiDAR reader stopped");
    Ok(())
}

/// Parse a 5-byte packet into a measurement point.
///
/// Packet format:
/// Byte 0: Quality (bits 7-2), Start flag (bit 0)
/// Byte 1: Angle low byte
/// Byte 2: Angle high byte (angle in 1/64 degree increments)
/// Byte 3: Distance low byte
/// Byte 4: Distance high byte (distance in 1/4 mm increments)
fn parse_packet(packet: &[u8; PACKET_SIZE]) -> Result<MeasurementPoint, LidarError> {
    // Parse byte 0: start flag and quality
    let start = (packet[0] & 0x01) != 0;
    let quality = (packet[0] >> 2) & 0x3F;

    // Parse angle (bytes 1-2): 1/64 degree increments
    let angle_raw = (packet[2] as u16) << 8 | packet[1] as u16;
    let angle_deg = (angle_raw as f32) / 64.0;

    // Parse distance (bytes 3-4): 1/4 mm increments
    let distance_raw = (packet[4] as u16) << 8 | packet[3] as u16;
    let distance_m = (distance_raw as f32) / 4000.0; // Convert 1/4 mm to meters

    Ok(MeasurementPoint {
        start,
        quality,
        angle: angle_deg,
        distance: distance_m,
    })
}

/// Build a LaserScan from accumulated measurement points.
fn build_scan(points: Vec<MeasurementPoint>, timestamp: Instant) -> LaserScan {
    if points.is_empty() {
        return LaserScan::default();
    }

    // Sort points by angle to handle out-of-order measurements
    let mut sorted_points = points;
    sorted_points.sort_by(|a, b| a.angle.partial_cmp(&b.angle).unwrap());

    // Determine angular resolution (typical: ~1 degree for RPLidar A1)
    let angle_increment = if sorted_points.len() > 1 {
        // Calculate average angular spacing
        let total_angle = sorted_points.last().unwrap().angle - sorted_points[0].angle;
        (total_angle / (sorted_points.len() - 1) as f32).to_radians()
    } else {
        1.0f32.to_radians()
    };

    // Create dense arrays (one entry per degree, 0-359)
    const ARRAY_SIZE: usize = 360;
    let mut ranges = vec![0.0f32; ARRAY_SIZE];
    let mut intensities = vec![0u8; ARRAY_SIZE];

    // Fill arrays with measurements
    for point in sorted_points {
        let index = (point.angle.round() as usize) % ARRAY_SIZE;
        ranges[index] = point.distance;
        intensities[index] = point.quality;
    }

    LaserScan {
        timestamp,
        angle_increment,
        range_min: 0.2,  // RPLidar A1 spec
        range_max: 12.0, // RPLidar A1 spec
        ranges,
        intensities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_packet() {
        // Example packet: start=true, quality=15, angle=90°, distance=1m
        // Byte 0: start(1) + quality(15<<2) = 0x01 | 0x3C = 0x3D
        // Angle: 90° * 64 = 5760 = 0x1680
        // Distance: 1m * 4000 = 4000 = 0x0FA0
        let packet = [
            0x3D,      // Start flag + quality
            0x80, 0x16, // Angle low, high
            0xA0, 0x0F, // Distance low, high
        ];

        let point = parse_packet(&packet).unwrap();
        assert!(point.start);
        assert_eq!(point.quality, 15);
        assert!((point.angle - 90.0).abs() < 0.1);
        assert!((point.distance - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_packet_no_start() {
        // No start flag: bit 0 clear
        // Quality=10, angle=45°, distance=2.5m
        let packet = [
            0x28,      // Quality 10 << 2 = 0x28
            0x40, 0x0B, // 45° * 64 = 2880 = 0x0B40
            0x10, 0x27, // 2.5m * 4000 = 10000 = 0x2710
        ];

        let point = parse_packet(&packet).unwrap();
        assert!(!point.start);
        assert_eq!(point.quality, 10);
        assert!((point.angle - 45.0).abs() < 0.1);
        assert!((point.distance - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_build_scan_empty() {
        let scan = build_scan(Vec::new(), Instant::now());
        assert_eq!(scan.ranges.len(), 0);
        assert_eq!(scan.intensities.len(), 0);
    }

    #[test]
    fn test_build_scan_single_point() {
        let points = vec![MeasurementPoint {
            start: true,
            quality: 20,
            angle: 180.0,
            distance: 5.0,
        }];

        let scan = build_scan(points, Instant::now());
        assert_eq!(scan.ranges.len(), 360);
        assert_eq!(scan.intensities.len(), 360);
        assert!((scan.ranges[180] - 5.0).abs() < 0.01);
        assert_eq!(scan.intensities[180], 20);
    }

    #[test]
    fn test_build_scan_multiple_points() {
        let points = vec![
            MeasurementPoint {
                start: true,
                quality: 10,
                angle: 0.0,
                distance: 1.0,
            },
            MeasurementPoint {
                start: false,
                quality: 15,
                angle: 90.0,
                distance: 2.0,
            },
            MeasurementPoint {
                start: false,
                quality: 20,
                angle: 180.0,
                distance: 3.0,
            },
            MeasurementPoint {
                start: false,
                quality: 25,
                angle: 270.0,
                distance: 4.0,
            },
        ];

        let scan = build_scan(points, Instant::now());
        assert_eq!(scan.ranges.len(), 360);
        assert!((scan.ranges[0] - 1.0).abs() < 0.01);
        assert!((scan.ranges[90] - 2.0).abs() < 0.01);
        assert!((scan.ranges[180] - 3.0).abs() < 0.01);
        assert!((scan.ranges[270] - 4.0).abs() < 0.01);
    }
}
