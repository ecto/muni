//! Motor mount for 8" hub motors
//!
//! Designed for mounting hub motors to 2020 aluminum extrusion frame.
//! Can be manufactured via laser cutting (SendCutSend) or 3D printing.

use crate::{bolt_pattern, centered_cube, centered_cylinder, Part};

/// Configuration for motor mount
#[derive(Debug, Clone)]
pub struct MotorMountConfig {
    /// Motor axle diameter (mm)
    pub axle_diameter: f64,
    /// Bolt circle diameter for motor mounting (mm)
    pub bolt_circle_diameter: f64,
    /// Number of mounting bolts
    pub num_motor_bolts: usize,
    /// Motor mounting bolt hole diameter (mm)
    pub motor_bolt_diameter: f64,
    /// Plate thickness (mm)
    pub plate_thickness: f64,
    /// Vertical plate width (mm)
    pub plate_width: f64,
    /// Vertical plate height (mm)
    pub plate_height: f64,
    /// Horizontal tab length for frame mounting (mm)
    pub tab_length: f64,
    /// Horizontal tab width (mm)
    pub tab_width: f64,
    /// Frame mounting bolt diameter (M5 = 5.5mm clearance) (mm)
    pub frame_bolt_diameter: f64,
    /// Distance between frame mounting holes (20mm for 2020 extrusion) (mm)
    pub frame_bolt_spacing: f64,
}

impl Default for MotorMountConfig {
    fn default() -> Self {
        Self {
            // 8" hub motor typical specs
            axle_diameter: 15.0,         // 15mm axle
            bolt_circle_diameter: 70.0,  // 70mm bolt circle
            num_motor_bolts: 4,          // 4 mounting bolts
            motor_bolt_diameter: 6.5,    // M6 clearance holes
            plate_thickness: 6.0,        // 6mm aluminum plate
            plate_width: 100.0,          // 100mm wide plate
            plate_height: 100.0,         // 100mm tall plate
            tab_length: 40.0,            // 40mm tabs
            tab_width: 20.0,             // 20mm wide tabs (matches 2020)
            frame_bolt_diameter: 5.5,    // M5 clearance holes
            frame_bolt_spacing: 20.0,    // 20mm spacing for 2020
        }
    }
}

impl MotorMountConfig {
    /// Create config for common 8" hub motor
    pub fn hub_motor_8in() -> Self {
        Self::default()
    }

    /// Create config for 6.5" hoverboard motor (like BVR0)
    pub fn hub_motor_6_5in() -> Self {
        Self {
            axle_diameter: 12.0,
            bolt_circle_diameter: 55.0,
            num_motor_bolts: 4,
            motor_bolt_diameter: 5.5, // M5
            plate_thickness: 5.0,
            plate_width: 80.0,
            plate_height: 80.0,
            tab_length: 35.0,
            tab_width: 20.0,
            frame_bolt_diameter: 5.5,
            frame_bolt_spacing: 20.0,
        }
    }
}

/// Motor mount part generator
pub struct MotorMount {
    config: MotorMountConfig,
}

impl MotorMount {
    pub fn new(config: MotorMountConfig) -> Self {
        Self { config }
    }

    /// Create with default 8" hub motor config
    pub fn hub_motor_8in() -> Self {
        Self::new(MotorMountConfig::hub_motor_8in())
    }

    /// Generate the motor mount part
    ///
    /// The mount is an L-bracket:
    /// - Vertical plate: faces outward, motor mounts here
    /// - Horizontal tabs: extend inward to bolt to 2020 extrusion
    ///
    /// Origin is at center of motor axle position.
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 64; // High quality circles

        // Main vertical plate (motor mounting surface)
        let main_plate = centered_cube(
            "main_plate",
            cfg.plate_width,
            cfg.plate_thickness,
            cfg.plate_height,
        );

        // Center axle hole
        let axle_hole = centered_cylinder("axle_hole", cfg.axle_diameter / 2.0, cfg.plate_thickness * 2.0, segments)
            .rotate(90.0, 0.0, 0.0);

        // Motor mounting bolt holes
        let motor_bolts = bolt_pattern(
            cfg.num_motor_bolts,
            cfg.bolt_circle_diameter,
            cfg.motor_bolt_diameter,
            cfg.plate_thickness * 2.0,
            segments,
        )
        .rotate(90.0, 0.0, 0.0);

