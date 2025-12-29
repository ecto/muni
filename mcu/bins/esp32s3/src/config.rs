//! Device configuration for the attachment controller.
//!
//! Configuration defines what this device does:
//! - Device metadata (name, type)
//! - CAN ID assignment
//! - Pin mappings for inputs/outputs

/// Maximum number of GPIO pins that can be configured.
pub const MAX_PINS: usize = 8;

/// Device type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
#[repr(u8)]
pub enum DeviceType {
    /// Unknown/unconfigured device.
    Unknown = 0x00,
    /// LED strip controller (WS2812, etc.).
    LedController = 0x01,
    /// Brush/sweeper motor attachment.
    BrushAttachment = 0x02,
    /// Spreader attachment (salt, sand).
    SpreaderAttachment = 0x03,
    /// Plow attachment.
    PlowAttachment = 0x04,
    /// Generic sensor node.
    SensorNode = 0x10,
    /// Generic actuator node.
    ActuatorNode = 0x11,
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Pin function assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
#[repr(u8)]
pub enum PinFunction {
    /// Pin not used.
    Unused = 0x00,
    /// Digital input (active high).
    DigitalInputHigh = 0x01,
    /// Digital input (active low).
    DigitalInputLow = 0x02,
    /// Digital output (push-pull).
    DigitalOutput = 0x03,
    /// PWM output.
    PwmOutput = 0x04,
    /// Analog input (ADC).
    AnalogInput = 0x05,
    /// WS2812 LED data output.
    LedData = 0x10,
    /// Motor driver PWM.
    MotorPwm = 0x20,
    /// Motor driver direction.
    MotorDir = 0x21,
    /// Motor driver enable.
    MotorEnable = 0x22,
}

impl Default for PinFunction {
    fn default() -> Self {
        Self::Unused
    }
}

/// Configuration for a single GPIO pin.
#[derive(Debug, Clone, Copy, Default, defmt::Format)]
pub struct PinConfig {
    /// GPIO number (0-48 for ESP32-S3).
    pub gpio: u8,
    /// Pin function.
    pub function: PinFunction,
    /// Optional: channel/index for multi-channel functions.
    pub channel: u8,
}

/// CAN bus configuration.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct CanConfig {
    /// TWAI RX pin.
    pub rx: u8,
    /// TWAI TX pin.
    pub tx: u8,
    /// Base CAN ID for this device.
    pub base_id: u16,
    /// Baud rate in kbps (125, 250, 500, 1000).
    pub baud_kbps: u16,
}

impl Default for CanConfig {
    fn default() -> Self {
        Self {
            rx: 4,
            tx: 5,
            base_id: 0x0A00,
            baud_kbps: 500,
        }
    }
}

/// Complete device configuration.
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    /// Device type.
    pub device_type: DeviceType,
    /// Human-readable device name.
    pub name: &'static str,
    /// CAN bus configuration.
    pub can: CanConfig,
    /// Status LED pin (0 = not used).
    pub status_led: u8,
    /// Whether OLED is present.
    pub has_oled: bool,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_type: DeviceType::Unknown,
            name: "Unconfigured",
            can: CanConfig::default(),
            status_led: 35, // Heltec V3 onboard LED
            has_oled: true,
        }
    }
}

impl DeviceConfig {
    /// Create a configuration for the Heltec LoRa 32 V3 as an LED controller.
    pub const fn heltec_led_controller() -> Self {
        Self {
            device_type: DeviceType::LedController,
            name: "Rover LEDs",
            can: CanConfig {
                rx: 4,
                tx: 5,
                base_id: 0x0B00,
                baud_kbps: 500,
            },
            status_led: 35,
            has_oled: true,
        }
    }

    /// Create a configuration for a brush attachment.
    pub const fn heltec_brush_attachment() -> Self {
        Self {
            device_type: DeviceType::BrushAttachment,
            name: "Brush",
            can: CanConfig {
                rx: 4,
                tx: 5,
                base_id: 0x0A00,
                baud_kbps: 500,
            },
            status_led: 35,
            has_oled: true,
        }
    }

    /// Create an unconfigured device.
    pub const fn unconfigured() -> Self {
        Self {
            device_type: DeviceType::Unknown,
            name: "Unconfigured",
            can: CanConfig {
                rx: 4,
                tx: 5,
                base_id: 0x0A00,
                baud_kbps: 500,
            },
            status_led: 35,
            has_oled: true,
        }
    }
}

// Compile-time configuration selection.
#[cfg(feature = "config-led-controller")]
pub fn get_config() -> DeviceConfig {
    DeviceConfig::heltec_led_controller()
}

#[cfg(feature = "config-brush")]
pub fn get_config() -> DeviceConfig {
    DeviceConfig::heltec_brush_attachment()
}

#[cfg(not(any(feature = "config-led-controller", feature = "config-brush")))]
pub fn get_config() -> DeviceConfig {
    DeviceConfig::unconfigured()
}
