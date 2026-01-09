//! UUMotor Hub Motors
//!
//! Hub motors for BVR rovers, based on supplier specifications.
//! Supplier: UUMotor (uumotor.com)
//!
//! Models:
//! - KN6104: 10" (270mm) hub motor for heavy duty applications
//! - SVB6HS: 6.5" (168mm) hub motor for ADA-compliant sidewalk rovers

use crate::{centered_cube, centered_cylinder, Part};

/// UUMotor configuration based on supplier drawings
#[derive(Debug, Clone)]
pub struct UUMotorConfig {
    /// Wheel/tire outer diameter (mm)
    pub wheel_diameter: f64,
    /// Tire width (mm)
    pub tire_width: f64,
    /// Motor hub diameter (mm)
    pub hub_diameter: f64,
    /// Motor hub width (mm)
    pub hub_width: f64,
    /// Axle diameter (mm)
    pub axle_diameter: f64,
    /// Axle length - motor side (mm)
    pub axle_length_motor: f64,
    /// Axle length - outer side (mm)
    pub axle_length_outer: f64,
    /// Mounting flange diameter (mm)
    pub flange_diameter: f64,
    /// Mounting flange thickness (mm)
    pub flange_thickness: f64,
    /// Number of mounting holes
    pub num_mount_holes: usize,
    /// Mounting hole bolt circle diameter (mm)
    pub mount_hole_pcd: f64,
    /// Mounting hole diameter (mm)
    pub mount_hole_diameter: f64,
}

impl UUMotorConfig {
    /// KN6104 with standard shaft (170mm total width)
    pub fn kn6104_standard() -> Self {
        Self {
            wheel_diameter: 270.0,      // 10x4.00-6 tire
            tire_width: 80.0,
            hub_diameter: 180.0,        // Estimated from photos
            hub_width: 114.0,           // From drawing (shorter option)
            axle_diameter: 14.0,
            axle_length_motor: 28.0,    // Cable side
            axle_length_outer: 29.0,    // Threaded mount side
            flange_diameter: 160.0,     // Estimated
            flange_thickness: 8.0,
            num_mount_holes: 5,         // Visible in photos
            mount_hole_pcd: 120.0,      // Estimated
            mount_hole_diameter: 6.0,   // M5 or M6
        }
    }

    /// KN6104 with extended shaft (199mm total width)
    pub fn kn6104_extended() -> Self {
        Self {
            wheel_diameter: 270.0,
            tire_width: 80.0,
            hub_diameter: 180.0,
            hub_width: 134.0,           // From drawing (longer option)
            axle_diameter: 14.0,
            axle_length_motor: 32.0,
            axle_length_outer: 33.0,
            flange_diameter: 160.0,
            flange_thickness: 8.0,
            num_mount_holes: 5,
            mount_hole_pcd: 120.0,
            mount_hole_diameter: 6.0,
        }
    }

    /// SVB6HS 6.5" hub motor with encoder
    /// From dimensional drawing: Ø168mm wheel, 69mm total width, M16x1.5 axle
    pub fn svb6hs() -> Self {
        Self {
            wheel_diameter: 168.0,      // Ø168 from drawing
            tire_width: 45.0,           // Estimated from 69mm total - 52mm hub
            hub_diameter: 140.0,        // Estimated from photos
            hub_width: 52.0,            // From drawing
            axle_diameter: 16.0,        // M16x1.5 from drawing
            axle_length_motor: 7.5,     // Cable side (7.5mm from drawing)
            axle_length_outer: 38.0,    // Mount side (38mm from drawing)
            flange_diameter: 130.0,     // Estimated
            flange_thickness: 8.0,
            num_mount_holes: 5,         // Visible in photos
            mount_hole_pcd: 100.0,      // Estimated
            mount_hole_diameter: 6.0,
        }
    }

    /// Total width including axles
    pub fn total_width(&self) -> f64 {
        self.hub_width + self.axle_length_motor + self.axle_length_outer
    }
}

/// UUMotor KN6104 hub motor model
pub struct UUMotor {
    config: UUMotorConfig,
}

impl UUMotor {
    pub fn new(config: UUMotorConfig) -> Self {
        Self { config }
    }

