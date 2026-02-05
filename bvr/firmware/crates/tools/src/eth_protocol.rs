//! UDP protocol for Ethernet-attached tool MCUs.
//!
//! Two ports:
//! - **4861** (discovery): MCU broadcasts identity/heartbeat every 500ms
//! - **4862** (commands): bvrd sends commands to MCU via unicast
//!
//! All multi-byte fields are little-endian.

use crate::{Capabilities, ToolType};

/// Default discovery broadcast port (MCU → bvrd).
pub const DISCOVERY_PORT: u16 = 4861;

/// Default command unicast port (bvrd → MCU).
pub const COMMAND_PORT: u16 = 4862;

/// Identity/heartbeat packet size.
pub const IDENTITY_SIZE: usize = 24;

/// Command packet size.
pub const COMMAND_SIZE: usize = 16;

// Message types
pub const MSG_IDENTITY: u8 = 0x01;
pub const MSG_SET_STATE: u8 = 0x10;
pub const MSG_TOOL_COMMAND: u8 = 0x11;

/// Protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// MCU state (reported in heartbeat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum McuState {
    Idle = 0,
    Running = 1,
    Error = 2,
    Disabled = 3,
}

impl From<u8> for McuState {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Idle,
            1 => Self::Running,
            2 => Self::Error,
            3 => Self::Disabled,
            _ => Self::Idle,
        }
    }
}

/// Parsed identity/heartbeat packet from MCU (24 bytes).
#[derive(Debug, Clone)]
pub struct EthIdentity {
    pub tool_type: ToolType,
    pub state: McuState,
    pub capabilities: Capabilities,
    pub serial: u32,
    pub command_port: u16,
    pub hw_rev: u8,
    pub sw_major: u8,
    pub sw_minor: u8,
    pub fault_flags: u8,
    pub role_status: [u8; 8],
}

impl EthIdentity {
    /// Parse a 24-byte identity packet.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < IDENTITY_SIZE {
            return None;
        }
        if data[0] != MSG_IDENTITY {
            return None;
        }
        if data[1] != PROTOCOL_VERSION {
            return None;
        }

        Some(Self {
            tool_type: ToolType::from(data[2]),
            state: McuState::from(data[3]),
            capabilities: Capabilities::from_bits_truncate(u16::from_le_bytes([data[4], data[5]])),
            serial: u32::from_le_bytes([data[6], data[7], data[8], data[9]]),
            command_port: u16::from_le_bytes([data[10], data[11]]),
            hw_rev: data[12],
            sw_major: data[13],
            sw_minor: data[14],
            fault_flags: data[15],
            role_status: data[16..24].try_into().unwrap(),
        })
    }

    /// Serialize to 24 bytes (used by MCU firmware).
    pub fn serialize(&self) -> [u8; IDENTITY_SIZE] {
        let mut buf = [0u8; IDENTITY_SIZE];
        buf[0] = MSG_IDENTITY;
        buf[1] = PROTOCOL_VERSION;
        buf[2] = self.tool_type as u8;
        buf[3] = self.state as u8;
        buf[4..6].copy_from_slice(&self.capabilities.bits().to_le_bytes());
        buf[6..10].copy_from_slice(&self.serial.to_le_bytes());
        buf[10..12].copy_from_slice(&self.command_port.to_le_bytes());
        buf[12] = self.hw_rev;
        buf[13] = self.sw_major;
        buf[14] = self.sw_minor;
        buf[15] = self.fault_flags;
        buf[16..24].copy_from_slice(&self.role_status);
        buf
    }
}

/// Command packet sent from bvrd to MCU (16 bytes).
#[derive(Debug, Clone)]
pub struct EthCommand {
    pub msg_type: u8,
    pub flags: u8,
    pub sequence: u16,
    pub timestamp_ms: u32,
    pub payload: [u8; 8],
}

impl EthCommand {
    /// Build a SetState command.
    pub fn set_state(rover_state: u8, sequence: u16, timestamp_ms: u32) -> Self {
        let mut payload = [0u8; 8];
        payload[0] = rover_state;
        Self {
            msg_type: MSG_SET_STATE,
            flags: 0,
            sequence,
            timestamp_ms,
            payload,
        }
    }

    /// Build a ToolCommand (pass-through from teleop).
    pub fn tool_command(
        axis: i16,
        motor: i16,
        action_a: u8,
        action_b: u8,
        sequence: u16,
        timestamp_ms: u32,
    ) -> Self {
        let mut payload = [0u8; 8];
        payload[0..2].copy_from_slice(&axis.to_le_bytes());
        payload[2..4].copy_from_slice(&motor.to_le_bytes());
        payload[4] = action_a;
        payload[5] = action_b;
        Self {
            msg_type: MSG_TOOL_COMMAND,
            flags: 0,
            sequence,
            timestamp_ms,
            payload,
        }
    }

    /// Serialize to 16 bytes.
    pub fn serialize(&self) -> [u8; COMMAND_SIZE] {
        let mut buf = [0u8; COMMAND_SIZE];
        buf[0] = self.msg_type;
        buf[1] = self.flags;
        buf[2..4].copy_from_slice(&self.sequence.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[8..16].copy_from_slice(&self.payload);
        buf
    }

    /// Parse from 16 bytes.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < COMMAND_SIZE {
            return None;
        }
        Some(Self {
            msg_type: data[0],
            flags: data[1],
            sequence: u16::from_le_bytes([data[2], data[3]]),
            timestamp_ms: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            payload: data[8..16].try_into().unwrap(),
        })
    }
}

