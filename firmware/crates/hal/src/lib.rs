//! Hardware abstraction layer for bvr.
//!
//! Provides interfaces to GPIO, ADC, and power monitoring on the Jetson Orin NX.

use thiserror::Error;
use types::PowerStatus;

#[derive(Error, Debug)]
pub enum HalError {
    #[error("GPIO error: {0}")]
    Gpio(String),
    #[error("ADC error: {0}")]
    Adc(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
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

/// E-Stop input handling.
pub struct EStopInput {
    gpio_pin: u32,
    active_low: bool,
}

impl EStopInput {
    pub fn new(gpio_pin: u32, active_low: bool) -> Result<Self, HalError> {
        // TODO: Configure GPIO as input via sysfs or gpiod
        Ok(Self {
            gpio_pin,
            active_low,
        })
    }

    /// Check if e-stop is triggered.
    ///
    /// TODO: Implement actual GPIO read.
    pub fn is_triggered(&self) -> bool {
        // Placeholder - would read from /sys/class/gpio/...
        let _ = (self.gpio_pin, self.active_low);
        false
    }
}

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
