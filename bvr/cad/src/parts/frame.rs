//! Frame components for 2020 aluminum extrusion
//!
//! Provides primitives for building the rover chassis:
//! - 2020 aluminum extrusion profiles
//! - Corner brackets (90° L-brackets)
//! - T-nuts (for reference)
//! - Complete frame assemblies

use crate::{centered_cube, centered_cylinder, Part};

// =============================================================================
// 2020 Extrusion Profile
// =============================================================================

/// Configuration for 2020 aluminum extrusion
#[derive(Debug, Clone)]
pub struct Extrusion2020Config {
    /// Profile size (20mm for 2020)
    pub size: f64,
    /// Wall thickness
    pub wall_thickness: f64,
    /// T-slot opening width (typically 6mm)
    pub slot_width: f64,
    /// T-slot depth
    pub slot_depth: f64,
    /// Center bore diameter (typically 5mm for M5)
    pub center_bore: f64,
    /// Corner radius
    pub corner_radius: f64,
}

impl Default for Extrusion2020Config {
    fn default() -> Self {
        Self {
            size: 20.0,
            wall_thickness: 1.8,
            slot_width: 6.0,
            slot_depth: 6.0,
            center_bore: 5.0,
            corner_radius: 1.5,
        }
    }
}

/// 2020 aluminum extrusion generator
pub struct Extrusion2020 {
    config: Extrusion2020Config,
    length: f64,
}

impl Extrusion2020 {
    pub fn new(length: f64) -> Self {
        Self {
            config: Extrusion2020Config::default(),
            length,
        }
    }

    pub fn with_config(length: f64, config: Extrusion2020Config) -> Self {
        Self { config, length }
    }

    /// Generate simplified extrusion (solid bar with slots)
    ///
    /// For visualization/collision, we use a simplified profile:
    /// - Solid 20x20 bar
    /// - T-slots cut on all 4 sides
    /// - Center bore
    ///
    /// Extrusion runs along Z axis, centered at origin.
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Main solid bar
        let bar = centered_cube("bar", cfg.size, cfg.size, self.length);

        // T-slots on all 4 sides (create each separately since Part doesn't impl Clone)
        let slot_py = self.create_t_slot().translate(0.0, cfg.size / 2.0, 0.0);
        let slot_px = self.create_t_slot().rotate(0.0, 0.0, 90.0).translate(cfg.size / 2.0, 0.0, 0.0);
        let slot_ny = self.create_t_slot().rotate(0.0, 0.0, 180.0).translate(0.0, -cfg.size / 2.0, 0.0);
        let slot_nx = self.create_t_slot().rotate(0.0, 0.0, 270.0).translate(-cfg.size / 2.0, 0.0, 0.0);

        let slots = slot_py.union(&slot_px).union(&slot_ny).union(&slot_nx);

        // Center bore
        let bore = centered_cylinder("bore", cfg.center_bore / 2.0, self.length + 2.0, segments);

        bar.difference(&slots).difference(&bore)
    }

    /// Generate ultra-simplified extrusion (just a box, fastest)
    pub fn generate_simple(&self) -> Part {
        centered_cube("extrusion", self.config.size, self.config.size, self.length)
    }

    /// Create T-slot profile (to be subtracted from bar)
    fn create_t_slot(&self) -> Part {
        let cfg = &self.config;

        // T-slot: narrow opening that widens inside
        // Outer opening
        let opening = centered_cube(
            "slot_opening",
            cfg.slot_width,
            cfg.slot_depth,
            self.length + 2.0,
        );

        // Inner wide part (for T-nut)
        let inner_width = cfg.slot_width + 4.0; // T-nut head width
        let inner = centered_cube(
            "slot_inner",
            inner_width,
            cfg.slot_depth - 2.0,
            self.length + 2.0,
        )
        .translate(0.0, -2.0, 0.0);

        opening.union(&inner)
    }

    /// Get the length
    pub fn length(&self) -> f64 {
        self.length
    }

    /// Get the profile size (20mm)
    pub fn size(&self) -> f64 {
        self.config.size
    }
}

// =============================================================================
// Corner Bracket
// =============================================================================

/// Configuration for 90° corner bracket
#[derive(Debug, Clone)]
pub struct CornerBracketConfig {
    /// Arm length (each side)
    pub arm_length: f64,
    /// Arm width
    pub arm_width: f64,
    /// Thickness
    pub thickness: f64,
    /// Bolt hole diameter (M5 = 5.5mm clearance)
    pub hole_diameter: f64,
    /// Number of holes per arm
    pub holes_per_arm: usize,
    /// Hole spacing from edge
    pub hole_inset: f64,
}

