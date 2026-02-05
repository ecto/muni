//! TOML configuration parsing for per-board tool config.

use serde::Deserialize;

/// Top-level config from `/config/tool.toml`.
#[derive(Debug, Deserialize)]
pub struct ToolConfig {
    pub identity: IdentityConfig,
    pub network: NetworkConfig,
    #[serde(default)]
    pub lights: Option<LightsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct IdentityConfig {
    /// Tool role: "lights", "auger", "spreader", "sensor"
    pub tool_type: String,
    /// Unique board serial number
    pub serial: u32,
    /// Human-readable name
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    /// "dhcp" or "static"
    #[serde(default = "default_dhcp")]
    pub mode: String,
    /// Static IP (used if mode=static)
    pub static_ip: Option<String>,
    /// UDP broadcast port for discovery
    #[serde(default = "default_discovery_port")]
    pub discovery_port: u16,
    /// UDP unicast port for commands
    #[serde(default = "default_command_port")]
    pub command_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LightsConfig {
    /// GPIO pin for MOSFET gate
    pub headlight_pin: u8,
    /// Number of headlight channels
    #[serde(default = "default_one")]
    pub headlight_count: u8,
    /// GPIO pin for WS2812 data
    pub led_strip_pin: u8,
    /// Number of addressable LEDs
    #[serde(default = "default_led_count")]
    pub led_count: u16,
}

fn default_dhcp() -> String {
    "dhcp".to_string()
}
fn default_discovery_port() -> u16 {
    4861
}
fn default_command_port() -> u16 {
    4862
}
fn default_one() -> u8 {
    1
}
fn default_led_count() -> u16 {
    24
}

impl ToolConfig {
    /// Default config for headlights role (used until SPIFFS config loading is wired).
    pub fn default_lights() -> Self {
        Self {
            identity: IdentityConfig {
                tool_type: "lights".to_string(),
                serial: 0x0001,
                name: "Front Headlights".to_string(),
            },
            network: NetworkConfig {
                mode: "dhcp".to_string(),
                static_ip: None,
                discovery_port: 4861,
                command_port: 4862,
            },
            lights: Some(LightsConfig {
                headlight_pin: 4,
                headlight_count: 2,
                led_strip_pin: 5,
                led_count: 24,
            }),
        }
    }

    /// Tool type as protocol u8 value.
    pub fn tool_type_id(&self) -> u8 {
        match self.identity.tool_type.as_str() {
            "auger" | "snow_auger" => 1,
            "spreader" => 2,
            "mower" => 3,
            "plow" => 4,
            "blower" => 5,
            "lights" => 6,
            "sensor" => 7,
            _ => 0,
        }
    }
}
