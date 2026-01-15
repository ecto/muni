//! bvrd — main daemon for the Base Vectoring Rover.

use anyhow::Result;
use camera::Config as CameraConfig;
use dispatch::{DispatchClient, DispatchEvent, TaskAssignment, Waypoint as DispatchWaypoint};
use can::vesc::Drivetrain;
use can::Bus;
use clap::Parser;
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use gps::{Config as GpsConfig, GpsReader, GpsState};
use lidar::{Config as LidarConfig, LidarReader};
use localization::{PoseEstimator, WheelOdometry};
use metrics::{Config as MetricsConfig, DiscoveryClient, DiscoveryConfig, MetricsPusher, MetricsSnapshot};
use policy::{NormalizationConfig, Policy, PolicyManager, PolicyObservation};
use recording::{Config as RecordingConfig, Recorder};
use serde::Deserialize;
use sim::SimBus;
use state::{Event, StateMachine};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use teleop::video::{VideoConfig, VideoFrame, VideoServer};
use teleop::video_ws::{WsVideoConfig, WsVideoServer};
use teleop::ws::{WsConfig, WsServer};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch};
use tools::{protocol, Registry as ToolRegistry, ToolOutput};
use tracing::{debug, error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use types::{Command, Mode, Pose, PowerStatus, Twist};
use ui::{Config as UiConfig, Dashboard};

/// Configuration file structure (bvr.toml).
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FileConfig {
    identity: IdentityConfig,
    chassis: ChassisFileConfig,
    metrics: MetricsFileConfig,
    discovery: DiscoveryFileConfig,
    autonomous: AutonomousFileConfig,
    dispatch: DispatchFileConfig,
    lidar: LidarFileConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct DispatchFileConfig {
    /// Enable dispatch service connection
    enabled: bool,
    /// Dispatch service endpoint (ws://host:port/ws)
    endpoint: String,
}

impl Default for DispatchFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "ws://depot.local:4890/ws".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct LidarFileConfig {
    /// Enable LiDAR sensor
    enabled: bool,
    /// Serial port for RPLidar A1
    port: String,
    /// Baud rate (always 115200 for RPLidar A1)
    baud_rate: u32,
}

impl Default for LidarFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ChassisFileConfig {
    /// Wheel diameter in meters
    wheel_diameter_m: f64,
    /// Distance between left and right wheels (track width)
    track_width_m: f64,
    /// Distance between front and rear axles
    wheelbase_m: f64,
}

impl Default for ChassisFileConfig {
    fn default() -> Self {
        Self {
            wheel_diameter_m: 0.165,
            track_width_m: 0.55,
            wheelbase_m: 0.55,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct AutonomousFileConfig {
    /// Enable autonomous mode support
    enabled: bool,
    /// Directory containing policy files
    policy_dir: String,
    /// Specific policy file to load (overrides default)
    policy_file: Option<String>,
    /// Maximum linear velocity in autonomous mode (m/s)
    max_linear_vel: f64,
    /// Maximum angular velocity in autonomous mode (rad/s)
    max_angular_vel: f64,
    /// Goal waypoint [x, y] in local frame (for testing)
    goal: Option<[f64; 2]>,
}

impl Default for AutonomousFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            policy_dir: "/var/lib/bvr/policies".to_string(),
            policy_file: None,
            max_linear_vel: 1.0,
            max_angular_vel: 1.5,
            goal: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct IdentityConfig {
    rover_id: String,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            rover_id: "bvr-01".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct MetricsFileConfig {
    enabled: bool,
    endpoint: String,
    interval_hz: u32,
}

impl Default for MetricsFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "depot.local:8089".to_string(),
            interval_hz: 1,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct DiscoveryFileConfig {
    enabled: bool,
    endpoint: String,
    rover_id: Option<String>,
    rover_name: Option<String>,
    ws_port: u16,
    ws_video_port: u16,
    heartbeat_secs: u32,
}

impl Default for DiscoveryFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "depot.local:4860".to_string(),
            rover_id: None,
            rover_name: None,
            ws_port: 4850,
            ws_video_port: 4851,
            heartbeat_secs: 2,
        }
    }
}

impl FileConfig {
    fn load(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: FileConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            warn!(path = %path.display(), "Config file not found, using defaults");
            Ok(FileConfig::default())
        }
    }
}

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

    /// Motor pole pairs (32 poles = 16 pole pairs)
    #[arg(long, default_value = "16")]
    pole_pairs: u8,

    /// Enable simulation mode (no real hardware)
    #[arg(long)]
    sim: bool,

    /// Dashboard web UI port (0 to disable)
    #[arg(long, default_value = "8080")]
    ui_port: u16,

    /// WebSocket teleop port for browser-based operators
    #[arg(long, default_value = "4850")]
    ws_port: u16,

    /// WebSocket video streaming port for browser-based operators
    #[arg(long, default_value = "4851")]
    ws_video_port: u16,

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

    /// Rover ID for logging and recording
    #[arg(long, default_value = "bvr-01")]
    rover_id: String,

    /// Disable telemetry recording
    #[arg(long)]
    no_recording: bool,

    /// Recording session directory
    #[arg(long, default_value = "/var/log/bvr/sessions")]
    recording_dir: PathBuf,

    /// Log directory for text logs
    #[arg(long, default_value = "/var/log/bvr")]
    log_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Disable metrics push to Depot (overrides config file)
    #[arg(long)]
    no_metrics: bool,

    /// Depot metrics endpoint - overrides config file (e.g., "192.168.1.100:8089")
    #[arg(long)]
    metrics_endpoint: Option<String>,

    /// Metrics push rate in Hz - overrides config file
    #[arg(long)]
    metrics_hz: Option<u32>,

    /// Disable discovery service registration (overrides config file)
    #[arg(long)]
    no_discovery: bool,

    /// Depot discovery endpoint - overrides config file (e.g., "192.168.1.100:4860")
    #[arg(long)]
    discovery_endpoint: Option<String>,

    /// Disable autonomous mode (policy loading)
    #[arg(long)]
    no_autonomous: bool,

    /// Path to policy file for autonomous mode
    #[arg(long)]
    policy: Option<PathBuf>,

    /// Directory containing policy files
    #[arg(long)]
    policy_dir: Option<PathBuf>,

    /// Autonomous mode goal position [x,y] (for testing)
    #[arg(long, value_parser = parse_goal)]
    goal: Option<[f64; 2]>,

    /// Disable dispatch service connection (overrides config file)
    #[arg(long)]
    no_dispatch: bool,

    /// Dispatch service endpoint - overrides config file (e.g., "ws://192.168.1.100:4890/ws")
    #[arg(long)]
    dispatch_endpoint: Option<String>,
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

/// State for a dispatched task being executed
#[derive(Debug, Clone)]
struct DispatchedTask {
    task_id: uuid::Uuid,
    waypoints: Vec<DispatchWaypoint>,
    current_waypoint: usize,
    lap: i32,
    is_loop: bool,
}

impl DispatchedTask {
    fn from_assignment(assignment: TaskAssignment) -> Self {
        Self {
            task_id: assignment.task_id,
            waypoints: assignment.zone.waypoints,
            current_waypoint: 0,
            lap: 0,
            is_loop: assignment.zone.is_loop,
        }
    }

    fn current_goal(&self) -> Option<[f64; 2]> {
        self.waypoints.get(self.current_waypoint).map(|wp| [wp.x, wp.y])
    }

    /// Advance to next waypoint. Returns true if task is complete.
    fn advance(&mut self) -> bool {
        self.current_waypoint += 1;
        if self.current_waypoint >= self.waypoints.len() {
            if self.is_loop {
                self.current_waypoint = 0;
                self.lap += 1;
                info!(lap = self.lap, "Starting new lap");
                false
            } else {
                true // Task complete
            }
        } else {
            false
        }
    }

    fn progress_percent(&self) -> i32 {
        if self.waypoints.is_empty() {
            return 100;
        }
        ((self.current_waypoint as f64 / self.waypoints.len() as f64) * 100.0) as i32
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
    let args = Args::parse();

    // Initialize logging with rolling file appender
    // The _guard must be held for the lifetime of the program to ensure logs are flushed
    let _log_guard = init_logging(&args.log_dir, &args.log_level)?;

    // Load configuration file
    let file_config = FileConfig::load(&args.config)?;
    info!(path = %args.config.display(), "Loaded config");

    // Resolve rover_id from config file
    let rover_id = file_config.identity.rover_id.clone();

    // CAN interface: CLI arg > default "can0"
    let can_iface = args.can_interface.as_deref().unwrap_or("can0");

    if args.sim {
        info!("Starting bvrd in SIMULATION mode");
    } else {
        info!(config = ?args.config, can = %can_iface, rover = %rover_id, "Starting bvrd");
    }

    // Initialize telemetry recorder
    let recorder = if args.no_recording {
        info!("Telemetry recording disabled");
        Recorder::disabled()
    } else {
        let recording_config = RecordingConfig {
            session_dir: args.recording_dir.clone(),
            rover_id: rover_id.clone(),
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            include_camera: false,
            include_lidar: false,
            enabled: true,
            ..Default::default()
        };
        match Recorder::new(&recording_config) {
            Ok(r) => {
                if let Some(path) = r.session_path() {
                    info!(path = %path.display(), "Recording session started");
                }
                r
            }
            Err(e) => {
                warn!(?e, "Failed to start recorder, continuing without recording");
                Recorder::disabled()
            }
        }
    };

    // Initialize metrics pusher for Depot
    // Priority: CLI args > config file
    let (metrics_tx, metrics_rx) = watch::channel(MetricsSnapshot::default());
    let metrics_enabled = !args.no_metrics && file_config.metrics.enabled;
    if metrics_enabled {
        let metrics_config = MetricsConfig {
            enabled: true,
            endpoint: args.metrics_endpoint.clone().unwrap_or(file_config.metrics.endpoint.clone()),
            interval_hz: args.metrics_hz.unwrap_or(file_config.metrics.interval_hz),
            rover_id: rover_id.clone(),
        };
        match MetricsPusher::new(&metrics_config) {
            Ok(pusher) => {
                info!(
                    endpoint = %metrics_config.endpoint,
                    hz = metrics_config.interval_hz,
                    "Metrics push enabled"
                );
                tokio::spawn(pusher.run(metrics_rx.clone()));
            }
            Err(e) => {
                warn!(?e, "Failed to start metrics pusher - continuing without metrics");
            }
        }
    } else {
        info!("Metrics push disabled");
    }

    // Initialize discovery client for Depot
    // Priority: CLI args > config file
    let discovery_enabled = !args.no_discovery && file_config.discovery.enabled;
    if discovery_enabled {
        let discovery_endpoint = args.discovery_endpoint.clone()
            .unwrap_or(file_config.discovery.endpoint.clone());
        let discovery_rover_id = file_config.discovery.rover_id.clone()
            .unwrap_or(rover_id.clone());
        let discovery_rover_name = file_config.discovery.rover_name.clone()
            .unwrap_or_else(|| discovery_rover_id.replace("bvr-", "Beaver-").replace("frog-", "Frog-"));

        let discovery_config = DiscoveryConfig {
            enabled: true,
            endpoint: discovery_endpoint.clone(),
            heartbeat_secs: file_config.discovery.heartbeat_secs,
            rover_id: discovery_rover_id,
            rover_name: discovery_rover_name,
            ws_port: file_config.discovery.ws_port,
            ws_video_port: file_config.discovery.ws_video_port,
        };

        let discovery_client = DiscoveryClient::new(discovery_config);
        let discovery_rx = metrics_rx.clone();

        tokio::spawn(async move {
            discovery_client.run(discovery_rx).await;
        });

        info!(
            endpoint = %discovery_endpoint,
            "Discovery client started"
        );
    } else {
        info!("Discovery registration disabled");
    }

    // Initialize autonomous mode (policy loading)
    let autonomous_enabled = !args.no_autonomous && file_config.autonomous.enabled;
    let mut loaded_policy: Option<Policy> = None;
    let autonomous_goal: Option<[f64; 2]> = args.goal.or(file_config.autonomous.goal);
    let autonomous_max_linear = file_config.autonomous.max_linear_vel;
    let autonomous_max_angular = file_config.autonomous.max_angular_vel;
    let norm_config = NormalizationConfig::default();

    if autonomous_enabled {
        // Determine policy path
        let policy_path = if let Some(ref path) = args.policy {
            Some(path.clone())
        } else if let Some(ref file) = file_config.autonomous.policy_file {
            Some(PathBuf::from(file))
        } else {
            None
        };

        // Load policy
        if let Some(path) = policy_path {
            match Policy::load(&path) {
                Ok(policy) => {
                    info!(
                        name = %policy.name(),
                        version = %policy.version(),
                        path = %path.display(),
                        "Loaded autonomous policy"
                    );
                    loaded_policy = Some(policy);
                }
                Err(e) => {
                    warn!(?e, path = %path.display(), "Failed to load policy");
                }
            }
        } else {
            // Try to load from policy directory
            let policy_dir = args.policy_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from(&file_config.autonomous.policy_dir));

            let mut manager = PolicyManager::new(&policy_dir);
            match manager.load_default() {
                Ok(policy) => {
                    info!(
                        name = %policy.name(),
                        version = %policy.version(),
                        dir = %policy_dir.display(),
                        "Loaded default policy from directory"
                    );
                    loaded_policy = Some(policy.clone());
                }
                Err(e) => {
                    debug!(?e, "No policy found (autonomous mode will be unavailable)");
                }
            }
        }

        if loaded_policy.is_some() {
            info!("Autonomous mode enabled");
            if let Some(goal) = autonomous_goal {
                info!(x = goal[0], y = goal[1], "Initial goal set");
            }
        }
    } else {
        info!("Autonomous mode disabled");
    }

    // Initialize dispatch client for receiving mission waypoints
    let dispatch_enabled = !args.no_dispatch && file_config.dispatch.enabled;
    let dispatch_client = if dispatch_enabled {
        let dispatch_endpoint = args.dispatch_endpoint.clone()
            .unwrap_or(file_config.dispatch.endpoint.clone());
        
        info!(endpoint = %dispatch_endpoint, "Connecting to dispatch service");
        let client = DispatchClient::new(&dispatch_endpoint, &rover_id);
        Some(client)
    } else {
        info!("Dispatch service connection disabled");
        None
    };

    // Subscribe to dispatch events if enabled
    let mut dispatch_rx = dispatch_client.as_ref().map(|c| c.subscribe());

    // Current dispatched task state
    let current_dispatch_task: Arc<Mutex<Option<DispatchedTask>>> = Arc::new(Mutex::new(None));
    let dispatch_task_clone = current_dispatch_task.clone();

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

    // Chassis parameters from config
    let chassis = ChassisParams::new(
        file_config.chassis.wheel_diameter_m,
        file_config.chassis.track_width_m,
        file_config.chassis.wheelbase_m,
    );
    info!(
        wheel_diameter = chassis.wheel_radius * 2.0,
        track_width = chassis.track_width,
        "Chassis parameters loaded"
    );
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

    // Spawn teleop server (UDP)
    let teleop_config = TeleopConfig::default();
    let teleop = TeleopServer::new(teleop_config, cmd_tx.clone(), telemetry_rx.clone());

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Spawn WebSocket teleop server (for browser-based operators)
    if args.ws_port > 0 {
        let ws_config = WsConfig {
            port: args.ws_port,
            ..Default::default()
        };
        let ws_server = WsServer::new(ws_config, cmd_tx.clone(), telemetry_rx.clone());

        tokio::spawn(async move {
            if let Err(e) = ws_server.run().await {
                error!(?e, "WebSocket teleop server error");
            }
        });

        info!(port = args.ws_port, "WebSocket teleop server started");
    }

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

                    // Spawn UDP video server (for native operator)
                    let video_config = VideoConfig::default();
                    let video_rx_udp = video_rx.clone();
                    let video_server = VideoServer::new(video_config.clone(), video_rx_udp);
                    info!(port = video_config.port, "UDP video server starting");

                    tokio::spawn(async move {
                        if let Err(e) = video_server.run().await {
                            error!(?e, "UDP video server error");
                        }
                    });

                    // Spawn WebSocket video server (for browser-based operator)
                    if args.ws_video_port > 0 {
                        let ws_video_config = WsVideoConfig {
                            port: args.ws_video_port,
                            ..Default::default()
                        };
                        let ws_video_server = WsVideoServer::new(ws_video_config, video_rx);
                        info!(port = args.ws_video_port, "WebSocket video server starting");

                        tokio::spawn(async move {
                            if let Err(e) = ws_video_server.run().await {
                                error!(?e, "WebSocket video server error");
                            }
                        });
                    }
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

    // LiDAR reader
    let (lidar_tx, mut lidar_rx) = watch::channel(None);
    if file_config.lidar.enabled {
        let lidar_config = LidarConfig {
            port: file_config.lidar.port.clone(),
            baud_rate: file_config.lidar.baud_rate,
        };

        let lidar_reader = LidarReader::new(lidar_config);
        match lidar_reader.spawn(lidar_tx) {
            Ok(_handle) => {
                info!(
                    port = %file_config.lidar.port,
                    baud = file_config.lidar.baud_rate,
                    "LiDAR reader started"
                );
            }
            Err(e) => {
                warn!(?e, "Failed to start LiDAR reader - continuing without LiDAR");
            }
        }
    } else {
        info!("LiDAR disabled in config");
    }

    // Control loop setup
    let mixer = DiffDriveMixer::new(chassis);
    let mut rate_limiter = RateLimiter::new(limits);
    let mut watchdog = Watchdog::new(Duration::from_millis(500)); // Allow for network jitter over Tailscale

    let control_period = Duration::from_millis(10); // 100 Hz
    let mut last_tick = Instant::now();

    // Current tool command (from teleop)
    let mut tool_command = types::ToolCommand::default();

    // Track mode for change detection (for recording annotations)
    let mut last_mode = initial_mode;

    // Send initial LED command for current mode
    {
        let state = shared.lock().unwrap();
        let led_cmd = state.state_machine.led_command();
        let led_frame = led_cmd.to_frame();
        if let Err(e) = can_interface.send(&led_frame) {
            debug!(?e, "Failed to send initial LED command");
        } else {
            info!(?initial_mode, "Initial LED state set");
        }
    }

    // Diagnostic counter for battery logging
    let mut loop_count: u64 = 0;

    info!("Entering control loop");
    info!("Dashboard available at http://localhost:{}", args.ui_port);
    info!("Send commands to UDP port 4840");
    if args.ws_port > 0 {
        info!("WebSocket teleop at ws://localhost:{}", args.ws_port);
    }
    if args.ws_video_port > 0 {
        info!("WebSocket video at ws://localhost:{}", args.ws_video_port);
    }
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

        loop_count += 1;

        // Log battery voltage every 10 seconds for diagnostics
        if loop_count % 1000 == 0 {
            let state = shared.lock().unwrap();
            let voltage = state.drivetrain.battery_voltage();
            if voltage > 0.0 {
                info!(voltage = format!("{:.1}V", voltage), "Battery status");
            } else {
                warn!("Battery voltage not received - check VESC CAN status settings");
            }
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
                    rate_limiter.reset();
                }
                Command::EStopRelease => {
                    info!("E-Stop release command received");
                    state.state_machine.transition(Event::EStopRelease);
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

        // Process dispatch events (task assignments/cancellations)
        if let Some(ref mut rx) = dispatch_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    DispatchEvent::TaskAssigned(assignment) => {
                        info!(
                            task_id = %assignment.task_id,
                            mission_id = %assignment.mission_id,
                            waypoints = assignment.zone.waypoints.len(),
                            is_loop = assignment.zone.is_loop,
                            "Task assigned from dispatch"
                        );
                        
                        let task = DispatchedTask::from_assignment(assignment);
                        *dispatch_task_clone.lock().unwrap() = Some(task);
                        
                        // Transition to autonomous mode to execute the task
                        let mut state = shared.lock().unwrap();
                        match state.state_machine.mode() {
                            Mode::Idle | Mode::Disabled => {
                                if state.state_machine.mode() == Mode::Disabled {
                                    state.state_machine.transition(Event::Enable);
                                }
                                state.state_machine.transition(Event::AutonomousRequest);
                                info!("Transitioned to Autonomous mode for dispatch task");
                            }
                            Mode::Teleop => {
                                // Don't interrupt teleop - operator has control
                                info!("Task queued - currently in Teleop mode");
                            }
                            Mode::Autonomous => {
                                // Already autonomous, will pick up new task
                                info!("New task received while autonomous");
                            }
                            _ => {}
                        }
                    }
                    DispatchEvent::TaskCancelled(task_id) => {
                        info!(task_id = %task_id, "Task cancelled by dispatch");
                        let mut task_guard = dispatch_task_clone.lock().unwrap();
                        if let Some(ref task) = *task_guard {
                            if task.task_id == task_id {
                                *task_guard = None;
                                // Return to idle if we were executing this task
                                let mut state = shared.lock().unwrap();
                                if state.state_machine.mode() == Mode::Autonomous {
                                    state.state_machine.transition(Event::CommandTimeout);
                                    info!("Returned to Idle after task cancellation");
                                }
                            }
                        }
                    }
                    DispatchEvent::Connected(true) => {
                        info!("Connected to dispatch service");
                    }
                    DispatchEvent::Connected(false) => {
                        warn!("Disconnected from dispatch service");
                    }
                }
            }
        }

        // Check watchdog
        {
            let mut state = shared.lock().unwrap();
            if watchdog.is_timed_out() && state.state_machine.is_driving() {
                warn!("Command watchdog timeout");
                state.state_machine.transition(Event::CommandTimeout);
                state.commanded_twist = Twist::default();
                rate_limiter.reset();
            }
        }

        // Get current mode and commanded twist for control decisions
        let (current_mode, commanded_twist) = {
            let state = shared.lock().unwrap();
            (state.state_machine.mode(), state.commanded_twist)
        };

        // Compute motor outputs based on mode
        let (target_twist, boost_active) = match current_mode {
            Mode::Autonomous => {
                // Autonomous mode: use policy for navigation
                // Priority: dispatched task waypoints > static autonomous_goal
                let goal = {
                    let task_guard = current_dispatch_task.lock().unwrap();
                    if let Some(ref task) = *task_guard {
                        task.current_goal()
                    } else {
                        autonomous_goal
                    }
                };

                if let (Some(policy), Some(goal)) = (&loaded_policy, goal) {
                    // Get current pose estimate
                    let current_pose = if args.sim {
                        can_interface.pose()
                    } else {
                        pose_estimator.pose()
                    };

                    // Get velocity estimate (use last commanded as approximation)
                    let linear_vel = commanded_twist.linear;
                    let angular_vel = commanded_twist.angular;

                    // Build observation for policy
                    let obs = PolicyObservation::from_raw(
                        current_pose.x,
                        current_pose.y,
                        current_pose.theta,
                        linear_vel,
                        angular_vel,
                        goal[0],
                        goal[1],
                        &norm_config,
                    );

                    // Run policy inference
                    match policy.infer(&obs) {
                        Ok(action) => {
                            let twist = action.to_twist(autonomous_max_linear, autonomous_max_angular);
                            debug!(
                                linear = twist.linear,
                                angular = twist.angular,
                                goal_x = goal[0],
                                goal_y = goal[1],
                                "Autonomous policy output"
                            );

                            // Check if waypoint reached (within 0.5m)
                            let dx = goal[0] - current_pose.x;
                            let dy = goal[1] - current_pose.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 0.5 {
                                // Handle dispatched task waypoint progression
                                let mut task_guard = current_dispatch_task.lock().unwrap();
                                if let Some(ref mut task) = *task_guard {
                                    let wp_idx = task.current_waypoint;
                                    info!(
                                        waypoint = wp_idx,
                                        total = task.waypoints.len(),
                                        distance = dist,
                                        "Waypoint reached"
                                    );

                                    // Report progress to dispatch service
                                    if let Some(ref client) = dispatch_client {
                                        let progress = task.progress_percent();
                                        let waypoint = task.current_waypoint as i32;
                                        let lap = task.lap;
                                        let task_id = task.task_id;
                                        // Spawn async task for the report
                                        let client_clone = client.clone();
                                        tokio::spawn(async move {
                                            let _ = client_clone.report_progress(task_id, progress, waypoint, lap).await;
                                        });
                                    }

                                    // Advance to next waypoint
                                    let task_complete = task.advance();
                                    if task_complete {
                                        info!(task_id = %task.task_id, "Dispatch task complete");
                                        
                                        // Report completion
                                        if let Some(ref client) = dispatch_client {
                                            let task_id = task.task_id;
                                            let laps = task.lap;
                                            let client_clone = client.clone();
                                            tokio::spawn(async move {
                                                let _ = client_clone.report_complete(task_id, laps).await;
                                            });
                                        }
                                        
                                        // Clear task and return to idle
                                        let task_id = task.task_id;
                                        drop(task_guard);
                                        *current_dispatch_task.lock().unwrap() = None;
                                        
                                        let mut state = shared.lock().unwrap();
                                        state.state_machine.transition(Event::CommandTimeout);
                                        info!(task_id = %task_id, "Returned to Idle after task completion");
                                    }
                                } else {
                                    // Static goal reached (testing mode)
                                    info!(distance = dist, "Static goal reached!");
                                }
                            }

                            (twist, false)
                        }
                        Err(e) => {
                            warn!(?e, "Policy inference failed, stopping");
                            
                            // Report failure if executing dispatch task
                            if let Some(ref task) = *current_dispatch_task.lock().unwrap() {
                                if let Some(ref client) = dispatch_client {
                                    let task_id = task.task_id;
                                    let client_clone = client.clone();
                                    tokio::spawn(async move {
                                        let _ = client_clone.report_failed(task_id, "Policy inference failed").await;
                                    });
                                }
                            }
                            
                            (Twist::default(), false)
                        }
                    }
                } else {
                    // No policy or goal, stay stopped
                    if loaded_policy.is_none() {
                        debug!("Autonomous mode but no policy loaded");
                    } else {
                        debug!("Autonomous mode but no goal/waypoints");
                    }
                    (Twist::default(), false)
                }
            }
            Mode::Teleop => {
                // Teleop mode: use commanded twist directly
                (commanded_twist, commanded_twist.boost)
            }
            _ => {
                // Not driving: zero velocity
                (Twist::default(), false)
            }
        };

        // Lock state for motor commands and telemetry
        let mut state = shared.lock().unwrap();

        // Rate limit for safety (acceleration limits only)
        let mut twist = rate_limiter.limit(target_twist);

        // Boost angular for skid steering (requires more torque than forward motion)
        twist.angular *= 2.5;

        let wheel_vels = mixer.mix(twist);

        // Convert wheel velocities (rad/s) to duty cycle (-1.0 to 1.0)
        // Using duty cycle control for smoother low-speed operation with hall sensors
        // Max wheel velocity at 5 m/s with 0.08m radius = 62.5 rad/s
        const MAX_WHEEL_VEL: f64 = 62.5;
        const NORMAL_DUTY: f64 = 0.5;  // Normal mode: ~50% power (~3 m/s)
        const BOOST_DUTY: f64 = 0.95;  // Boost mode: full blast
        let max_duty = if boost_active { BOOST_DUTY } else { NORMAL_DUTY };
        let wheel_duties: [f64; 4] = [
            (wheel_vels.front_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.front_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.rear_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.rear_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
        ];

        // Log wheel commands when turning (left != right)
        if (wheel_duties[0] - wheel_duties[1]).abs() > 0.01 {
            info!(
                fl = format!("{:.2}", wheel_duties[0]),
                fr = format!("{:.2}", wheel_duties[1]),
                rl = format!("{:.2}", wheel_duties[2]),
                rr = format!("{:.2}", wheel_duties[3]),
                "Wheel duties (turning)"
            );
        }

        // Send to VESCs using duty cycle control (smoother than RPM at low speeds)
        let vesc_cmds = state.drivetrain.build_duty_commands(wheel_duties);
        for frame in vesc_cmds {
            if let Err(e) = can_interface.send(&frame) {
                error!(?e, "Failed to send duty to drivetrain");
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

        // Check for LiDAR updates
        if lidar_rx.has_changed().unwrap_or(false) {
            if let Some(ref scan) = &*lidar_rx.borrow_and_update() {
                let _ = recorder.log_lidar_scan(scan);

                // Log stats periodically (every 50 scans = ~5-10 seconds)
                static LIDAR_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let count = LIDAR_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if count % 50 == 0 {
                    debug!(
                        points = scan.ranges.len(),
                        "LiDAR scan"
                    );
                }
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

        let _ = telemetry_tx.send(telemetry.clone());

        // Update metrics snapshot for Depot push
        let gps_state = gps_rx.borrow();
        let metrics_snapshot = MetricsSnapshot {
            mode: telemetry.mode,
            battery_voltage: telemetry.power.battery_voltage,
            system_current: telemetry.power.system_current,
            motor_temps: telemetry.motor_temps,
            motor_currents: telemetry.motor_currents,
            velocity_linear: telemetry.velocity.linear,
            velocity_angular: telemetry.velocity.angular,
            gps_latitude: gps_state.coord.as_ref().map(|c| c.lat).unwrap_or(0.0),
            gps_longitude: gps_state.coord.as_ref().map(|c| c.lon).unwrap_or(0.0),
            gps_accuracy: gps_state.coord.as_ref().map(|c| c.accuracy).unwrap_or(0.0),
            ..Default::default()
        };
        drop(gps_state);
        let _ = metrics_tx.send(metrics_snapshot);

        // Record telemetry to Rerun session
        let time_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        recorder.set_time(time_secs);

        // Log all telemetry data
        let _ = recorder.log_pose(&pose);
        let _ = recorder.log_velocity(&state.commanded_twist, &twist);
        let _ = recorder.log_motors(&motor_currents, &motor_temps);
        let _ = recorder.log_power(telemetry.power.battery_voltage, telemetry.power.system_current);
        let _ = recorder.log_odometry(dx, dy, dtheta);

        // Log mode changes and update LEDs
        let current_mode = telemetry.mode;
        if current_mode != last_mode {
            let _ = recorder.log_mode(current_mode);

            // Send LED command for new mode
            let led_cmd = state.state_machine.led_command();
            let led_frame = led_cmd.to_frame();
            if let Err(e) = can_interface.send(&led_frame) {
                debug!(?e, "Failed to send LED command");
            } else {
                debug!(?current_mode, "LED command sent for mode change");
            }

            last_mode = current_mode;
        }

        // Log tool state if active
        if let Some(ref status) = telemetry.tool_status {
            let _ = recorder.log_tool(&status.name, status.position, status.current);
        }
    }
}

/// Initialize logging with stdout and rolling file output.
///
/// Returns a guard that must be held for the lifetime of the program to ensure
/// logs are properly flushed on shutdown.
fn init_logging(
    log_dir: &std::path::Path,
    level: &str,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        eprintln!("Error: Cannot create log directory '{}': {}", log_dir.display(), e);
        eprintln!();
        eprintln!("The default log directory requires root permissions.");
        eprintln!("Try running with local directories:");
        eprintln!();
        eprintln!("  cargo run --bin bvrd -- --sim --log-dir ./logs --recording-dir ./sessions");
        eprintln!();
        eprintln!("Or disable recording for quick testing:");
        eprintln!();
        eprintln!("  cargo run --bin bvrd -- --sim --no-recording --log-dir /tmp");
        eprintln!();
        return Err(e.into());
    }

    // Rolling file appender: daily rotation
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "bvrd.log");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Build filter from level string, with fallback
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("bvrd={},recording=info", level)));

    // Stdout layer: human-readable, colored
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false);

    // File layer: no ANSI codes, includes timestamps
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

