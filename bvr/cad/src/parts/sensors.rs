//! Sensor reference geometry
//!
//! Simplified models of sensors for visualization and assembly.
//! Not for manufacturing - these are reference parts.

use crate::{centered_cylinder, Part};

// =============================================================================
// LiDAR
// =============================================================================

/// LiDAR configuration
#[derive(Debug, Clone)]
pub struct LidarConfig {
    /// Body diameter (mm)
    pub diameter: f64,
    /// Body height (mm)
    pub height: f64,
    /// Base/mounting plate diameter (mm)
    pub base_diameter: f64,
    /// Base thickness (mm)
    pub base_thickness: f64,
}

impl LidarConfig {
    /// Livox Mid-360 (primary sensor for BVR1)
    pub fn livox_mid360() -> Self {
        Self {
            diameter: 100.0,
            height: 77.0,
            base_diameter: 110.0,
            base_thickness: 10.0,
        }
    }

    /// Livox Avia (alternative)
    pub fn livox_avia() -> Self {
        Self {
            diameter: 80.0,
            height: 65.0,
            base_diameter: 90.0,
            base_thickness: 8.0,
        }
    }
}

/// LiDAR sensor reference model
pub struct Lidar {
    config: LidarConfig,
}

impl Lidar {
    pub fn new(config: LidarConfig) -> Self {
        Self { config }
    }

    pub fn mid360() -> Self {
        Self::new(LidarConfig::livox_mid360())
    }

    /// Generate LiDAR geometry
    ///
    /// Orientation: scanning plane is XY, sensor points up (+Z)
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Main body (cylindrical)
        let body = centered_cylinder("body", cfg.diameter / 2.0, cfg.height, segments)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.height / 2.0);

        // Mounting base (wider disk)
        let base = centered_cylinder("base", cfg.base_diameter / 2.0, cfg.base_thickness, segments)
            .translate(0.0, 0.0, cfg.base_thickness / 2.0);

        // Lens dome (top)
        let dome_height = cfg.height * 0.3;
        let dome = centered_cylinder("dome", cfg.diameter / 2.0 - 5.0, dome_height, segments)
            .translate(0.0, 0.0, cfg.base_thickness + cfg.height + dome_height / 2.0);

        base.union(&body).union(&dome)
    }

    /// Generate simplified geometry (just a cylinder)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let total_height = cfg.base_thickness + cfg.height;

        centered_cylinder("lidar", cfg.diameter / 2.0, total_height, 32)
            .translate(0.0, 0.0, total_height / 2.0)
    }
}

// =============================================================================
// Camera
// =============================================================================

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// Body diameter (for 360 cameras) or width (mm)
    pub diameter: f64,
    /// Body height (mm)
    pub height: f64,
    /// Lens diameter (mm)
    pub lens_diameter: f64,
}

impl CameraConfig {
    /// Insta360 X4 (360° camera for BVR1)
    pub fn insta360_x4() -> Self {
        Self {
            diameter: 46.0,
            height: 125.0,
            lens_diameter: 20.0,
        }
    }

    /// Insta360 ONE RS (alternative)
    pub fn insta360_one_rs() -> Self {
        Self {
            diameter: 48.0,
            height: 110.0,
            lens_diameter: 18.0,
        }
    }
}

/// Camera sensor reference model
pub struct Camera {
    config: CameraConfig,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        Self { config }
    }

    pub fn insta360_x4() -> Self {
        Self::new(CameraConfig::insta360_x4())
    }

    /// Generate camera geometry
    ///
    /// Orientation: camera stands upright, lenses on top
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Main body (rounded box approximated as cylinder)
        let body = centered_cylinder("body", cfg.diameter / 2.0, cfg.height, segments)
            .translate(0.0, 0.0, cfg.height / 2.0);

        // Front lens (hemisphere approximated as short cylinder)
        let lens_height = cfg.lens_diameter * 0.5;
        let front_lens = centered_cylinder("front_lens", cfg.lens_diameter / 2.0, lens_height, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, cfg.diameter / 2.0, cfg.height - 20.0);

        // Back lens
        let back_lens = centered_cylinder("back_lens", cfg.lens_diameter / 2.0, lens_height, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, -cfg.diameter / 2.0, cfg.height - 20.0);

        body.union(&front_lens).union(&back_lens)
    }

    /// Generate simplified geometry
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;

        centered_cylinder("camera", cfg.diameter / 2.0, cfg.height, 24)
            .translate(0.0, 0.0, cfg.height / 2.0)
    }
}

// =============================================================================
// GPS/RTK Antenna
// =============================================================================

/// GPS antenna configuration
#[derive(Debug, Clone)]
pub struct GpsAntennaConfig {
    /// Antenna diameter (mm)
    pub diameter: f64,
    /// Antenna height (mm)
    pub height: f64,
    /// Base/ground plane diameter (mm)
    pub ground_plane_diameter: f64,
}

impl Default for GpsAntennaConfig {
    fn default() -> Self {
        Self {
            diameter: 60.0,
            height: 25.0,
            ground_plane_diameter: 100.0,
        }
    }
}

/// GPS/RTK antenna reference model
pub struct GpsAntenna {
    config: GpsAntennaConfig,
}

impl GpsAntenna {
    pub fn new(config: GpsAntennaConfig) -> Self {
        Self { config }
    }

    pub fn default_rtk() -> Self {
        Self::new(GpsAntennaConfig::default())
    }

    /// Generate GPS antenna geometry
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Ground plane (disk)
        let ground_plane = centered_cylinder("ground_plane", cfg.ground_plane_diameter / 2.0, 3.0, segments)
            .translate(0.0, 0.0, 1.5);

        // Antenna dome
        let antenna = centered_cylinder("antenna", cfg.diameter / 2.0, cfg.height, segments)
            .translate(0.0, 0.0, 3.0 + cfg.height / 2.0);

        ground_plane.union(&antenna)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lidar_mid360() {
        let lidar = Lidar::mid360();
        let part = lidar.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_lidar_simple() {
        let lidar = Lidar::mid360();
        let simple = lidar.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_camera_insta360() {
        let camera = Camera::insta360_x4();
        let part = camera.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_camera_simple() {
        let camera = Camera::insta360_x4();
        let simple = camera.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_gps_antenna() {
        let gps = GpsAntenna::default_rtk();
        let part = gps.generate();
        assert!(!part.is_empty());
    }
}
