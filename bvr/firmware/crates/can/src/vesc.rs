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

    /// Build an RPM command frame (without sending).
    pub fn build_rpm_frame(&self, erpm: i32) -> crate::Frame {
        let id = self.make_id(CommandId::SetRpm);
        let data = erpm.to_be_bytes();
        crate::Frame::new_extended(id, &data)
    }

    /// Build a duty cycle command frame (without sending).
    pub fn build_duty_frame(&self, duty: f32) -> crate::Frame {
        let id = self.make_id(CommandId::SetDuty);
        let duty_scaled = (duty.clamp(-1.0, 1.0) * 100_000.0) as i32;
        let data = duty_scaled.to_be_bytes();
        crate::Frame::new_extended(id, &data)
    }

    /// Build a current command frame (without sending).
    pub fn build_current_frame(&self, current: f32) -> crate::Frame {
        let id = self.make_id(CommandId::SetCurrent);
        let current_ma = (current * 1000.0) as i32;
        let data = current_ma.to_be_bytes();
        crate::Frame::new_extended(id, &data)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frame;

    #[test]
    fn test_make_id_rpm_command() {
        let vesc = Vesc::new(1);
        let id = vesc.make_id(CommandId::SetRpm);
        // SetRpm = 3, so ID = (3 << 8) | 1 = 0x0301
        assert_eq!(id, 0x0301);
    }

    #[test]
    fn test_make_id_duty_command() {
        let vesc = Vesc::new(4);
        let id = vesc.make_id(CommandId::SetDuty);
        // SetDuty = 0, so ID = (0 << 8) | 4 = 0x0004
        assert_eq!(id, 0x0004);
    }

    #[test]
    fn test_build_rpm_frame() {
        let vesc = Vesc::new(2);
        let frame = vesc.build_rpm_frame(1000);

        assert_eq!(frame.id, 0x0302); // SetRpm(3) << 8 | 2
        assert!(frame.extended);
        assert_eq!(frame.data, 1000_i32.to_be_bytes().to_vec());
    }

    #[test]
    fn test_build_rpm_frame_negative() {
        let vesc = Vesc::new(1);
        let frame = vesc.build_rpm_frame(-5000);

        assert_eq!(frame.data, (-5000_i32).to_be_bytes().to_vec());
    }

    #[test]
    fn test_build_duty_frame_clamped() {
        let vesc = Vesc::new(1);

        // Test normal range
        let frame = vesc.build_duty_frame(0.5);
        let expected = (0.5 * 100_000.0) as i32;
        assert_eq!(frame.data, expected.to_be_bytes().to_vec());

        // Test clamping above 1.0
        let frame = vesc.build_duty_frame(2.0);
        let expected = 100_000_i32; // Clamped to 1.0 * 100_000
        assert_eq!(frame.data, expected.to_be_bytes().to_vec());

        // Test clamping below -1.0
        let frame = vesc.build_duty_frame(-1.5);
        let expected = -100_000_i32; // Clamped to -1.0 * 100_000
        assert_eq!(frame.data, expected.to_be_bytes().to_vec());
    }

    #[test]
    fn test_build_current_frame() {
        let vesc = Vesc::new(3);
        let frame = vesc.build_current_frame(10.5);

        assert_eq!(frame.id, 0x0103); // SetCurrent(1) << 8 | 3
        // 10.5 amps = 10500 mA
        let expected = 10500_i32;
        assert_eq!(frame.data, expected.to_be_bytes().to_vec());
    }

    #[test]
    fn test_parse_status1() {
        let mut vesc = Vesc::new(1);

        // Build a STATUS1 frame
        // ERPM: 3000, Current: 15.5A (155 * 10), Duty: 0.500 (500)
        let mut data = [0u8; 8];
        data[0..4].copy_from_slice(&3000_i32.to_be_bytes());
        data[4..6].copy_from_slice(&155_i16.to_be_bytes()); // 15.5A
        data[6..8].copy_from_slice(&500_i16.to_be_bytes()); // 0.500 duty

        let frame = Frame {
            id: (9 << 8) | 1, // STATUS1 for VESC ID 1
            extended: true,
            data: data.to_vec(),
        };

        assert!(vesc.process_frame(&frame));
        assert_eq!(vesc.state.status.erpm, 3000);
        assert!((vesc.state.status.current - 15.5).abs() < 0.01);
        assert!((vesc.state.status.duty - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_status4() {
        let mut vesc = Vesc::new(2);

        // Build a STATUS4 frame
        // FET temp: 45.0C (450), Motor temp: 55.0C (550), Current in: 8.5A (85)
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(&450_i16.to_be_bytes()); // 45.0C
        data[2..4].copy_from_slice(&550_i16.to_be_bytes()); // 55.0C
        data[4..6].copy_from_slice(&85_i16.to_be_bytes()); // 8.5A

        let frame = Frame {
            id: (16 << 8) | 2, // STATUS4 for VESC ID 2
            extended: true,
            data: data.to_vec(),
        };

        assert!(vesc.process_frame(&frame));
        assert!((vesc.state.status4.temp_fet - 45.0).abs() < 0.01);
        assert!((vesc.state.status4.temp_motor - 55.0).abs() < 0.01);
        assert!((vesc.state.status4.current_in - 8.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_status5() {
        let mut vesc = Vesc::new(3);

        // Build a STATUS5 frame
        // Tachometer: 12345, Voltage: 48.5V (485)
        let mut data = [0u8; 8];
        data[0..4].copy_from_slice(&12345_i32.to_be_bytes());
        data[4..6].copy_from_slice(&485_i16.to_be_bytes()); // 48.5V

        let frame = Frame {
            id: (27 << 8) | 3, // STATUS5 for VESC ID 3
            extended: true,
            data: data.to_vec(),
        };

        assert!(vesc.process_frame(&frame));
        assert_eq!(vesc.state.status5.tachometer, 12345);
        assert!((vesc.state.status5.voltage_in - 48.5).abs() < 0.01);
    }

    #[test]
    fn test_process_frame_wrong_vesc_id() {
        let mut vesc = Vesc::new(1);

        let frame = Frame {
            id: (9 << 8) | 2, // STATUS1 for VESC ID 2 (not ours)
            extended: true,
            data: vec![0; 8],
        };

        assert!(!vesc.process_frame(&frame));
    }

    #[test]
    fn test_process_frame_standard_id_ignored() {
        let mut vesc = Vesc::new(1);

        let frame = Frame {
            id: (9 << 8) | 1,
            extended: false, // Standard frame, not extended
            data: vec![0; 8],
        };

        assert!(!vesc.process_frame(&frame));
    }

    #[test]
    fn test_process_frame_unknown_command_ignored() {
        let mut vesc = Vesc::new(1);

        let frame = Frame {
            id: (99 << 8) | 1, // Unknown command type
            extended: true,
            data: vec![0; 8],
        };

        assert!(!vesc.process_frame(&frame));
    }

    #[test]
    fn test_parse_status_short_data_ignored() {
        let mut vesc = Vesc::new(1);

        // STATUS1 with only 4 bytes (should need 8)
        let frame = Frame {
            id: (9 << 8) | 1,
            extended: true,
            data: vec![0; 4],
        };

        // Should return true (frame was for us) but not crash
        assert!(vesc.process_frame(&frame));
        // Values should remain at defaults (0)
        assert_eq!(vesc.state.status.erpm, 0);
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
        // Return voltage from any VESC that has a valid reading
        // Check all 4 VESCs in case some aren't sending STATUS5
        for vesc in [&self.front_left, &self.front_right, &self.rear_left, &self.rear_right] {
            let v = vesc.state.status5.voltage_in;
            if v > 0.0 {
                return v;
            }
        }
        0.0
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

    /// Build RPM command frames for all wheels (without sending).
    pub fn build_rpm_commands(&self, rpm: [f64; 4]) -> Vec<crate::Frame> {
        let pp = self.pole_pairs as f64;
        vec![
            self.front_left.build_rpm_frame((rpm[0] * pp) as i32),
            self.front_right.build_rpm_frame((rpm[1] * pp) as i32),
            self.rear_left.build_rpm_frame((rpm[2] * pp) as i32),
            self.rear_right.build_rpm_frame((rpm[3] * pp) as i32),
        ]
    }

    /// Build duty cycle command frames for all wheels (without sending).
    ///
    /// Duty values are -1.0 to 1.0 (full reverse to full forward).
    pub fn build_duty_commands(&self, duty: [f64; 4]) -> Vec<crate::Frame> {
        vec![
            self.front_left.build_duty_frame(duty[0] as f32),
            self.front_right.build_duty_frame(duty[1] as f32),
            self.rear_left.build_duty_frame(duty[2] as f32),
            self.rear_right.build_duty_frame(duty[3] as f32),
        ]
    }

    /// Build current (torque) command frames for all wheels (without sending).
    ///
    /// Current values are in amps (positive = forward torque).
    pub fn build_current_commands(&self, current: [f64; 4]) -> Vec<crate::Frame> {
        vec![
            self.front_left.build_current_frame(current[0] as f32),
            self.front_right.build_current_frame(current[1] as f32),
            self.rear_left.build_current_frame(current[2] as f32),
            self.rear_right.build_current_frame(current[3] as f32),
        ]
    }
}



