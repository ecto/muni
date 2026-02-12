//! CLI argument parsing for bvrd.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bvrd", about = "Base Vectoring Rover daemon")]
pub(crate) struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/bvr.toml")]
    pub config: PathBuf,

    /// CAN interface (e.g., can0). Overrides config file.
    #[arg(long)]
    pub can_interface: Option<String>,

    /// VESC IDs [FL, FR, RL, RR]
    #[arg(long, default_values_t = vec![0, 1, 2, 3])]
    pub vesc_ids: Vec<u8>,

    /// Motor pole pairs (32 poles = 16 pole pairs)
    #[arg(long, default_value = "16")]
    pub pole_pairs: u8,

    /// Enable simulation mode (no real hardware)
    #[arg(long)]
    pub sim: bool,

    /// Dashboard web UI port (0 to disable)
    #[arg(long, default_value = "8080")]
    pub ui_port: u16,

    /// WebRTC signaling port for teleop (0 to disable)
    #[arg(long, default_value = "4852")]
    pub rtc_port: u16,

    /// Disable camera auto-detection
    #[arg(long)]
    pub no_camera: bool,

    /// Camera capture resolution (e.g., "1280x720")
    #[arg(long)]
    pub camera_resolution: Option<String>,

    /// Camera FPS
    #[arg(long, default_value = "30")]
    pub camera_fps: u32,

    /// GPS serial port (e.g., "/dev/ttyUSB0", "/dev/ttyACM0")
    #[arg(long)]
    pub gps_port: Option<String>,

    /// GPS baud rate
    #[arg(long, default_value = "9600")]
    pub gps_baud: u32,

    /// Rover ID for logging and recording
    #[arg(long, default_value = "bvr-01")]
    pub rover_id: String,

    /// Disable telemetry recording
    #[arg(long)]
    pub no_recording: bool,

    /// Recording session directory
    #[arg(long, default_value = "/var/log/bvr/sessions")]
    pub recording_dir: PathBuf,

    /// Log directory for text logs
    #[arg(long, default_value = "/var/log/bvr")]
    pub log_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Disable metrics push to Depot (overrides config file)
    #[arg(long)]
    pub no_metrics: bool,

    /// Depot metrics endpoint - overrides config file (e.g., "192.168.1.100:8089")
    #[arg(long)]
    pub metrics_endpoint: Option<String>,

    /// Metrics push rate in Hz - overrides config file
    #[arg(long)]
    pub metrics_hz: Option<u32>,

    /// Disable discovery service registration (overrides config file)
    #[arg(long)]
    pub no_discovery: bool,

    /// Depot discovery endpoint - overrides config file (e.g., "192.168.1.100:4860")
    #[arg(long)]
    pub discovery_endpoint: Option<String>,

    /// Disable autonomous mode (policy loading)
    #[arg(long)]
    pub no_autonomous: bool,

    /// Path to policy file for autonomous mode
    #[arg(long)]
    pub policy: Option<PathBuf>,

    /// Directory containing policy files
    #[arg(long)]
    pub policy_dir: Option<PathBuf>,

    /// Autonomous mode goal position [x,y] (for testing)
    #[arg(long, value_parser = parse_goal)]
    pub goal: Option<[f64; 2]>,

    /// Disable dispatch service connection (overrides config file)
    #[arg(long)]
    pub no_dispatch: bool,

    /// Dispatch service endpoint - overrides config file (e.g., "ws://192.168.1.100:4890/ws")
    #[arg(long)]
    pub dispatch_endpoint: Option<String>,

    /// Connect to sim-bridge remote CAN (e.g., "tcp://127.0.0.1:4910")
    #[arg(long)]
    pub remote_can: Option<String>,

    /// VESC serial port for USB CAN gateway (e.g., "/dev/ttyACM0")
    #[arg(long)]
    pub vesc_serial: Option<String>,

    /// VESC CAN ID of the USB-connected gateway VESC (default: first vesc-id)
    #[arg(long)]
    pub vesc_gateway_id: Option<u8>,

    /// Enable tokio-console for runtime introspection (requires --features console)
    #[arg(long)]
    pub console: bool,
}

/// Parse goal position from "x,y" string.
fn parse_goal(s: &str) -> Result<[f64; 2], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("goal must be in format 'x,y'".to_string());
    }
    let x = parts[0].parse::<f64>().map_err(|e| e.to_string())?;
    let y = parts[1].parse::<f64>().map_err(|e| e.to_string())?;
    Ok([x, y])
}
