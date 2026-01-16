//! Rover Assemblies
//!
//! Complete rover models for BVR0 (prototype) and BVR1 (production).

use crate::{Part, Scene};
use super::{
    BVR1Frame,
    HubMotor,
    UUMotor,
    LBracketMount,
    BaseTray,
    AccessPanel,
    Vesc,
    Jetson,
    DcDc,
    EStopButton,
    CustomBattery,
    DowntubeBattery,
    Lidar,
    Camera,
    GpsAntenna,
    ShellAssembly,
    WallWrap, WallWrapConfig,
    TopLid, TopLidConfig,
    frame::BVR1FrameConfig,
};

// =============================================================================
// BVR0 Assembly (Prototype)
// =============================================================================

/// BVR0 assembly configuration
#[derive(Debug, Clone)]
pub struct BVR0AssemblyConfig {
    /// Frame configuration
    pub frame: BVR1FrameConfig,
    /// Sensor mast height (mm)
    pub mast_height: f64,
    /// Ground clearance (mm) - bottom of frame to ground
    pub ground_clearance: f64,
}

impl Default for BVR0AssemblyConfig {
    fn default() -> Self {
        // Hoverboard wheel radius ~82mm, we want some clearance
        Self {
            frame: BVR1FrameConfig::default(),
            mast_height: 500.0,
            ground_clearance: 50.0, // Frame bottom 50mm above ground
        }
    }
}

/// BVR0 prototype assembly
///
/// Characteristics:
/// - Hoverboard hub motors mounted directly to bottom frame rail (through-hole axle)
/// - Downtube e-bike battery on central spine
/// - Electronics taped/velcroed to frame (no plate)
/// - VESCs on vertical posts near each wheel
pub struct BVR0Assembly {
    config: BVR0AssemblyConfig,
}

impl BVR0Assembly {
    pub fn new(config: BVR0AssemblyConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr0() -> Self {
        Self::new(BVR0AssemblyConfig::default())
    }

    /// Ground clearance for this assembly
    fn ground_clearance(&self) -> f64 {
        self.config.ground_clearance
    }

    /// Generate complete BVR0 assembly
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // Frame raised by ground clearance
        let frame = BVR1Frame::new(cfg.frame.clone())
            .generate()
            .translate(0.0, 0.0, gc);

        let wheels = self.add_wheels();
        let electronics = self.add_electronics();
        let battery = self.add_battery();
        let sensors = self.add_sensors();

        frame
            .union(&wheels)
            .union(&electronics)
            .union(&battery)
            .union(&sensors)
    }

    /// Generate simplified BVR0 assembly
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let frame = BVR1Frame::new(cfg.frame.clone())
            .generate_simple()
            .translate(0.0, 0.0, gc);

        let wheels = self.add_wheels_simple();
        let jetson = Jetson::recomputer().generate_simple()
            .translate(0.0, 50.0, gc + cfg.frame.height + 25.0);

