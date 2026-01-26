//! Per-rover simulation task.
//!
//! Each rover runs:
//! - TCP server for CAN frames (bidirectional with bvrd)
//! - UDP sender for LiDAR point cloud + IMU packets
//! - 100Hz physics loop driven by VESC motor commands

use crate::can_protocol::{self, CanFrame, Message};
use crate::imu_sim;
use crate::lidar_sim::{LidarConfig, LidarSim};
use crate::livox_protocol;
use crate::physics::Physics;
use crate::scenario::RoverConfig;
use crate::tool::SimTool;
use crate::vesc::SimVesc;
use crate::world::World;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Shared rover state for status API.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoverStatus {
    pub id: String,
    pub connected: bool,
    pub pose: PoseStatus,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PoseStatus {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

/// Run a single rover simulation.
pub async fn run_rover(
    config: RoverConfig,
    world: Arc<World>,
    status: Arc<tokio::sync::Mutex<RoverStatus>>,
) {
    let id = config.id.clone();
    info!(rover = %id, can_port = config.can_port, "Starting rover simulation");

    // TCP server for CAN frames
    let listener = match TcpListener::bind(("0.0.0.0", config.can_port)).await {
        Ok(l) => l,
        Err(e) => {
            error!(rover = %id, ?e, "Failed to bind CAN TCP port");
            return;
        }
    };

    info!(rover = %id, port = config.can_port, "Waiting for bvrd connection");

    // Accept connections in a loop (reconnect support)
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!(rover = %id, %addr, "bvrd connected");
                {
                    let mut s = status.lock().await;
                    s.connected = true;
                }
                run_rover_session(&config, &world, stream, &status).await;
                {
                    let mut s = status.lock().await;
                    s.connected = false;
                }
                warn!(rover = %id, "bvrd disconnected, waiting for reconnect");
            }
            Err(e) => {
                error!(rover = %id, ?e, "TCP accept error");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn run_rover_session(
    config: &RoverConfig,
    world: &World,
    stream: tokio::net::TcpStream,
    status: &Arc<tokio::sync::Mutex<RoverStatus>>,
) {
    let id = &config.id;
    stream.set_nodelay(true).ok();
    let (mut reader, mut writer) = tokio::io::split(stream);

    // UDP socket for LiDAR + IMU
    let udp = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            error!(rover = %id, ?e, "Failed to bind UDP socket");
            return;
        }
    };
    let point_addr: SocketAddr = ([127, 0, 0, 1], config.point_cloud_port).into();
    let imu_addr: SocketAddr = ([127, 0, 0, 1], config.imu_port).into();

    // Physics + VESC sim + LiDAR sim
    let mut physics = Physics::new();
    physics.set_position(config.spawn.x, config.spawn.y, config.spawn.theta);
    let mut vescs = [
        SimVesc::new(0),
        SimVesc::new(1),
        SimVesc::new(2),
        SimVesc::new(3),
    ];
    let mut tool = SimTool::new_auger(0);
    let mut lidar = LidarSim::new(LidarConfig::default());
    let mut tick_count = 0u64;
    let mut udp_seq = 0u16;

    // Incoming CAN frames buffer
    let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<CanFrame>(256);

    // Background TCP reader task
    let reader_id = id.clone();
    tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        let mut pending = Vec::new();

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    debug!(rover = %reader_id, "TCP reader: connection closed");
                    break;
                }
                Ok(n) => {
                    pending.extend_from_slice(&buf[..n]);
                    // Parse messages
                    loop {
                        match can_protocol::decode_message(&pending) {
                            Some((Message::Frame(frame), consumed)) => {
                                if frame_tx.try_send(frame).is_err() {
                                    // Queue full, drop frame
                                }
                                pending.drain(..consumed);
                            }
                            Some((Message::Pose { .. }, consumed)) => {
                                // bvrd shouldn't send pose, but skip gracefully
                                pending.drain(..consumed);
                            }
                            None => break,
                        }
                    }
                }
                Err(e) => {
                    debug!(rover = %reader_id, ?e, "TCP reader error");
                    break;
                }
            }
        }
    });

    // 100Hz simulation loop
    let dt = 0.01;
    let mut tick_interval = interval(Duration::from_millis(10));
    let start = std::time::Instant::now();

    loop {
        tick_interval.tick().await;
        tick_count += 1;

        // Drain incoming CAN frames from bvrd
        while let Ok(frame) = frame_rx.try_recv() {
            for vesc in &mut vescs {
                vesc.process_command(&frame);
            }
            tool.process_command(&frame);
        }

        // Tick VESCs
        for vesc in &mut vescs {
            vesc.tick(dt);
        }
        tool.tick(dt);

        // Get wheel RPMs and tick physics
        let wheel_rpms = [
            vescs[0].rpm(),
            vescs[1].rpm(),
            vescs[2].rpm(),
            vescs[3].rpm(),
        ];
        physics.update_with_world(wheel_rpms, dt, Some(world));

        // Send VESC status frames back to bvrd (at ~50Hz per VESC)
        let mut outgoing = Vec::new();
        for vesc in &mut vescs {
            if vesc.should_send_status() {
                for frame in vesc.generate_status() {
                    outgoing.extend_from_slice(&can_protocol::encode_can_frame(&frame));
                }
            }
        }
        // Tool frames
        if let Some(frame) = tool.generate_frame() {
            outgoing.extend_from_slice(&can_protocol::encode_can_frame(&frame));
        }

        // Send pose update at 10Hz
        if tick_count % 10 == 0 {
            let (x, y, theta) = physics.position();
            outgoing.extend_from_slice(&can_protocol::encode_pose(x, y, theta));

            // Update status
            {
                let mut s = status.lock().await;
                s.pose = PoseStatus { x, y, theta };
            }
        }

        // Write all outgoing data
        if !outgoing.is_empty() {
            if writer.write_all(&outgoing).await.is_err() {
                debug!(rover = %id, "TCP write failed, session ending");
                break;
            }
        }

        // LiDAR + IMU at 10Hz (every 10 ticks)
        if tick_count % 10 == 0 {
            let (x, y, theta) = physics.position();
            let timestamp_ns = start.elapsed().as_nanos() as u64;

            // Raycast LiDAR
            let points = lidar.scan(world, x, y, theta);

            // Split into Livox packets and send via UDP
            let packets = livox_protocol::split_into_packets(&points, udp_seq, timestamp_ns);
            for pkt in &packets {
                if let Err(e) = udp.send_to(pkt, point_addr).await {
                    debug!(rover = %id, ?e, "UDP point cloud send error");
                    break;
                }
            }
            udp_seq = udp_seq.wrapping_add(packets.len() as u16);

            // Send IMU packet
            let imu = imu_sim::imu_from_physics(&physics);
            let imu_pkt = livox_protocol::build_imu_packet(imu.gyro, imu.accel, udp_seq, timestamp_ns);
            udp_seq = udp_seq.wrapping_add(1);
            if let Err(e) = udp.send_to(&imu_pkt, imu_addr).await {
                debug!(rover = %id, ?e, "UDP IMU send error");
            }
        }
    }
}
