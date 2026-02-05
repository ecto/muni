//! Tool discovery and implementations for bvr.
//!
//! Tools are attachments like snow augers, spreaders, etc. that connect
//! to the rover via CAN bus and are auto-discovered.

pub mod auger;
pub mod blower;
pub mod discovery;
pub mod eth_discovery;
pub mod eth_protocol;
pub mod lights;
pub mod protocol;

use bitflags::bitflags;
use types::ToolCommand;

pub use auger::SnowAuger;
pub use blower::Blower;
pub use discovery::Registry;
pub use eth_discovery::EthDiscovery;
pub use lights::EthHeadlights;

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
        /// Has controllable lights (headlights, LED strips)
        const LIGHTS        = 0x0020;
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
    Blower = 5,
    Lights = 6,
    Sensor = 7,
}

impl From<u8> for ToolType {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::SnowAuger,
            2 => Self::Spreader,
            3 => Self::Mower,
            4 => Self::Plow,
            5 => Self::Blower,
            6 => Self::Lights,
            7 => Self::Sensor,
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

/// Output command to send to tool hardware.
#[derive(Debug, Clone)]
pub enum ToolOutput {
    None,
    SetAxis(f32),
    SetMotor(f32),
    SetBoth { axis: f32, motor: f32 },
    /// Raw CAN frame (for VESC-based tools that bypass tool protocol).
    Raw(can::Frame),
    /// UDP packet to send to an Ethernet-attached tool MCU.
    Udp {
        addr: std::net::SocketAddr,
        data: Vec<u8>,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_type_from_u8() {
        assert_eq!(ToolType::from(0), ToolType::Unknown);
        assert_eq!(ToolType::from(1), ToolType::SnowAuger);
        assert_eq!(ToolType::from(2), ToolType::Spreader);
        assert_eq!(ToolType::from(3), ToolType::Mower);
        assert_eq!(ToolType::from(4), ToolType::Plow);
        assert_eq!(ToolType::from(5), ToolType::Blower);
        assert_eq!(ToolType::from(6), ToolType::Lights);
        assert_eq!(ToolType::from(7), ToolType::Sensor);
        assert_eq!(ToolType::from(8), ToolType::Unknown);
        assert_eq!(ToolType::from(255), ToolType::Unknown);
    }

    #[test]
    fn test_tool_type_repr() {
        assert_eq!(ToolType::Unknown as u8, 0);
        assert_eq!(ToolType::SnowAuger as u8, 1);
        assert_eq!(ToolType::Spreader as u8, 2);
        assert_eq!(ToolType::Mower as u8, 3);
        assert_eq!(ToolType::Plow as u8, 4);
        assert_eq!(ToolType::Blower as u8, 5);
        assert_eq!(ToolType::Lights as u8, 6);
        assert_eq!(ToolType::Sensor as u8, 7);
    }

    #[test]
    fn test_capabilities_individual_flags() {
        let axis = Capabilities::AXIS_CONTROL;
        assert!(axis.contains(Capabilities::AXIS_CONTROL));
        assert!(!axis.contains(Capabilities::MOTOR_CONTROL));

        let motor = Capabilities::MOTOR_CONTROL;
        assert!(motor.contains(Capabilities::MOTOR_CONTROL));
        assert!(!motor.contains(Capabilities::AXIS_CONTROL));
    }

    #[test]
    fn test_capabilities_combined() {
        let caps = Capabilities::AXIS_CONTROL | Capabilities::MOTOR_CONTROL;
        assert!(caps.contains(Capabilities::AXIS_CONTROL));
        assert!(caps.contains(Capabilities::MOTOR_CONTROL));
        assert!(!caps.contains(Capabilities::POSITION_FB));
    }

    #[test]
    fn test_capabilities_all_flags() {
        let all = Capabilities::AXIS_CONTROL
            | Capabilities::MOTOR_CONTROL
            | Capabilities::POSITION_FB
            | Capabilities::CURRENT_FB
            | Capabilities::TEMP_FB;

        assert!(all.contains(Capabilities::AXIS_CONTROL));
        assert!(all.contains(Capabilities::MOTOR_CONTROL));
        assert!(all.contains(Capabilities::POSITION_FB));
        assert!(all.contains(Capabilities::CURRENT_FB));
        assert!(all.contains(Capabilities::TEMP_FB));
    }

    #[test]
    fn test_capabilities_bits() {
        assert_eq!(Capabilities::AXIS_CONTROL.bits(), 0x0001);
        assert_eq!(Capabilities::MOTOR_CONTROL.bits(), 0x0002);
        assert_eq!(Capabilities::POSITION_FB.bits(), 0x0004);
        assert_eq!(Capabilities::CURRENT_FB.bits(), 0x0008);
        assert_eq!(Capabilities::TEMP_FB.bits(), 0x0010);
        assert_eq!(Capabilities::LIGHTS.bits(), 0x0020);
    }

    #[test]
    fn test_capabilities_from_bits() {
        let caps = Capabilities::from_bits(0x0003).unwrap();
        assert!(caps.contains(Capabilities::AXIS_CONTROL));
        assert!(caps.contains(Capabilities::MOTOR_CONTROL));
        assert!(!caps.contains(Capabilities::POSITION_FB));
    }

    #[test]
    fn test_tool_status_default() {
        let status = ToolStatus::default();
        assert_eq!(status.name, "");
        assert!(status.position.is_none());
        assert!(!status.active);
        assert!(status.current.is_none());
        assert!(!status.fault);
    }

    #[test]
    fn test_tool_output_variants() {
        // Just verify the enum variants exist and can be constructed
        let _none = ToolOutput::None;
        let _axis = ToolOutput::SetAxis(0.5);
        let _motor = ToolOutput::SetMotor(-0.5);
        let _both = ToolOutput::SetBoth {
            axis: 0.25,
            motor: -0.75,
        };
    }
}



