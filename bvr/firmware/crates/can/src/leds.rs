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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_led_mode_repr() {
        assert_eq!(LedMode::Off as u8, 0x00);
        assert_eq!(LedMode::Solid as u8, 0x01);
        assert_eq!(LedMode::Pulse as u8, 0x02);
        assert_eq!(LedMode::Chase as u8, 0x03);
        assert_eq!(LedMode::Flash as u8, 0x04);
        assert_eq!(LedMode::StateLinked as u8, 0x10);
    }

    #[test]
    fn test_led_command_off() {
        let cmd = LedCommand::off();
        assert_eq!(cmd.mode, LedMode::Off);
        assert_eq!(cmd.r, 0);
        assert_eq!(cmd.g, 0);
        assert_eq!(cmd.b, 0);
        assert_eq!(cmd.brightness, 0);
        assert_eq!(cmd.period_ms, 0);
    }

    #[test]
    fn test_led_command_solid() {
        let cmd = LedCommand::solid(255, 128, 64, 200);
        assert_eq!(cmd.mode, LedMode::Solid);
        assert_eq!(cmd.r, 255);
        assert_eq!(cmd.g, 128);
        assert_eq!(cmd.b, 64);
        assert_eq!(cmd.brightness, 200);
        assert_eq!(cmd.period_ms, 0);
    }

    #[test]
    fn test_led_command_pulse() {
        let cmd = LedCommand::pulse(100, 150, 200, 180, 1500);
        assert_eq!(cmd.mode, LedMode::Pulse);
        assert_eq!(cmd.r, 100);
        assert_eq!(cmd.g, 150);
        assert_eq!(cmd.b, 200);
        assert_eq!(cmd.brightness, 180);
        assert_eq!(cmd.period_ms, 1500);
    }

    #[test]
    fn test_led_command_flash() {
        let cmd = LedCommand::flash(255, 0, 0, 255, 200);
        assert_eq!(cmd.mode, LedMode::Flash);
        assert_eq!(cmd.r, 255);
        assert_eq!(cmd.g, 0);
        assert_eq!(cmd.b, 0);
        assert_eq!(cmd.brightness, 255);
        assert_eq!(cmd.period_ms, 200);
    }

    #[test]
    fn test_led_command_to_frame() {
        let cmd = LedCommand::pulse(100, 150, 200, 180, 0x1234);
        let frame = cmd.to_frame();

        assert_eq!(frame.id, can_id::LED_CMD);
        assert!(frame.extended);
        assert_eq!(frame.data.len(), 8);
        assert_eq!(frame.data[0], LedMode::Pulse as u8);
        assert_eq!(frame.data[1], 100); // R
        assert_eq!(frame.data[2], 150); // G
        assert_eq!(frame.data[3], 200); // B
        assert_eq!(frame.data[4], 180); // Brightness
        // Period in little-endian
        assert_eq!(frame.data[5], 0x34);
        assert_eq!(frame.data[6], 0x12);
        assert_eq!(frame.data[7], 0); // Reserved
    }

    #[test]
    fn test_state_commands() {
        // Verify state commands have expected modes
        assert_eq!(LedCommand::state_disabled().mode, LedMode::Off);
        assert_eq!(LedCommand::state_idle().mode, LedMode::Solid);
        assert_eq!(LedCommand::state_teleop().mode, LedMode::Pulse);
        assert_eq!(LedCommand::state_autonomous().mode, LedMode::Pulse);
        assert_eq!(LedCommand::state_estop().mode, LedMode::Flash);
        assert_eq!(LedCommand::state_fault().mode, LedMode::Flash);
    }

    #[test]
    fn test_state_estop_is_red() {
        let cmd = LedCommand::state_estop();
        assert_eq!(cmd.r, 255);
        assert_eq!(cmd.g, 0);
        assert_eq!(cmd.b, 0);
    }

    #[test]
    fn test_state_idle_is_green() {
        let cmd = LedCommand::state_idle();
        assert_eq!(cmd.r, 0);
        assert_eq!(cmd.g, 255);
        assert_eq!(cmd.b, 0);
    }

    #[test]
    fn test_led_status_from_bytes() {
        let data = [0x00, 42];
        let status = LedStatus::from_bytes(&data).unwrap();
        assert_eq!(status.status, 0);
        assert_eq!(status.uptime_secs, 42);
        assert!(status.is_ok());
    }

    #[test]
    fn test_led_status_fault() {
        let data = [0x01, 100];
        let status = LedStatus::from_bytes(&data).unwrap();
        assert_eq!(status.status, 1);
        assert!(!status.is_ok());
    }

    #[test]
    fn test_led_status_short_data() {
        let data = [0x00];
        assert!(LedStatus::from_bytes(&data).is_none());

        let empty: [u8; 0] = [];
        assert!(LedStatus::from_bytes(&empty).is_none());
    }

    #[test]
    fn test_can_ids() {
        assert_eq!(can_id::LED_CMD, 0x0B00);
        assert_eq!(can_id::LED_STATUS, 0x0B01);
    }
}

