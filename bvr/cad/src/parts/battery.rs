//! Battery reference geometry
//!
//! Simplified models for visualization and assembly.
//! Not for manufacturing - these are reference parts.

use crate::{centered_cube, Part};

// =============================================================================
// E-Bike Downtube Battery (BVR0)
// =============================================================================

/// E-bike downtube battery configuration
#[derive(Debug, Clone)]
pub struct DowntubeBatteryConfig {
    /// Battery length (mm)
    pub length: f64,
    /// Battery width (mm)
    pub width: f64,
    /// Battery height (mm)
    pub height: f64,
}

impl DowntubeBatteryConfig {
    /// 48V 20Ah typical downtube battery
    pub fn battery_48v_20ah() -> Self {
        Self {
            length: 420.0,
            width: 85.0,
            height: 55.0,
        }
    }

    /// 52V 20Ah larger downtube battery
    pub fn battery_52v_20ah() -> Self {
        Self {
            length: 450.0,
            width: 90.0,
            height: 60.0,
        }
    }
}

/// E-bike downtube battery reference model
pub struct DowntubeBattery {
    config: DowntubeBatteryConfig,
}

impl DowntubeBattery {
    pub fn new(config: DowntubeBatteryConfig) -> Self {
        Self { config }
    }

    pub fn standard_48v() -> Self {
        Self::new(DowntubeBatteryConfig::battery_48v_20ah())
    }

    /// Generate battery geometry
    ///
    /// Orientation: Long axis along Y, flat side down
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Main body
        let body = centered_cube("battery_body", cfg.width, cfg.length, cfg.height);

        // Mounting rail (on bottom)
        let rail_width = 10.0;
        let rail_height = 8.0;
        let rail = centered_cube("rail", rail_width, cfg.length - 40.0, rail_height)
            .translate(0.0, 0.0, -cfg.height / 2.0 - rail_height / 2.0);

        // Key/lock housing (on side)
        let key_housing = centered_cube("key", 20.0, 30.0, 15.0)
            .translate(cfg.width / 2.0 + 5.0, -cfg.length / 2.0 + 50.0, 0.0);

        body.union(&rail).union(&key_housing)
    }

    /// Generate simplified geometry
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("battery", cfg.width, cfg.length, cfg.height)
    }
}

// =============================================================================
// Custom Battery Pack (BVR1)
// =============================================================================

/// Custom battery pack configuration (13S4P 21700)
#[derive(Debug, Clone)]
pub struct CustomBatteryConfig {
    /// Pack length (mm)
    pub length: f64,
    /// Pack width (mm)
    pub width: f64,
    /// Pack height (mm)
    pub height: f64,
    /// Cell configuration (series, parallel)
    pub series: u32,
    pub parallel: u32,
}

impl Default for CustomBatteryConfig {
    fn default() -> Self {
        // 13S4P 21700 pack
        // 21700 cell: 21mm diameter, 70mm length
        // 4 cells wide × ~21mm = 84mm width
        // 13 cells long × ~21mm = 273mm length (staggered packing)
        // 70mm height
        Self {
            length: 280.0,
            width: 90.0,
            height: 75.0,
            series: 13,
            parallel: 4,
        }
    }
}

/// Custom battery pack reference model
pub struct CustomBattery {
    config: CustomBatteryConfig,
}

impl CustomBattery {
    pub fn new(config: CustomBatteryConfig) -> Self {
        Self { config }
    }

    pub fn bvr1_pack() -> Self {
        Self::new(CustomBatteryConfig::default())
    }

    /// Generate battery pack geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Main enclosure
        let enclosure = centered_cube("enclosure", cfg.width, cfg.length, cfg.height);

        // BMS housing (on one end)
        let bms = centered_cube("bms", cfg.width - 10.0, 40.0, cfg.height - 10.0)
            .translate(0.0, cfg.length / 2.0 + 15.0, 0.0);

        // Mounting tabs (on sides)
        let tab_width = 15.0;
        let tab_height = 10.0;
        let tab_length = cfg.length * 0.6;

        let left_tab = centered_cube("left_tab", tab_width, tab_length, tab_height)
            .translate(-cfg.width / 2.0 - tab_width / 2.0, 0.0, -cfg.height / 2.0 + tab_height / 2.0);

        let right_tab = centered_cube("right_tab", tab_width, tab_length, tab_height)
            .translate(cfg.width / 2.0 + tab_width / 2.0, 0.0, -cfg.height / 2.0 + tab_height / 2.0);

        enclosure.union(&bms).union(&left_tab).union(&right_tab)
    }

    /// Generate simplified geometry
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("battery", cfg.width, cfg.length, cfg.height)
    }

    /// Get total energy (Wh)
    pub fn energy_wh(&self) -> f64 {
        // Assuming 21700 cells at 3.6V nominal, 5000mAh
        let voltage = 3.6 * self.config.series as f64;
        let capacity_ah = 5.0 * self.config.parallel as f64;
        voltage * capacity_ah
    }
}

// =============================================================================
// Battery Tray (BVR1)
// =============================================================================

/// Battery tray configuration
#[derive(Debug, Clone)]
pub struct BatteryTrayConfig {
    /// Tray length (mm)
    pub length: f64,
    /// Tray width (mm)
    pub width: f64,
    /// Tray depth (mm)
    pub depth: f64,
    /// Wall thickness (mm)
    pub wall_thickness: f64,
}

impl Default for BatteryTrayConfig {
    fn default() -> Self {
        Self {
            length: 300.0,
            width: 110.0,
            depth: 40.0,
            wall_thickness: 3.0,
        }
    }
}

/// Battery tray (custom fabricated for BVR1)
pub struct BatteryTray {
    config: BatteryTrayConfig,
}

impl BatteryTray {
    pub fn new(config: BatteryTrayConfig) -> Self {
        Self { config }
    }

    pub fn bvr1_tray() -> Self {
        Self::new(BatteryTrayConfig::default())
    }

    /// Generate battery tray geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Outer shell
        let outer = centered_cube("outer", cfg.width, cfg.length, cfg.depth);

        // Inner cavity
        let inner = centered_cube(
            "inner",
            cfg.width - cfg.wall_thickness * 2.0,
            cfg.length - cfg.wall_thickness * 2.0,
            cfg.depth - cfg.wall_thickness,
        )
        .translate(0.0, 0.0, cfg.wall_thickness / 2.0);

        outer.difference(&inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downtube_battery() {
        let battery = DowntubeBattery::standard_48v();
        let part = battery.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_downtube_battery_simple() {
        let battery = DowntubeBattery::standard_48v();
        let simple = battery.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_custom_battery() {
        let battery = CustomBattery::bvr1_pack();
        let part = battery.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_custom_battery_energy() {
        let battery = CustomBattery::bvr1_pack();
        let energy = battery.energy_wh();
        // 13S4P at 3.6V * 5Ah = 46.8V * 20Ah = 936Wh
        assert!(energy > 900.0 && energy < 1000.0);
    }

    #[test]
    fn test_battery_tray() {
        let tray = BatteryTray::bvr1_tray();
        let part = tray.generate();
        assert!(!part.is_empty());
    }
}
