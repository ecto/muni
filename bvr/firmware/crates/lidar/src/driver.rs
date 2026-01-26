//! Livox Mid360 UDP protocol implementation.
//!
//! Protocol reference: https://livox-wiki-en.readthedocs.io/en/latest/tutorials/new_product/mid360/livox_eth_protocol_mid360.html

use crate::{Config, ImuData, LidarCommand, LidarError, Point3D, PointCloud};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, trace, warn};

/// Transform for rotating points from LiDAR frame to rover body frame.
/// Precomputed sin/cos for efficiency.
struct MountingTransform {
    cos_pitch: f32,
    sin_pitch: f32,
}

impl MountingTransform {
    fn new(pitch_deg: f32) -> Self {
        let pitch_rad = pitch_deg.to_radians();
        Self {
            cos_pitch: pitch_rad.cos(),
            sin_pitch: pitch_rad.sin(),
        }
    }

    /// Apply pitch rotation (around Y axis) to a point.
    /// Transforms from LiDAR frame to rover body frame.
    #[inline]
    fn apply(&self, point: &mut Point3D) {
        // Rotation matrix for pitch (around Y axis):
        // [cos(θ)   0   sin(θ)]   [x]
        // [  0      1     0   ] * [y]
        // [-sin(θ)  0   cos(θ)]   [z]
        let x = point.x * self.cos_pitch + point.z * self.sin_pitch;
        let z = -point.x * self.sin_pitch + point.z * self.cos_pitch;
        point.x = x;
        point.z = z;
    }

    /// Apply transform to all points in a cloud.
    fn apply_to_cloud(&self, cloud: &mut PointCloud) {
        for point in &mut cloud.points {
            self.apply(point);
        }
    }
}

/// Livox Mid-360 command port (unicast control, NOT the broadcast discovery port)
const LIVOX_CMD_PORT: u16 = 56100;

/// SDK2 command IDs (u16)
const CMD_ID_WORK_MODE: u16 = 0x0100; // Parameter configuration (set work mode)

/// Work target mode values (key 0x001A)
const WORK_MODE_SAMPLING: u8 = 0x01;
const WORK_MODE_IDLE: u8 = 0x02;

/// Parameter key for work target mode
const KEY_WORK_TGT_MODE: u16 = 0x001A;

/// Point cloud packet header size (bytes)
/// SDK2 header: version(1) + length(2) + time_interval(2) + dot_num(2) + udp_cnt(2) +
/// frame_cnt(1) + data_type(1) + time_type(1) + reserved(12) + crc32(4) + timestamp(8) = 36 bytes
const POINT_HEADER_SIZE: usize = 36;

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
    _last_frame_counter: u8,
    timestamp_ns: u64,
    frame_start: Instant,
}

impl FrameAccumulator {
    fn new() -> Self {
        Self {
            points: Vec::with_capacity(POINTS_PER_PACKET * 100), // ~10k points typical
            frame_id: AtomicU32::new(0),
            _last_frame_counter: 0,
            timestamp_ns: 0,
            frame_start: Instant::now(),
        }
    }

    /// Add points from a packet. Returns Some(PointCloud) when frame is complete.
    ///
    /// Frame detection uses time-based accumulation (100ms = 10Hz) since some
    /// Livox firmware versions don't increment frame_cnt properly.
    fn add_packet(&mut self, _frame_counter: u8, timestamp_ns: u64, points: Vec<Point3D>) -> Option<PointCloud> {
        // Time-based frame detection: 100ms per frame (10Hz)
        const FRAME_DURATION: Duration = Duration::from_millis(100);

        let frame_boundary = if self.points.is_empty() {
            false
        } else {
            // Emit frame when 100ms has elapsed OR we have enough points
            self.frame_start.elapsed() >= FRAME_DURATION || self.points.len() >= 25000
        };

        if frame_boundary {
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
            self.points.extend(points);
            Some(cloud)
        } else {
            if self.points.is_empty() {
                self.frame_start = Instant::now();
                self.timestamp_ns = timestamp_ns;
            }
            self.points.extend(points);
            None
        }
    }
}

