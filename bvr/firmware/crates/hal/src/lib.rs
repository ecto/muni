//! Hardware abstraction layer for bvr.
//!
//! Provides interfaces to GPIO, ADC, and power monitoring on the Jetson Orin NX.

use thiserror::Error;
use tracing::warn;
use types::PowerStatus;

#[cfg(feature = "gpio")]
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum HalError {
    #[error("GPIO error: {0}")]
    Gpio(String),
    #[error("ADC error: {0}")]
    Adc(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GPIO not supported on this platform")]
    GpioNotSupported,
}

/// Power monitoring interface.
///
/// Primary voltage reading comes from VESCs over CAN.
/// This provides supplementary monitoring via ADC for 12V rail, etc.
pub struct PowerMonitor {
    low_voltage_threshold: f64,
    critical_voltage_threshold: f64,
}

impl PowerMonitor {
    pub fn new(low_threshold: f64, critical_threshold: f64) -> Self {
        Self {
            low_voltage_threshold: low_threshold,
            critical_voltage_threshold: critical_threshold,
        }
    }

    /// Check if battery is below low threshold.
    pub fn is_low(&self, voltage: f64) -> bool {
        voltage < self.low_voltage_threshold
    }

    /// Check if battery is below critical threshold.
    pub fn is_critical(&self, voltage: f64) -> bool {
        voltage < self.critical_voltage_threshold
    }

    /// Read 12V rail voltage (via ADC).
    ///
    /// TODO: Implement actual sysfs IIO read.
    pub fn read_12v_rail(&self) -> Result<f64, HalError> {
        // Placeholder - would read from /sys/bus/iio/devices/...
        Ok(12.1)
    }

    /// Build a PowerStatus from VESC voltage + local readings.
    pub fn build_status(&self, vesc_voltage: f64, system_current: f64) -> PowerStatus {
        PowerStatus {
            battery_voltage: vesc_voltage,
            system_current,
        }
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        // 48V nominal, 13S LiPo
        Self::new(42.0, 39.0)
    }
}

/// E-Stop events from hardware GPIO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EStopEvent {
    /// E-stop button pressed (trigger safety stop)
    Pressed,
    /// E-stop button released (ready for operator confirmation)
    Released,
}

/// Configuration for hardware e-stop GPIO.
#[derive(Debug, Clone)]
pub struct EStopConfig {
    /// GPIO chip device name (e.g., "gpiochip0")
    pub gpio_chip: String,
    /// GPIO line number (e.g., 16 for Pin 36 on Jetson 40-pin header)
    pub gpio_line: u32,
    /// Active-low logic (true = button connects GPIO to GND when pressed)
    pub active_low: bool,
    /// Debounce time in milliseconds
    pub debounce_ms: u64,
}

impl Default for EStopConfig {
    fn default() -> Self {
        Self {
            gpio_chip: "gpiochip0".to_string(),
            gpio_line: 16, // Pin 36 on Jetson 40-pin header
            active_low: true,
            debounce_ms: 50,
        }
    }
}

// ============================================================================
// GPIO Implementation (when feature enabled)
// ============================================================================

#[cfg(feature = "gpio")]
mod gpio_impl {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::sync::mpsc;
    use tokio_gpiod::{Chip, EdgeDetect, Lines, Options};

    /// Hardware e-stop input using GPIO with edge detection.
    ///
    /// Spawns an async task that monitors the GPIO line and sends events
    /// through a channel when the button is pressed or released.
    pub struct EStopInput {
        config: EStopConfig,
    }

    impl EStopInput {
        /// Create a new e-stop input with the given configuration.
        pub fn new(config: EStopConfig) -> Self {
            Self { config }
        }

