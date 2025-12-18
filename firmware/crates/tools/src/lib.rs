//! Tool discovery and implementations for bvr.
//!
//! Tools are attachments like snow augers, spreaders, etc. that connect
//! to the rover via CAN bus and are auto-discovered.

pub mod auger;
pub mod discovery;
pub mod protocol;

use bitflags::bitflags;
use types::ToolCommand;

pub use auger::SnowAuger;
pub use discovery::Registry;

bitflags! {
    /// Tool capability flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Capabilities: u16 {
        /// Has a controllable axis (raise/lower)
        const AXIS_CONTROL  = 0x0001;
        /// Has a controllable motor (spin)
        const MOTOR_CONTROL = 0x0002;
        /// Reports position feedback
        const POSITION_FB   = 0x0004;
        /// Reports current draw
        const CURRENT_FB    = 0x0008;
        /// Reports temperature
        const TEMP_FB       = 0x0010;
    }
}

/// Known tool types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ToolType {
    Unknown = 0,
    SnowAuger = 1,
    Spreader = 2,
    Mower = 3,
    Plow = 4,
}

impl From<u8> for ToolType {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::SnowAuger,
            2 => Self::Spreader,
            3 => Self::Mower,
            4 => Self::Plow,
            _ => Self::Unknown,
        }
    }
}

/// Tool metadata from discovery.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub slot: u8,
    pub tool_type: ToolType,
    pub capabilities: Capabilities,
    pub serial: u32,
    pub name: &'static str,
}

/// Tool status for telemetry.
#[derive(Debug, Clone, Default)]
pub struct ToolStatus {
    pub name: &'static str,
    pub position: Option<f32>,
    pub active: bool,
    pub current: Option<f32>,
    pub fault: bool,
}

/// Output command to send to tool MCU.
#[derive(Debug, Clone)]
pub enum ToolOutput {
    None,
    SetAxis(f32),
    SetMotor(f32),
    SetBoth { axis: f32, motor: f32 },
}

/// Trait for tool implementations.
pub trait Tool: Send + Sync {
    /// Get tool info.
    fn info(&self) -> &ToolInfo;

    /// Update with controller input, return command to send.
    fn update(&mut self, input: &ToolCommand) -> ToolOutput;

    /// Process status frame from tool MCU.
    fn handle_status(&mut self, data: &[u8]);

    /// Get current status for telemetry.
    fn status(&self) -> ToolStatus;
}


