//! CAN protocol for tool communication.

use can::{Bus, CanError, Frame};

/// Tool CAN protocol constants.
///
/// Extended CAN IDs: 0x0A00 | (slot << 4) | msg_type
pub mod can_id {
    pub const BASE: u32 = 0x0A00;

    pub const MSG_DISCOVERY: u8 = 0x0;
    pub const MSG_COMMAND: u8 = 0x1;
    pub const MSG_STATUS: u8 = 0x2;

    /// Build CAN ID for a tool message.
    pub fn make(slot: u8, msg_type: u8) -> u32 {
        BASE | ((slot as u32) << 4) | (msg_type as u32)
    }

    /// Parse CAN ID to extract slot and message type.
    pub fn parse(id: u32) -> Option<(u8, u8)> {
        if (id & 0xFF00) != BASE {
            return None;
        }
        let slot = ((id >> 4) & 0x0F) as u8;
        let msg_type = (id & 0x0F) as u8;
        Some((slot, msg_type))
    }
}

/// Build a command frame for a tool.
pub fn build_command(slot: u8, axis: f32, motor: f32) -> Frame {
    let id = can_id::make(slot, can_id::MSG_COMMAND);

    let axis_i16 = (axis.clamp(-1.0, 1.0) * 32767.0) as i16;
    let motor_i16 = (motor.clamp(-1.0, 1.0) * 32767.0) as i16;

    let mut data = [0u8; 8];
    data[0] = 0; // Command type: set values
    data[1..3].copy_from_slice(&axis_i16.to_le_bytes());
    data[3..5].copy_from_slice(&motor_i16.to_le_bytes());

    Frame::new_extended(id, &data)
}

/// Send a command to a tool.
///
/// Command payload:
/// - [0]: Command type (0 = set axis/motor)
/// - [1-2]: Axis value (i16, -32768 to 32767, scaled from -1.0 to 1.0)
/// - [3-4]: Motor value (i16)
/// - [5-7]: Reserved
pub fn send_command(bus: &Bus, slot: u8, axis: f32, motor: f32) -> Result<(), CanError> {
    bus.send(&build_command(slot, axis, motor))
}

/// Discovery frame payload.
#[derive(Debug, Clone)]
pub struct DiscoveryPayload {
    pub tool_type: u8,
    pub protocol_version: u8,
    pub capabilities: u16,
    pub serial: u32,
}

impl DiscoveryPayload {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        Some(Self {
            tool_type: data[0],
            protocol_version: data[1],
            capabilities: u16::from_le_bytes([data[2], data[3]]),
            serial: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        })
    }
}

/// Status frame payload.
#[derive(Debug, Clone, Default)]
pub struct StatusPayload {
    /// Position (0-255, mapped to 0.0-1.0)
    pub position: u8,
    /// Motor RPM
    pub motor_rpm: u16,
    /// Current draw in mA
    pub current_ma: u16,
    /// Fault flags
    pub faults: u8,
}

impl StatusPayload {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }
        Some(Self {
            position: data[0],
            motor_rpm: u16::from_le_bytes([data[1], data[2]]),
            current_ma: u16::from_le_bytes([data[3], data[4]]),
            faults: data[5],
        })
    }

    pub fn position_normalized(&self) -> f32 {
        self.position as f32 / 255.0
    }

    pub fn current_amps(&self) -> f32 {
        self.current_ma as f32 / 1000.0
    }
}

