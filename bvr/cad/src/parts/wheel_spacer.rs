//! Wheel spacer for hub motors
//!
//! Spaces the wheel/hub motor from the motor mount plate.
//! Simple flat plate with center hole and bolt pattern.

use crate::{bolt_pattern, centered_cube, centered_cylinder, Part};

/// Configuration for wheel spacer
#[derive(Debug, Clone)]
pub struct WheelSpacerConfig {
    /// Motor axle diameter (mm)
    pub axle_diameter: f64,
    /// Bolt circle diameter (mm)
    pub bolt_circle_diameter: f64,
    /// Number of bolts
    pub num_bolts: usize,
    /// Bolt hole diameter (mm)
    pub bolt_diameter: f64,
    /// Spacer thickness (mm)
    pub thickness: f64,
    /// Outer diameter (mm)
    pub outer_diameter: f64,
}

impl Default for WheelSpacerConfig {
    fn default() -> Self {
        Self {
            axle_diameter: 15.0,        // Match motor mount
            bolt_circle_diameter: 70.0, // Match motor mount
            num_bolts: 4,
            bolt_diameter: 6.5,         // M6 clearance
            thickness: 10.0,            // 10mm spacer
            outer_diameter: 90.0,       // Slightly larger than bolt circle
        }
    }
}

impl WheelSpacerConfig {
    pub fn hub_motor_8in() -> Self {
        Self::default()
    }

    pub fn hub_motor_6_5in() -> Self {
        Self {
            axle_diameter: 12.0,
            bolt_circle_diameter: 55.0,
            num_bolts: 4,
            bolt_diameter: 5.5,
            thickness: 8.0,
            outer_diameter: 75.0,
        }
    }
}

/// Wheel spacer generator
pub struct WheelSpacer {
    config: WheelSpacerConfig,
}

impl WheelSpacer {
    pub fn new(config: WheelSpacerConfig) -> Self {
        Self { config }
    }

    pub fn hub_motor_8in() -> Self {
        Self::new(WheelSpacerConfig::hub_motor_8in())
    }

    /// Generate the wheel spacer
    ///
    /// Circular plate with center axle hole and bolt pattern.
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 64;

        // Main circular body
        let body = centered_cylinder("body", cfg.outer_diameter / 2.0, cfg.thickness, segments);

        // Center axle hole
        let axle_hole = centered_cylinder("axle", cfg.axle_diameter / 2.0, cfg.thickness * 2.0, segments);

        // Bolt holes
        let bolt_holes = bolt_pattern(
            cfg.num_bolts,
            cfg.bolt_circle_diameter,
            cfg.bolt_diameter,
            cfg.thickness * 2.0,
            segments,
        );

        body.difference(&axle_hole).difference(&bolt_holes)
    }

    /// Generate flat version for laser cutting (rectangular stock)
    pub fn generate_flat(&self) -> Part {
        let cfg = &self.config;
        let segments = 64;

        // Square plate that fits the circular part
        let size = cfg.outer_diameter;
        let body = centered_cube("body", size, size, cfg.thickness);

        // Center axle hole
        let axle_hole = centered_cylinder("axle", cfg.axle_diameter / 2.0, cfg.thickness * 2.0, segments);

        // Bolt holes
        let bolt_holes = bolt_pattern(
            cfg.num_bolts,
            cfg.bolt_circle_diameter,
            cfg.bolt_diameter,
            cfg.thickness * 2.0,
            segments,
        );

        body.difference(&axle_hole).difference(&bolt_holes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_spacer_generation() {
        let spacer = WheelSpacer::hub_motor_8in();
        let part = spacer.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_wheel_spacer_flat() {
        let spacer = WheelSpacer::hub_motor_8in();
        let part = spacer.generate_flat();
        assert!(!part.is_empty());
    }
}
