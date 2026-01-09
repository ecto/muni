//! Rover Assemblies
//!
//! Complete rover models for BVR0 (prototype) and BVR1 (production).

use crate::Part;
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
#[derive(Debug, Clone)]
pub struct BVR1AssemblyConfig {
    /// Frame configuration
    pub frame: BVR1FrameConfig,
    /// Sensor mast height (mm)
    pub mast_height: f64,
    /// Ground clearance (mm) - bottom of frame to ground
    pub ground_clearance: f64,
}

impl Default for BVR1AssemblyConfig {
    fn default() -> Self {
        // UUMotor SVB6HS: 168mm wheel = 84mm radius
        // Mount bridge is 36mm above axle (plate_height/2 - tab_height/2 = 80/2 - 8/2)
        // Wheel center at Z = 84, bridge at Z = 84 + 36 = 120
        // Frame bottom should be at bridge height for proper mounting
        Self {
            frame: BVR1FrameConfig::default(),
            mast_height: 400.0,
            ground_clearance: 120.0, // Frame bottom at 120mm (aligns with mount bridge)
        }
    }
}

/// BVR1 production assembly
///
/// Characteristics:
/// - 8" hub motors with custom L-bracket mounts at bottom corners
/// - Custom 13S4P battery pack in tray
/// - Electronics plate with proper mounting
/// - VESCs on vertical posts with mounting brackets
/// - Headlights and tail lights
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

        frame
            .union(&motor_mounts)
            .union(&wheels)
            .union(&base_assembly)
            .union(&top_assembly)
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
    /// - Wheels positioned INSIDE the frame
    /// - Axle extends OUTWARD to bracket mounted at frame edge
    /// - Wheel center offset from bracket axle hole by motor.axle_offset()
    fn add_wheels(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let wheel = motor.generate();

        // Motor axle offset: distance from wheel center to axle tip
        // This is how far the axle protrudes from the hub
        let axle_offset = motor.axle_offset();  // hub_width/2 + axle_length = 26 + 38 = 64mm

        // Wheel Z: aligned with bracket's axle hole
        let wheel_z = gc - mount.axle_drop();

        // Frame geometry
        let frame_edge_x = cfg.frame.width / 2.0;  // 190mm

        // Bracket axle hole X position (for left side):
        // Bracket origin at -frame_edge_x, axle hole offset by -axle_x_offset
        let bracket_axle_x = frame_edge_x + mount.axle_x_offset();  // 190 + 6 = 196mm

        // Wheel center X: axle tip at bracket, so wheel center is axle_offset INWARD
        let wheel_x = bracket_axle_x - axle_offset;  // 196 - 64 = 132mm (INSIDE frame!)

        // Wheel Y: aligned with bracket's axle Y position
        let bracket_y_front = cfg.frame.length / 2.0 - mount.arm_length();
        let wheel_y_front = bracket_y_front + mount.axle_y_offset();
        let wheel_y_rear = -cfg.frame.length / 2.0 + mount.arm_length() - mount.axle_y_offset();

        // Wheel orientation: axle points outward (toward ±X)
        // Motor default: axle along +Y
        // Rotate around Z by 90° to point axle toward -X (for left side wheels)
        let wheel_left = wheel.rotate(0.0, 0.0, 90.0);   // Axle toward -X
        let wheel_right = wheel.rotate(0.0, 0.0, -90.0); // Axle toward +X

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
        let bracket_axle_x = frame_edge_x + mount.axle_x_offset();
        let wheel_x = bracket_axle_x - axle_offset;

        let bracket_y_front = cfg.frame.length / 2.0 - mount.arm_length();
        let wheel_y_front = bracket_y_front + mount.axle_y_offset();
        let wheel_y_rear = -cfg.frame.length / 2.0 + mount.arm_length() - mount.axle_y_offset();

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

    /// Add access panel on top with sensors
    fn add_access_panel_assembly(&self) -> Part {
        let cfg = &self.config;
        let gc = self.ground_clearance();

        // Access panel sits on top of frame
        let panel_thickness = 4.0;
        let panel_z = gc + cfg.frame.height + panel_thickness / 2.0;

        let panel = AccessPanel::default_bvr1().generate()
            .translate(0.0, 0.0, panel_z);

        // Sensor mast goes through the panel
        let mast_y = 200.0; // Same as AccessPanel::mast_offset_y
        let mast_top_z = panel_z + cfg.mast_height;

        // LiDAR on top of mast
        let lidar = Lidar::mid360().generate()
            .translate(0.0, mast_y, mast_top_z);

        // Camera below LiDAR
        let camera = Camera::insta360_x4().generate()
            .translate(0.0, mast_y, mast_top_z - 100.0);

        // GPS antenna offset from mast
        let gps = GpsAntenna::default_rtk().generate()
            .translate(80.0, mast_y, mast_top_z - 50.0);

        // E-Stop on the panel (accessible from top)
        let estop = EStopButton::new().generate()
            .translate(-150.0, -200.0, panel_z + 20.0);

        panel
            .union(&lidar)
            .union(&camera)
            .union(&gps)
            .union(&estop)
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

        // With new design: wheel is INSIDE frame
        let bracket_axle_x = frame_edge + mount.axle_x_offset();
        let wheel_center_x = bracket_axle_x - axle_offset;

        assert!(wheel_center_x < frame_edge,
            "Wheel center ({:.1}mm) should be INSIDE frame edge ({:.1}mm)",
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

    /// Test ADA sidewalk compliance with wheels INSIDE frame
    #[test]
    fn test_ada_sidewalk_compliance() {
        let mount = LBracketMount::default_bvr1();
        let frame_config = BVR1FrameConfig::default();

        // With wheels inside frame, total width is approximately frame width
        // plus the bracket overhang on each side
        let frame_edge = frame_config.width / 2.0;
        let bracket_overhang = mount.axle_x_offset() + mount.total_depth();
        let total_width = (frame_edge + bracket_overhang) * 2.0;

        // ADA minimum clear width is 36" (914mm)
        assert!(total_width < 600.0,
            "Total width ({:.0}mm) should be under 600mm for ADA compliance",
            total_width);

        // Verify we fit on a standard 48" (1220mm) sidewalk with generous clearance
        let sidewalk_48in = 1220.0;
        let clearance_each_side = (sidewalk_48in - total_width) / 2.0;
        assert!(clearance_each_side >= 300.0,
            "Should have 300mm+ clearance on each side of 48\" sidewalk, got {:.0}mm",
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

    /// Test that wheels are INSIDE frame footprint (new design)
    #[test]
    fn test_wheel_inside_frame() {
        let motor = UUMotor::svb6hs();
        let mount = LBracketMount::default_bvr1();
        let frame_config = BVR1FrameConfig::default();

        let frame_edge = frame_config.width / 2.0;
        let axle_offset = motor.axle_offset();
        let bracket_axle_x = frame_edge + mount.axle_x_offset();
        let wheel_center_x = bracket_axle_x - axle_offset;

        // Wheel center should be inside frame
        assert!(wheel_center_x < frame_edge,
            "Wheel center ({:.1}mm) should be inside frame edge ({:.1}mm)",
            wheel_center_x, frame_edge);

        // Wheel should have clearance from frame centerline
        assert!(wheel_center_x > 50.0,
            "Wheel center ({:.1}mm) should not be too close to centerline",
            wheel_center_x);
    }
}
