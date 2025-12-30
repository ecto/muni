//! Simple GPIO status LED driver
//!
//! Provides visual feedback using basic LEDs connected to GPIO pins.
//! Supports single-color onboard LED and multi-LED status bar.
//!
//! ## Heltec V3 Pinout
//! - GPIO35: Onboard LED (active high)
//!
//! ## External Status LEDs (breadboard)
//! Connect LEDs with current-limiting resistors (~330Ω):
//! - GPIO19: Red (error/fault)
//! - GPIO20: Yellow (warning/attention)
//! - GPIO26: Green (idle/ok)
//! - GPIO48: Blue (running/active)
//! - GPIO47: White (heartbeat)

use esp_hal::gpio::Output;

/// Status LED state patterns
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusPattern {
    /// LED off
    Off,
    /// Solid on
    Solid,
    /// Slow pulse (idle, ~1Hz)
    SlowPulse,
    /// Fast pulse (active, ~2Hz)
    FastPulse,
    /// Quick double-blink (attention)
    DoubleBlink,
    /// Rapid flash (error/fault)
    RapidFlash,
}

impl Default for StatusPattern {
    fn default() -> Self {
        Self::SlowPulse
    }
}

/// Simple single-LED status indicator
pub struct StatusLed<'d> {
    pin: Output<'d>,
    pattern: StatusPattern,
    frame: u32,
}

impl<'d> StatusLed<'d> {
    /// Create a new status LED
    pub fn new(pin: Output<'d>) -> Self {
        Self {
            pin,
            pattern: StatusPattern::default(),
            frame: 0,
        }
    }

    /// Set the blink pattern
    pub fn set_pattern(&mut self, pattern: StatusPattern) {
        if self.pattern != pattern {
            self.pattern = pattern;
            self.frame = 0;
        }
    }

    /// Update the LED state (call at ~30fps)
    pub fn update(&mut self) {
        let on = match self.pattern {
            StatusPattern::Off => false,
            StatusPattern::Solid => true,
            StatusPattern::SlowPulse => {
                // ~1Hz: on for 15 frames, off for 15 frames
                (self.frame % 30) < 15
            }
            StatusPattern::FastPulse => {
                // ~2Hz: on for 7 frames, off for 7 frames
                (self.frame % 14) < 7
            }
            StatusPattern::DoubleBlink => {
                // Double blink every ~2 seconds
                let phase = self.frame % 60;
                // Blink at frame 0-3 and 6-9
                phase < 3 || (6..9).contains(&phase)
            }
            StatusPattern::RapidFlash => {
                // ~5Hz flash
                (self.frame % 6) < 3
            }
        };

        if on {
            self.pin.set_high();
        } else {
            self.pin.set_low();
        }

        self.frame = self.frame.wrapping_add(1);
    }

    /// Force LED on
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// Force LED off
    pub fn off(&mut self) {
        self.pin.set_low();
    }
}

/// Multi-LED status bar for visual state indication
///
/// Uses 5 LEDs to show different states:
/// - Red: Error/fault
/// - Yellow: Warning/attention
/// - Green: Idle/OK
/// - Blue: Running/active
/// - White: Heartbeat (always pulses to show system is alive)
pub struct StatusBar<'d> {
    red: Output<'d>,
    yellow: Output<'d>,
    green: Output<'d>,
    blue: Output<'d>,
    white: Output<'d>,
    state: BarState,
    frame: u32,
}

/// Status bar display states
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarState {
    /// All off except heartbeat
    Off,
    /// Idle: green solid, white heartbeat
    Idle,
    /// Running: blue solid, white heartbeat
    Running,
    /// Error: red flashing, white heartbeat
    Error,
    /// Warning: yellow flashing
    Warning,
    /// Startup: chase animation
    Startup,
    /// All on (test mode)
    AllOn,
}

impl Default for BarState {
    fn default() -> Self {
        Self::Idle
    }
}

impl<'d> StatusBar<'d> {
    /// Create a new status bar with 5 LEDs
    pub fn new(
        red: Output<'d>,
        yellow: Output<'d>,
        green: Output<'d>,
        blue: Output<'d>,
        white: Output<'d>,
    ) -> Self {
        Self {
            red,
            yellow,
            green,
            blue,
            white,
            state: BarState::default(),
            frame: 0,
        }
    }

    /// Set the display state
    pub fn set_state(&mut self, state: BarState) {
        if self.state != state {
            self.state = state;
            self.frame = 0;
        }
    }

    /// Update LED states (call at ~30fps)
    pub fn update(&mut self) {
        // Heartbeat on white LED: minimal 1-frame flash every 2 seconds (battery saving)
        let heartbeat = match self.state {
            BarState::Off => false,
            BarState::AllOn => true,
            _ => self.frame.is_multiple_of(60), // Single frame flash every 2 sec (~33ms on)
        };

        match self.state {
            BarState::Off => {
                self.set_leds(false, false, false, false, false);
            }
            BarState::Idle => {
                // Green solid, white heartbeat
                self.set_leds(false, false, true, false, heartbeat);
            }
            BarState::Running => {
                // Blue solid, white heartbeat
                self.set_leds(false, false, false, true, heartbeat);
            }
            BarState::Error => {
                // Red flashing fast
                let red_on = (self.frame % 10) < 5;
                self.set_leds(red_on, false, false, false, heartbeat);
            }
            BarState::Warning => {
                // Yellow flashing slow
                let yellow_on = (self.frame % 20) < 10;
                self.set_leds(false, yellow_on, false, false, heartbeat);
            }
            BarState::Startup => {
                // Chase animation: light each LED in sequence
                let phase = (self.frame / 6) % 5;
                self.set_leds(
                    phase == 0,
                    phase == 1,
                    phase == 2,
                    phase == 3,
                    phase == 4,
                );
            }
            BarState::AllOn => {
                self.set_leds(true, true, true, true, true);
            }
        }

        self.frame = self.frame.wrapping_add(1);
    }

    /// Set individual LED states
    fn set_leds(&mut self, r: bool, y: bool, g: bool, b: bool, w: bool) {
        if r { self.red.set_high(); } else { self.red.set_low(); }
        if y { self.yellow.set_high(); } else { self.yellow.set_low(); }
        if g { self.green.set_high(); } else { self.green.set_low(); }
        if b { self.blue.set_high(); } else { self.blue.set_low(); }
        if w { self.white.set_high(); } else { self.white.set_low(); }
    }

    /// Run startup animation (blocking)
    pub fn startup_animation(&mut self, delay: &esp_hal::delay::Delay) {
        // Quick chase through all LEDs
        for led in 0..5 {
            self.set_leds(led == 0, led == 1, led == 2, led == 3, led == 4);
            delay.delay_millis(60);
        }
        // Brief all-on flash
        self.set_leds(true, true, true, true, true);
        delay.delay_millis(100);
        self.set_leds(false, false, false, false, false);
    }

    /// All LEDs off
    pub fn off(&mut self) {
        self.set_leds(false, false, false, false, false);
    }

    /// All LEDs on
    pub fn all_on(&mut self) {
        self.set_leds(true, true, true, true, true);
    }
}
