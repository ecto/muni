//! Ethernet tool discovery via UDP broadcast.
//!
//! Listens for MCU identity/heartbeat broadcasts on port 4861.
//! Tracks online MCUs and implements hot-plug/unplug detection.

use crate::eth_protocol::{self, EthIdentity, IDENTITY_SIZE};
use crate::ToolType;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

/// Timeout before marking an MCU as offline (no heartbeat).
const OFFLINE_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout before deregistering an offline MCU entirely.
const DEREGISTER_TIMEOUT: Duration = Duration::from_secs(15);

/// A discovered Ethernet-attached MCU.
#[derive(Debug, Clone)]
pub struct EthMcu {
    /// Source address of the MCU (IP from received packet).
    pub addr: SocketAddr,
    /// Port the MCU listens on for commands.
    pub command_port: u16,
    /// Last received identity.
    pub identity: EthIdentity,
    /// When we last received a heartbeat.
    pub last_seen: Instant,
    /// Whether the MCU is considered online.
    pub online: bool,
}

impl EthMcu {
    /// Get the command address (MCU IP + command port).
    pub fn command_addr(&self) -> SocketAddr {
        SocketAddr::new(self.addr.ip(), self.command_port)
    }
}

/// Shared state for discovered MCUs.
#[derive(Debug, Default)]
pub struct EthToolState {
    /// MCUs keyed by serial number.
    pub mcus: HashMap<u32, EthMcu>,
}

/// Ethernet tool discovery service.
pub struct EthDiscovery {
    /// Shared state with discovered MCUs.
    state: Arc<Mutex<EthToolState>>,
    /// Port to listen on for discovery broadcasts.
    #[allow(dead_code)]
    discovery_port: u16,
    /// UDP socket for sending commands (bound lazily).
    command_socket: Option<Arc<UdpSocket>>,
    /// Sequence counter for outgoing commands.
    sequence: u16,
}

impl EthDiscovery {
    pub fn new(discovery_port: u16) -> Self {
        Self {
            state: Arc::new(Mutex::new(EthToolState::default())),
            discovery_port,
            command_socket: None,
            sequence: 0,
        }
    }

    /// Get shared state handle (for reading MCU info from other tasks).
    pub fn state(&self) -> Arc<Mutex<EthToolState>> {
        self.state.clone()
    }

    /// Next sequence number.
    fn next_seq(&mut self) -> u16 {
        let s = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        s
    }

    /// Send a SetState command to ALL online MCUs.
    pub async fn broadcast_state(&mut self, rover_state: u8) {
        let seq = self.next_seq();
        let packet = eth_protocol::build_set_state(rover_state, seq);

        let targets: Vec<SocketAddr> = {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state
                .mcus
                .values()
                .filter(|m| m.online)
                .map(|m| m.command_addr())
                .collect()
        };

        if targets.is_empty() {
            return;
        }

        let socket = match self.ensure_command_socket().await {
            Some(s) => s,
            None => return,
        };

        for addr in targets {
            if let Err(e) = socket.send_to(&packet, addr).await {
                debug!(?e, %addr, "Failed to send SetState to MCU");
            }
        }
    }

    /// Send a ToolCommand to a specific MCU by serial.
    pub async fn send_tool_command(
        &mut self,
        serial: u32,
        axis: f32,
        motor: f32,
        action_a: bool,
        action_b: bool,
    ) -> Option<()> {
        let addr = {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let mcu = state.mcus.get(&serial)?;
            if !mcu.online {
                return None;
            }
            mcu.command_addr()
        };

        let seq = self.next_seq();
        let packet = eth_protocol::build_tool_command(axis, motor, action_a, action_b, seq);

        let socket = self.ensure_command_socket().await?;
        socket.send_to(&packet, addr).await.ok()?;
        Some(())
    }