/// CRC-16/CCITT-FALSE: poly=0x1021, init=0xFFFF, refin=false, refout=false, xorout=0x0000
fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// CRC-32 (Ethernet/ADCCP): poly=0x04C11DB7, init=0xFFFFFFFF, reflected, xorout=0xFFFFFFFF
fn crc32_ethernet(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// Build a properly formatted SDK2 command frame.
///
/// Frame layout (24-byte header + payload):
///   [0]     SOF = 0xAA
///   [1]     Version = 0x00
///   [2..4]  Length (u16 LE) = total frame bytes
///   [4..8]  Seq_Num (u32 LE)
///   [8..10] Cmd_ID (u16 LE)
///   [10]    Cmd_Type (0x00=REQ)
///   [11]    Sender_Type (0x00=Host)
///   [12..18] Reserved (6 zeros)
///   [18..20] CRC-16 over bytes [0..18)
///   [20..24] CRC-32 over payload (0 if empty)
///   [24..]  Data payload
fn build_sdk2_frame(seq: u32, cmd_id: u16, payload: &[u8]) -> Vec<u8> {
    let total_len = 24 + payload.len();
    let mut frame = Vec::with_capacity(total_len);

    // Header fields [0..18)
    frame.push(0xAA);                                        // [0] SOF
    frame.push(0x00);                                        // [1] Version
    frame.extend_from_slice(&(total_len as u16).to_le_bytes()); // [2..4] Length
    frame.extend_from_slice(&seq.to_le_bytes());             // [4..8] Seq
    frame.extend_from_slice(&cmd_id.to_le_bytes());          // [8..10] Cmd_ID
    frame.push(0x00);                                        // [10] Cmd_Type = REQ
    frame.push(0x00);                                        // [11] Sender_Type = Host
    frame.extend_from_slice(&[0u8; 6]);                      // [12..18] Reserved

    // CRC-16 over header [0..18)
    let crc16 = crc16_ccitt_false(&frame[..18]);
    frame.extend_from_slice(&crc16.to_le_bytes());           // [18..20]

    // CRC-32 over payload (0 if empty)
    let crc32 = if payload.is_empty() { 0 } else { crc32_ethernet(payload) };
    frame.extend_from_slice(&crc32.to_le_bytes());           // [20..24]

    // Payload
    frame.extend_from_slice(payload);                        // [24..]

    frame
}

/// Build payload for work mode parameter configuration.
///
/// Format: key_num(2) + rsvd(2) + [key(2) + length(2) + value(N)]...
fn build_work_mode_payload(mode: u8) -> Vec<u8> {
    let mut payload = Vec::with_capacity(9);
    payload.extend_from_slice(&1u16.to_le_bytes());              // key_num = 1
    payload.extend_from_slice(&0u16.to_le_bytes());              // reserved
    payload.extend_from_slice(&KEY_WORK_TGT_MODE.to_le_bytes()); // key = 0x001A
    payload.extend_from_slice(&1u16.to_le_bytes());              // value length = 1
    payload.push(mode);                                           // value
    payload
}

/// Send a work mode command (start/stop sampling) to the Livox Mid-360.
///
/// Uses the SDK2 parameter configuration command (0x0100) with work_tgt_mode key.
pub async fn connect_and_start(config: &Config) -> Result<(), LidarError> {
    let control_addr: SocketAddr = (config.lidar_ip, LIVOX_CMD_PORT).into();

    let cmd_socket = UdpSocket::bind((config.host_ip, 0))
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind command socket: {}", e)))?;

    info!(%control_addr, "Sending SDK2 work mode SAMPLING to Livox Mid360");

    let payload = build_work_mode_payload(WORK_MODE_SAMPLING);
    let frame = build_sdk2_frame(1, CMD_ID_WORK_MODE, &payload);

    cmd_socket
        .send_to(&frame, control_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send start command: {}", e)))?;

    // Wait for ACK
    let mut response = [0u8; 256];
    match tokio::time::timeout(Duration::from_secs(2), cmd_socket.recv(&mut response)).await {
        Ok(Ok(len)) => {
            info!(len, "Received work mode ACK");
            if len >= 24 {
                debug!(response = ?&response[..len.min(32)], "ACK data");
            }
        }
        Ok(Err(e)) => {
            warn!(?e, "Error receiving work mode ACK");
        }
        Err(_) => {
            warn!("Work mode ACK timeout - device may already be sampling");
        }
    }

    info!("Livox SDK2 start sequence complete");
    Ok(())
}

/// Send a work mode command (SAMPLING or IDLE) to the Livox Mid-360.
///
/// Uses SDK2 parameter configuration (cmd_id 0x0100) with work_tgt_mode key.
async fn send_work_mode(
    socket: &UdpSocket,
    control_addr: SocketAddr,
    command: LidarCommand,
    seq: u32,
) -> Result<(), LidarError> {
    let (mode, label) = match command {
        LidarCommand::Start => (WORK_MODE_SAMPLING, "SAMPLING"),
        LidarCommand::Stop => (WORK_MODE_IDLE, "IDLE"),
    };

    let payload = build_work_mode_payload(mode);
    let frame = build_sdk2_frame(seq, CMD_ID_WORK_MODE, &payload);

    debug!(label, seq, frame_len = frame.len(), "Sending SDK2 work mode command");

    socket
        .send_to(&frame, control_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send {label}: {e}")))?;

    // Wait for ACK (non-blocking)
    let mut response = [0u8; 256];
    match tokio::time::timeout(Duration::from_millis(500), socket.recv(&mut response)).await {
        Ok(Ok(len)) => {
            debug!(len, label, "Received work mode ACK");
        }
        Ok(Err(e)) => {
            warn!(?e, label, "Error receiving work mode ACK");
        }
        Err(_) => {
            debug!(label, "Work mode ACK timeout");
        }
    }

    Ok(())
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
        mounting_pitch_deg = config.mounting_pitch_deg,
        "Starting Livox Mid360 reader"
    );

    // Create mounting transform (applied to all points)
    let transform = MountingTransform::new(config.mounting_pitch_deg);
    let has_transform = config.mounting_pitch_deg.abs() > 0.01;
    if has_transform {
        info!(
            pitch_deg = config.mounting_pitch_deg,
            "Applying mounting pitch transform to point cloud"
        );
    }

    // Attempt to connect and start data streaming
    if let Err(e) = connect_and_start(&config).await {
        warn!(?e, "Failed to send connect/start commands - will listen anyway");
    }

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
                    Ok((len, src)) => {
                        packet_count += 1;
                        if packet_count == 1 || packet_count % 1000 == 0 {
                            debug!(packets = packet_count, len, %src, "Point cloud packet received");
                        }

                        match parse_point_packet(&point_buf[..len]) {
                            Ok((frame_counter, timestamp_ns, points)) => {
                                debug!(
                                    frame_counter,
                                    points = points.len(),
                                    "Parsed point packet"
                                );

                                if let Some(mut cloud) = frame_acc.add_packet(frame_counter, timestamp_ns, points) {
                                    // Apply mounting transform if configured
                                    if has_transform {
                                        transform.apply_to_cloud(&mut cloud);
                                    }

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
                                debug!(?e, len, "Failed to parse point packet");
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
                            // Log first IMU packet and then every 1000th
                            static IMU_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            let count = IMU_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if count == 0 {
                                info!(
                                    gyro_x = imu.gyro_x,
                                    gyro_y = imu.gyro_y,
                                    gyro_z = imu.gyro_z,
                                    accel_x = imu.accel_x,
                                    accel_y = imu.accel_y,
                                    accel_z = imu.accel_z,
                                    "First IMU packet received"
                                );
                            } else if count % 10000 == 0 {
                                debug!(
                                    count,
                                    accel_z = imu.accel_z,
                                    "IMU packets received"
                                );
                            }
                            let _ = imu_tx.send(Some(imu));
                        }
                        Err(e) => {
                            trace!(?e, len, "Failed to parse IMU packet");
                        }
                    }
                }
            }
        }
    }

    info!("Livox Mid360 reader stopped");
    Ok(())
}

