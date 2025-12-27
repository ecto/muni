//! CAN protocol constants and message definitions.
//!
//! ID Ranges:
//! - 0x0A00-0x0AFF: Tool attachments (uses discovery)
//! - 0x0B00-0x0BFF: Base rover peripherals (fixed assignment)

/// Base rover peripheral CAN IDs.
pub mod peripheral {
    /// LED controller command (Jetson -> MCU).
    pub const LED_CMD: u32 = 0x0B00;
    /// LED controller status/heartbeat (MCU -> Jetson).
    pub const LED_STATUS: u32 = 0x0B01;
}

/// LED modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
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
    /// Follow rover state (mode determined by bvrd).
    StateLinked = 0x10,
}

impl From<u8> for LedMode {
    fn from(v: u8) -> Self {
        match v {
            0x01 => Self::Solid,
            0x02 => Self::Pulse,
            0x03 => Self::Chase,
            0x04 => Self::Flash,
            0x10 => Self::StateLinked,
            _ => Self::Off,
        }
    }
}

/// LED command parsed from CAN frame.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct LedCommand {
    pub mode: LedMode,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
    pub period_ms: u16,
}

impl LedCommand {
    /// Parse LED command from CAN data bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 7 {
            return None;
        }
        Some(Self {
            mode: LedMode::from(data[0]),
            r: data[1],
            g: data[2],
            b: data[3],
            brightness: data[4],
            period_ms: u16::from_le_bytes([data[5], data[6]]),
        })
    }

    /// Serialize LED command to CAN data bytes.
    pub fn to_bytes(&self) -> [u8; 8] {
        let period = self.period_ms.to_le_bytes();
        [
            self.mode as u8,
            self.r,
            self.g,
            self.b,
            self.brightness,
            period[0],
            period[1],
            0, // Reserved
        ]
    }

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
}

/// Heartbeat status sent from MCU to Jetson.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct HeartbeatStatus {
    /// 0x00 = OK, 0x01 = fault.
    pub status: u8,
    /// Uptime in seconds (wraps at 255).
    pub uptime_secs: u8,
}

impl HeartbeatStatus {
    pub fn ok(uptime_secs: u8) -> Self {
        Self {
            status: 0x00,
            uptime_secs,
        }
    }

    pub fn fault(uptime_secs: u8) -> Self {
        Self {
            status: 0x01,
            uptime_secs,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.status,
            self.uptime_secs,
            0,
            0,
            0,
            0,
            0,
            0, // Reserved
        ]
    }
}

