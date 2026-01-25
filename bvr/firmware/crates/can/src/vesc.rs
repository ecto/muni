//! VESC CAN protocol implementation.
//!
//! Reference: https://github.com/vedderb/bldc/blob/master/comm/comm_can.c

use std::time::Instant;

use crate::{Bus, CanError, Frame};
use tracing::debug;

/// Simple exponential moving average (low-pass) filter.
///
/// Smooths noisy sensor readings: `filtered = α × new + (1-α) × filtered`
#[derive(Debug, Clone, Copy)]
pub struct LowPassFilter {
    value: f32,
    alpha: f32,
    initialized: bool,
}

impl LowPassFilter {
    /// Create a new filter with the given smoothing factor.
    ///
    /// Alpha should be in (0, 1]:
    /// - Lower alpha = more smoothing, slower response
    /// - Higher alpha = less smoothing, faster response
    /// - Typical values: 0.1 (heavy smoothing) to 0.5 (light smoothing)
    pub fn new(alpha: f32) -> Self {
        Self {
            value: 0.0,
            alpha: alpha.clamp(0.01, 1.0),
            initialized: false,
        }
    }

    /// Update the filter with a new measurement and return the filtered value.
    pub fn update(&mut self, measurement: f32) -> f32 {
        if !self.initialized {
            self.value = measurement;
            self.initialized = true;
        } else {
            self.value = self.alpha * measurement + (1.0 - self.alpha) * self.value;
        }
        self.value
    }

    /// Get the current filtered value without updating.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Reset the filter to uninitialized state.
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.initialized = false;
    }
}

/// 2D Kalman filter for scalar signals with rate estimation.
///
/// State vector: `[value, rate_of_change]`
/// Uses a constant-velocity motion model to predict between measurements,
/// providing smoother output than EMA while tracking real changes faster.
#[derive(Debug, Clone)]
pub struct KalmanFilter1D {
    // State estimate
    x0: f32, // value
    x1: f32, // rate of change
    // Error covariance (2x2)
    p00: f32,
    p01: f32,
    p10: f32,
    p11: f32,
    // Tuning
    q_value: f32, // process noise on value
    q_rate: f32,  // process noise on rate
    r: f32,       // measurement noise
    // Internal time tracking
    last_update: Option<Instant>,
    initialized: bool,
}

impl KalmanFilter1D {
    /// Create a new Kalman filter.
    ///
    /// - `q_value`: process noise for the value state (higher = trusts measurements more)
    /// - `q_rate`: process noise for the rate state (higher = allows faster rate changes)
    /// - `r`: measurement noise variance (higher = trusts model more, smoother output)
    pub fn new(q_value: f32, q_rate: f32, r: f32) -> Self {
        Self {
            x0: 0.0,
            x1: 0.0,
            p00: 100.0,
            p01: 0.0,
            p10: 0.0,
            p11: 100.0,
            q_value,
            q_rate,
            r,
            last_update: None,
            initialized: false,
        }
    }

    /// Predict step: propagate state and covariance forward by dt seconds.
    ///
    /// Motion model: `F = [[1, dt], [0, 1]]` (constant velocity)
    pub fn predict(&mut self, dt: f32) {
        self.x0 += dt * self.x1;

        let p00 = self.p00;
        let p01 = self.p01;
        let p10 = self.p10;
        let p11 = self.p11;

        // P = F * P * F' + Q
        self.p00 = p00 + dt * (p10 + p01) + dt * dt * p11 + self.q_value;
        self.p01 = p01 + dt * p11;
        self.p10 = p10 + dt * p11;
        self.p11 = p11 + self.q_rate;
    }

    /// Correction step: incorporate a measurement.
    fn correct(&mut self, measurement: f32) {
        let innovation = measurement - self.x0;
        let s = self.p00 + self.r;
        if s.abs() < 1e-10 {
            return;
        }
        let k0 = self.p00 / s;
        let k1 = self.p10 / s;

        self.x0 += k0 * innovation;
        self.x1 += k1 * innovation;

        // Covariance update (Joseph form for numerical stability)
        let m = 1.0 - k0;
        let p00 = self.p00;
        let p01 = self.p01;
        let p10 = self.p10;
        let p11 = self.p11;

        self.p00 = m * m * p00 + self.r * k0 * k0;
        self.p01 = m * (p01 - k1 * p00) + self.r * k0 * k1;
        self.p10 = m * (p10 - k1 * p00) + self.r * k1 * k0;
        self.p11 = p11 - k1 * (p01 + p10) + k1 * k1 * (p00 + self.r);
    }

