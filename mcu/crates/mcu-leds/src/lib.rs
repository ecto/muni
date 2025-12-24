//! LED controller using WS2812 via PIO.
//!
//! Supports multiple animation modes: solid, pulse, chase, flash.

#![no_std]

use embassy_rp::pio::Instance;
use embassy_time::Instant;
use smart_leds::RGB8;

pub mod animations;

/// LED mode with parameters.
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
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

/// LED controller state.
pub struct LedController<const N: usize> {
    mode: LedMode,
    buffer: [RGB8; N],
    start_time: Instant,
}

impl<const N: usize> LedController<N> {
    /// Create a new LED controller.
    pub fn new() -> Self {
        Self {
            mode: LedMode::Off,
            buffer: [RGB8::default(); N],
            start_time: Instant::now(),
        }
    }

    /// Set the LED mode.
    pub fn set_mode(&mut self, mode: LedMode) {
        if self.mode != mode {
            self.mode = mode;
            self.start_time = Instant::now();
            defmt::info!("LED mode changed: {:?}", mode);
        }
    }

    /// Get current mode.
    pub fn mode(&self) -> LedMode {
        self.mode
    }

    /// Update the LED buffer based on current mode and elapsed time.
    /// Returns the buffer to be sent to the LED strip.
    pub fn update(&mut self) -> &[RGB8; N] {
        let elapsed_ms = self.start_time.elapsed().as_millis() as u32;

        match self.mode {
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
                let scaled_brightness = ((brightness as u32 * phase) / 255) as u8;
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

        &self.buffer
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