    /// KN6104 10" motor - standard shaft configuration
    pub fn kn6104() -> Self {
        Self::new(UUMotorConfig::kn6104_standard())
    }

    /// KN6104 10" motor - extended shaft configuration
    pub fn kn6104_extended() -> Self {
        Self::new(UUMotorConfig::kn6104_extended())
    }

    /// SVB6HS 6.5" motor with encoder - ADA sidewalk compliant
    pub fn svb6hs() -> Self {
        Self::new(UUMotorConfig::svb6hs())
    }

    /// Get wheel diameter
    pub fn wheel_diameter(&self) -> f64 {
        self.config.wheel_diameter
    }

    /// Get total width
    pub fn total_width(&self) -> f64 {
        self.config.total_width()
    }

    /// Get axle diameter
    pub fn axle_diameter(&self) -> f64 {
        self.config.axle_diameter
    }

    /// Get outer axle length (for mount design)
    pub fn axle_length(&self) -> f64 {
        self.config.axle_length_outer
    }

    /// Distance from wheel center to outer axle tip
    /// (hub half-width + outer axle length)
    pub fn axle_offset(&self) -> f64 {
        self.config.hub_width / 2.0 + self.config.axle_length_outer
    }

    /// Generate the motor assembly
    ///
    /// Orientation:
    /// - Wheel in XZ plane (rolls along Y)
    /// - Axle along Y axis
    /// - Motor cable side at -Y, mount side at +Y
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 48;

        // Tire (torus-like shape approximated as cylinder with rounded profile)
        let tire = self.create_tire(segments);

        // Motor hub (center cylinder)
        let hub = centered_cylinder("hub", cfg.hub_diameter / 2.0, cfg.hub_width, segments)
            .rotate(90.0, 0.0, 0.0);

        // Hub side covers (visible flanges in photos)
        let flange = centered_cylinder("flange", cfg.flange_diameter / 2.0, cfg.flange_thickness, segments);

        let flange_inner = flange
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, -cfg.hub_width / 2.0 + cfg.flange_thickness / 2.0, 0.0);

        let flange_outer = centered_cylinder("flange_outer", cfg.flange_diameter / 2.0, cfg.flange_thickness, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, cfg.hub_width / 2.0 - cfg.flange_thickness / 2.0, 0.0);

        // Axle - motor side (with cable exit area)
        let axle_motor = centered_cylinder("axle_motor", cfg.axle_diameter / 2.0, cfg.axle_length_motor, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, -cfg.hub_width / 2.0 - cfg.axle_length_motor / 2.0, 0.0);

        // Axle - outer/mount side (threaded)
        let axle_outer = centered_cylinder("axle_outer", cfg.axle_diameter / 2.0, cfg.axle_length_outer, segments)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, cfg.hub_width / 2.0 + cfg.axle_length_outer / 2.0, 0.0);

        // Axle nut (visible in photos)
        let nut_size = cfg.axle_diameter * 1.8;
        let nut = centered_cylinder("nut", nut_size / 2.0, 10.0, 6) // Hexagonal approximation
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, cfg.hub_width / 2.0 + cfg.axle_length_outer - 5.0, 0.0);

        tire
            .union(&hub)
            .union(&flange_inner)
            .union(&flange_outer)
            .union(&axle_motor)
            .union(&axle_outer)
            .union(&nut)
    }

    /// Create tire geometry
    fn create_tire(&self, segments: u32) -> Part {
        let cfg = &self.config;

        // Main tire body
        let tire_outer = centered_cylinder("tire_outer", cfg.wheel_diameter / 2.0, cfg.tire_width, segments)
            .rotate(90.0, 0.0, 0.0);

        // Inner cutout (where hub goes)
        let tire_inner = centered_cylinder("tire_inner", cfg.hub_diameter / 2.0 - 10.0, cfg.tire_width + 2.0, segments)
            .rotate(90.0, 0.0, 0.0);

        tire_outer.difference(&tire_inner)
    }

    /// Generate simplified model (for fast preview)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let segments = 24;

        // Just tire and hub as cylinders
        let tire = centered_cylinder("tire", cfg.wheel_diameter / 2.0, cfg.tire_width, segments)
            .rotate(90.0, 0.0, 0.0);

        let hub = centered_cylinder("hub", cfg.hub_diameter / 2.0, cfg.hub_width, segments)
            .rotate(90.0, 0.0, 0.0);

        tire.union(&hub)
    }
}

