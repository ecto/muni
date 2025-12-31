//! Muni CLI - unified command-line interface for Muni robotics
//!
//! Usage:
//!   muni rover drive --linear 0.5
//!   muni rover scan --interface can0
//!   muni gps monitor --port /dev/ttyUSB0
//!   muni gps configure-base --port /dev/ttyUSB0

mod gps;
mod rover;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "muni")]
#[command(about = "Muni robotics CLI: rover control, GPS configuration, and diagnostics")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rover control and diagnostics
    #[command(subcommand)]
    Rover(rover::RoverCommands),

    /// GPS/GNSS receiver configuration and monitoring
    #[command(subcommand)]
    Gps(gps::GpsCommands),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Rover(cmd) => rover::run(cmd).await,
        Commands::Gps(cmd) => gps::run(cmd).await,
    }
}
