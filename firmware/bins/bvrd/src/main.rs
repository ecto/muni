//! bvrd — main daemon for the Base Vectoring Rover.

use anyhow::Result;
use can::vesc::Drivetrain;
use can::Bus;
use clap::Parser;
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use state::{Event, StateMachine};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch};
use tools::{protocol, Registry as ToolRegistry, ToolOutput};
use tracing::{error, info, warn};
use types::{Command, Mode, PowerStatus, Twist};

#[derive(Parser)]
#[command(name = "bvrd", about = "Base Vectoring Rover daemon")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/bvr.toml")]
    config: PathBuf,

    /// CAN interface (e.g., can0)
    #[arg(long, default_value = "can0")]
    can_interface: String,

    /// VESC IDs [FL, FR, RL, RR]
    #[arg(long, default_values_t = vec![1, 2, 3, 4])]
    vesc_ids: Vec<u8>,

    /// Motor pole pairs
    #[arg(long, default_value = "15")]
    pole_pairs: u8,
}

/// Shared state between threads.
struct SharedState {
    state_machine: StateMachine,
    commanded_twist: Twist,
    drivetrain: Drivetrain,
    tool_registry: ToolRegistry,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("bvrd=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    info!(config = ?args.config, can = %args.can_interface, "Starting bvrd");

    // Open CAN bus
    let can_bus = Arc::new(Bus::open(&args.can_interface)?);
    info!(interface = %args.can_interface, "CAN bus opened");

    // Initialize drivetrain
    let vesc_ids: [u8; 4] = args.vesc_ids.try_into().expect("Need exactly 4 VESC IDs");
    let drivetrain = Drivetrain::new(vesc_ids, args.pole_pairs);

    // Chassis parameters (TODO: load from config)
    let chassis = ChassisParams::new(0.165, 0.55, 0.55);
    let limits = Limits::default();

    // Shared state
    let shared = Arc::new(Mutex::new(SharedState {
        state_machine: StateMachine::new(),
        commanded_twist: Twist::default(),
        drivetrain,
        tool_registry: ToolRegistry::new(),
    }));

    // Channels
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);
    let initial_telemetry = Telemetry {
        timestamp_ms: 0,
        mode: Mode::Disabled,
        power: PowerStatus::default(),
        velocity: Twist::default(),
        motor_temps: [0.0; 4],
        motor_currents: [0.0; 4],
        active_tool: None,
        tool_status: None,
    };
    let (telemetry_tx, telemetry_rx) = watch::channel(initial_telemetry);

    // Spawn teleop server
    let teleop_config = TeleopConfig::default();
    let teleop = TeleopServer::new(teleop_config, cmd_tx, telemetry_rx);

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Control loop setup
    let mixer = DiffDriveMixer::new(chassis);
    let mut rate_limiter = RateLimiter::new(limits);
    let mut watchdog = Watchdog::new(Duration::from_millis(250));

    let control_period = Duration::from_millis(10); // 100 Hz
    let mut last_tick = Instant::now();

    // Current tool command (from teleop)
    let mut tool_command = types::ToolCommand::default();

    info!("Entering control loop");

    loop {
        // Wait for next tick
        let elapsed = last_tick.elapsed();
        if elapsed < control_period {
            std::thread::sleep(control_period - elapsed);
        }
        last_tick = Instant::now();

        // Read CAN frames
        while let Ok(Some(frame)) = can_bus.recv() {
            let mut state = shared.lock().unwrap();
            state.drivetrain.process_frame(&frame);
            state.tool_registry.process_frame(&frame);
        }

        // Process incoming commands (non-blocking)
        while let Ok(cmd) = cmd_rx.try_recv() {
            let mut state = shared.lock().unwrap();

            match cmd {
                Command::Twist(twist) => {
                    watchdog.feed();
                    state.commanded_twist = twist;

                    if state.state_machine.mode() == Mode::Idle {
                        state.state_machine.transition(Event::TeleopCommand);
                    }
                }
                Command::EStop => {
                    warn!("E-Stop command received");
                    state.state_machine.transition(Event::EStop);
                    rate_limiter.reset();
                }
                Command::SetMode(mode) => {
                    let event = match mode {
                        Mode::Disabled => Event::Disable,
                        Mode::Idle => Event::Enable,
                        Mode::Teleop => Event::TeleopCommand,
                        Mode::Autonomous => Event::AutonomousRequest,
                        Mode::EStop => Event::EStop,
                        _ => continue,
                    };
                    state.state_machine.transition(event);
                }
                Command::Heartbeat => {
                    watchdog.feed();
                }
                Command::Tool(tc) => {
                    watchdog.feed();
                    tool_command = tc;
                }
            }
        }

        // Check watchdog
        let mut state = shared.lock().unwrap();
        if watchdog.is_timed_out() && state.state_machine.is_driving() {
            warn!("Command watchdog timeout");
            state.state_machine.transition(Event::CommandTimeout);
            state.commanded_twist = Twist::default();
            rate_limiter.reset();
        }

        // Compute motor outputs
        let twist = if state.state_machine.is_driving() {
            rate_limiter.limit(state.commanded_twist)
        } else {
            Twist::default()
        };

        let wheel_vels = mixer.mix(twist);
        let wheel_rpms = mixer.to_rpm(&wheel_vels);

        // Send to VESCs
        if let Err(e) = state.drivetrain.set_rpm(&can_bus, wheel_rpms) {
            error!(?e, "Failed to send RPM to drivetrain");
        }

        // Update active tool
        if let Some(tool) = state.tool_registry.active_mut() {
            let output = tool.update(&tool_command);

            // Send tool command
            let slot = tool.info().slot;
            match output {
                ToolOutput::SetAxis(axis) => {
                    let _ = protocol::send_command(&can_bus, slot, axis, 0.0);
                }
                ToolOutput::SetMotor(motor) => {
                    let _ = protocol::send_command(&can_bus, slot, 0.0, motor);
                }
                ToolOutput::SetBoth { axis, motor } => {
                    let _ = protocol::send_command(&can_bus, slot, axis, motor);
                }
                ToolOutput::None => {}
            }
        }

        // Build telemetry
        let vesc_states = state.drivetrain.states();
        let motor_temps: [f32; 4] = [
            vesc_states[0].status4.temp_motor,
            vesc_states[1].status4.temp_motor,
            vesc_states[2].status4.temp_motor,
            vesc_states[3].status4.temp_motor,
        ];
        let motor_currents: [f32; 4] = [
            vesc_states[0].status.current,
            vesc_states[1].status.current,
            vesc_states[2].status.current,
            vesc_states[3].status.current,
        ];

        let (active_tool, tool_status) = if let Some(tool) = state.tool_registry.active() {
            let status = tool.status();
            (
                Some(tool.info().name.to_string()),
                Some(teleop::ToolStatus {
                    name: status.name.to_string(),
                    position: status.position,
                    active: status.active,
                    current: status.current,
                }),
            )
        } else {
            (None, None)
        };

        let telemetry = Telemetry {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            mode: state.state_machine.mode(),
            power: PowerStatus {
                battery_voltage: state.drivetrain.battery_voltage() as f64,
                system_current: motor_currents.iter().sum::<f32>() as f64,
            },
            velocity: twist,
            motor_temps,
            motor_currents,
            active_tool,
            tool_status,
        };

        let _ = telemetry_tx.send(telemetry);
    }
}
