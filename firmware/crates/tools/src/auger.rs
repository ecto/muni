//! Snow auger tool implementation.

use crate::protocol::StatusPayload;
use crate::{Capabilities, Tool, ToolInfo, ToolOutput, ToolStatus, ToolType};
use types::ToolCommand;

/// Snow auger tool.
///
/// Controls:
/// - Axis (RT/LT): Raise/lower the auger
/// - Action A: Toggle auger spin
pub struct SnowAuger {
    info: ToolInfo,
    position: f32,
    auger_rpm: f32,
    motor_current: f32,
    auger_enabled: bool,
    fault: bool,
    // Debounce for action button
    action_a_prev: bool,
}

impl SnowAuger {
    pub fn new(slot: u8, serial: u32) -> Self {
        Self {
            info: ToolInfo {
                slot,
                tool_type: ToolType::SnowAuger,
                capabilities: Capabilities::AXIS_CONTROL
                    | Capabilities::MOTOR_CONTROL
                    | Capabilities::POSITION_FB
                    | Capabilities::CURRENT_FB,
                serial,
                name: "Snow Auger",
            },
            position: 0.0,
            auger_rpm: 0.0,
            motor_current: 0.0,
            auger_enabled: false,
            fault: false,
            action_a_prev: false,
        }
    }
}

impl Tool for SnowAuger {
    fn info(&self) -> &ToolInfo {
        &self.info
    }

    fn update(&mut self, input: &ToolCommand) -> ToolOutput {
        // Toggle auger on A button press (rising edge)
        if input.action_a && !self.action_a_prev {
            self.auger_enabled = !self.auger_enabled;
        }
        self.action_a_prev = input.action_a;

        // RT/LT controls lift axis
        let axis = input.axis;

        // Motor controlled by enabled state
        let motor = if self.auger_enabled { 1.0 } else { 0.0 };

        ToolOutput::SetBoth { axis, motor }
    }

    fn handle_status(&mut self, data: &[u8]) {
        if let Some(status) = StatusPayload::parse(data) {
            self.position = status.position_normalized();
            self.auger_rpm = status.motor_rpm as f32;
            self.motor_current = status.current_amps();
            self.fault = status.faults != 0;
        }
    }

    fn status(&self) -> ToolStatus {
        ToolStatus {
            name: self.info.name,
            position: Some(self.position),
            active: self.auger_enabled,
            current: Some(self.motor_current),
            fault: self.fault,
        }
    }
}



