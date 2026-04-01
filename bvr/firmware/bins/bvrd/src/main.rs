//! bvrd — main daemon for the Base Vectoring Rover.

mod camera_init;
mod can_iface;
mod cli;
mod config;
mod depth_thread;
mod init;
mod logging;
mod run_loop;
mod state_types;
mod vesc_serial;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    let (ctx, _log_guard) = init::initialize(args).await?;

    let handle = std::thread::Builder::new()
        .name("control-loop".to_string())
        .spawn(move || run_loop::run(ctx))
        .expect("Failed to spawn control loop thread");

    handle.join().expect("Control loop thread panicked");
    Ok(())
}