/// Build a ToolCommand packet from teleop ToolCommand values.
pub fn build_tool_command(
    axis: f32,
    motor: f32,
    action_a: bool,
    action_b: bool,
    sequence: u16,
) -> [u8; COMMAND_SIZE] {
    let axis_i16 = (axis.clamp(-1.0, 1.0) * 32767.0) as i16;
    let motor_i16 = (motor.clamp(-1.0, 1.0) * 32767.0) as i16;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(0);

    EthCommand::tool_command(
        axis_i16,
        motor_i16,
        action_a as u8,
        action_b as u8,
        sequence,
        ts,
    )
    .serialize()
}

/// Build a SetState packet from rover Mode.
pub fn build_set_state(rover_state: u8, sequence: u16) -> [u8; COMMAND_SIZE] {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(0);

    EthCommand::set_state(rover_state, sequence, ts).serialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_roundtrip() {
        let identity = EthIdentity {
            tool_type: ToolType::Lights,
            state: McuState::Running,
            capabilities: Capabilities::LIGHTS | Capabilities::AXIS_CONTROL,
            serial: 0x12345678,
            command_port: 4862,
            hw_rev: 1,
            sw_major: 0,
            sw_minor: 1,
            fault_flags: 0,
            role_status: [0xFF, 0x80, 0, 0, 0, 0, 0, 0],
        };

        let bytes = identity.serialize();
        assert_eq!(bytes.len(), IDENTITY_SIZE);

        let parsed = EthIdentity::parse(&bytes).unwrap();
        assert_eq!(parsed.tool_type, ToolType::Lights);
        assert_eq!(parsed.state, McuState::Running);
        assert!(parsed.capabilities.contains(Capabilities::LIGHTS));
        assert!(parsed.capabilities.contains(Capabilities::AXIS_CONTROL));
        assert_eq!(parsed.serial, 0x12345678);
        assert_eq!(parsed.command_port, 4862);
        assert_eq!(parsed.hw_rev, 1);
        assert_eq!(parsed.sw_major, 0);
        assert_eq!(parsed.sw_minor, 1);
        assert_eq!(parsed.fault_flags, 0);
        assert_eq!(parsed.role_status[0], 0xFF);
        assert_eq!(parsed.role_status[1], 0x80);
    }

    #[test]
    fn test_identity_parse_wrong_msg_type() {
        let mut bytes = [0u8; IDENTITY_SIZE];
        bytes[0] = 0xFF; // wrong type
        bytes[1] = PROTOCOL_VERSION;
        assert!(EthIdentity::parse(&bytes).is_none());
    }

    #[test]
    fn test_identity_parse_wrong_version() {
        let mut bytes = [0u8; IDENTITY_SIZE];
        bytes[0] = MSG_IDENTITY;
        bytes[1] = 99; // wrong version
        assert!(EthIdentity::parse(&bytes).is_none());
    }

    #[test]
    fn test_identity_parse_too_short() {
        let bytes = [0u8; 10];
        assert!(EthIdentity::parse(&bytes).is_none());
    }

    #[test]
    fn test_command_roundtrip() {
        let cmd = EthCommand::tool_command(16383, -24575, 1, 0, 42, 1000);
        let bytes = cmd.serialize();
        assert_eq!(bytes.len(), COMMAND_SIZE);

        let parsed = EthCommand::parse(&bytes).unwrap();
        assert_eq!(parsed.msg_type, MSG_TOOL_COMMAND);
        assert_eq!(parsed.sequence, 42);
        assert_eq!(parsed.timestamp_ms, 1000);

        // Verify payload
        let axis = i16::from_le_bytes([parsed.payload[0], parsed.payload[1]]);
        let motor = i16::from_le_bytes([parsed.payload[2], parsed.payload[3]]);
        assert_eq!(axis, 16383);
        assert_eq!(motor, -24575);
        assert_eq!(parsed.payload[4], 1); // action_a
        assert_eq!(parsed.payload[5], 0); // action_b
    }

    #[test]
    fn test_set_state_command() {
        let cmd = EthCommand::set_state(2, 1, 5000); // Teleop state
        let bytes = cmd.serialize();

        let parsed = EthCommand::parse(&bytes).unwrap();
        assert_eq!(parsed.msg_type, MSG_SET_STATE);
        assert_eq!(parsed.payload[0], 2); // Teleop
        assert_eq!(parsed.sequence, 1);
    }

    #[test]
    fn test_command_parse_too_short() {
        let bytes = [0u8; 8];
        assert!(EthCommand::parse(&bytes).is_none());
    }

    #[test]
    fn test_build_tool_command_clamping() {
        let bytes = build_tool_command(2.0, -2.0, true, false, 0);
        let cmd = EthCommand::parse(&bytes).unwrap();
        let axis = i16::from_le_bytes([cmd.payload[0], cmd.payload[1]]);
        let motor = i16::from_le_bytes([cmd.payload[2], cmd.payload[3]]);
        assert_eq!(axis, 32767);  // clamped to 1.0
        assert_eq!(motor, -32767); // clamped to -1.0
    }

    #[test]
    fn test_mcu_state_from_u8() {
        assert_eq!(McuState::from(0), McuState::Idle);
        assert_eq!(McuState::from(1), McuState::Running);
        assert_eq!(McuState::from(2), McuState::Error);
        assert_eq!(McuState::from(3), McuState::Disabled);
        assert_eq!(McuState::from(99), McuState::Idle); // unknown → Idle
    }
}