/// Run the LiDAR reader with runtime start/stop control.
///
/// This version keeps a control socket open and responds to Start/Stop commands
/// to control the lidar hardware. When stopped, the motor stops spinning.
pub async fn run_reader_with_control(
    config: Config,
    point_tx: watch::Sender<Option<PointCloud>>,
    imu_tx: watch::Sender<Option<ImuData>>,
    mut cmd_rx: mpsc::Receiver<LidarCommand>,
    start_immediately: bool,
) -> Result<(), LidarError> {
    info!(
        lidar_ip = %config.lidar_ip,
        point_port = config.point_cloud_port,
        start_immediately,
        "Starting Livox Mid360 reader with control"
    );

    let control_addr: SocketAddr = (config.lidar_ip, LIVOX_CMD_PORT).into();

    // Create mounting transform
    let transform = MountingTransform::new(config.mounting_pitch_deg);
    let has_transform = config.mounting_pitch_deg.abs() > 0.01;

    // Bind command socket for sending start/stop (SDK2 commands to port 56100)
    let cmd_socket = UdpSocket::bind((config.host_ip, 0))
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind command socket: {e}")))?;

    info!(%control_addr, "SDK2 command socket ready");

    let mut seq: u32 = 1;
    let mut sampling = false;

    if start_immediately {
        if let Err(e) = send_work_mode(&cmd_socket, control_addr, LidarCommand::Start, seq).await {
            warn!(?e, "Failed to send initial SAMPLING command");
        }
        seq += 1;
        sampling = true;
        info!("LiDAR work mode set to SAMPLING");
    } else {
        info!("LiDAR initialized but waiting for start command");
    }

    // Bind UDP sockets for data
    let point_addr: SocketAddr = (config.host_ip, config.point_cloud_port).into();
    let point_socket = UdpSocket::bind(point_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind point cloud socket: {e}")))?;

    let imu_socket = if config.imu_port > 0 {
        let imu_addr: SocketAddr = (config.host_ip, config.imu_port).into();
        UdpSocket::bind(imu_addr).await.ok()
    } else {
        None
    };

    let mut frame_acc = FrameAccumulator::new();
    let mut point_buf = vec![0u8; 2048];
    let mut imu_buf = vec![0u8; IMU_PACKET_SIZE + 32];

    // Heartbeat keeps the Livox control session alive (especially while stopped).
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(1));
    heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            // Handle control commands
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    LidarCommand::Start if !sampling => {
                        info!("Setting LiDAR work mode to SAMPLING");
                        if let Err(e) = send_work_mode(&cmd_socket, control_addr, cmd, seq).await {
                            warn!(?e, "Failed to send SAMPLING command");
                        }
                        seq += 1;
                        sampling = true;
                        // Reset frame accumulator for clean start
                        frame_acc = FrameAccumulator::new();
                    }
                    LidarCommand::Stop if sampling => {
                        info!("Setting LiDAR work mode to IDLE");
                        if let Err(e) = send_work_mode(&cmd_socket, control_addr, cmd, seq).await {
                            warn!(?e, "Failed to send IDLE command");
                        }
                        seq += 1;
                        sampling = false;
                    }
                    _ => {
                        // Redundant command (already in requested state)
                        debug!(?cmd, sampling, "Ignoring redundant lidar command");
                    }
                }
            }

            // Handle point cloud packets
            result = point_socket.recv_from(&mut point_buf) => {
                if !sampling {
                    // Discard data when stopped (shouldn't receive much after stop command)
                    continue;
                }

                match result {
                    Ok((len, _src)) => {
                        match parse_point_packet(&point_buf[..len]) {
                            Ok((frame_counter, timestamp_ns, points)) => {
                                if let Some(mut cloud) = frame_acc.add_packet(frame_counter, timestamp_ns, points) {
                                    if has_transform {
                                        transform.apply_to_cloud(&mut cloud);
                                    }
                                    if point_tx.send(Some(cloud)).is_err() {
                                        info!("Point cloud receiver dropped, stopping");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                debug!(?e, len, "Failed to parse point packet");
                            }
                        }
                    }
                    Err(e) => {
                        error!(?e, "Point cloud socket error");
                        break;
                    }
                }
            }

            // Handle IMU packets
            result = async {
                if let Some(ref socket) = imu_socket {
                    socket.recv_from(&mut imu_buf).await
                } else {
                    std::future::pending().await
                }
            } => {
                if !sampling {
                    continue;
                }

                if let Ok((len, _src)) = result {
                    if let Ok(imu) = parse_imu_packet(&imu_buf[..len]) {
                        let _ = imu_tx.send(Some(imu));
                    }
                }
            }

            // Periodic keepalive: re-send current work mode to maintain session
            _ = heartbeat_interval.tick() => {
                // Silently re-assert current state (don't log at info level)
                let payload = build_work_mode_payload(
                    if sampling { WORK_MODE_SAMPLING } else { WORK_MODE_IDLE }
                );
                let frame = build_sdk2_frame(seq, CMD_ID_WORK_MODE, &payload);
                if let Err(e) = cmd_socket.send_to(&frame, control_addr).await {
                    debug!(?e, "Keepalive send failed");
                }
                // Drain any response
                let mut buf = [0u8; 256];
                let _ = tokio::time::timeout(
                    Duration::from_millis(100),
                    cmd_socket.recv(&mut buf),
                ).await;
                seq += 1;
            }
        }
    }

    // Send IDLE on exit
    if sampling {
        let _ = send_work_mode(&cmd_socket, control_addr, LidarCommand::Stop, seq).await;
    }

    info!("Livox Mid360 reader with control stopped");
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

    // Parse header (SDK2 format for Mid-360)
    // Bytes 0: version
    // Bytes 1-2: length (little-endian)
    // Bytes 3-4: time_interval (0.1us units)
    // Bytes 5-6: dot_num (points in packet)
    // Bytes 7-8: udp_cnt (sequence number, u16)
    // Byte 9: frame_cnt (per SDK2 protocol spec)
    // Byte 10: data_type
    // Byte 11: time_type
    // Bytes 12-23: reserved
    // Bytes 24-27: crc32
    // Bytes 28-35: timestamp (8 bytes, nanoseconds)
    // Bytes 36+: point data

    let _version = data[0];
    let length = u16::from_le_bytes([data[1], data[2]]) as usize;
    let dot_num = u16::from_le_bytes([data[5], data[6]]) as usize;
    let frame_cnt = data[9]; // Byte 9 per SDK2 protocol (often 0 on Mid-360)
    let data_type = data[10];

    // Validate length
    if data.len() < length {
        return Err(LidarError::Parse(format!(
            "Incomplete packet: {} < {}",
            data.len(),
            length
        )));
    }

    // Timestamp is at offset 28 in SDK2 header (after crc32 at offset 24)
    let timestamp_offset = 28;
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
///
/// SDK2 IMU packet format (60 bytes total):
/// Bytes 0: version
/// Bytes 1-2: length (LE)
/// Bytes 3-4: time_interval
/// Bytes 5-6: data_num
/// Bytes 7-8: udp_cnt
/// Byte 9: frame_cnt
/// Byte 10: data_type (0x05 for IMU)
/// Byte 11: time_type
/// Bytes 12-23: reserved
/// Bytes 24-27: crc32
/// Bytes 28-35: timestamp (8 bytes, nanoseconds)
/// Bytes 36-59: IMU data (6 floats: gyro_x, gyro_y, gyro_z, accel_x, accel_y, accel_z)
fn parse_imu_packet(data: &[u8]) -> Result<ImuData, LidarError> {
    // Minimum packet size: 36 byte header + 24 bytes IMU data = 60 bytes
    if data.len() < 60 {
        return Err(LidarError::Parse(format!("IMU packet too small: {} bytes", data.len())));
    }

    // Timestamp at offset 28 (after header, before IMU data)
    let timestamp_ns = u64::from_le_bytes(data[28..36].try_into().unwrap());

    // IMU data starts at offset 36
    let imu_offset = 36;
    let gyro_x = f32::from_le_bytes(data[imu_offset..imu_offset + 4].try_into().unwrap());
    let gyro_y = f32::from_le_bytes(data[imu_offset + 4..imu_offset + 8].try_into().unwrap());
    let gyro_z = f32::from_le_bytes(data[imu_offset + 8..imu_offset + 12].try_into().unwrap());
    let accel_x = f32::from_le_bytes(data[imu_offset + 12..imu_offset + 16].try_into().unwrap());
    let accel_y = f32::from_le_bytes(data[imu_offset + 16..imu_offset + 20].try_into().unwrap());
    let accel_z = f32::from_le_bytes(data[imu_offset + 20..imu_offset + 24].try_into().unwrap());

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
        let mut data = vec![0u8; 60];
        // Timestamp at offset 28 = 1000000ns
        data[28..36].copy_from_slice(&1000000u64.to_le_bytes());
        // IMU data at offset 36
        // Gyro X = 0.1 rad/s
        data[36..40].copy_from_slice(&0.1f32.to_le_bytes());
        // Accel X = 9.8 m/s² (at offset 36 + 12 = 48)
        data[48..52].copy_from_slice(&9.8f32.to_le_bytes());

        let imu = parse_imu_packet(&data).unwrap();
        assert_eq!(imu.timestamp_ns, 1000000);
        assert!((imu.gyro_x - 0.1).abs() < 0.001);
        assert!((imu.accel_x - 9.8).abs() < 0.001);
    }

    #[test]
    fn test_crc16_ccitt_false() {
        // Known test vector: "123456789" → 0x29B1
        let data = b"123456789";
        assert_eq!(crc16_ccitt_false(data), 0x29B1);
    }

    #[test]
    fn test_crc32_ethernet() {
        // Known test vector: "123456789" → 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32_ethernet(data), 0xCBF43926);
    }

    #[test]
    fn test_sdk2_frame_structure() {
        let frame = build_sdk2_frame(42, 0x0100, &[0x01, 0x02]);
        // Total length: 24 header + 2 payload = 26
        assert_eq!(frame.len(), 26);
        assert_eq!(frame[0], 0xAA); // SOF
        assert_eq!(frame[1], 0x00); // Version
        assert_eq!(u16::from_le_bytes([frame[2], frame[3]]), 26); // Length
        assert_eq!(u32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]), 42); // Seq
        assert_eq!(u16::from_le_bytes([frame[8], frame[9]]), 0x0100); // Cmd_ID
        assert_eq!(frame[10], 0x00); // Cmd_Type = REQ
        assert_eq!(frame[11], 0x00); // Sender_Type = Host
        // CRC-16 at [18..20] should match header checksum
        let expected_crc16 = crc16_ccitt_false(&frame[..18]);
        assert_eq!(u16::from_le_bytes([frame[18], frame[19]]), expected_crc16);
        // CRC-32 at [20..24] should match payload checksum
        let expected_crc32 = crc32_ethernet(&[0x01, 0x02]);
        assert_eq!(u32::from_le_bytes([frame[20], frame[21], frame[22], frame[23]]), expected_crc32);
        // Payload
        assert_eq!(frame[24], 0x01);
        assert_eq!(frame[25], 0x02);
    }

    #[test]
    fn test_work_mode_payload() {
        let payload = build_work_mode_payload(WORK_MODE_SAMPLING);
        assert_eq!(payload.len(), 9);
        // key_num = 1
        assert_eq!(u16::from_le_bytes([payload[0], payload[1]]), 1);
        // reserved = 0
        assert_eq!(u16::from_le_bytes([payload[2], payload[3]]), 0);
        // key = 0x001A
        assert_eq!(u16::from_le_bytes([payload[4], payload[5]]), 0x001A);
        // length = 1
        assert_eq!(u16::from_le_bytes([payload[6], payload[7]]), 1);
        // value = SAMPLING (0x01)
        assert_eq!(payload[8], 0x01);
    }
}
