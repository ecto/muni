//! Blower tool implementation (90mm EDF via VESC CAN).
//!
//! The blower is a VESC-based tool that sends SetDuty commands directly
//! on the VESC extended CAN protocol, bypassing the tool MCU protocol.
//!
//! Controls:
//! - Motor (RT): Blower power (0-1)
//! - Action A: Toggle blower on/off

use crate::{Capabilities, Tool, ToolInfo, ToolOutput, ToolStatus, ToolType};
use can::Frame;
use serde::Deserialize;
use std::time::Instant;
use tracing::debug;
use types::ToolCommand;

/// Blower configuration (from bvr.toml `[blower]` section).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BlowerConfig {
    /// Enable blower subsystem.
    pub enabled: bool,
    /// VESC CAN node ID (default 0x0C = 12).
    pub can_id: u8,
    /// Maximum power in autonomous mode (percent, 0-100).
    pub max_power_autonomous: f32,
    /// Power ramp rate (percent per second).
    pub power_ramp_rate: f32,
    /// Minimum rover velocity to engage blower in autonomous mode (m/s).
    pub min_velocity_threshold: f32,
    /// Rover velocity at which autonomous blower reaches max power (m/s).
    pub max_velocity_for_scaling: f32,
}

impl Default for BlowerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            can_id: 0x0C,
            max_power_autonomous: 50.0,
            power_ramp_rate: 10.0,
            min_velocity_threshold: 0.1,
            max_velocity_for_scaling: 3.0,
        }
    }
}

/// Blower tool — controls a VESC-driven EDF via CAN SetDuty.
pub struct Blower {
    info: ToolInfo,
    config: BlowerConfig,
    /// Current output power (ramped, 0.0-1.0).
    current_power: f32,
    /// Target power (unramped, 0.0-1.0).
    target_power: f32,
    /// Whether the blower is toggled on.
    active: bool,
    /// Debounce for action_a toggle.
    action_a_prev: bool,
    /// Last update time for dt calculation.
    last_update: Option<Instant>,
}

impl Blower {
    pub fn new(config: BlowerConfig) -> Self {
        let can_id = config.can_id;
        Self {
            info: ToolInfo {
                slot: can_id,
                tool_type: ToolType::Blower,
                capabilities: Capabilities::MOTOR_CONTROL,
                serial: 0,
                name: "Blower",
            },
            config,
            current_power: 0.0,
            target_power: 0.0,
            active: false,
            action_a_prev: false,
            last_update: None,
        }
    }

    /// Build a VESC SetDuty CAN frame.
    ///
    /// VESC extended frame: ID = (command_id << 8) | vesc_id
    /// SetDuty = command 0x00, payload = duty * 100000 as i32 big-endian
    fn build_duty_frame(&self, duty: f32) -> Frame {
        let id: u32 = self.config.can_id as u32; // SetDuty cmd = 0, so (0 << 8) | id = id
        let duty_raw = (duty.clamp(0.0, 1.0) * 100_000.0) as i32;
        Frame::new_extended(id, &duty_raw.to_be_bytes())
    }
}

impl Tool for Blower {
    fn info(&self) -> &ToolInfo {
        &self.info
    }

    fn update(&mut self, input: &ToolCommand) -> ToolOutput {
        // Calculate dt from last update
        let now = Instant::now();
        let dt = self
            .last_update
            .map(|last| now.duration_since(last).as_secs_f32())
            .unwrap_or(0.01); // Default 10ms on first call
        self.last_update = Some(now);

        // Toggle active on action_a rising edge
        if input.action_a && !self.action_a_prev {
            self.active = !self.active;
        }
        self.action_a_prev = input.action_a;

        // Set target power
        if self.active {
            self.target_power = input.motor.clamp(0.0, 1.0);
        } else {
            self.target_power = 0.0;
        }

        // Ramp toward target
        let max_delta = self.config.power_ramp_rate / 100.0 * dt;
        let diff = self.target_power - self.current_power;
        if diff.abs() <= max_delta {
            self.current_power = self.target_power;
        } else if diff > 0.0 {
            self.current_power += max_delta;
        } else {
            // Ramp down faster (2x) for safety
            self.current_power -= max_delta * 2.0;
        }
        self.current_power = self.current_power.clamp(0.0, 1.0);

        debug!(
            power = format!("{:.1}%", self.current_power * 100.0),
            target = format!("{:.1}%", self.target_power * 100.0),
            active = self.active,
            "Blower update"
        );

        ToolOutput::Raw(self.build_duty_frame(self.current_power))
    }

