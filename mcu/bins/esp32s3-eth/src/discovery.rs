//! UDP discovery broadcast — sends identity/heartbeat packets every 500ms.

use crate::config::ToolConfig;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

/// Discovery broadcast interval.
const BROADCAST_INTERVAL: Duration = Duration::from_millis(500);

/// Identity packet size (matches bvrd eth_protocol).
const IDENTITY_SIZE: usize = 24;

/// Message type for identity/heartbeat.
const MSG_IDENTITY: u8 = 0x01;

/// Protocol version.
const PROTOCOL_VERSION: u8 = 1;

/// MCU state.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum McuState {
    Idle = 0,
    Running = 1,
    Error = 2,
    Disabled = 3,
}

/// Capabilities bitflags.
const CAP_LIGHTS: u16 = 0x0020;
const CAP_AXIS: u16 = 0x0001;

/// Discovery broadcaster.
pub struct DiscoveryBroadcaster {
    socket: UdpSocket,
    broadcast_addr: String,
    identity_packet: [u8; IDENTITY_SIZE],
    last_broadcast: Instant,
}

impl DiscoveryBroadcaster {
    pub fn new(config: &ToolConfig) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.set_nonblocking(true)?;

        let broadcast_addr = format!("255.255.255.255:{}", config.network.discovery_port);

        let mut packet = [0u8; IDENTITY_SIZE];
        packet[0] = MSG_IDENTITY;
        packet[1] = PROTOCOL_VERSION;
        packet[2] = config.tool_type_id();
        packet[3] = McuState::Idle as u8;

        // Capabilities
        let caps: u16 = match config.identity.tool_type.as_str() {
            "lights" => CAP_LIGHTS | CAP_AXIS,
            _ => CAP_AXIS,
        };
        packet[4..6].copy_from_slice(&caps.to_le_bytes());

        // Serial
        packet[6..10].copy_from_slice(&config.identity.serial.to_le_bytes());

        // Command port
        packet[10..12].copy_from_slice(&config.network.command_port.to_le_bytes());

        // HW/SW version
        packet[12] = 0x01; // hw_rev
        packet[13] = 0;    // sw_major
        packet[14] = 1;    // sw_minor
        packet[15] = 0;    // fault_flags

        Ok(Self {
            socket,
            broadcast_addr,
            identity_packet: packet,
            last_broadcast: Instant::now() - BROADCAST_INTERVAL,
        })
    }

    /// Update state byte in the identity packet.
    pub fn set_state(&mut self, state: McuState) {
        self.identity_packet[3] = state as u8;
    }

    /// Set role status bytes [16-23].
    pub fn set_role_status(&mut self, status: &[u8]) {
        let len = status.len().min(8);
        self.identity_packet[16..16 + len].copy_from_slice(&status[..len]);
    }

    /// Send broadcast if interval has elapsed. Returns true if sent.
    pub fn tick(&mut self) -> bool {
        if self.last_broadcast.elapsed() < BROADCAST_INTERVAL {
            return false;
        }
        self.last_broadcast = Instant::now();

        match self.socket.send_to(&self.identity_packet, &self.broadcast_addr) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
