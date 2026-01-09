//! Electronics reference geometry
//!
//! Simplified models for visualization and assembly.
//! Not for manufacturing - these are reference parts.

use crate::{centered_cube, centered_cylinder, Part};

// =============================================================================
// VESC Motor Controller
// =============================================================================

/// VESC configuration
#[derive(Debug, Clone)]
pub struct VescConfig {
    /// PCB length (mm)
    pub length: f64,
    /// PCB width (mm)
    pub width: f64,
    /// Total height including heatsink (mm)
    pub height: f64,
    /// Mounting hole spacing X (mm)
    pub mount_spacing_x: f64,
    /// Mounting hole spacing Y (mm)
    pub mount_spacing_y: f64,
}

impl VescConfig {
    /// VESC 6.6 / 6.7 (Flipsky)
    pub fn vesc_6() -> Self {
        Self {
            length: 77.0,
            width: 56.0,
            height: 25.0,
            mount_spacing_x: 67.0,
            mount_spacing_y: 46.0,
        }
    }

    /// VESC 75/300 (high power)
    pub fn vesc_75_300() -> Self {
        Self {
            length: 100.0,
            width: 70.0,
            height: 35.0,
            mount_spacing_x: 90.0,
            mount_spacing_y: 60.0,
        }
    }
}

/// VESC motor controller reference model
pub struct Vesc {
    config: VescConfig,
}

impl Vesc {
    pub fn new(config: VescConfig) -> Self {
        Self { config }
    }

    pub fn vesc_6() -> Self {
        Self::new(VescConfig::vesc_6())
    }

    /// Generate VESC geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Main body (simplified as box)
        let body = centered_cube("body", cfg.length, cfg.width, cfg.height);

        // Heatsink fins (simplified)
        let fin_count = 5;
        let fin_width = cfg.width * 0.8;
        let fin_height = cfg.height * 0.3;
        let fin_spacing = cfg.length / (fin_count as f64 + 1.0);

        let mut fins = Part::empty("fins");
        for i in 1..=fin_count {
            let x = -cfg.length / 2.0 + fin_spacing * (i as f64);
            let fin = centered_cube("fin", 2.0, fin_width, fin_height)
                .translate(x, 0.0, cfg.height / 2.0 + fin_height / 2.0);
            fins = fins.union(&fin);
        }

        body.union(&fins)
    }

    /// Generate simplified geometry
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("vesc", cfg.length, cfg.width, cfg.height)
    }
}

// =============================================================================
// Jetson Orin NX + Carrier Board
// =============================================================================

/// Jetson carrier board configuration
#[derive(Debug, Clone)]
pub struct JetsonConfig {
    /// Board length (mm)
    pub length: f64,
    /// Board width (mm)
    pub width: f64,
    /// Total height including module and heatsink (mm)
    pub height: f64,
    /// Mounting hole spacing X (mm)
    pub mount_spacing_x: f64,
    /// Mounting hole spacing Y (mm)
    pub mount_spacing_y: f64,
}

impl JetsonConfig {
    /// Seeed reComputer J401 carrier with Orin NX
    pub fn recomputer_j401() -> Self {
        Self {
            length: 130.0,
            width: 120.0,
            height: 50.0,  // Including heatsink
            mount_spacing_x: 58.0,
            mount_spacing_y: 58.0,
        }
    }

    /// Waveshare carrier
    pub fn waveshare() -> Self {
        Self {
            length: 100.0,
            width: 80.0,
            height: 45.0,
            mount_spacing_x: 58.0,
            mount_spacing_y: 58.0,
        }
    }
}

/// Jetson compute module + carrier board reference model
pub struct Jetson {
    config: JetsonConfig,
}

impl Jetson {
    pub fn new(config: JetsonConfig) -> Self {
        Self { config }
    }

    pub fn recomputer() -> Self {
        Self::new(JetsonConfig::recomputer_j401())
    }

    /// Generate Jetson geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Carrier board (base)
        let board_thickness = 5.0;
        let board = centered_cube("board", cfg.length, cfg.width, board_thickness)
            .translate(0.0, 0.0, board_thickness / 2.0);

