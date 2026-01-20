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
    /// KN6104 with single shaft - bicycle-style dropout mount
    ///
    /// From Tony @ UUMotor (Jan 2026):
    /// - M16×1.5 threaded axle (not M14!)
    /// - Single shaft extends from motor body
    /// - Mounts like a bicycle wheel in dropout forks
    /// - Disc brake compatible
    pub fn kn6104_single_shaft() -> Self {
        Self {
            wheel_diameter: 270.0,      // 10×4.00-6 tire
            tire_width: 100.0,          // Wide pneumatic tire
            hub_diameter: 180.0,        // Motor housing
            hub_width: 85.0,            // Motor body width (without axle)
            axle_diameter: 16.0,        // M16×1.5 from Tony's email
            axle_length_motor: 0.0,     // Single shaft - no cable side protrusion
            axle_length_outer: 45.0,    // Mount side - enough for fork + nut
            flange_diameter: 160.0,
            flange_thickness: 8.0,
            num_mount_holes: 5,
            mount_hole_pcd: 120.0,
            mount_hole_diameter: 6.0,
        }
    }

    /// KN6104 standard (legacy dual-shaft config)
    pub fn kn6104_standard() -> Self {
        Self {
            wheel_diameter: 270.0,
            tire_width: 80.0,
            hub_diameter: 180.0,
            hub_width: 114.0,
            axle_diameter: 16.0,        // M16×1.5
            axle_length_motor: 28.0,
            axle_length_outer: 29.0,
            flange_diameter: 160.0,
            flange_thickness: 8.0,
            num_mount_holes: 5,
            mount_hole_pcd: 120.0,
            mount_hole_diameter: 6.0,
        }
    }

    /// KN6104 with extended shaft (199mm total width)
    pub fn kn6104_extended() -> Self {
        Self {
            wheel_diameter: 270.0,
            tire_width: 80.0,
            hub_diameter: 180.0,
            hub_width: 134.0,
            axle_diameter: 16.0,        // M16×1.5
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

    /// KN6104 10" motor - single shaft for dropout fork mounting
    ///
    /// This is the configuration from Tony @ UUMotor (Jan 2026)
    pub fn kn6104() -> Self {
        Self::new(UUMotorConfig::kn6104_single_shaft())
    }

    /// KN6104 10" motor - extended shaft configuration (legacy)
    pub fn kn6104_extended() -> Self {
        Self::new(UUMotorConfig::kn6104_extended())
    }

    /// SVB6HS 6.5" motor with encoder - ADA sidewalk compliant
    pub fn svb6hs() -> Self {
        Self::new(UUMotorConfig::svb6hs())
    }

    /// Get the motor configuration
    pub fn config(&self) -> &UUMotorConfig {
        &self.config
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

        // Axle - motor/cable side (only if present - single shaft motors have 0 length)
        let mut result = tire
            .union(&hub)
            .union(&flange_inner)
            .union(&flange_outer);

        if cfg.axle_length_motor > 0.0 {
            let axle_motor = centered_cylinder("axle_motor", cfg.axle_diameter / 2.0, cfg.axle_length_motor, segments)
                .rotate(90.0, 0.0, 0.0)
                .translate(0.0, -cfg.hub_width / 2.0 - cfg.axle_length_motor / 2.0, 0.0);
            result = result.union(&axle_motor);
        }

        // Axle - outer/mount side (threaded) - always present
        if cfg.axle_length_outer > 0.0 {
            let axle_outer = centered_cylinder("axle_outer", cfg.axle_diameter / 2.0, cfg.axle_length_outer, segments)
                .rotate(90.0, 0.0, 0.0)
                .translate(0.0, cfg.hub_width / 2.0 + cfg.axle_length_outer / 2.0, 0.0);

            // Axle nut (visible in photos)
            let nut_size = cfg.axle_diameter * 1.8;
            let nut = centered_cylinder("nut", nut_size / 2.0, 10.0, 6) // Hexagonal approximation
                .rotate(90.0, 0.0, 0.0)
                .translate(0.0, cfg.hub_width / 2.0 + cfg.axle_length_outer - 5.0, 0.0);

            result = result.union(&axle_outer).union(&nut);
        }

        result
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

    /// L-bracket config for KN6104 10" motor (M16×1.5 axle)
    ///
    /// Dropout fork style mount - axle sits in slot, secured with nut
    /// Based on Tony @ UUMotor specs (Jan 2026)
    pub fn for_kn6104() -> Self {
        Self {
            // M16 axle with 1mm clearance
            axle_hole_diameter: 17.0,
            axle_boss_diameter: 40.0,    // Larger boss for 10" wheel loads
            axle_boss_thickness: 12.0,

            // Vertical arm: sized for 270mm wheel clearance
            // Wheel radius = 135mm, need axle ~140mm below frame
            vertical_arm_width: 100.0,
            vertical_arm_height: 160.0,  // Taller for larger wheel
            vertical_arm_thickness: 15.0,

            // Horizontal arm: spans 100mm for 5 bolts
            horizontal_arm_length: 100.0,
            horizontal_arm_width: 100.0,
            horizontal_arm_thickness: 10.0,

            // Gusset for rigidity
            gusset_thickness: 10.0,

            // Frame mounting (M5 into 2020 T-slot)
            frame_hole_diameter: 5.5,
            frame_hole_spacing_y: 20.0,
            frame_hole_rows: 5,

            // Torque arm slot (prevents motor rotation)
            torque_slot_width: 8.0,
            torque_slot_length: 30.0,
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

    /// Default mount for BVR1 - KN6104 10" motors
    pub fn default_bvr1() -> Self {
        Self::new(LBracketMountConfig::for_kn6104())
    }

    /// Mount for SVB6HS 6.5" motors (ADA compact variant)
    pub fn for_svb6hs() -> Self {
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
    ///
    /// Axle positioned near bottom of vertical arm, leaving room
    /// for gusset at top and wheel clearance below
    pub fn axle_drop(&self) -> f64 {
        // Axle at ~70% down the vertical arm
        // For KN6104: 160mm × 0.7 = 112mm from top
        // For SVB6HS: 90mm × 0.7 = 63mm from top
        self.config.vertical_arm_height * 0.7
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

// =============================================================================
// Single-sided dropout mount (torque arm plate for single-shaft motor)
// =============================================================================

#[derive(Debug, Clone)]
pub struct SingleDropoutMountConfig {
    pub axle_diameter: f64,
    pub slot_width: f64,
    pub slot_depth: f64,
    pub plate_thickness: f64,
    pub plate_width: f64,
    pub plate_height: f64,
    pub arm_length: f64,
    pub pinch_hole_diameter: f64,
    pub pinch_hole_offset_z: f64,
    pub frame_hole_diameter: f64,
    pub frame_hole_spacing: f64,
    pub frame_hole_count: usize,
    pub brace_thickness: f64,
    pub brace_height: f64,
}

impl SingleDropoutMountConfig {
    pub fn for_kn6104() -> Self {
        Self {
            axle_diameter: 16.0,
            slot_width: 18.0,   // 1-2mm clearance on M16 axle
            slot_depth: 26.0,   // Deep enough for nut engagement
            plate_thickness: 12.0,
            plate_width: 90.0,
            plate_height: 170.0,
            arm_length: 90.0,   // distance from frame edge to plate center
            pinch_hole_diameter: 0.0,   // no pinch by default
            pinch_hole_offset_z: 0.0,   // disabled
            frame_hole_diameter: 5.5, // M5 clearance
            frame_hole_spacing: 30.0,
            frame_hole_count: 4,
            brace_thickness: 0.0, // keep simple (no gusset by default)
            brace_height: 0.0,
        }
    }
}

pub struct SingleDropoutMount {
    config: SingleDropoutMountConfig,
}

impl SingleDropoutMount {
    pub fn new(config: SingleDropoutMountConfig) -> Self {
        Self { config }
    }

    pub fn for_kn6104() -> Self {
        Self::new(SingleDropoutMountConfig::for_kn6104())
    }

    pub fn config(&self) -> &SingleDropoutMountConfig {
        &self.config
    }

    /// Distance from plate top (frame interface) to axle center
    pub fn axle_drop(&self) -> f64 {
        self.config.plate_height - self.config.slot_depth / 2.0
    }

    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let segments = 32;

        // Plate: thickness along X, width along Y, height along Z (downward), centered at origin (axle center at X=0)
        let plate = centered_cube(
            "dropout_plate",
            cfg.plate_thickness,
            cfg.plate_width,
            cfg.plate_height,
        ).translate(0.0, 0.0, -cfg.plate_height / 2.0);

        // Slot: open bottom dropout
        let slot = centered_cube(
            "slot",
            cfg.slot_width,
            cfg.plate_width + 2.0,
            cfg.slot_depth,
        ).translate(
            0.0,
            0.0,
            -cfg.plate_height + cfg.slot_depth / 2.0,
        );

        // Bottom opening continuation (keeps slot open to bottom)
        let bottom_open = centered_cube(
            "slot_open",
            cfg.slot_width,
            cfg.plate_width + 2.0,
            cfg.slot_depth,
        ).translate(
            0.0,
            0.0,
            -cfg.plate_height - cfg.slot_depth / 2.0,
        );

        // Frame holes near top of plate
        let mut holes = Part::empty("holes");
        let hole = centered_cylinder(
            "frame_hole",
            cfg.frame_hole_diameter / 2.0,
            cfg.plate_thickness + 2.0,
            segments,
        ).rotate(90.0, 0.0, 0.0);

        let start_z = -cfg.frame_hole_spacing * ((cfg.frame_hole_count as f64 - 1.0) / 2.0);
        for i in 0..cfg.frame_hole_count {
            let z = start_z + i as f64 * cfg.frame_hole_spacing;
            let h = hole.translate(0.0, 0.0, z);
            holes = holes.union(&h);
        }

        // Pinch/clamp bolt through plate (along X)
        let pinch = if cfg.pinch_hole_diameter > 0.0 && cfg.pinch_hole_offset_z > 0.0 {
            let pinch_hole = centered_cylinder(
                "pinch_hole",
                cfg.pinch_hole_diameter / 2.0,
                cfg.plate_thickness + 2.0,
                segments,
            ).rotate(0.0, 90.0, 0.0)
             .translate(0.0, 0.0, -cfg.plate_height + cfg.pinch_hole_offset_z);
            pinch_hole
        } else {
            Part::empty("no_pinch")
        };

        // Top arm connecting frame edge to plate (same thickness as plate), extending from frame (negative X) to plate at X=0
        let arm = centered_cube(
            "arm",
            cfg.arm_length,
            cfg.plate_width,
            cfg.plate_thickness,
        ).translate(
            -cfg.arm_length / 2.0,
            0.0,
            -cfg.plate_thickness / 2.0,
        );

        arm.union(&plate)
            .difference(&slot.union(&bottom_open).union(&holes).union(&pinch))
    }

    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        centered_cube(
            "dropout_plate_simple",
            cfg.plate_thickness,
            cfg.plate_width,
            cfg.plate_height,
        ).translate(cfg.arm_length, 0.0, -cfg.plate_height / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uumotor_kn6104() {
        // KN6104 single-shaft config from Tony @ UUMotor (Jan 2026)
        let motor = UUMotor::kn6104();
        assert_eq!(motor.wheel_diameter(), 270.0);
        assert_eq!(motor.axle_diameter(), 16.0);  // M16×1.5
        assert_eq!(motor.config().axle_length_motor, 0.0);  // Single shaft - no cable side
        assert_eq!(motor.axle_length(), 45.0);  // Mount side axle
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
        // KN6104 single-shaft: 85mm hub + 0mm motor axle + 45mm outer axle = 130mm
        let motor = UUMotor::kn6104();
        assert_eq!(motor.total_width(), 130.0);
    }

    #[test]
    fn test_uumotor_extended() {
        let motor = UUMotor::kn6104_extended();
        // Extended dual-shaft: 134 + 32 + 33 = 199mm
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

    // Dropout mount tests
    #[test]
    fn test_dropout_generate() {
        let mount = SingleDropoutMount::for_kn6104();
        let part = mount.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_dropout_simple() {
        let mount = SingleDropoutMount::for_kn6104();
        let part = mount.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_dropout_geometry() {
        let mount = SingleDropoutMount::for_kn6104();
        let cfg = mount.config();

        // Slot clearance
        assert!(cfg.slot_width >= cfg.axle_diameter + 1.0,
            "Slot width ({}) must exceed axle ({})",
            cfg.slot_width, cfg.axle_diameter);

        // Axle drop should position wheel below frame
        assert!(mount.axle_drop() > 140.0,
            "Axle drop ({}) should be substantial for wheel positioning",
            mount.axle_drop());
    }

    #[test]
    fn test_dropout_slot_depth() {
        let mount = SingleDropoutMount::for_kn6104();
        assert!(mount.config().slot_depth >= 20.0);
    }
}
