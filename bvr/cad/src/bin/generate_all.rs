//! Generate all BVR1 CAD parts in multiple formats
//!
//! Outputs:
//! - STL: Manufacturing meshes
//! - glTF/GLB: Visualization with PBR materials
//! - USD: Isaac Sim robot description

use anyhow::Result;
use bvr_cad::export::{export_stl, Materials};
#[cfg(feature = "gltf")]
use bvr_cad::export::export_glb;
use bvr_cad::export::{export_usd, export_robot_usd, WheelConfig};
use bvr_cad::parts::{
    // Custom fabricated
    BVR1Frame, CornerBracket, ElectronicsPlate, Extrusion2020, MotorMount, SensorMount, TNut,
    WheelSpacer, BatteryTray, BaseTray, AccessPanel, UUMotor, UUMotorMount,
    // Reference parts
    HubMotor, Lidar, Camera, GpsAntenna, Vesc, Jetson, DcDc, EStopButton,
    DowntubeBattery, CustomBattery,
    // Scale reference models
    Banana, Human,
    // Complete assemblies
    BVR0Assembly, BVR1Assembly,
};
use bvr_cad::Part;
use std::path::Path;

fn main() -> Result<()> {
    println!("BVR1 Multi-Format CAD Generator");
    println!("================================\n");

    // Load materials database
    let materials = Materials::load("config/materials.toml")
        .unwrap_or_else(|e| {
            eprintln!("Warning: Could not load materials.toml: {}", e);
            eprintln!("Using default materials.\n");
            Materials::parse("[materials.default]\ncolor = [0.5, 0.5, 0.5]").unwrap()
        });

    // Create export directories
    let base_dir = Path::new("exports");
    let stl_dir = base_dir.join("stl");
    let gltf_dir = base_dir.join("gltf");
    let usd_dir = base_dir.join("usd");

    std::fs::create_dir_all(&stl_dir)?;
    std::fs::create_dir_all(&gltf_dir)?;
    std::fs::create_dir_all(&usd_dir)?;

    let mut stl_count = 0;
    let mut gltf_count = 0;
    let mut usd_count = 0;

    // Helper to export a part in all formats
    let mut export_part = |name: &str, part: &Part, mat_key: &str| -> Result<()> {
        let material = materials.get_for_part_or_default(mat_key);

        // STL
        let stl_path = stl_dir.join(format!("{}.stl", name));
        export_stl(part, &stl_path)?;
        stl_count += 1;

        // glTF
        #[cfg(feature = "gltf")]
        {
            let glb_path = gltf_dir.join(format!("{}.glb", name));
            export_glb(part, &material, &glb_path)?;
            gltf_count += 1;
        }

        // USD (skip for assemblies, handled separately)
        if !name.contains("assembly") {
            let usd_path = usd_dir.join(format!("{}.usda", name));
            export_usd(part, &material, &usd_path)?;
            usd_count += 1;
        }

        Ok(())
    };

    // =========================================================================
    // FRAME COMPONENTS
    // =========================================================================
    println!("Frame Components");
    println!("----------------");

    let part = Extrusion2020::new(100.0).generate();
    export_part("extrusion_2020_100mm", &part, "extrusion_2020")?;
    println!("  2020 extrusion (100mm)");

    let part = Extrusion2020::new(500.0).generate();
    export_part("extrusion_2020_500mm", &part, "extrusion_2020")?;
    println!("  2020 extrusion (500mm)");

    let part = CornerBracket::standard().generate();
    export_part("corner_bracket", &part, "corner_bracket")?;
    println!("  Corner bracket");

    let part = TNut::m5().generate();
    export_part("tnut_m5", &part, "tnut")?;
    println!("  T-nut (M5)");

    let part = BVR1Frame::default_bvr1().generate();
    export_part("bvr1_frame", &part, "bvr1_frame")?;
    println!("  BVR1 Frame");

    // =========================================================================
    // CUSTOM FABRICATED PARTS
    // =========================================================================
    println!("\nCustom Fabricated Parts");
    println!("-----------------------");

    let mount = MotorMount::hub_motor_8in();
    let part = mount.generate();
    export_part("motor_mount_8in", &part, "motor_mount")?;
    println!("  Motor mount L-bracket");

    let part = mount.generate_flat_plate();
    export_part("motor_mount_plate_8in", &part, "motor_mount")?;
    println!("  Motor mount flat plate");

    let part = mount.generate_tabs();
    export_part("motor_mount_tab_8in", &part, "motor_mount")?;
    println!("  Motor mount tab");

    let spacer = WheelSpacer::hub_motor_8in();
    let part = spacer.generate();
    export_part("wheel_spacer_8in", &part, "wheel_spacer")?;
    println!("  Wheel spacer");

    let part = spacer.generate_flat();
    export_part("wheel_spacer_flat_8in", &part, "wheel_spacer")?;
    println!("  Wheel spacer (flat)");

    let plate = ElectronicsPlate::default_bvr1();
    let part = plate.generate();
    export_part("electronics_plate", &part, "electronics_plate")?;
    println!("  Electronics plate");

    let sensor = SensorMount::default_bvr1();
    let part = sensor.generate();
    export_part("sensor_mount", &part, "sensor_mount")?;
    println!("  Sensor mount");

    let part = sensor.generate_base_plate();
    export_part("sensor_mount_base", &part, "sensor_mount")?;
    println!("  Sensor mount base");

    let tray = BatteryTray::bvr1_tray();
    let part = tray.generate();
    export_part("battery_tray", &part, "battery_tray")?;
    println!("  Battery tray");

    let base = BaseTray::default_bvr1();
    let part = base.generate();
    export_part("base_tray", &part, "base_tray")?;
    println!("  Base tray");

    let panel = AccessPanel::default_bvr1();
    let part = panel.generate();
    export_part("access_panel", &part, "access_panel")?;
    println!("  Access panel");

    // =========================================================================
    // DRIVETRAIN
    // =========================================================================
    println!("\nDrivetrain");
    println!("----------");

    let part = UUMotor::kn6104().generate();
    export_part("uumotor_kn6104", &part, "uumotor_body")?;
    println!("  UUMotor KN6104 (10\")");

    let part = UUMotor::svb6hs().generate();
    export_part("uumotor_svb6hs", &part, "uumotor_body")?;
    println!("  UUMotor SVB6HS (6.5\")");

    let part = UUMotorMount::default_bvr1().generate();
    export_part("uumotor_mount", &part, "motor_mount")?;
    println!("  UUMotor fork mount");

    let part = HubMotor::hub_8in().generate();
    export_part("ref_hub_motor_8in", &part, "hub_motor_body")?;
    println!("  Hub motor 8\"");

    let part = HubMotor::hoverboard().generate();
    export_part("ref_hub_motor_hoverboard", &part, "hub_motor_body")?;
    println!("  Hub motor hoverboard");

    // =========================================================================
    // ELECTRONICS & SENSORS
    // =========================================================================
    println!("\nElectronics & Sensors");
    println!("---------------------");

    let part = Lidar::mid360().generate();
    export_part("ref_lidar_mid360", &part, "lidar")?;
    println!("  Livox Mid-360");

    let part = Camera::insta360_x4().generate();
    export_part("ref_camera_insta360", &part, "camera")?;
    println!("  Insta360 X4");

    let part = GpsAntenna::default_rtk().generate();
    export_part("ref_gps_antenna", &part, "gps_antenna")?;
    println!("  GPS antenna");

    let part = Vesc::vesc_6().generate();
    export_part("ref_vesc_6", &part, "vesc")?;
    println!("  VESC 6");

    let part = Jetson::recomputer().generate();
    export_part("ref_jetson_recomputer", &part, "jetson")?;
    println!("  Jetson reComputer");

    let part = DcDc::default_48v_12v().generate();
    export_part("ref_dcdc", &part, "dcdc")?;
    println!("  DC-DC converter");

    let part = EStopButton::new().generate();
    export_part("ref_estop", &part, "estop")?;
    println!("  E-Stop button");

    // =========================================================================
    // BATTERIES
    // =========================================================================
    println!("\nBatteries");
    println!("---------");

    let part = DowntubeBattery::standard_48v().generate();
    export_part("ref_battery_downtube", &part, "battery_downtube")?;
    println!("  Downtube battery");

    let part = CustomBattery::bvr1_pack().generate();
    export_part("ref_battery_custom", &part, "battery_custom")?;
    println!("  Custom battery pack");

    // =========================================================================
    // SCALE REFERENCES
    // =========================================================================
    println!("\nScale References");
    println!("----------------");

    let part = Banana::generate();
    export_part("ref_banana", &part, "banana")?;
    println!("  Banana (180mm)");

    let part = Human::generate();
    export_part("ref_human", &part, "human")?;
    println!("  Human (1750mm)");

    // =========================================================================
    // COMPLETE ASSEMBLIES
    // =========================================================================
    println!("\nComplete Assemblies");
    println!("-------------------");

    let bvr0 = BVR0Assembly::default_bvr0().generate();
    export_part("bvr0_assembly", &bvr0, "bvr0_assembly")?;
    println!("  BVR0 (prototype)");

    let bvr1 = BVR1Assembly::default_bvr1().generate();
    export_part("bvr1_assembly", &bvr1, "bvr1_assembly")?;
    println!("  BVR1 (production)");

    // =========================================================================
    // ROBOT USD FOR ISAAC SIM
    // =========================================================================
    println!("\nIsaac Sim Robot USD");
    println!("-------------------");

    // Generate complete robot with articulations
    let wheel_configs = vec![
        (UUMotor::kn6104().generate(), WheelConfig {
            name: "WheelFL".to_string(),
            position: [-0.28, 0.082, 0.28],
            axis: [1.0, 0.0, 0.0],
            max_velocity: 100.0,
            max_torque: 50.0,
        }),
        (UUMotor::kn6104().generate(), WheelConfig {
            name: "WheelFR".to_string(),
            position: [0.28, 0.082, 0.28],
            axis: [1.0, 0.0, 0.0],
            max_velocity: 100.0,
            max_torque: 50.0,
        }),
        (UUMotor::kn6104().generate(), WheelConfig {
            name: "WheelRL".to_string(),
            position: [-0.28, 0.082, -0.28],
            axis: [1.0, 0.0, 0.0],
            max_velocity: 100.0,
            max_torque: 50.0,
        }),
        (UUMotor::kn6104().generate(), WheelConfig {
            name: "WheelRR".to_string(),
            position: [0.28, 0.082, -0.28],
            axis: [1.0, 0.0, 0.0],
            max_velocity: 100.0,
            max_torque: 50.0,
        }),
    ];

    let robot_usd_path = usd_dir.join("bvr1_robot.usda");
    export_robot_usd(&bvr1, &wheel_configs, &materials, &robot_usd_path)?;
    usd_count += 1;
    println!("  bvr1_robot.usda (articulated)");

    // =========================================================================
    // SUMMARY
    // =========================================================================
    println!("\n════════════════════════════════════════════════════════════");
    println!("Export complete:");
    println!("  STL:  {} files -> exports/stl/", stl_count);
    #[cfg(feature = "gltf")]
    println!("  glTF: {} files -> exports/gltf/", gltf_count);
    println!("  USD:  {} files -> exports/usd/", usd_count);
    println!("════════════════════════════════════════════════════════════");

    Ok(())
}