        // Compute module (centered on board)
        let module_size = 70.0;
        let module_height = 15.0;
        let module = centered_cube("module", module_size, module_size, module_height)
            .translate(0.0, 0.0, board_thickness + module_height / 2.0);

        // Heatsink (on top of module)
        let heatsink_height = cfg.height - board_thickness - module_height;
        let heatsink = centered_cube("heatsink", module_size + 10.0, module_size + 10.0, heatsink_height)
            .translate(0.0, 0.0, board_thickness + module_height + heatsink_height / 2.0);

        // IO connectors (simplified as boxes on edges)
        let usb_block = centered_cube("usb", 15.0, 15.0, 12.0)
            .translate(cfg.length / 2.0 - 10.0, cfg.width / 2.0 - 20.0, board_thickness + 6.0);

        let ethernet = centered_cube("ethernet", 16.0, 14.0, 14.0)
            .translate(-cfg.length / 2.0 + 10.0, cfg.width / 2.0 - 15.0, board_thickness + 7.0);

        board.union(&module).union(&heatsink).union(&usb_block).union(&ethernet)
    }

    /// Generate simplified geometry
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("jetson", cfg.length, cfg.width, cfg.height)
    }
}

// =============================================================================
// DC-DC Converter
// =============================================================================

/// DC-DC converter configuration
#[derive(Debug, Clone)]
pub struct DcdcConfig {
    /// Body length (mm)
    pub length: f64,
    /// Body width (mm)
    pub width: f64,
    /// Body height (mm)
    pub height: f64,
}

impl Default for DcdcConfig {
    fn default() -> Self {
        Self {
            length: 60.0,
            width: 45.0,
            height: 25.0,
        }
    }
}

/// DC-DC converter reference model
pub struct DcDc {
    config: DcdcConfig,
}

impl DcDc {
    pub fn new(config: DcdcConfig) -> Self {
        Self { config }
    }

    pub fn default_48v_12v() -> Self {
        Self::new(DcdcConfig::default())
    }

    /// Generate DC-DC converter geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        centered_cube("dcdc", cfg.length, cfg.width, cfg.height)
    }
}

// =============================================================================
// E-Stop Button
// =============================================================================

/// E-Stop mushroom button reference model
pub struct EStopButton {
    /// Button diameter (mm)
    pub diameter: f64,
    /// Button height (mm)
    pub height: f64,
    /// Base diameter (mm)
    pub base_diameter: f64,
}

impl Default for EStopButton {
    fn default() -> Self {
        Self {
            diameter: 40.0,
            height: 30.0,
            base_diameter: 50.0,
        }
    }
}

impl EStopButton {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate E-Stop button geometry
    pub fn generate(&self) -> Part {
        let segments = 32;

        // Base/mounting ring
        let base = centered_cylinder("base", self.base_diameter / 2.0, 10.0, segments)
            .translate(0.0, 0.0, 5.0);

        // Mushroom button (dome approximated as cylinder + smaller cylinder)
        let button_body = centered_cylinder("button", self.diameter / 2.0, self.height * 0.7, segments)
            .translate(0.0, 0.0, 10.0 + self.height * 0.35);

        let button_top = centered_cylinder("button_top", self.diameter / 2.0 - 5.0, self.height * 0.3, segments)
            .translate(0.0, 0.0, 10.0 + self.height * 0.85);

        base.union(&button_body).union(&button_top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesc_6() {
        let vesc = Vesc::vesc_6();
        let part = vesc.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_vesc_simple() {
        let vesc = Vesc::vesc_6();
        let simple = vesc.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_jetson_recomputer() {
        let jetson = Jetson::recomputer();
        let part = jetson.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_jetson_simple() {
        let jetson = Jetson::recomputer();
        let simple = jetson.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_dcdc() {
        let dcdc = DcDc::default_48v_12v();
        let part = dcdc.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_estop() {
        let estop = EStopButton::new();
        let part = estop.generate();
        assert!(!part.is_empty());
    }
}
