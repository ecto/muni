//! Livox Mid360 UDP protocol implementation.
//!
//! Protocol reference: https://livox-wiki-en.readthedocs.io/en/latest/tutorials/new_product/mid360/livox_eth_protocol_mid360.html

use crate::{Config, ImuData, LidarError, Point3D, PointCloud};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tracing::{debug, error, info, trace, warn};

/// Point cloud packet header size (bytes)
const POINT_HEADER_SIZE: usize = 24;

/// Points per packet (fixed for Mid360)
const POINTS_PER_PACKET: usize = 96;

/// Point data type 1: Cartesian 32-bit (14 bytes per point)
/// x, y, z: i32 (mm), reflectivity: u8, tag: u8
const POINT_TYPE_CARTESIAN_32: u8 = 1;

/// Point data type 2: Cartesian 16-bit (8 bytes per point)
const POINT_TYPE_CARTESIAN_16: u8 = 2;

/// Point data type 3: Spherical (10 bytes per point)
const POINT_TYPE_SPHERICAL: u8 = 3;

/// IMU packet size (fixed)
const IMU_PACKET_SIZE: usize = 48;

/// Frame accumulator for building complete point clouds from packets
struct FrameAccumulator {
    points: Vec<Point3D>,
    frame_id: AtomicU32,
    last_frame_counter: u8,
    timestamp_ns: u64,
    frame_start: Instant,
}

impl FrameAccumulator {
    fn new() -> Self {
        Self {
            points: Vec::with_capacity(POINTS_PER_PACKET * 100), // ~10k points typical
            frame_id: AtomicU32::new(0),
            last_frame_counter: 0,
            timestamp_ns: 0,
            frame_start: Instant::now(),
        }
    }

    /// Add points from a packet. Returns Some(PointCloud) when frame is complete.
    fn add_packet(&mut self, frame_counter: u8, timestamp_ns: u64, points: Vec<Point3D>) -> Option<PointCloud> {
        // Detect new frame (frame_counter wraps or changes significantly)
        let frame_boundary = if self.points.is_empty() {
            false
        } else {
            // New frame starts when frame_counter wraps or we've accumulated enough
            frame_counter < self.last_frame_counter || self.points.len() > 20000
        };

        if frame_boundary && !self.points.is_empty() {
            // Complete current frame
            let cloud = PointCloud {
                timestamp: self.frame_start,
                timestamp_ns: self.timestamp_ns,
                frame_id: self.frame_id.fetch_add(1, Ordering::Relaxed),
                points: std::mem::take(&mut self.points),
            };
            self.points.reserve(POINTS_PER_PACKET * 100);
            self.frame_start = Instant::now();
            self.timestamp_ns = timestamp_ns;
            self.last_frame_counter = frame_counter;
            self.points.extend(points);
            Some(cloud)
        } else {
            if self.points.is_empty() {
                self.frame_start = Instant::now();
                self.timestamp_ns = timestamp_ns;
            }
            self.last_frame_counter = frame_counter;
            self.points.extend(points);
            None
        }
    }
}

/// Run the LiDAR reader (point cloud only).
pub async fn run_reader(
    config: Config,
    tx: watch::Sender<Option<PointCloud>>,
) -> Result<(), LidarError> {
    let dummy_tx = watch::channel(None).0;
    run_reader_with_imu(config, tx, dummy_tx).await
}

