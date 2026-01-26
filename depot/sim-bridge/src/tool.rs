//! Simulated tool (attachment).
//!
//! Copied from bvr/firmware/crates/sim/src/tool.rs with local CanFrame type.
//! Tool CAN protocol: Extended IDs 0x0A00 | (slot << 4) | msg_type

use crate::can_protocol::CanFrame;

const CAN_BASE: u32 = 0x0A00;
const MSG_DISCOVERY: u8 = 0x0;
const MSG_COMMAND: u8 = 0x1;
const MSG_STATUS: u8 = 0x2;

fn can_make(slot: u8, msg_type: u8) -> u32 {
    CAN_BASE | ((slot as u32) << 4) | (msg_type as u32)
}

fn can_parse(id: u32) -> Option<(u8, u8)> {
    if (id & 0xFF00) != CAN_BASE {
        return None;
    }
    let slot = ((id >> 4) & 0x0F) as u8;
    let msg_type = (id & 0x0F) as u8;
    Some((slot, msg_type))
}

/// Simulated tool.
pub struct SimTool {
    slot: u8,
    tool_type: u8,
    serial: u32,
    position: f32,
    motor_rpm: u16,
    current_ma: u16,
    axis_cmd: f32,
    motor_cmd: f32,
    discovery_timer: f64,
    status_timer: f64,
}

impl SimTool {
    pub fn new_auger(slot: u8) -> Self {
        Self {
            slot,
            tool_type: 1,
            serial: 0x12345678,
            position: 0.0,
            motor_rpm: 0,
            current_ma: 0,
            axis_cmd: 0.0,
            motor_cmd: 0.0,
            discovery_timer: 0.0,
            status_timer: 0.0,
        }
    }

    pub fn process_command(&mut self, frame: &CanFrame) {
        let Some((slot, msg_type)) = can_parse(frame.id) else {
            return;
        };

        if slot != self.slot || msg_type != MSG_COMMAND {
            return;
        }

        if frame.data.len() >= 5 {
            let axis_i16 = i16::from_le_bytes([frame.data[1], frame.data[2]]);
            let motor_i16 = i16::from_le_bytes([frame.data[3], frame.data[4]]);
            self.axis_cmd = axis_i16 as f32 / 32767.0;
            self.motor_cmd = motor_i16 as f32 / 32767.0;
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.position += self.axis_cmd * dt as f32 * 0.5;
        self.position = self.position.clamp(0.0, 1.0);

        let target_rpm = (self.motor_cmd.abs() * 3000.0) as u16;
        if target_rpm > self.motor_rpm {
            self.motor_rpm = (self.motor_rpm + (1000.0 * dt as f32) as u16).min(target_rpm);
        } else {
            self.motor_rpm = self.motor_rpm.saturating_sub((2000.0 * dt as f32) as u16);
        }

        self.current_ma = self.motor_rpm / 2;

        self.discovery_timer += dt;
        self.status_timer += dt;
    }

    pub fn generate_frame(&mut self) -> Option<CanFrame> {
        if self.discovery_timer >= 1.0 {
            self.discovery_timer = 0.0;
            return Some(self.discovery_frame());
        }

        if self.status_timer >= 0.05 {
            self.status_timer = 0.0;
            return Some(self.status_frame());
        }

        None
    }

    fn discovery_frame(&self) -> CanFrame {
        let id = can_make(self.slot, MSG_DISCOVERY);
        let mut data = [0u8; 8];
        data[0] = self.tool_type;
        data[1] = 1;
        data[2] = 0x0F;
        data[3] = 0x00;
        data[4..8].copy_from_slice(&self.serial.to_le_bytes());
        CanFrame::new_extended(id, &data)
    }

    fn status_frame(&self) -> CanFrame {
        let id = can_make(self.slot, MSG_STATUS);
        let mut data = [0u8; 8];
        data[0] = (self.position * 255.0) as u8;
        data[1..3].copy_from_slice(&self.motor_rpm.to_le_bytes());
        data[3..5].copy_from_slice(&self.current_ma.to_le_bytes());
        data[5] = 0;
        CanFrame::new_extended(id, &data)
    }
}