    /// Ensure we have a command socket ready.
    async fn ensure_command_socket(&mut self) -> Option<Arc<UdpSocket>> {
        if self.command_socket.is_none() {
            match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => self.command_socket = Some(Arc::new(s)),
                Err(e) => {
                    warn!(?e, "Failed to bind UDP command socket");
                    return None;
                }
            }
        }
        self.command_socket.clone()
    }

    /// Run the discovery listener (call this as a tokio task).
    ///
    /// Listens for MCU broadcasts and maintains the online/offline state.
    pub async fn run(state: Arc<Mutex<EthToolState>>, discovery_port: u16) {
        let bind_addr = format!("0.0.0.0:{}", discovery_port);
        let socket = match UdpSocket::bind(&bind_addr).await {
            Ok(s) => {
                info!(port = discovery_port, "Eth tool discovery listening");
                s
            }
            Err(e) => {
                warn!(?e, port = discovery_port, "Failed to bind discovery socket");
                return;
            }
        };

        let mut buf = [0u8; 128];
        let mut gc_interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            if len >= IDENTITY_SIZE {
                                if let Some(identity) = EthIdentity::parse(&buf[..len]) {
                                    Self::handle_identity(&state, addr, identity);
                                }
                            }
                        }
                        Err(e) => {
                            warn!(?e, "Discovery socket recv error");
                        }
                    }
                }
                _ = gc_interval.tick() => {
                    Self::gc_stale(&state);
                }
            }
        }
    }

    /// Handle a received identity packet.
    fn handle_identity(state: &Arc<Mutex<EthToolState>>, addr: SocketAddr, identity: EthIdentity) {
        let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
        let serial = identity.serial;

        if let Some(mcu) = state.mcus.get_mut(&serial) {
            // Update existing MCU
            let was_offline = !mcu.online;
            mcu.last_seen = Instant::now();
            mcu.online = true;
            mcu.identity = identity;
            mcu.addr = addr;

            if was_offline {
                info!(serial, %addr, "Eth MCU re-discovered (hot-plug)");
            }
        } else {
            // New MCU discovered
            info!(
                serial,
                tool_type = ?identity.tool_type,
                %addr,
                command_port = identity.command_port,
                "Eth MCU discovered"
            );

            state.mcus.insert(serial, EthMcu {
                addr,
                command_port: identity.command_port,
                identity,
                last_seen: Instant::now(),
                online: true,
            });
        }
    }

    /// Garbage-collect stale MCUs.
    fn gc_stale(state: &Arc<Mutex<EthToolState>>) {
        let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        // Collect serials to deregister (can't modify while iterating)
        let mut to_remove = Vec::new();

        for (serial, mcu) in state.mcus.iter_mut() {
            let elapsed = now.duration_since(mcu.last_seen);

            if elapsed >= DEREGISTER_TIMEOUT {
                to_remove.push(*serial);
            } else if elapsed >= OFFLINE_TIMEOUT && mcu.online {
                info!(serial, "Eth MCU offline (heartbeat timeout)");
                mcu.online = false;
            }
        }

        for serial in to_remove {
            info!(serial, "Eth MCU deregistered (prolonged offline)");
            state.mcus.remove(&serial);
        }
    }

    /// Find the first online MCU with a given tool type.
    pub fn find_by_type(&self, tool_type: ToolType) -> Option<(u32, SocketAddr)> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .mcus
            .iter()
            .find(|(_, m)| m.online && m.identity.tool_type == tool_type)
            .map(|(serial, m)| (*serial, m.command_addr()))
    }

    /// Get all online MCUs.
    pub fn online_mcus(&self) -> Vec<(u32, ToolType, SocketAddr)> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .mcus
            .iter()
            .filter(|(_, m)| m.online)
            .map(|(serial, m)| (*serial, m.identity.tool_type, m.command_addr()))
            .collect()
    }

    /// Get the number of online MCUs.
    pub fn online_count(&self) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.mcus.values().filter(|m| m.online).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Capabilities;

    fn make_identity(serial: u32, tool_type: ToolType) -> EthIdentity {
        EthIdentity {
            tool_type,
            state: eth_protocol::McuState::Running,
            capabilities: Capabilities::LIGHTS,
            serial,
            command_port: 4862,
            hw_rev: 1,
            sw_major: 0,
            sw_minor: 1,
            fault_flags: 0,
            role_status: [0; 8],
        }
    }

    #[test]
    fn test_handle_identity_new_mcu() {
        let state = Arc::new(Mutex::new(EthToolState::default()));
        let addr: SocketAddr = "10.0.0.50:4861".parse().unwrap();
        let identity = make_identity(0x0001, ToolType::Lights);

        EthDiscovery::handle_identity(&state, addr, identity);

        let s = state.lock().unwrap();
        assert_eq!(s.mcus.len(), 1);
        let mcu = s.mcus.get(&0x0001).unwrap();
        assert!(mcu.online);
        assert_eq!(mcu.identity.tool_type, ToolType::Lights);
        assert_eq!(mcu.command_addr(), "10.0.0.50:4862".parse().unwrap());
    }

    #[test]
    fn test_handle_identity_update_existing() {
        let state = Arc::new(Mutex::new(EthToolState::default()));
        let addr: SocketAddr = "10.0.0.50:4861".parse().unwrap();

        // First discovery
        EthDiscovery::handle_identity(&state, addr, make_identity(0x0001, ToolType::Lights));

        // Mark offline manually
        {
            let mut s = state.lock().unwrap();
            s.mcus.get_mut(&0x0001).unwrap().online = false;
        }

        // Re-discover
        EthDiscovery::handle_identity(&state, addr, make_identity(0x0001, ToolType::Lights));

        let s = state.lock().unwrap();
        assert!(s.mcus.get(&0x0001).unwrap().online);
    }

    #[test]
    fn test_gc_stale_offline() {
        let state = Arc::new(Mutex::new(EthToolState::default()));
        let addr: SocketAddr = "10.0.0.50:4861".parse().unwrap();
        EthDiscovery::handle_identity(&state, addr, make_identity(0x0001, ToolType::Lights));

        // Backdate last_seen
        {
            let mut s = state.lock().unwrap();
            s.mcus.get_mut(&0x0001).unwrap().last_seen =
                Instant::now() - OFFLINE_TIMEOUT - Duration::from_millis(100);
        }

        EthDiscovery::gc_stale(&state);

        let s = state.lock().unwrap();
        assert!(!s.mcus.get(&0x0001).unwrap().online);
    }

    #[test]
    fn test_gc_stale_deregister() {
        let state = Arc::new(Mutex::new(EthToolState::default()));
        let addr: SocketAddr = "10.0.0.50:4861".parse().unwrap();
        EthDiscovery::handle_identity(&state, addr, make_identity(0x0001, ToolType::Lights));

        // Backdate beyond deregister timeout
        {
            let mut s = state.lock().unwrap();
            s.mcus.get_mut(&0x0001).unwrap().last_seen =
                Instant::now() - DEREGISTER_TIMEOUT - Duration::from_millis(100);
        }

        EthDiscovery::gc_stale(&state);

        let s = state.lock().unwrap();
        assert!(s.mcus.is_empty());
    }

    #[test]
    fn test_multiple_mcus() {
        let state = Arc::new(Mutex::new(EthToolState::default()));

        EthDiscovery::handle_identity(
            &state,
            "10.0.0.50:4861".parse().unwrap(),
            make_identity(0x0001, ToolType::Lights),
        );
        EthDiscovery::handle_identity(
            &state,
            "10.0.0.51:4861".parse().unwrap(),
            make_identity(0x0002, ToolType::SnowAuger),
        );

        let s = state.lock().unwrap();
        assert_eq!(s.mcus.len(), 2);
    }
}
