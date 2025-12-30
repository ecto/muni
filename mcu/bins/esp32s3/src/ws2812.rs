//! WS2811/WS2812 LED driver using ESP32 RMT peripheral
//!
//! Uses the RMT (Remote Control Transceiver) for precise pulse timing.
//! Software bit-banging doesn't work reliably because ESP32's delay
//! functions lack nanosecond precision.

use esp_hal::rmt::{PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator};
use esp_hal::Blocking;
use smart_leds::RGB8;

/// WS2812/SK6812 timing at 80MHz RMT clock (12.5ns per tick)
/// 800kHz protocol
const SK68_T0H: u16 = 32;   // 0.4μs = 32 ticks
const SK68_T0L: u16 = 68;   // 0.85μs = 68 ticks
const SK68_T1H: u16 = 64;   // 0.8μs = 64 ticks
const SK68_T1L: u16 = 36;   // 0.45μs = 36 ticks

/// WS2811 timing at 80MHz (400kHz protocol)
const WS2811_T0H: u16 = 40;   // 0.5μs = 40 ticks
const WS2811_T0L: u16 = 160;  // 2.0μs = 160 ticks
const WS2811_T1H: u16 = 96;   // 1.2μs = 96 ticks
const WS2811_T1L: u16 = 104;  // 1.3μs = 104 ticks

/// Color order for different LED chip variants
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorOrder {
    Rgb,
    Grb,
    Bgr,
}

impl Default for ColorOrder {
    fn default() -> Self {
        Self::Grb
    }
}

/// LED timing profile
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedTiming {
    /// SK6812, WS2812B (800kHz) - most common 5V strips
    Sk68xx,
    /// WS2811 (400kHz) - common for 12V strips
    Ws2811,
}

impl Default for LedTiming {
    fn default() -> Self {
        Self::Sk68xx
    }
}

impl LedTiming {
    fn t0h(&self) -> u16 {
        match self {
            LedTiming::Sk68xx => SK68_T0H,
            LedTiming::Ws2811 => WS2811_T0H,
        }
    }
    fn t0l(&self) -> u16 {
        match self {
            LedTiming::Sk68xx => SK68_T0L,
            LedTiming::Ws2811 => WS2811_T0L,
        }
    }
    fn t1h(&self) -> u16 {
        match self {
            LedTiming::Sk68xx => SK68_T1H,
            LedTiming::Ws2811 => WS2811_T1H,
        }
    }
    fn t1l(&self) -> u16 {
        match self {
            LedTiming::Sk68xx => SK68_T1L,
            LedTiming::Ws2811 => WS2811_T1L,
        }
    }
}

/// LED strip controller using RMT peripheral
pub struct LedStrip<'d, const N: usize> {
    channel: Option<esp_hal::rmt::Channel<Blocking, 0>>,
    buffer: [RGB8; N],
    color_order: ColorOrder,
    timing: LedTiming,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, const N: usize> LedStrip<'d, N> {
    /// Create a new LED strip controller
    pub fn new(rmt: Rmt<'d, Blocking>, pin: esp_hal::gpio::GpioPin<4>) -> Self {
        let tx_config = TxChannelConfig {
            clk_divider: 1,
            idle_output_level: false,
            idle_output: true,
            carrier_modulation: false,
            carrier_high: 1,
            carrier_low: 1,
            carrier_level: false,
        };

        let channel = rmt.channel0.configure(pin, tx_config).unwrap();

        Self {
            channel: Some(channel),
            buffer: [RGB8::default(); N],
            color_order: ColorOrder::default(),
            timing: LedTiming::default(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Set color order
    pub fn set_color_order(&mut self, order: ColorOrder) {
        self.color_order = order;
    }

    /// Set LED timing profile
    pub fn set_timing(&mut self, timing: LedTiming) {
        self.timing = timing;
    }

    /// Fill all LEDs with a single color
    pub fn fill(&mut self, color: RGB8) {
        self.buffer.fill(color);
    }

    /// Set a specific LED
    #[allow(dead_code)]
    pub fn set(&mut self, index: usize, color: RGB8) {
        if index < N {
            self.buffer[index] = color;
        }
    }

    /// Write the buffer to the LED strip
    pub fn show(&mut self) {
        // Each LED needs 24 bits = 24 pulse codes (u32 each)
        // Plus 1 for the end marker
        // Max 64 LEDs supported
        const MAX_PULSES: usize = 64 * 24 + 1;
        let mut pulses: [u32; MAX_PULSES] = [0; MAX_PULSES];

        let num_pulses = N * 24;
        if num_pulses >= MAX_PULSES {
            return;
        }

        let mut pulse_idx = 0;
        for color in self.buffer.iter() {
            // Reorder color bytes based on color order
            let (b1, b2, b3) = match self.color_order {
                ColorOrder::Rgb => (color.r, color.g, color.b),
                ColorOrder::Grb => (color.g, color.r, color.b),
                ColorOrder::Bgr => (color.b, color.g, color.r),
            };

            // Convert each byte to 8 pulse codes
            Self::byte_to_pulses(b1, self.timing, &mut pulses[pulse_idx..pulse_idx + 8]);
            pulse_idx += 8;
            Self::byte_to_pulses(b2, self.timing, &mut pulses[pulse_idx..pulse_idx + 8]);
            pulse_idx += 8;
            Self::byte_to_pulses(b3, self.timing, &mut pulses[pulse_idx..pulse_idx + 8]);
            pulse_idx += 8;
        }

        // End marker (all zeros)
        pulses[pulse_idx] = u32::empty();

        // Transmit
        if let Some(channel) = self.channel.take() {
            match channel.transmit(&pulses[..pulse_idx + 1]) {
                Ok(transaction) => {
                    match transaction.wait() {
                        Ok(ch) => self.channel = Some(ch),
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        }
    }

    /// Convert a byte to 8 RMT pulse codes (MSB first)
    fn byte_to_pulses(byte: u8, timing: LedTiming, pulses: &mut [u32]) {
        for i in 0..8 {
            let bit = (byte >> (7 - i)) & 1 != 0;
            pulses[i] = if bit {
                u32::new(true, timing.t1h(), false, timing.t1l())
            } else {
                u32::new(true, timing.t0h(), false, timing.t0l())
            };
        }
    }
}