        // Create horizontal tabs at top and bottom
        let tab_offset_z = cfg.plate_height / 2.0 - cfg.tab_width / 2.0;

        let top_tab = centered_cube("top_tab", cfg.tab_width, cfg.tab_length, cfg.plate_thickness)
            .translate(0.0, -cfg.tab_length / 2.0 - cfg.plate_thickness / 2.0, tab_offset_z);

        let bottom_tab = centered_cube("bottom_tab", cfg.tab_width, cfg.tab_length, cfg.plate_thickness)
            .translate(0.0, -cfg.tab_length / 2.0 - cfg.plate_thickness / 2.0, -tab_offset_z);

        // Frame mounting holes in tabs
        let frame_hole_offset = cfg.tab_length / 2.0;

        let top_frame_holes = self.create_tab_holes(segments)
            .translate(0.0, -cfg.plate_thickness / 2.0 - frame_hole_offset, tab_offset_z);

        let bottom_frame_holes = self.create_tab_holes(segments)
            .translate(0.0, -cfg.plate_thickness / 2.0 - frame_hole_offset, -tab_offset_z);

        // Combine all parts
        let base = main_plate.union(&top_tab).union(&bottom_tab);
        let holes = axle_hole
            .union(&motor_bolts)
            .union(&top_frame_holes)
            .union(&bottom_frame_holes);

        base.difference(&holes)
    }

    /// Create frame mounting holes for a tab (holes along Y axis)
    fn create_tab_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;

        // Two holes spaced along the tab length (Y direction)
        let hole1 = centered_cylinder("frame_hole1", cfg.frame_bolt_diameter / 2.0, cfg.plate_thickness * 2.0, segments)
            .translate(0.0, -cfg.frame_bolt_spacing / 2.0, 0.0);
        let hole2 = centered_cylinder("frame_hole2", cfg.frame_bolt_diameter / 2.0, cfg.plate_thickness * 2.0, segments)
            .translate(0.0, cfg.frame_bolt_spacing / 2.0, 0.0);

        hole1.union(&hole2)
    }

    /// Generate a simplified 2D profile for laser cutting (DXF export)
    ///
    /// This creates just the vertical plate as a flat profile.
    pub fn generate_flat_plate(&self) -> Part {
        let cfg = &self.config;
        let segments = 64;

        // Flat plate in XY plane
        let plate = centered_cube("plate", cfg.plate_width, cfg.plate_height, cfg.plate_thickness);

        // Axle hole
        let axle_hole = centered_cylinder("axle", cfg.axle_diameter / 2.0, cfg.plate_thickness * 2.0, segments);

        // Motor bolt holes
        let motor_bolts = bolt_pattern(
            cfg.num_motor_bolts,
            cfg.bolt_circle_diameter,
            cfg.motor_bolt_diameter,
            cfg.plate_thickness * 2.0,
            segments,
        );

        plate.difference(&axle_hole).difference(&motor_bolts)
    }

    /// Generate the mounting tabs as separate flat parts
    pub fn generate_tabs(&self) -> Part {
        let cfg = &self.config;
        let segments = 64;

        // Single tab: width (X) × length (Y) × thickness (Z)
        let tab = centered_cube("tab", cfg.tab_width, cfg.tab_length, cfg.plate_thickness);

        // Mounting holes spaced along Y (tab length), not X (tab width)
        let hole1 = centered_cylinder("hole1", cfg.frame_bolt_diameter / 2.0, cfg.plate_thickness * 2.0, segments)
            .translate(0.0, -cfg.frame_bolt_spacing / 2.0, 0.0);
        let hole2 = centered_cylinder("hole2", cfg.frame_bolt_diameter / 2.0, cfg.plate_thickness * 2.0, segments)
            .translate(0.0, cfg.frame_bolt_spacing / 2.0, 0.0);
        let holes = hole1.union(&hole2);

        tab.difference(&holes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_mount_generation() {
        let mount = MotorMount::hub_motor_8in();
        let part = mount.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_flat_plate_generation() {
        let mount = MotorMount::hub_motor_8in();
        let part = mount.generate_flat_plate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_tabs_generation() {
        let mount = MotorMount::hub_motor_8in();
        let part = mount.generate_tabs();
        assert!(!part.is_empty());
    }
}
