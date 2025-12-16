//! Simulated VESC motor controller.

use can::Frame;

/// Simulated VESC.
pub struct SimVesc {
    id: u8,
    /// Commanded RPM (ERPM)
    target_erpm: i32,
    /// Current RPM (with lag)
    current_erpm: i32,
    /// Simulated current draw
    current_amps: f32,
    /// Simulated temperature
    temp_motor: f32,
    temp_fet: f32,
    /// Simulated voltage
    voltage: f32,
    /// Time since last status
    status_timer: f64,
}

impl SimVesc {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            target_erpm: 0,
            current_erpm: 0,
            current_amps: 0.0,
            temp_motor: 25.0,
            temp_fet: 25.0,
            voltage: 48.0,
            status_timer: 0.0,
        }
    }

    /// Process a command frame. Returns true if it was for this VESC.
    pub fn process_command(&mut self, frame: &Frame) -> bool {
        if !frame.extended {
            return false;
        }

        let frame_id = (frame.id & 0xFF) as u8;
        if frame_id != self.id {
            return false;
        }

        let cmd = (frame.id >> 8) as u8;

        match cmd {
            // Set RPM
            3 if frame.data.len() >= 4 => {
                self.target_erpm =
                    i32::from_be_bytes([frame.data[0], frame.data[1], frame.data[2], frame.data[3]]);
                true
            }
            // Set current
            1 if frame.data.len() >= 4 => {
                let current_ma =
                    i32::from_be_bytes([frame.data[0], frame.data[1], frame.data[2], frame.data[3]]);
                // Convert current command to approximate RPM
                self.target_erpm = (current_ma as f32 * 10.0) as i32;
                true
            }
            // Set duty
            0 if frame.data.len() >= 4 => {
                let duty =
                    i32::from_be_bytes([frame.data[0], frame.data[1], frame.data[2], frame.data[3]]);
                // Convert duty to approximate RPM
                self.target_erpm = (duty as f32 * 0.5) as i32;
                true
            }
            _ => false,
        }
    }

    /// Update simulation state.
    pub fn tick(&mut self, dt: f64) {
        // RPM approaches target with lag
        let rpm_diff = self.target_erpm - self.current_erpm;
        let max_change = (5000.0 * dt) as i32; // Max 5000 ERPM/s acceleration
        if rpm_diff.abs() > max_change {
            self.current_erpm += rpm_diff.signum() * max_change;
        } else {
            self.current_erpm = self.target_erpm;
        }

        // Current proportional to RPM (simplified)
        self.current_amps = (self.current_erpm.abs() as f32) * 0.001;

        // Temperature rises with current, falls toward ambient
        let ambient = 25.0;
        let heat = self.current_amps * 0.5;
        self.temp_motor += (heat - (self.temp_motor - ambient) * 0.1) * dt as f32;
        self.temp_fet += (heat * 0.5 - (self.temp_fet - ambient) * 0.1) * dt as f32;

        // Voltage sag under load (simplified)
        self.voltage = 48.0 - self.current_amps * 0.05;

        // Status timer
        self.status_timer += dt;
    }

    /// Check if status should be sent (~50Hz).
    pub fn should_send_status(&mut self) -> bool {
        if self.status_timer >= 0.02 {
            self.status_timer = 0.0;
            true
        } else {
            false
        }
    }

    /// Generate status frames.
    pub fn generate_status(&self) -> Option<Vec<Frame>> {
        let mut frames = Vec::new();

        // STATUS1 (ID = 9): ERPM, current, duty
        let id1 = ((9u32) << 8) | (self.id as u32);
        let duty = (self.current_erpm as f32 / 50000.0 * 1000.0) as i16;
        let current_i16 = (self.current_amps * 10.0) as i16;
        let mut data1 = [0u8; 8];
        data1[0..4].copy_from_slice(&self.current_erpm.to_be_bytes());
        data1[4..6].copy_from_slice(&current_i16.to_be_bytes());
        data1[6..8].copy_from_slice(&duty.to_be_bytes());
        frames.push(Frame::new_extended(id1, &data1));

        // STATUS4 (ID = 16): temps, current in
        let id4 = ((16u32) << 8) | (self.id as u32);
        let temp_fet_i16 = (self.temp_fet * 10.0) as i16;
        let temp_motor_i16 = (self.temp_motor * 10.0) as i16;
        let current_in_i16 = (self.current_amps * 10.0) as i16;
        let mut data4 = [0u8; 8];
        data4[0..2].copy_from_slice(&temp_fet_i16.to_be_bytes());
        data4[2..4].copy_from_slice(&temp_motor_i16.to_be_bytes());
        data4[4..6].copy_from_slice(&current_in_i16.to_be_bytes());
        frames.push(Frame::new_extended(id4, &data4));

        // STATUS5 (ID = 27): tachometer, voltage
        let id5 = ((27u32) << 8) | (self.id as u32);
        let voltage_i16 = (self.voltage * 10.0) as i16;
        let mut data5 = [0u8; 8];
        data5[0..4].copy_from_slice(&0i32.to_be_bytes()); // tachometer
        data5[4..6].copy_from_slice(&voltage_i16.to_be_bytes());
        frames.push(Frame::new_extended(id5, &data5));

        Some(frames)
    }

    /// Get current mechanical RPM (ERPM / pole_pairs).
    pub fn rpm(&self) -> f64 {
        self.current_erpm as f64 / 15.0 // Assuming 15 pole pairs
    }
}