/// L-Bracket mount for single-axle hub motors (SVB6HS, etc.)
///
/// Single-axle motors have a fixed shaft protruding from one side only.
/// This L-bracket design properly supports the cantilever load:
///
/// ```text
/// Side view:
///
///     ════════════════════  Frame bottom rail
///           ││││││││││││    Bolts (4x M5)
///     ┌─────┴┴┴┴┴┴┴┴┴┴──┐
///     │  HORIZONTAL ARM │   (bolts to frame underside)
///     └────────┬────────┘
///              │╲
///              │ ╲  Gusset
///              │  ╲
///     ┌────────┴───╲────┐
///     │              ╲  │
///     │  VERTICAL ARM   │   (thick plate with axle boss)
///     │                 │
///     │    ┌───────┐    │
///     │    │ BOSS  ○────╫── M16 axle (nut on outside)
///     │    └───────┘    │
///     │    ═══════════  │   Torque arm slot
///     └─────────────────┘
///              │
///          [WHEEL]
/// ```
///
/// Manufacturing: 6061-T6 aluminum, CNC machined or waterjet + machining
#[derive(Debug, Clone)]
pub struct LBracketMountConfig {
    // Axle interface
    /// Axle hole diameter (motor axle + clearance)
    pub axle_hole_diameter: f64,
    /// Axle boss outer diameter (reinforcement around hole)
    pub axle_boss_diameter: f64,
    /// Axle boss thickness (how far boss protrudes)
    pub axle_boss_thickness: f64,

    // Vertical arm (supports wheel)
    /// Vertical arm width (X direction, perpendicular to axle)
    pub vertical_arm_width: f64,
    /// Vertical arm height (Z direction, from horizontal arm to bottom)
    pub vertical_arm_height: f64,
    /// Vertical arm thickness (Y direction, toward wheel)
    pub vertical_arm_thickness: f64,

    // Horizontal arm (bolts to frame)
    /// Horizontal arm length (Y direction, along frame rail)
    pub horizontal_arm_length: f64,
    /// Horizontal arm width (X direction, same as vertical arm)
    pub horizontal_arm_width: f64,
    /// Horizontal arm thickness (Z direction)
    pub horizontal_arm_thickness: f64,

    // Gusset (triangular reinforcement)
    /// Gusset thickness (Y direction)
    pub gusset_thickness: f64,

    // Frame mounting
    /// Frame bolt hole diameter (M5 = 5.5mm clearance)
    pub frame_hole_diameter: f64,
    /// Frame bolt spacing along Y (20mm for 2020)
    pub frame_hole_spacing_y: f64,
    /// Number of bolt rows along Y
    pub frame_hole_rows: usize,

    // Torque arm
    /// Torque arm slot width
    pub torque_slot_width: f64,
    /// Torque arm slot length
    pub torque_slot_length: f64,
}

impl Default for LBracketMountConfig {
    fn default() -> Self {
        Self::for_svb6hs()
    }
}

impl LBracketMountConfig {
    /// L-bracket config for SVB6HS 6.5" motor (M16 axle, 38mm protrusion)
    ///
    /// Design rationale:
    /// - 12mm vertical arm for cantilever strength
    /// - Large axle boss for stress distribution
    /// - 4 frame bolts for secure attachment
    /// - Torque slot prevents motor rotation under load
    pub fn for_svb6hs() -> Self {
        Self {
            // M16 axle with 1mm clearance
            axle_hole_diameter: 17.0,
            axle_boss_diameter: 36.0,   // 2x+ axle for strength
            axle_boss_thickness: 8.0,   // Additional meat around hole

            // Vertical arm: sized for 168mm wheel clearance
            vertical_arm_width: 70.0,
            vertical_arm_height: 90.0,   // Axle ~45mm from top
            vertical_arm_thickness: 12.0, // Thick for cantilever

            // Horizontal arm: spans 80mm for 4 bolts
            horizontal_arm_length: 80.0,
            horizontal_arm_width: 70.0,   // Match vertical arm
            horizontal_arm_thickness: 8.0,

            // Gusset for rigidity
            gusset_thickness: 8.0,

            // Frame mounting (M5 into 2020 T-slot)
            frame_hole_diameter: 5.5,
            frame_hole_spacing_y: 20.0,
            frame_hole_rows: 4,

            // Torque arm slot (for anti-rotation tab on motor)
            torque_slot_width: 6.0,
            torque_slot_length: 20.0,
        }
    }

