//! LED controller using WS2812 via PIO.
//!
//! Supports multiple animation modes: solid, pulse, chase, flash.
//! Includes smooth wipe transitions between modes.

#![no_std]

use embassy_time::Instant;
use smart_leds::RGB8;

pub mod animations;

/// LED mode with parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LedMode {
    /// All LEDs off.
    Off,
    /// Solid color.
    Solid { color: RGB8, brightness: u8 },
    /// Pulsing/breathing effect.
    Pulse {
        color: RGB8,
        brightness: u8,
        period_ms: u16,
    },
    /// Chase/running effect.
    Chase {
        color: RGB8,
        brightness: u8,
        period_ms: u16,
    },
    /// Flashing/strobe effect.
    Flash {
        color: RGB8,
        brightness: u8,
        period_ms: u16,
    },
}

impl Default for LedMode {
    fn default() -> Self {
        Self::Off
    }
}

impl LedMode {
    /// Create from protocol command.
    pub fn from_command(
        mode: u8,
        r: u8,
        g: u8,
        b: u8,
        brightness: u8,
        period_ms: u16,
    ) -> Self {
        let color = RGB8::new(r, g, b);
        match mode {
            0x01 => Self::Solid { color, brightness },
            0x02 => Self::Pulse {
                color,
                brightness,
                period_ms,
            },
            0x03 => Self::Chase {
                color,
                brightness,
                period_ms,
            },
            0x04 => Self::Flash {
                color,
                brightness,
                period_ms,
            },
            _ => Self::Off,
        }
    }

    /// Get the base color for this mode (used for transitions).
    pub fn base_color(&self) -> RGB8 {
        match self {
            LedMode::Off => RGB8::default(),
            LedMode::Solid { color, brightness } => scale_color(*color, *brightness),
            LedMode::Pulse { color, brightness, .. } => scale_color(*color, *brightness),
            LedMode::Chase { color, brightness, .. } => scale_color(*color, *brightness),
            LedMode::Flash { color, brightness, .. } => scale_color(*color, *brightness),
        }
    }

    /// Convenience constructors for rover states.
    pub fn idle() -> Self {
        Self::Solid {
            color: RGB8::new(0, 255, 0), // Green
            brightness: 128,
        }
    }

    pub fn teleop() -> Self {
        Self::Pulse {
            color: RGB8::new(0, 100, 255), // Blue
            brightness: 200,
            period_ms: 2000,
        }
    }

    pub fn autonomous() -> Self {
        Self::Pulse {
            color: RGB8::new(0, 255, 200), // Cyan
            brightness: 200,
            period_ms: 1500,
        }
    }

    pub fn estop() -> Self {
        Self::Flash {
            color: RGB8::new(255, 0, 0), // Red
            brightness: 255,
            period_ms: 200,
        }
    }

    pub fn fault() -> Self {
        Self::Flash {
            color: RGB8::new(255, 100, 0), // Orange
            brightness: 255,
            period_ms: 500,
        }
    }
}

/// Controller state for transitions.
#[derive(Clone, Copy, PartialEq)]
enum ControllerState {
    /// Running the current mode's animation.
    Running,
    /// Playing a wipe transition to the new mode.
    Transitioning,
}

/// LED controller state with transition support.
pub struct LedController<const N: usize> {
    current_mode: LedMode,
    target_mode: LedMode,
    state: ControllerState,
    buffer: [RGB8; N],
    transition_buffer: [RGB8; N], // Snapshot of buffer at transition start
    mode_start_time: Instant,
    transition_start_time: Instant,
    transition_duration_ms: u32,
}

impl<const N: usize> LedController<N> {
    /// Create a new LED controller.
    pub fn new() -> Self {
        Self {
            current_mode: LedMode::Off,
            target_mode: LedMode::Off,
            state: ControllerState::Running,
            buffer: [RGB8::default(); N],
            transition_buffer: [RGB8::default(); N],
            mode_start_time: Instant::now(),
            transition_start_time: Instant::now(),
            transition_duration_ms: animations::WIPE_DURATION_MS,
        }
    }