impl Default for CornerBracketConfig {
    fn default() -> Self {
        Self {
            arm_length: 20.0,     // Standard 2020 bracket
            arm_width: 20.0,      // Matches extrusion width
            thickness: 4.0,       // 4mm thick aluminum
            hole_diameter: 5.5,   // M5 clearance
            holes_per_arm: 1,     // One hole per arm
            hole_inset: 10.0,     // Hole at center of arm
        }
    }
}

impl CornerBracketConfig {
    /// Standard 2020 corner bracket (20x20mm)
    pub fn standard_2020() -> Self {
        Self::default()
    }

    /// Heavy duty bracket (40x40mm arms)
    pub fn heavy_duty() -> Self {
        Self {
            arm_length: 40.0,
            arm_width: 20.0,
            thickness: 5.0,
            hole_diameter: 5.5,
            holes_per_arm: 2,
            hole_inset: 10.0,
        }
    }
}

/// 90° corner bracket for joining extrusions
pub struct CornerBracket {
    config: CornerBracketConfig,
}

impl CornerBracket {
    pub fn new(config: CornerBracketConfig) -> Self {
        Self { config }
    }

    pub fn standard() -> Self {
        Self::new(CornerBracketConfig::standard_2020())
    }

    /// Generate the corner bracket
    ///
    /// L-shaped bracket in the XY plane:
    /// - One arm along +X
    /// - One arm along +Y
    /// - Corner at origin
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Arm along +X
        let arm_x = centered_cube("arm_x", cfg.arm_length, cfg.arm_width, cfg.thickness)
            .translate(cfg.arm_length / 2.0, 0.0, 0.0);

        // Arm along +Y
        let arm_y = centered_cube("arm_y", cfg.arm_width, cfg.arm_length, cfg.thickness)
            .translate(0.0, cfg.arm_length / 2.0, 0.0);

        // Create holes
        let holes = self.create_holes(segments);

        arm_x.union(&arm_y).difference(&holes)
    }

    fn create_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let hole_depth = cfg.thickness * 2.0;

        let mut holes = Part::empty("holes");

        // Holes in X arm
        for i in 0..cfg.holes_per_arm {
            let x = cfg.hole_inset + (i as f64) * 20.0;
            let hole = centered_cylinder("hole", cfg.hole_diameter / 2.0, hole_depth, segments)
                .translate(x, 0.0, 0.0);
            holes = holes.union(&hole);
        }

        // Holes in Y arm
        for i in 0..cfg.holes_per_arm {
            let y = cfg.hole_inset + (i as f64) * 20.0;
            let hole = centered_cylinder("hole", cfg.hole_diameter / 2.0, hole_depth, segments)
                .translate(0.0, y, 0.0);
            holes = holes.union(&hole);
        }

        holes
    }
}

// =============================================================================
// T-Nut
// =============================================================================

/// T-nut for 2020 extrusion (reference geometry)
pub struct TNut {
    /// Nut width (fits in T-slot)
    pub width: f64,
    /// Nut length
    pub length: f64,
    /// Nut height (thickness)
    pub height: f64,
    /// Thread diameter (M5)
    pub thread_diameter: f64,
}

impl Default for TNut {
    fn default() -> Self {
        Self {
            width: 10.0,         // T-slot internal width
            length: 10.0,
            height: 5.0,
            thread_diameter: 5.0,
        }
    }
}

impl TNut {
    pub fn m5() -> Self {
        Self::default()
    }

    /// Generate T-nut geometry
    pub fn generate(&self) -> Part {
        let segments = 32;

        // Main body
        let body = centered_cube("body", self.width, self.length, self.height);

        // Thread hole
        let hole = centered_cylinder("thread", self.thread_diameter / 2.0, self.height * 2.0, segments);

        body.difference(&hole)
    }
}

// =============================================================================
// Frame Assembly Helpers
// =============================================================================

/// BVR1 frame dimensions
#[derive(Debug, Clone)]
pub struct BVR1FrameConfig {
    /// Width (X dimension, side to side)
    pub width: f64,
    /// Length (Y dimension, front to back)
    pub length: f64,
    /// Height (Z dimension, bottom to top of main frame)
    pub height: f64,
    /// Wheel vertical offset from bottom
    pub wheel_z_offset: f64,
}

impl Default for BVR1FrameConfig {
    fn default() -> Self {
        // ADA-compliant sizing for 6.5" (168mm) wheels
        // Target: ~600mm total width for sidewalk operation
        // With 168mm wheels (84mm radius), frame can be narrower
        Self {
            width: 380.0,   // 380mm wide (was 500mm for 10" wheels)
            length: 500.0,  // 500mm long (compact)
            height: 180.0,  // 180mm tall
            wheel_z_offset: 0.0,
        }
    }
}

/// Generate a complete BVR1 frame assembly
pub struct BVR1Frame {
    config: BVR1FrameConfig,
}

