//! LTE teleop communications for bvr.
//!
//! Handles command reception and telemetry transmission over unreliable links.

use bvr_types::{Command, RoverState, Twist};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum TeleopError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection timeout")]
    Timeout,
}

/// Configuration for teleop.
#[derive(Debug, Clone)]
pub struct TeleopConfig {
    pub listen_port: u16,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
}

impl Default for TeleopConfig {
    fn default() -> Self {
        Self {
            listen_port: 4840,
            heartbeat_interval: Duration::from_millis(100),
            connection_timeout: Duration::from_secs(1),
        }
    }
}

/// Teleop server — runs in async context.
pub struct TeleopServer {
    config: TeleopConfig,
    command_tx: mpsc::Sender<Command>,
    state_rx: watch::Receiver<RoverState>,
}

impl TeleopServer {
    pub fn new(
        config: TeleopConfig,
        command_tx: mpsc::Sender<Command>,
        state_rx: watch::Receiver<RoverState>,
    ) -> Self {
        Self {
            config,
            command_tx,
            state_rx,
        }
    }

    /// Run the teleop server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.listen_port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        info!(addr, "Teleop server listening");

        let mut buf = [0u8; 1024];
        let mut operator_addr: Option<SocketAddr> = None;
        let mut last_recv = std::time::Instant::now();

        loop {
            tokio::select! {
                // Receive commands
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            operator_addr = Some(addr);
                            last_recv = std::time::Instant::now();

                            if let Some(cmd) = Self::parse_command(&buf[..len]) {
                                debug!(?cmd, "Received command");
                                let _ = self.command_tx.send(cmd).await;
                            }
                        }
                        Err(e) => {
                            error!(?e, "Socket receive error");
                        }
                    }
                }

                // Send telemetry
                _ = tokio::time::sleep(self.config.heartbeat_interval) => {
                    if let Some(addr) = operator_addr {
                        let state = self.state_rx.borrow().clone();
                        if let Some(data) = Self::serialize_state(&state) {
                            let _ = socket.send_to(&data, addr).await;
                        }

                        // Check connection timeout
                        if last_recv.elapsed() > self.config.connection_timeout {
                            warn!("Operator connection timeout");
                            let _ = self.command_tx.send(Command::Heartbeat).await;
                        }
                    }
                }
            }
        }
    }

    /// Parse a command from raw bytes.
    fn parse_command(data: &[u8]) -> Option<Command> {
        // Simple binary protocol:
        // [0] = message type
        // [1..] = payload
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0x01 => {
                // Twist command
                if data.len() >= 17 {
                    let linear = f64::from_le_bytes(data[1..9].try_into().ok()?);
                    let angular = f64::from_le_bytes(data[9..17].try_into().ok()?);
                    Some(Command::Twist(Twist { linear, angular }))
                } else {
                    None
                }
            }
            0x02 => Some(Command::EStop),
            0x03 => Some(Command::Heartbeat),
            _ => None,
        }
    }

    /// Serialize rover state for transmission.
    fn serialize_state(state: &RoverState) -> Option<Vec<u8>> {
        // Simple binary encoding
        // In production, use protobuf or similar
        let mut buf = Vec::with_capacity(64);

        buf.push(0x10); // Telemetry message type
        buf.push(state.mode as u8);
        buf.extend_from_slice(&state.power.battery_voltage.to_le_bytes());
        buf.extend_from_slice(&state.timestamp_ms.to_le_bytes());

        Some(buf)
    }
}

/// Simple command sender for testing.
pub async fn send_twist(addr: &str, twist: Twist) -> Result<(), TeleopError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let mut buf = Vec::with_capacity(17);
    buf.push(0x01);
    buf.extend_from_slice(&twist.linear.to_le_bytes());
    buf.extend_from_slice(&twist.angular.to_le_bytes());

    socket.send_to(&buf, addr).await?;
    Ok(())
}

