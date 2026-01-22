//! bvrd — main daemon for the Base Vectoring Rover.

use anyhow::Result;
use camera::Config as CameraConfig;
use dispatch::{DispatchClient, DispatchEvent, TaskAssignment, Waypoint as DispatchWaypoint};
use can::vesc::Drivetrain;
use can::Bus;
use clap::Parser;
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use gps::{Config as GpsConfig, GpsReader, GpsState};
use hal::EStopEvent;
use lidar::{Config as LidarConfig, LidarReader};
use localization::{PoseEstimator, WheelOdometry};
use slam::{SlamConfig, SlamProcessor};
use metrics::{Config as MetricsConfig, DiscoveryClient, DiscoveryConfig, MetricsPusher, MetricsSnapshot, SystemMetricsCollector};
use policy::{NormalizationConfig, Policy, PolicyManager, PolicyObservation};
use recording::{Config as RecordingConfig, Recorder};
use serde::Deserialize;
use sim::SimBus;
use state::{Event, StateMachine};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use teleop::video::{VideoConfig, VideoFrame, VideoServer};
use teleop::rtc::{RtcConfig, RtcServer};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch};
use tools::{protocol, Registry as ToolRegistry, ToolOutput};
use tracing::{debug, error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use types::{Command, Mode, Pose, PowerStatus, SlamStatus, SubsystemHealth, Twist};
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
    slam: SlamFileConfig,
    estop: EStopFileConfig,
}

/// Hardware e-stop configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct EStopFileConfig {
    /// Enable hardware e-stop GPIO monitoring
    enabled: bool,
    /// GPIO chip device name (e.g., "gpiochip0")
    gpio_chip: String,
    /// GPIO line number (e.g., 16 for Pin 36 on Jetson 40-pin header)
    gpio_line: u32,
    /// Active-low logic (true = button connects GPIO to GND when pressed)
    active_low: bool,
    /// Debounce time in milliseconds
    debounce_ms: u64,
    /// Require operator confirmation to release e-stop (not just button release)
    require_confirmation: bool,
}

impl Default for EStopFileConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default - opt-in when hardware is connected
            gpio_chip: "gpiochip0".to_string(),
            gpio_line: 16,
            active_low: true,
            debounce_ms: 50,
            require_confirmation: true,
        }
    }
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
    /// LiDAR IP address (Livox Mid360)
    lidar_ip: String,
    /// Point cloud UDP port (default 56301)
    point_cloud_port: u16,
    /// IMU data UDP port (default 56401, 0 to disable)
    imu_port: u16,
}

impl Default for LidarFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lidar_ip: "192.168.1.100".to_string(),
            point_cloud_port: 56301,
            imu_port: 56401,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct SlamFileConfig {
    /// Enable SLAM processing
    enabled: bool,
    /// Resolution for scan matching correlation grid (meters)
    scan_match_resolution: f64,
    /// Distance threshold for inserting new keyframe (meters)
    keyframe_distance: f64,
    /// Rotation threshold for inserting new keyframe (radians)
    keyframe_rotation: f64,
    /// Score threshold for accepting loop closure match
    loop_closure_threshold: f64,
}

impl Default for SlamFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_match_resolution: 0.05,
            keyframe_distance: 1.0,
            keyframe_rotation: 0.5,
            loop_closure_threshold: 0.7,
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
    /// WebRTC signaling port to advertise (default: 4852)
    rtc_port: u16,
    heartbeat_secs: u32,
    /// Hostname to advertise instead of auto-detected IP (e.g., "frog-0" for Tailscale)
    advertise_host: Option<String>,
}

