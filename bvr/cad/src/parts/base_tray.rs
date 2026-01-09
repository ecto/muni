//! Base tray for BVR1
//!
//! Combined mounting tray for battery and electronics at the bottom of the frame.
//! All components are coplanar for simpler design and lower center of gravity.

use crate::{centered_cube, centered_cylinder, Part};

/// Base tray configuration
#[derive(Debug, Clone)]
pub struct BaseTrayConfig {
    /// Tray width (X dimension, mm)
    pub width: f64,
    /// Tray length (Y dimension, mm)
    pub length: f64,
    /// Tray thickness (mm)
    pub thickness: f64,
    /// Corner radius for mounting holes (mm)
    pub corner_inset: f64,
    /// Mounting hole diameter (mm)
    pub mount_hole_diameter: f64,
}

impl Default for BaseTrayConfig {
    fn default() -> Self {
        // Sized for compact 380x500mm ADA-compliant frame
        Self {
            width: 340.0,   // Fits inside 380mm frame (380 - 2*20)
            length: 460.0,  // Fits inside 500mm frame (500 - 2*20)
            thickness: 6.0, // 6mm aluminum
            corner_inset: 15.0,
            mount_hole_diameter: 5.5, // M5 clearance
        }
    }
}

/// Base tray for battery and electronics
pub struct BaseTray {
    config: BaseTrayConfig,
}

impl BaseTray {
    pub fn new(config: BaseTrayConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(BaseTrayConfig::default())
    }

    /// Generate the base tray
    ///
    /// Layout (top view):
    /// ```text
    /// ┌─────────────────────────────────────┐
    /// │  ○                             ○    │  <- Mounting holes
    /// │  ┌─────┐ ┌─────┐     ┌──────────┐   │
    /// │  │VESC │ │VESC │     │  JETSON  │   │
    /// │  └─────┘ └─────┘     └──────────┘   │
    /// │  ┌─────────────────────────────────┐│
    /// │  │                                 ││
    /// │  │        BATTERY PACK             ││
    /// │  │         (center)                ││
    /// │  │                                 ││
    /// │  └─────────────────────────────────┘│
    /// │  ┌─────┐ ┌─────┐     ┌──────────┐   │
    /// │  │VESC │ │VESC │     │  DC-DC   │   │
    /// │  └─────┘ └─────┘     └──────────┘   │
    /// │  ○                             ○    │
    /// └─────────────────────────────────────┘
    /// ```
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Main tray plate
        let plate = centered_cube("tray", cfg.width, cfg.length, cfg.thickness);

        // Mounting holes at corners
        let hole_x = cfg.width / 2.0 - cfg.corner_inset;
        let hole_y = cfg.length / 2.0 - cfg.corner_inset;

        let hole = centered_cylinder("mount_hole", cfg.mount_hole_diameter / 2.0, cfg.thickness * 2.0, segments);

        let hole_fl = hole.translate(-hole_x, hole_y, 0.0);
        let hole_fr = hole.translate(hole_x, hole_y, 0.0);
        let hole_bl = hole.translate(-hole_x, -hole_y, 0.0);
        let hole_br = hole.translate(hole_x, -hole_y, 0.0);

        // Additional mounting holes for components (Jetson pattern)
        let jetson_x = cfg.width / 2.0 - 80.0;
        let jetson_y = cfg.length / 2.0 - 80.0;
        let jetson_hole_spacing = 58.0;

        let jh1 = hole.translate(jetson_x - jetson_hole_spacing / 2.0, jetson_y - jetson_hole_spacing / 2.0, 0.0);
        let jh2 = hole.translate(jetson_x + jetson_hole_spacing / 2.0, jetson_y - jetson_hole_spacing / 2.0, 0.0);
        let jh3 = hole.translate(jetson_x - jetson_hole_spacing / 2.0, jetson_y + jetson_hole_spacing / 2.0, 0.0);
        let jh4 = hole.translate(jetson_x + jetson_hole_spacing / 2.0, jetson_y + jetson_hole_spacing / 2.0, 0.0);

        let holes = hole_fl
            .union(&hole_fr)
            .union(&hole_bl)
            .union(&hole_br)
            .union(&jh1)
            .union(&jh2)
            .union(&jh3)
            .union(&jh4);

        plate.difference(&holes)
    }

    /// Generate simplified tray (just the plate outline)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube("tray", cfg.width, cfg.length, cfg.thickness)
    }

    /// Get tray dimensions
    pub fn dimensions(&self) -> (f64, f64, f64) {
        (self.config.width, self.config.length, self.config.thickness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_tray() {
        let tray = BaseTray::default_bvr1();
        let part = tray.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_base_tray_simple() {
        let tray = BaseTray::default_bvr1();
        let part = tray.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_tray_dimensions() {
        let tray = BaseTray::default_bvr1();
        let (w, l, t) = tray.dimensions();
        // Sized for compact 380x500mm ADA-compliant frame
        assert_eq!(w, 340.0);  // 380 - 2*20
        assert_eq!(l, 460.0);  // 500 - 2*20
        assert_eq!(t, 6.0);
    }
}
