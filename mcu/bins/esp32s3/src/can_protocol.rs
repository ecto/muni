//! CAN Protocol for Muni Attachments
//!
//! Defines CAN message IDs and frame formats for attachment communication.
//! Uses standard 11-bit CAN IDs.
//!
//! ## ID Scheme
//!
//! ```text
//! 0x100-0x1FF: Rover core (drive, power, etc.)
//! 0x200-0x2FF: Attachments
//! 0x300-0x3FF: Sensors
//! 0x400-0x4FF: Reserved
//! ```
//!
//! ## Attachment Messages (base = 0x200 + slot*0x10)
//!
//! | Offset | Direction | Name            | Description                    |
//! |--------|-----------|-----------------|--------------------------------|
//! | +0x00  | A→H       | Heartbeat       | Periodic status beacon         |
//! | +0x01  | H→A       | Identify        | Request attachment info        |
//! | +0x02  | A→H       | Identity        | Attachment type/version        |
//! | +0x03  | H→A       | Command         | Control command                |
//! | +0x04  | A→H       | Ack             | Command acknowledgment         |
//! | +0x05  | A→H       | Sensor          | Sensor data broadcast          |
//! | +0x06  | H→A       | Config          | Configuration update           |
//! | +0x07  | A→H       | Error           | Error/fault report             |
//!
//! A = Attachment, H = Host (bvrd/Jetson)

/// Attachment slot (0-15 supported)
pub const ATTACHMENT_SLOT: u16 = 0;

/// Base CAN ID for this attachment
pub const BASE_ID: u16 = 0x200 + (ATTACHMENT_SLOT * 0x10);

/// CAN Message IDs for this attachment
pub mod msg_id {
    use super::BASE_ID;

    /// Heartbeat: attachment → host (periodic, 1Hz)
    /// Data: [state:u8, uptime_sec:u16, flags:u8, 0, 0, 0, 0]
    pub const HEARTBEAT: u16 = BASE_ID + 0x00;

    /// Identify request: host → attachment
    /// Data: [] (empty)
    pub const IDENTIFY_REQ: u16 = BASE_ID + 0x01;

    /// Identity response: attachment → host
    /// Data: [type:u8, hw_rev:u8, sw_major:u8, sw_minor:u8, caps:u8, 0, 0, 0]
    pub const IDENTITY: u16 = BASE_ID + 0x02;

    /// Command: host → attachment
    /// Data: [cmd:u8, arg0:u8, arg1:u8, arg2:u8, arg3:u8, 0, 0, 0]
    pub const COMMAND: u16 = BASE_ID + 0x03;

    /// Acknowledgment: attachment → host
    /// Data: [cmd:u8, result:u8, 0, 0, 0, 0, 0, 0]
    pub const ACK: u16 = BASE_ID + 0x04;

    /// Sensor data: attachment → host
    /// Data: [sensor_id:u8, data:u8[7]] - format depends on sensor
    pub const SENSOR: u16 = BASE_ID + 0x05;

    /// Config: host → attachment
    /// Data: [param:u8, value:u8[7]]
    pub const CONFIG: u16 = BASE_ID + 0x06;

    /// Error report: attachment → host
    /// Data: [error_code:u8, severity:u8, data:u8[6]]
    pub const ERROR: u16 = BASE_ID + 0x07;
}

/// Attachment types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentType {
    Unknown = 0x00,
    Generic = 0x01,
    LedStrip = 0x02,
    Gripper = 0x03,
    Camera = 0x04,
    Lidar = 0x05,
    Arm = 0x06,
}

/// Attachment state (matches DeviceState in UI)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AttachmentState {
    #[default]
    Idle = 0x00,
    Running = 0x01,
    Error = 0x02,
    Warning = 0x03,
    Disabled = 0x04,
}

/// Command codes
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// No operation / ping
    Nop = 0x00,
    /// Enable attachment
    Enable = 0x01,
    /// Disable attachment
    Disable = 0x02,
    /// Set state: arg0 = AttachmentState
    SetState = 0x03,
    /// Set LED: arg0=r, arg1=g, arg2=b
    SetLed = 0x10,
    /// LED cycle mode: arg0 = 0 off, 1 on
    LedCycle = 0x11,
    /// Set LED timing: arg0 = 0 SK68xx, 1 WS2811
    LedTiming = 0x12,
    /// Set LED color order: arg0 = 0 RGB, 1 GRB, 2 BGR
    LedOrder = 0x13,
}

impl TryFrom<u8> for Command {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Command::Nop),
            0x01 => Ok(Command::Enable),
            0x02 => Ok(Command::Disable),
            0x03 => Ok(Command::SetState),
            0x10 => Ok(Command::SetLed),
            0x11 => Ok(Command::LedCycle),
            0x12 => Ok(Command::LedTiming),
            0x13 => Ok(Command::LedOrder),
            _ => Err(()),
        }
    }
}

/// Capability flags
pub mod caps {
    pub const LED: u8 = 0x01;
    pub const SENSOR: u8 = 0x02;
    pub const ACTUATOR: u8 = 0x04;
    pub const CONFIG: u8 = 0x08;
}

/// Ack result codes
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckResult {
    Ok = 0x00,
    UnknownCommand = 0x01,
    InvalidArgs = 0x02,
    Busy = 0x03,
    Disabled = 0x04,
    Error = 0xFF,
}

/// Error severity
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info = 0x00,
    Warning = 0x01,
    Error = 0x02,
    Critical = 0x03,
}

/// This attachment's configuration
pub const ATTACHMENT_TYPE: AttachmentType = AttachmentType::LedStrip;
pub const HW_REV: u8 = 0x01;
pub const SW_MAJOR: u8 = 0;
pub const SW_MINOR: u8 = 1;
pub const CAPABILITIES: u8 = caps::LED;
