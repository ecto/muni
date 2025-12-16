//! bvr-cli — command-line tool for debugging and controlling the rover.

use anyhow::Result;
use bvr_teleop::send_twist;
use bvr_types::Twist;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bvr-cli", about = "BVR command-line interface")]
struct Args {
    /// Address of the rover (host:port)
    #[arg(short, long, default_value = "127.0.0.1:4840")]
    address: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a velocity command
    Drive {
        /// Linear velocity (m/s)
        #[arg(short, long, default_value = "0.0")]
        linear: f64,
        /// Angular velocity (rad/s)
        #[arg(short, long, default_value = "0.0")]
        angular: f64,
    },
    /// Send e-stop
    Estop,
    /// Monitor telemetry
    Monitor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Drive { linear, angular } => {
            let twist = Twist { linear, angular };
            println!("Sending twist: linear={:.2} m/s, angular={:.2} rad/s", linear, angular);
            send_twist(&args.address, twist).await?;
        }
        Commands::Estop => {
            println!("Sending E-STOP");
            // TODO: Implement e-stop send
        }
        Commands::Monitor => {
            println!("Monitoring telemetry from {}...", args.address);
            // TODO: Implement telemetry monitoring
        }
    }

    Ok(())
}

