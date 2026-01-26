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
use lidar::{Config as LidarConfig, LidarCommand, LidarReader};
use localization::{AttitudeFilter, EkfConfig, EkfPoseEstimator, WheelOdometry};
use slam::{SlamConfig, SlamProcessor, SlamState};
use planner::PlannerConfig;
use pursuit::PursuitConfig;
use control_loop::{NavigationConfig, NavigationController, NavigationState};
use costmap::CostmapSnapshot;
use transforms::Transform2D;
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
use teleop::rtc::{RtcConfig, RtcMetrics, RtcServer};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch, Semaphore};
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
    control: ControlFileConfig,
    metrics: MetricsFileConfig,
    discovery: DiscoveryFileConfig,
    autonomous: AutonomousFileConfig,
    dispatch: DispatchFileConfig,
    lidar: LidarFileConfig,
    localization: LocalizationFileConfig,
    slam: SlamFileConfig,
    navigation: NavigationFileConfig,
    estop: EStopFileConfig,
    camera: CameraFileConfig,
}

/// Control loop configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct ControlFileConfig {
    /// Seconds of inactivity in Idle/Disabled before auto-transitioning to Sleep (0 = disabled)
    sleep_timeout_secs: u64,
}

impl Default for ControlFileConfig {
    fn default() -> Self {
        Self {
            sleep_timeout_secs: 0, // disabled by default
        }
    }
}

/// Camera configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct CameraFileConfig {
    /// Enable camera capture
    enabled: bool,
    /// Maximum number of cameras to initialize (0 = all detected)
    /// On Jetson, initializing multiple CSI cameras can hang GStreamer.
    /// Set to 1 if experiencing camera init hangs.
    max_cameras: usize,
}

impl Default for CameraFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cameras: 1, // Default to 1 to avoid Jetson CSI init issues
        }
    }
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
    /// Host IP address for receiving data (must match rover's eth interface)
    host_ip: String,
    /// Point cloud UDP port (default 56301)
    point_cloud_port: u16,
    /// IMU data UDP port (default 56401, 0 to disable)
    imu_port: u16,
    /// Voxel size for point cloud downsampling (meters). Larger = fewer points.
    stream_voxel_size: f32,
    /// Maximum points per frame for streaming. Higher = more detail but more bandwidth.
    stream_max_points: usize,
    /// Mounting pitch angle in degrees (positive = sensor tilted forward/down).
    /// Applied to both point cloud transform and IMU readings.
    mounting_pitch_deg: f32,
    /// Power save mode: only run lidar during Teleop/Autonomous, stop in Idle.
    /// Reduces heat and power consumption when rover is not active.
    power_save: bool,
}

impl Default for LidarFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lidar_ip: "192.168.1.100".to_string(),
            host_ip: "0.0.0.0".to_string(),
            point_cloud_port: 56301,
            imu_port: 56401,
            stream_voxel_size: 0.3,
            stream_max_points: 500,
            mounting_pitch_deg: 0.0,
            power_save: true,
        }
    }
}

/// Localization (EKF) configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct LocalizationFileConfig {
    odom_linear_noise: f64,
    odom_angular_noise: f64,
    gps_noise_rtk_fixed: f64,
    gps_noise_rtk_float: f64,
    gps_noise_dgps: f64,
    gps_noise_standalone: f64,
    gps_heading_min_speed: f64,
    imu_yaw_weight: f64,
}

impl Default for LocalizationFileConfig {
    fn default() -> Self {
        Self {
            odom_linear_noise: 0.05,
            odom_angular_noise: 0.02,
            gps_noise_rtk_fixed: 0.02,
            gps_noise_rtk_float: 0.5,
            gps_noise_dgps: 1.0,
            gps_noise_standalone: 3.0,
            gps_heading_min_speed: 0.3,
            imu_yaw_weight: 0.7,
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
    /// Huber loss threshold for robust loop closure weighting (meters)
    huber_threshold: f64,
    /// Maximum disagreement between loop closure and odometry (in σ)
    loop_closure_max_disagreement_sigma: f64,
}

impl Default for SlamFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_match_resolution: 0.05,
            keyframe_distance: 1.0,
            keyframe_rotation: 0.5,
            loop_closure_threshold: 0.7,
            huber_threshold: 1.0,
            loop_closure_max_disagreement_sigma: 3.0,
        }
    }
}

/// Classical navigation stack configuration (planner + pursuit).
#[derive(Debug, Deserialize)]
#[serde(default)]
struct NavigationFileConfig {
    /// Enable classical navigation (A* + pure pursuit) instead of policy
    enabled: bool,
    /// Costmap width in meters
    costmap_width: f64,
    /// Costmap height in meters
    costmap_height: f64,
    /// Costmap resolution (meters per cell)
    costmap_resolution: f64,
    /// Inflation radius around obstacles (meters)
    inflation_radius: f64,
    /// Robot inscribed radius (meters)
    inscribed_radius: f64,
    /// Lookahead distance for pure pursuit (meters)
    lookahead_distance: f64,
    /// Goal tolerance (meters)
    goal_tolerance: f64,
    /// Maximum linear velocity (m/s)
    max_linear_vel: f64,
    /// Maximum angular velocity (rad/s)
    max_angular_vel: f64,
    /// Blocked timeout before giving up (seconds)
    blocked_timeout: f64,
}

