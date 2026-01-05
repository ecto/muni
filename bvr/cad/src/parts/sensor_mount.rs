//! Sensor mount for LiDAR and cameras
//!
//! Mounting bracket for sensor mast (1" aluminum tube).
//! Attaches to 2020 extrusion frame.

use crate::{centered_cube, centered_cylinder, Part};

/// Configuration for sensor mount
#[derive(Debug, Clone)]
pub struct SensorMountConfig {
    /// Mast tube outer diameter (1" = 25.4mm)
    pub tube_od: f64,
    /// Mast tube clamp thickness
    pub clamp_thickness: f64,
    /// Base plate width
    pub base_width: f64,
    /// Base plate depth
    pub base_depth: f64,
    /// Base plate thickness
    pub base_thickness: f64,
    /// Frame bolt hole diameter (M5)
    pub frame_hole_diameter: f64,
    /// Clamp bolt hole diameter (M5)
    pub clamp_hole_diameter: f64,
    /// Height of vertical section
    pub vertical_height: f64,
}

impl Default for SensorMountConfig {
    fn default() -> Self {
        Self {
            tube_od: 25.4,             // 1" tube
            clamp_thickness: 8.0,       // 8mm clamp walls
            base_width: 60.0,           // 60mm base
            base_depth: 40.0,           // 40mm deep
            base_thickness: 6.0,        // 6mm plate
            frame_hole_diameter: 5.5,   // M5 clearance
            clamp_hole_diameter: 5.5,   // M5 clearance
            vertical_height: 50.0,      // 50mm riser
        }
    }
}

/// Sensor mount generator
pub struct SensorMount {
    config: SensorMountConfig,
}

impl SensorMount {
    pub fn new(config: SensorMountConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(SensorMountConfig::default())
    }

    /// Generate the sensor mount bracket
    ///
    /// L-shaped bracket with tube clamp on top:
    /// ```text
    ///        ___________
    ///       |   (O)    |  <- tube clamp
    ///       |__________|
    ///            |
    ///            |        <- vertical riser
    ///            |
    ///     _______|_______
    ///    |   o       o  |  <- base plate with frame holes
    ///    |______________|
    /// ```
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Base plate (horizontal)
        let base = centered_cube("base", cfg.base_width, cfg.base_depth, cfg.base_thickness)
            .translate(0.0, 0.0, cfg.base_thickness / 2.0);

        // Vertical riser
        let riser_width = cfg.tube_od + cfg.clamp_thickness * 2.0;
        let riser = centered_cube("riser", riser_width, cfg.base_thickness, cfg.vertical_height)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.vertical_height / 2.0);

        // Clamp block (at top of riser)
        let clamp_height = cfg.tube_od + cfg.clamp_thickness * 2.0;
        let clamp_block = centered_cube("clamp", riser_width, cfg.base_depth, clamp_height)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.vertical_height + clamp_height / 2.0);

        // Tube hole through clamp
        let tube_hole = centered_cylinder("tube", cfg.tube_od / 2.0, cfg.base_depth * 2.0, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.vertical_height + clamp_height / 2.0);

        // Clamp slit (to allow clamping)
        let slit_width = 3.0;
        let slit = centered_cube("slit", slit_width, cfg.base_depth * 2.0, clamp_height + 1.0)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.vertical_height + clamp_height / 2.0);

        // Clamp bolt holes (through the slit)
        let clamp_bolt_y = cfg.base_depth / 4.0;
        let clamp_bolt1 = centered_cylinder("clamp_bolt1", cfg.clamp_hole_diameter / 2.0, riser_width * 2.0, segments)
            .rotate(0.0, 90.0, 0.0)
            .translate(0.0, clamp_bolt_y, cfg.base_thickness + cfg.vertical_height + clamp_height / 2.0);
        let clamp_bolt2 = centered_cylinder("clamp_bolt2", cfg.clamp_hole_diameter / 2.0, riser_width * 2.0, segments)
            .rotate(0.0, 90.0, 0.0)
            .translate(0.0, -clamp_bolt_y, cfg.base_thickness + cfg.vertical_height + clamp_height / 2.0);

        // Base mounting holes
        let base_hole_x = cfg.base_width / 2.0 - 10.0;
        let base_hole1 = centered_cylinder("base_bolt1", cfg.frame_hole_diameter / 2.0, cfg.base_thickness * 2.0, segments)
            .translate(-base_hole_x, 0.0, cfg.base_thickness / 2.0);
        let base_hole2 = centered_cylinder("base_bolt2", cfg.frame_hole_diameter / 2.0, cfg.base_thickness * 2.0, segments)
            .translate(base_hole_x, 0.0, cfg.base_thickness / 2.0);

        base.union(&riser)
            .union(&clamp_block)
            .difference(&tube_hole)
            .difference(&slit)
            .difference(&clamp_bolt1)
            .difference(&clamp_bolt2)
            .difference(&base_hole1)
            .difference(&base_hole2)
    }

    /// Generate just the base plate (for laser cutting)
    pub fn generate_base_plate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        let base = centered_cube("base", cfg.base_width, cfg.base_depth, cfg.base_thickness);

        // Mounting holes
        let hole_x = cfg.base_width / 2.0 - 10.0;
        let hole1 = centered_cylinder("hole1", cfg.frame_hole_diameter / 2.0, cfg.base_thickness * 2.0, segments)
            .translate(-hole_x, 0.0, 0.0);
        let hole2 = centered_cylinder("hole2", cfg.frame_hole_diameter / 2.0, cfg.base_thickness * 2.0, segments)
            .translate(hole_x, 0.0, 0.0);

        // Center hole for tube or wiring
        let center_hole = centered_cylinder("center", cfg.tube_od / 2.0 + 2.0, cfg.base_thickness * 2.0, segments);

        base.difference(&hole1).difference(&hole2).difference(&center_hole)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_mount_generation() {
        let mount = SensorMount::default_bvr1();
        let part = mount.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_base_plate_generation() {
        let mount = SensorMount::default_bvr1();
        let part = mount.generate_base_plate();
        assert!(!part.is_empty());
    }
}
