//! Generate motor mount STL files

use anyhow::Result;
use bvr_cad::parts::MotorMount;
use std::path::Path;

fn main() -> Result<()> {
    println!("BVR1 Motor Mount Generator");
    println!("==========================\n");

    let export_dir = Path::new("exports");
    std::fs::create_dir_all(export_dir)?;

    // Generate 8" hub motor mount
    let mount = MotorMount::hub_motor_8in();

    // Full L-bracket mount
    println!("Generating full motor mount (L-bracket)...");
    let full_mount = mount.generate();
    let stl_path = export_dir.join("motor_mount_8in.stl");
    full_mount.write_stl(&stl_path)?;
    println!("  → Exported to: {}", stl_path.display());

    // Flat plate only (for laser cutting)
    println!("\nGenerating flat plate (for laser cutting)...");
    let flat_plate = mount.generate_flat_plate();
    let flat_path = export_dir.join("motor_mount_plate_8in.stl");
    flat_plate.write_stl(&flat_path)?;
    println!("  → Exported to: {}", flat_path.display());

    // Mounting tabs
    println!("\nGenerating mounting tab (for laser cutting)...");
    let tab = mount.generate_tabs();
    let tab_path = export_dir.join("motor_mount_tab_8in.stl");
    tab.write_stl(&tab_path)?;
    println!("  → Exported to: {}", tab_path.display());

    println!("\n✓ All parts exported successfully!");
    println!("\nMotor mount specifications:");
    println!("  - Designed for 8\" hub motor (200mm OD)");
    println!("  - 15mm axle hole");
    println!("  - 4x M6 motor mounting holes on 70mm bolt circle");
    println!("  - 6mm plate thickness (aluminum)");
    println!("  - M5 holes for 2020 extrusion mounting");

    Ok(())
}
