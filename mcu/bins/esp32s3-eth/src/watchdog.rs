//! Software watchdog for safety timeout.
//!
//! If no command is received within the timeout, the watchdog triggers
//! and the role's `on_timeout()` is called.

use std::time::{Duration, Instant};

/// Default command timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default "safe shutdown" timeout (no bvrd heartbeat at all).
const SAFE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Watchdog {
    last_command: Instant,
    timeout: Duration,
    safe_shutdown_timeout: Duration,
    timed_out: bool,
    shutdown: bool,
}

impl Watchdog {
    pub fn new() -> Self {
        Self {
            last_command: Instant::now(),
            timeout: DEFAULT_TIMEOUT,
            safe_shutdown_timeout: SAFE_SHUTDOWN_TIMEOUT,
            timed_out: false,
            shutdown: false,
        }
    }

    /// Feed the watchdog (call on every received command).
    pub fn feed(&mut self) {
        self.last_command = Instant::now();
        self.timed_out = false;
        self.shutdown = false;
    }

    /// Check watchdog state. Returns (just_timed_out, should_shutdown).
    pub fn check(&mut self) -> (bool, bool) {
        let elapsed = self.last_command.elapsed();

        let just_timed_out = if elapsed >= self.timeout && !self.timed_out {
            self.timed_out = true;
            true
        } else {
            false
        };

        let should_shutdown = if elapsed >= self.safe_shutdown_timeout && !self.shutdown {
            self.shutdown = true;
            true
        } else {
            false
        };

        (just_timed_out, should_shutdown)
    }

    /// Whether we're in timed-out state.
    pub fn is_timed_out(&self) -> bool {
        self.timed_out
    }

    /// Whether we should be in safe shutdown.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }
}
