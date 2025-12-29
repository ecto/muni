//! ESP32-S3 firmware with animated OLED display and LED control
//! For Heltec LoRa 32 V3 and compatible boards
//!
//! Serial Commands (over USB):
//!   led <r>,<g>,<b>     - Set LED color (0-255 each)
//!   led off             - Turn off LEDs
//!   state <s>           - Set state (idle, running, error)
//!   help                - Show commands

#![no_std]
#![no_main]

mod ui;
mod ws2812;

use embedded_io::ReadReady;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::RateExtU32,
    uart::{Config as UartConfig, Uart},
};
use esp_println::println;
use heapless::String;
use smart_leds::RGB8;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use ui::{render, DeviceState, UiState};

// LED strip configuration
const NUM_LEDS: usize = 4; // Number of LEDs in strip

// Timing constants
const FRAME_DELAY_MS: u32 = 33; // ~30 fps
const PAGE_CYCLE_FRAMES: u32 = 90; // ~3 seconds per page

/// Parse a serial command and update state
fn parse_command(cmd: &str, ui_state: &mut UiState, led_color: &mut RGB8) {
    let cmd = cmd.trim();

    if cmd.starts_with("led ") {
        let args = &cmd[4..];
        if args == "off" {
            *led_color = RGB8::new(0, 0, 0);
            println!("LED: off");
        } else {
            // Parse r,g,b manually to avoid heapless overflow
            let mut parts = args.split(',');
            let r_str = parts.next();
            let g_str = parts.next();
            let b_str = parts.next();
            let extra = parts.next(); // Should be None

            if let (Some(r_s), Some(g_s), Some(b_s), None) = (r_str, g_str, b_str, extra) {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    r_s.trim().parse::<u8>(),
                    g_s.trim().parse::<u8>(),
                    b_s.trim().parse::<u8>(),
                ) {
                    *led_color = RGB8::new(r, g, b);
                    println!("LED: {},{},{}", r, g, b);
                } else {
                    println!("Error: invalid RGB values");
                }
            } else {
                println!("Usage: led <r>,<g>,<b> or led off");
            }
        }
    } else if cmd.starts_with("state ") {
        let state_str = cmd[6..].trim();
        match state_str {
            "idle" => {
                ui_state.state = DeviceState::Idle;
                println!("State: IDLE");
            }
            "running" => {
                ui_state.state = DeviceState::Running;
                println!("State: RUNNING");
            }
            "error" => {
                ui_state.state = DeviceState::Error;
                println!("State: ERROR");
            }
            _ => {
                println!("Usage: state idle|running|error");
            }
        }
    } else if cmd == "rgb" {
        println!("Setting color order: RGB");
    } else if cmd == "grb" {
        println!("Setting color order: GRB");
    } else if cmd == "bgr" {
        println!("Setting color order: BGR");
    } else if cmd == "test" {
        println!("Testing LED signal...");
        // Return true to indicate we need to run the test
    } else if cmd == "help" || cmd == "?" {
        println!("Commands:");
        println!("  led <r>,<g>,<b>  - Set LED color (0-255)");
        println!("  led off          - Turn off LEDs");
        println!("  state <s>        - Set state (idle/running/error)");
        println!("  test             - Test LED signal");
        println!("  help             - Show this help");
    } else if !cmd.is_empty() {
        println!("Unknown command: {}", cmd);
        println!("Type 'help' for commands");
    }
}

