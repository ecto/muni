//! Generate all BVR1 CAD parts

use anyhow::Result;
use bvr_cad::parts::{ElectronicsPlate, MotorMount, SensorMount, WheelSpacer};
use std::path::Path;

fn main() -> Result<()> {
    println!("BVR1 Parts Generator");
    println!("====================\n");

    let export_dir = Path::new("exports");
    std::fs::create_dir_all(export_dir)?;

    // Motor mount
    println!("Motor Mount (8\" hub motor)");
    println!("--------------------------");
    let mount = MotorMount::hub_motor_8in();

    let part = mount.generate();
    let path = export_dir.join("motor_mount_8in.stl");
    part.write_stl(&path)?;
    println!("  L-bracket:  {}", path.display());

    let part = mount.generate_flat_plate();
    let path = export_dir.join("motor_mount_plate_8in.stl");
    part.write_stl(&path)?;
    println!("  Flat plate: {}", path.display());

    let part = mount.generate_tabs();
    let path = export_dir.join("motor_mount_tab_8in.stl");
    part.write_stl(&path)?;
    println!("  Tab:        {}", path.display());

    // Wheel spacer
    println!("\nWheel Spacer");
    println!("------------");
    let spacer = WheelSpacer::hub_motor_8in();

    let part = spacer.generate();
    let path = export_dir.join("wheel_spacer_8in.stl");
    part.write_stl(&path)?;
    println!("  Round:  {}", path.display());

    let part = spacer.generate_flat();
    let path = export_dir.join("wheel_spacer_flat_8in.stl");
    part.write_stl(&path)?;
    println!("  Square: {}", path.display());

    // Electronics plate
    println!("\nElectronics Plate");
    println!("-----------------");
    let plate = ElectronicsPlate::default_bvr1();

    let part = plate.generate();
    let path = export_dir.join("electronics_plate.stl");
    part.write_stl(&path)?;
    println!("  Full:   {}", path.display());

    let part = plate.generate_simple();
    let path = export_dir.join("electronics_plate_simple.stl");
    part.write_stl(&path)?;
    println!("  Simple: {}", path.display());

    // Sensor mount
    println!("\nSensor Mount (1\" tube clamp)");
    println!("----------------------------");
    let sensor = SensorMount::default_bvr1();

    let part = sensor.generate();
    let path = export_dir.join("sensor_mount.stl");
    part.write_stl(&path)?;
    println!("  Bracket:    {}", path.display());

    let part = sensor.generate_base_plate();
    let path = export_dir.join("sensor_mount_base.stl");
    part.write_stl(&path)?;
    println!("  Base plate: {}", path.display());

    println!("\n✓ All {} parts exported to {}/", 10, export_dir.display());

    Ok(())
}
