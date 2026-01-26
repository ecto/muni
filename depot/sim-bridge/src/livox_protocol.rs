//! Construct Livox SDK2 UDP packets (inverse of driver.rs parsing).
//!
//! Point cloud: Cartesian-32 (data_type=1), 14 bytes per point, up to 96 per packet.
//! IMU: data_type=0, 24 bytes of float data.

use crate::lidar_sim::LidarPoint;

/// SDK2 header size.
const HEADER_SIZE: usize = 36;

/// Max points per packet (Livox Mid-360).
const MAX_POINTS_PER_PACKET: usize = 96;

/// Point size for Cartesian-32 format.
const POINT_SIZE: usize = 14;

/// Build a Livox SDK2 point cloud packet (Cartesian-32, data_type=1).
///
/// Returns a complete UDP packet ready to send.
pub fn build_point_packet(
    points: &[LidarPoint],
    seq: u16,
    timestamp_ns: u64,
) -> Vec<u8> {
    let dot_num = points.len().min(MAX_POINTS_PER_PACKET) as u16;
    let total_len = HEADER_SIZE + (dot_num as usize) * POINT_SIZE;

    let mut buf = Vec::with_capacity(total_len);

    // Header (36 bytes)
    buf.push(1); // version
    buf.extend_from_slice(&(total_len as u16).to_le_bytes()); // length
    buf.extend_from_slice(&0u16.to_le_bytes()); // time_interval
    buf.extend_from_slice(&dot_num.to_le_bytes()); // dot_num
    buf.extend_from_slice(&seq.to_le_bytes()); // udp_cnt
    buf.push(0); // frame_cnt
    buf.push(1); // data_type = Cartesian-32
    buf.push(0); // time_type
    buf.extend_from_slice(&[0u8; 12]); // reserved
    buf.extend_from_slice(&0u32.to_le_bytes()); // crc32 (not validated)
    buf.extend_from_slice(&timestamp_ns.to_le_bytes()); // timestamp

    // Points (14 bytes each)
    for point in points.iter().take(MAX_POINTS_PER_PACKET) {
        let x_mm = (point.x * 1000.0) as i32;
        let y_mm = (point.y * 1000.0) as i32;
        let z_mm = (point.z * 1000.0) as i32;
        buf.extend_from_slice(&x_mm.to_le_bytes());
        buf.extend_from_slice(&y_mm.to_le_bytes());
        buf.extend_from_slice(&z_mm.to_le_bytes());
        buf.push(point.intensity);
        buf.push(0); // tag
    }

    buf
}

/// Build a Livox SDK2 IMU packet (data_type=0).
///
/// IMU data: gyro_x/y/z (rad/s), accel_x/y/z (m/s²) as f32 LE.
pub fn build_imu_packet(
    gyro: [f32; 3],
    accel: [f32; 3],
    seq: u16,
    timestamp_ns: u64,
) -> Vec<u8> {
    let total_len = HEADER_SIZE + 24; // 6 * f32
    let mut buf = Vec::with_capacity(total_len);

    // Header (36 bytes)
    buf.push(1); // version
    buf.extend_from_slice(&(total_len as u16).to_le_bytes()); // length
    buf.extend_from_slice(&0u16.to_le_bytes()); // time_interval
    buf.extend_from_slice(&1u16.to_le_bytes()); // data_num = 1
    buf.extend_from_slice(&seq.to_le_bytes()); // udp_cnt
    buf.push(0); // frame_cnt
    buf.push(0); // data_type = IMU (documented as 0x05 in some specs, but driver.rs uses 0)
    buf.push(0); // time_type
    buf.extend_from_slice(&[0u8; 12]); // reserved
    buf.extend_from_slice(&0u32.to_le_bytes()); // crc32
    buf.extend_from_slice(&timestamp_ns.to_le_bytes()); // timestamp

    // IMU data (24 bytes: 6 floats)
    buf.extend_from_slice(&gyro[0].to_le_bytes());
    buf.extend_from_slice(&gyro[1].to_le_bytes());
    buf.extend_from_slice(&gyro[2].to_le_bytes());
    buf.extend_from_slice(&accel[0].to_le_bytes());
    buf.extend_from_slice(&accel[1].to_le_bytes());
    buf.extend_from_slice(&accel[2].to_le_bytes());

    buf
}

/// Split a large point set into 96-point Livox packets.
pub fn split_into_packets(
    points: &[LidarPoint],
    base_seq: u16,
    timestamp_ns: u64,
) -> Vec<Vec<u8>> {
    points
        .chunks(MAX_POINTS_PER_PACKET)
        .enumerate()
        .map(|(i, chunk)| {
            build_point_packet(chunk, base_seq.wrapping_add(i as u16), timestamp_ns)
        })
        .collect()
}
