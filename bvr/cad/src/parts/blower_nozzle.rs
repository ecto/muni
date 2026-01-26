//! Blower Nozzle - 3D printable circular-to-slot transition
//!
//! Circular inlet (matches 90mm EDF) → rectangular slot outlet (200mm × 15mm).
//! Cross-section interpolation: circle → rounded rect → rectangle.
//!
//! Print orientation: inlet face down (circle gives great bed adhesion).
//! No internal supports needed — all overhangs are gradual.

use crate::{centered_cube, centered_cylinder, Part};
use std::f64::consts::PI;

/// Blower nozzle configuration
#[derive(Debug, Clone)]
pub struct BlowerNozzleConfig {
    /// Inlet diameter (mm) - circular opening matching EDF
    pub inlet_diameter: f64,
    /// Outlet slot width (mm)
    pub outlet_width: f64,
    /// Outlet slot height (mm)
    pub outlet_height: f64,
    /// Total nozzle body length (mm) - transition zone
    pub length: f64,
    /// Wall thickness (mm)
    pub wall_thickness: f64,
    /// Flange width around outlet (mm)
    pub flange_width: f64,
    /// Flange thickness (mm)
    pub flange_thickness: f64,
    /// Mount hole diameter (mm) - M4 clearance
    pub mount_hole_diameter: f64,
    /// Number of mount holes on flange
    pub mount_hole_count: usize,
    /// Inlet collar length (mm) - slip-fit onto EDF duct
    pub collar_length: f64,
    /// Inlet collar inner diameter (mm) - slightly larger than EDF OD for slip fit
    pub collar_id: f64,
    /// Number of set-screw bosses on inlet collar
    pub set_screw_count: usize,
    /// Set-screw tap drill diameter (mm) - M4 tap = 3.3mm
    pub set_screw_diameter: f64,
}

impl Default for BlowerNozzleConfig {
    fn default() -> Self {
        Self {
            inlet_diameter: 90.0,
            outlet_width: 200.0,
            outlet_height: 15.0,
            length: 180.0,
            wall_thickness: 3.0,
            flange_width: 15.0,
            flange_thickness: 5.0,
            mount_hole_diameter: 4.5,
            mount_hole_count: 8,
            collar_length: 30.0,
            collar_id: 91.0,
            set_screw_count: 3,
            set_screw_diameter: 3.3,
        }
    }
}

impl BlowerNozzleConfig {
    /// Inlet area (mm²) - circular
    pub fn inlet_area(&self) -> f64 {
        PI * (self.inlet_diameter / 2.0).powi(2)
    }

    /// Outlet area (mm²) - rectangular
    pub fn outlet_area(&self) -> f64 {
        self.outlet_width * self.outlet_height
    }

    /// Velocity ratio (inlet / outlet). Values > 1.0 mean converging (accelerating).
    pub fn velocity_ratio(&self) -> f64 {
        self.inlet_area() / self.outlet_area()
    }

    /// Total length including collar
    pub fn total_length(&self) -> f64 {
        self.collar_length + self.length
    }

    /// Bounding box dimensions [x, y, z] in mm (for build volume check)
    pub fn bounding_box(&self) -> [f64; 3] {
        let x = self.outlet_width + 2.0 * self.flange_width;
        let y = self.inlet_diameter + 2.0 * self.wall_thickness;
        let z = self.length;
        [x, y, z]
    }

    /// Check if the nozzle fits in a cubic build volume
    pub fn fits_build_volume(&self, max_dim: f64) -> bool {
        let bb = self.bounding_box();
        bb[0] <= max_dim && bb[1] <= max_dim && bb[2] <= max_dim
    }

    // Legacy compatibility
    pub fn inlet_size(&self) -> f64 {
        self.inlet_diameter
    }
    pub fn transition_length(&self) -> f64 {
        self.length * 0.83
    }
    pub fn inlet_length(&self) -> f64 {
        self.length * 0.17
    }
}

/// Blower nozzle: circular inlet → rectangular slot outlet
///
/// Geometry from inlet (Z=0) to outlet (Z=length):
/// - Collar extends below Z=0 (slip-fit onto EDF duct)
/// - Cross-sections interpolate from circle to rounded-rect to rectangle
/// - Reinforcement ribs near outlet
/// - Mounting flange at outlet end
pub struct BlowerNozzle {
    config: BlowerNozzleConfig,
}

impl BlowerNozzle {
    pub fn new(config: BlowerNozzleConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(BlowerNozzleConfig::default())
    }