        frame.union(&wheels).union(&jetson)
    }

    /// Add hoverboard wheels - mounted to bottom frame rail via through-hole axle
    fn add_wheels(&self) -> Part {
        let cfg = &self.config;
        let profile = 20.0;
        let gc = self.ground_clearance();

        let wheel = HubMotor::hoverboard().generate();

        // For BVR0: axle goes through bottom frame rail (through-hole mount)
        // Wheel center at bottom frame rail height
        let x_offset = cfg.frame.width / 2.0 + 30.0;
        let y_offset = cfg.frame.length / 2.0 - profile - 50.0;
        let z_offset = gc + profile / 2.0; // Wheel center at bottom frame rail height

        let fl = wheel.translate(-x_offset, y_offset, z_offset);
        let fr = wheel.rotate(0.0, 0.0, 180.0).translate(x_offset, y_offset, z_offset);
        let rl = wheel.translate(-x_offset, -y_offset, z_offset);
        let rr = wheel.rotate(0.0, 0.0, 180.0).translate(x_offset, -y_offset, z_offset);

        fl.union(&fr).union(&rl).union(&rr)
    }

    fn add_wheels_simple(&self) -> Part {
        let cfg = &self.config;
        let profile = 20.0;
        let gc = self.ground_clearance();

        let wheel = HubMotor::hoverboard().generate_simple();

        let x_offset = cfg.frame.width / 2.0 + 30.0;
        let y_offset = cfg.frame.length / 2.0 - profile - 50.0;
        let z_offset = gc + profile / 2.0;

        let fl = wheel.translate(-x_offset, y_offset, z_offset);
        let fr = wheel.rotate(0.0, 0.0, 180.0).translate(x_offset, y_offset, z_offset);
        let rl = wheel.translate(-x_offset, -y_offset, z_offset);
        let rr = wheel.rotate(0.0, 0.0, 180.0).translate(x_offset, -y_offset, z_offset);

        fl.union(&fr).union(&rl).union(&rr)
    }

    /// Add electronics - taped to frame
    fn add_electronics(&self) -> Part {
        let cfg = &self.config;
        let profile = 20.0;
        let gc = self.ground_clearance();

        // Jetson on top of frame
        let jetson = Jetson::recomputer().generate();
        let jetson_z = gc + cfg.frame.height + 25.0;
        let jetson_placed = jetson.translate(0.0, 50.0, jetson_z);

        // DC-DC next to Jetson
        let dcdc = DcDc::default_48v_12v().generate();
        let dcdc_placed = dcdc.translate(0.0, -30.0, jetson_z);

        // E-Stop on frame
        let estop = EStopButton::new().generate();
        let estop_placed = estop.translate(
            0.0,
            cfg.frame.length / 2.0 - 30.0,
            gc + cfg.frame.height + 20.0,
        );

        // VESCs on vertical posts near each wheel
        let vesc = Vesc::vesc_6().generate();
        let vesc_x = cfg.frame.width / 2.0 - profile * 1.5;
        let vesc_y = cfg.frame.length / 2.0 - profile * 2.5;
        let vesc_z = gc + cfg.frame.height / 2.0 + profile;

        let vesc_fl = vesc.rotate(0.0, 90.0, 0.0).translate(-vesc_x, vesc_y, vesc_z);
        let vesc_fr = vesc.rotate(0.0, -90.0, 0.0).translate(vesc_x, vesc_y, vesc_z);
        let vesc_rl = vesc.rotate(0.0, 90.0, 0.0).translate(-vesc_x, -vesc_y, vesc_z);
        let vesc_rr = vesc.rotate(0.0, -90.0, 0.0).translate(vesc_x, -vesc_y, vesc_z);

        jetson_placed
            .union(&dcdc_placed)
            .union(&estop_placed)
            .union(&vesc_fl)
            .union(&vesc_fr)
            .union(&vesc_rl)
            .union(&vesc_rr)
    }

    /// Add downtube battery on central spine
    fn add_battery(&self) -> Part {
        let gc = self.ground_clearance();
        let battery = DowntubeBattery::standard_48v().generate();
        battery.translate(0.0, 0.0, gc + 25.0)
    }

    /// Add sensor mast
    fn add_sensors(&self) -> Part {
        let gc = self.ground_clearance();
        let mast_top_z = gc + self.config.frame.height + self.config.mast_height;

        let lidar = Lidar::mid360().generate();
        let lidar_placed = lidar.translate(0.0, 0.0, mast_top_z);

        let camera = Camera::insta360_x4().generate();
        let camera_placed = camera.translate(0.0, 0.0, mast_top_z - 100.0);

        let gps = GpsAntenna::default_rtk().generate();
        let gps_placed = gps.translate(80.0, 0.0, mast_top_z - 50.0);

        lidar_placed.union(&camera_placed).union(&gps_placed)
    }
}

// =============================================================================
// BVR1 Assembly (Production)
// =============================================================================

/// BVR1 assembly configuration
///
/// Overall robot dimensions (with default config):
/// - Total width: ~485mm (frame 380mm + wheel protrusion)
/// - Total length: ~550mm (frame 500mm + wheel protrusion)
/// - Total height: ~700mm (ground to top of mast)
/// - Ground clearance: 75mm (effective, with L-bracket mounts)
/// - Mass: ~20kg target
///
/// See `bvr/docs/hardware/bvr1-dimensions.md` for optimization analysis.
#[derive(Debug, Clone)]
pub struct BVR1AssemblyConfig {
    /// Frame configuration
    pub frame: BVR1FrameConfig,
    /// Sensor mast height above frame top (mm)
    pub mast_height: f64,
    /// Ground clearance (mm) - bottom of frame to ground
    /// This is set by the L-bracket mount geometry
    pub ground_clearance: f64,
}