    /// Set the LED mode. Triggers a wipe transition if mode changes.
    pub fn set_mode(&mut self, mode: LedMode) {
        if self.target_mode != mode {
            self.target_mode = mode;

            // Snapshot current buffer for transition
            self.transition_buffer = self.buffer;
            self.transition_start_time = Instant::now();
            self.state = ControllerState::Transitioning;
        }
    }

    /// Set the LED mode immediately without transition.
    pub fn set_mode_immediate(&mut self, mode: LedMode) {
        self.current_mode = mode;
        self.target_mode = mode;
        self.state = ControllerState::Running;
        self.mode_start_time = Instant::now();
    }

    /// Set transition duration.
    pub fn set_transition_duration(&mut self, duration_ms: u32) {
        self.transition_duration_ms = duration_ms;
    }

    /// Get current mode.
    pub fn mode(&self) -> LedMode {
        self.current_mode
    }

    /// Get target mode (what we're transitioning to).
    pub fn target_mode(&self) -> LedMode {
        self.target_mode
    }

    /// Check if currently in a transition.
    pub fn is_transitioning(&self) -> bool {
        self.state == ControllerState::Transitioning
    }

    /// Update the LED buffer based on current mode and elapsed time.
    /// Returns the buffer to be sent to the LED strip.
    pub fn update(&mut self) -> &[RGB8; N] {
        match self.state {
            ControllerState::Transitioning => {
                self.update_transition();
            }
            ControllerState::Running => {
                self.update_mode();
            }
        }

        &self.buffer
    }

    /// Update during transition.
    fn update_transition(&mut self) {
        let elapsed_ms = self.transition_start_time.elapsed().as_millis() as u32;

        if elapsed_ms >= self.transition_duration_ms {
            // Transition complete
            self.current_mode = self.target_mode;
            self.state = ControllerState::Running;
            self.mode_start_time = Instant::now();
            self.update_mode();
            return;
        }

        // Calculate wipe position
        let (_, position) = animations::wipe_progress(elapsed_ms, self.transition_duration_ms, N);

        // Edge width for soft transition (about 10% of strip)
        let edge_width = (N / 10).max(2);

        // Get target color for the wipe
        let target_color = self.target_mode.base_color();

        // Apply wipe with soft edge
        animations::apply_wipe_soft(
            &mut self.buffer,
            &self.transition_buffer,
            target_color,
            position,
            edge_width,
        );
    }

    /// Update the current mode animation.
    fn update_mode(&mut self) {
        let elapsed_ms = self.mode_start_time.elapsed().as_millis() as u32;

        match self.current_mode {
            LedMode::Off => {
                self.buffer.fill(RGB8::default());
            }
            LedMode::Solid { color, brightness } => {
                let scaled = scale_color(color, brightness);
                self.buffer.fill(scaled);
            }
            LedMode::Pulse {
                color,
                brightness,
                period_ms,
            } => {
                let phase = animations::pulse_phase(elapsed_ms, period_ms as u32);
                let scaled_brightness = ((brightness as u32 * phase as u32) / 255) as u8;
                let scaled = scale_color(color, scaled_brightness);
                self.buffer.fill(scaled);
            }
            LedMode::Chase {
                color,
                brightness,
                period_ms,
            } => {
                let scaled = scale_color(color, brightness);
                animations::chase(&mut self.buffer, scaled, elapsed_ms, period_ms as u32);
            }
            LedMode::Flash {
                color,
                brightness,
                period_ms,
            } => {
                let on = animations::flash_state(elapsed_ms, period_ms as u32);
                if on {
                    let scaled = scale_color(color, brightness);
                    self.buffer.fill(scaled);
                } else {
                    self.buffer.fill(RGB8::default());
                }
            }
        }
    }

    /// Get the LED buffer without updating.
    pub fn buffer(&self) -> &[RGB8; N] {
        &self.buffer
    }
}

impl<const N: usize> Default for LedController<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Scale a color by brightness (0-255).
fn scale_color(color: RGB8, brightness: u8) -> RGB8 {
    RGB8::new(
        ((color.r as u16 * brightness as u16) / 255) as u8,
        ((color.g as u16 * brightness as u16) / 255) as u8,
        ((color.b as u16 * brightness as u16) / 255) as u8,
    )
}

