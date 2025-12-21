//! Discovery client for registering rovers with the Depot discovery service.
//!
//! Rovers register themselves on startup and send periodic heartbeats with
//! their current state. This allows operators to discover and connect to
//! available rovers without manual configuration.

use crate::MetricsSnapshot;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

/// Discovery service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable discovery registration
    pub enabled: bool,

    /// Discovery service endpoint (e.g., "depot.local:4860" or "192.168.1.100:4860")
    pub endpoint: String,

    /// Heartbeat interval in seconds (default: 2)
    #[serde(default = "default_heartbeat_secs")]
    pub heartbeat_secs: u32,

    /// Rover ID
    pub rover_id: String,

    /// Human-readable rover name
    pub rover_name: String,

    /// WebSocket teleop port (for building connection URL)
    pub ws_port: u16,

    /// WebSocket video port
    pub ws_video_port: u16,
}

fn default_heartbeat_secs() -> u32 {
    2
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "depot.local:4860".to_string(),
            heartbeat_secs: 2,
            rover_id: "bvr-01".to_string(),
            rover_name: "Beaver-01".to_string(),
            ws_port: 4850,
            ws_video_port: 4851,
        }
    }
}

/// Discovery errors.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("HTTP request failed: {0}")]
    RequestError(String),

    #[error("Failed to get local IP address")]
    NoLocalAddress,
}

/// Registration payload sent to discovery service.
#[derive(Debug, Serialize)]
struct RegistrationPayload {
    id: String,
    name: String,
    address: String,
    video_address: String,
}

/// Heartbeat payload sent to discovery service.
#[derive(Debug, Serialize)]
struct HeartbeatPayload {
    battery_voltage: f64,
    mode: u8,
    pose: PosePayload,
}

#[derive(Debug, Serialize)]
struct PosePayload {
    x: f64,
    y: f64,
    theta: f64,
}

/// Discovery client that registers with and sends heartbeats to the Depot.
pub struct DiscoveryClient {
    config: DiscoveryConfig,
    client: reqwest::Client,
    base_url: String,
    local_address: Option<String>,
}

impl DiscoveryClient {
    /// Create a new discovery client.
    pub fn new(config: DiscoveryConfig) -> Self {
        let base_url = format!("http://{}", config.endpoint);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        info!(
            endpoint = %config.endpoint,
            rover = %config.rover_id,
            "Discovery client initialized"
        );

        Self {
            config,
            client,
            base_url,
            local_address: None,
        }
    }

    /// Get the local IP address for building connection URLs.
    fn get_local_address(&self) -> Option<String> {
        // Try to determine our local IP by connecting to a remote address
        // This is a common trick to find the outbound interface IP
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            // Try to "connect" to the discovery endpoint to determine our local IP
            if socket.connect(&self.config.endpoint).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }

            // Fallback: try a public address
            if socket.connect("8.8.8.8:53").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }
        }

        None
    }

    /// Register with the discovery service.
    pub async fn register(&mut self) -> Result<(), DiscoveryError> {
        // Determine local address if not cached
        if self.local_address.is_none() {
            self.local_address = self.get_local_address();
        }

        let local_ip = self
            .local_address
            .as_ref()
            .ok_or(DiscoveryError::NoLocalAddress)?;

        let payload = RegistrationPayload {
            id: self.config.rover_id.clone(),
            name: self.config.rover_name.clone(),
            address: format!("ws://{}:{}", local_ip, self.config.ws_port),
            video_address: format!("ws://{}:{}", local_ip, self.config.ws_video_port),
        };

        let url = format!("{}/register", self.base_url);
        debug!(url = %url, rover = %payload.id, "Registering with discovery service");

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!(
                        rover = %self.config.rover_id,
                        address = %payload.address,
                        "Registered with discovery service"
                    );
                    Ok(())
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    warn!(status = %status, body = %body, "Discovery registration failed");
                    Err(DiscoveryError::RequestError(format!(
                        "Status {}: {}",
                        status, body
                    )))
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to reach discovery service");
                Err(DiscoveryError::RequestError(e.to_string()))
            }
        }
    }

    /// Send a heartbeat with current state.
    pub async fn heartbeat(&self, snapshot: &MetricsSnapshot) -> Result<(), DiscoveryError> {
        let payload = HeartbeatPayload {
            battery_voltage: snapshot.battery_voltage,
            mode: snapshot.mode as u8,
            pose: PosePayload {
                x: 0.0, // TODO: Add pose to MetricsSnapshot
                y: 0.0,
                theta: 0.0,
            },
        };

        let url = format!("{}/heartbeat/{}", self.base_url, self.config.rover_id);

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    debug!(rover = %self.config.rover_id, "Heartbeat sent");
                    Ok(())
                } else {
                    debug!(status = %resp.status(), "Heartbeat failed");
                    Err(DiscoveryError::RequestError(format!(
                        "Status {}",
                        resp.status()
                    )))
                }
            }
            Err(e) => {
                debug!(error = %e, "Heartbeat failed");
                Err(DiscoveryError::RequestError(e.to_string()))
            }
        }
    }

    /// Run the discovery loop: register then send periodic heartbeats.
    pub async fn run(mut self, rx: watch::Receiver<MetricsSnapshot>) {
        let heartbeat_interval = Duration::from_secs(self.config.heartbeat_secs as u64);

        // Initial registration with retries
        let mut registered = false;
        for attempt in 1..=5 {
            match self.register().await {
                Ok(()) => {
                    registered = true;
                    break;
                }
                Err(e) => {
                    warn!(
                        attempt,
                        max_attempts = 5,
                        error = %e,
                        "Discovery registration failed, retrying..."
                    );
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }

        if !registered {
            error!("Failed to register with discovery service after 5 attempts");
            // Continue anyway - we'll retry on heartbeat failures
        }

        // Heartbeat loop
        let mut ticker = tokio::time::interval(heartbeat_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!(
            interval_secs = self.config.heartbeat_secs,
            "Starting discovery heartbeat loop"
        );

        let mut consecutive_failures: u32 = 0;

        loop {
            ticker.tick().await;

            // If never registered, keep trying on each tick
            if !registered {
                if self.register().await.is_ok() {
                    registered = true;
                    consecutive_failures = 0;
                }
                continue;
            }

            let snapshot = rx.borrow().clone();

            if self.heartbeat(&snapshot).await.is_err() {
                consecutive_failures += 1;

                // Re-register after several failures
                if consecutive_failures >= 3 {
                    warn!("Multiple heartbeat failures, attempting re-registration");
                    match self.register().await {
                        Ok(()) => consecutive_failures = 0,
                        Err(_) => registered = false, // Will retry registration on next tick
                    }
                }
            } else {
                consecutive_failures = 0;
            }
        }
    }
}
