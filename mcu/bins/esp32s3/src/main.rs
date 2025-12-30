//! ESP32-S3 firmware with animated OLED display and LED control
//! For Heltec LoRa 32 V3 and compatible boards
//!
//! Serial Commands (over USB):
//!   led <r>,<g>,<b>     - Set LED color (0-255 each) [addressable-leds]
//!   led off             - Turn off LEDs
//!   state <s>           - Set state (idle, running, error)
//!   help                - Show commands

#![no_std]
#![no_main]

mod ui;
#[cfg(feature = "addressable-leds")]
mod ws2812;
#[cfg(any(feature = "status-led", feature = "status-bar"))]
mod status_led;

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
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use ui::{render, DeviceState, UiState};

#[cfg(feature = "addressable-leds")]
use {
    esp_hal::rmt::Rmt,
    smart_leds::RGB8,
    ws2812::{ColorOrder, LedStrip, LedTiming},
};

#[cfg(feature = "status-led")]
use status_led::{StatusLed, StatusPattern};

#[cfg(feature = "status-bar")]
use status_led::{StatusBar, BarState};

// LED strip configuration (for addressable-leds feature)
#[cfg(feature = "addressable-leds")]
const NUM_LEDS: usize = 4;

// Timing constants
const FRAME_DELAY_MS: u32 = 33; // ~30 fps
const PAGE_CYCLE_FRAMES: u32 = 90; // ~3 seconds per page

/// Parse a serial command and update state
fn parse_command(
    cmd: &str,
    ui_state: &mut UiState,
    #[cfg(feature = "status-led")] status_pattern: &mut StatusPattern,
    #[cfg(feature = "status-bar")] bar_state: &mut BarState,
    #[cfg(feature = "addressable-leds")] led_color: &mut RGB8,
    #[cfg(feature = "addressable-leds")] color_order: &mut ColorOrder,
    #[cfg(feature = "addressable-leds")] timing: &mut LedTiming,
    #[cfg(feature = "addressable-leds")] force_update: &mut bool,
) {
    let cmd = cmd.trim();

    if cmd.starts_with("state ") {
        let state_str = cmd[6..].trim();
        match state_str {
            "idle" => {
                ui_state.state = DeviceState::Idle;
                #[cfg(feature = "status-led")]
                { *status_pattern = StatusPattern::SlowPulse; }
                #[cfg(feature = "status-bar")]
                { *bar_state = BarState::Idle; }
                println!("State: IDLE");
            }
            "running" => {
                ui_state.state = DeviceState::Running;
                #[cfg(feature = "status-led")]
                { *status_pattern = StatusPattern::FastPulse; }
                #[cfg(feature = "status-bar")]
                { *bar_state = BarState::Running; }
                println!("State: RUNNING");
            }
            "error" => {
                ui_state.state = DeviceState::Error;
                #[cfg(feature = "status-led")]
                { *status_pattern = StatusPattern::RapidFlash; }
                #[cfg(feature = "status-bar")]
                { *bar_state = BarState::Error; }
                println!("State: ERROR");
            }
            "warn" | "warning" => {
                #[cfg(feature = "status-bar")]
                { *bar_state = BarState::Warning; }
                println!("State: WARNING");
            }
            _ => {
                println!("Usage: state idle|running|error|warn");
            }
        }
    } else if cmd == "on" {
        #[cfg(feature = "status-led")]
        { *status_pattern = StatusPattern::Solid; }
        #[cfg(feature = "status-bar")]
        { *bar_state = BarState::AllOn; }
        println!("LED: on");
    } else if cmd == "off" {
        #[cfg(feature = "status-led")]
        { *status_pattern = StatusPattern::Off; }
        #[cfg(feature = "status-bar")]
        { *bar_state = BarState::Off; }
        println!("LED: off");
    } else if cmd == "blink" {
        #[cfg(feature = "status-led")]
        { *status_pattern = StatusPattern::DoubleBlink; }
        #[cfg(feature = "status-bar")]
        { *bar_state = BarState::Startup; } // Chase animation
        println!("LED: blink");
    }
    // Addressable LED commands
    #[cfg(feature = "addressable-leds")]
    {
        if cmd.starts_with("led ") {
            let args = &cmd[4..];
            if args == "off" {
                *led_color = RGB8::new(0, 0, 0);
                *force_update = true;
                println!("LED: off");
            } else {
                let mut parts = args.split(',');
                let r_str = parts.next();
                let g_str = parts.next();
                let b_str = parts.next();
                let extra = parts.next();

                if let (Some(r_s), Some(g_s), Some(b_s), None) = (r_str, g_str, b_str, extra) {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        r_s.trim().parse::<u8>(),
                        g_s.trim().parse::<u8>(),
                        b_s.trim().parse::<u8>(),
                    ) {
                        *led_color = RGB8::new(r, g, b);
                        *force_update = true;
                        println!("LED: {},{},{}", r, g, b);
                    } else {
                        println!("Error: invalid RGB values");
                    }
                } else {
                    println!("Usage: led <r>,<g>,<b> or led off");
                }
            }
        } else if cmd == "rgb" {
            *color_order = ColorOrder::Rgb;
            *force_update = true;
            println!("Color order: RGB");
        } else if cmd == "grb" {
            *color_order = ColorOrder::Grb;
            *force_update = true;
            println!("Color order: GRB");
        } else if cmd == "bgr" {
            *color_order = ColorOrder::Bgr;
            *force_update = true;
            println!("Color order: BGR");
        } else if cmd == "ws2812" || cmd == "sk68" {
            *timing = LedTiming::Sk68xx;
            *force_update = true;
            println!("Timing: SK68xx/WS2812 (800kHz)");
        } else if cmd == "ws2811" {
            *timing = LedTiming::Ws2811;
            *force_update = true;
            println!("Timing: WS2811 (400kHz)");
        }
    }

    if cmd == "help" || cmd == "?" {
        println!("Commands:");
        println!("  state <s>   - Set state (idle/running/error)");
        println!("  on/off      - LED on/off");
        println!("  blink       - LED double-blink pattern");
        #[cfg(feature = "addressable-leds")]
        {
            println!("  led r,g,b   - Set RGB color (0-255)");
            println!("  rgb/grb/bgr - Color order");
            println!("  ws2812/ws2811 - Timing");
        }
        println!("  help        - Show this help");
    }
}

