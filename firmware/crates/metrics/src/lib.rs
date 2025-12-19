//! Real-time metrics push to InfluxDB via UDP line protocol.
//!
//! This crate provides lightweight metrics reporting from rovers to the Depot
//! base station. Metrics are sent as InfluxDB line protocol over UDP at a
//! configurable rate (default 1Hz).
//!
//! UDP is fire-and-forget: if Depot is unreachable, metrics are silently dropped.
//! This ensures network issues don't affect rover operation.

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, error, info, trace, warn};

/// Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Enable metrics push
    pub enabled: bool,

    /// Depot endpoint (e.g., "depot.local:8089" or "192.168.1.100:8089")
    pub endpoint: String,

    /// Push rate in Hz (default: 1)
    #[serde(default = "default_interval_hz")]
    pub interval_hz: u32,

    /// Rover ID (used as tag in metrics)
    pub rover_id: String,
}

fn default_interval_hz() -> u32 {
    1
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "depot.local:8089".to_string(),
            interval_hz: 1,
            rover_id: "bvr-01".to_string(),
        }
    }
}

/// Metrics push errors.
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Failed to resolve endpoint: {0}")]
    ResolveError(String),

    #[error("Failed to create UDP socket: {0}")]
    SocketError(#[from] std::io::Error),
}

/// Snapshot of rover state for metrics reporting.
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Current operating mode
    pub mode: types::Mode,

    /// Battery voltage (V)
    pub battery_voltage: f64,

    /// Total system current (A)
    pub system_current: f64,

    /// Motor temperatures [FL, FR, RL, RR] (°C)
    pub motor_temps: [f32; 4],

    /// Motor currents [FL, FR, RL, RR] (A)
    pub motor_currents: [f32; 4],

    /// Linear velocity (m/s)
    pub velocity_linear: f64,

    /// Angular velocity (rad/s)
    pub velocity_angular: f64,

    /// GPS latitude (degrees, 0 if no fix)
    pub gps_latitude: f64,

    /// GPS longitude (degrees, 0 if no fix)
    pub gps_longitude: f64,

    /// GPS accuracy (meters, 0 if no fix)
    pub gps_accuracy: f32,
}

/// Metrics pusher that sends rover telemetry to Depot.
pub struct MetricsPusher {
    config: Config,
    socket: UdpSocket,
    dest: SocketAddr,
}

impl MetricsPusher {
    /// Create a new metrics pusher.
    pub fn new(config: &Config) -> Result<Self, MetricsError> {
        // Parse endpoint
        let dest: SocketAddr = config
            .endpoint
            .parse()
            .or_else(|_| {
                // Try with default port if not specified
                format!("{}:8089", config.endpoint).parse()
            })
            .map_err(|_| MetricsError::ResolveError(config.endpoint.clone()))?;

        // Create UDP socket (bind to any available port)
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        info!(endpoint = %dest, rover = %config.rover_id, "Metrics pusher initialized");

        Ok(Self {
            config: config.clone(),
            socket,
            dest,
        })
    }

    /// Send a metrics snapshot to Depot.
    pub fn push(&self, snapshot: &MetricsSnapshot) -> Result<(), MetricsError> {
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let rover = &self.config.rover_id;
        let mode = format!("{:?}", snapshot.mode);

        // Build InfluxDB line protocol messages
        // Format: measurement,tag=value field=value timestamp
        let mut lines = Vec::with_capacity(4);

        // Rover status (mode, battery, current)
        lines.push(format!(
            "rover_status,rover={} battery={:.2},current={:.2},mode=\"{}\" {}",
            rover,
            snapshot.battery_voltage,
            snapshot.system_current,
            mode,
            timestamp_ns
        ));

        // Motor data (one line per motor for easier querying)
        const MOTOR_NAMES: [&str; 4] = ["fl", "fr", "rl", "rr"];
        for (i, name) in MOTOR_NAMES.iter().enumerate() {
            lines.push(format!(
                "motors,rover={},motor={} temp={:.1},current={:.2} {}",
                rover, name, snapshot.motor_temps[i], snapshot.motor_currents[i], timestamp_ns
            ));
        }

        // Velocity
        lines.push(format!(
            "velocity,rover={} linear={:.3},angular={:.3} {}",
            rover, snapshot.velocity_linear, snapshot.velocity_angular, timestamp_ns
        ));

        // GPS (only if we have a fix)
        if snapshot.gps_latitude != 0.0 || snapshot.gps_longitude != 0.0 {
            lines.push(format!(
                "gps,rover={} latitude={:.7},longitude={:.7},accuracy={:.1} {}",
                rover,
                snapshot.gps_latitude,
                snapshot.gps_longitude,
                snapshot.gps_accuracy,
                timestamp_ns
            ));
        }

        // Send all lines as a single UDP packet (newline-separated)
        let payload = lines.join("\n");
        trace!(bytes = payload.len(), "Sending metrics");

        match self.socket.send_to(payload.as_bytes(), self.dest) {
            Ok(_) => {
                debug!(lines = lines.len(), "Metrics sent");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Non-blocking socket, just skip this send
                Ok(())
            }
            Err(e) => {
                // Log but don't fail - metrics are best-effort
                warn!(?e, "Failed to send metrics");
                Ok(())
            }
        }
    }

    /// Run the metrics push loop.
    ///
    /// This spawns a background task that reads from a watch channel
    /// and pushes metrics at the configured interval.
    pub async fn run(self, rx: watch::Receiver<MetricsSnapshot>) {
        let interval = Duration::from_secs_f64(1.0 / self.config.interval_hz as f64);
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!(
            hz = self.config.interval_hz,
            endpoint = %self.dest,
            "Metrics push loop started"
        );

        loop {
            ticker.tick().await;

            let snapshot = rx.borrow().clone();
            if let Err(e) = self.push(&snapshot) {
                error!(?e, "Metrics push error");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_protocol_format() {
        let config = Config {
            enabled: true,
            endpoint: "127.0.0.1:8089".to_string(),
            interval_hz: 1,
            rover_id: "test-rover".to_string(),
        };

        let pusher = MetricsPusher::new(&config).unwrap();

        let snapshot = MetricsSnapshot {
            mode: types::Mode::Teleop,
            battery_voltage: 48.5,
            system_current: 12.3,
            motor_temps: [45.0, 46.0, 44.0, 45.5],
            motor_currents: [3.0, 3.1, 2.9, 3.0],
            velocity_linear: 1.5,
            velocity_angular: 0.2,
            gps_latitude: 37.7749,
            gps_longitude: -122.4194,
            gps_accuracy: 2.5,
        };

        // Just verify it doesn't panic
        let _ = pusher.push(&snapshot);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.enabled);
        assert_eq!(config.interval_hz, 1);
    }
}
