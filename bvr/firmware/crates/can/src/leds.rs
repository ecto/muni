//! LED controller CAN protocol.
//!
//! Base rover peripherals use CAN ID range 0x0B00-0x0BFF.

use crate::Frame;

/// CAN IDs for LED controller.
pub mod can_id {
    /// LED command (Jetson -> MCU).
    pub const LED_CMD: u32 = 0x0B00;
    /// LED status/heartbeat (MCU -> Jetson).
    pub const LED_STATUS: u32 = 0x0B01;
}

/// LED modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedMode {
    /// LEDs off.
    Off = 0x00,
    /// Solid color.
    Solid = 0x01,
    /// Pulsing/breathing effect.
    Pulse = 0x02,
    /// Chase/running effect.
    Chase = 0x03,
    /// Flashing/strobe effect.
    Flash = 0x04,
    /// Follow rover state (handled by MCU).
    StateLinked = 0x10,
}

/// LED command to send to the MCU.
#[derive(Debug, Clone, Copy)]
pub struct LedCommand {
    pub mode: LedMode,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
    pub period_ms: u16,
}

impl LedCommand {
    /// Create an "off" command.
    pub fn off() -> Self {
        Self {
            mode: LedMode::Off,
            r: 0,
            g: 0,
            b: 0,
            brightness: 0,
            period_ms: 0,
        }
    }

    /// Create a solid color command.
    pub fn solid(r: u8, g: u8, b: u8, brightness: u8) -> Self {
        Self {
            mode: LedMode::Solid,
            r,
            g,
            b,
            brightness,
            period_ms: 0,
        }
    }

    /// Create a pulsing command.
    pub fn pulse(r: u8, g: u8, b: u8, brightness: u8, period_ms: u16) -> Self {
        Self {
            mode: LedMode::Pulse,
            r,
            g,
            b,
            brightness,
            period_ms,
        }
    }

    /// Create a flashing command.
    pub fn flash(r: u8, g: u8, b: u8, brightness: u8, period_ms: u16) -> Self {
        Self {
            mode: LedMode::Flash,
            r,
            g,
            b,
            brightness,
            period_ms,
        }
    }

    /// Build CAN frame for this command.
    pub fn to_frame(&self) -> Frame {
        let period = self.period_ms.to_le_bytes();
        Frame::new_extended(
            can_id::LED_CMD,
            &[
                self.mode as u8,
                self.r,
                self.g,
                self.b,
                self.brightness,
                period[0],
                period[1],
                0, // Reserved
            ],
        )
    }
}

/// Predefined LED commands for rover states.
impl LedCommand {
    /// Disabled state: off or dim white.
    pub fn state_disabled() -> Self {
        Self::off()
    }

    /// Idle state: solid green.
    pub fn state_idle() -> Self {
        Self::solid(0, 255, 0, 128)
    }

    /// Teleop state: pulsing blue.
    pub fn state_teleop() -> Self {
        Self::pulse(0, 100, 255, 200, 2000)
    }

    /// Autonomous state: pulsing cyan.
    pub fn state_autonomous() -> Self {
        Self::pulse(0, 255, 200, 200, 1500)
    }

    /// E-Stop state: flashing red.
    pub fn state_estop() -> Self {
        Self::flash(255, 0, 0, 255, 200)
    }

    /// Fault state: flashing orange.
    pub fn state_fault() -> Self {
        Self::flash(255, 100, 0, 255, 500)
    }
}

/// Parse LED status from MCU.
#[derive(Debug, Clone, Copy)]
pub struct LedStatus {
    /// 0 = OK, 1 = fault.
    pub status: u8,
    /// Uptime in seconds (wraps at 255).
    pub uptime_secs: u8,
}

impl LedStatus {
    /// Parse from CAN frame data.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }
        Some(Self {
            status: data[0],
            uptime_secs: data[1],
        })
    }

    /// Check if MCU reports OK status.
    pub fn is_ok(&self) -> bool {
        self.status == 0
    }
}