/// Run the LiDAR reader with IMU support.
pub async fn run_reader_with_imu(
    config: Config,
    point_tx: watch::Sender<Option<PointCloud>>,
    imu_tx: watch::Sender<Option<ImuData>>,
) -> Result<(), LidarError> {
    info!(
        lidar_ip = %config.lidar_ip,
        point_port = config.point_cloud_port,
        "Starting Livox Mid360 reader"
    );

    // Bind UDP socket for point cloud data
    let point_addr: SocketAddr = (config.host_ip, config.point_cloud_port).into();
    let point_socket = UdpSocket::bind(point_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind point cloud socket: {}", e)))?;

    info!(addr = %point_addr, "Listening for point cloud data");

    // Optionally bind IMU socket
    let imu_socket = if config.imu_port > 0 {
        let imu_addr: SocketAddr = (config.host_ip, config.imu_port).into();
        match UdpSocket::bind(imu_addr).await {
            Ok(s) => {
                info!(addr = %imu_addr, "Listening for IMU data");
                Some(s)
            }
            Err(e) => {
                warn!("Failed to bind IMU socket: {} - continuing without IMU", e);
                None
            }
        }
    } else {
        None
    };

    let mut frame_acc = FrameAccumulator::new();
    let mut point_buf = vec![0u8; 2048]; // Max packet size
    let mut imu_buf = vec![0u8; IMU_PACKET_SIZE + 32];
    let mut packet_count: u64 = 0;

    loop {
        tokio::select! {
            result = point_socket.recv_from(&mut point_buf) => {
                match result {
                    Ok((len, _src)) => {
                        packet_count += 1;
                        if packet_count % 1000 == 0 {
                            debug!(packets = packet_count, "Point cloud packets received");
                        }

                        match parse_point_packet(&point_buf[..len]) {
                            Ok((frame_counter, timestamp_ns, points)) => {
                                trace!(
                                    frame_counter,
                                    points = points.len(),
                                    "Parsed point packet"
                                );

                                if let Some(cloud) = frame_acc.add_packet(frame_counter, timestamp_ns, points) {
                                    debug!(
                                        frame = cloud.frame_id,
                                        points = cloud.points.len(),
                                        "Completed point cloud frame"
                                    );

                                    if point_tx.send(Some(cloud)).is_err() {
                                        info!("Point cloud receiver dropped, stopping");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                trace!(?e, len, "Failed to parse point packet");
                            }
                        }
                    }
                    Err(e) => {
                        error!(?e, "Point cloud socket error");
                        break;
                    }
                }
            }

            result = async {
                if let Some(ref socket) = imu_socket {
                    socket.recv_from(&mut imu_buf).await
                } else {
                    // Never resolves if no IMU socket
                    std::future::pending().await
                }
            } => {
                if let Ok((len, _src)) = result {
                    match parse_imu_packet(&imu_buf[..len]) {
                        Ok(imu) => {
                            let _ = imu_tx.send(Some(imu));
                        }
                        Err(e) => {
                            trace!(?e, "Failed to parse IMU packet");
                        }
                    }
                }
            }
        }
    }

    info!("Livox Mid360 reader stopped");
    Ok(())
}

/// Parse a point cloud data packet.
///
/// Returns (frame_counter, timestamp_ns, points).
fn parse_point_packet(data: &[u8]) -> Result<(u8, u64, Vec<Point3D>), LidarError> {
    if data.len() < POINT_HEADER_SIZE {
        return Err(LidarError::Parse(format!(
            "Packet too small: {} bytes",
            data.len()
        )));
    }

    // Parse header
    // Bytes 0: version
    // Bytes 1-2: length (little-endian)
    // Bytes 3-4: time_interval (0.1us units)
    // Bytes 5-6: dot_num (points in packet)
    // Bytes 7-8: udp_cnt (sequence number)
    // Byte 9: frame_cnt
    // Byte 10: data_type
    // Byte 11: timestamp_type
    // Bytes 12-23: reserved
    // Bytes 24+: point data (or with some versions, timestamp first)

    let _version = data[0];
    let length = u16::from_le_bytes([data[1], data[2]]) as usize;
    let dot_num = u16::from_le_bytes([data[5], data[6]]) as usize;
    let frame_cnt = data[9];
    let data_type = data[10];

    // Validate length
    if data.len() < length {
        return Err(LidarError::Parse(format!(
            "Incomplete packet: {} < {}",
            data.len(),
            length
        )));
    }

    // Timestamp is 8 bytes after the fixed header portion
    // In the Livox protocol, timestamp comes after initial header
    let timestamp_offset = 16; // Approximate - may need adjustment
    let timestamp_ns = if data.len() >= timestamp_offset + 8 {
        u64::from_le_bytes(data[timestamp_offset..timestamp_offset + 8].try_into().unwrap_or([0; 8]))
    } else {
        0
    };

    // Point data starts after header (exact offset depends on protocol version)
    let point_data_offset = POINT_HEADER_SIZE;
    let point_data = &data[point_data_offset..];

    let points = match data_type {
        POINT_TYPE_CARTESIAN_32 => parse_cartesian_32(point_data, dot_num)?,
        POINT_TYPE_CARTESIAN_16 => parse_cartesian_16(point_data, dot_num)?,
        POINT_TYPE_SPHERICAL => parse_spherical(point_data, dot_num)?,
        _ => {
            return Err(LidarError::Parse(format!(
                "Unknown data type: {}",
                data_type
            )));
        }
    };

    Ok((frame_cnt, timestamp_ns, points))
}

/// Parse Cartesian 32-bit point data (type 1).
/// Each point: x(i32), y(i32), z(i32), reflectivity(u8), tag(u8) = 14 bytes
fn parse_cartesian_32(data: &[u8], count: usize) -> Result<Vec<Point3D>, LidarError> {
    const POINT_SIZE: usize = 14;
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * POINT_SIZE;
        if offset + POINT_SIZE > data.len() {
            break;
        }

        let x_mm = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let y_mm = i32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        let z_mm = i32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
        let reflectivity = data[offset + 12];
        let tag = data[offset + 13];

        // Convert mm to meters
        points.push(Point3D {
            x: x_mm as f32 / 1000.0,
            y: y_mm as f32 / 1000.0,
            z: z_mm as f32 / 1000.0,
            reflectivity,
            tag,
        });
    }

    Ok(points)
}

/// Parse Cartesian 16-bit point data (type 2).
/// Each point: x(i16), y(i16), z(i16), reflectivity(u8), tag(u8) = 8 bytes
/// Coordinates are in 10mm units.
fn parse_cartesian_16(data: &[u8], count: usize) -> Result<Vec<Point3D>, LidarError> {
    const POINT_SIZE: usize = 8;
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * POINT_SIZE;
        if offset + POINT_SIZE > data.len() {
            break;
        }

        let x_cm = i16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        let y_cm = i16::from_le_bytes(data[offset + 2..offset + 4].try_into().unwrap());
        let z_cm = i16::from_le_bytes(data[offset + 4..offset + 6].try_into().unwrap());
        let reflectivity = data[offset + 6];
        let tag = data[offset + 7];

        // Convert 10mm (cm) to meters
        points.push(Point3D {
            x: x_cm as f32 / 100.0,
            y: y_cm as f32 / 100.0,
            z: z_cm as f32 / 100.0,
            reflectivity,
            tag,
        });
    }

    Ok(points)
}

/// Parse spherical point data (type 3).
/// Each point: depth(u32), theta(u16), phi(u16), reflectivity(u8), tag(u8) = 10 bytes
fn parse_spherical(data: &[u8], count: usize) -> Result<Vec<Point3D>, LidarError> {
    const POINT_SIZE: usize = 10;
    let mut points = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * POINT_SIZE;
        if offset + POINT_SIZE > data.len() {
            break;
        }

        let depth_mm = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let theta_raw = u16::from_le_bytes(data[offset + 4..offset + 6].try_into().unwrap());
        let phi_raw = u16::from_le_bytes(data[offset + 6..offset + 8].try_into().unwrap());
        let reflectivity = data[offset + 8];
        let tag = data[offset + 9];

        // Convert spherical to Cartesian
        // theta and phi are in 0.01 degree units
        let depth = depth_mm as f32 / 1000.0; // meters
        let theta = (theta_raw as f32 * 0.01).to_radians();
        let phi = (phi_raw as f32 * 0.01).to_radians();

        let x = depth * phi.cos() * theta.cos();
        let y = depth * phi.cos() * theta.sin();
        let z = depth * phi.sin();

        points.push(Point3D {
            x,
            y,
            z,
            reflectivity,
            tag,
        });
    }

    Ok(points)
}

