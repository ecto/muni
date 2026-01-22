//! Test binary for RRD extraction
//!
//! Usage: cargo run --bin test_extract -- /path/to/file.rrd

use std::path::PathBuf;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-rrd-file> [output-dir]", args[0]);
        std::process::exit(1);
    }

    let rrd_path = PathBuf::from(&args[1]);
    let output_dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        rrd_path.with_extension("extracted")
    };

    println!("Input:  {}", rrd_path.display());
    println!("Output: {}", output_dir.display());
    println!();

    // Extract from RRD
    match mapper::rrd_extractor::extract_from_rrd(&rrd_path) {
        Ok(result) => {
            println!("Extraction successful!");
            println!("  Poses:        {}", result.poses.len());
            println!("  LiDAR frames: {}", result.lidar_frames.len());
            println!("  Camera frames: {}", result.camera_frames.len());
            println!("  GPS bounds:   {:?}", result.gps_bounds);
            println!("  Session ID:   {:?}", result.session_id);
            println!("  Rover ID:     {:?}", result.rover_id);
            println!();

            // Write extracted data
            match mapper::rrd_extractor::write_extracted_data(&result, &output_dir) {
                Ok(()) => {
                    println!("Data written to: {}", output_dir.display());
                }
                Err(e) => {
                    eprintln!("Failed to write data: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Extraction failed: {}", e);
            std::process::exit(1);
        }
    }
}