impl Default for BVR1AssemblyConfig {
    fn default() -> Self {
        // Ground clearance calculation:
        //
        // UUMotor SVB6HS: 168mm wheel = 84mm radius
        // L-bracket mount geometry:
        //   - Axle drop below frame bottom: ~36mm
        //   - Wheel center height: ground_clearance - axle_drop
        //   - For wheel to touch ground: wheel_center = wheel_radius
        //   - Therefore: ground_clearance = wheel_radius + axle_drop
        //   - ground_clearance = 84 + 36 = 120mm
        //
        // Effective ground clearance (lowest point of frame): ~75mm
        // (frame bottom at 120mm, but L-bracket extends below)
        Self {
            frame: BVR1FrameConfig::default(),
            mast_height: 400.0,
            ground_clearance: 120.0,
        }
    }
}

/// BVR1 production assembly
///
/// Optimized compact design for sidewalk accessibility:
/// - Frame: 380×500×180mm (W×L×H)
/// - Total footprint: ~485×550mm
/// - Mass target: ~20kg
///
/// Characteristics:
/// - 6.5" (168mm) UUMotor hub motors with L-bracket mounts
/// - Custom 13S4P battery pack in base tray (~720Wh)
/// - All electronics on bottom tray (coplanar, serviceable)
/// - Top access panel with sensor mast
/// - ~75mm effective ground clearance
pub struct BVR1Assembly {
    config: BVR1AssemblyConfig,
}

impl BVR1Assembly {
    pub fn new(config: BVR1AssemblyConfig) -> Self {
        Self { config }
    }

    pub fn default_bvr1() -> Self {
        Self::new(BVR1AssemblyConfig::default())
    }

    /// Ground clearance for this assembly
    fn ground_clearance(&self) -> f64 {
        self.config.ground_clearance
    }

    /// Generate complete BVR1 assembly
    ///
    /// New design:
    /// - All electronics and battery on bottom tray (coplanar)
    /// - Top access panel with sensor mast
    /// - 3-panel clam shell enclosure (wall wrap + top lid + skid plate)
    /// - Clean, serviceable layout
    pub fn generate(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // Frame raised by ground clearance
        let frame = BVR1Frame::new(cfg.frame.clone())
            .generate()
            .translate(0.0, 0.0, gc);

        // Motor mounts and wheels
        let motor_mounts = self.add_motor_mounts();
        let wheels = self.add_wheels();

        // Bottom: base tray with all electronics and battery
        let base_assembly = self.add_base_tray_assembly();

        // Top: access panel with sensors
        let top_assembly = self.add_access_panel_assembly();

        // Shell enclosure (wall wrap + top lid + skid plate)
        let shell = self.add_shell();

        frame
            .union(&motor_mounts)
            .union(&wheels)
            .union(&base_assembly)
            .union(&top_assembly)
            .union(&shell)
    }

    /// Generate simplified BVR1 assembly
    pub fn generate_simple(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let frame = BVR1Frame::new(cfg.frame.clone())
            .generate_simple()
            .translate(0.0, 0.0, gc);

        let wheels = self.add_wheels_simple();

        // Simple base tray
        let tray = BaseTray::default_bvr1().generate_simple()
            .translate(0.0, 0.0, gc + 20.0);

        // Simple access panel
        let panel = AccessPanel::default_bvr1().generate_simple()
            .translate(0.0, 0.0, gc + cfg.frame.height);

        frame.union(&wheels).union(&tray).union(&panel)
    }

