//! LED animation helpers.

use smart_leds::RGB8;

/// Calculate pulse brightness (0-255) based on elapsed time and period.
/// Uses a sine-like curve for smooth breathing.
pub fn pulse_phase(elapsed_ms: u32, period_ms: u32) -> u8 {
    if period_ms == 0 {
        return 255;
    }

    // Phase in range [0, 1024) for fixed-point math
    let phase = ((elapsed_ms % period_ms) * 1024 / period_ms) as u16;

    // Triangle wave approximation of sine
    // 0-512: rising, 512-1024: falling
    let triangle = if phase < 512 {
        phase
    } else {
        1024 - phase
    };

    // Scale to 0-255
    (triangle / 2) as u8
}

/// Calculate chase pattern - a moving lit section.
pub fn chase<const N: usize>(
    buffer: &mut [RGB8; N],
    color: RGB8,
    elapsed_ms: u32,
    period_ms: u32,
) {
    if period_ms == 0 || N == 0 {
        buffer.fill(color);
        return;
    }

    // Width of the lit section (about 20% of strip)
    let width = (N / 5).max(1);

    // Position of the lit section
    let pos = ((elapsed_ms % period_ms) * N as u32 / period_ms) as usize;

    buffer.fill(RGB8::default());

    for i in 0..width {
        let idx = (pos + i) % N;
        // Fade based on position in the chase section
        let fade = 255 - (i * 255 / width) as u8;
        buffer[idx] = RGB8::new(
            ((color.r as u16 * fade as u16) / 255) as u8,
            ((color.g as u16 * fade as u16) / 255) as u8,
            ((color.b as u16 * fade as u16) / 255) as u8,
        );
    }
}

/// Calculate flash state (on/off) based on elapsed time and period.
pub fn flash_state(elapsed_ms: u32, period_ms: u32) -> bool {
    if period_ms == 0 {
        return true;
    }
    // 50% duty cycle
    (elapsed_ms % period_ms) < (period_ms / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_phase() {
        // At start, should be dim
        assert!(pulse_phase(0, 1000) < 10);
        // At quarter period, should be rising
        assert!(pulse_phase(250, 1000) > 50);
        // At half period, should be at peak
        assert!(pulse_phase(500, 1000) > 120);
        // At 3/4 period, should be falling
        assert!(pulse_phase(750, 1000) > 50);
    }

    #[test]
    fn test_flash_state() {
        assert!(flash_state(0, 200));
        assert!(flash_state(50, 200));
        assert!(!flash_state(100, 200));
        assert!(!flash_state(150, 200));
        assert!(flash_state(200, 200)); // Wraps
    }
}
