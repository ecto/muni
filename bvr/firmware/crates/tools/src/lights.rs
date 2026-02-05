//! Ethernet headlight tool implementation.
//!
//! Controls headlights and LED strips via an Ethernet-attached MCU.
//! The MCU runs the lights role firmware and responds to ToolCommand
//! and SetState packets over UDP.
//!
//! State-linked behavior (implemented on MCU side):
//! - Idle/Sleep/Disabled → lights off
//! - Teleop/Autonomous → lights on (full brightness)
//! - EStop → flash 200ms
//! - Fault → flash 500ms

use crate::eth_protocol::EthCommand;
use crate::{Capabilities, Tool, ToolInfo, ToolOutput, ToolStatus, ToolType};
use std::net::SocketAddr;
use types::ToolCommand;

/// Ethernet headlight tool.
///
/// Controls:
/// - Axis: Brightness (-1.0=off, 1.0=full)
/// - Action A: Toggle on/off
pub struct EthHeadlights {
    info: ToolInfo,
    /// MCU serial number.
    serial: u32,
    /// Command address of the MCU.
    command_addr: SocketAddr,
    /// Whether lights are toggled on.
    active: bool,
    /// Current brightness (0.0-1.0).
    brightness: f32,
    /// Debounce for action_a.
    action_a_prev: bool,
    /// Sequence counter for commands.
    sequence: u16,
}

impl EthHeadlights {
    pub fn new(serial: u32, command_addr: SocketAddr) -> Self {
        Self {
            info: ToolInfo {
                slot: 0xE0, // Ethernet tools use high slot range
                tool_type: ToolType::Lights,
                capabilities: Capabilities::LIGHTS | Capabilities::AXIS_CONTROL,
                serial,
                name: "Headlights",
            },
            serial,
            command_addr,
            active: false,
            brightness: 1.0,
            action_a_prev: false,
            sequence: 0,
        }
    }

    /// Get the MCU serial.
    pub fn serial(&self) -> u32 {
        self.serial
    }

    /// Get the command address.
    pub fn command_addr(&self) -> SocketAddr {
        self.command_addr
    }

    fn next_seq(&mut self) -> u16 {
        let s = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        s
    }
}

impl Tool for EthHeadlights {
    fn info(&self) -> &ToolInfo {
        &self.info
    }

    fn update(&mut self, input: &ToolCommand) -> ToolOutput {
        // Toggle on action_a rising edge
        if input.action_a && !self.action_a_prev {
            self.active = !self.active;
        }
        self.action_a_prev = input.action_a;

        // Map axis to brightness (0-1 range, ignore negative)
        if input.axis > 0.0 {
            self.brightness = input.axis.clamp(0.0, 1.0);
        }

        // Build UDP command packet
        let effective_brightness = if self.active { self.brightness } else { 0.0 };
        let seq = self.next_seq();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u32)
            .unwrap_or(0);

        // Use tool command format: axis=brightness, action_a=toggle state
        let axis_i16 = (effective_brightness * 32767.0) as i16;
        let cmd = EthCommand::tool_command(
            axis_i16,
            0,
            self.active as u8,
            0,
            seq,
            ts,
        );

        ToolOutput::Udp {
            addr: self.command_addr,
            data: cmd.serialize().to_vec(),
        }
    }

    fn handle_status(&mut self, data: &[u8]) {
        // Parse role_status from heartbeat if available
        if data.len() >= 2 {
            // role_status[0] = mode byte from MCU
            // role_status[1] = brightness byte from MCU
            self.brightness = data[1] as f32 / 255.0;
        }
    }

    fn status(&self) -> ToolStatus {
        ToolStatus {
            name: self.info.name,
            position: Some(self.brightness),
            active: self.active,
            current: None,
            fault: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth_protocol;

    fn make_headlights() -> EthHeadlights {
        EthHeadlights::new(0x0001, "10.0.0.50:4862".parse().unwrap())
    }

    #[test]
    fn test_info() {
        let h = make_headlights();
        assert_eq!(h.info().tool_type, ToolType::Lights);
        assert_eq!(h.info().name, "Headlights");
        assert!(h.info().capabilities.contains(Capabilities::LIGHTS));
    }

    #[test]
    fn test_toggle_on_action_a() {
        let mut h = make_headlights();
        assert!(!h.active);

        // Rising edge toggles on
        h.update(&ToolCommand {
            axis: 1.0,
            action_a: true,
            ..Default::default()
        });
        assert!(h.active);

        // Held down — no change
        h.update(&ToolCommand {
            axis: 1.0,
            action_a: true,
            ..Default::default()
        });
        assert!(h.active);

        // Release then press again — toggles off
        h.update(&ToolCommand {
            axis: 1.0,
            action_a: false,
            ..Default::default()
        });
        h.update(&ToolCommand {
            axis: 1.0,
            action_a: true,
            ..Default::default()
        });
        assert!(!h.active);
    }

    #[test]
    fn test_output_is_udp() {
        let mut h = make_headlights();
        // Toggle on
        let output = h.update(&ToolCommand {
            axis: 0.5,
            action_a: true,
            ..Default::default()
        });
        match output {
            ToolOutput::Udp { addr, data } => {
                assert_eq!(addr, "10.0.0.50:4862".parse::<SocketAddr>().unwrap());
                assert_eq!(data.len(), eth_protocol::COMMAND_SIZE);
            }
            _ => panic!("Expected Udp output"),
        }
    }

    #[test]
    fn test_brightness_from_axis() {
        let mut h = make_headlights();
        h.active = true;

        h.update(&ToolCommand {
            axis: 0.75,
            ..Default::default()
        });
        assert!((h.brightness - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_status() {
        let mut h = make_headlights();
        h.active = true;
        h.brightness = 0.8;
        let s = h.status();
        assert_eq!(s.name, "Headlights");
        assert!(s.active);
        assert!((s.position.unwrap() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_serial() {
        let h = make_headlights();
        assert_eq!(h.serial(), 0x0001);
    }
}
