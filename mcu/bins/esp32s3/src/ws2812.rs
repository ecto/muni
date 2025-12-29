//! WS2811 LED driver - trying inverted timing
//!
//! Some WS2811 variants interpret bits differently.
//! Trying: longer LOW for 1, longer HIGH for 0 (inverted from typical)

use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use smart_leds::RGB8;

/// Color order options
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ColorOrder {
    Rgb,
    Grb,
    Bgr,
}

/// Send a single bit - trying different timing
#[inline(always)]
fn send_bit(pin: &mut Output<'_>, delay: &Delay, bit: bool) {
    // Standard WS2811 timing but with cleaner edges
    if bit {
        // 1 bit: long high
        pin.set_high();
        delay.delay_micros(1);
        delay.delay_nanos(200);
        pin.set_low();
        delay.delay_micros(1);
    } else {
        // 0 bit: short high
        pin.set_high();
        delay.delay_nanos(500);
        pin.set_low();
        delay.delay_micros(2);
    }
}

/// Send a single byte (MSB first)
#[inline(always)]
fn send_byte(pin: &mut Output<'_>, delay: &Delay, byte: u8) {
    for i in (0..8).rev() {
        send_bit(pin, delay, (byte >> i) & 1 != 0);
    }
}

/// Send a color with specified order
#[inline(always)]
fn send_color(pin: &mut Output<'_>, delay: &Delay, color: RGB8, order: ColorOrder) {
    match order {
        ColorOrder::Rgb => {
            send_byte(pin, delay, color.r);
            send_byte(pin, delay, color.g);
            send_byte(pin, delay, color.b);
        }
        ColorOrder::Grb => {
            send_byte(pin, delay, color.g);
            send_byte(pin, delay, color.r);
            send_byte(pin, delay, color.b);
        }
        ColorOrder::Bgr => {
            send_byte(pin, delay, color.b);
            send_byte(pin, delay, color.g);
            send_byte(pin, delay, color.r);
        }
    }
}

/// Fill all LEDs with a single color (sends data 3 times for reliability)
pub fn fill_leds(pin: &mut Output<'_>, delay: &Delay, color: RGB8, count: usize) {
    // Send data multiple times to ensure it "sticks"
    for _ in 0..3 {
        critical_section::with(|_| {
            for _ in 0..count {
                send_color(pin, delay, color, ColorOrder::Rgb);
            }
        });
        
        pin.set_low();
        delay.delay_micros(100);
    }
}
