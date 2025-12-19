//! Simulated tool (attachment).

use can::Frame;
use tools::protocol::can_id;

/// Simulated tool.
pub struct SimTool {
    slot: u8,
    tool_type: u8,
    serial: u32,
    /// Position (0.0 to 1.0)
    position: f32,
    /// Motor RPM
    motor_rpm: u16,
    /// Current draw (mA)
    current_ma: u16,
    /// Commanded axis
    axis_cmd: f32,
    /// Commanded motor
    motor_cmd: f32,
    /// Timer for periodic messages
    discovery_timer: f64,
    status_timer: f64,
}

impl SimTool {
    pub fn new_auger(slot: u8) -> Self {
        Self {
            slot,
            tool_type: 1, // Snow auger
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

    /// Process a command frame.
    pub fn process_command(&mut self, frame: &Frame) {
        let Some((slot, msg_type)) = can_id::parse(frame.id) else {
            return;
        };

        if slot != self.slot || msg_type != can_id::MSG_COMMAND {
            return;
        }

        if frame.data.len() >= 5 {
            let axis_i16 = i16::from_le_bytes([frame.data[1], frame.data[2]]);
            let motor_i16 = i16::from_le_bytes([frame.data[3], frame.data[4]]);

            self.axis_cmd = axis_i16 as f32 / 32767.0;
            self.motor_cmd = motor_i16 as f32 / 32767.0;
        }
    }

    /// Update simulation state.
    pub fn tick(&mut self, dt: f64) {
        // Position moves based on axis command
        self.position += self.axis_cmd * dt as f32 * 0.5;
        self.position = self.position.clamp(0.0, 1.0);

        // Motor RPM based on motor command
        let target_rpm = (self.motor_cmd.abs() * 3000.0) as u16;
        if target_rpm > self.motor_rpm {
            self.motor_rpm = (self.motor_rpm + (1000.0 * dt as f32) as u16).min(target_rpm);
        } else {
            self.motor_rpm = self.motor_rpm.saturating_sub((2000.0 * dt as f32) as u16);
        }

        // Current based on motor RPM
        self.current_ma = self.motor_rpm / 2;

        // Timers
        self.discovery_timer += dt;
        self.status_timer += dt;
    }

    /// Generate a frame if it's time.
    pub fn generate_frame(&mut self) -> Option<Frame> {
        // Discovery every 1 second
        if self.discovery_timer >= 1.0 {
            self.discovery_timer = 0.0;
            return Some(self.discovery_frame());
        }

        // Status every 50ms (20Hz)
        if self.status_timer >= 0.05 {
            self.status_timer = 0.0;
            return Some(self.status_frame());
        }

        None
    }

    fn discovery_frame(&self) -> Frame {
        let id = can_id::make(self.slot, can_id::MSG_DISCOVERY);
        let mut data = [0u8; 8];
        data[0] = self.tool_type;
        data[1] = 1; // Protocol version
        data[2] = 0x0F; // Capabilities: all
        data[3] = 0x00;
        data[4..8].copy_from_slice(&self.serial.to_le_bytes());
        Frame::new_extended(id, &data)
    }

    fn status_frame(&self) -> Frame {
        let id = can_id::make(self.slot, can_id::MSG_STATUS);
        let mut data = [0u8; 8];
        data[0] = (self.position * 255.0) as u8;
        data[1..3].copy_from_slice(&self.motor_rpm.to_le_bytes());
        data[3..5].copy_from_slice(&self.current_ma.to_le_bytes());
        data[5] = 0; // No faults
        Frame::new_extended(id, &data)
    }
}