#[main]
fn main() -> ! {
    println!("Muni MCU v0.1");
    #[cfg(feature = "status-led")]
    println!("  [status-led: GPIO35]");
    #[cfg(feature = "status-bar")]
    println!("  [status-bar: GPIO19,20,26,48,47]");
    #[cfg(feature = "addressable-leds")]
    println!("  [addressable-leds: GPIO4]");
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

        let _ = Text::new("The", Point::new(52, 8), tiny).draw(&mut display);
        let _ = Text::new("MUNICIPAL", Point::new(22, 20), normal).draw(&mut display);
        let _ = Text::new("ROBOTICS", Point::new(28, 30), normal).draw(&mut display);
        let _ = Text::new("Corporation of", Point::new(28, 42), tiny).draw(&mut display);
        let _ = Text::new("Cleveland, Ohio", Point::new(24, 52), normal).draw(&mut display);

        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let _ = Line::new(Point::new(10, 56), Point::new(118, 56))
            .into_styled(line_style)
            .draw(&mut display);

        let _ = display.flush();
        delay.delay_millis(2000);
    }

    // Initialize UART0 for command input
    let mut uart0 = Uart::new(peripherals.UART0, UartConfig::default())
        .unwrap()
        .with_rx(peripherals.GPIO44)
        .with_tx(peripherals.GPIO43);

    // === Status LED (GPIO35 onboard) ===
    #[cfg(feature = "status-led")]
    let mut status_led = {
        let pin = Output::new(peripherals.GPIO35, Level::Low);
        let mut led = StatusLed::new(pin);
        // Flash on startup
        led.on();
        delay.delay_millis(100);
        led.off();
        delay.delay_millis(100);
        led.on();
        delay.delay_millis(100);
        led.off();
        led.set_pattern(StatusPattern::SlowPulse);
        led
    };

    #[cfg(feature = "status-led")]
    let mut status_pattern = StatusPattern::SlowPulse;

    // === Status Bar (5 LEDs on breadboard) ===
    // GPIO19=Red, GPIO20=Yellow, GPIO26=Green, GPIO48=Blue, GPIO47=White
    #[cfg(feature = "status-bar")]
    let mut status_bar = {
        let red = Output::new(peripherals.GPIO19, Level::Low);
        let yellow = Output::new(peripherals.GPIO20, Level::Low);
        let green = Output::new(peripherals.GPIO26, Level::Low);
        let blue = Output::new(peripherals.GPIO48, Level::Low);
        let white = Output::new(peripherals.GPIO47, Level::Low);
        let mut bar = StatusBar::new(red, yellow, green, blue, white);
        bar.startup_animation(&delay);
        bar.set_state(BarState::Idle);
        bar
    };

    #[cfg(feature = "status-bar")]
    let mut bar_state = BarState::Idle;

    // === Addressable LEDs (RMT) ===
    #[cfg(feature = "addressable-leds")]
    let mut led_strip: LedStrip<'_, NUM_LEDS> = {
        let rmt = Rmt::new(peripherals.RMT, 80u32.MHz()).unwrap();
        let mut strip = LedStrip::new(rmt, peripherals.GPIO4);

        // Startup flash
        strip.fill(RGB8::new(64, 0, 0));
        strip.show();
        delay.delay_millis(200);
        strip.fill(RGB8::new(0, 64, 0));
        strip.show();
        delay.delay_millis(200);
        strip.fill(RGB8::new(0, 0, 64));
        strip.show();
        delay.delay_millis(200);
        strip.fill(RGB8::new(0, 0, 0));
        strip.show();

        strip
    };

    #[cfg(feature = "addressable-leds")]
    let mut led_color = RGB8::new(0, 0, 0);
    #[cfg(feature = "addressable-leds")]
    let mut last_led_color = led_color;
    #[cfg(feature = "addressable-leds")]
    let mut color_order = ColorOrder::Grb;
    #[cfg(feature = "addressable-leds")]
    let mut timing = LedTiming::Sk68xx;
    #[cfg(feature = "addressable-leds")]
    let mut force_update = false;

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
                            parse_command(
                                &cmd_buf,
                                &mut ui_state,
                                #[cfg(feature = "status-led")]
                                &mut status_pattern,
                                #[cfg(feature = "status-bar")]
                                &mut bar_state,
                                #[cfg(feature = "addressable-leds")]
                                &mut led_color,
                                #[cfg(feature = "addressable-leds")]
                                &mut color_order,
                                #[cfg(feature = "addressable-leds")]
                                &mut timing,
                                #[cfg(feature = "addressable-leds")]
                                &mut force_update,
                            );
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
        }

        // Cycle bottom bar page
        if frame_counter % PAGE_CYCLE_FRAMES == 0 && frame_counter > 0 {
            ui_state.next_page();
        }

        // Update status LED
        #[cfg(feature = "status-led")]
        {
            status_led.set_pattern(status_pattern);
            status_led.update();
        }

        // Update status bar
        #[cfg(feature = "status-bar")]
        {
            status_bar.set_state(bar_state);
            status_bar.update();
        }

        // Update addressable LEDs
        #[cfg(feature = "addressable-leds")]
        {
            if force_update
                || led_color.r != last_led_color.r
                || led_color.g != last_led_color.g
                || led_color.b != last_led_color.b
            {
                led_strip.set_timing(timing);
                led_strip.set_color_order(color_order);
                led_strip.fill(led_color);
                led_strip.show();
                last_led_color = led_color;
                force_update = false;
            }
        }

        // Render UI
        render(&mut display, &ui_state);
        let _ = display.flush();

        // Frame delay
        delay.delay_millis(FRAME_DELAY_MS);
        frame_counter = frame_counter.wrapping_add(1);
    }
}
