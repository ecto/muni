//! Livox Mid360 UDP protocol implementation.
//!
//! Protocol reference: https://livox-wiki-en.readthedocs.io/en/latest/tutorials/new_product/mid360/livox_eth_protocol_mid360.html

use crate::{Config, FovRegion, ImuData, LidarError, Point3D, PointCloud, PointDataType, TimeSyncMode};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tracing::{debug, error, info, trace, warn};

/// Livox SDK2 control port on the device
const LIVOX_CONTROL_PORT: u16 = 56000;

// SDK2 general command set (cmd_set = 0x00)
const CMD_HANDSHAKE: u8 = 0x01;
const CMD_HEARTBEAT: u8 = 0x03;
const CMD_START_SAMPLING: u8 = 0x04;
#[allow(dead_code)]
const CMD_STOP_SAMPLING: u8 = 0x05;
#[allow(dead_code)]
const CMD_PUSH_CONFIG: u8 = 0x07;

// SDK2 LiDAR command set (cmd_set = 0x01)
const CMD_SET_PARAM: u8 = 0x00;      // Set LiDAR parameters
#[allow(dead_code)]
const CMD_GET_PARAM: u8 = 0x01;      // Get LiDAR parameters

// Parameter keys for CMD_SET_PARAM
const PARAM_KEY_DATA_TYPE: u16 = 0x0000;     // Point cloud data format
#[allow(dead_code)]
const PARAM_KEY_PATTERN_MODE: u16 = 0x0001;  // Scan pattern mode
const PARAM_KEY_FOV_CFG0: u16 = 0x0005;      // FOV region 1 configuration
const PARAM_KEY_FOV_CFG1: u16 = 0x0006;      // FOV region 2 configuration
const PARAM_KEY_FOV_CFG_EN: u16 = 0x0007;    // FOV enable flags
const PARAM_KEY_TIME_SYNC: u16 = 0x0008;     // Time synchronization mode

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
    #[allow(dead_code)] // Reserved for future frame_cnt-based detection
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