impl Default for NavigationFileConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Use classical navigation by default
            costmap_width: 20.0,
            costmap_height: 20.0,
            costmap_resolution: 0.1,
            inflation_radius: 0.4,
            inscribed_radius: 0.35,
            lookahead_distance: 0.5,
            goal_tolerance: 0.3,
            max_linear_vel: 1.0,
            max_angular_vel: 0.5, // Note: gets 2.5x boost for skid steering, so effective is ~1.25 rad/s
            blocked_timeout: 30.0,
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

    // Initialize classical navigation controller (planner + pursuit)
    let navigation_enabled = file_config.navigation.enabled && file_config.lidar.enabled;
    let mut navigation_controller: Option<NavigationController> = if navigation_enabled {
        let nav_config = NavigationConfig {
            costmap_width: file_config.navigation.costmap_width,
            costmap_height: file_config.navigation.costmap_height,
            costmap_resolution: file_config.navigation.costmap_resolution,
            inflation_radius: file_config.navigation.inflation_radius,
            inscribed_radius: file_config.navigation.inscribed_radius,
            planner: PlannerConfig::default(),
            pursuit: PursuitConfig {
                lookahead_distance: file_config.navigation.lookahead_distance,
                goal_tolerance: file_config.navigation.goal_tolerance,
                max_linear_velocity: file_config.navigation.max_linear_vel,
                max_angular_velocity: file_config.navigation.max_angular_vel,
                ..PursuitConfig::default()
            },
            replan_distance: 0.5,
            blocked_timeout: file_config.navigation.blocked_timeout,
        };
        info!(
            costmap = format!("{}x{}m @ {}m", nav_config.costmap_width, nav_config.costmap_height, nav_config.costmap_resolution),
            "Classical navigation stack enabled (A* + pure pursuit)"
        );
        let mut nav = NavigationController::new(nav_config);

        // Set static goal if provided via CLI or config
        if let Some(goal) = autonomous_goal {
            use planner::Waypoint;
            nav.set_goal(Waypoint::new(goal[0], goal[1]));
            info!(x = goal[0], y = goal[1], "Static goal set on navigation controller");
        }

        Some(nav)
    } else {
        if !file_config.navigation.enabled {
            info!("Classical navigation disabled (using policy)");
        } else if !file_config.lidar.enabled {
            info!("Classical navigation requires LiDAR (using policy)");
        }
        None
    };

    // Auto-start autonomous mode if we have a goal and navigation is ready
    let auto_start_autonomous = navigation_controller.as_ref()
        .map(|nav| nav.current_task().is_some())
        .unwrap_or(false)
        || (autonomous_goal.is_some() && loaded_policy.is_some());

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

    // In sim mode, auto-start autonomous if we have a goal (already boots into Idle)
    if args.sim {
        info!("Sim mode: booted into Idle");

        if auto_start_autonomous {
            state_machine.transition(state::Event::AutonomousRequest);
            info!("Auto-starting Autonomous mode (goal provided)");
        }
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
        roll: 0.0,
        pitch: 0.0,
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
    // watch for LiDAR point cloud streaming (created early for RtcServer, populated later)
    let (lidar_tx, lidar_rx) = watch::channel::<Option<lidar::PointCloud>>(None);
    // watch for costmap streaming (created early for RtcServer, populated from control loop)
    let (costmap_tx, costmap_rx) = watch::channel::<Option<CostmapSnapshot>>(None);
    // watch for obstacle streaming (extracted from costmap, populated from control loop)
    let (obstacles_tx, obstacles_rx) = watch::channel::<Option<Vec<costmap::Obstacle>>>(None);

    // Spawn teleop server (UDP)
    let teleop_config = TeleopConfig::default();
    let teleop = TeleopServer::new(teleop_config, cmd_tx.clone(), telemetry_rx.clone());

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Spawn WebRTC teleop server (primary browser teleop with video)
    let mut rtc_metrics: Option<std::sync::Arc<RtcMetrics>> = None;
    if args.rtc_port > 0 {
        let rtc_config = RtcConfig {
            signaling_port: args.rtc_port,
            pointcloud: teleop::pointcloud::PointCloudConfig {
                voxel_size: file_config.lidar.stream_voxel_size,
                max_points: file_config.lidar.stream_max_points,
            },
            ..Default::default()
        };
        // Use with_video_and_lidar to enable WebRTC video and point cloud streaming
        let mut rtc_server = RtcServer::with_video_and_lidar(
            rtc_config,
            cmd_tx.clone(),
            telemetry_rx.clone(),
            video_rx_rtc,
            lidar_rx.clone(),
        );
        // Enable costmap and obstacle streaming if navigation is enabled
        if navigation_enabled {
            rtc_server.set_costmap_rx(costmap_rx.clone());
            rtc_server.set_obstacles_rx(obstacles_rx.clone());
        }

        // Get metrics handle before spawning (for InfluxDB push)
        rtc_metrics = Some(rtc_server.metrics());

        tokio::spawn(async move {
            if let Err(e) = rtc_server.run().await {
                error!(?e, "WebRTC teleop server error");
            }
        });

        info!(
            port = args.rtc_port,
            voxel_size = file_config.lidar.stream_voxel_size,
            max_points = file_config.lidar.stream_max_points,
            "WebRTC teleop server started (low-latency + video)"
        );
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

    // Auto-detect and start cameras (unless disabled via CLI or config)
    let camera_enabled = !args.no_camera && file_config.camera.enabled;
    if camera_enabled {
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

        // Apply max_cameras limit from config (0 = unlimited)
        let max_cameras = file_config.camera.max_cameras;
        let cameras_to_start: Vec<_> = if max_cameras > 0 {
            cameras.into_iter().take(max_cameras).collect()
        } else {
            cameras
        };

        if cameras_to_start.is_empty() {
            info!("No cameras detected");
        } else {
            info!(count = cameras_to_start.len(), max = max_cameras, "Cameras detected");
            for cam in &cameras_to_start {
                info!(name = %cam.name, "  - {:?}", cam.camera_type);
            }

            // Start capture on detected cameras using the new async V4L2/nokhwa API
            // This avoids GStreamer's threading conflicts with tokio
            info!("Starting camera capture for {} camera(s)...", cameras_to_start.len());

            let mut cameras_started = 0u8;

            for (camera_id, cam) in cameras_to_start.iter().enumerate() {
                let camera_id = camera_id as u8;

                // Use the GStreamer-based camera API (sync, runs in dedicated thread)
                match camera::spawn_capture(cam, camera_config.clone()) {
                    Ok((frame_rx, _camera_handle)) => {
                        info!(
                            camera_id,
                            camera = %cam.name,
                            "Camera {} started: {}x{} @ {}fps (GStreamer)",
                            camera_id,
                            width,
                            height,
                            args.camera_fps
                        );

                        cameras_started += 1;

                        // Clone video tx channels for this camera task
                        let video_tx_rtc_camera = video_tx_rtc.clone();
                        let video_tx_udp_camera = video_tx_udp.clone();

                        // Bridge sync channel to async using spawn_blocking for proper blocking receive.
                        // This avoids the CPU-spinning issue with try_recv polling.
                        let (async_tx, mut async_rx) = mpsc::channel::<camera::Frame>(4);

                        // Blocking task: receive from sync channel, send to async channel
                        tokio::task::spawn_blocking(move || {
                            loop {
                                // Blocking receive - waits for frame without spinning
                                match frame_rx.recv() {
                                    Ok(frame) => {
                                        // Send to async channel (blocking if full, which throttles camera)
                                        if async_tx.blocking_send(frame).is_err() {
                                            break; // Receiver dropped
                                        }
                                    }
                                    Err(_) => break, // Camera thread stopped
                                }
                            }
                        });

                        // Async task: receive from async channel, forward to video servers
                        tokio::spawn(async move {
                            let mut frame_count: u64 = 0;
                            let mut total_bytes: u64 = 0;
                            let mut last_log = std::time::Instant::now();

                            while let Some(frame) = async_rx.recv().await {
                                let frame_size = frame.data.len();
                                // frame.data is already Arc<Vec<u8>> - zero-copy sharing
                                let video_frame = VideoFrame {
                                    camera_id,
                                    data: frame.data, // Arc clone is cheap (~8 bytes)
                                    width: frame.width,
                                    height: frame.height,
                                    sequence: frame.sequence,
                                    timestamp_ms: frame.timestamp_ms,
                                };

                                // Send to WebRTC (mpsc - queues frames)
                                // Use try_send to avoid backpressure stalling the camera
                                // Clone is cheap since data is Arc
                                match video_tx_rtc_camera.try_send(video_frame.clone()) {
                                    Ok(_) => {}
                                    Err(mpsc::error::TrySendError::Full(_)) => {
                                        // Channel full - drop frame (no WebRTC clients consuming)
                                    }
                                    Err(mpsc::error::TrySendError::Closed(_)) => {
                                        break; // Receiver dropped
                                    }
                                }

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
                                        "camera.async_stats"
                                    );
                                    frame_count = 0;
                                    total_bytes = 0;
                                    last_log = std::time::Instant::now();
                                }
                            }

                            debug!(camera_id, "Camera frame forwarder stopped");
                        });
                    }
                    Err(e) => {
                        warn!(camera_id, ?e, "Failed to start camera {} - skipping", camera_id);
                    }
                }
            }

            if cameras_started > 0 {
                // Mark camera as active in health status
                shared.lock().unwrap().health.camera_active = true;
                info!(count = cameras_started, "Camera capture started (GStreamer)");

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
    let ekf_config = EkfConfig {
        odom_linear_noise: file_config.localization.odom_linear_noise,
        odom_angular_noise: file_config.localization.odom_angular_noise,
        gps_noise_rtk_fixed: file_config.localization.gps_noise_rtk_fixed,
        gps_noise_rtk_float: file_config.localization.gps_noise_rtk_float,
        gps_noise_dgps: file_config.localization.gps_noise_dgps,
        gps_noise_standalone: file_config.localization.gps_noise_standalone,
        gps_heading_min_speed: file_config.localization.gps_heading_min_speed,
        imu_yaw_weight: file_config.localization.imu_yaw_weight,
        ..Default::default()
    };
    let mut pose_estimator = EkfPoseEstimator::with_config(ekf_config);

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

    // LiDAR reader (Livox Mid360) - uses lidar_tx created earlier for RtcServer
    // IMU channel for orientation data (roll/pitch) from the Mid360's built-in IMU
    let (imu_tx, imu_rx) = watch::channel::<Option<lidar::ImuData>>(None);
    let mut lidar_rx_local = lidar_rx.clone();

    // Command channel for runtime lidar control (power save mode)
    let lidar_cmd_tx: Option<tokio::sync::mpsc::Sender<LidarCommand>> = if file_config.lidar.enabled
    {
        let lidar_ip = file_config
            .lidar
            .lidar_ip
            .parse()
            .unwrap_or(std::net::Ipv4Addr::new(192, 168, 1, 100));
        let host_ip = file_config
            .lidar
            .host_ip
            .parse()
            .unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0));
        let lidar_config = LidarConfig {
            lidar_ip,
            host_ip,
            point_cloud_port: file_config.lidar.point_cloud_port,
            imu_port: file_config.lidar.imu_port,
            mounting_pitch_deg: file_config.lidar.mounting_pitch_deg,
            ..Default::default()
        };

        let lidar_reader = LidarReader::new(lidar_config);

        if file_config.lidar.power_save {
            // Power save mode: lidar starts immediately, only stops during Sleep
            let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<LidarCommand>(4);
            let _lidar_handle =
                lidar_reader.spawn_with_control(lidar_tx, imu_tx, cmd_rx, true);
            info!(
                ip = %file_config.lidar.lidar_ip,
                port = file_config.lidar.point_cloud_port,
                "LiDAR reader started (power save: stops only in Sleep mode)"
            );
            Some(cmd_tx)
        } else {
            // Always-on mode (legacy behavior)
            let _lidar_handle = lidar_reader.spawn_with_imu(lidar_tx, imu_tx);
            info!(
                ip = %file_config.lidar.lidar_ip,
                port = file_config.lidar.point_cloud_port,
                imu_port = file_config.lidar.imu_port,
                "LiDAR reader started with IMU (always on)"
            );
            None
        }
    } else {
        info!("LiDAR disabled in config");
        None
    };

    // Initialize SLAM processor as background task
    // This moves expensive scan matching (10-100ms) off the main control loop
    let (slam_scan_tx, slam_state_rx): (
        Option<mpsc::Sender<(lidar::PointCloud, Pose)>>,
        Option<watch::Receiver<SlamState>>,
    ) = if file_config.slam.enabled && file_config.lidar.enabled {
        let slam_config = SlamConfig {
            scan_match_resolution: file_config.slam.scan_match_resolution,
            keyframe_distance: file_config.slam.keyframe_distance,
            keyframe_rotation: file_config.slam.keyframe_rotation,
            loop_closure_threshold: file_config.slam.loop_closure_threshold,
            huber_threshold: file_config.slam.huber_threshold,
            loop_closure_max_disagreement_sigma: file_config.slam.loop_closure_max_disagreement_sigma,
            ..Default::default()
        };

        // Channel for sending scans to SLAM (bounded to prevent backpressure)
        let (scan_tx, mut scan_rx) = mpsc::channel::<(lidar::PointCloud, Pose)>(4);
        // Watch channel for receiving SLAM state updates
        let (state_tx, state_rx) = watch::channel(SlamState::default());

        // Spawn SLAM on a dedicated thread (NOT tokio::spawn) to avoid blocking the async runtime.
        // SLAM's process_scan() is CPU-intensive (10-100ms+) and would starve tokio worker threads.
        // Using std::sync::mpsc for thread-safe channel from async to sync.
        let (slam_thread_tx, slam_thread_rx) = std::sync::mpsc::channel::<(lidar::PointCloud, Pose)>();

        // Bridge async mpsc to sync mpsc
        tokio::spawn(async move {
            while let Some(item) = scan_rx.recv().await {
                if slam_thread_tx.send(item).is_err() {
                    break; // SLAM thread stopped
                }
            }
        });

        // Dedicated SLAM thread - completely isolated from tokio runtime
        std::thread::Builder::new()
            .name("slam".to_string())
            .spawn(move || {
                let mut slam = SlamProcessor::new(slam_config);
                info!("SLAM thread started (dedicated, non-blocking)");

                while let Ok((scan, odom_pose)) = slam_thread_rx.recv() {
                    // Update odometry before processing scan
                    slam.update_odometry(&odom_pose);

                    // Process scan (CPU-intensive: 10-100ms) - safe because we're on dedicated thread
                    if let Some(update) = slam.process_scan(&scan) {
                        if update.keyframe_added {
                            debug!(
                                keyframe_count = update.keyframe_count,
                                "SLAM keyframe added"
                            );
                        }
                        if update.loop_closure_detected {
                            info!(
                                loop_closure_count = update.loop_closure_count,
                                "SLAM loop closure detected"
                            );
                        }
                    }

                    // Send updated state to main loop
                    let state = SlamState {
                        pose: slam.pose(),
                        keyframe_count: slam.keyframe_count() as u32,
                        loop_closure_count: slam.loop_closure_count() as u32,
                        keyframe_poses: slam.keyframe_poses(),
                        pose_covariance: slam.pose_covariance_array(),
                    };
                    let _ = state_tx.send(state);
                }

                info!("SLAM thread shutting down");
            })
            .expect("Failed to spawn SLAM thread");

        info!("SLAM enabled (background task)");
        (Some(scan_tx), Some(state_rx))
    } else {
        if !file_config.slam.enabled {
            info!("SLAM disabled in config");
        }
        (None, None)
    };

    // Clone state receiver for main loop
    let mut slam_state_rx_local: Option<watch::Receiver<SlamState>> = slam_state_rx.clone();

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

    // Feed watchdog immediately if starting in autonomous mode
    // (otherwise is_timed_out() returns true on first iteration before we can feed it)
    if initial_mode == Mode::Autonomous {
        watchdog.feed();
    }

    let control_period = Duration::from_millis(10); // 100 Hz
    let mut last_tick = Instant::now();

    // Heartbeat tracking for debugging watchdog issues
    let mut heartbeat_count: u32 = 0;
    let mut last_heartbeat_log = Instant::now();

    // Current tool command (from teleop)
    let mut tool_command = types::ToolCommand::default();

    // Auto-sleep after idle/disabled timeout
    let sleep_timeout_secs = file_config.control.sleep_timeout_secs;
    let sleep_timeout = Duration::from_secs(sleep_timeout_secs);
    let mut idle_since = Instant::now();

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
    const POLICY_TIMEOUT: Duration = Duration::from_millis(30); // Hard timeout for spawn_blocking inference
    let mut policy_slow_count: u32 = 0;
    let mut last_policy_action: Option<policy::PolicyAction> = None;

    // Bounded task spawning for dispatch reports
    // Prevents unbounded task accumulation during long missions
    const MAX_DISPATCH_TASKS: usize = 100;
    let dispatch_semaphore = Arc::new(Semaphore::new(MAX_DISPATCH_TASKS));

    // Get tokio runtime handle for spawning async tasks from control loop thread
    let tokio_handle = tokio::runtime::Handle::current();

    // IMU attitude filter (fuses gyro + accelerometer for smooth roll/pitch)
    let mut attitude_filter = AttitudeFilter::new();

    // Pre-compute IMU mount transform (used for both attitude and yaw extraction)
    let mount_pitch_rad = file_config.lidar.mounting_pitch_deg.to_radians();
    let (sin_mount_pitch, cos_mount_pitch) = (mount_pitch_rad as f64).sin_cos();

    // Heartbeat thread for deadlock detection - runs on std::thread, not tokio
    // This helps diagnose if main loop is stuck vs tokio starvation
    let heartbeat_counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let heartbeat_counter_clone = heartbeat_counter.clone();
    std::thread::spawn(move || {
        let mut last_count = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let current = heartbeat_counter_clone.load(std::sync::atomic::Ordering::Relaxed);
            if current == last_count {
                eprintln!("HEARTBEAT: Main loop STUCK at iteration {}", current);
            } else {
                eprintln!(
                    "HEARTBEAT: Main loop running, iterations: {} (+{})",
                    current,
                    current - last_count
                );
            }
            last_count = current;
        }
    });

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

    // Spawn control loop on dedicated std::thread to isolate it from tokio scheduler
    // This ensures the 100Hz control loop runs independently of WebRTC/async task load
    let control_loop_handle = std::thread::Builder::new()
        .name("control-loop".to_string())
        .spawn(move || {
            // Checkpoint logging at end of each 100-iteration block
            let mut last_checkpoint = 0u64;

            loop {
        loop_count += 1;
        heartbeat_counter.store(loop_count, std::sync::atomic::Ordering::Relaxed);

        // Wait for next tick - use std::thread::sleep to avoid tokio scheduler dependency.
        // This prevents WebRTC/async task starvation from affecting motor control timing.
        // With the multi-threaded runtime (8 workers), other workers handle async tasks.
        let elapsed = last_tick.elapsed();
        if elapsed < control_period {
            std::thread::sleep(control_period - elapsed);
        }

        let dt = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();

        // Debug: warn if control loop is running slow
        if dt > 0.1 {
            warn!(dt_ms = format!("{:.0}", dt * 1000.0), "Control loop slow iteration");
        }

        // Tick simulation if in sim mode
        can_interface.tick(dt);

        // Read CAN frames (limited per iteration to prevent cascade delays)
        // VESCs send ~1kHz status each, so 4 VESCs = 4000 frames/sec.
        // At 100Hz loop, expect ~40 frames/iteration. Cap at 100 to bound latency.
        // Process all frames with a single mutex acquisition to minimize contention.
        const MAX_CAN_FRAMES_PER_ITERATION: usize = 100;

        // Collect frames first (no mutex needed for read)
        let mut frames: Vec<can::Frame> = Vec::with_capacity(MAX_CAN_FRAMES_PER_ITERATION);
        while frames.len() < MAX_CAN_FRAMES_PER_ITERATION {
            match can_interface.recv() {
                Ok(Some(frame)) => frames.push(frame),
                Ok(None) => break, // No more frames
                Err(_) => break,   // Error reading
            }
        }

        // Process all collected frames with a single mutex acquisition
        if !frames.is_empty() {
            let mut state = shared.lock().unwrap();
            for frame in &frames {
                state.drivetrain.process_frame(frame);
                state.tool_registry.process_frame(frame);
            }
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
                        info!(
                            requested_mode = ?mode,
                            current_mode = ?state.state_machine.mode(),
                            "SetMode command processing"
                        );
                        let current = state.state_machine.mode();
                        let event = match mode {
                            Mode::Disabled => Event::Disable, // old consoles send byte 0 → maps to Idle now
                            Mode::Idle => {
                                // Wake from Sleep, otherwise no-op (already Idle or Disable returns to Idle)
                                if current == Mode::Sleep {
                                    Event::Wake
                                } else {
                                    Event::Disable // Disable is a no-op when already Idle
                                }
                            }
                            Mode::Teleop => {
                                // Feed watchdog when entering Teleop to prevent immediate timeout
                                watchdog.feed();
                                Event::TeleopCommand
                            }
                            Mode::Autonomous => {
                                // Feed watchdog when entering Autonomous to prevent immediate timeout
                                watchdog.feed();
                                Event::AutonomousRequest
                            }
                            Mode::EStop => Event::EStop,
                            Mode::Sleep => Event::Sleep,
                            _ => continue,
                        };
                        state.state_machine.transition(event);
                    }
                    Command::Heartbeat => {
                        watchdog.feed();
                        heartbeat_count += 1;
                        // Log heartbeat rate every 10 seconds when in teleop
                        // Note: we already hold `state` from line 1621, use it directly
                        if last_heartbeat_log.elapsed() > Duration::from_secs(10) {
                            if state.state_machine.mode() == Mode::Teleop {
                                let rate = heartbeat_count as f64 / last_heartbeat_log.elapsed().as_secs_f64();
                                info!(count = heartbeat_count, rate = format!("{:.1}/s", rate), "Heartbeats received (10s window)");
                            }
                            heartbeat_count = 0;
                            last_heartbeat_log = Instant::now();
                        }
                    }
                    Command::Tool(tc) => {
                        watchdog.feed();
                        tool_command = tc;
                    }
                    Command::Twist(_) => unreachable!(), // Already handled above
                    Command::LidarToggle(_) => {} // Handled per-connection in WebRTC
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

                        // Set task on navigation controller if using classical nav
                        if let Some(ref mut nav) = navigation_controller {
                            nav.set_task(assignment.clone());
                        }

                        // Also set on legacy dispatch task for policy fallback
                        let task = DispatchedTask::from_assignment(assignment);
                        *dispatch_task_clone.lock().unwrap() = Some(task);

                        // Transition to autonomous mode to execute the task
                        let mut state = shared.lock().unwrap();
                        match state.state_machine.mode() {
                            Mode::Idle => {
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

                        // Cancel on navigation controller
                        if let Some(ref mut nav) = navigation_controller {
                            if nav.current_task().map(|t| t.task_id == task_id).unwrap_or(false) {
                                nav.cancel();
                            }
                        }

                        // Also handle legacy dispatch task
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
                        let elapsed_ms = watchdog.elapsed_since_last_command()
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        warn!(
                            elapsed_ms,
                            heartbeat_count,
                            "Command watchdog timeout in teleop mode (no commands received)"
                        );
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
                                let sem = dispatch_semaphore.clone();
                                tokio_handle.spawn(async move {
                                    // Acquire permit (bounded concurrency)
                                    let _permit = sem.acquire().await;
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

        // Auto-sleep after idle/disabled timeout
        if sleep_timeout_secs > 0 {
            let mode = shared.lock().unwrap().state_machine.mode();
            if matches!(mode, Mode::Idle)
                && idle_since.elapsed() > sleep_timeout
            {
                info!(
                    elapsed_secs = idle_since.elapsed().as_secs(),
                    mode = ?mode,
                    "Auto-sleep timeout reached"
                );
                let mut state = shared.lock().unwrap();
                state.state_machine.transition(Event::Sleep);
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
                // Autonomous mode: use classical navigation (A* + pursuit) or policy
                let current_pose = if args.sim {
                    can_interface.pose()
                } else {
                    pose_estimator.pose()
                };

                // Classical navigation stack (preferred when enabled)
                if let Some(ref mut nav) = navigation_controller {
                    // Update navigation controller
                    let nav_output = nav.update(&current_pose, dt);

                    // Debug: log navigation state on first few iterations or state changes
                    if loop_count < 5 || loop_count % 100 == 0 {
                        debug!(
                            state = ?nav_output.state,
                            dist = format!("{:.2}", nav_output.distance_to_goal),
                            twist_linear = format!("{:.3}", nav_output.twist.linear),
                            twist_angular = format!("{:.3}", nav_output.twist.angular),
                            pose_x = format!("{:.2}", current_pose.x),
                            pose_y = format!("{:.2}", current_pose.y),
                            pose_theta = format!("{:.2}", current_pose.theta),
                            "Navigation update"
                        );
                    }

                    // Handle navigation state transitions
                    match nav_output.state {
                        NavigationState::GoalReached => {
                            // Advance to next waypoint or complete task
                            let task_complete = nav.advance_waypoint();

                            if let Some((task_id, progress, waypoint, lap)) = nav.task_progress() {
                                // Report progress to dispatch service
                                if let Some(ref client) = dispatch_client {
                                    let client_clone = client.clone();
                                    let sem = dispatch_semaphore.clone();
                                    tokio_handle.spawn(async move {
                                        let _permit = sem.acquire().await;
                                        let _ = client_clone.report_progress(task_id, progress, waypoint, lap).await;
                                    });
                                }
                            }

                            if task_complete {
                                // Task complete - report and return to idle
                                if let Some(task) = nav.current_task() {
                                    let task_id = task.task_id;
                                    let laps = task.lap;
                                    info!(task_id = %task_id, laps, "Navigation task complete");

                                    if let Some(ref client) = dispatch_client {
                                        let client_clone = client.clone();
                                        let sem = dispatch_semaphore.clone();
                                        tokio_handle.spawn(async move {
                                            let _permit = sem.acquire().await;
                                            let _ = client_clone.report_complete(task_id, laps).await;
                                        });
                                    }
                                }

                                let mut state = shared.lock().unwrap();
                                state.state_machine.transition(Event::CommandTimeout);
                            }
                        }
                        NavigationState::Failed => {
                            // Navigation failed - report and return to idle
                            if let Some(task) = nav.current_task() {
                                let task_id = task.task_id;
                                let error = nav_output.error.clone().unwrap_or_else(|| "Navigation failed".to_string());
                                error!(task_id = %task_id, error = %error, "Navigation failed");

                                if let Some(ref client) = dispatch_client {
                                    let client_clone = client.clone();
                                    let sem = dispatch_semaphore.clone();
                                    tokio_handle.spawn(async move {
                                        let _permit = sem.acquire().await;
                                        let _ = client_clone.report_failed(task_id, &error).await;
                                    });
                                }
                            }

                            nav.cancel();
                            let mut state = shared.lock().unwrap();
                            state.state_machine.transition(Event::CommandTimeout);
                        }
                        NavigationState::ObstacleStopped => {
                            debug!("Stopped for obstacle");
                        }
                        _ => {}
                    }

                    // Feed watchdog on active navigation (any non-idle/failed state)
                    match nav_output.state {
                        NavigationState::Planning
                        | NavigationState::Replanning
                        | NavigationState::Following
                        | NavigationState::ObstacleStopped => {
                            watchdog.feed();
                        }
                        _ => {}
                    }

                    (nav_output.twist, false)
                } else {
                    // Fallback: Policy-based navigation
                    let goal = {
                        let task_guard = current_dispatch_task.lock().unwrap();
                        if let Some(ref task) = *task_guard {
                            task.current_goal()
                        } else {
                            autonomous_goal
                        }
                    };

                    if let (Some(policy), Some(goal)) = (&loaded_policy, goal) {
                        let linear_vel = commanded_twist.linear;
                        let angular_vel = commanded_twist.angular;

                        let obs = PolicyObservation::from_raw(
                            current_pose.x, current_pose.y, current_pose.theta,
                            linear_vel, angular_vel,
                            goal[0], goal[1],
                            &norm_config,
                        );

                        let policy_start = Instant::now();
                        let policy_clone = policy.clone();
                        let obs_clone = obs.clone();

                        // Use block_on for policy inference from dedicated control thread
                        // This blocks the thread but that's fine since we're on a dedicated thread
                        let handle = tokio_handle.clone();
                        let inference_result = handle.block_on(async move {
                            tokio::time::timeout(
                                POLICY_TIMEOUT,
                                tokio::task::spawn_blocking(move || policy_clone.infer(&obs_clone)),
                            ).await
                        });

                        let inference_time = policy_start.elapsed();

                        let action = match inference_result {
                            Ok(Ok(Ok(action))) => {
                                if inference_time > POLICY_ERROR_THRESHOLD {
                                    policy_slow_count += 1;
                                    error!(duration_ms = inference_time.as_millis(), "Policy critically slow");
                                } else if inference_time > POLICY_WARN_THRESHOLD {
                                    policy_slow_count += 1;
                                    warn!(duration_ms = inference_time.as_millis(), "Policy slow");
                                } else {
                                    policy_slow_count = 0;
                                }
                                last_policy_action = Some(action);
                                Some(action)
                            }
                            Ok(Ok(Err(e))) => { warn!(?e, "Policy error"); last_policy_action }
                            Ok(Err(e)) => { error!("Policy panic: {}", e); last_policy_action }
                            Err(_) => { warn!("Policy timeout"); policy_slow_count += 1; last_policy_action }
                        };

                        if let Some(action) = action {
                            watchdog.feed();
                            let twist = action.to_twist(autonomous_max_linear, autonomous_max_angular);

                            // Check waypoint reached (policy mode)
                            let dx = goal[0] - current_pose.x;
                            let dy = goal[1] - current_pose.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 0.5 {
                                let mut task_guard = current_dispatch_task.lock().unwrap();
                                if let Some(ref mut task) = *task_guard {
                                    info!(waypoint = task.current_waypoint, "Waypoint reached");

                                    if let Some(ref client) = dispatch_client {
                                        let progress = task.progress_percent();
                                        let task_id = task.task_id;
                                        let waypoint = task.current_waypoint as i32;
                                        let lap = task.lap;
                                        let client_clone = client.clone();
                                        let sem = dispatch_semaphore.clone();
                                        tokio_handle.spawn(async move {
                                            let _permit = sem.acquire().await;
                                            let _ = client_clone.report_progress(task_id, progress, waypoint, lap).await;
                                        });
                                    }

                                    let task_complete = task.advance();
                                    if task_complete {
                                        let task_id = task.task_id;
                                        let laps = task.lap;
                                        info!(task_id = %task_id, "Task complete");

                                        if let Some(ref client) = dispatch_client {
                                            let client_clone = client.clone();
                                            let sem = dispatch_semaphore.clone();
                                            tokio_handle.spawn(async move {
                                                let _permit = sem.acquire().await;
                                                let _ = client_clone.report_complete(task_id, laps).await;
                                            });
                                        }

                                        drop(task_guard);
                                        *current_dispatch_task.lock().unwrap() = None;

                                        let mut state = shared.lock().unwrap();
                                        state.state_machine.transition(Event::CommandTimeout);
                                    }
                                }
                            }

                            (twist, false)
                        } else {
                            warn!("Policy failed, no fallback");
                            (Twist::default(), false)
                        }
                    } else {
                        debug!("No policy or goal");
                        (Twist::default(), false)
                    }
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
        // VESC "Invert Motor Direction" inverts both duty AND tachometer readings,
        // so right-side values already use the correct sign convention (positive = forward).
        let tach: [i32; 4] = [
            vesc_states[0].status5.tachometer,  // FL
            vesc_states[1].status5.tachometer,  // FR
            vesc_states[2].status5.tachometer,  // RL
            vesc_states[3].status5.tachometer,  // RR
        ];
        let (dx, dy, dtheta) = odometry.update(tach);

        // Extract body-frame yaw rate from IMU (if available)
        let gyro_z_body: Option<f32> = imu_rx.borrow().as_ref().map(|imu| {
            (-imu.gyro_x as f64 * sin_mount_pitch + imu.gyro_z as f64 * cos_mount_pitch) as f32
        });

        // Update EKF prediction with odometry + IMU yaw
        pose_estimator.predict(dx, dy, dtheta, gyro_z_body, dt);

        // Check for SLAM pose updates (non-blocking)
        // SLAM runs in background task, we just check for new state.
        // When new SLAM data arrives, feed it as a measurement update into the EKF.
        let slam_state: Option<SlamState> = if let Some(ref mut rx) = slam_state_rx_local {
            if rx.has_changed().unwrap_or(false) {
                let state = rx.borrow_and_update().clone();
                // Feed SLAM correction into EKF as measurement update
                if let Some(ref cov_arr) = state.pose_covariance {
                    pose_estimator.update_slam_from_array(&state.pose, cov_arr);
                }
                Some(state)
            } else {
                Some(rx.borrow().clone())
            }
        } else {
            None
        };

        // Check for GPS updates
        if gps_rx.has_changed().unwrap_or(false) {
            let gps_state = gps_rx.borrow_and_update().clone();
            pose_estimator.update_gps(&gps_state);
            if let Some(ref coord) = gps_state.coord {
                debug!(
                    lat = coord.lat,
                    lon = coord.lon,
                    sats = gps_state.satellites,
                    fix = gps_state.fix_quality,
                    "GPS update"
                );
            }
        }

        // Check for LiDAR updates
        // Clone the scan OUTSIDE the shared lock to avoid blocking issues with rerun SDK
        let lidar_scan_clone = if lidar_rx_local.has_changed().unwrap_or(false) {
            lidar_rx_local.borrow_and_update().clone()
        } else {
            None
        };

        // Process LiDAR scan (navigation costmap only - recorder moved to async task)
        if let Some(ref scan) = lidar_scan_clone {
            // Send scan to SLAM background task (non-blocking)
            if let Some(ref tx) = slam_scan_tx {
                let _ = tx.try_send((scan.clone(), pose_estimator.pose()));
            }

            // Update costmap for navigation controller
            let costmap_start = Instant::now();
            if let Some(ref mut nav) = navigation_controller {
                let robot_tf = Transform2D::from_pose(&pose_estimator.pose());
                nav.update_costmap(&scan, &robot_tf);
                // Publish costmap snapshot and obstacles for console streaming
                let _ = costmap_tx.send(Some(nav.costmap_snapshot()));
                let _ = obstacles_tx.send(Some(nav.extract_obstacles()));
            }
            let costmap_ms = costmap_start.elapsed().as_millis();

            if costmap_ms > 50 {
                warn!(costmap_ms = costmap_ms, points = scan.points.len(), "Costmap processing slow");
            }

            // NOTE: recorder.log_lidar_scan() removed - was causing deadlock with rerun/rayon
            // TODO: Move to async recording task
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

        // Get pose from EKF (unified: fuses odom + GPS + IMU + SLAM)
        // In sim mode, use simulation ground truth instead
        drop(state);
        let pose = if args.sim {
            can_interface.pose()
        } else {
            pose_estimator.pose()
        };

        // Build SLAM status if enabled
        let slam_status = slam_state.as_ref().map(|state| SlamStatus {
            pose,
            confidence: 0.8, // TODO: track actual match score
            keyframe_count: state.keyframe_count,
            loop_closure_count: state.loop_closure_count,
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
        state.health.slam_running = slam_scan_tx.is_some();
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

        // Compute roll/pitch from IMU using Kalman filter (fuses gyro + accelerometer)
        // The IMU is mounted tilted forward, so we transform readings to body frame first.
        let (roll, pitch) = if let Some(imu) = imu_rx.borrow().as_ref() {
            // Transform both accel and gyro from sensor frame to body frame.
            // Sensor is pitched forward by mounting_pitch_deg degrees, so we rotate back.
            let mount_pitch_rad = file_config.lidar.mounting_pitch_deg.to_radians();
            let (sin_p, cos_p) = mount_pitch_rad.sin_cos();

            // Livox Mid360 IMU outputs acceleration in g's, convert to m/s²
            const GRAVITY: f32 = 9.81;
            let accel_x_ms2 = imu.accel_x * GRAVITY;
            let accel_y_ms2 = imu.accel_y * GRAVITY;
            let accel_z_ms2 = imu.accel_z * GRAVITY;

            let accel = [
                accel_x_ms2 * cos_p + accel_z_ms2 * sin_p,
                accel_y_ms2,
                -accel_x_ms2 * sin_p + accel_z_ms2 * cos_p,
            ];
            let gyro = [
                imu.gyro_x * cos_p + imu.gyro_z * sin_p,
                imu.gyro_y,
                -imu.gyro_x * sin_p + imu.gyro_z * cos_p,
            ];

            let att = attitude_filter.update(gyro, accel, dt);
            (att.roll, att.pitch)
        } else {
            (0.0, 0.0)
        };

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
                system_current: state.drivetrain.total_current() as f64,
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
            roll,
            pitch,
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

        // Collect WebRTC channel metrics (take and reset counters)
        let webrtc_metrics = if let Some(ref rtc) = rtc_metrics {
            let (telem_sent, telem_us, telem_max) = rtc.telemetry.take();
            let (video_sent, video_us, video_max) = rtc.video.take();
            let (cloud_sent, cloud_us, cloud_max) = rtc.pointcloud.take();
            metrics::WebRtcChannelMetrics {
                telemetry_sent: telem_sent,
                telemetry_send_us: telem_us,
                telemetry_send_max_us: telem_max,
                video_sent,
                video_send_us: video_us,
                video_send_max_us: video_max,
                pointcloud_sent: cloud_sent,
                pointcloud_send_us: cloud_us,
                pointcloud_send_max_us: cloud_max,
            }
        } else {
            metrics::WebRtcChannelMetrics::default()
        };

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
            webrtc: webrtc_metrics,
            ..Default::default()
        };
        drop(gps_state);
        let _ = metrics_tx.send(metrics_snapshot);

        // Capture data needed for recording BEFORE releasing mutex
        // (rerun uses rayon internally, which deadlocks if called while holding a mutex)
        let commanded_twist = state.commanded_twist;
        let current_mode = telemetry.mode;
        let led_frame = if current_mode != last_mode {
            Some(state.state_machine.led_command().to_frame())
        } else {
            None
        };

        // CRITICAL: Release mutex before any recorder calls to avoid rayon deadlock
        // See docs/postmortem-2026-01-control-loop-deadlock.md
        drop(state);

        // Record telemetry to Rerun session (MUST be outside mutex)
        let recorder_start = Instant::now();
        let time_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        recorder.set_time(time_secs);

        // Log all telemetry data
        let _ = recorder.log_pose(&pose);
        let _ = recorder.log_velocity(&commanded_twist, &twist);
        let _ = recorder.log_motors(&motor_currents, &motor_temps);
        let _ = recorder.log_power(telemetry.power.battery_voltage, telemetry.power.system_current);
        let _ = recorder.log_odometry(dx, dy, dtheta);

        // Log SLAM data if enabled
        if let Some(ref slam) = slam_state {
            let _ = recorder.log_slam_trajectory(&pose);
            let _ = recorder.log_slam_keyframes(&slam.keyframe_poses);
            let _ = recorder.log_slam_status(
                slam.keyframe_count as usize,
                slam.loop_closure_count as usize,
                0.0, // TODO: track match score
            );
        }

        // Warn if recorder is slow
        let recorder_elapsed = recorder_start.elapsed();
        if recorder_elapsed > Duration::from_millis(50) {
            warn!(ms = recorder_elapsed.as_millis(), "Recorder logging slow");
        }

        // Log mode changes and update LEDs
        if current_mode != last_mode {
            // Reset sleep timer when entering Idle (restart countdown)
            if matches!(current_mode, Mode::Idle) {
                idle_since = Instant::now();
            }

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

            // Handle Sleep mode transitions
            if current_mode == Mode::Sleep && last_mode != Mode::Sleep {
                // Entering Sleep - stop lidar and end recording
                info!("Entering Sleep mode - shutting down sensors");
                recorder.end_session();
                if let Some(ref tx) = lidar_cmd_tx {
                    if let Err(e) = tx.try_send(LidarCommand::Stop) {
                        warn!(?e, "Failed to send lidar stop command for sleep");
                    } else {
                        info!("Lidar stopped (entering sleep mode)");
                    }
                }
            } else if last_mode == Mode::Sleep && current_mode != Mode::Sleep {
                // Waking from Sleep - restart lidar
                info!("Waking from Sleep mode - starting lidar");
                if let Some(ref tx) = lidar_cmd_tx {
                    if let Err(e) = tx.try_send(LidarCommand::Start) {
                        warn!(?e, "Failed to send lidar start command on wake");
                    } else {
                        info!("Lidar started (waking from sleep)");
                    }
                }
            }

            let _ = recorder.log_mode(current_mode);

            // Send LED command for new mode (frame was captured before mutex release)
            if let Some(frame) = led_frame {
                if let Err(e) = can_interface.send(&frame) {
                    debug!(?e, "Failed to send LED command");
                } else {
                    debug!(?current_mode, "LED command sent for mode change");
                }
            }

            last_mode = current_mode;
        }

        // Log tool state if active
        if let Some(ref status) = telemetry.tool_status {
            let _ = recorder.log_tool(&status.name, status.position, status.current);
        }

            } // end of loop
        }).expect("Failed to spawn control loop thread");

    // Keep async main alive so tokio continues running WebRTC/dashboard tasks
    // The control loop runs on its own dedicated thread
    control_loop_handle.join().expect("Control loop thread panicked");

    Ok(())
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

