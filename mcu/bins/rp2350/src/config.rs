//! Device configuration for RP2350-based controllers.

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
    /// Generic sensor node.
    SensorNode = 0x10,
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// CAN bus configuration.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct CanConfig {
    /// Base CAN ID for this device.
    pub base_id: u16,
    /// Baud rate in kbps.
    pub baud_kbps: u16,
}

impl Default for CanConfig {
    fn default() -> Self {
        Self {
            base_id: 0x0B00, // Base rover peripheral range
            baud_kbps: 500,
        }
    }
}

/// LED strip configuration.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct LedConfig {
    /// Number of LEDs in the strip.
    pub num_leds: usize,
    /// Data pin GPIO number.
    pub data_pin: u8,
}

impl Default for LedConfig {
    fn default() -> Self {
        Self {
            num_leds: 24,
            data_pin: 0, // GP0
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
    /// LED configuration (if LED controller).
    pub led: Option<LedConfig>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_type: DeviceType::Unknown,
            name: "Unconfigured",
            can: CanConfig::default(),
            led: None,
        }
    }
}

impl DeviceConfig {
    /// Configuration for the base rover LED controller.
    pub const fn rover_leds() -> Self {
        Self {
            device_type: DeviceType::LedController,
            name: "Rover LEDs",
            can: CanConfig {
                base_id: 0x0B00,
                baud_kbps: 500,
            },
            led: Some(LedConfig {
                num_leds: 24,
                data_pin: 0,
            }),
        }
    }

    /// Configuration for a brush attachment.
    pub const fn brush_attachment() -> Self {
        Self {
            device_type: DeviceType::BrushAttachment,
            name: "Brush",
            can: CanConfig {
                base_id: 0x0A00,
                baud_kbps: 500,
            },
            led: None,
        }
    }

    /// Unconfigured device.
    pub const fn unconfigured() -> Self {
        Self {
            device_type: DeviceType::Unknown,
            name: "Unconfigured",
            can: CanConfig {
                base_id: 0x0A00,
                baud_kbps: 500,
            },
            led: None,
        }
    }
}

// Compile-time configuration selection.
#[cfg(feature = "config-led-controller")]
pub fn get_config() -> DeviceConfig {
    DeviceConfig::rover_leds()
}

#[cfg(feature = "config-brush")]
pub fn get_config() -> DeviceConfig {
    DeviceConfig::brush_attachment()
}

#[cfg(not(any(feature = "config-led-controller", feature = "config-brush")))]
pub fn get_config() -> DeviceConfig {
    // Default to LED controller for backwards compatibility
    DeviceConfig::rover_leds()
}