    /// Generate BVR1 assembly as a Scene with multiple parts and materials
    ///
    /// Unlike generate() which unions everything into one mesh, this preserves
    /// individual parts for multi-material rendering.
    pub fn generate_scene(&self) -> Scene {
        let cfg = &self.config;
        let gc = self.ground_clearance();
        let shell_cfg = super::shell::ShellConfig::default();

        let mut scene = Scene::new("bvr1_assembly");

        // Frame (silver aluminum 6061)
        let frame = BVR1Frame::new(cfg.frame.clone())
            .generate()
            .translate(0.0, 0.0, gc);
        scene.add(frame, "aluminum_6061");

        // Motor mounts (black anodized aluminum)
        let motor_mounts = self.add_motor_mounts();
        scene.add(motor_mounts, "aluminum_anodized_black");

        // Wheels with tires
        let wheels = self.add_wheels();
        scene.add(wheels, "rubber_tire");

        // Base tray assembly (black HDPE + components)
        let base_tray = BaseTray::default_bvr1()
            .generate()
            .translate(0.0, 0.0, gc + 20.0);
        scene.add(base_tray, "hdpe_black");

        // Electronics on base tray
        let vesc_z = gc + 20.0 + 2.0 + 20.0;
        for (x, y) in [(-80.0, 150.0), (80.0, 150.0), (-80.0, -150.0), (80.0, -150.0)] {
            let vesc = Vesc::vesc_6().generate().translate(x, y, vesc_z);
            scene.add(vesc, "heatsink_aluminum");
        }

        let jetson = Jetson::recomputer()
            .generate()
            .translate(0.0, 0.0, vesc_z);
        scene.add(jetson, "heatsink_aluminum");

        // Battery (black shrink wrap)
        let battery = CustomBattery::bvr1_pack()
            .generate()
            .translate(0.0, -50.0, gc + 20.0 + 2.0);
        scene.add(battery, "battery_shrink");

        // Access panel (black HDPE)
        let panel = AccessPanel::default_bvr1()
            .generate()
            .translate(0.0, 0.0, gc + cfg.frame.height);
        scene.add(panel, "hdpe_black");

        // Shell panels (ORANGE powder-coated aluminum!)
        // Shell walls should match frame height and sit on the frame bottom rail
        // WallWrap generates walls from z=0 to z=shell_height, then we translate up

        // Use frame_height directly for shell walls (no extra clearance needed)
        let mut adjusted_shell_cfg = shell_cfg.clone();
        adjusted_shell_cfg.frame_height = cfg.frame.height;  // 180mm
        adjusted_shell_cfg.clearance = 0.0;  // No extra height beyond frame
        // shell_height() = 180 + 0 = 180mm (matches vertical posts)

        // Position shell bottom at frame bottom (gc = 50mm)
        let wall_wrap = WallWrap::new(WallWrapConfig {
            shell: adjusted_shell_cfg.clone(),
            include_bottom: false,  // No bottom panel - shell sits on frame
            ..Default::default()
        }).generate().translate(0.0, 0.0, gc);
        scene.add(wall_wrap, "aluminum_powder_orange");

        // Top lid sits on top of the walls (at frame top)
        let lid_z = gc + cfg.frame.height;  // 230mm
        let top_lid = TopLid::new(TopLidConfig {
            shell: adjusted_shell_cfg.clone(),
            ..Default::default()
        }).generate().translate(0.0, 0.0, lid_z);
        scene.add(top_lid, "aluminum_powder_orange");

        // Sensors mount on top of the lid
        let mast_base_z = lid_z + 2.0; // lid thickness
        let mast_height = 150.0;

        let lidar = Lidar::mid360()
            .generate()
            .translate(0.0, -shell_cfg.shell_length() / 4.0, mast_base_z + mast_height);
        scene.add(lidar, "sensor_housing");

        let camera = Camera::insta360_x4()
            .generate()
            .translate(0.0, -shell_cfg.shell_length() / 4.0, mast_base_z + mast_height - 80.0);
        scene.add(camera, "sensor_housing");

        let gps = GpsAntenna::default_rtk()
            .generate()
            .translate(80.0, -shell_cfg.shell_length() / 4.0, mast_base_z + mast_height + 30.0);
        scene.add(gps, "abs_black");

        scene
    }