impl Default for DiscoveryFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "depot.local:4860".to_string(),
            rover_id: None,
            rover_name: None,
            rtc_port: 4852,
            heartbeat_secs: 2,
            advertise_host: None,
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

    /// WebRTC signaling port for teleop (0 to disable)
    #[arg(long, default_value = "4852")]
    rtc_port: u16,

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

    /// Enable tokio-console for runtime introspection (requires --features console)
    #[arg(long)]
    console: bool,
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
    /// Subsystem health status (updated by various components)
    health: SubsystemHealth,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with rolling file appender
    // The _guard must be held for the lifetime of the program to ensure logs are flushed
    let _log_guard = init_logging(&args.log_dir, &args.log_level, args.console)?;

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

    // Initialize telemetry recorder (paused - starts on Teleop/Autonomous entry)
    let mut recorder = if args.no_recording {
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
        info!("Recording enabled (will start on Teleop/Autonomous entry)");
        Recorder::new_paused(&recording_config)
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
            rtc_port: args.rtc_port,
            advertise_host: file_config.discovery.advertise_host.clone(),
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
        health: SubsystemHealth::default(),
    }));

    // Channels
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);

    let initial_telemetry = Telemetry {
        sequence: 0,
        timestamp_us: 0,
        mode: initial_mode,
        pose: Pose::default(),
        power: PowerStatus {
            battery_voltage: 48.0,  // Simulated full battery
            system_current: 0.0,
        },
        cmd_velocity: Twist::default(),
        meas_velocity: (0.0, 0.0),
        acceleration: (0.0, 0.0),
        motor_temps: [25.0; 4],  // Ambient temp
        motor_currents: [0.0; 4],
        health: SubsystemHealth::default(),
        odometry_quality: 0,
        dt_ms: 0.0,
        last_cmd_seq: 0,
        ack_bits: 0,
        cpu_percent: 0,
        mem_percent: 0,
        disk_percent: 0,
        active_tool: None,
        tool_status: None,
        slam_status: None,
    };
    let (telemetry_tx, telemetry_rx) = watch::channel(initial_telemetry);

    // Video frame channels
    // mpsc for WebRTC: queues frames from all cameras without overwriting
    let (video_tx_rtc, video_rx_rtc) = tokio::sync::mpsc::channel::<teleop::video::VideoFrame>(32);
    // watch for UDP: backward compatibility for native CLI (shows latest frame only)
    let (video_tx_udp, video_rx_udp) = watch::channel::<Option<teleop::video::VideoFrame>>(None);

    // Spawn teleop server (UDP)
    let teleop_config = TeleopConfig::default();
    let teleop = TeleopServer::new(teleop_config, cmd_tx.clone(), telemetry_rx.clone());

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Spawn WebRTC teleop server (primary browser teleop with video)
    if args.rtc_port > 0 {
        let rtc_config = RtcConfig {
            signaling_port: args.rtc_port,
            ..Default::default()
        };
        // Use with_video to enable WebRTC video streaming
        let rtc_server = RtcServer::with_video(
            rtc_config,
            cmd_tx.clone(),
            telemetry_rx.clone(),
            video_rx_rtc,
        );

        tokio::spawn(async move {
            if let Err(e) = rtc_server.run().await {
                error!(?e, "WebRTC teleop server error");
            }
        });

        info!(port = args.rtc_port, "WebRTC teleop server started (low-latency + video)");
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

        // Auto-detect cameras (fast filesystem scan, no GStreamer init needed)
        info!("Detecting cameras...");
        let cameras = camera::detect_cameras();

        if cameras.is_empty() {
            info!("No cameras detected");
        } else {
            info!(count = cameras.len(), "Cameras detected");
            for cam in &cameras {
                info!(name = %cam.name, "  - {:?}", cam.camera_type);
            }

            // Start capture on all detected cameras
            // GStreamer init happens here and can hang on Jetson, so wrap in timeout
            info!("Starting camera capture for {} camera(s)...", cameras.len());

            let mut cameras_started = 0u8;

            for (camera_id, cam) in cameras.iter().enumerate() {
                let camera_id = camera_id as u8;
                let cam_clone = cam.clone();
                let cam_config = camera_config.clone();

                let capture_result = tokio::time::timeout(
                    Duration::from_secs(15),
                    tokio::task::spawn_blocking(move || {
                        camera::spawn_capture(&cam_clone, cam_config)
                    }),
                )
                .await;

                match capture_result {
                    Ok(Ok(Ok((frame_rx, _camera_handle)))) => {
                        info!(
                            camera_id,
                            camera = %cam.name,
                            "Camera {} started: {}x{} @ {}fps",
                            camera_id,
                            width,
                            height,
                            args.camera_fps
                        );

                        cameras_started += 1;

                        // Clone both video tx channels for this camera thread
                        let video_tx_rtc_camera = video_tx_rtc.clone();
                        let video_tx_udp_camera = video_tx_udp.clone();

                        // Spawn task to bridge sync camera frames to async video servers
                        std::thread::spawn(move || {
                            let mut frame_count: u64 = 0;
                            let mut total_bytes: u64 = 0;
                            let mut last_log = std::time::Instant::now();

                            while let Ok(frame) = frame_rx.recv() {
                                let frame_size = frame.data.len();
                                let video_frame = VideoFrame {
                                    camera_id,
                                    data: frame.data,
                                    width: frame.width,
                                    height: frame.height,
                                    sequence: frame.sequence,
                                    timestamp_ms: frame.timestamp_ms,
                                };

                                // Send to WebRTC (mpsc - queues all frames)
                                let send_start = std::time::Instant::now();
                                if video_tx_rtc_camera.blocking_send(video_frame.clone()).is_err() {
                                    break;
                                }
                                let send_us = send_start.elapsed().as_micros();

                                // Send to UDP (watch - latest frame only, for native CLI)
                                let _ = video_tx_udp_camera.send(Some(video_frame));

                                frame_count += 1;
                                total_bytes += frame_size as u64;

                                // Log stats every 5 seconds
                                if last_log.elapsed() >= std::time::Duration::from_secs(5) {
                                    let elapsed = last_log.elapsed().as_secs_f64();
                                    let fps = frame_count as f64 / elapsed;
                                    let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
                                    tracing::debug!(
                                        camera_id,
                                        frame_count,
                                        fps = format!("{:.1}", fps),
                                        mbps = format!("{:.2}", mbps),
                                        last_send_us = send_us,
                                        "camera.bridge_stats"
                                    );
                                    frame_count = 0;
                                    total_bytes = 0;
                                    last_log = std::time::Instant::now();
                                }
                            }
                        });
                    }
                    Ok(Ok(Err(e))) => {
                        warn!(camera_id, ?e, "Failed to start camera {} - skipping", camera_id);
                    }
                    Ok(Err(e)) => {
                        warn!(camera_id, ?e, "Camera {} capture task panicked", camera_id);
                    }
                    Err(_) => {
                        warn!(camera_id, "Camera {} capture timed out (GStreamer init hung)", camera_id);
                    }
                }
            }

            if cameras_started > 0 {
                // Mark camera as active in health status
                shared.lock().unwrap().health.camera_active = true;
                info!(count = cameras_started, "Camera capture started");

                // Spawn UDP video server (for native operator CLI)
                let video_config = VideoConfig::default();
                let video_server = VideoServer::new(video_config.clone(), video_rx_udp.clone());
                info!(port = video_config.port, "UDP video server starting");

                tokio::spawn(async move {
                    if let Err(e) = video_server.run().await {
                        error!(?e, "UDP video server error");
                    }
                });
            } else {
                warn!("No cameras could be started - continuing without video");
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

    // LiDAR reader (Livox Mid360)
    let (lidar_tx, mut lidar_rx) = watch::channel(None);
    if file_config.lidar.enabled {
        let lidar_ip = file_config.lidar.lidar_ip.parse()
            .unwrap_or(std::net::Ipv4Addr::new(192, 168, 1, 100));
        let lidar_config = LidarConfig {
            lidar_ip,
            point_cloud_port: file_config.lidar.point_cloud_port,
            imu_port: file_config.lidar.imu_port,
            ..Default::default()
        };

        let lidar_reader = LidarReader::new(lidar_config);
        let _lidar_handle = lidar_reader.spawn(lidar_tx);
        info!(
            ip = %file_config.lidar.lidar_ip,
            port = file_config.lidar.point_cloud_port,
            "LiDAR reader started (Livox Mid360)"
        );
    } else {
        info!("LiDAR disabled in config");
    }

    // Initialize SLAM processor
    let mut slam_processor = if file_config.slam.enabled && file_config.lidar.enabled {
        let slam_config = SlamConfig {
            scan_match_resolution: file_config.slam.scan_match_resolution,
            keyframe_distance: file_config.slam.keyframe_distance,
            keyframe_rotation: file_config.slam.keyframe_rotation,
            loop_closure_threshold: file_config.slam.loop_closure_threshold,
            ..Default::default()
        };
        info!("SLAM enabled");
        Some(SlamProcessor::new(slam_config))
    } else {
        if !file_config.slam.enabled {
            info!("SLAM disabled in config");
        }
        None
    };

    // Hardware e-stop GPIO monitoring (Jetson only, requires gpio feature)
    let (estop_tx, mut estop_rx) = mpsc::channel::<EStopEvent>(8);
    let mut hardware_estop_latched = false; // Latches until explicit release confirmation

    #[cfg(feature = "gpio")]
    {
        if file_config.estop.enabled {
            use hal::{EStopConfig, EStopInput};

            let estop_config = EStopConfig {
                gpio_chip: file_config.estop.gpio_chip.clone(),
                gpio_line: file_config.estop.gpio_line,
                active_low: file_config.estop.active_low,
                debounce_ms: file_config.estop.debounce_ms,
            };

            let estop_input = EStopInput::new(estop_config.clone());

            match estop_input.spawn(estop_tx).await {
                Ok(handle) => {
                    info!(
                        gpio_chip = %estop_config.gpio_chip,
                        gpio_line = estop_config.gpio_line,
                        active_low = estop_config.active_low,
                        require_confirmation = file_config.estop.require_confirmation,
                        "Hardware e-stop GPIO monitoring started"
                    );
                    // Let the handle run in the background
                    drop(handle);
                }
                Err(e) => {
                    warn!(?e, "Failed to start hardware e-stop GPIO - continuing without hardware e-stop");
                }
            }
        } else {
            drop(estop_tx); // Not using GPIO, drop the sender
            info!("Hardware e-stop disabled in config");
        }
    }

    #[cfg(not(feature = "gpio"))]
    {
        drop(estop_tx); // Not using GPIO, drop the sender
        if file_config.estop.enabled {
            warn!("Hardware e-stop enabled in config but compiled without 'gpio' feature");
        }
    }

    // Track which subsystems are enabled for health reporting
    let lidar_enabled = file_config.lidar.enabled;
    let recording_enabled = recorder.is_enabled();

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

    // System metrics collector (CPU, memory, disk)
    let mut sys_metrics = SystemMetricsCollector::new();
    let mut last_sys_refresh = Instant::now();

    // CAN bus health tracking - avoid overwhelming driver when bus is unhealthy
    let mut can_errors_active = false;
    let mut can_backoff_active = false;
    let mut can_error_count: u64 = 0;
    let mut consecutive_can_errors: u32 = 0;
    const CAN_ERROR_BACKOFF_THRESHOLD: u32 = 10; // Skip commands after 10 consecutive errors
    const CAN_RETRY_INTERVAL: u64 = 100; // Retry every 100 loops (1 second at 100Hz)

    // Policy execution timeout tracking for autonomous mode safety
    const POLICY_WARN_THRESHOLD: Duration = Duration::from_millis(20); // Warn if >20ms (2x control period)
    const POLICY_ERROR_THRESHOLD: Duration = Duration::from_millis(50); // Error if >50ms (5x control period)
    let mut policy_slow_count: u32 = 0;

    info!("Entering control loop");
    info!("Dashboard available at http://localhost:{}", args.ui_port);
    info!("Send commands to UDP port 4840");
    if args.rtc_port > 0 {
        info!(
            "WebRTC teleop at ws://localhost:{} for signaling",
            args.rtc_port
        );
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

        // Process hardware e-stop events (highest priority - before any other commands)
        while let Ok(event) = estop_rx.try_recv() {
            match event {
                EStopEvent::Pressed => {
                    warn!("Hardware e-stop button PRESSED");
                    let mut state = shared.lock().unwrap();
                    state.state_machine.transition(Event::EStop);
                    state.commanded_twist = Twist::default();
                    rate_limiter.reset();
                    hardware_estop_latched = true;

                    // Send e-stop LED pattern
                    let led_cmd = state.state_machine.led_command();
                    if let Err(e) = can_interface.send(&led_cmd.to_frame()) {
                        debug!(?e, "Failed to send e-stop LED command");
                    }
                }
                EStopEvent::Released => {
                    if file_config.estop.require_confirmation {
                        // Latch semantics: button release doesn't auto-clear e-stop
                        // Operator must send explicit EStopRelease command via teleop
                        info!(
                            "Hardware e-stop button released (awaiting operator confirmation to resume)"
                        );
                    } else {
                        // Auto-release: button release clears e-stop immediately
                        info!("Hardware e-stop button released (auto-release enabled)");
                        let mut state = shared.lock().unwrap();
                        state.state_machine.transition(Event::EStopRelease);
                        hardware_estop_latched = false;

                        // Send idle LED pattern
                        let led_cmd = state.state_machine.led_command();
                        if let Err(e) = can_interface.send(&led_cmd.to_frame()) {
                            debug!(?e, "Failed to send e-stop release LED command");
                        }
                    }
                }
            }
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

        // Process incoming commands (non-blocking, "latest Twist wins")
        // Drain all commands first, keeping only the latest Twist
        let mut latest_twist: Option<Twist> = None;
        let mut other_commands: Vec<Command> = Vec::new();

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Twist(twist) => {
                    // Always overwrite - only the latest Twist matters
                    latest_twist = Some(twist);
                }
                other => {
                    // Queue non-Twist commands for ordered processing
                    other_commands.push(other);
                }
            }
        }

        // Process non-Twist commands (order matters for these)
        if !other_commands.is_empty() {
            let mut state = shared.lock().unwrap();
            for cmd in other_commands {
                match cmd {
                    Command::EStop => {
                        warn!("E-Stop command received");
                        state.state_machine.transition(Event::EStop);
                        rate_limiter.reset();
                    }
                    Command::EStopRelease => {
                        if hardware_estop_latched {
                            // This is the operator confirmation to clear hardware e-stop
                            info!("E-Stop release confirmed by operator (clearing hardware latch)");
                            hardware_estop_latched = false;
                        } else {
                            info!("E-Stop release command received");
                        }
                        state.state_machine.transition(Event::EStopRelease);
                    }
                    Command::SetMode(mode) => {
                        // Debug: log all SetMode commands to trace source of mode flipping
                        warn!(
                            requested_mode = ?mode,
                            current_mode = ?state.state_machine.mode(),
                            "SetMode command received"
                        );
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
                    Command::Twist(_) => unreachable!(), // Already handled above
                }
            }
        }

        // Apply latest Twist command only if in Teleop mode (requires explicit control takeover)
        if let Some(twist) = latest_twist {
            let mut state = shared.lock().unwrap();
            if state.state_machine.mode() == Mode::Teleop {
                watchdog.feed();
                state.commanded_twist = twist;
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

        // Check watchdog - different handling for teleop vs autonomous
        {
            let is_timed_out;
            let current_mode;
            {
                let state = shared.lock().unwrap();
                is_timed_out = watchdog.is_timed_out() && state.state_machine.is_driving();
                current_mode = state.state_machine.mode();
            }

            if is_timed_out {
                match current_mode {
                    Mode::Teleop => {
                        // Teleop timeout: operator stopped sending commands, return to Idle
                        warn!("Command watchdog timeout in teleop mode");
                        let mut state = shared.lock().unwrap();
                        state.state_machine.transition(Event::CommandTimeout);
                        state.commanded_twist = Twist::default();
                    }
                    Mode::Autonomous => {
                        // Autonomous timeout: control loop or policy hung - this is critical
                        // Since watchdog is fed on successful policy inference, timeout means
                        // either policy is hanging or control loop stopped executing
                        error!("Watchdog timeout in autonomous mode - control loop or policy hung");

                        // Report failure if executing dispatch task
                        if let Some(ref task) = *current_dispatch_task.lock().unwrap() {
                            if let Some(ref client) = dispatch_client {
                                let task_id = task.task_id;
                                let client_clone = client.clone();
                                tokio::spawn(async move {
                                    let _ = client_clone
                                        .report_failed(task_id, "Watchdog timeout - control loop hung")
                                        .await;
                                });
                            }
                        }

                        // Transition to Fault (more severe than Idle) for autonomous failures
                        let mut state = shared.lock().unwrap();
                        state.state_machine.transition(Event::Fault);
                        state.commanded_twist = Twist::default();
                    }
                    _ => {
                        // Shouldn't happen (is_driving only true for Teleop/Autonomous)
                        warn!("Unexpected watchdog timeout in mode {:?}", current_mode);
                        let mut state = shared.lock().unwrap();
                        state.state_machine.transition(Event::CommandTimeout);
                        state.commanded_twist = Twist::default();
                    }
                }
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

                    // Run policy inference with timing
                    let policy_start = Instant::now();
                    match policy.infer(&obs) {
                        Ok(action) => {
                            let inference_time = policy_start.elapsed();

                            // Track slow policy inference (safety monitoring)
                            if inference_time > POLICY_ERROR_THRESHOLD {
                                policy_slow_count += 1;
                                error!(
                                    duration_ms = inference_time.as_millis(),
                                    slow_count = policy_slow_count,
                                    "Policy inference critically slow - may cause control issues"
                                );
                            } else if inference_time > POLICY_WARN_THRESHOLD {
                                policy_slow_count += 1;
                                warn!(
                                    duration_ms = inference_time.as_millis(),
                                    slow_count = policy_slow_count,
                                    "Policy inference slow"
                                );
                            } else {
                                policy_slow_count = 0; // Reset counter on normal execution
                            }

                            // Feed watchdog on successful autonomous control loop
                            // This ensures we detect control loop hangs, not just teleop command absence
                            watchdog.feed();

                            let twist = action.to_twist(autonomous_max_linear, autonomous_max_angular);
                            debug!(
                                linear = twist.linear,
                                angular = twist.angular,
                                goal_x = goal[0],
                                goal_y = goal[1],
                                inference_ms = inference_time.as_micros() as f64 / 1000.0,
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
        // Skip VESC commands when CAN bus is unhealthy to avoid overwhelming the driver
        // (which can cause kernel-level issues on Jetson)
        let should_send_vesc = consecutive_can_errors < CAN_ERROR_BACKOFF_THRESHOLD
            || loop_count % CAN_RETRY_INTERVAL == 0;

        let mut send_failed = false;
        if should_send_vesc {
            let vesc_cmds = state.drivetrain.build_duty_commands(wheel_duties);
            for frame in vesc_cmds {
                if let Err(_e) = can_interface.send(&frame) {
                    send_failed = true;
                    can_error_count += 1;
                }
            }
        }

        // Track consecutive errors for backoff
        if send_failed {
            consecutive_can_errors = consecutive_can_errors.saturating_add(1);
        } else if should_send_vesc {
            // Only reset on successful send (not when skipping)
            consecutive_can_errors = 0;
        }

        // Rate-limited CAN error logging - only log state transitions
        if send_failed && !can_errors_active {
            error!(
                "CAN bus errors started - drivetrain commands failing (VESCs powered?)"
            );
            can_errors_active = true;
        } else if consecutive_can_errors >= CAN_ERROR_BACKOFF_THRESHOLD && !can_backoff_active {
            warn!(
                "CAN bus unhealthy - backing off motor commands (retrying every {}ms)",
                CAN_RETRY_INTERVAL * 10
            );
            can_backoff_active = true;
        } else if !send_failed && should_send_vesc && can_errors_active {
            info!(errors = can_error_count, "CAN bus recovered - drivetrain commands succeeding");
            can_errors_active = false;
            can_backoff_active = false;
            can_error_count = 0;
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
        // Wheel status: 0=offline, 1=degraded, 2=online
        // - Offline: FET temp <= 5°C (not receiving data, default is 0.0)
        // - Degraded: FET temp > 5°C but motor temp <= 5°C (VESC online, motor disconnected)
        // - Online: FET temp > 5°C and motor temp > 5°C (fully operational)
        let wheel_status: [u8; 4] = std::array::from_fn(|i| {
            let fet_temp = vesc_states[i].status4.temp_fet;
            let motor_temp = vesc_states[i].status4.temp_motor;
            if fet_temp <= 5.0 {
                types::wheel_status::OFFLINE
            } else if motor_temp <= 5.0 {
                types::wheel_status::DEGRADED
            } else {
                types::wheel_status::ONLINE
            }
        });

        // Update wheel odometry from VESC tachometers
        // Negate right side (FR=1, RR=3) - motors are mounted facing opposite direction
        // but VESC motor direction config only affects duty, not tachometer readings
        let tach: [i32; 4] = [
            vesc_states[0].status5.tachometer,  // FL
            -vesc_states[1].status5.tachometer, // FR
            vesc_states[2].status5.tachometer,  // RL
            -vesc_states[3].status5.tachometer, // RR
        ];
        let (dx, dy, dtheta) = odometry.update(tach);

        // Update pose estimator with odometry
        pose_estimator.update_odometry(dx, dy, dtheta);

        // Feed odometry to SLAM (runs at control loop rate)
        if let Some(ref mut slam) = slam_processor {
            slam.update_odometry(&pose_estimator.pose());
        }

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

                // Process scan through SLAM
                if let Some(ref mut slam) = slam_processor {
                    if let Some(slam_update) = slam.process_scan(scan) {
                        if slam_update.keyframe_added {
                            debug!(
                                keyframe_count = slam_update.keyframe_count,
                                "SLAM keyframe added"
                            );
                        }
                        if slam_update.loop_closure_detected {
                            info!(
                                loop_closure_count = slam_update.loop_closure_count,
                                "SLAM loop closure detected"
                            );
                        }
                    }
                }

                // Log stats periodically (every 50 scans = ~5-10 seconds)
                static LIDAR_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let count = LIDAR_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if count % 50 == 0 {
                    debug!(
                        points = scan.points.len(),
                        "LiDAR point cloud"
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
            // In real mode, use SLAM-corrected pose if available, otherwise pose estimator
            if let Some(ref slam) = slam_processor {
                slam.pose()
            } else {
                pose_estimator.pose()
            }
        };

        // Build SLAM status if enabled
        let slam_status = slam_processor.as_ref().map(|slam| SlamStatus {
            pose,
            confidence: 0.8, // TODO: track actual match score
            keyframe_count: slam.keyframe_count() as u32,
            loop_closure_count: slam.loop_closure_count() as u32,
            mapping_active: true,
        });

        // Update health status based on current subsystem states
        let gps_state = gps_rx.borrow();
        let has_gps_fix = gps_state.coord.is_some() && gps_state.fix_quality > 0;
        drop(gps_state);

        let mut state = shared.lock().unwrap();

        // Update health from available state
        state.health.can_healthy = state.drivetrain.battery_voltage() > 0.0;
        state.health.gps_fix = has_gps_fix;
        state.health.slam_running = slam_processor.is_some();
        state.health.lidar_active = lidar_enabled;
        state.health.recording_active = recording_enabled;
        state.health.discovery_connected = discovery_enabled;
        state.health.dispatch_connected = dispatch_enabled;
        // Note: camera_active is set when camera starts successfully

        // Wheel/VESC status
        state.health.wheel_status = wheel_status;

        // Increment telemetry sequence
        static TELEM_SEQ: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
        let telem_seq = TELEM_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Get system metrics for telemetry (reuses cached values from sys_metrics)
        let sys = sys_metrics.metrics();

        let telemetry = Telemetry {
            sequence: telem_seq,
            timestamp_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0),
            mode: state.state_machine.mode(),
            pose,
            power: PowerStatus {
                battery_voltage: state.drivetrain.battery_voltage() as f64,
                system_current: motor_currents.iter().sum::<f32>() as f64,
            },
            cmd_velocity: twist,
            meas_velocity: (0.0, 0.0),  // TODO: populate from VelocityTracker
            acceleration: (0.0, 0.0),   // TODO: populate from VelocityTracker
            motor_temps,
            motor_currents,
            health: state.health,
            odometry_quality: 100,  // TODO: get from odometry
            dt_ms: (dt * 1000.0) as f32,
            last_cmd_seq: 0,  // TODO: populate from SequenceTracker
            ack_bits: 0,      // TODO: populate from SequenceTracker
            cpu_percent: sys.cpu_percent as u8,
            mem_percent: sys.mem_percent as u8,
            disk_percent: sys.disk_percent as u8,
            active_tool,
            tool_status,
            slam_status,
        };

        let _ = telemetry_tx.send(telemetry.clone());

        // Update metrics snapshot for Depot push
        // Refresh system metrics once per second (expensive operation)
        if last_sys_refresh.elapsed() >= Duration::from_secs(1) {
            sys_metrics.refresh();
            last_sys_refresh = Instant::now();
        }

        let gps_state = gps_rx.borrow();
        let metrics_snapshot = MetricsSnapshot {
            mode: telemetry.mode,
            battery_voltage: telemetry.power.battery_voltage,
            system_current: telemetry.power.system_current,
            motor_temps: telemetry.motor_temps,
            motor_currents: telemetry.motor_currents,
            velocity_linear: telemetry.cmd_velocity.linear,
            velocity_angular: telemetry.cmd_velocity.angular,
            gps_latitude: gps_state.coord.as_ref().map(|c| c.lat).unwrap_or(0.0),
            gps_longitude: gps_state.coord.as_ref().map(|c| c.lon).unwrap_or(0.0),
            gps_accuracy: gps_state.coord.as_ref().map(|c| c.accuracy).unwrap_or(0.0),
            system: sys_metrics.metrics(),
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

        // Log SLAM data if enabled
        if let Some(ref slam) = slam_processor {
            let _ = recorder.log_slam_trajectory(&pose);
            let _ = recorder.log_slam_keyframes(&slam.keyframe_poses());
            let _ = recorder.log_slam_status(
                slam.keyframe_count(),
                slam.loop_closure_count(),
                0.0, // TODO: track match score
            );
        }

        // Log mode changes and update LEDs
        let current_mode = telemetry.mode;
        if current_mode != last_mode {
            // Start/stop recording based on mode (only record during Teleop/Autonomous)
            let should_record = matches!(current_mode, Mode::Teleop | Mode::Autonomous);
            let was_recording = matches!(last_mode, Mode::Teleop | Mode::Autonomous);

            if should_record && !was_recording {
                // Entering a recording mode - start session
                if let Err(e) = recorder.start_session() {
                    warn!(?e, "Failed to start recording session");
                } else if let Some(path) = recorder.session_path() {
                    info!(path = %path.display(), mode = ?current_mode, "Recording session started");
                }
            } else if !should_record && was_recording {
                // Leaving a recording mode - end session
                recorder.end_session();
            }

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
    enable_console: bool,
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
    // Include teleop for WebSocket command timing diagnostics
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("bvrd={},teleop={},recording=info", level, level)));

    // Stdout layer: human-readable, colored
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false);

    // File layer: no ANSI codes, includes timestamps
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true);

    // Initialize with or without console-subscriber
    #[cfg(feature = "console")]
    {
        if enable_console {
            // Use console_subscriber's builder which handles layer composition
            console_subscriber::ConsoleLayer::builder()
                .server_addr(([0, 0, 0, 0], 6669))
                .with_default_env()
                .init();

            eprintln!("tokio-console enabled on port 6669 (default tracing, file logging disabled)");

            // Note: When using console, we use its built-in subscriber setup
            // File logging is disabled in this mode for simplicity
            return Ok(guard);
        }

        // Non-console path
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    #[cfg(not(feature = "console"))]
    {
        let _ = enable_console; // Suppress unused warning
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    Ok(guard)
}

