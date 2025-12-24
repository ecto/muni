//! Rover LED controller firmware
//!
//! Drives WS2811 LED strip via PIO on GP0.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program, Grb};
use embassy_time::{Duration, Ticker, Timer};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

use mcu_leds::{LedController, LedMode};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

// Program metadata for `picotool info`.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Rover LEDs"),
    embassy_rp::binary_info::rp_program_description!(c"WS2811 LED controller for Muni rover"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// Number of addressable LED units (WS2811 12V has 3 LEDs per IC).
/// Your strip has 4 sections = 4 addressable units.
const NUM_LEDS: usize = 4;

/// LED update rate (~30 FPS).
const LED_UPDATE_MS: u64 = 33;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Rover LED controller starting ===");

    // Initialize PIO for WS2812/WS2811
    info!("Initializing PIO...");
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    // Load the WS2812 program into PIO
    info!("Loading WS2812 program...");
    let prg = PioWs2812Program::new(&mut common);

    // Create WS2812 driver on GP0
    info!("Creating WS2812 driver on GP0...");
    let mut ws2812: PioWs2812<'_, PIO0, 0, NUM_LEDS, Grb> =
        PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_0, &prg);

    info!("WS2812 initialized!");

    // Test pattern: cycle through colors
    info!("Starting test pattern: Red -> Green -> Blue");

    // Red
    let red: [RGB8; NUM_LEDS] = [RGB8::new(255, 0, 0); NUM_LEDS];
    ws2812.write(&red).await;
    info!("RED");
    Timer::after(Duration::from_secs(2)).await;

    // Green
    let green: [RGB8; NUM_LEDS] = [RGB8::new(0, 255, 0); NUM_LEDS];
    ws2812.write(&green).await;
    info!("GREEN");
    Timer::after(Duration::from_secs(2)).await;

    // Blue
    let blue: [RGB8; NUM_LEDS] = [RGB8::new(0, 0, 255); NUM_LEDS];
    ws2812.write(&blue).await;
    info!("BLUE");
    Timer::after(Duration::from_secs(2)).await;

    info!("Test pattern complete, starting animation loop");

    // Initialize LED controller for animations
    let mut leds = LedController::<NUM_LEDS>::new();

    // LED update ticker
    let mut led_ticker = Ticker::every(Duration::from_millis(LED_UPDATE_MS));

    // Cycle through different modes for demo
    let modes = [
        LedMode::idle(),       // Solid green
        LedMode::teleop(),     // Pulsing blue
        LedMode::autonomous(), // Pulsing cyan
        LedMode::estop(),      // Flashing red
    ];
    let mut mode_index = 0;
    let mut mode_ticks = 0u32;
    let mode_duration_ticks = 150; // ~5 seconds per mode at 30fps

    leds.set_mode(modes[mode_index]);
    info!("Starting mode: idle (green)");

    loop {
        led_ticker.next().await;

        // Update LED buffer
        let buffer = leds.update();

        // Write to LED strip
        ws2812.write(buffer).await;

        // Cycle modes for demo
        mode_ticks += 1;
        if mode_ticks >= mode_duration_ticks {
            mode_ticks = 0;
            mode_index = (mode_index + 1) % modes.len();
            leds.set_mode(modes[mode_index]);
            info!("Switched to mode {}", mode_index);
        }
    }
}
