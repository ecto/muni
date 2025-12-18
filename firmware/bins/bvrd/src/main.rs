//! bvrd — main daemon for the Base Vectoring Rover.

use anyhow::Result;
use camera::Config as CameraConfig;
use can::vesc::Drivetrain;
use can::Bus;
use clap::Parser;
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use gps::{Config as GpsConfig, GpsReader, GpsState};
use localization::{PoseEstimator, WheelOdometry};
use sim::SimBus;
use state::{Event, StateMachine};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teleop::video::{VideoConfig, VideoFrame, VideoServer};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch};
use tools::{protocol, Registry as ToolRegistry, ToolOutput};
use tracing::{debug, error, info, warn};
use types::{Command, Mode, Pose, PowerStatus, Twist};
use ui::{Config as UiConfig, Dashboard};

#[derive(Parser)]
#[command(name = "bvrd", about = "Base Vectoring Rover daemon")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/bvr.toml")]
    config: PathBuf,

    /// CAN interface (e.g., can0). Overrides config file.
    #[arg(long)]
    can_interface: Option<String>,

    /// VESC IDs [FL, FR, RL, RR]
    #[arg(long, default_values_t = vec![0, 1, 2, 3])]
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

    /// Disable camera auto-detection
    #[arg(long)]
    no_camera: bool,

    /// Camera capture resolution (e.g., "1280x720")
    #[arg(long)]
    camera_resolution: Option<String>,

    /// Camera FPS
    #[arg(long, default_value = "30")]
    camera_fps: u32,

    /// GPS serial port (e.g., "/dev/ttyUSB0", "/dev/ttyACM0")
    #[arg(long)]
    gps_port: Option<String>,

    /// GPS baud rate
    #[arg(long, default_value = "9600")]
    gps_baud: u32,
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

    // CAN interface: CLI arg > default "can0"
    // TODO: load from config file
    let can_iface = args.can_interface.as_deref().unwrap_or("can0");

    if args.sim {
        info!("Starting bvrd in SIMULATION mode");
    } else {
        info!(config = ?args.config, can = %can_iface, "Starting bvrd");
    }

    // Initialize CAN interface
    let vesc_ids: [u8; 4] = args.vesc_ids.try_into().expect("Need exactly 4 VESC IDs");
    let can_interface = if args.sim {
        info!("Using simulated CAN bus");
        let sim_bus = SimBus::new(vesc_ids);
        CanInterface::Sim(Arc::new(Mutex::new(sim_bus)))
    } else {
        info!(interface = %can_iface, "Opening CAN bus");
        CanInterface::Real(Bus::open(can_iface)?)
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

    // Get initial mode before moving state_machine
    let initial_mode = state_machine.mode();

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
        mode: initial_mode,
        pose: Pose::default(),
        power: PowerStatus {
            battery_voltage: 48.0,  // Simulated full battery
            system_current: 0.0,
        },
        velocity: Twist::default(),
        motor_temps: [25.0; 4],  // Ambient temp
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

    // Auto-detect and start cameras (unless disabled)
    if !args.no_camera {
        // Parse resolution if provided
        let (width, height) = if let Some(res) = &args.camera_resolution {
            let parts: Vec<&str> = res.split('x').collect();
            if parts.len() == 2 {
                (
                    parts[0].parse().unwrap_or(640),
                    parts[1].parse().unwrap_or(480),
                )
            } else {
                (640, 480)
            }
        } else {
            (640, 480)
        };

        let camera_config = CameraConfig {
            width,
            height,
            fps: args.camera_fps,
            jpeg_quality: 60,
        };

        // Auto-detect cameras
        let cameras = camera::detect_cameras();
        if cameras.is_empty() {
            info!("No cameras detected");
        } else {
            info!(count = cameras.len(), "Detected cameras");
            for cam in &cameras {
                info!(name = %cam.name, "  - {:?}", cam.camera_type);
            }

            // Start capture on first available camera
            match camera::spawn_capture(&cameras[0], camera_config) {
                Ok((frame_rx, _camera_handle)) => {
                    info!(
                        camera = %cameras[0].name,
                        "{}x{} @ {}fps",
                        width,
                        height,
                        args.camera_fps
                    );

                    // Create video frame channel
                    let (video_tx, video_rx) = watch::channel(None);

                    // Spawn task to bridge sync camera frames to async video server
                    std::thread::spawn(move || {
                        while let Ok(frame) = frame_rx.recv() {
                            let video_frame = VideoFrame {
                                data: frame.data,
                                width: frame.width,
                                height: frame.height,
                                sequence: frame.sequence,
                                timestamp_ms: frame.timestamp_ms,
                            };
                            if video_tx.send(Some(video_frame)).is_err() {
                                break;
                            }
                        }
                    });

                    // Spawn video server
                    let video_config = VideoConfig::default();
                    let video_server = VideoServer::new(video_config.clone(), video_rx);
                    info!(port = video_config.port, "Video server starting");

                    tokio::spawn(async move {
                        if let Err(e) = video_server.run().await {
                            error!(?e, "Video server error");
                        }
                    });
                }
                Err(e) => {
                    warn!(?e, "Failed to start camera - continuing without video");
                }
            }
        }
    }

    // Initialize localization
    let mut odometry = WheelOdometry::new(chassis.clone(), args.pole_pairs);
    let mut pose_estimator = PoseEstimator::new();

    // GPS state channel (updated by GPS reader thread)
    let (gps_tx, mut gps_rx) = watch::channel(GpsState::default());

    // Start GPS reader if port specified
    if let Some(ref port) = args.gps_port {
        let gps_config = GpsConfig {
            port: port.clone(),
            baud_rate: args.gps_baud,
        };
        let gps_reader = GpsReader::new(gps_config);
        match gps_reader.spawn(gps_tx) {
            Ok(_handle) => {
                info!(port = %port, baud = args.gps_baud, "GPS reader started");
            }
            Err(e) => {
                warn!(?e, "Failed to start GPS reader - continuing without GPS");
            }
        }
    }

    // Control loop setup
    let mixer = DiffDriveMixer::new(chassis);
    let mut rate_limiter = RateLimiter::new(limits);
    let mut watchdog = Watchdog::new(Duration::from_millis(500)); // Allow for network jitter over Tailscale

    let control_period = Duration::from_millis(10); // 100 Hz
    let mut last_tick = Instant::now();

    // Smoothing filter for commanded twist (reduces jitter from network/input noise)
    // Alpha = 0.15 gives ~7 cycle time constant (~70ms smoothing at 100Hz)
    let smoothing_alpha = 0.15;
    let mut smoothed_twist = Twist::default();

    // Current tool command (from teleop)
    let mut tool_command = types::ToolCommand::default();

    info!("Entering control loop");
    info!("Dashboard available at http://localhost:{}", args.ui_port);
    info!("Send commands to UDP port 4840");
    if args.gps_port.is_some() {
        info!("GPS enabled");
    }

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
                    if twist.angular.abs() > 0.1 {
                        info!(linear = twist.linear, angular = twist.angular, "Twist command with angular");
                    }
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
                    smoothed_twist = Twist::default();
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
            smoothed_twist = Twist::default();
            rate_limiter.reset();
        }

        // Compute motor outputs
        let target_twist = if state.state_machine.is_driving() {
            state.commanded_twist
        } else {
            Twist::default()
        };

        // Apply exponential smoothing to reduce jitter from network/input noise
        smoothed_twist.linear += smoothing_alpha * (target_twist.linear - smoothed_twist.linear);
        smoothed_twist.angular += smoothing_alpha * (target_twist.angular - smoothed_twist.angular);

        // Rate limit the smoothed twist
        let mut twist = rate_limiter.limit(smoothed_twist);

        // Boost angular for skid steering (requires more torque than forward motion)
        twist.angular *= 2.5;

        let wheel_vels = mixer.mix(twist);
        let wheel_rpms = mixer.to_rpm(&wheel_vels);

        // Log wheel commands when turning (left != right)
        if (wheel_rpms[0] - wheel_rpms[1]).abs() > 1.0 {
            info!(
                fl = wheel_rpms[0] as i32,
                fr = wheel_rpms[1] as i32,
                rl = wheel_rpms[2] as i32,
                rr = wheel_rpms[3] as i32,
                "Wheel RPMs (turning)"
            );
        }

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

        // Update wheel odometry from VESC tachometers
        let tach: [i32; 4] = [
            vesc_states[0].status5.tachometer,
            vesc_states[1].status5.tachometer,
            vesc_states[2].status5.tachometer,
            vesc_states[3].status5.tachometer,
        ];
        let (dx, dy, dtheta) = odometry.update(tach);

        // Update pose estimator with odometry
        pose_estimator.update_odometry(dx, dy, dtheta);

        // Check for GPS updates
        if gps_rx.has_changed().unwrap_or(false) {
            let gps_state = gps_rx.borrow_and_update();
            if let Some(ref coord) = gps_state.coord {
                pose_estimator.update_gps(coord);
                debug!(
                    lat = coord.lat,
                    lon = coord.lon,
                    sats = gps_state.satellites,
                    "GPS update"
                );
            }
        }

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

        // Get pose from estimator (or sim ground truth in sim mode for comparison)
        drop(state);
        let pose = if args.sim {
            // In sim mode, use simulation ground truth for accurate feedback
            can_interface.pose()
        } else {
            // In real mode, use the pose estimator
            pose_estimator.pose()
        };

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