    /// Update the filter with a new measurement.
    ///
    /// Internally tracks time between calls for the predict step.
    pub fn update(&mut self, measurement: f32) -> f32 {
        if !self.initialized {
            self.x0 = measurement;
            self.x1 = 0.0;
            self.initialized = true;
            self.last_update = Some(Instant::now());
            return self.x0;
        }

        let now = Instant::now();
        if let Some(last) = self.last_update {
            let dt = now.duration_since(last).as_secs_f32();
            if dt > 0.0 {
                self.predict(dt);
            }
        }
        self.last_update = Some(now);

        self.correct(measurement);
        self.x0
    }

    /// Update with an explicit time delta (for testing or external time sources).
    pub fn update_with_dt(&mut self, dt: f32, measurement: f32) -> f32 {
        if !self.initialized {
            self.x0 = measurement;
            self.x1 = 0.0;
            self.initialized = true;
            return self.x0;
        }

        if dt > 0.0 {
            self.predict(dt);
        }
        self.correct(measurement);
        self.x0
    }

    /// Get the current filtered value without updating.
    pub fn value(&self) -> f32 {
        self.x0
    }

    /// Get the estimated rate of change.
    pub fn rate(&self) -> f32 {
        self.x1
    }

    /// Reset the filter to uninitialized state.
    pub fn reset(&mut self) {
        self.x0 = 0.0;
        self.x1 = 0.0;
        self.p00 = 100.0;
        self.p01 = 0.0;
        self.p10 = 0.0;
        self.p11 = 100.0;
        self.last_update = None;
        self.initialized = false;
    }
}

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

    #[test]
    fn test_low_pass_filter_initialization() {
        let mut filter = LowPassFilter::new(0.5);
        // First value should be used directly
        assert_eq!(filter.update(10.0), 10.0);
        assert_eq!(filter.value(), 10.0);
    }

    #[test]
    fn test_low_pass_filter_smoothing() {
        let mut filter = LowPassFilter::new(0.5);
        filter.update(10.0); // Initialize
        // With alpha=0.5: new = 0.5 * 20 + 0.5 * 10 = 15
        assert!((filter.update(20.0) - 15.0).abs() < 0.01);
        // Next: 0.5 * 20 + 0.5 * 15 = 17.5
        assert!((filter.update(20.0) - 17.5).abs() < 0.01);
    }

    #[test]
    fn test_low_pass_filter_heavy_smoothing() {
        let mut filter = LowPassFilter::new(0.1);
        filter.update(10.0);
        // With alpha=0.1: new = 0.1 * 20 + 0.9 * 10 = 11
        assert!((filter.update(20.0) - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_low_pass_filter_reset() {
        let mut filter = LowPassFilter::new(0.5);
        filter.update(100.0);
        filter.reset();
        // After reset, next value is used directly
        assert_eq!(filter.update(50.0), 50.0);
    }

    #[test]
    fn test_kalman_initialization() {
        let mut kf = KalmanFilter1D::new(0.1, 0.01, 1.0);
        // First measurement is adopted directly
        assert_eq!(kf.update_with_dt(0.0, 48.0), 48.0);
        assert_eq!(kf.value(), 48.0);
        assert_eq!(kf.rate(), 0.0);
    }

    #[test]
    fn test_kalman_smoothing() {
        let mut kf = KalmanFilter1D::new(0.001, 0.0001, 0.5);
        kf.update_with_dt(0.0, 48.0); // init

        // Feed noisy readings around 48V at 100Hz
        let readings = [48.3, 47.8, 48.1, 47.9, 48.2, 48.0, 47.7, 48.4, 48.1, 47.9];
        let mut last = 0.0_f32;
        for &r in &readings {
            last = kf.update_with_dt(0.01, r);
        }
        // Should converge near 48.0 with less variance than raw signal
        assert!((last - 48.0).abs() < 0.5, "filtered value {last} too far from 48.0");
    }

    #[test]
    fn test_kalman_tracks_step_change() {
        // Current filter with high process noise to track fast changes
        let mut kf = KalmanFilter1D::new(1.0, 0.1, 1.0);
        kf.update_with_dt(0.0, 0.0); // init at 0A

        // Step to 10A
        for _ in 0..50 {
            kf.update_with_dt(0.01, 10.0);
        }
        // Should have converged close to 10A within 50 steps at 100Hz
        assert!(
            (kf.value() - 10.0).abs() < 0.5,
            "kalman value {} didn't track step to 10.0",
            kf.value()
        );
    }

    #[test]
    fn test_kalman_rate_estimation() {
        let mut kf = KalmanFilter1D::new(0.1, 0.01, 0.5);
        kf.update_with_dt(0.0, 0.0); // init

        // Feed a linear ramp: 1 unit/sec for 1 second at 100Hz
        for i in 1..=100 {
            let t = i as f32 * 0.01;
            kf.update_with_dt(0.01, t); // value = t, so rate should approach 1.0
        }
        // Rate estimate should be roughly 1.0 V/s
        assert!(
            (kf.rate() - 1.0).abs() < 0.3,
            "rate estimate {} not close to 1.0",
            kf.rate()
        );
    }

    #[test]
    fn test_kalman_reset() {
        let mut kf = KalmanFilter1D::new(0.1, 0.01, 1.0);
        kf.update_with_dt(0.0, 48.0);
        kf.update_with_dt(0.01, 48.5);
        kf.reset();
        // After reset, next value is adopted directly
        assert_eq!(kf.update_with_dt(0.0, 24.0), 24.0);
        assert_eq!(kf.rate(), 0.0);
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
    /// Kalman-filtered battery voltage
    voltage_filter: KalmanFilter1D,
    /// Kalman-filtered total system current (sum of all VESCs)
    current_filter: KalmanFilter1D,
}

// Voltage Kalman filter tuning (battery changes slowly)
const VOLTAGE_Q_VALUE: f32 = 0.001;  // low process noise on value
const VOLTAGE_Q_RATE: f32 = 0.0001;  // very low — voltage rate barely changes
const VOLTAGE_R: f32 = 0.5;          // moderate measurement noise

// Current Kalman filter tuning (load current changes rapidly)
const CURRENT_Q_VALUE: f32 = 1.0;    // high process noise — current swings fast
const CURRENT_Q_RATE: f32 = 0.1;     // moderate — current rate varies
const CURRENT_R: f32 = 1.0;          // moderate measurement noise

impl Drivetrain {
    pub fn new(ids: [u8; 4], pole_pairs: u8) -> Self {
        Self {
            front_left: Vesc::new(ids[0]),
            front_right: Vesc::new(ids[1]),
            rear_left: Vesc::new(ids[2]),
            rear_right: Vesc::new(ids[3]),
            pole_pairs,
            voltage_filter: KalmanFilter1D::new(VOLTAGE_Q_VALUE, VOLTAGE_Q_RATE, VOLTAGE_R),
            current_filter: KalmanFilter1D::new(CURRENT_Q_VALUE, CURRENT_Q_RATE, CURRENT_R),
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

    /// Process a received CAN frame and update filters.
    ///
    /// Filters are only updated when the underlying measurement actually changes,
    /// preventing covariance inflation from processing batched CAN frames with
    /// near-zero dt between them.
    pub fn process_frame(&mut self, frame: &Frame) {
        let old_voltage = self.raw_battery_voltage();
        let old_current = self.raw_total_current();

        self.front_left.process_frame(frame);
        self.front_right.process_frame(frame);
        self.rear_left.process_frame(frame);
        self.rear_right.process_frame(frame);

        // Only update filters when the raw reading actually changed
        let new_voltage = self.raw_battery_voltage();
        if new_voltage > 0.0 && new_voltage != old_voltage {
            self.voltage_filter.update(new_voltage);
        }

        let new_current = self.raw_total_current();
        if (new_current - old_current).abs() > f32::EPSILON {
            self.current_filter.update(new_current);
        }
    }

    /// Get raw battery voltage (unfiltered, from any VESC that has reported).
    pub fn raw_battery_voltage(&self) -> f32 {
        for vesc in [&self.front_left, &self.front_right, &self.rear_left, &self.rear_right] {
            let v = vesc.state.status5.voltage_in;
            if v > 0.0 {
                return v;
            }
        }
        0.0
    }

    /// Get filtered battery voltage (smoothed to reduce noise).
    pub fn battery_voltage(&self) -> f32 {
        self.voltage_filter.value()
    }

    /// Get raw total system current (unfiltered).
    ///
    /// Sums battery-side input current (current_in from STATUS4) across all VESCs.
    /// Returns 0.0 until STATUS4 messages have been received.
    pub fn raw_total_current(&self) -> f32 {
        self.front_left.state.status4.current_in
            + self.front_right.state.status4.current_in
            + self.rear_left.state.status4.current_in
            + self.rear_right.state.status4.current_in
    }

    /// Get filtered total system current (smoothed to reduce noise).
    pub fn total_current(&self) -> f32 {
        self.current_filter.value()
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