#[main]
fn main() -> ! {
    println!("Muni MCU v0.1");
    println!("Type 'help' for commands");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // Enable Vext (power to OLED) - active low
    let vext = Output::new(peripherals.GPIO36, Level::Low);
    delay.delay_millis(50);

    // Reset OLED
    let mut oled_rst = Output::new(peripherals.GPIO21, Level::Low);
    delay.delay_millis(50);
    oled_rst.set_high();
    delay.delay_millis(50);

    // Initialize I2C for OLED
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(400u32.kHz()),
    )
    .unwrap()
    .with_sda(peripherals.GPIO17)
    .with_scl(peripherals.GPIO18);

    // Initialize OLED display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if display.init().is_ok() {

        // Show splash screen
        use embedded_graphics::{
            mono_font::{ascii::FONT_5X8, ascii::FONT_6X10, MonoTextStyleBuilder},
            pixelcolor::BinaryColor,
            prelude::*,
            primitives::{Line, PrimitiveStyle},
            text::Text,
        };

        display.clear_buffer();

        let tiny = MonoTextStyleBuilder::new()
            .font(&FONT_5X8)
            .text_color(BinaryColor::On)
            .build();
        let normal = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        // "The" (italic feel with small font)
        let _ = Text::new("The", Point::new(52, 8), tiny).draw(&mut display);

        // "Municipal Robotics" (main title)
        let _ = Text::new("MUNICIPAL", Point::new(22, 20), normal).draw(&mut display);
        let _ = Text::new("ROBOTICS", Point::new(28, 30), normal).draw(&mut display);

        // "Corporation of"
        let _ = Text::new("Corporation of", Point::new(28, 42), tiny).draw(&mut display);

        // "Cleveland, Ohio"
        let _ = Text::new("Cleveland, Ohio", Point::new(24, 52), normal).draw(&mut display);

        // Decorative lines
        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let _ = Line::new(Point::new(10, 56), Point::new(118, 56))
            .into_styled(line_style)
            .draw(&mut display);

        let _ = display.flush();
        delay.delay_millis(2000); // Show splash for 2 seconds
    }

    // Initialize UART0 for command input (CP2102 USB-to-UART on Heltec V3)
    // TX=GPIO43, RX=GPIO44 (directly connected to CP2102)
    let mut uart0 = Uart::new(
        peripherals.UART0,
        UartConfig::default(),
    )
    .unwrap()
    .with_rx(peripherals.GPIO44)
    .with_tx(peripherals.GPIO43);

    // Initialize LED output (GPIO4)
    let mut led_pin = Output::new(peripherals.GPIO4, Level::Low);

    // Flash LEDs briefly on startup
    ws2812::fill_leds(&mut led_pin, &delay, RGB8::new(64, 0, 0), NUM_LEDS);
    delay.delay_millis(500);
    ws2812::fill_leds(&mut led_pin, &delay, RGB8::new(0, 0, 0), NUM_LEDS);

    // Keep power pins alive
    core::mem::forget(vext);
    core::mem::forget(oled_rst);

    // Initialize UI state
    let mut ui_state = UiState {
        device_name: "ATTACHMENT",
        state: DeviceState::Idle,
        device_id: 0x0A01,
        version: "v0.1",
        can_bitrate_k: 500,
        ..Default::default()
    };

    // LED state
    let mut led_color = RGB8::new(0, 0, 0);
    let mut last_led_color = led_color;

    // Debug mode: auto-cycle colors (disabled - need level shifter for WS2811)
    let debug_led_mode = false;
    let mut debug_color_index: u8 = 0;

    // Command buffer
    let mut cmd_buf: String<64> = String::new();

    let mut frame_counter: u32 = 0;
    let mut last_second: u32 = 0;

    // Force initial render
    render(&mut display, &ui_state);
    let _ = display.flush();

    loop {
        // Check for serial input (non-blocking)
        if uart0.read_ready().unwrap_or(false) {
            let mut byte_buf = [0u8; 1];
            if let Ok(count) = embedded_io::Read::read(&mut uart0, &mut byte_buf) {
                if count > 0 {
                    let ch = byte_buf[0] as char;
                    if ch == '\n' || ch == '\r' {
                        if !cmd_buf.is_empty() {
                            parse_command(&cmd_buf, &mut ui_state, &mut led_color);
                            cmd_buf.clear();
                        }
                    } else if ch.is_ascii() && !ch.is_control() {
                        let _ = cmd_buf.push(ch);
                    }
                }
            }
        }

        // Update animation state
        ui_state.tick();

        // Update uptime every ~30 frames (1 second)
        let current_second = frame_counter / 30;
        if current_second != last_second {
            ui_state.uptime_secs = current_second;
            last_second = current_second;

            // Debug mode: cycle LED colors every second
            if debug_led_mode {
                debug_color_index = (debug_color_index + 1) % 8;
                led_color = match debug_color_index {
                    0 => { println!("Debug LED: RED"); RGB8::new(255, 0, 0) }
                    1 => { println!("Debug LED: GREEN"); RGB8::new(0, 255, 0) }
                    2 => { println!("Debug LED: BLUE"); RGB8::new(0, 0, 255) }
                    3 => { println!("Debug LED: YELLOW"); RGB8::new(255, 255, 0) }
                    4 => { println!("Debug LED: CYAN"); RGB8::new(0, 255, 255) }
                    5 => { println!("Debug LED: MAGENTA"); RGB8::new(255, 0, 255) }
                    6 => { println!("Debug LED: WHITE"); RGB8::new(255, 255, 255) }
                    _ => { println!("Debug LED: OFF"); RGB8::new(0, 0, 0) }
                };
            }
        }

        // Cycle bottom bar page every PAGE_CYCLE_FRAMES
        if frame_counter % PAGE_CYCLE_FRAMES == 0 && frame_counter > 0 {
            ui_state.next_page();
        }

        // Update LEDs if color changed
        if led_color.r != last_led_color.r 
            || led_color.g != last_led_color.g 
            || led_color.b != last_led_color.b 
        {
            ws2812::fill_leds(&mut led_pin, &delay, led_color, NUM_LEDS);
            last_led_color = led_color;
        }

        // Render UI
        render(&mut display, &ui_state);

        // Flush to display
        let _ = display.flush();

        // Frame delay
        delay.delay_millis(FRAME_DELAY_MS);
        frame_counter = frame_counter.wrapping_add(1);
    }
}