/// Parse an IMU data packet.
fn parse_imu_packet(data: &[u8]) -> Result<ImuData, LidarError> {
    // IMU packet format (simplified):
    // Bytes 0-7: timestamp (ns)
    // Bytes 8-11: gyro_x (float)
    // Bytes 12-15: gyro_y (float)
    // Bytes 16-19: gyro_z (float)
    // Bytes 20-23: accel_x (float)
    // Bytes 24-27: accel_y (float)
    // Bytes 28-31: accel_z (float)

    if data.len() < 32 {
        return Err(LidarError::Parse("IMU packet too small".into()));
    }

    let timestamp_ns = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let gyro_x = f32::from_le_bytes(data[8..12].try_into().unwrap());
    let gyro_y = f32::from_le_bytes(data[12..16].try_into().unwrap());
    let gyro_z = f32::from_le_bytes(data[16..20].try_into().unwrap());
    let accel_x = f32::from_le_bytes(data[20..24].try_into().unwrap());
    let accel_y = f32::from_le_bytes(data[24..28].try_into().unwrap());
    let accel_z = f32::from_le_bytes(data[28..32].try_into().unwrap());

    Ok(ImuData {
        timestamp_ns,
        gyro_x,
        gyro_y,
        gyro_z,
        accel_x,
        accel_y,
        accel_z,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cartesian_32() {
        // Create a test packet with one point at (1m, 2m, 3m)
        let mut data = Vec::new();
        data.extend_from_slice(&1000i32.to_le_bytes()); // x = 1000mm = 1m
        data.extend_from_slice(&2000i32.to_le_bytes()); // y = 2000mm = 2m
        data.extend_from_slice(&3000i32.to_le_bytes()); // z = 3000mm = 3m
        data.push(128); // reflectivity
        data.push(0); // tag

        let points = parse_cartesian_32(&data, 1).unwrap();
        assert_eq!(points.len(), 1);
        assert!((points[0].x - 1.0).abs() < 0.001);
        assert!((points[0].y - 2.0).abs() < 0.001);
        assert!((points[0].z - 3.0).abs() < 0.001);
        assert_eq!(points[0].reflectivity, 128);
    }

    #[test]
    fn test_parse_cartesian_16() {
        // Create a test packet with one point at (1m, 2m, 0.5m)
        let mut data = Vec::new();
        data.extend_from_slice(&100i16.to_le_bytes()); // x = 100 * 10mm = 1m
        data.extend_from_slice(&200i16.to_le_bytes()); // y = 200 * 10mm = 2m
        data.extend_from_slice(&50i16.to_le_bytes()); // z = 50 * 10mm = 0.5m
        data.push(200); // reflectivity
        data.push(1); // tag

        let points = parse_cartesian_16(&data, 1).unwrap();
        assert_eq!(points.len(), 1);
        assert!((points[0].x - 1.0).abs() < 0.01);
        assert!((points[0].y - 2.0).abs() < 0.01);
        assert!((points[0].z - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_imu() {
        let mut data = vec![0u8; 32];
        // Timestamp = 1000000ns
        data[0..8].copy_from_slice(&1000000u64.to_le_bytes());
        // Gyro X = 0.1 rad/s
        data[8..12].copy_from_slice(&0.1f32.to_le_bytes());
        // Accel X = 9.8 m/s²
        data[20..24].copy_from_slice(&9.8f32.to_le_bytes());

        let imu = parse_imu_packet(&data).unwrap();
        assert_eq!(imu.timestamp_ns, 1000000);
        assert!((imu.gyro_x - 0.1).abs() < 0.001);
        assert!((imu.accel_x - 9.8).abs() < 0.001);
    }
}
