//! Periodic heartbeat broadcast to indicate MCU is alive.

use embassy_time::{Duration, Instant, Ticker};

use crate::protocol::HeartbeatStatus;

/// Heartbeat generator that tracks uptime and status.
pub struct Heartbeat {
    start_time: Instant,
    interval: Duration,
    fault: bool,
}

impl Heartbeat {
    /// Create a new heartbeat with the given broadcast interval.
    pub fn new(interval: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            interval,
            fault: false,
        }
    }

    /// Create with default 500ms interval.
    pub fn default_interval() -> Self {
        Self::new(Duration::from_millis(500))
    }

    /// Set fault state.
    pub fn set_fault(&mut self, fault: bool) {
        self.fault = fault;
    }

    /// Get uptime in seconds (wraps at 255).
    pub fn uptime_secs(&self) -> u8 {
        let secs = self.start_time.elapsed().as_secs();
        (secs % 256) as u8
    }

    /// Get the heartbeat interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Generate current heartbeat status.
    pub fn status(&self) -> HeartbeatStatus {
        if self.fault {
            HeartbeatStatus::fault(self.uptime_secs())
        } else {
            HeartbeatStatus::ok(self.uptime_secs())
        }
    }

    /// Create a ticker for the heartbeat interval.
    pub fn ticker(&self) -> Ticker {
        Ticker::every(self.interval)
    }
}

impl defmt::Format for Heartbeat {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Heartbeat {{ uptime: {}s, fault: {} }}",
            self.uptime_secs(),
            self.fault
        )
    }
}