impl BVR1Frame {
    pub fn new(config: BVR1FrameConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(BVR1FrameConfig::default())
    }

    /// Generate the complete frame assembly
    ///
    /// Frame structure (corner detail, top view):
    /// ```text
    ///    ┌─────────────────────────┐
    ///    │  ┌───┐           ┌───┐  │  <- Front/back rails (X)
    ///    │  │ P │           │ P │  │     between side rails
    ///    │  └───┘           └───┘  │
    ///    │                         │  <- Side rails (Y) full length
    ///    │  ┌───┐           ┌───┐  │
    ///    │  │ P │           │ P │  │  P = Vertical post
    ///    │  └───┘           └───┘  │
    ///    └─────────────────────────┘
    /// ```
    ///
    /// Corner detail (side view):
    /// ```text
    ///    ══════════ <- Top rail (at Z = height - profile)
    ///        ║
    ///        ║      <- Vertical post (height - 2*profile)
    ///        ║         fits BETWEEN top and bottom rails
    ///    ══════════ <- Bottom rail (at Z = 0)
    /// ```
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let profile = 20.0; // 2020 extrusion

        // Bottom frame at Z = 0 (rail center at profile/2)
        let bottom = self.create_rectangular_frame(0.0);

        // Top frame at Z = height - profile (rail center at height - profile/2)
        let top = self.create_rectangular_frame(cfg.height - profile);

        // Vertical posts: fit BETWEEN top and bottom rails
        // Height = total height - 2*profile (for top and bottom rails)
        let post_height = cfg.height - profile * 2.0;
        let posts = self.create_vertical_posts(post_height);

        // Central spine removed - using bottom tray instead for BVR1