    /// L-bracket config for KN6104 10" motor (M14 axle)
    pub fn for_kn6104() -> Self {
        Self {
            axle_hole_diameter: 15.0,
            axle_boss_diameter: 32.0,
            axle_boss_thickness: 10.0,

            vertical_arm_width: 90.0,
            vertical_arm_height: 120.0,
            vertical_arm_thickness: 15.0,  // Thicker for larger wheel

            horizontal_arm_length: 100.0,
            horizontal_arm_width: 90.0,
            horizontal_arm_thickness: 10.0,

            gusset_thickness: 10.0,

            frame_hole_diameter: 5.5,
            frame_hole_spacing_y: 20.0,
            frame_hole_rows: 5,

            torque_slot_width: 8.0,
            torque_slot_length: 25.0,
        }
    }
}

/// L-Bracket mount for single-axle hub motors
pub struct LBracketMount {
    config: LBracketMountConfig,
}

impl LBracketMount {
    pub fn new(config: LBracketMountConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(LBracketMountConfig::for_svb6hs())
    }

    pub fn for_kn6104() -> Self {
        Self::new(LBracketMountConfig::for_kn6104())
    }

    /// Vertical arm thickness
    pub fn arm_thickness(&self) -> f64 {
        self.config.vertical_arm_thickness
    }

    /// Total depth from frame to outer face of vertical arm
    pub fn total_depth(&self) -> f64 {
        self.config.vertical_arm_thickness + self.config.axle_boss_thickness
    }

    /// Vertical arm height
    pub fn arm_height(&self) -> f64 {
        self.config.vertical_arm_height
    }

    /// Horizontal arm length
    pub fn arm_length(&self) -> f64 {
        self.config.horizontal_arm_length
    }

    /// Horizontal arm thickness (Z)
    pub fn flange_thickness(&self) -> f64 {
        self.config.horizontal_arm_thickness
    }

    /// Distance from top of bracket to axle center
    pub fn axle_drop(&self) -> f64 {
        // Axle positioned 45mm from top (leaves room for gusset)
        45.0
    }

    /// Distance from frame surface to wheel center (Y direction)
    /// This is how far the wheel sticks out from the frame
    pub fn wheel_offset(&self) -> f64 {
        self.config.vertical_arm_thickness / 2.0 + self.config.axle_boss_thickness
    }

    /// Generate the L-bracket mount
    ///
    /// Orientation (default, for LEFT side of rover):
    /// - Horizontal arm in XY plane, extends in +Y (along frame rail)
    /// - Vertical arm at the FAR end of horizontal arm, extends in -X (outward) and -Z (down)
    /// - Axle hole through vertical arm, axle points in -X direction
    /// - Origin at the INNER end of horizontal arm (closest to frame center)
    ///
    /// The wheel sits INSIDE the frame, with its axle extending outward to this bracket.
    ///
    /// Top view (left side of rover):
    /// ```text
    ///                  +Y (front)
    ///                   │
    ///     ──────────────┼──────── Frame left rail (X = -frame_width/2)
    ///                   │
    ///     ┌─────────────┤ Horizontal arm (under rail)
    ///     │             │
    ///     │   ○ axle    │ Vertical arm (hangs down, outside frame)
    ///     │             │
    ///     └─────────────┘
    ///     │
    ///    -X (outward)
    /// ```
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // === Horizontal arm (bolts to frame underside) ===
        // Extends in +Y from origin, top surface at Z=0
        let h_arm = centered_cube(
            "h_arm",
            cfg.horizontal_arm_width,
            cfg.horizontal_arm_length,
            cfg.horizontal_arm_thickness,
        ).translate(0.0, cfg.horizontal_arm_length / 2.0, -cfg.horizontal_arm_thickness / 2.0);

