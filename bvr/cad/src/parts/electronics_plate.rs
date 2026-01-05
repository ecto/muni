//! Electronics mounting plate
//!
//! Main plate for mounting electronics: Jetson, VESCs, etc.
//! Designed for 2020 extrusion frame mounting.

use crate::{centered_cube, centered_cylinder, Part};

/// Configuration for electronics plate
#[derive(Debug, Clone)]
pub struct ElectronicsPlateConfig {
    /// Plate width (X) in mm
    pub width: f64,
    /// Plate depth (Y) in mm
    pub depth: f64,
    /// Plate thickness in mm (1/4" = 6.35mm)
    pub thickness: f64,
    /// Corner radius for rounded corners (0 for sharp)
    pub corner_radius: f64,
    /// Frame mounting hole diameter (M5 = 5.5mm)
    pub frame_hole_diameter: f64,
    /// Frame mounting hole inset from edge
    pub frame_hole_inset: f64,
    /// Jetson carrier mounting hole spacing (typical 58x58mm or custom)
    pub jetson_mount_spacing: (f64, f64),
    /// Jetson mounting hole diameter (M3 = 3.5mm)
    pub jetson_hole_diameter: f64,
    /// VESC mounting hole spacing (typical 77x56mm for VESC 6)
    pub vesc_mount_spacing: (f64, f64),
    /// VESC mounting hole diameter (M4 = 4.5mm)
    pub vesc_hole_diameter: f64,
}

impl Default for ElectronicsPlateConfig {
    fn default() -> Self {
        Self {
            width: 300.0,              // 300mm wide
            depth: 200.0,              // 200mm deep
            thickness: 6.35,           // 1/4" aluminum
            corner_radius: 10.0,       // Rounded corners
            frame_hole_diameter: 5.5,  // M5 clearance
            frame_hole_inset: 15.0,    // 15mm from edge
            // Seeed Studio Jetson carrier (approximate)
            jetson_mount_spacing: (58.0, 58.0),
            jetson_hole_diameter: 3.5, // M3 clearance
            // VESC 6.6 mounting pattern
            vesc_mount_spacing: (77.0, 56.0),
            vesc_hole_diameter: 4.5,   // M4 clearance
        }
    }
}

/// Electronics plate generator
pub struct ElectronicsPlate {
    config: ElectronicsPlateConfig,
}

impl ElectronicsPlate {
    pub fn new(config: ElectronicsPlateConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(ElectronicsPlateConfig::default())
    }

    /// Generate the full electronics plate with all mounting holes
    ///
    /// Layout (top view):
    /// ```text
    /// +----------------------------------+
    /// |  o                            o  |  <- frame mount holes (corners)
    /// |      +------+    +------+        |
    /// |      | VESC |    | VESC |        |  <- 2x VESC (front)
    /// |      +------+    +------+        |
    /// |                                  |
    /// |          +----------+            |
    /// |          |  JETSON  |            |  <- Jetson (center)
    /// |          +----------+            |
    /// |                                  |
    /// |      +------+    +------+        |
    /// |      | VESC |    | VESC |        |  <- 2x VESC (rear)
    /// |      +------+    +------+        |
    /// |  o                            o  |  <- frame mount holes (corners)
    /// +----------------------------------+
    /// ```
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Main plate body
        let plate = centered_cube("plate", cfg.width, cfg.depth, cfg.thickness);

        // Frame mounting holes (4 corners)
        let frame_holes = self.create_corner_holes(segments);

        // Jetson mounting holes (center)
        let jetson_holes = self.create_jetson_mount(segments);

        // VESC mounting holes (4 units in 2x2 grid)
        let vesc_holes = self.create_vesc_mounts(segments);

        // Ventilation/cable routing slots (optional)
        let vent_slots = self.create_vent_slots(segments);