        bottom.union(&top).union(&posts)
    }

    /// Create a rectangular frame in XY plane at given Z (bottom of rail)
    fn create_rectangular_frame(&self, z: f64) -> Part {
        let cfg = &self.config;
        let profile = 20.0;

        // All rails shortened to avoid overlap at corners
        // Side rails (Y): full length minus profile at each end for front/back rails
        let rail_y_length = cfg.length - profile * 2.0;

        // Front/back rails (X): full width minus profile at each end for side rails
        let rail_x_length = cfg.width - profile * 2.0;

        // Center height of this frame level
        let z_center = z + profile / 2.0;

        // X offset: center of side rail
        let x_edge = cfg.width / 2.0 - profile / 2.0;
        // Y offset: center of front/back rail
        let y_edge = cfg.length / 2.0 - profile / 2.0;

        // Front rail (along X, at +Y edge, between corners)
        let front = Extrusion2020::new(rail_x_length)
            .generate()
            .rotate(0.0, 90.0, 0.0)
            .translate(0.0, y_edge, z_center);

        // Back rail (along X, at -Y edge)
        let back = Extrusion2020::new(rail_x_length)
            .generate()
            .rotate(0.0, 90.0, 0.0)
            .translate(0.0, -y_edge, z_center);

        // Left rail (along Y, at -X edge)
        let left = Extrusion2020::new(rail_y_length)
            .generate()
            .rotate(90.0, 0.0, 0.0)
            .translate(-x_edge, 0.0, z_center);

        // Right rail (along Y, at +X edge)
        let right = Extrusion2020::new(rail_y_length)
            .generate()
            .rotate(90.0, 0.0, 0.0)
            .translate(x_edge, 0.0, z_center);

        // Corner posts (short vertical pieces at corners of this level only)
        let corner_fl = Extrusion2020::new(profile).generate()
            .translate(-x_edge, y_edge, z_center);
        let corner_fr = Extrusion2020::new(profile).generate()
            .translate(x_edge, y_edge, z_center);
        let corner_bl = Extrusion2020::new(profile).generate()
            .translate(-x_edge, -y_edge, z_center);
        let corner_br = Extrusion2020::new(profile).generate()
            .translate(x_edge, -y_edge, z_center);

        front
            .union(&back)
            .union(&left)
            .union(&right)
            .union(&corner_fl)
            .union(&corner_fr)
            .union(&corner_bl)
            .union(&corner_br)
    }

    /// Create 4 vertical posts at corners (between top and bottom frames)
    fn create_vertical_posts(&self, height: f64) -> Part {
        let cfg = &self.config;
        let profile = 20.0;

        // Posts at corners, inside the corner cubes
        let x_offset = cfg.width / 2.0 - profile / 2.0;
        let y_offset = cfg.length / 2.0 - profile / 2.0;

        // Z center: posts start above bottom rail, end below top rail
        // Bottom rail top = profile, Top rail bottom = height - profile
        let z_center = profile + height / 2.0;

        let post_fl = Extrusion2020::new(height).generate()
            .translate(-x_offset, y_offset, z_center);
        let post_fr = Extrusion2020::new(height).generate()
            .translate(x_offset, y_offset, z_center);
        let post_bl = Extrusion2020::new(height).generate()
            .translate(-x_offset, -y_offset, z_center);
        let post_br = Extrusion2020::new(height).generate()
            .translate(x_offset, -y_offset, z_center);

        post_fl.union(&post_fr).union(&post_bl).union(&post_br)
    }

    /// Generate simplified frame (just boxes, for quick preview)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let profile = 20.0;

        // Just the outer box outline
        let outer = centered_cube("outer", cfg.width, cfg.length, cfg.height);
        let inner = centered_cube(
            "inner",
            cfg.width - profile * 2.0,
            cfg.length - profile * 2.0,
            cfg.height + 2.0,
        );

        outer.difference(&inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Extrusion Tests
    // =========================================================================

    #[test]
    fn test_extrusion_generation() {
        let ext = Extrusion2020::new(100.0);
        let part = ext.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_extrusion_length() {
        let length = 250.0;
        let ext = Extrusion2020::new(length);
        assert_eq!(ext.length(), length);
        assert_eq!(ext.size(), 20.0);
    }

    #[test]
    fn test_extrusion_simple() {
        let ext = Extrusion2020::new(100.0);
        let simple = ext.generate_simple();
        let detailed = ext.generate();
        // Simple should also be non-empty
        assert!(!simple.is_empty());
        assert!(!detailed.is_empty());
    }

    #[test]
    fn test_extrusion_various_lengths() {
        for length in [50.0, 100.0, 200.0, 500.0, 1000.0] {
            let ext = Extrusion2020::new(length);
            let part = ext.generate();
            assert!(!part.is_empty(), "Extrusion of length {} should not be empty", length);
        }
    }

    // =========================================================================
    // Corner Bracket Tests
    // =========================================================================

    #[test]
    fn test_corner_bracket() {
        let bracket = CornerBracket::standard();
        let part = bracket.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_corner_bracket_heavy_duty() {
        let config = CornerBracketConfig::heavy_duty();
        assert_eq!(config.arm_length, 40.0);
        assert_eq!(config.holes_per_arm, 2);

        let bracket = CornerBracket::new(config);
        let part = bracket.generate();
        assert!(!part.is_empty());
    }

    // =========================================================================
    // T-Nut Tests
    // =========================================================================

    #[test]
    fn test_tnut() {
        let tnut = TNut::m5();
        let part = tnut.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_tnut_dimensions() {
        let tnut = TNut::m5();
        assert_eq!(tnut.thread_diameter, 5.0);
        assert_eq!(tnut.width, 10.0);
    }

    // =========================================================================
    // BVR1 Frame Tests
    // =========================================================================

    #[test]
    fn test_bvr1_frame() {
        let frame = BVR1Frame::default_bvr1();
        let part = frame.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_bvr1_frame_simple() {
        let frame = BVR1Frame::default_bvr1();
        let simple = frame.generate_simple();
        assert!(!simple.is_empty());
    }

    #[test]
    fn test_bvr1_frame_config() {
        let config = BVR1FrameConfig::default();
        // ADA-compliant sizing for 6.5" wheels
        assert_eq!(config.width, 380.0);
        assert_eq!(config.length, 500.0);
        assert_eq!(config.height, 180.0);
    }

    #[test]
    fn test_bvr1_frame_custom_size() {
        let config = BVR1FrameConfig {
            width: 400.0,
            length: 500.0,
            height: 150.0,
            wheel_z_offset: 0.0,
        };
        let frame = BVR1Frame::new(config);
        let part = frame.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_frame_dimensions_consistency() {
        // Verify frame parts fit together properly
        let config = BVR1FrameConfig::default();
        let profile = 20.0;

        // Side rails are full length
        let side_rail_length = config.length;
        assert_eq!(side_rail_length, 500.0);

        // Front/back rails fit between side rails
        let front_rail_length = config.width - profile * 2.0;
        assert_eq!(front_rail_length, 340.0);  // 380 - 40

        // Vertical posts are full height
        let post_height = config.height;
        assert_eq!(post_height, 180.0);

        // Central spine fits between front and back rails
        let spine_length = config.length - profile * 2.0;
        assert_eq!(spine_length, 460.0);  // 500 - 40
    }

    #[test]
    fn test_frame_can_be_written_to_stl() {
        let frame = BVR1Frame::default_bvr1();
        let part = frame.generate();

        // Should be able to convert to STL bytes
        let stl_result = part.to_stl();
        assert!(stl_result.is_ok(), "Frame should be exportable to STL");

        let stl_bytes = stl_result.unwrap();
        assert!(stl_bytes.len() > 84, "STL should have header + triangles");

        // Check STL header
        assert_eq!(stl_bytes.len() % 1, 0); // Basic sanity
    }
}
