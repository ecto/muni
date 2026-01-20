//! Sensor reference geometry
//!
//! Simplified models of sensors for visualization and assembly.
//! Not for manufacturing - these are reference parts.

use crate::{centered_cube, centered_cylinder, Part};

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
    ///
    /// Actual dimensions from spec sheet:
    /// - Body: 65 × 65 × 60mm
    /// - Hemispherical window: 45 × 25mm (visible portion)
    /// - Weight: 265g
    /// - 4× M3 mounting holes on bottom (5mm depth)
    pub fn livox_mid360() -> Self {
        Self {
            diameter: 65.0,      // 65mm square, using diameter for cylindrical approx
            height: 60.0,        // 60mm tall
            base_diameter: 65.0, // Same as body (no separate base plate)
            base_thickness: 5.0, // Mounting surface thickness
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
    /// Livox Mid-360 representation:
    /// - 65×65×60mm total dimensions
    /// - Rectangular base with hemispherical scanning window
    /// - Window is recessed into top, not added on top
    ///
    /// Orientation: scanning window faces up (+Z), mounts to surface at Z=0
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Main body - rectangular base (65×65mm)
        // The body is about 48mm tall, with dome taking remaining height
        let body_height = cfg.height - 12.0; // Leave room for dome (48mm)
        let body = centered_cube("body", cfg.diameter, cfg.diameter, body_height)
            .translate(0.0, 0.0, body_height / 2.0);

        // Scanning dome - sits in a recess on top
        // The dome protrudes ~12mm above the body
        // Actual window is 45mm diameter hemisphere
        let dome_radius = 22.5; // 45mm diameter / 2
        let dome_height = 12.0;
        let dome = centered_cylinder("dome", dome_radius, dome_height, segments)
            .translate(0.0, 0.0, body_height + dome_height / 2.0);

        // Total height: 48 + 12 = 60mm ✓

        body.union(&dome)
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
// GPS/RTK Antenna (standalone)
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

// =============================================================================
// Proxicast 5-in-1 Combo Antenna
// =============================================================================

/// Proxicast 5-in-1 antenna configuration
///
/// Model: ANT-500-221 or similar
/// Combines: 2x LTE/cellular, 2x WiFi, 1x GPS
/// Mounts flush through lid with threaded stud
#[derive(Debug, Clone)]
pub struct ProxicastAntennaConfig {
    /// Puck diameter (mm)
    pub diameter: f64,
    /// Puck height above mounting surface (mm)
    pub height: f64,
    /// Mounting stud diameter (mm) - goes through the lid hole
    pub stud_diameter: f64,
    /// Mounting stud length below surface (mm)
    pub stud_length: f64,
}

impl Default for ProxicastAntennaConfig {
    fn default() -> Self {
        // Based on Proxicast ANT-500-221 specs
        Self {
            diameter: 88.0,    // ~3.5" diameter puck
            height: 18.0,      // Squat profile, ~0.7" tall
            stud_diameter: 25.0, // Threaded mounting stud
            stud_length: 15.0,   // Length below surface for nut
        }
    }
}

/// Proxicast 5-in-1 combo antenna reference model
///
/// A "puck" style antenna that provides:
/// - 2x LTE/4G cellular (MIMO)
/// - 2x WiFi 2.4/5GHz (MIMO)
/// - 1x GPS/GLONASS
///
/// Mounts flush on the lid with a threaded stud through a 32mm hole.
/// Needs clear sky view for GPS and good cellular/WiFi coverage.
pub struct ProxicastAntenna {
    config: ProxicastAntennaConfig,
}

impl ProxicastAntenna {
    pub fn new(config: ProxicastAntennaConfig) -> Self {
        Self { config }
    }

    pub fn default_5in1() -> Self {
        Self::new(ProxicastAntennaConfig::default())
    }

    /// Generate antenna geometry
    ///
    /// Orientation: puck sits on XY plane, stud extends down (-Z)
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Main puck body (squat cylinder with slight dome effect)
        let puck = centered_cylinder("puck", cfg.diameter / 2.0, cfg.height, segments)
            .translate(0.0, 0.0, cfg.height / 2.0);

        // Slight raised center (where cables meet internally)
        let center_bump = centered_cylinder("center", cfg.diameter / 6.0, cfg.height * 0.3, segments)
            .translate(0.0, 0.0, cfg.height + cfg.height * 0.15);

        // Mounting stud (extends below the puck)
        let stud = centered_cylinder("stud", cfg.stud_diameter / 2.0, cfg.stud_length, segments)
            .translate(0.0, 0.0, -cfg.stud_length / 2.0);

        puck.union(&center_bump).union(&stud)
    }

    /// Generate simplified geometry (just the puck)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cylinder("proxicast", cfg.diameter / 2.0, cfg.height, 32)
            .translate(0.0, 0.0, cfg.height / 2.0)
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

    #[test]
    fn test_proxicast_antenna() {
        let antenna = ProxicastAntenna::default_5in1();
        let part = antenna.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_proxicast_antenna_simple() {
        let antenna = ProxicastAntenna::default_5in1();
        let simple = antenna.generate_simple();
        assert!(!simple.is_empty());
    }
}