    /// Add L-bracket mounts at each corner
    ///
    /// Mount geometry (L-bracket for single-axle hub motor):
    /// - Horizontal arm bolts to underside of the Y-direction bottom rail
    /// - Vertical arm at outer end, extends outward (±X) and down
    /// - Wheel is INSIDE frame, axle extends OUTWARD to bracket
    ///
    /// Top view (front-left corner):
    /// ```text
    ///                    +Y (front)
    ///                     │
    ///     Frame rail ─────┼───────
    ///                     │
    ///     ┌───────────────┤ Horizontal arm (under left rail)
    ///     │    WHEEL  ○───┤ Vertical arm + axle
    ///     └───────────────┘
    ///     │
    ///    -X (left/outward)
    /// ```
    fn add_motor_mounts(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let mount = LBracketMount::default_bvr1();
        let mount_part = mount.generate();

        // The bracket's default orientation:
        // - Horizontal arm extends in +Y
        // - Vertical arm at far end (Y = arm_length), extends in -X
        // - Axle points in -X
        // This is correct for the LEFT side of the rover

        // Position brackets under the Y-direction rails (left and right side rails)
        let frame_edge_x = cfg.frame.width / 2.0;  // 190mm

        // Bracket origin (inner end of horizontal arm) positioned along the rail
        // Place near corners: front brackets near +Y, rear brackets near -Y
        let bracket_y_front = cfg.frame.length / 2.0 - mount.arm_length();  // Front edge of h_arm at frame front
        let bracket_y_rear = -cfg.frame.length / 2.0;  // Rear of h_arm at frame rear

        // Z position: horizontal arm top surface at frame bottom rail
        let mount_z = gc;

        // Front-left: default orientation (vertical arm extends -X, axle points -X)
        // Origin at X = -frame_edge (under left rail), Y = bracket_y_front
        let mount_fl = mount_part
            .translate(-frame_edge_x, bracket_y_front, mount_z);

        // Front-right: mirror in X (vertical arm extends +X, axle points +X)
        // Rotate 180° around Z to flip the bracket
        let mount_fr = mount_part
            .scale(-1.0, 1.0, 1.0)  // Mirror in X
            .translate(frame_edge_x, bracket_y_front, mount_z);

        // Rear-left: rotate 180° around Z (horizontal arm extends -Y, vertical arm at -Y end)
        let mount_rl = mount_part
            .rotate(0.0, 0.0, 180.0)
            .translate(-frame_edge_x, bracket_y_rear + mount.arm_length(), mount_z);

        // Rear-right: mirror and rotate
        let mount_rr = mount_part
            .scale(-1.0, 1.0, 1.0)
            .rotate(0.0, 0.0, 180.0)
            .translate(frame_edge_x, bracket_y_rear + mount.arm_length(), mount_z);

        mount_fl.union(&mount_fr).union(&mount_rl).union(&mount_rr)
    }

    /// Add UUMotor SVB6HS wheels (168mm / 6.5" wheels)
    ///
    /// Wheel geometry with L-bracket mount:
    /// - Wheels positioned OUTSIDE the frame (protruding from corners)
    /// - Bracket mounted at frame edge, wheel beyond bracket
    /// - Axle points INWARD from wheel to bracket
    ///
    /// Layout (top view, left side):
    /// ```text
    ///     Frame edge (-190mm)
    ///           │
    ///     ──────┼──────  Frame rail
    ///           │
    ///       [Bracket]
    ///           │
    ///         ◯─┘  Wheel (outside, at ~-230mm)
    /// ```
    fn add_wheels(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let wheel = motor.generate();

        // Motor axle offset: distance from wheel center to axle tip
        let axle_offset = motor.axle_offset();  // hub_width/2 + axle_length = 26 + 38 = 64mm

        // Wheel Z: aligned with bracket's axle hole
        let wheel_z = gc - mount.axle_drop();

        // Frame geometry
        let frame_edge_x = cfg.frame.width / 2.0;  // 190mm

        // Bracket axle hole X position (distance from frame center):
        // Bracket at frame edge, axle hole is mount.total_depth() outward
        let bracket_axle_x = frame_edge_x + mount.total_depth();  // 190 + 20 = 210mm

        // Wheel center X: wheel is OUTSIDE bracket, axle points inward toward bracket
        // Wheel center is axle_offset further out from bracket
        let wheel_x = bracket_axle_x + axle_offset;  // 210 + 64 = 274mm (OUTSIDE frame!)

        // Wheel Y: aligned with bracket's axle Y position
        let bracket_y_front = cfg.frame.length / 2.0 - mount.arm_length();
        let wheel_y_front = bracket_y_front + mount.axle_y_offset();
        let wheel_y_rear = -cfg.frame.length / 2.0 + mount.arm_length() - mount.axle_y_offset();

        // Wheel orientation: axle points INWARD (toward bracket at frame edge)
        // Motor default: mount side (axle) at +Y
        // Rotate to point axle toward +X (for left wheels) or -X (for right wheels)
        let wheel_left = wheel.rotate(0.0, 0.0, -90.0);  // Axle toward +X (inward)
        let wheel_right = wheel.rotate(0.0, 0.0, 90.0);  // Axle toward -X (inward)

        let fl = wheel_left.translate(-wheel_x, wheel_y_front, wheel_z);
        let fr = wheel_right.translate(wheel_x, wheel_y_front, wheel_z);
        let rl = wheel_left.translate(-wheel_x, wheel_y_rear, wheel_z);
        let rr = wheel_right.translate(wheel_x, wheel_y_rear, wheel_z);

        fl.union(&fr).union(&rl).union(&rr)
    }

