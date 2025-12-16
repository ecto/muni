//! Hardware abstraction layer for bvr.
//!
//! Provides interfaces to GPIO, ADC, and power monitoring on the Jetson Orin NX.

use bvr_types::PowerStatus;
use thiserror::Error;

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
/// Reads battery voltage, rail voltages, and current via ADC.
pub struct PowerMonitor {
    // ADC channel mappings (placeholder — real impl would use sysfs or IIO)
    #[allow(dead_code)]
    battery_channel: u8,
    #[allow(dead_code)]
    rail_12v_channel: u8,
    #[allow(dead_code)]
    current_channel: u8,
    // Calibration values
    #[allow(dead_code)]
    battery_scale: f64,
    #[allow(dead_code)]
    rail_12v_scale: f64,
    #[allow(dead_code)]
    current_scale: f64,
    #[allow(dead_code)]
    current_offset: f64,
}

impl PowerMonitor {
    pub fn new() -> Result<Self, HalError> {
        // TODO: Initialize IIO or sysfs ADC interface
        Ok(Self {
            battery_channel: 0,
            rail_12v_channel: 1,
            current_channel: 2,
            battery_scale: 0.02, // V per ADC count (with voltage divider)
            rail_12v_scale: 0.005,
            current_scale: 0.01, // A per ADC count
            current_offset: 0.0,
        })
    }

    /// Read current power status.
    pub fn read(&self) -> Result<PowerStatus, HalError> {
        // TODO: Actual ADC reads
        // For now, return placeholder values
        Ok(PowerStatus {
            battery_voltage: 48.0,
            rail_12v: 12.1,
            system_current: 5.0,
        })
    }

    /// Check if battery is below critical threshold.
    pub fn is_critical(&self, status: &PowerStatus, threshold: f64) -> bool {
        status.battery_voltage < threshold
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize power monitor")
    }
}

/// E-Stop input handling.
pub struct EStopInput {
    // GPIO pin for e-stop (active low typically)
    #[allow(dead_code)]
    gpio_pin: u32,
}

impl EStopInput {
    pub fn new(gpio_pin: u32) -> Result<Self, HalError> {
        // TODO: Configure GPIO as input with pull-up
        Ok(Self { gpio_pin })
    }

    /// Check if e-stop is triggered.
    pub fn is_triggered(&self) -> bool {
        // TODO: Read GPIO state
        // For now, always return false (not triggered)
        false
    }
}

/// Status LED control.
pub struct StatusLed {
    #[allow(dead_code)]
    gpio_pin: u32,
}

impl StatusLed {
    pub fn new(gpio_pin: u32) -> Result<Self, HalError> {
        // TODO: Configure GPIO as output
        Ok(Self { gpio_pin })
    }

    pub fn set(&self, on: bool) -> Result<(), HalError> {
        // TODO: Write GPIO state
        let _ = on;
        Ok(())
    }
}

