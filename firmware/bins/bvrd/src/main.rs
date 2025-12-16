//! bvrd — main daemon for the Base Vectoring Rover.

use anyhow::Result;
use can::vesc::Drivetrain;
use can::Bus;
use clap::Parser;
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use sim::SimBus;
use state::{Event, StateMachine};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch};
use tools::{protocol, Registry as ToolRegistry, ToolOutput};
use tracing::{error, info, warn};
use types::{Command, Mode, Pose, PowerStatus, Twist};
use ui::{Config as UiConfig, Dashboard};

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

    /// Enable simulation mode (no real hardware)
    #[arg(long)]
    sim: bool,

    /// Dashboard web UI port (0 to disable)
    #[arg(long, default_value = "8080")]
    ui_port: u16,
}

/// CAN interface abstraction for real or simulated hardware.
enum CanInterface {
    Real(Bus),
    Sim(Arc<Mutex<SimBus>>),
}

impl CanInterface {
    fn send(&self, frame: &can::Frame) -> Result<(), can::CanError> {
        match self {
            Self::Real(bus) => bus.send(frame),
            Self::Sim(sim) => {
                sim.lock().unwrap().process_tx(frame);
                Ok(())
            }
        }
    }

    fn recv(&self) -> Result<Option<can::Frame>, can::CanError> {
        match self {
            Self::Real(bus) => bus.recv(),
            Self::Sim(sim) => Ok(sim.lock().unwrap().recv()),
        }
    }

    fn tick(&self, dt: f64) {
        if let Self::Sim(sim) = self {
            sim.lock().unwrap().tick(dt);
        }
    }

    /// Get current pose from simulation (returns default for real hardware).
    fn pose(&self) -> Pose {
        match self {
            Self::Real(_) => Pose::default(), // TODO: get from odometry/GPS
            Self::Sim(sim) => {
                let (x, y, theta) = sim.lock().unwrap().position();
                Pose { x, y, theta }
            }
        }
    }
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

    if args.sim {
        info!("Starting bvrd in SIMULATION mode");
    } else {
        info!(config = ?args.config, can = %args.can_interface, "Starting bvrd");
    }

    // Initialize CAN interface
    let vesc_ids: [u8; 4] = args.vesc_ids.try_into().expect("Need exactly 4 VESC IDs");
    let can_interface = if args.sim {
        info!("Using simulated CAN bus");
        let sim_bus = SimBus::new(vesc_ids);
        CanInterface::Sim(Arc::new(Mutex::new(sim_bus)))
    } else {
        info!(interface = %args.can_interface, "Opening CAN bus");
        CanInterface::Real(Bus::open(&args.can_interface)?)
    };

    // Initialize drivetrain
    let drivetrain = Drivetrain::new(vesc_ids, args.pole_pairs);

    // Chassis parameters (TODO: load from config)
    let chassis = ChassisParams::new(0.165, 0.55, 0.55);
    let limits = Limits::default();

    // Shared state
    let mut state_machine = StateMachine::new();

    // In sim mode, auto-enable to Idle (no safety concern)
    if args.sim {
        state_machine.transition(state::Event::Enable);
        info!("Sim mode: auto-enabled to Idle");
    }

    let shared = Arc::new(Mutex::new(SharedState {
        state_machine,
        commanded_twist: Twist::default(),
        drivetrain,
        tool_registry: ToolRegistry::new(),
    }));

    // Channels
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);
    let initial_telemetry = Telemetry {
        timestamp_ms: 0,
        mode: Mode::Disabled,
        pose: Pose::default(),
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
    let teleop = TeleopServer::new(teleop_config, cmd_tx, telemetry_rx.clone());

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Spawn dashboard if enabled
    if args.ui_port > 0 {
        let ui_config = UiConfig { port: args.ui_port };
        let dashboard = Dashboard::new(ui_config, telemetry_rx.clone());

        tokio::spawn(async move {
            if let Err(e) = dashboard.run().await {
                error!(?e, "Dashboard server error");
            }
        });
    }

    // Control loop setup
    let mixer = DiffDriveMixer::new(chassis);
    let mut rate_limiter = RateLimiter::new(limits);
    let mut watchdog = Watchdog::new(Duration::from_millis(250));

    let control_period = Duration::from_millis(10); // 100 Hz
    let mut last_tick = Instant::now();

    // Current tool command (from teleop)
    let mut tool_command = types::ToolCommand::default();

    info!("Entering control loop");
    info!("Dashboard available at http://localhost:{}", args.ui_port);
    info!("Send commands to UDP port 4840");

    loop {
        // Wait for next tick
        let elapsed = last_tick.elapsed();
        if elapsed < control_period {
            std::thread::sleep(control_period - elapsed);
        }
        let dt = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();

        // Tick simulation if in sim mode
        can_interface.tick(dt);

        // Read CAN frames
        while let Ok(Some(frame)) = can_interface.recv() {
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

                    // Auto-transition to teleop when receiving commands
                    match state.state_machine.mode() {
                        Mode::Disabled => {
                            state.state_machine.transition(Event::Enable);
                            state.state_machine.transition(Event::TeleopCommand);
                        }
                        Mode::Idle => {
                            state.state_machine.transition(Event::TeleopCommand);
                        }
                        _ => {}
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
        let vesc_cmds = state.drivetrain.build_rpm_commands(wheel_rpms);
        for frame in vesc_cmds {
            if let Err(e) = can_interface.send(&frame) {
                error!(?e, "Failed to send RPM to drivetrain");
            }
        }

        // Update active tool
        if let Some(tool) = state.tool_registry.active_mut() {
            let output = tool.update(&tool_command);

            // Send tool command
            let slot = tool.info().slot;
            let frame = match output {
                ToolOutput::SetAxis(axis) => Some(protocol::build_command(slot, axis, 0.0)),
                ToolOutput::SetMotor(motor) => Some(protocol::build_command(slot, 0.0, motor)),
                ToolOutput::SetBoth { axis, motor } => {
                    Some(protocol::build_command(slot, axis, motor))
                }
                ToolOutput::None => None,
            };
            if let Some(f) = frame {
                let _ = can_interface.send(&f);
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

        // Get pose (drop the lock first to avoid holding it during telemetry send)
        drop(state);
        let pose = can_interface.pose();

        let state = shared.lock().unwrap();
        let telemetry = Telemetry {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            mode: state.state_machine.mode(),
            pose,
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
