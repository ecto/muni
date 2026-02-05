//! Role trait and implementations.
//!
//! Each MCU board runs a single role determined by its config file.
//! The role defines how the MCU responds to commands and state changes.

pub mod lights;

/// Rover state values (matches bvrd types::Mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RoverState {
    Disabled = 0,
    Idle = 1,
    Teleop = 2,
    Autonomous = 3,
    EStop = 4,
    Fault = 5,
    Sleep = 6,
}

impl From<u8> for RoverState {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Disabled,
            1 => Self::Idle,
            2 => Self::Teleop,
            3 => Self::Autonomous,
            4 => Self::EStop,
            5 => Self::Fault,
            6 => Self::Sleep,
            _ => Self::Idle,
        }
    }
}

/// Trait for tool role implementations.
pub trait Role {
    /// Handle a tool command packet from bvrd.
    fn handle_command(&mut self, axis: i16, motor: i16, action_a: bool, action_b: bool);

    /// Handle rover state change (for state-linked peripherals).
    fn on_state_change(&mut self, state: RoverState);

    /// Called on watchdog timeout (no commands for 5s).
    fn on_timeout(&mut self);

    /// Called on safe shutdown (no bvrd for 10s).
    fn on_shutdown(&mut self);

    /// Fill status bytes for heartbeat (up to 8 bytes).
    fn status(&self, buf: &mut [u8]);

    /// Called every main loop iteration for animations/PWM updates.
    fn tick(&mut self);
}
