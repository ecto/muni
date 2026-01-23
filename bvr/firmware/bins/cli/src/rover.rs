//! Rover control commands

use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use teleop::{send_estop, send_set_mode, send_twist};
use types::{Mode, Twist};

/// Mode argument for SetMode command
#[derive(Clone, Copy, ValueEnum)]
pub enum ModeArg {
    Disabled,
    Idle,
    Teleop,
    Autonomous,
}

impl From<ModeArg> for Mode {
    fn from(arg: ModeArg) -> Self {
        match arg {
            ModeArg::Disabled => Mode::Disabled,
            ModeArg::Idle => Mode::Idle,
            ModeArg::Teleop => Mode::Teleop,
            ModeArg::Autonomous => Mode::Autonomous,
        }
    }
}

#[derive(Subcommand)]
pub enum RoverCommands {
    /// Send a velocity command
    Drive {
        /// Rover address (host:port)
        #[arg(short, long, default_value = "127.0.0.1:4840")]
        address: String,
        /// Linear velocity (m/s)
        #[arg(short, long, default_value = "0.0")]
        linear: f64,
        /// Angular velocity (rad/s)
        #[arg(short = 'w', long, default_value = "0.0")]
        angular: f64,
    },
    /// Send e-stop
    Estop {
        /// Rover address (host:port)
        #[arg(short, long, default_value = "127.0.0.1:4840")]
        address: String,
    },
    /// Set rover mode (Disabled, Idle, Teleop, Autonomous)
    SetMode {
        /// Rover address (host:port)
        #[arg(short, long, default_value = "127.0.0.1:4840")]
        address: String,
        /// Mode to set
        #[arg(value_enum)]
        mode: ModeArg,
    },
    /// Scan CAN bus for VESCs
    Scan {
        /// CAN interface
        #[arg(short, long, default_value = "can0")]
        interface: String,
        /// Scan duration in seconds
        #[arg(short, long, default_value = "2")]
        duration: u64,
    },
}

pub async fn run(cmd: RoverCommands) -> Result<()> {
    match cmd {
        RoverCommands::Drive {
            address,
            linear,
            angular,
        } => {
            let twist = Twist {
                linear,
                angular,
                boost: false,
            };
            println!(
                "Sending twist: linear={:.2} m/s, angular={:.2} rad/s",
                linear, angular
            );
            send_twist(&address, twist).await?;
            println!("Sent.");
        }
        RoverCommands::Estop { address } => {
            println!("Sending E-STOP to {}", address);
            send_estop(&address).await?;
            println!("E-STOP sent.");
        }
        RoverCommands::SetMode { address, mode } => {
            let mode_type: Mode = mode.into();
            println!("Setting mode to {:?} on {}", mode_type, address);
            send_set_mode(&address, mode_type).await?;
            println!("SetMode sent.");
        }
        RoverCommands::Scan {
            interface,
            duration,
        } => {
            scan_can(&interface, duration)?;
        }
    }
    Ok(())
}

/// VESC info collected during scan
#[derive(Default)]
struct VescInfo {
    erpm: i32,
    current: f32,
    duty: f32,
    temp_fet: f32,
    temp_motor: f32,
    voltage_in: f32,
    #[allow(dead_code)]
    msg_count: u32,
}

fn scan_can(interface: &str, duration_secs: u64) -> Result<()> {
    println!("Scanning {} for {} seconds...\n", interface, duration_secs);

    let bus = can::Bus::open(interface)?;
    let mut vescs: HashMap<u8, VescInfo> = HashMap::new();
    let start = Instant::now();
    let duration = Duration::from_secs(duration_secs);

    while start.elapsed() < duration {
        if let Ok(Some(frame)) = bus.recv() {
            if !frame.extended {
                continue;
            }

            let vesc_id = (frame.id & 0xFF) as u8;
            let cmd = (frame.id >> 8) as u8;
            let info = vescs.entry(vesc_id).or_default();
            info.msg_count += 1;

            match cmd {
                9 if frame.data.len() >= 8 => {
                    // STATUS1: ERPM, current, duty
                    info.erpm = i32::from_be_bytes([
                        frame.data[0],
                        frame.data[1],
                        frame.data[2],
                        frame.data[3],
                    ]);
                    info.current =
                        i16::from_be_bytes([frame.data[4], frame.data[5]]) as f32 / 10.0;
                    info.duty =
                        i16::from_be_bytes([frame.data[6], frame.data[7]]) as f32 / 1000.0;
                }
                16 if frame.data.len() >= 6 => {
                    // STATUS4: temps, current_in
                    info.temp_fet =
                        i16::from_be_bytes([frame.data[0], frame.data[1]]) as f32 / 10.0;
                    info.temp_motor =
                        i16::from_be_bytes([frame.data[2], frame.data[3]]) as f32 / 10.0;
                }
                27 if frame.data.len() >= 6 => {
                    // STATUS5: tachometer, voltage
                    info.voltage_in =
                        i16::from_be_bytes([frame.data[4], frame.data[5]]) as f32 / 10.0;
                }
                _ => {}
            }
        }
    }

    if vescs.is_empty() {
        println!("No VESCs found on {}", interface);
        println!("\nTroubleshooting:");
        println!("  - Check CAN wiring (CANH/CANL)");
        println!("  - Verify VESCs are powered");
        println!("  - Confirm CAN is enabled in VESC Tool");
        println!("  - Check baud rate matches (500kbps)");
        return Ok(());
    }

    // Sort by ID
    let mut ids: Vec<_> = vescs.keys().copied().collect();
    ids.sort();

    println!("┌─────────┬────────┬─────────┬─────────┬──────────┬──────────┬──────────┐");
    println!("│   ID    │  ERPM  │ Current │  Duty   │ FET Temp │ Mot Temp │ Voltage  │");
    println!("├─────────┼────────┼─────────┼─────────┼──────────┼──────────┼──────────┤");

    for id in &ids {
        let info = &vescs[id];
        println!(
            "│ {:3} (0x{:02X}) │ {:6} │ {:5.1} A │ {:5.1}%  │  {:5.1}°C │  {:5.1}°C │  {:5.1} V │",
            id,
            id,
            info.erpm,
            info.current,
            info.duty * 100.0,
            info.temp_fet,
            info.temp_motor,
            info.voltage_in
        );
    }

    println!("└─────────┴────────┴─────────┴─────────┴──────────┴──────────┴──────────┘");
    println!("\nFound {} VESC(s)", ids.len());

    if ids.len() < 4 {
        println!(
            "\n⚠ Expected 4 VESCs, only found {}. Missing VESC(s) may have:",
            ids.len()
        );
        println!("  - Disconnected CAN wiring");
        println!("  - CAN disabled in settings");
        println!("  - Power issue");
    }

    // Suggest ID cleanup if needed
    let sequential: Vec<u8> = (0..ids.len() as u8).collect();
    if ids != sequential {
        println!(
            "\n💡 Tip: Consider reassigning IDs to 0-{} for cleaner code",
            ids.len() - 1
        );
    }

    Ok(())
}