    /// Generate the complete nozzle part
    pub fn generate(&self) -> Part {
        let cfg = &self.config;

        // Outer shell via cross-section loft
        let outer = self.generate_loft(
            cfg.inlet_diameter,
            cfg.inlet_diameter,
            cfg.outlet_width,
            cfg.outlet_height,
            cfg.length,
        );

        // Inner cavity (wall_thickness inset on all sides)
        let inner_inlet_d = cfg.inlet_diameter - 2.0 * cfg.wall_thickness;
        let inner_outlet_w = cfg.outlet_width - 2.0 * cfg.wall_thickness;
        let inner_outlet_h = (cfg.outlet_height - 2.0 * cfg.wall_thickness).max(2.0);
        let inner = self
            .generate_loft(
                inner_inlet_d,
                inner_inlet_d,
                inner_outlet_w,
                inner_outlet_h,
                cfg.length + 2.0,
            )
            .translate(0.0, 0.0, -1.0);

        let shell = outer.difference(&inner);

        // Inlet collar with set-screw bosses
        let collar = self.generate_collar();

        // Reinforcement near outlet
        let reinforce = self.generate_reinforcement();

        // Mounting flange
        let flange = self.generate_flange();

        shell
            .union(&collar)
            .union(&reinforce)
            .union(&flange)
    }

    /// Loft from inlet cross-section to outlet cross-section.
    ///
    /// Uses rounded-rectangle cross-sections that interpolate:
    /// - t=0: circle (corner_radius = diameter/2)
    /// - t=1: rectangle (corner_radius → 0)
    fn generate_loft(
        &self,
        inlet_w: f64,
        inlet_h: f64,
        outlet_w: f64,
        outlet_h: f64,
        length: f64,
    ) -> Part {
        let num_layers = 60;
        let dz = length / num_layers as f64;
        let layer_thickness = dz * 1.5; // overlap for watertight joins

        let mut solid = Part::empty("loft");

        for i in 0..=num_layers {
            let t = i as f64 / num_layers as f64;
            let z = t * length;

            let w = inlet_w + t * (outlet_w - inlet_w);
            let h = inlet_h + t * (outlet_h - inlet_h);

            // Corner radius: circle at inlet, rectangle at outlet.
            // Cubic ease-out for smoother visual transition.
            let t_ease = 1.0 - (1.0 - t).powi(3);
            let max_r = h.min(w) / 2.0;
            let corner_r = max_r * (1.0 - t_ease);

            let slice = rounded_rect_slice(w, h, corner_r, layer_thickness)
                .translate(0.0, 0.0, z);

            solid = solid.union(&slice);
        }

        solid
    }

    /// Inlet collar: cylindrical tube that slip-fits over the EDF duct.
    /// Extends below Z=0. Includes set-screw bosses for clamping.
    fn generate_collar(&self) -> Part {
        let cfg = &self.config;

        let collar_od = cfg.collar_id + 2.0 * cfg.wall_thickness;

        // Outer cylinder
        let outer = centered_cylinder("collar_outer", collar_od / 2.0, cfg.collar_length, 48)
            .translate(0.0, 0.0, -cfg.collar_length / 2.0);
        // Inner bore
        let inner =
            centered_cylinder("collar_inner", cfg.collar_id / 2.0, cfg.collar_length + 2.0, 48)
                .translate(0.0, 0.0, -cfg.collar_length / 2.0);

        let collar = outer.difference(&inner);

        // Bridge: overlap zone where collar merges with nozzle body at Z=0
        let bridge_height = 15.0;
        let bridge_cyl = centered_cylinder("bridge_cyl", collar_od / 2.0, bridge_height, 48)
            .translate(0.0, 0.0, bridge_height / 2.0);
        let bridge_sq = centered_cube(
            "bridge_sq",
            cfg.inlet_diameter,
            cfg.inlet_diameter,
            bridge_height,
        )
        .translate(0.0, 0.0, bridge_height / 2.0);
        let bridge_outer = bridge_cyl.union(&bridge_sq);
        let bridge_inner = centered_cylinder(
            "bridge_inner",
            (cfg.inlet_diameter - 2.0 * cfg.wall_thickness) / 2.0,
            bridge_height + 2.0,
            48,
        )
        .translate(0.0, 0.0, bridge_height / 2.0);
        let bridge = bridge_outer.difference(&bridge_inner);

        let collar_with_bridge = collar.union(&bridge);
        self.add_set_screw_bosses(collar_with_bridge)
    }