    fn handle_status(&mut self, data: &[u8]) {
        // VESC status frames are handled by Drivetrain, not the tool.
        // If we wanted blower VESC telemetry, we'd parse STATUS1/4/5 here.
        let _ = data;
    }

    fn status(&self) -> ToolStatus {
        ToolStatus {
            name: self.info.name,
            position: None,
            active: self.active,
            current: None,
            fault: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_blower() -> Blower {
        Blower::new(BlowerConfig::default())
    }

    #[test]
    fn test_info() {
        let b = default_blower();
        assert_eq!(b.info().tool_type, ToolType::Blower);
        assert_eq!(b.info().name, "Blower");
        assert_eq!(b.info().slot, 0x0C);
    }

    #[test]
    fn test_toggle_on_action_a() {
        let mut b = default_blower();
        assert!(!b.active);

        // Rising edge toggles on
        b.update(&ToolCommand {
            motor: 0.5,
            action_a: true,
            ..Default::default()
        });
        assert!(b.active);

        // Held down — no change
        b.update(&ToolCommand {
            motor: 0.5,
            action_a: true,
            ..Default::default()
        });
        assert!(b.active);

        // Release then press again — toggles off
        b.update(&ToolCommand {
            motor: 0.5,
            action_a: false,
            ..Default::default()
        });
        b.update(&ToolCommand {
            motor: 0.5,
            action_a: true,
            ..Default::default()
        });
        assert!(!b.active);
    }

    #[test]
    fn test_inactive_sends_zero() {
        let mut b = default_blower();
        // Not active, motor at 100%
        let output = b.update(&ToolCommand {
            motor: 1.0,
            action_a: false,
            ..Default::default()
        });
        assert_eq!(b.target_power, 0.0);
        // Should produce a raw frame with 0 duty
        match output {
            ToolOutput::Raw(frame) => {
                let duty = i32::from_be_bytes([
                    frame.data[0],
                    frame.data[1],
                    frame.data[2],
                    frame.data[3],
                ]);
                assert_eq!(duty, 0);
            }
            _ => panic!("Expected Raw output"),
        }
    }

    #[test]
    fn test_duty_frame_format() {
        let b = default_blower();
        let frame = b.build_duty_frame(0.5);

        // CAN ID: 0x0C (SetDuty cmd 0 << 8 | 12)
        assert_eq!(frame.id, 0x0C);
        assert!(frame.extended);

        // Duty: 0.5 * 100000 = 50000
        let duty = i32::from_be_bytes([
            frame.data[0],
            frame.data[1],
            frame.data[2],
            frame.data[3],
        ]);
        assert_eq!(duty, 50000);
    }

    #[test]
    fn test_power_ramping() {
        let mut b = default_blower();

        // Toggle on and set motor to 50%
        b.update(&ToolCommand {
            motor: 0.5,
            action_a: true,
            ..Default::default()
        });

        // Power should start ramping, not jump immediately
        assert!(b.current_power < 0.5);
        assert!(b.current_power >= 0.0);
    }

    #[test]
    fn test_status() {
        let mut b = default_blower();
        b.active = true;
        let s = b.status();
        assert_eq!(s.name, "Blower");
        assert!(s.active);
        assert!(s.position.is_none());
    }
}
