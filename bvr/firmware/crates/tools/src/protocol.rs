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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_id_make() {
        // Slot 0, discovery message
        assert_eq!(can_id::make(0, can_id::MSG_DISCOVERY), 0x0A00);

        // Slot 1, command message
        assert_eq!(can_id::make(1, can_id::MSG_COMMAND), 0x0A11);

        // Slot 2, status message
        assert_eq!(can_id::make(2, can_id::MSG_STATUS), 0x0A22);

        // Slot 15 (max), command
        assert_eq!(can_id::make(15, can_id::MSG_COMMAND), 0x0AF1);
    }

    #[test]
    fn test_can_id_parse() {
        // Valid tool IDs
        assert_eq!(can_id::parse(0x0A00), Some((0, 0))); // Slot 0, discovery
        assert_eq!(can_id::parse(0x0A11), Some((1, 1))); // Slot 1, command
        assert_eq!(can_id::parse(0x0A22), Some((2, 2))); // Slot 2, status
        assert_eq!(can_id::parse(0x0AF1), Some((15, 1))); // Slot 15, command

        // Invalid - wrong base
        assert_eq!(can_id::parse(0x0B00), None); // Peripheral range
        assert_eq!(can_id::parse(0x0900), None);
        assert_eq!(can_id::parse(0x0000), None);
    }

    #[test]
    fn test_can_id_roundtrip() {
        for slot in 0..16u8 {
            for msg_type in [can_id::MSG_DISCOVERY, can_id::MSG_COMMAND, can_id::MSG_STATUS] {
                let id = can_id::make(slot, msg_type);
                let parsed = can_id::parse(id);
                assert_eq!(parsed, Some((slot, msg_type)));
            }
        }
    }

    #[test]
    fn test_build_command() {
        let frame = build_command(1, 0.5, -0.75);

        // Check CAN ID
        assert_eq!(frame.id, can_id::make(1, can_id::MSG_COMMAND));
        assert!(frame.extended);

        // Check data format
        assert_eq!(frame.data[0], 0); // Command type

        // Axis: 0.5 * 32767 = 16383
        let axis = i16::from_le_bytes([frame.data[1], frame.data[2]]);
        assert_eq!(axis, 16383);

        // Motor: -0.75 * 32767 = -24575
        let motor = i16::from_le_bytes([frame.data[3], frame.data[4]]);
        assert_eq!(motor, -24575);
    }

    #[test]
    fn test_build_command_clamping() {
        // Values beyond -1.0 to 1.0 should be clamped
        let frame = build_command(0, 2.0, -2.0);

        let axis = i16::from_le_bytes([frame.data[1], frame.data[2]]);
        let motor = i16::from_le_bytes([frame.data[3], frame.data[4]]);

        // 1.0 * 32767 = 32767
        assert_eq!(axis, 32767);
        // -1.0 * 32767 = -32767
        assert_eq!(motor, -32767);
    }

    #[test]
    fn test_build_command_zero() {
        let frame = build_command(0, 0.0, 0.0);

        let axis = i16::from_le_bytes([frame.data[1], frame.data[2]]);
        let motor = i16::from_le_bytes([frame.data[3], frame.data[4]]);

        assert_eq!(axis, 0);
        assert_eq!(motor, 0);
    }

    #[test]
    fn test_discovery_payload_parse() {
        let data = [
            1u8,  // tool_type: SnowAuger
            2,    // protocol_version
            0x03, 0x00, // capabilities: AXIS_CONTROL | MOTOR_CONTROL (little-endian)
            0x78, 0x56, 0x34, 0x12, // serial: 0x12345678 (little-endian)
        ];

        let payload = DiscoveryPayload::parse(&data).unwrap();
        assert_eq!(payload.tool_type, 1);
        assert_eq!(payload.protocol_version, 2);
        assert_eq!(payload.capabilities, 0x0003);
        assert_eq!(payload.serial, 0x12345678);
    }

    #[test]
    fn test_discovery_payload_parse_short_data() {
        let data = [1, 2, 3, 4, 5, 6, 7]; // 7 bytes, need 8
        assert!(DiscoveryPayload::parse(&data).is_none());
    }

    #[test]
    fn test_status_payload_parse() {
        let data = [
            128u8,      // position: 128 (middle)
            0xE8, 0x03, // motor_rpm: 1000 (little-endian)
            0xD0, 0x07, // current_ma: 2000mA (little-endian)
            0x01,       // faults: some fault
        ];

        let payload = StatusPayload::parse(&data).unwrap();
        assert_eq!(payload.position, 128);
        assert_eq!(payload.motor_rpm, 1000);
        assert_eq!(payload.current_ma, 2000);
        assert_eq!(payload.faults, 1);
    }

    #[test]
    fn test_status_payload_parse_short_data() {
        let data = [1, 2, 3, 4, 5]; // 5 bytes, need 6
        assert!(StatusPayload::parse(&data).is_none());
    }

    #[test]
    fn test_status_payload_position_normalized() {
        let mut payload = StatusPayload::default();

        payload.position = 0;
        assert!((payload.position_normalized() - 0.0).abs() < 0.001);

        payload.position = 255;
        assert!((payload.position_normalized() - 1.0).abs() < 0.001);

        payload.position = 128;
        assert!((payload.position_normalized() - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_status_payload_current_amps() {
        let mut payload = StatusPayload::default();

        payload.current_ma = 0;
        assert_eq!(payload.current_amps(), 0.0);

        payload.current_ma = 1000;
        assert!((payload.current_amps() - 1.0).abs() < 0.001);

        payload.current_ma = 2500;
        assert!((payload.current_amps() - 2.5).abs() < 0.001);
    }
}