    /// Add set-screw bosses around the collar.
    ///
    /// Each boss is a rectangular pad on the collar exterior with a radial hole
    /// for an M4 set screw to clamp onto the EDF duct.
    fn add_set_screw_bosses(&self, collar: Part) -> Part {
        let cfg = &self.config;

        let collar_od = cfg.collar_id + 2.0 * cfg.wall_thickness;
        let boss_width = 12.0;
        let boss_height = 12.0;
        let boss_depth = 5.0;

        let r_inner = cfg.collar_id / 2.0;

        let mut result = collar;

        for i in 0..cfg.set_screw_count {
            let angle_deg = 360.0 * i as f64 / cfg.set_screw_count as f64;
            let z_center = -cfg.collar_length / 2.0;

            // Boss pad
            let boss = centered_cube(
                "boss",
                boss_depth + cfg.wall_thickness,
                boss_width,
                boss_height,
            )
            .translate(
                collar_od / 2.0 + boss_depth / 2.0 - cfg.wall_thickness,
                0.0,
                z_center,
            )
            .rotate(0.0, 0.0, angle_deg);

            result = result.union(&boss);

            // Radial set-screw hole
            let hole_depth = collar_od / 2.0 + boss_depth - r_inner + 2.0;
            let hole = centered_cylinder("set_screw", cfg.set_screw_diameter / 2.0, hole_depth, 16)
                .rotate(0.0, 90.0, 0.0)
                .translate(r_inner + hole_depth / 2.0, 0.0, z_center)
                .rotate(0.0, 0.0, angle_deg);

            result = result.difference(&hole);
        }

        result
    }

    /// Reinforcement zone near the outlet where the nozzle is thinnest.
    fn generate_reinforcement(&self) -> Part {
        let cfg = &self.config;

        let reinforce_length = 30.0;
        let reinforce_start = cfg.length - reinforce_length;
        let num_layers = 20;

        let mut reinforce = Part::empty("reinforcement");

        for i in 0..=num_layers {
            let t = i as f64 / num_layers as f64;
            let z = reinforce_start + t * reinforce_length;

            let body_t = z / cfg.length;
            let w = cfg.inlet_diameter + body_t * (cfg.outlet_width - cfg.inlet_diameter);
            let h = cfg.inlet_diameter + body_t * (cfg.outlet_height - cfg.inlet_diameter);

            let extra = t * cfg.flange_width * 0.5;
            let dz = reinforce_length / num_layers as f64 * 2.0;

            let layer = centered_cube("rib", w + extra * 2.0, h + extra * 2.0, dz)
                .translate(0.0, 0.0, z);

            reinforce = reinforce.union(&layer);
        }

        // Hollow out interior
        let inner_outlet_w = cfg.outlet_width - 2.0 * cfg.wall_thickness;
        let inner_outlet_h = (cfg.outlet_height - 2.0 * cfg.wall_thickness).max(2.0);
        let hollow = centered_cube(
            "hollow",
            inner_outlet_w,
            inner_outlet_h,
            reinforce_length + 10.0,
        )
        .translate(0.0, 0.0, reinforce_start + reinforce_length / 2.0);

        reinforce.difference(&hollow)
    }

    /// Mounting flange at the outlet end
    fn generate_flange(&self) -> Part {
        let cfg = &self.config;

        let outer_w = cfg.outlet_width + 2.0 * cfg.flange_width;
        let outer_h = cfg.outlet_height + 2.0 * cfg.flange_width;

        let flange_body = centered_cube("flange", outer_w, outer_h, cfg.flange_thickness)
            .translate(0.0, 0.0, cfg.length);

        let cutout = centered_cube(
            "cutout",
            cfg.outlet_width,
            cfg.outlet_height,
            cfg.flange_thickness + 2.0,
        )
        .translate(0.0, 0.0, cfg.length);

        let flange = flange_body.difference(&cutout);
        self.add_flange_holes(flange)
    }

    /// Add mounting holes to the flange
    fn add_flange_holes(&self, flange: Part) -> Part {
        let cfg = &self.config;

        let hole_r = cfg.mount_hole_diameter / 2.0;
        let hole_y = cfg.outlet_height / 2.0 + cfg.flange_width / 2.0;
        let num_per_side = cfg.mount_hole_count / 2;
        let spacing = cfg.outlet_width / (num_per_side as f64 + 1.0);

        let mut result = flange;
        for i in 1..=num_per_side {
            let x = -cfg.outlet_width / 2.0 + i as f64 * spacing;

            let hole_top =
                centered_cylinder("hole", hole_r, cfg.flange_thickness * 3.0, 16)
                    .translate(x, hole_y, cfg.length);
            result = result.difference(&hole_top);

            let hole_bottom =
                centered_cylinder("hole", hole_r, cfg.flange_thickness * 3.0, 16)
                    .translate(x, -hole_y, cfg.length);
            result = result.difference(&hole_bottom);
        }

        result
    }

