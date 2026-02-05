//! Headlight + LED strip role.
//!
//! Controls headlights via MOSFET PWM and optional WS2812 LED strips.
//!
//! State-linked behavior:
//! - Idle/Sleep/Disabled → lights off
//! - Teleop/Autonomous → lights on (full brightness)
//! - EStop → flash 200ms
//! - Fault → flash 500ms
//!
//! Manual override via ToolCommand:
//! - axis → brightness (i16, 0-32767 maps to 0-100%)
//! - action_a → toggle on/off

use super::{Role, RoverState};
use std::time::{Duration, Instant};

/// Light operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMode {
    Off,
    On,
    Flash(Duration),
}

pub struct LightsRole {
    /// Current mode (from state or manual).
    mode: LightMode,
    /// Manual brightness (0.0-1.0).
    brightness: f32,
    /// Whether lights are toggled on manually.
    manual_on: bool,
    /// Whether state-linked behavior is active (vs manual override).
    state_linked: bool,
    /// Current rover state.
    rover_state: RoverState,
    /// Flash timer.
    last_toggle: Instant,
    /// Current flash output state.
    flash_on: bool,
    /// GPIO pin for headlight MOSFET.
    _headlight_pin: u8,
    /// Number of headlight channels.
    _headlight_count: u8,
    /// GPIO pin for WS2812 data.
    _led_strip_pin: u8,
    /// Number of addressable LEDs.
    _led_count: u16,
}

impl LightsRole {
    pub fn new(headlight_pin: u8, headlight_count: u8, led_strip_pin: u8, led_count: u16) -> Self {
        Self {
            mode: LightMode::Off,
            brightness: 1.0,
            manual_on: false,
            state_linked: true,
            rover_state: RoverState::Idle,
            last_toggle: Instant::now(),
            flash_on: false,
            _headlight_pin: headlight_pin,
            _headlight_count: headlight_count,
            _led_strip_pin: led_strip_pin,
            _led_count: led_count,
        }
    }

    /// Get current effective duty cycle (0.0-1.0).
    pub fn duty(&self) -> f32 {
        match self.mode {
            LightMode::Off => 0.0,
            LightMode::On => self.brightness,
            LightMode::Flash(_) => {
                if self.flash_on {
                    self.brightness
                } else {
                    0.0
                }
            }
        }
    }

    /// Update mode from rover state.
    fn update_from_state(&mut self) {
        if !self.state_linked {
            return;
        }
        self.mode = match self.rover_state {
            RoverState::Idle | RoverState::Sleep | RoverState::Disabled => LightMode::Off,
            RoverState::Teleop | RoverState::Autonomous => LightMode::On,
            RoverState::EStop => LightMode::Flash(Duration::from_millis(200)),
            RoverState::Fault => LightMode::Flash(Duration::from_millis(500)),
        };
    }
}

impl Role for LightsRole {
    fn handle_command(&mut self, axis: i16, _motor: i16, action_a: bool, _action_b: bool) {
        // Toggle manual mode on action_a
        if action_a {
            self.manual_on = !self.manual_on;
            self.state_linked = false;
        }

        // Brightness from axis (0-32767 → 0.0-1.0)
        if axis > 0 {
            self.brightness = (axis as f32 / 32767.0).clamp(0.0, 1.0);
        }

        if !self.state_linked {
            self.mode = if self.manual_on {
                LightMode::On
            } else {
                LightMode::Off
            };
        }
    }

    fn on_state_change(&mut self, state: RoverState) {
        self.rover_state = state;
        // Re-enable state-linked mode on state change
        self.state_linked = true;
        self.update_from_state();
    }

    fn on_timeout(&mut self) {
        // On timeout, keep current light state (don't turn off headlights
        // just because commands stopped — the operator may still need them).
    }

    fn on_shutdown(&mut self) {
        // Safe shutdown: turn off everything
        self.mode = LightMode::Off;
        self.manual_on = false;
    }

    fn status(&self, buf: &mut [u8]) {
        if buf.len() >= 8 {
            buf[0] = match self.mode {
                LightMode::Off => 0,
                LightMode::On => 1,
                LightMode::Flash(_) => 2,
            };
            buf[1] = (self.brightness * 255.0) as u8;
            buf[2] = self.manual_on as u8;
            buf[3] = self.rover_state as u8;
            // [4-7] reserved
        }
    }

    fn tick(&mut self) {
        // Update flash animation
        if let LightMode::Flash(period) = self.mode {
            if self.last_toggle.elapsed() >= period {
                self.flash_on = !self.flash_on;
                self.last_toggle = Instant::now();
            }
        }

        // TODO: Actually write GPIO/PWM here using esp-idf-hal
        // For now, `duty()` returns the value that should be written.
        let _duty = self.duty();
    }
}