        plate
            .difference(&frame_holes)
            .difference(&jetson_holes)
            .difference(&vesc_holes)
            .difference(&vent_slots)
    }

    /// Create corner mounting holes for frame attachment
    fn create_corner_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let hole = |name: &str, x: f64, y: f64| {
            centered_cylinder(name, cfg.frame_hole_diameter / 2.0, cfg.thickness * 2.0, segments)
                .translate(x, y, 0.0)
        };

        let inset = cfg.frame_hole_inset;
        let hx = cfg.width / 2.0 - inset;
        let hy = cfg.depth / 2.0 - inset;

        hole("corner_fl", -hx, hy, )
            .union(&hole("corner_fr", hx, hy))
            .union(&hole("corner_rl", -hx, -hy))
            .union(&hole("corner_rr", hx, -hy))
    }

    /// Create Jetson carrier mounting holes (center of plate)
    fn create_jetson_mount(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let (sx, sy) = cfg.jetson_mount_spacing;
        let hx = sx / 2.0;
        let hy = sy / 2.0;

        let hole = |name: &str, x: f64, y: f64| {
            centered_cylinder(name, cfg.jetson_hole_diameter / 2.0, cfg.thickness * 2.0, segments)
                .translate(x, y, 0.0)
        };

        // Jetson centered at origin
        hole("jetson_1", -hx, hy)
            .union(&hole("jetson_2", hx, hy))
            .union(&hole("jetson_3", -hx, -hy))
            .union(&hole("jetson_4", hx, -hy))
    }

    /// Create VESC mounting holes (4 units in 2x2 layout)
    fn create_vesc_mounts(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let (vx, vy) = cfg.vesc_mount_spacing;

        // VESC positions (offset from center)
        let vesc_offset_x = 80.0; // Distance from center to VESC center
        let vesc_offset_y = 60.0;

        let create_vesc_holes = |name_prefix: &str, cx: f64, cy: f64| {
            let hx = vx / 2.0;
            let hy = vy / 2.0;

            let hole = |suffix: &str, x: f64, y: f64| {
                centered_cylinder(
                    &format!("{}_{}", name_prefix, suffix),
                    cfg.vesc_hole_diameter / 2.0,
                    cfg.thickness * 2.0,
                    segments,
                )
                .translate(cx + x, cy + y, 0.0)
            };

            hole("1", -hx, hy)
                .union(&hole("2", hx, hy))
                .union(&hole("3", -hx, -hy))
                .union(&hole("4", hx, -hy))
        };

        // 4 VESCs in corners
        create_vesc_holes("vesc_fl", -vesc_offset_x, vesc_offset_y)
            .union(&create_vesc_holes("vesc_fr", vesc_offset_x, vesc_offset_y))
            .union(&create_vesc_holes("vesc_rl", -vesc_offset_x, -vesc_offset_y))
            .union(&create_vesc_holes("vesc_rr", vesc_offset_x, -vesc_offset_y))
    }

    /// Create ventilation/cable routing slots
    fn create_vent_slots(&self, _segments: u32) -> Part {
        let cfg = &self.config;

        // Simple rectangular slots between components
        let slot_width = 15.0;
        let slot_length = 40.0;

        let slot = centered_cube("slot", slot_width, slot_length, cfg.thickness * 2.0);

        // Slots on left and right sides for cable routing
        slot.translate(-cfg.width / 4.0, 0.0, 0.0)
            .union(&slot.translate(cfg.width / 4.0, 0.0, 0.0))
    }

    /// Generate simplified plate with only frame mounting holes
    /// (for initial testing/ordering)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        let plate = centered_cube("plate", cfg.width, cfg.depth, cfg.thickness);
        let frame_holes = self.create_corner_holes(segments);

        plate.difference(&frame_holes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electronics_plate_generation() {
        let plate = ElectronicsPlate::default_bvr1();
        let part = plate.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_simple_plate() {
        let plate = ElectronicsPlate::default_bvr1();
        let part = plate.generate_simple();
        assert!(!part.is_empty());
    }
}
