//! VESC CAN protocol implementation.
//!
//! Reference: https://github.com/vedderb/bldc/blob/master/comm/comm_can.c

use crate::{Bus, CanError, Frame};
use tracing::debug;

/// VESC CAN command IDs (used in extended frame ID).
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum CommandId {
    SetDuty = 0,
    SetCurrent = 1,
    SetCurrentBrake = 2,
    SetRpm = 3,
    SetPos = 4,
    Status1 = 9,  // ERPM, current, duty
    Status2 = 14, // Ah, Ah charged
    Status3 = 15, // Wh, Wh charged
    Status4 = 16, // Temp FET, temp motor, current in, PID pos
    Status5 = 27, // Tachometer, voltage in
}

/// VESC status from STATUS1 message.
#[derive(Debug, Clone, Copy, Default)]
pub struct VescStatus {
    /// Electrical RPM (divide by motor pole pairs for mechanical RPM)
    pub erpm: i32,
    /// Motor current in amps
    pub current: f32,
    /// Duty cycle (-1.0 to 1.0)
    pub duty: f32,
}

/// VESC extended status from STATUS4 message.
#[derive(Debug, Clone, Copy, Default)]
pub struct VescStatus4 {
    /// FET temperature in °C
    pub temp_fet: f32,
    /// Motor temperature in °C
    pub temp_motor: f32,
    /// Input current in amps
    pub current_in: f32,
}

/// VESC extended status from STATUS5 message.
#[derive(Debug, Clone, Copy, Default)]
pub struct VescStatus5 {
    /// Tachometer value (ERPM counts)
    pub tachometer: i32,
    /// Input voltage
    pub voltage_in: f32,
}

/// Combined VESC state.
#[derive(Debug, Clone, Copy, Default)]
pub struct VescState {
    pub id: u8,
    pub status: VescStatus,
    pub status4: VescStatus4,
    pub status5: VescStatus5,
}

/// VESC controller interface.
pub struct Vesc {
    id: u8,
    state: VescState,
}

impl Vesc {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            state: VescState { id, ..Default::default() },
        }
    }

    /// Get the VESC CAN ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Get current state.
    pub fn state(&self) -> &VescState {
        &self.state
    }

    /// Build extended CAN ID for a command.
    fn make_id(&self, cmd: CommandId) -> u32 {
        ((cmd as u32) << 8) | (self.id as u32)
    }

    /// Send RPM command (ERPM = mechanical RPM × pole pairs).
    pub fn set_rpm(&self, bus: &Bus, erpm: i32) -> Result<(), CanError> {
        let id = self.make_id(CommandId::SetRpm);
        let data = erpm.to_be_bytes();
        bus.send(&Frame::new_extended(id, &data))?;
        debug!(vesc_id = self.id, erpm, "VESC set RPM");
        Ok(())
    }

    /// Send current command in amps.
    pub fn set_current(&self, bus: &Bus, current: f32) -> Result<(), CanError> {
        let id = self.make_id(CommandId::SetCurrent);
        let current_ma = (current * 1000.0) as i32;
        let data = current_ma.to_be_bytes();
        bus.send(&Frame::new_extended(id, &data))?;
        debug!(vesc_id = self.id, current, "VESC set current");
        Ok(())
    }

    /// Send duty cycle command (-1.0 to 1.0).
    pub fn set_duty(&self, bus: &Bus, duty: f32) -> Result<(), CanError> {
        let id = self.make_id(CommandId::SetDuty);
        let duty_scaled = (duty.clamp(-1.0, 1.0) * 100_000.0) as i32;
        let data = duty_scaled.to_be_bytes();
        bus.send(&Frame::new_extended(id, &data))?;
        debug!(vesc_id = self.id, duty, "VESC set duty");
        Ok(())
    }

    /// Process a received CAN frame. Returns true if it was for this VESC.
    pub fn process_frame(&mut self, frame: &Frame) -> bool {
        if !frame.extended {
            return false;
        }

        let frame_vesc_id = (frame.id & 0xFF) as u8;
        if frame_vesc_id != self.id {
            return false;
        }

        let cmd = (frame.id >> 8) as u8;
        match cmd {
            9 => self.parse_status1(&frame.data),
            16 => self.parse_status4(&frame.data),
            27 => self.parse_status5(&frame.data),
            _ => return false,
        }

        true
    }

    fn parse_status1(&mut self, data: &[u8]) {
        if data.len() < 8 {
            return;
        }
        self.state.status.erpm = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        self.state.status.current = i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0;
        self.state.status.duty = i16::from_be_bytes([data[6], data[7]]) as f32 / 1000.0;
    }

    fn parse_status4(&mut self, data: &[u8]) {
        if data.len() < 8 {
            return;
        }
        self.state.status4.temp_fet = i16::from_be_bytes([data[0], data[1]]) as f32 / 10.0;
        self.state.status4.temp_motor = i16::from_be_bytes([data[2], data[3]]) as f32 / 10.0;
        self.state.status4.current_in = i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0;
    }

    fn parse_status5(&mut self, data: &[u8]) {
        if data.len() < 8 {
            return;
        }
        self.state.status5.tachometer = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        self.state.status5.voltage_in = i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0;
    }
}

/// Drivetrain: manages 4 VESCs for the wheels.
pub struct Drivetrain {
    pub front_left: Vesc,
    pub front_right: Vesc,
    pub rear_left: Vesc,
    pub rear_right: Vesc,
    /// Motor pole pairs (for ERPM to RPM conversion)
    pub pole_pairs: u8,
}

impl Drivetrain {
    pub fn new(ids: [u8; 4], pole_pairs: u8) -> Self {
        Self {
            front_left: Vesc::new(ids[0]),
            front_right: Vesc::new(ids[1]),
            rear_left: Vesc::new(ids[2]),
            rear_right: Vesc::new(ids[3]),
            pole_pairs,
        }
    }

    /// Send RPM commands to all wheels.
    ///
    /// Takes mechanical RPM values (converted to ERPM internally).
    pub fn set_rpm(&self, bus: &Bus, rpm: [f64; 4]) -> Result<(), CanError> {
        let pp = self.pole_pairs as f64;
        self.front_left.set_rpm(bus, (rpm[0] * pp) as i32)?;
        self.front_right.set_rpm(bus, (rpm[1] * pp) as i32)?;
        self.rear_left.set_rpm(bus, (rpm[2] * pp) as i32)?;
        self.rear_right.set_rpm(bus, (rpm[3] * pp) as i32)?;
        Ok(())
    }

    /// Process a received CAN frame.
    pub fn process_frame(&mut self, frame: &Frame) {
        self.front_left.process_frame(frame);
        self.front_right.process_frame(frame);
        self.rear_left.process_frame(frame);
        self.rear_right.process_frame(frame);
    }

    /// Get battery voltage (from any VESC that has reported).
    pub fn battery_voltage(&self) -> f32 {
        // Return voltage from front-left, or any that has a reading
        let v = self.front_left.state.status5.voltage_in;
        if v > 0.0 {
            v
        } else {
            self.front_right.state.status5.voltage_in
        }
    }

    /// Get all VESC states.
    pub fn states(&self) -> [&VescState; 4] {
        [
            self.front_left.state(),
            self.front_right.state(),
            self.rear_left.state(),
            self.rear_right.state(),
        ]
    }
}
