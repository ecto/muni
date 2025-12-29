//! OLED Display UI for Muni MCU
//!
//! Layout:
//! ┌────────────────────────────┐
//! │ ● MUNI         BRUSH CTRL  │  <- Status dot + identity
//! ├────────────────────────────┤
//! │                            │
//! │         I D L E            │  <- Large state
//! │                            │
//! ├────────────────────────────┤
//! │ ◄12 ►34        UP 00:12:34 │  <- Bottom bar (cycles)
//! └────────────────────────────┘

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::Text,
};
use heapless::String;

/// Display dimensions
pub const DISPLAY_WIDTH: i32 = 128;
pub const DISPLAY_HEIGHT: i32 = 64;

/// Layout constants
const HEADER_HEIGHT: i32 = 12;
const FOOTER_HEIGHT: i32 = 12;
const FOOTER_Y: i32 = DISPLAY_HEIGHT - FOOTER_HEIGHT;

/// Device operational state
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeviceState {
    Idle,
    Running,
    Error,
    Disconnected,
}

impl DeviceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceState::Idle => "IDLE",
            DeviceState::Running => "RUNNING",
            DeviceState::Error => "ERROR",
            DeviceState::Disconnected => "DISCONNECTED",
        }
    }
}

/// Bottom bar page types
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BottomPage {
    Activity,  // RX/TX counts + uptime
    CanHealth, // CAN bus status
    DeviceInfo, // ID + version
}

impl BottomPage {
    pub fn next(self) -> Self {
        match self {
            BottomPage::Activity => BottomPage::CanHealth,
            BottomPage::CanHealth => BottomPage::DeviceInfo,
            BottomPage::DeviceInfo => BottomPage::Activity,
        }
    }
}

/// UI state tracking
pub struct UiState {
    /// Device name (e.g., "BRUSH CTRL")
    pub device_name: &'static str,
    /// Current operational state
    pub state: DeviceState,
    /// CAN messages received
    pub rx_count: u32,
    /// CAN messages transmitted
    pub tx_count: u32,
    /// CAN errors
    pub can_errors: u32,
    /// CAN bus OK
    pub can_bus_ok: bool,
    /// CAN bitrate in kbps
    pub can_bitrate_k: u16,
    /// Uptime in seconds
    pub uptime_secs: u32,
    /// Device CAN ID
    pub device_id: u16,
    /// Firmware version string
    pub version: &'static str,
    /// Current bottom bar page
    pub bottom_page: BottomPage,
    /// Animation frame counter (for pulsing dot)
    pub frame: u8,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            device_name: "MUNI MCU",
            state: DeviceState::Idle,
            rx_count: 0,
            tx_count: 0,
            can_errors: 0,
            can_bus_ok: true,
            can_bitrate_k: 500,
            uptime_secs: 0,
            device_id: 0x0A00,
            version: "v0.1",
            bottom_page: BottomPage::Activity,
            frame: 0,
        }
    }
}

impl UiState {
    /// Advance to next animation frame (call at ~30fps or so)
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Advance to next bottom page (instant)
    pub fn next_page(&mut self) {
        self.bottom_page = self.bottom_page.next();
    }

    /// Format uptime as HH:MM:SS
    fn format_uptime(&self) -> String<12> {
        let hours = self.uptime_secs / 3600;
        let mins = (self.uptime_secs % 3600) / 60;
        let secs = self.uptime_secs % 60;

        let mut s: String<12> = String::new();
        if hours > 0 {
            let _ = write!(s, "{:02}:{:02}:{:02}", hours, mins, secs);
        } else {
            let _ = write!(s, "{:02}:{:02}", mins, secs);
        }
        s
    }
}

/// Render the UI to a display
pub fn render<D>(display: &mut D, state: &UiState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    // Clear display
    let _ = display.clear(BinaryColor::Off);

    // Draw header
    draw_header(display, state);

    // Draw separator line under header
    let _ = Line::new(
        Point::new(0, HEADER_HEIGHT),
        Point::new(DISPLAY_WIDTH - 1, HEADER_HEIGHT),
    )
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(display);

    // Draw main state in center
    draw_state(display, state);

    // Draw separator line above footer
    let _ = Line::new(
        Point::new(0, FOOTER_Y),
        Point::new(DISPLAY_WIDTH - 1, FOOTER_Y),
    )
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(display);

    // Draw footer (bottom bar)
    draw_footer(display, state);
}

fn draw_header<D>(display: &mut D, state: &UiState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Status dot (pulses based on frame)
    let dot_on = match state.state {
        DeviceState::Error => (state.frame / 8) % 2 == 0, // Fast blink for error
        DeviceState::Disconnected => (state.frame / 16) % 2 == 0, // Slow blink
        _ => {
            // Pulse effect: on most of the time, brief off
            let phase = state.frame % 32;
            phase < 28
        }
    };

    if dot_on {
        let _ = Circle::new(Point::new(2, 2), 8)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(display);
    } else {
        // Draw outline only when "off"
        let _ = Circle::new(Point::new(2, 2), 8)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display);
    }

    // "MUNI" text
    let _ = Text::new("MUNI", Point::new(14, 9), text_style).draw(display);

    // Device name (right-aligned)
    let name_width = state.device_name.len() as i32 * 6;
    let name_x = DISPLAY_WIDTH - name_width - 2;
    let _ = Text::new(state.device_name, Point::new(name_x, 9), text_style).draw(display);
}

fn draw_state<D>(display: &mut D, state: &UiState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let bold_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    // Center the state text
    let state_str = state.state.as_str();
    let text_width = state_str.len() as i32 * 9;
    let x = (DISPLAY_WIDTH - text_width) / 2;
    let y = HEADER_HEIGHT + (FOOTER_Y - HEADER_HEIGHT) / 2 + 6; // Center vertically

    let _ = Text::new(state_str, Point::new(x, y), bold_style).draw(display);
}

fn draw_footer<D>(display: &mut D, state: &UiState)
where
    D: DrawTarget<Color = BinaryColor>,
{
    let y = FOOTER_Y + 10;
    draw_footer_page(display, state, state.bottom_page, 2, y);
}

fn draw_footer_page<D>(
    display: &mut D,
    state: &UiState,
    page: BottomPage,
    x: i32,
    y: i32,
) where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let mut buf: String<32> = String::new();

    match page {
        BottomPage::Activity => {
            // Format: "◄123 ►456   UP 00:12:34"
            // Using ASCII arrows since we might not have Unicode
            let uptime = state.format_uptime();
            let _ = write!(
                buf,
                "<{:03} >{:03}   UP {}",
                state.rx_count % 1000,
                state.tx_count % 1000,
                uptime
            );
        }
        BottomPage::CanHealth => {
            // Format: "CAN 500k  ERR:0  OK"
            let status = if state.can_bus_ok { "OK" } else { "ERR" };
            let _ = write!(
                buf,
                "CAN {}k  ERR:{}  {}",
                state.can_bitrate_k, state.can_errors, status
            );
        }
        BottomPage::DeviceInfo => {
            // Format: "ID:0x0A00  ESP32 v0.1"
            let _ = write!(buf, "ID:{:#04X}  ESP32 {}", state.device_id, state.version);
        }
    }

    let _ = Text::new(&buf, Point::new(x, y), style).draw(display);
}
