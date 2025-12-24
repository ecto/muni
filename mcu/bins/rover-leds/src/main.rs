//! Rover LED controller firmware.
//!
//! Runs on Pico 2 (RP2350), receives LED commands via CAN (MCP2515),
//! and drives a WS2812 LED strip via PIO.

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIO0, SPI0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::spi::{Config as SpiConfig, Spi};
use embassy_time::{Duration, Ticker, Timer};
use smart_leds::SmartLedsWrite;
use ws2812_pio::Ws2812;

use mcu_core::heartbeat::Heartbeat;
use mcu_core::protocol::{peripheral, LedCommand};
use mcu_core::watchdog::Watchdog;
use mcu_leds::{LedController, LedMode};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

/// Number of LEDs in the strip.
const NUM_LEDS: usize = 60;

/// Command timeout before reverting to safe state.
const WATCHDOG_TIMEOUT_MS: u64 = 5000;

/// LED update rate (~60 FPS).
const LED_UPDATE_MS: u64 = 16;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    defmt::info!("Rover LED controller starting...");

    // Initialize SPI for MCP2515
    // GP16 = MISO, GP17 = CS, GP18 = SCK, GP19 = MOSI, GP20 = INT
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = 1_000_000; // 1 MHz for MCP2515

    let spi = Spi::new(
        p.SPI0,
        p.PIN_18, // SCK
        p.PIN_19, // MOSI
        p.PIN_16, // MISO
        p.DMA_CH0,
        p.DMA_CH1,
        spi_config,
    );

    let cs = Output::new(p.PIN_17, Level::High);
    let int = Input::new(p.PIN_20, Pull::Up);

    defmt::info!("SPI initialized");

    // Initialize PIO for WS2812
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut ws2812 = Ws2812::new(&mut common, sm0, p.DMA_CH2, p.PIN_0);

    defmt::info!("WS2812 PIO initialized");

    // Initialize LED controller
    let mut leds = LedController::<NUM_LEDS>::new();

    // Initialize heartbeat
    let mut heartbeat = Heartbeat::default_interval();
    let mut heartbeat_ticker = heartbeat.ticker();

    // Initialize watchdog
    let mut watchdog = Watchdog::new(Duration::from_millis(WATCHDOG_TIMEOUT_MS));

    // LED update ticker
    let mut led_ticker = Ticker::every(Duration::from_millis(LED_UPDATE_MS));

    // Start with idle state
    leds.set_mode(LedMode::idle());

    defmt::info!("Entering main loop");

    // Note: CAN initialization commented out until MCP2515 driver is properly integrated
    // For now, we'll just run the LED animations
    //
    // TODO: Initialize MCP2515 CAN controller:
    // let mut can = CanBus::new(spi, cs, int).await.expect("CAN init failed");
    // can.set_filter(peripheral::LED_CMD).ok();

    loop {
        // Wait for next LED update tick
        led_ticker.next().await;

        // Check watchdog
        if watchdog.check() {
            // Watchdog triggered - go to fault state
            if leds.mode() != LedMode::fault() {
                defmt::warn!("Watchdog triggered, entering fault state");
                leds.set_mode(LedMode::fault());
                heartbeat.set_fault(true);
            }
        }

        // Update LED buffer
        let buffer = leds.update();

        // Write to LED strip
        ws2812.write(buffer.iter().copied()).ok();

        // Send heartbeat periodically
        // TODO: Send via CAN when integrated
        // For now just log it
        if heartbeat_ticker.next().now_or_never().is_some() {
            let status = heartbeat.status();
            defmt::debug!(
                "Heartbeat: status={}, uptime={}s",
                status.status,
                status.uptime_secs
            );
        }
    }
}

/// Parse LED command from CAN frame data.
fn parse_led_command(data: &[u8]) -> Option<LedMode> {
    let cmd = LedCommand::from_bytes(data)?;
    Some(LedMode::from_command(
        cmd.mode as u8,
        cmd.r,
        cmd.g,
        cmd.b,
        cmd.brightness,
        cmd.period_ms,
    ))
}

// Placeholder for CAN receive loop (to be enabled when MCP2515 driver is integrated)
/*
async fn can_receive_loop(
    mut can: CanBus<'_>,
    leds: &mut LedController<NUM_LEDS>,
    watchdog: &mut Watchdog,
) {
    loop {
        match can.try_receive() {
            Ok(Some(frame)) => {
                if frame.id == peripheral::LED_CMD {
                    if let Some(mode) = parse_led_command(&frame.data) {
                        leds.set_mode(mode);
                        watchdog.feed();
                    }
                }
            }
            Ok(None) => {
                // No message, yield
                Timer::after(Duration::from_millis(1)).await;
            }
            Err(e) => {
                defmt::error!("CAN receive error");
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}
*/