        /// Spawn the GPIO monitoring task.
        ///
        /// Returns a receiver for e-stop events. The task will run until
        /// the sender is dropped (e.g., when the receiver is dropped).
        pub async fn spawn(
            self,
            tx: mpsc::Sender<EStopEvent>,
        ) -> Result<tokio::task::JoinHandle<()>, HalError> {
            let config = self.config.clone();

            // Open GPIO chip
            let chip = Chip::new(&config.gpio_chip)
                .await
                .map_err(|e| HalError::Gpio(format!("Failed to open GPIO chip: {}", e)))?;

            // Configure line as input with edge detection
            let opts = Options::input([config.gpio_line])
                .edge(EdgeDetect::Both)
                .consumer("bvr-estop");

            let lines = chip
                .request_lines(opts)
                .await
                .map_err(|e| HalError::Gpio(format!("Failed to configure GPIO line: {}", e)))?;

            info!(
                chip = %config.gpio_chip,
                line = config.gpio_line,
                active_low = config.active_low,
                debounce_ms = config.debounce_ms,
                "Hardware e-stop GPIO initialized"
            );

            // Spawn monitoring task
            let handle = tokio::spawn(async move {
                Self::monitor_loop(lines, config, tx).await;
            });

            Ok(handle)
        }

        async fn monitor_loop(
            mut lines: Lines<tokio_gpiod::Input>,
            config: EStopConfig,
            tx: mpsc::Sender<EStopEvent>,
        ) {
            let debounce_duration = Duration::from_millis(config.debounce_ms);
            let mut last_event_time = Instant::now();
            let mut last_state: Option<bool> = None;

            loop {
                match lines.read_event().await {
                    Ok(event) => {
                        let now = Instant::now();

                        // Software debouncing: ignore events within debounce window
                        if now.duration_since(last_event_time) < debounce_duration {
                            debug!("E-stop GPIO event ignored (debounce)");
                            continue;
                        }
                        last_event_time = now;

                        // Determine logical state (account for active-low)
                        let gpio_high = event.edge == tokio_gpiod::Edge::Rising;
                        let is_pressed = if config.active_low {
                            !gpio_high // Active-low: pressed when GPIO is LOW
                        } else {
                            gpio_high // Active-high: pressed when GPIO is HIGH
                        };

                        // Only send events on actual state changes
                        if last_state != Some(is_pressed) {
                            last_state = Some(is_pressed);

                            let estop_event = if is_pressed {
                                EStopEvent::Pressed
                            } else {
                                EStopEvent::Released
                            };

                            if is_pressed {
                                warn!("Hardware e-stop PRESSED");
                            } else {
                                info!("Hardware e-stop released");
                            }

                            if tx.send(estop_event).await.is_err() {
                                // Channel closed, exit the loop
                                info!("E-stop event channel closed, stopping GPIO monitor");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!(?e, "GPIO read error, retrying");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "gpio")]
pub use gpio_impl::EStopInput;

// ============================================================================
// Stub Implementation (when GPIO feature not enabled)
// ============================================================================

#[cfg(not(feature = "gpio"))]
mod stub_impl {
    use super::*;

    /// Stub e-stop input for platforms without GPIO support.
    pub struct EStopInput {
        gpio_pin: u32,
        active_low: bool,
    }

    impl EStopInput {
        pub fn new(gpio_pin: u32, active_low: bool) -> Result<Self, HalError> {
            warn!(
                gpio_pin,
                "Hardware e-stop not available (compiled without 'gpio' feature)"
            );
            Ok(Self {
                gpio_pin,
                active_low,
            })
        }

        /// Check if e-stop is triggered (always returns false in stub).
        pub fn is_triggered(&self) -> bool {
            let _ = (self.gpio_pin, self.active_low);
            false
        }
    }
}

#[cfg(not(feature = "gpio"))]
pub use stub_impl::EStopInput;

/// Status LED control.
pub struct StatusLed {
    gpio_pin: u32,
}

impl StatusLed {
    pub fn new(gpio_pin: u32) -> Result<Self, HalError> {
        // TODO: Configure GPIO as output
        Ok(Self { gpio_pin })
    }

    pub fn set(&self, on: bool) -> Result<(), HalError> {
        // TODO: Write GPIO state
        let _ = (self.gpio_pin, on);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_monitor_thresholds() {
        let monitor = PowerMonitor::new(42.0, 39.0);

        assert!(!monitor.is_low(48.0));
        assert!(!monitor.is_critical(48.0));

        assert!(monitor.is_low(41.0));
        assert!(!monitor.is_critical(41.0));

        assert!(monitor.is_low(38.0));
        assert!(monitor.is_critical(38.0));
    }

    #[test]
    fn test_estop_config_default() {
        let config = EStopConfig::default();
        assert_eq!(config.gpio_chip, "gpiochip0");
        assert_eq!(config.gpio_line, 16);
        assert!(config.active_low);
        assert_eq!(config.debounce_ms, 50);
    }

    #[test]
    fn test_estop_config_custom() {
        let config = EStopConfig {
            gpio_chip: "gpiochip1".to_string(),
            gpio_line: 24,
            active_low: false,
            debounce_ms: 100,
        };
        assert_eq!(config.gpio_chip, "gpiochip1");
        assert_eq!(config.gpio_line, 24);
        assert!(!config.active_low);
        assert_eq!(config.debounce_ms, 100);
    }

    #[test]
    fn test_estop_event_equality() {
        assert_eq!(EStopEvent::Pressed, EStopEvent::Pressed);
        assert_eq!(EStopEvent::Released, EStopEvent::Released);
        assert_ne!(EStopEvent::Pressed, EStopEvent::Released);
    }

    #[test]
    fn test_estop_event_clone() {
        let event = EStopEvent::Pressed;
        let cloned = event;
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_estop_event_debug() {
        // Verify Debug impl works (used for logging)
        let pressed = format!("{:?}", EStopEvent::Pressed);
        let released = format!("{:?}", EStopEvent::Released);
        assert!(pressed.contains("Pressed"));
        assert!(released.contains("Released"));
    }

    #[test]
    fn test_active_low_logic() {
        // Active-low: GPIO LOW = pressed, GPIO HIGH = released
        // This mirrors the logic in gpio_impl::monitor_loop
        let active_low = true;

        // Simulate rising edge (GPIO goes HIGH)
        let gpio_high = true;
        let is_pressed_on_rising = if active_low { !gpio_high } else { gpio_high };
        assert!(!is_pressed_on_rising, "Rising edge with active-low should be released");

        // Simulate falling edge (GPIO goes LOW)
        let gpio_high = false;
        let is_pressed_on_falling = if active_low { !gpio_high } else { gpio_high };
        assert!(is_pressed_on_falling, "Falling edge with active-low should be pressed");
    }

    #[test]
    fn test_active_high_logic() {
        // Active-high: GPIO HIGH = pressed, GPIO LOW = released
        let active_low = false;

        // Simulate rising edge (GPIO goes HIGH)
        let gpio_high = true;
        let is_pressed_on_rising = if active_low { !gpio_high } else { gpio_high };
        assert!(is_pressed_on_rising, "Rising edge with active-high should be pressed");

        // Simulate falling edge (GPIO goes LOW)
        let gpio_high = false;
        let is_pressed_on_falling = if active_low { !gpio_high } else { gpio_high };
        assert!(!is_pressed_on_falling, "Falling edge with active-high should be released");
    }

    #[test]
    fn test_power_monitor_default() {
        let monitor = PowerMonitor::default();
        // Default is 48V nominal (42V low, 39V critical)
        assert!(monitor.is_low(40.0));
        assert!(!monitor.is_low(43.0));
        assert!(monitor.is_critical(38.0));
        assert!(!monitor.is_critical(40.0));
    }

    #[test]
    fn test_power_status_build() {
        let monitor = PowerMonitor::default();
        let status = monitor.build_status(48.5, 2.3);
        assert!((status.battery_voltage - 48.5).abs() < f64::EPSILON);
        assert!((status.system_current - 2.3).abs() < f64::EPSILON);
    }

    #[cfg(not(feature = "gpio"))]
    #[test]
    fn test_stub_estop_never_triggered() {
        // Stub always returns false for is_triggered
        let estop = EStopInput::new(16, true).unwrap();
        assert!(!estop.is_triggered());

        let estop = EStopInput::new(24, false).unwrap();
        assert!(!estop.is_triggered());
    }
}