    fn add_wheels_simple(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let wheel = motor.generate_simple();

        let axle_offset = motor.axle_offset();
        let wheel_z = gc - mount.axle_drop();
        let frame_edge_x = cfg.frame.width / 2.0;

        // Wheels OUTSIDE frame (same logic as add_wheels)
        let bracket_axle_x = frame_edge_x + mount.total_depth();
        let wheel_x = bracket_axle_x + axle_offset;

        let bracket_y_front = cfg.frame.length / 2.0 - mount.arm_length();
        let wheel_y_front = bracket_y_front + mount.axle_y_offset();
        let wheel_y_rear = -cfg.frame.length / 2.0 + mount.arm_length() - mount.axle_y_offset();

        let wheel_left = wheel.rotate(0.0, 0.0, -90.0);  // Axle toward +X (inward)
        let wheel_right = wheel.rotate(0.0, 0.0, 90.0);  // Axle toward -X (inward)

        let fl = wheel_left.translate(-wheel_x, wheel_y_front, wheel_z);
        let fr = wheel_right.translate(wheel_x, wheel_y_front, wheel_z);
        let rl = wheel_left.translate(-wheel_x, wheel_y_rear, wheel_z);
        let rr = wheel_right.translate(wheel_x, wheel_y_rear, wheel_z);

        fl.union(&fr).union(&rl).union(&rr)
    }

    /// Add base tray with all electronics and battery (coplanar at bottom)
    fn add_base_tray_assembly(&self) -> Part {
        let profile = 20.0;
        let gc = self.ground_clearance();

        // Base tray sits on the bottom rails
        let tray_thickness = 6.0;
        let tray_z = gc + profile + tray_thickness / 2.0;

        let tray = BaseTray::default_bvr1().generate()
            .translate(0.0, 0.0, tray_z);

        // All components mounted on TOP of the tray
        let component_z = tray_z + tray_thickness / 2.0;

        // Battery pack (center, takes up most of the middle)
        let battery = CustomBattery::bvr1_pack().generate()
            .translate(0.0, 0.0, component_z + 40.0);

        // Jetson (front right)
        let jetson = Jetson::recomputer().generate()
            .translate(130.0, 180.0, component_z + 25.0);

        // DC-DC converter (rear right)
        let dcdc = DcDc::default_48v_12v().generate()
            .translate(130.0, -180.0, component_z + 12.0);

        // 4x VESCs arranged around the battery (left side)
        let vesc = Vesc::vesc_6().generate();

        let vesc_fl = vesc.translate(-160.0, 120.0, component_z + 12.0);
        let vesc_rl = vesc.translate(-160.0, -120.0, component_z + 12.0);
        let vesc_fr = vesc.translate(-160.0, 40.0, component_z + 12.0);
        let vesc_rr = vesc.translate(-160.0, -40.0, component_z + 12.0);

        tray
            .union(&battery)
            .union(&jetson)
            .union(&dcdc)
            .union(&vesc_fl)
            .union(&vesc_fr)
            .union(&vesc_rl)
            .union(&vesc_rr)
    }

