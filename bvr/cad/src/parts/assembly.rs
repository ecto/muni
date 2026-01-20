//! Rover Assemblies
//!
//! Complete rover models for BVR0 (prototype) and BVR1 (production).

use crate::{Part, Scene};
use super::{
    BVR1Frame,
    HubMotor,
    UUMotor,
    SingleDropoutMount,
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
    ProxicastAntenna,
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
        // Ground clearance calculation for KN6104 10" motors with single-sided dropout mount:
        //
        // UUMotor KN6104: 270mm wheel = 135mm radius
        // Dropout geometry (simplified plate):
        //   - Plate height: 170mm
        //   - Slot depth: 26mm
        //   - Axle drop: 170 - 13 ≈ 157mm below frame bottom
        //   - Wheel center height: ground_clearance - axle_drop
        //   - For wheel to touch ground: wheel_center = wheel_radius
        //   - Therefore: ground_clearance ≈ 135 + 157 = 292mm
        // Using 295mm for margin
        Self {
            frame: BVR1FrameConfig::default(),
            mast_height: 400.0,
            ground_clearance: 295.0,
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
    /// "Friendly Industrial" design:
    /// - All electronics and battery on bottom tray (coplanar)
    /// - Top access panel with e-stop
    /// - Shell enclosure with chamfered corners and sensor dome
    /// - Sensors housed in low dome ("subtle, not towering")
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

        // Top: access panel (sensors now in dome)
        let top_assembly = self.add_access_panel_assembly();

        // Shell enclosure (wall wrap + top lid, no dome)
        let shell = self.add_shell();

        // LiDAR mounted directly on lid
        let lidar = self.add_lidar();

        // Stereo cameras in the front face ("eyes")
        let front_cameras = self.add_front_cameras();

        // Proxicast antenna on lid (rear-right)
        let proxicast_antenna = self.add_proxicast_antenna();

        frame
            .union(&motor_mounts)
            .union(&wheels)
            .union(&base_assembly)
            .union(&top_assembly)
            .union(&shell)
            .union(&lidar)
            .union(&front_cameras)
            .union(&proxicast_antenna)
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

        // NOTE: Insta360 camera removed - using CSI stereo cameras in front face
        // NOTE: GPS antenna removed - Proxicast 5-in-1 handles GPS (flush mounted)

        scene
    }

    /// Add single-sided dropout/torque-arm mounts at each corner
    ///
    /// Mount geometry:
    /// - Thick plate with M16 slot (dropout) capturing the axle flats
    /// - Plate hangs below frame; axle sits near the bottom of the slot
    /// - Short width along Y; thickness along X (axle passes through thickness)
    /// - Intended for single-shaft KN6104 (bicycle-style)
    fn add_motor_mounts(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let mount = SingleDropoutMount::for_kn6104();
        let mount_part = mount.generate();

        // Position plates at corners, top flush to frame bottom
        let _frame_edge_x = cfg.frame.width / 2.0;  // 270mm

        // Mount center X: under shell edge (outer edge ≈ 290mm)
        let mount_center_x = 240.0;

        // Y positions: near front and rear of frame
        let mount_y_front = cfg.frame.length / 2.0 - 60.0;   // 60mm from front edge
        let mount_y_rear = -cfg.frame.length / 2.0 + 60.0;   // 60mm from rear edge

        // Z: plate top at frame bottom
        let mount_z = gc;

        // Plate orientation: arm runs from frame (inboard) to wheel (outboard) along +X for right, -X for left
        // Front-left: mirror in X
        let mount_fl = mount_part
            .scale(-1.0, 1.0, 1.0)
            .translate(-mount_center_x, mount_y_front, mount_z);

        // Front-right: default orientation
        let mount_fr = mount_part
            .translate(mount_center_x, mount_y_front, mount_z);

        // Rear-left: mirror in X
        let mount_rl = mount_part
            .scale(-1.0, 1.0, 1.0)
            .translate(-mount_center_x, mount_y_rear, mount_z);

        // Rear-right: default orientation
        let mount_rr = mount_part
            .translate(mount_center_x, mount_y_rear, mount_z);

        mount_fl.union(&mount_fr).union(&mount_rl).union(&mount_rr)
    }

    /// Add UUMotor KN6104 wheels (270mm / 10" wheels)
    ///
    /// Wheel geometry with single-sided dropout mount:
    /// - Thick plate captures the axle flats
    /// - Wheel sits outboard of the frame, axle along X
    /// - Brace to frame provides bending stiffness
    ///
    /// Layout (front view, left side):
    /// ```text
    ///     Frame (270mm)  Shell (290mm)
    ///           │             │
    ///     ──────┼─────────────┼───────────────
    ///           │             │
    ///      ┌────┴────┐   ┌────┴────┐
    ///      │  Bridge │   │  Fork   │
    ///      ├─────────┤   ├─────────┤
    ///      │ ┌─────┐ │   │ ┌─────┐ │
    ///      │ │     │ │   │ │     │ │
    ///      │ │ LEG │◯┼───┼◯│ LEG │ │
    ///      │ └─────┘ │   │ └─────┘ │
    ///      └─────────┘   └─────────┘
    ///                │
    ///            [WHEEL]
    /// ```
    fn add_wheels(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // KN6104 10" motors (from Tony @ UUMotor, Jan 2026)
        let motor = UUMotor::kn6104();
        let mount = SingleDropoutMount::for_kn6104();
        let wheel = motor.generate();

        // Wheel center X: under shell edge (outer edge ≈ 290mm)
        let wheel_x = 240.0;

        // Wheel Z: in the dropout slot (top of plate at gc)
        let wheel_z = gc - mount.axle_drop();  // ≈ wheel radius

        // Y positions: aligned with fork centers
        let wheel_y_front = cfg.frame.length / 2.0 - 60.0;
        let wheel_y_rear = -cfg.frame.length / 2.0 + 60.0;

        // Wheel orientation: rotate so axle lies along X to match dropout slot
        let wheel_left = wheel.rotate(0.0, 0.0, 90.0);
        let wheel_right = wheel.rotate(0.0, 0.0, -90.0);

        let fl = wheel_left.translate(-wheel_x, wheel_y_front, wheel_z);
        let fr = wheel_right.translate(wheel_x, wheel_y_front, wheel_z);
        let rl = wheel_left.translate(-wheel_x, wheel_y_rear, wheel_z);
        let rr = wheel_right.translate(wheel_x, wheel_y_rear, wheel_z);

        fl.union(&fr).union(&rl).union(&rr)
    }

    fn add_wheels_simple(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let motor = UUMotor::kn6104();
        let mount = SingleDropoutMount::for_kn6104();
        let wheel = motor.generate_simple();

        let wheel_x = 240.0;
        let wheel_z = gc - mount.axle_drop();

        let wheel_y_front = cfg.frame.length / 2.0 - 60.0;
        let wheel_y_rear = -cfg.frame.length / 2.0 + 60.0;

        // Match wheel rotation to fork rotation
        let wheel_left = wheel.rotate(0.0, 0.0, 90.0);
        let wheel_right = wheel.rotate(0.0, 0.0, -90.0);

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

    /// Add access panel on top of frame (inside shell)
    ///
    /// Note: Sensors are now housed in the shell's sensor dome, not on a tall mast.
    /// The "friendly industrial" design keeps sensors low and protected.
    fn add_access_panel_assembly(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // Access panel sits on top of frame (inside the shell)
        let panel_thickness = 4.0;
        let panel_z = gc + cfg.frame.height + panel_thickness / 2.0;

        let panel = AccessPanel::default_bvr1().generate()
            .translate(0.0, 0.0, panel_z);

        // E-Stop aligned with the lid hole (front-left)
        let lid_cfg = super::shell::TopLidConfig::default();
        let estop = EStopButton::new().generate()
            .translate(lid_cfg.estop_hole_offset.0, lid_cfg.estop_hole_offset.1, panel_z + 20.0);

        // Note: Sensors (LiDAR, camera, GPS) are now positioned inside
        // the sensor dome via add_dome_sensors()
        panel.union(&estop)
    }

    /// Add LiDAR mounted directly on the lid
    ///
    /// Livox Mid-360 specs:
    /// - 65×65×60mm body with hemispherical scanning window
    /// - Mounts flat on surface with 4× M3 screws
    /// - 360° horizontal FOV, 59° vertical FOV
    /// - Only 265g - no dome enclosure needed
    fn add_lidar(&self) -> Part {
        let gc = self.ground_clearance();
        let shell_cfg = super::shell::ShellConfig::default();

        // LiDAR sits on top of lid surface
        let lid_top_z = gc + shell_cfg.shell_height() + shell_cfg.thickness;

        // Position centered on lid (slightly forward for balance)
        let lidar_x = 0.0;
        let lidar_y = 0.0; // Centered

        Lidar::mid360().generate()
            .translate(lidar_x, lidar_y, lid_top_z)
    }

    /// Add stereo cameras in the front face
    ///
    /// "The Face" - two cameras as "eyes" for pareidolia effect
    /// Positioned in the visor (forehead), spaced at human IPD (63mm)
    /// NOTE: Cameras are now in the lid visor, so they lift with the lid
    fn add_front_cameras(&self) -> Part {
        let gc = self.ground_clearance();
        let shell_cfg = super::shell::ShellConfig::default();
        let lid_cfg = super::shell::TopLidConfig::default();

        // Front face position (visor hangs from lid front edge)
        let shell_front_y = shell_cfg.shell_length() / 2.0;
        let shell_height = shell_cfg.shell_height();

        // Camera Z position (in the visor, which hangs below the lid)
        // Visor is on the lid at height shell_height + thickness, cameras in visor
        let camera_z = gc + shell_height - lid_cfg.visor_camera_offset_from_top;

        // Stereo cameras spaced at IPD (63mm)
        let spacing = lid_cfg.visor_camera_spacing;

        // Left camera (simple representation)
        let left_cam = Camera::new(super::sensors::CameraConfig {
            diameter: 25.0,  // CSI camera module size
            height: 25.0,
            lens_diameter: 12.0,
        }).generate()
        .rotate(90.0, 0.0, 0.0)  // Point forward
        .translate(-spacing / 2.0, shell_front_y, camera_z);

        // Right camera
        let right_cam = Camera::new(super::sensors::CameraConfig {
            diameter: 25.0,
            height: 25.0,
            lens_diameter: 12.0,
        }).generate()
        .rotate(90.0, 0.0, 0.0)
        .translate(spacing / 2.0, shell_front_y, camera_z);

        left_cam.union(&right_cam)
    }

    /// Add Proxicast 5-in-1 combo antenna on the lid
    ///
    /// Mounted flush on rear-right of lid, provides:
    /// - LTE/4G cellular (MIMO)
    /// - WiFi 2.4/5GHz (MIMO)
    /// - GPS/GLONASS
    fn add_proxicast_antenna(&self) -> Part {
        let gc = self.ground_clearance();
        let shell_cfg = super::shell::ShellConfig::default();
        let lid_cfg = super::shell::TopLidConfig::default();

        // Antenna sits on top of the lid
        let lid_top_z = gc + shell_cfg.shell_height() + shell_cfg.thickness;

        // Position at the antenna hole (rear-right quadrant)
        let antenna_x = lid_cfg.antenna_hole_offset.0;
        let antenna_y = lid_cfg.antenna_hole_offset.1;

        ProxicastAntenna::default_5in1()
            .generate()
            .translate(antenna_x, antenna_y, lid_top_z)
    }

    /// Add 3-panel clam shell enclosure
    ///
    /// Shell components:
    /// - Wall Wrap: Front + sides + rear (single bent piece)
    /// - Top Lid: Removable panel for maintenance access
    /// - Skid Plate: Bottom protection panel
    fn add_shell(&self) -> Part {
        let gc = self.ground_clearance();

        // Shell assembly wraps around the frame
        // Frame is at Z=gc (120mm) to Z=gc+frame.height (300mm)
        // Shell walls should start below frame bottom to fully enclose it

        // The shell's wall wrap starts at its Z=0 and extends to shell_height (200mm)
        // Position shell so its walls wrap the frame:
        // - Shell bottom at gc - some_margin (to cover frame bottom rail)
        // - Shell top at gc + frame.height + lid_clearance

        // For BVR1: gc=120, frame.height=180, shell_height=200
        // Shell walls go from shell_z to shell_z+200
        // We want walls to cover frame (120-300), so shell_z should be ~100mm
        // But the shell is designed for the frame dimensions, so position it at gc

        let shell = ShellAssembly::default_bvr1().generate();

        // Position shell bottom at ground clearance level
        // The shell's integrated bottom sits at gc level (on top of wheels)
        // Wall wrap wraps the frame from gc to gc+shell_height
        shell.translate(0.0, 0.0, gc)
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
        // BVR1 with KN6104 10" wheels + dropout mount: frame positioned high for wheel clearance
        assert_eq!(bvr1.ground_clearance(), 300.0);
    }

    /// Test dropout mount geometry for KN6104 motor
    #[test]
    fn test_dropout_mount_geometry() {
        let motor = UUMotor::kn6104();
        let mount = SingleDropoutMount::for_kn6104();

        // Slot must accept the M16 axle with clearance
        assert!(mount.config().slot_width >= motor.axle_diameter() + 1.0,
            "Slot width ({:.1}mm) should exceed axle ({:.1}mm)",
            mount.config().slot_width, motor.axle_diameter());

        // Axle drop should position wheel near ground
        assert!(mount.axle_drop() > 140.0,
            "Axle drop ({:.1}mm) should be substantial for wheel positioning",
            mount.axle_drop());
    }

    /// Test dropout properly supports KN6104 motor
    #[test]
    fn test_dropout_axle_support() {
        let motor = UUMotor::kn6104();
        let mount = SingleDropoutMount::for_kn6104();

        // KN6104 axle length: 45mm -> must extend beyond slot for nut
        let axle_length = motor.axle_length();
        let slot_depth = mount.config().slot_depth;

        // Slot should be deep enough for secure seating
        assert!(slot_depth >= 20.0,
            "Slot depth ({:.1}mm) should be >= 20mm for secure axle seating",
            slot_depth);

        // Axle should extend past slot for nut
        assert!(axle_length > slot_depth,
            "Axle ({:.1}mm) should extend past slot ({:.1}mm) for nut",
            axle_length, slot_depth);
    }

    /// Test ADA sidewalk compliance with dropout mount
    #[test]
    fn test_ada_sidewalk_compliance() {
        let motor = UUMotor::kn6104();

        // Wheel center at 350mm from centerline
        let wheel_center_x = 350.0;
        let tire_half_width = motor.config().tire_width / 2.0;  // 50mm

        // Total width = 2 * (wheel_center + tire_half)
        let total_width = (wheel_center_x + tire_half_width) * 2.0;  // 800mm

        // ADA minimum clear width is 36" (914mm)
        // Robot should fit within ADA minimum with clearance
        assert!(total_width < 914.0,
            "Total width ({:.0}mm) should be under 914mm for ADA compliance",
            total_width);

        // Verify we fit on a standard 48" (1220mm) sidewalk with clearance
        let sidewalk_48in = 1220.0;
        let clearance_each_side = (sidewalk_48in - total_width) / 2.0;
        assert!(clearance_each_side >= 150.0,
            "Should have 150mm+ clearance on each side of 48\" sidewalk, got {:.0}mm",
            clearance_each_side);
    }

    /// Test wheel Z position with dropout mount
    #[test]
    fn test_wheel_z_position() {
        let motor = UUMotor::kn6104();
        let mount = SingleDropoutMount::for_kn6104();
        let bvr1 = BVR1Assembly::default_bvr1();

        let wheel_radius = motor.wheel_diameter() / 2.0;  // 135mm
        let gc = bvr1.ground_clearance();                 // 300mm

        // Wheel center Z = gc - axle_drop
        let wheel_z = gc - mount.axle_drop();

        // Wheel should touch ground (wheel_z ≈ wheel_radius)
        let ground_gap = wheel_z - wheel_radius;
        assert!(ground_gap.abs() < 5.0,
            "Wheel should nearly touch ground. Gap: {:.1}mm (wheel_z={:.1}, radius={:.1})",
            ground_gap, wheel_z, wheel_radius);

        // Wheel should be below frame bottom
        assert!(wheel_z < gc,
            "Wheel center ({:.1}mm) must be below frame bottom ({:.1}mm)",
            wheel_z, gc);
    }

    /// Test that tire clears shell with dropout mount
    #[test]
    fn test_tire_clears_shell() {
        let motor = UUMotor::kn6104();

        // Shell outer edge at 290mm from centerline (580mm / 2)
        let shell_edge = 290.0;

        // Wheel center at 240mm (outer edge ≈ 290mm)
        let wheel_center_x = 240.0;
        let tire_half_width = motor.config().tire_width / 2.0;  // 50mm

        // Tire outer edge
        let tire_outer_edge = wheel_center_x + tire_half_width;  // 290mm

        // Tire should not exceed shell edge
        assert!(tire_outer_edge <= shell_edge + 1.0,
            "Tire outer edge ({:.1}mm) should be at/below shell edge ({:.1}mm)",
            tire_outer_edge, shell_edge);
    }
}