        // === Vertical arm (extends down and outward) ===
        // At the FAR end of horizontal arm (Y = arm_length), extends in -X
        // Inner face flush with horizontal arm outer edge
        let v_arm = centered_cube(
            "v_arm",
            cfg.vertical_arm_thickness,  // Thin in X (the outward direction)
            cfg.vertical_arm_width,      // Wide in Y
            cfg.vertical_arm_height,     // Tall in Z
        ).translate(
            -cfg.vertical_arm_thickness / 2.0,  // Extends in -X from origin
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,  // At far end of h_arm
            -cfg.vertical_arm_height / 2.0,     // Extends down
        );

        // === Gusset (triangular reinforcement) ===
        let gusset_size = 35.0;
        let gusset = self.create_gusset_corner(gusset_size, cfg.gusset_thickness);
        let gusset_placed = gusset.translate(
            0.0,
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,
            -cfg.horizontal_arm_thickness,
        );

        // === Axle boss (reinforced area around axle hole) ===
        // On the OUTER face of vertical arm, axle points in -X
        let boss = centered_cylinder(
            "boss",
            cfg.axle_boss_diameter / 2.0,
            cfg.axle_boss_thickness,
            segments,
        ).rotate(0.0, 90.0, 0.0)  // Rotate so cylinder axis is along X
         .translate(
            -cfg.vertical_arm_thickness - cfg.axle_boss_thickness / 2.0,
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,
            -self.axle_drop(),
        );

        // === Combine solid body ===
        let body = h_arm
            .union(&v_arm)
            .union(&gusset_placed)
            .union(&boss);

        // === Cutouts ===

        // Axle hole (through vertical arm and boss, along X axis)
        let axle_hole_depth = cfg.vertical_arm_thickness + cfg.axle_boss_thickness + 2.0;
        let axle_hole = centered_cylinder(
            "axle_hole",
            cfg.axle_hole_diameter / 2.0,
            axle_hole_depth,
            segments,
        ).rotate(0.0, 90.0, 0.0)
         .translate(
            -axle_hole_depth / 2.0 + 1.0,
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,
            -self.axle_drop(),
        );

        // Frame bolt holes (vertical, through horizontal arm)
        let frame_holes = self.create_frame_holes(segments);

        // Torque arm slot (vertical slot below axle for anti-rotation tab)
        let torque_slot = centered_cube(
            "torque_slot",
            cfg.vertical_arm_thickness + 2.0,
            cfg.torque_slot_width,
            cfg.torque_slot_length,
        ).translate(
            -cfg.vertical_arm_thickness / 2.0,
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,
            -self.axle_drop() - 25.0,
        );

        let cutouts = axle_hole
            .union(&frame_holes)
            .union(&torque_slot);