    /// Add access panel on top with sensors
    ///
    /// Sensor heights are optimized per bvr/docs/hardware/bvr1-dimensions.md:
    /// - LiDAR at mast top: 700mm from ground (min 620mm for camera visibility)
    /// - Camera 100mm below LiDAR: 600mm from ground (min 520mm to see near ground)
    /// - GPS offset to side, 50mm below LiDAR
    fn add_access_panel_assembly(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // Access panel sits on top of frame
        let panel_thickness = 4.0;
        let panel_z = gc + cfg.frame.height + panel_thickness / 2.0;

        let panel = AccessPanel::default_bvr1().generate()
            .translate(0.0, 0.0, panel_z);

        // Sensor mast position (from AccessPanel config)
        let access_panel = AccessPanel::default_bvr1();
        let (mast_x, mast_y) = access_panel.mast_position();

        // Mast top height
        // With gc=120, frame.height=180, panel=4, mast=400:
        // mast_top = 120 + 180 + 4 + 400 = 704mm from ground
        let mast_top_z = panel_z + panel_thickness / 2.0 + cfg.mast_height;

        // LiDAR on top of mast (at 700mm, must be >620mm for camera ground visibility)
        let lidar = Lidar::mid360().generate()
            .translate(mast_x, mast_y, mast_top_z);

        // Camera 100mm below LiDAR (at 600mm, must be >520mm to see ground at body edge)
        let camera = Camera::insta360_x4().generate()
            .translate(mast_x, mast_y, mast_top_z - 100.0);

        // GPS antenna offset 80mm to side of mast, 50mm below LiDAR
        // (offset reduces multipath from mast structure)
        let gps = GpsAntenna::default_rtk().generate()
            .translate(mast_x + 80.0, mast_y, mast_top_z - 50.0);

        // E-Stop on the panel (accessible from top)
        let estop = EStopButton::new().generate()
            .translate(-150.0, -200.0, panel_z + 20.0);

        panel
            .union(&lidar)
            .union(&camera)
            .union(&gps)
            .union(&estop)
    }