    pub fn config(&self) -> &BlowerNozzleConfig {
        &self.config
    }
}

/// Create a rounded rectangle cross-section slice.
///
/// When corner_radius >= min(w,h)/2, the shape approaches a circle/stadium.
/// When corner_radius = 0, it's a pure rectangle.
fn rounded_rect_slice(w: f64, h: f64, corner_r: f64, thickness: f64) -> Part {
    let r = corner_r.min(w / 2.0).min(h / 2.0);

    if r < 0.5 {
        return centered_cube("slice", w, h, thickness);
    }

    // Two overlapping rectangles forming a cross
    let cross_h = centered_cube("ch", w, (h - 2.0 * r).max(0.1), thickness);
    let cross_v = centered_cube("cv", (w - 2.0 * r).max(0.1), h, thickness);
    let mut shape = cross_h.union(&cross_v);

    // Quarter-cylinder at each corner
    let cx = w / 2.0 - r;
    let cy = h / 2.0 - r;
    for &(sx, sy) in &[(1.0_f64, 1.0_f64), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        let corner = centered_cylinder("corner", r, thickness, 16)
            .translate(sx * cx, sy * cy, 0.0);
        shape = shape.union(&corner);
    }

    shape
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = BlowerNozzleConfig::default();
        assert_eq!(cfg.inlet_diameter, 90.0);
        assert_eq!(cfg.outlet_width, 200.0);
        assert_eq!(cfg.outlet_height, 15.0);
        assert_eq!(cfg.length, 180.0);
        assert_eq!(cfg.wall_thickness, 3.0);
        assert_eq!(cfg.flange_thickness, 5.0);
    }

    #[test]
    fn test_inlet_area_circular() {
        let cfg = BlowerNozzleConfig::default();
        let area = cfg.inlet_area();
        // pi * 45^2 = 6361.7 mm^2
        assert!((area - 6361.7).abs() < 1.0);
    }

    #[test]
    fn test_outlet_area() {
        let cfg = BlowerNozzleConfig::default();
        assert_eq!(cfg.outlet_area(), 3000.0);
    }

    #[test]
    fn test_velocity_ratio_converging() {
        let cfg = BlowerNozzleConfig::default();
        let ratio = cfg.velocity_ratio();
        // 6362 / 3000 = 2.12x converging
        assert!(ratio > 2.0 && ratio < 2.3, "ratio = {}", ratio);
    }

    #[test]
    fn test_fits_256mm_build_volume() {
        let cfg = BlowerNozzleConfig::default();
        assert!(cfg.fits_build_volume(256.0));
        let bb = cfg.bounding_box();
        assert!(bb[0] < 256.0, "width {} exceeds 256", bb[0]);
        assert!(bb[1] < 256.0, "depth {} exceeds 256", bb[1]);
        assert!(bb[2] < 256.0, "height {} exceeds 256", bb[2]);
    }

    #[test]
    fn test_nozzle_generates() {
        let nozzle = BlowerNozzle::default_bvr1();
        let part = nozzle.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_stl_export() {
        let nozzle = BlowerNozzle::default_bvr1();
        let part = nozzle.generate();
        assert!(part.to_stl().is_ok());
    }

    #[test]
    fn test_outlet_fits_shell_slot() {
        let cfg = BlowerNozzleConfig::default();
        // Shell slot is 502mm x 52mm
        assert!(cfg.outlet_width < 502.0);
        assert!(cfg.outlet_height < 52.0);
        let flange_w = cfg.outlet_width + 2.0 * cfg.flange_width;
        let flange_h = cfg.outlet_height + 2.0 * cfg.flange_width;
        assert!(flange_w < 502.0);
        assert!(flange_h < 52.0);
    }

    #[test]
    fn test_rounded_rect_slice_circle() {
        let slice = rounded_rect_slice(90.0, 90.0, 45.0, 2.0);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_rounded_rect_slice_rectangle() {
        let slice = rounded_rect_slice(200.0, 15.0, 0.0, 2.0);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_collar_id_larger_than_edf() {
        let cfg = BlowerNozzleConfig::default();
        assert!(cfg.collar_id > cfg.inlet_diameter);
    }

    #[test]
    fn test_total_length() {
        let cfg = BlowerNozzleConfig::default();
        assert_eq!(cfg.total_length(), 210.0); // 30mm collar + 180mm body
    }

    // Legacy compatibility
    #[test]
    fn test_inlet_size_compat() {
        let cfg = BlowerNozzleConfig::default();
        assert_eq!(cfg.inlet_size(), 90.0);
    }
}
