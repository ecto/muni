//! Software watchdog for command timeout detection.
//!
//! If no valid command is received within the timeout period,
//! the watchdog triggers a safe stop.

use embassy_time::{Duration, Instant};

/// Software watchdog that triggers if not fed within timeout.
pub struct Watchdog {
    timeout: Duration,
    last_feed: Instant,
    triggered: bool,
}

impl Watchdog {
    /// Create a new watchdog with the given timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last_feed: Instant::now(),
            triggered: false,
        }
    }

    /// Feed the watchdog (reset the timer).
    pub fn feed(&mut self) {
        self.last_feed = Instant::now();
        self.triggered = false;
    }

    /// Check if the watchdog has timed out.
    pub fn check(&mut self) -> bool {
        if self.last_feed.elapsed() > self.timeout {
            self.triggered = true;
        }
        self.triggered
    }

    /// Returns true if the watchdog is currently triggered.
    pub fn is_triggered(&self) -> bool {
        self.triggered
    }

    /// Get time remaining until timeout.
    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.last_feed.elapsed();
        if elapsed >= self.timeout {
            Duration::from_millis(0)
        } else {
            self.timeout - elapsed
        }
    }
}

impl defmt::Format for Watchdog {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Watchdog {{ triggered: {}, remaining: {}ms }}",
            self.triggered,
            self.time_remaining().as_millis()
        )
    }
}

