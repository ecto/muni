//! Control Loop Crate
//!
//! Modular control loop implementation for bvrd. Orchestrates:
//! - Command processing from teleop
//! - Autonomous navigation with policy inference
//! - Motor control with rate limiting
//! - Telemetry collection and broadcasting
//! - Dispatch task management
//!
//! # Architecture
//!
//! The control loop runs at 100Hz and coordinates multiple subsystems:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Control Loop                             │
//! │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐   │
//! │  │  Command    │  │  Autonomous  │  │   Motor Control     │   │
//! │  │  Processor  │─▶│  Controller  │─▶│  (rate limit, CAN)  │   │
//! │  └─────────────┘  └──────────────┘  └─────────────────────┘   │
//! │         │                │                     │               │
//! │         ▼                ▼                     ▼               │
//! │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐   │
//! │  │  Dispatch   │  │  Telemetry   │  │   State Machine     │   │
//! │  │  Handler    │  │  Builder     │  │   (mode, LEDs)      │   │
//! │  └─────────────┘  └──────────────┘  └─────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

mod command_processor;
mod telemetry_builder;

pub use command_processor::{CommandProcessor, ProcessedCommands};
pub use telemetry_builder::TelemetryBuilder;

use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur in the control loop
#[derive(Error, Debug)]
pub enum ControlLoopError {
    #[error("CAN bus error: {0}")]
    CanError(String),
    #[error("State lock poisoned")]
    LockPoisoned,
    #[error("Channel send error")]
    ChannelError,
}

/// Control loop configuration
#[derive(Debug, Clone)]
pub struct ControlLoopConfig {
    /// Control loop period (default: 10ms = 100Hz)
    pub control_period: Duration,
    /// Watchdog timeout for command absence (default: 500ms)
    pub watchdog_timeout: Duration,
    /// Maximum linear velocity for autonomous mode (m/s)
    pub autonomous_max_linear: f64,
    /// Maximum angular velocity for autonomous mode (rad/s)
    pub autonomous_max_angular: f64,
    /// CAN error backoff threshold
    pub can_error_backoff_threshold: u32,
    /// CAN retry interval (in loop iterations)
    pub can_retry_interval: u64,
}

impl Default for ControlLoopConfig {
    fn default() -> Self {
        Self {
            control_period: Duration::from_millis(10),
            watchdog_timeout: Duration::from_millis(500),
            autonomous_max_linear: 1.0,
            autonomous_max_angular: 1.0,
            can_error_backoff_threshold: 10,
            can_retry_interval: 100,
        }
    }
}

/// Statistics tracked by the control loop
#[derive(Debug, Clone, Default)]
pub struct ControlLoopStats {
    /// Total loop iterations
    pub loop_count: u64,
    /// Total CAN errors
    pub can_error_count: u64,
    /// Consecutive CAN errors (for backoff)
    pub consecutive_can_errors: u32,
    /// Whether CAN bus is currently in backoff mode
    pub can_backoff_active: bool,
    /// Count of slow policy inferences
    pub policy_slow_count: u32,
}

impl ControlLoopStats {
    /// Record a successful CAN transmission
    pub fn record_can_success(&mut self) {
        self.consecutive_can_errors = 0;
        self.can_backoff_active = false;
    }

    /// Record a CAN transmission error
    pub fn record_can_error(&mut self, backoff_threshold: u32) {
        self.can_error_count += 1;
        self.consecutive_can_errors = self.consecutive_can_errors.saturating_add(1);
        if self.consecutive_can_errors >= backoff_threshold {
            self.can_backoff_active = true;
        }
    }

    /// Check if we should skip CAN transmission due to backoff
    pub fn should_skip_can(&self, backoff_threshold: u32, retry_interval: u64) -> bool {
        self.consecutive_can_errors >= backoff_threshold
            && self.loop_count % retry_interval != 0
    }
}

/// Timing information for the control loop
#[derive(Debug)]
pub struct LoopTiming {
    last_tick: Instant,
    control_period: Duration,
}

impl LoopTiming {
    pub fn new(control_period: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            control_period,
        }
    }

    /// Wait for the next tick and return the elapsed time since last tick
    pub fn wait_for_tick(&mut self) -> f64 {
        let elapsed = self.last_tick.elapsed();
        if elapsed < self.control_period {
            std::thread::sleep(self.control_period - elapsed);
        }
        let dt = self.last_tick.elapsed().as_secs_f64();
        self.last_tick = Instant::now();
        dt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ControlLoopConfig::default();
        assert_eq!(config.control_period.as_millis(), 10);
        assert_eq!(config.watchdog_timeout.as_millis(), 500);
    }

    #[test]
    fn test_stats_can_tracking() {
        let mut stats = ControlLoopStats::default();

        // Record some errors
        for _ in 0..5 {
            stats.record_can_error(10);
        }
        assert_eq!(stats.consecutive_can_errors, 5);
        assert!(!stats.can_backoff_active);

        // More errors trigger backoff
        for _ in 0..5 {
            stats.record_can_error(10);
        }
        assert_eq!(stats.consecutive_can_errors, 10);
        assert!(stats.can_backoff_active);

        // Success resets
        stats.record_can_success();
        assert_eq!(stats.consecutive_can_errors, 0);
        assert!(!stats.can_backoff_active);
    }

    #[test]
    fn test_loop_timing() {
        let timing = LoopTiming::new(Duration::from_millis(10));
        assert_eq!(timing.control_period.as_millis(), 10);
    }
}