        body.difference(&cutouts)
    }

    /// Create triangular gusset for corner (X-Z plane)
    fn create_gusset_corner(&self, size: f64, thickness: f64) -> Part {
        // Triangle in X-Z plane connecting horizontal arm to vertical arm
        let block = centered_cube("gusset_block", size, thickness, size)
            .translate(-size / 2.0, 0.0, -size / 2.0);

        let cutter = centered_cube("cutter", size * 2.0, thickness + 2.0, size * 2.0)
            .rotate(0.0, 45.0, 0.0)
            .translate(-size * 0.7, 0.0, -size * 0.7);

        block.difference(&cutter)
    }

    /// Create frame mounting holes
    fn create_frame_holes(&self, segments: u32) -> Part {
        let cfg = &self.config;
        let hole = centered_cylinder(
            "frame_hole",
            cfg.frame_hole_diameter / 2.0,
            cfg.horizontal_arm_thickness + 2.0,
            segments,
        ).translate(0.0, 0.0, -cfg.horizontal_arm_thickness / 2.0);

        let mut holes = Part::empty("frame_holes");

        // Two columns of holes (left and right of center)
        let x_offset = 20.0;

        for row in 0..cfg.frame_hole_rows {
            let y = 10.0 + (row as f64) * cfg.frame_hole_spacing_y;

            let left_hole = hole.translate(-x_offset, y, 0.0);
            let right_hole = hole.translate(x_offset, y, 0.0);

            holes = holes.union(&left_hole).union(&right_hole);
        }

        holes
    }

    /// Generate simplified mount (for preview)
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;

        // Just the L-shape without details
        let h_arm = centered_cube(
            "h_arm",
            cfg.horizontal_arm_width,
            cfg.horizontal_arm_length,
            cfg.horizontal_arm_thickness,
        ).translate(0.0, cfg.horizontal_arm_length / 2.0, -cfg.horizontal_arm_thickness / 2.0);

        let v_arm = centered_cube(
            "v_arm",
            cfg.vertical_arm_thickness,
            cfg.vertical_arm_width,
            cfg.vertical_arm_height,
        ).translate(
            -cfg.vertical_arm_thickness / 2.0,
            cfg.horizontal_arm_length - cfg.vertical_arm_width / 2.0,
            -cfg.vertical_arm_height / 2.0,
        );

        h_arm.union(&v_arm)
    }

    /// X offset from bracket origin to axle hole center
    /// (how far outward the axle is from the frame rail)
    pub fn axle_x_offset(&self) -> f64 {
        self.config.vertical_arm_thickness / 2.0
    }

    /// Y offset from bracket origin to axle hole center
    pub fn axle_y_offset(&self) -> f64 {
        self.config.horizontal_arm_length - self.config.vertical_arm_width / 2.0
    }
}

// Keep old type alias for backward compatibility during transition
pub type UUMotorMountConfig = LBracketMountConfig;
pub type UUMotorMount = LBracketMount;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uumotor_kn6104() {
        let motor = UUMotor::kn6104();
        assert_eq!(motor.wheel_diameter(), 270.0);
        assert_eq!(motor.axle_diameter(), 14.0);
    }

    #[test]
    fn test_uumotor_svb6hs() {
        let motor = UUMotor::svb6hs();
        assert_eq!(motor.wheel_diameter(), 168.0);
        assert_eq!(motor.axle_diameter(), 16.0);
        assert_eq!(motor.axle_length(), 38.0);  // Critical: single-axle protrusion
    }

    #[test]
    fn test_uumotor_dimensions() {
        let motor = UUMotor::kn6104();
        // Standard: 114 + 28 + 29 = 171mm (close to 170mm in drawing)
        assert!(motor.total_width() > 160.0 && motor.total_width() < 180.0);
    }

    #[test]
    fn test_uumotor_extended() {
        let motor = UUMotor::kn6104_extended();
        // Extended: 134 + 32 + 33 = 199mm
        assert_eq!(motor.total_width(), 199.0);
    }

    #[test]
    fn test_uumotor_generate() {
        let motor = UUMotor::kn6104();
        let part = motor.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_uumotor_simple() {
        let motor = UUMotor::kn6104();
        let part = motor.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_lbracket_mount() {
        let mount = LBracketMount::default_bvr1();
        let part = mount.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_lbracket_mount_simple() {
        let mount = LBracketMount::default_bvr1();
        let part = mount.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_lbracket_geometry() {
        let mount = LBracketMount::default_bvr1();

        // Axle drop should position wheel center below frame
        assert!(mount.axle_drop() > 0.0);

        // Wheel offset determines how far wheel sticks out
        assert!(mount.wheel_offset() > 0.0);

        // Total depth should accommodate axle + nut
        // SVB6HS has 38mm axle, need ~25mm for nut clearance
        assert!(mount.total_depth() < 38.0,
            "Mount depth ({}) must leave room for axle nut (38mm axle)",
            mount.total_depth());
    }

    #[test]
    fn test_lbracket_fits_motor() {
        let motor = UUMotor::svb6hs();

        // Axle hole must fit motor axle
        let cfg = LBracketMountConfig::for_svb6hs();
        assert!(cfg.axle_hole_diameter > motor.axle_diameter(),
            "Axle hole ({}) must be larger than axle ({})",
            cfg.axle_hole_diameter, motor.axle_diameter());
    }
}