    /// Add 3-panel clam shell enclosure
    ///
    /// Shell components:
    /// - Wall Wrap: Front + sides + rear (single bent piece)
    /// - Top Lid: Removable panel for maintenance access
    /// - Skid Plate: Bottom protection panel
    fn add_shell(&self) -> Part {
        // Shell assembly positioned to enclose the frame
        // The shell's Z=0 is at ground level, frame sits inside at gc height
        let shell = ShellAssembly::default_bvr1().generate();

        // Position shell so it encloses the frame
        // Shell's skid plate is at Z=0 (ground level) + small offset to clear ground
        // Shell's wall wrap starts at skid plate level
        // Shell's top lid is at shell_height above skid plate
        let skid_clearance = 5.0; // 5mm clearance above ground for skid plate
        shell.translate(0.0, 0.0, skid_clearance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvr0_assembly() {
        let assembly = BVR0Assembly::default_bvr0();
        let part = assembly.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_bvr0_assembly_simple() {
        let assembly = BVR0Assembly::default_bvr0();
        let part = assembly.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_bvr1_assembly() {
        let assembly = BVR1Assembly::default_bvr1();
        let part = assembly.generate();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_bvr1_assembly_simple() {
        let assembly = BVR1Assembly::default_bvr1();
        let part = assembly.generate_simple();
        assert!(!part.is_empty());
    }

    #[test]
    fn test_assemblies_can_export_stl() {
        let bvr0 = BVR0Assembly::default_bvr0().generate_simple();
        let bvr1 = BVR1Assembly::default_bvr1().generate_simple();

        assert!(bvr0.to_stl().is_ok());
        assert!(bvr1.to_stl().is_ok());
    }

    #[test]
    fn test_ground_clearance() {
        let bvr0 = BVR0Assembly::default_bvr0();
        let bvr1 = BVR1Assembly::default_bvr1();

        // BVR0 has hoverboard wheels
        assert_eq!(bvr0.ground_clearance(), 50.0);
        // BVR1: frame positioned so wheel center aligns with L-bracket axle
        assert_eq!(bvr1.ground_clearance(), 120.0);
    }

    /// Test L-bracket geometry for single-axle hub motor
    #[test]
    fn test_lbracket_mount_geometry() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let frame_config = BVR1FrameConfig::default();

        let frame_edge = frame_config.width / 2.0;  // 190mm
        let axle_offset = motor.axle_offset();      // 64mm

        // Wheel is OUTSIDE frame to avoid intersection
        let bracket_axle_x = frame_edge + mount.total_depth();
        let wheel_center_x = bracket_axle_x + axle_offset;

        assert!(wheel_center_x > frame_edge,
            "Wheel center ({:.1}mm) should be OUTSIDE frame edge ({:.1}mm)",
            wheel_center_x, frame_edge);

        // L-bracket arm thickness must fit under frame rail
        assert!(mount.flange_thickness() <= 20.0,
            "Horizontal arm ({:.1}mm) must fit against 2020 rail",
            mount.flange_thickness());
    }

    /// Test L-bracket properly supports single-axle motor
    #[test]
    fn test_lbracket_axle_support() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();

        // Motor axle length: 38mm
        // Mount must not exceed this (need room for nut)
        let axle_length = motor.axle_length();  // 38mm
        let mount_depth = mount.total_depth();

        assert!(mount_depth < axle_length,
            "Mount depth ({:.1}mm) must be less than axle length ({:.1}mm) to leave room for nut",
            mount_depth, axle_length);

        // Should have at least 10mm for nut + washer
        let nut_clearance = axle_length - mount_depth;
        assert!(nut_clearance >= 10.0,
            "Need at least 10mm for nut/washer, got {:.1}mm",
            nut_clearance);
    }

    /// Test ADA sidewalk compliance with wheels OUTSIDE frame
    #[test]
    fn test_ada_sidewalk_compliance() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let frame_config = BVR1FrameConfig::default();

        // With wheels outside frame, total width is:
        // frame_edge + bracket_depth + axle_offset + wheel_hub_half_width (on each side)
        // Updated for 540mm frame (500mm extrusions)
        let frame_edge = frame_config.width / 2.0;  // 270mm
        let bracket_axle_x = frame_edge + mount.total_depth();  // 270 + 20 = 290mm
        let wheel_center_x = bracket_axle_x + motor.axle_offset();  // 290 + 64 = 354mm

        // Total width = 2 * wheel_center_x (symmetric)
        // Plus some hub width on each side (hub is centered on wheel_center)
        let total_width = wheel_center_x * 2.0;  // ~708mm

        // ADA minimum clear width is 36" (914mm)
        // Robot should fit within ADA minimum with clearance
        assert!(total_width < 914.0,
            "Total width ({:.0}mm) should be under 914mm for ADA compliance",
            total_width);

        // Verify we fit on a standard 48" (1220mm) sidewalk with clearance
        let sidewalk_48in = 1220.0;
        let clearance_each_side = (sidewalk_48in - total_width) / 2.0;
        assert!(clearance_each_side >= 200.0,
            "Should have 200mm+ clearance on each side of 48\" sidewalk, got {:.0}mm",
            clearance_each_side);
    }

    /// Test wheel Z position with L-bracket
    #[test]
    fn test_wheel_z_position() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let bvr1 = BVR1Assembly::default_bvr1();

        let wheel_radius = motor.wheel_diameter() / 2.0;  // 84mm
        let gc = bvr1.ground_clearance();                 // 120mm

        // Wheel center Z = gc - axle_drop
        let wheel_z = gc - mount.axle_drop();

        // Wheel should touch ground (wheel_z == wheel_radius)
        let ground_gap = wheel_z - wheel_radius;
        assert!(ground_gap.abs() < 20.0,
            "Wheel should nearly touch ground. Gap: {:.1}mm (wheel_z={:.1}, radius={:.1})",
            ground_gap, wheel_z, wheel_radius);

        // Wheel should be below frame bottom
        assert!(wheel_z < gc,
            "Wheel center ({:.1}mm) must be below frame bottom ({:.1}mm)",
            wheel_z, gc);
    }

    /// Test that wheels are OUTSIDE frame footprint (no intersection)
    #[test]
    fn test_wheel_outside_frame() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let frame_config = BVR1FrameConfig::default();

        let frame_edge = frame_config.width / 2.0;  // 190mm
        let axle_offset = motor.axle_offset();      // 64mm
        let wheel_radius = motor.wheel_diameter() / 2.0;  // 84mm

        // Wheel center X (distance from centerline)
        let bracket_axle_x = frame_edge + mount.total_depth();  // 190 + 20 = 210mm
        let wheel_center_x = bracket_axle_x + axle_offset;      // 210 + 64 = 274mm

        // Wheel center should be OUTSIDE frame
        assert!(wheel_center_x > frame_edge,
            "Wheel center ({:.1}mm) should be outside frame edge ({:.1}mm)",
            wheel_center_x, frame_edge);

        // Wheel inner edge should clear frame edge
        let wheel_inner_edge = wheel_center_x - wheel_radius;  // 274 - 84 = 190mm
        assert!(wheel_inner_edge >= frame_edge,
            "Wheel inner edge ({:.1}mm) should clear frame edge ({:.1}mm)",
            wheel_inner_edge, frame_edge);
    }
}
