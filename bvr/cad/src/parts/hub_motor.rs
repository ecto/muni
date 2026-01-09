//! Hub motor and wheel reference geometry
//!
//! Simplified models for visualization and assembly.
//! Not for manufacturing - these are reference parts.

use crate::{centered_cylinder, Part};

/// Hub motor configuration
#[derive(Debug, Clone)]
pub struct HubMotorConfig {
    /// Wheel outer diameter (mm)
    pub wheel_diameter: f64,
    /// Wheel width (mm)
    pub wheel_width: f64,
    /// Hub/motor body diameter (mm)
    pub hub_diameter: f64,
    /// Hub/motor body width (mm)
    pub hub_width: f64,
    /// Axle diameter (mm)
    pub axle_diameter: f64,
    /// Axle length (protrusion from hub) (mm)
    pub axle_length: f64,
}

impl HubMotorConfig {
    /// 8" hub motor (common for e-bikes and rovers)
    pub fn hub_8in() -> Self {
        Self {
            wheel_diameter: 200.0,  // ~8 inches
            wheel_width: 55.0,
            hub_diameter: 120.0,
            hub_width: 45.0,
            axle_diameter: 15.0,
            axle_length: 25.0,
        }
    }

    /// 6.5" hoverboard motor (BVR0 style)
    pub fn hoverboard_6_5in() -> Self {
        Self {
            wheel_diameter: 165.0,  // ~6.5 inches
            wheel_width: 50.0,
            hub_diameter: 100.0,
            hub_width: 40.0,
            axle_diameter: 12.0,
            axle_length: 20.0,
        }
    }

    /// 10" scooter motor
    pub fn scooter_10in() -> Self {
        Self {
            wheel_diameter: 254.0,  // 10 inches
            wheel_width: 65.0,
            hub_diameter: 140.0,
            hub_width: 50.0,
            axle_diameter: 15.0,
            axle_length: 30.0,
        }
    }
}

/// Hub motor with integrated wheel
pub struct HubMotor {
    config: HubMotorConfig,
}

impl HubMotor {
    pub fn new(config: HubMotorConfig) -> Self {
        Self { config }
    }

    pub fn hub_8in() -> Self {
        Self::new(HubMotorConfig::hub_8in())
    }

    pub fn hoverboard() -> Self {
        Self::new(HubMotorConfig::hoverboard_6_5in())
    }

    /// Generate hub motor geometry
    ///
    /// Orientation: wheel rotates around X axis, axle points in +X direction
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Tire/wheel (outer ring)
        let tire = self.create_tire(segments);

        // Hub/motor body (center)
        let hub = centered_cylinder("hub", cfg.hub_diameter / 2.0, cfg.hub_width, segments)
            .rotate(0.0, 90.0, 0.0);

        // Axle (extends from hub)
        let axle = centered_cylinder("axle", cfg.axle_diameter / 2.0, cfg.axle_length, segments)
            .rotate(0.0, 90.0, 0.0)
            .translate(cfg.hub_width / 2.0 + cfg.axle_length / 2.0, 0.0, 0.0);

        tire.union(&hub).union(&axle)
    }

    /// Create the tire/wheel (simplified as a thick ring)
    fn create_tire(&self, segments: u32) -> Part {
        let cfg = &self.config;

        // Outer cylinder
        let outer = centered_cylinder("tire_outer", cfg.wheel_diameter / 2.0, cfg.wheel_width, segments)
            .rotate(0.0, 90.0, 0.0);

        // Inner cutout (where hub goes)
        let inner = centered_cylinder("tire_inner", cfg.hub_diameter / 2.0 + 5.0, cfg.wheel_width + 2.0, segments)
            .rotate(0.0, 90.0, 0.0);

        outer.difference(&inner)
    }

    /// Generate simplified wheel (just a cylinder, for fast preview)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        centered_cylinder("wheel", cfg.wheel_diameter / 2.0, cfg.wheel_width, segments)
            .rotate(0.0, 90.0, 0.0)
    }

    /// Get wheel diameter
    pub fn wheel_diameter(&self) -> f64 {
        self.config.wheel_diameter
    }

    /// Get total width (hub + axle)
    pub fn total_width(&self) -> f64 {
        self.config.hub_width + self.config.axle_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_motor_8in() {
        let motor = HubMotor::hub_8in();
        let part = motor.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_hub_motor_hoverboard() {
        let motor = HubMotor::hoverboard();
        let part = motor.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_hub_motor_simple() {
        let motor = HubMotor::hub_8in();
        let simple = motor.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_hub_motor_dimensions() {
        let motor = HubMotor::hub_8in();
        assert_eq!(motor.wheel_diameter(), 200.0);
    }
}