/// Connect to the Livox and start data streaming.
///
/// This sends the SDK2 handshake and start sampling commands.
pub async fn connect_and_start(config: &Config) -> Result<(), LidarError> {
    let control_addr: SocketAddr = (config.lidar_ip, LIVOX_CONTROL_PORT).into();

    // Bind a socket for sending commands and receiving responses
    let cmd_socket = UdpSocket::bind((config.host_ip, 0))
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind command socket: {}", e)))?;

    let local_addr = cmd_socket.local_addr()
        .map_err(|e| LidarError::Network(format!("Failed to get local addr: {}", e)))?;
    info!(?local_addr, ?control_addr, "Connecting to Livox Mid360");

    // Build and send handshake command
    // SDK2 format: [SOF(0xAA), version, length(2), seq(4), cmd_id, cmd_type, data...]
    let mut handshake = Vec::with_capacity(64);
    handshake.push(0xAA); // SOF
    handshake.push(0x01); // Version

    // Handshake data: host_ip(4) + cmd_port(2) + push_msg_port(2) + point_port(2) + imu_port(2) + log_port(2)
    let data_len: u16 = 14 + 8; // header (8) + ip(4) + ports(10)
    handshake.extend_from_slice(&data_len.to_le_bytes());

    // Sequence number
    let seq: u32 = 1;
    handshake.extend_from_slice(&seq.to_le_bytes());

    // Command
    handshake.push(CMD_HANDSHAKE);
    handshake.push(0x00); // Request type

    // Host IP
    let host_octets = config.host_ip.octets();
    handshake.extend_from_slice(&host_octets);

    // Ports: cmd_reply, push_msg, point_cloud, imu, log
    handshake.extend_from_slice(&local_addr.port().to_le_bytes()); // cmd reply
    handshake.extend_from_slice(&(local_addr.port() + 1).to_le_bytes()); // push msg
    handshake.extend_from_slice(&config.point_cloud_port.to_le_bytes());
    handshake.extend_from_slice(&config.imu_port.to_le_bytes());
    handshake.extend_from_slice(&0u16.to_le_bytes()); // log port (disabled)

    debug!(len = handshake.len(), "Sending handshake");
    cmd_socket.send_to(&handshake, control_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send handshake: {}", e)))?;

    // Wait for response
    let mut response = [0u8; 256];
    match tokio::time::timeout(Duration::from_secs(2), cmd_socket.recv(&mut response)).await {
        Ok(Ok(len)) => {
            info!(len, "Received handshake response");
            if len >= 2 && response[0] == 0xAA {
                // Check for success (response[9] should be 0 for success in SDK2)
                debug!(response = ?&response[..len.min(20)], "Handshake response data");
            }
        }
        Ok(Err(e)) => {
            warn!(?e, "Error receiving handshake response");
        }
        Err(_) => {
            warn!("Handshake response timeout - device may not support SDK2 handshake");
        }
    }

    // Send start sampling command
    let mut start_cmd = Vec::with_capacity(16);
    start_cmd.push(0xAA); // SOF
    start_cmd.push(0x01); // Version
    start_cmd.extend_from_slice(&8u16.to_le_bytes()); // Length (header only)
    start_cmd.extend_from_slice(&2u32.to_le_bytes()); // Seq
    start_cmd.push(CMD_START_SAMPLING);
    start_cmd.push(0x00); // Request

    debug!("Sending start sampling");
    cmd_socket.send_to(&start_cmd, control_addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send start: {}", e)))?;

    // Wait briefly for response
    match tokio::time::timeout(Duration::from_secs(1), cmd_socket.recv(&mut response)).await {
        Ok(Ok(len)) => {
            info!(len, "Received start sampling response");
        }
        Ok(Err(e)) => {
            warn!(?e, "Error receiving start response");
        }
        Err(_) => {
            info!("Start sampling response timeout - continuing anyway");
        }
    }

    info!("Livox connection sequence complete");
    Ok(())
}

/// Sequence number generator for commands
static CMD_SEQ: AtomicU32 = AtomicU32::new(10);

fn next_seq() -> u32 {
    CMD_SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Build a parameter set command packet.
fn build_set_param_cmd(key: u16, value: &[u8]) -> Vec<u8> {
    // SDK2 parameter command format:
    // Header: SOF(1) + Version(1) + Length(2) + Seq(4) + CmdSet(1) + CmdId(1)
    // Data: KeyNum(1) + [Key(2) + ValueLen(2) + Value(...)]
    let value_len = value.len() as u16;
    let data_len = 1 + 2 + 2 + value.len(); // KeyNum + Key + ValueLen + Value
    let total_len = 10 + data_len; // Header(10) + Data

    let mut cmd = Vec::with_capacity(total_len);
    cmd.push(0xAA); // SOF
    cmd.push(0x01); // Version
    cmd.extend_from_slice(&(total_len as u16).to_le_bytes()); // Length
    cmd.extend_from_slice(&next_seq().to_le_bytes()); // Seq
    cmd.push(0x01); // CmdSet (LiDAR command set)
    cmd.push(CMD_SET_PARAM); // CmdId

    // Data
    cmd.push(1); // KeyNum = 1 (setting one parameter)
    cmd.extend_from_slice(&key.to_le_bytes()); // Key
    cmd.extend_from_slice(&value_len.to_le_bytes()); // ValueLen
    cmd.extend_from_slice(value); // Value

    cmd
}

/// Send a configuration command and wait for response.
async fn send_config_cmd(
    socket: &UdpSocket,
    addr: SocketAddr,
    cmd: &[u8],
    name: &str,
) -> Result<(), LidarError> {
    debug!(name, len = cmd.len(), "Sending config command");
    socket.send_to(cmd, addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send {}: {}", name, e)))?;

    let mut response = [0u8; 256];
    match tokio::time::timeout(Duration::from_millis(500), socket.recv(&mut response)).await {
        Ok(Ok(len)) => {
            debug!(name, len, "Received config response");
            // Check response status (byte 10 in SDK2 response)
            if len >= 11 && response[10] != 0 {
                warn!(name, status = response[10], "Config command returned error status");
            }
        }
        Ok(Err(e)) => {
            warn!(name, ?e, "Error receiving config response");
        }
        Err(_) => {
            debug!(name, "Config response timeout - continuing");
        }
    }
    Ok(())
}

/// Configure point cloud data type.
pub async fn configure_data_type(
    socket: &UdpSocket,
    addr: SocketAddr,
    data_type: PointDataType,
) -> Result<(), LidarError> {
    let cmd = build_set_param_cmd(PARAM_KEY_DATA_TYPE, &[data_type as u8]);
    send_config_cmd(socket, addr, &cmd, "data_type").await
}

/// Configure time synchronization mode.
pub async fn configure_time_sync(
    socket: &UdpSocket,
    addr: SocketAddr,
    mode: TimeSyncMode,
) -> Result<(), LidarError> {
    let cmd = build_set_param_cmd(PARAM_KEY_TIME_SYNC, &[mode as u8]);
    send_config_cmd(socket, addr, &cmd, "time_sync").await
}

/// Configure FOV regions.
///
/// Each region is defined by yaw and pitch angle ranges.
/// If both regions are None, full FOV is used.
pub async fn configure_fov(
    socket: &UdpSocket,
    addr: SocketAddr,
    region1: Option<FovRegion>,
    region2: Option<FovRegion>,
) -> Result<(), LidarError> {
    // Build enable flags: bit 0 = region1, bit 1 = region2
    let enable_flags: u8 = (region1.is_some() as u8) | ((region2.is_some() as u8) << 1);

    // If no regions configured, just disable FOV limiting
    if enable_flags == 0 {
        let cmd = build_set_param_cmd(PARAM_KEY_FOV_CFG_EN, &[0]);
        return send_config_cmd(socket, addr, &cmd, "fov_disable").await;
    }

    // Configure region 1 if present
    if let Some(ref region) = region1 {
        let mut data = Vec::with_capacity(16);
        // FOV config format: yaw_start(i32, 0.01°) + yaw_stop(i32) + pitch_start(i32) + pitch_stop(i32)
        data.extend_from_slice(&((region.yaw_start * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.yaw_stop * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.pitch_start * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.pitch_stop * 100.0) as i32).to_le_bytes());
        let cmd = build_set_param_cmd(PARAM_KEY_FOV_CFG0, &data);
        send_config_cmd(socket, addr, &cmd, "fov_region1").await?;
    }

    // Configure region 2 if present
    if let Some(ref region) = region2 {
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&((region.yaw_start * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.yaw_stop * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.pitch_start * 100.0) as i32).to_le_bytes());
        data.extend_from_slice(&((region.pitch_stop * 100.0) as i32).to_le_bytes());
        let cmd = build_set_param_cmd(PARAM_KEY_FOV_CFG1, &data);
        send_config_cmd(socket, addr, &cmd, "fov_region2").await?;
    }

    // Enable configured regions
    let cmd = build_set_param_cmd(PARAM_KEY_FOV_CFG_EN, &[enable_flags]);
    send_config_cmd(socket, addr, &cmd, "fov_enable").await
}

/// Build and send a heartbeat command.
async fn send_heartbeat(socket: &UdpSocket, addr: SocketAddr) -> Result<(), LidarError> {
    let mut cmd = Vec::with_capacity(16);
    cmd.push(0xAA); // SOF
    cmd.push(0x01); // Version
    cmd.extend_from_slice(&10u16.to_le_bytes()); // Length
    cmd.extend_from_slice(&next_seq().to_le_bytes()); // Seq
    cmd.push(0x00); // CmdSet (general)
    cmd.push(CMD_HEARTBEAT); // CmdId

    socket.send_to(&cmd, addr)
        .await
        .map_err(|e| LidarError::Network(format!("Failed to send heartbeat: {}", e)))?;
    Ok(())
}

/// Spawn a heartbeat task that sends periodic heartbeats to the LiDAR.
fn spawn_heartbeat_task(
    lidar_ip: std::net::Ipv4Addr,
    host_ip: std::net::Ipv4Addr,
    interval_ms: u32,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if interval_ms == 0 {
            return; // Heartbeat disabled
        }

        let interval = Duration::from_millis(interval_ms as u64);
        let control_addr: SocketAddr = (lidar_ip, LIVOX_CONTROL_PORT).into();

        // Bind a socket for heartbeats
        let socket = match UdpSocket::bind((host_ip, 0)).await {
            Ok(s) => s,
            Err(e) => {
                warn!(?e, "Failed to bind heartbeat socket");
                return;
            }
        };

        let mut heartbeat_count: u64 = 0;
        loop {
            tokio::time::sleep(interval).await;

            if let Err(e) = send_heartbeat(&socket, control_addr).await {
                debug!(?e, "Heartbeat send failed");
            } else {
                heartbeat_count += 1;
                if heartbeat_count % 60 == 0 {
                    trace!(count = heartbeat_count, "Heartbeats sent");
                }
            }
        }
    })
}

/// Connect to Livox and apply all configuration.
pub async fn connect_and_configure(config: &Config) -> Result<(), LidarError> {
    let control_addr: SocketAddr = (config.lidar_ip, LIVOX_CONTROL_PORT).into();

    // First do the basic handshake and start sampling
    connect_and_start(config).await?;

    // Bind a socket for configuration commands
    let cmd_socket = UdpSocket::bind((config.host_ip, 0))
        .await
        .map_err(|e| LidarError::Network(format!("Failed to bind config socket: {}", e)))?;

    // Configure data type if not default
    if config.data_type != PointDataType::Cartesian32 {
        info!(data_type = ?config.data_type, "Configuring point data type");
        configure_data_type(&cmd_socket, control_addr, config.data_type).await?;
    }

    // Configure time sync if enabled
    if config.time_sync != TimeSyncMode::None {
        info!(mode = ?config.time_sync, "Configuring time synchronization");
        configure_time_sync(&cmd_socket, control_addr, config.time_sync).await?;
    }

    // Configure FOV if specified
    if config.fov_region1.is_some() || config.fov_region2.is_some() {
        info!(
            region1 = config.fov_region1.is_some(),
            region2 = config.fov_region2.is_some(),
            "Configuring FOV regions"
        );
        configure_fov(&cmd_socket, control_addr, config.fov_region1, config.fov_region2).await?;
    }

    info!("Livox configuration complete");
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
        data_type = ?config.data_type,
        time_sync = ?config.time_sync,
        heartbeat_ms = config.heartbeat_interval_ms,
        "Starting Livox Mid360 reader"
    );

    // Attempt to connect, configure, and start data streaming
    if let Err(e) = connect_and_configure(&config).await {
        warn!(?e, "Failed to send connect/configure commands - will listen anyway");
    }

    // Spawn heartbeat task if enabled
    let _heartbeat_handle = if config.heartbeat_interval_ms > 0 {
        info!(interval_ms = config.heartbeat_interval_ms, "Starting heartbeat task");
        Some(spawn_heartbeat_task(
            config.lidar_ip,
            config.host_ip,
            config.heartbeat_interval_ms,
        ))
    } else {
        None
    };

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
}
