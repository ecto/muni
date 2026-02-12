//! Daemon initialization — builds all subsystems and returns a [`DaemonContext`]
//! that carries every piece of state the control loop thread needs.

use crate::can_iface::CanInterface;
use crate::cli::Args;
use crate::config::FileConfig;
use crate::state_types::*;
use anyhow::Result;
use can::vesc::Drivetrain;
use can::Bus;
use collision_monitor::{CollisionMonitor, CollisionMonitorConfig};
use control::{ChassisParams, DiffDriveMixer, Limits, RateLimiter, Watchdog};
use control_loop::{ControllerType, NavigationConfig, NavigationController, RecoveryConfig};
use costmap::{CostmapSnapshot, ObstacleTracker, TrackerConfig};
use dispatch::{DispatchClient, DispatchEvent};
use gps::{Config as GpsConfig, GpsReader, GpsState};
use hal::EStopEvent;
use lidar::{Config as LidarConfig, LidarCommand, LidarReader, LidarStatus};
use localization::{AttitudeFilter, EkfConfig, EkfPoseEstimator, WheelOdometry};
use metrics::{
    Config as MetricsConfig, DiscoveryClient, DiscoveryConfig, MetricsPusher, MetricsSnapshot,
    SystemMetricsCollector,
};
use planner::PlannerConfig;
use policy::{NormalizationConfig, Policy, PolicyManager};
use pursuit::PursuitConfig;
use recording::{Config as RecordingConfig, Recorder};
use sim::SimBus;
use slam::{SlamConfig, SlamProcessor, SlamState};
use state::{Event, StateMachine};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teleop::rtc::{RtcConfig, RtcMetrics, RtcServer};
use teleop::video::{H264Frame, VideoConfig, VideoServer};
use teleop::{Config as TeleopConfig, Server as TeleopServer, Telemetry};
use tokio::sync::{mpsc, watch, Semaphore};
use tools::eth_discovery::EthDiscovery;
use tools::Registry as ToolRegistry;
use tracing::{debug, error, info, warn};
use types::{Command, Mode, Pose, PowerStatus, SubsystemHealth, Twist};
use ui::{Config as UiConfig, Dashboard};

/// Every piece of state the control loop thread needs after initialization.
pub(crate) struct DaemonContext {
    // -- CAN / hardware --
    pub(crate) can_interface: CanInterface,
    pub(crate) is_sim: bool,
    pub(crate) shared: Arc<Mutex<SharedState>>,

    // -- command & telemetry channels --
    pub(crate) cmd_tx: mpsc::Sender<Command>,
    pub(crate) cmd_rx: mpsc::Receiver<Command>,
    pub(crate) telemetry_tx: watch::Sender<Telemetry>,
    pub(crate) metrics_tx: watch::Sender<MetricsSnapshot>,

    // -- control --
    pub(crate) mixer: DiffDriveMixer,
    pub(crate) rate_limiter: RateLimiter,
    pub(crate) collision_monitor: CollisionMonitor,
    pub(crate) watchdog: Watchdog,

    // -- localization --
    pub(crate) odometry: WheelOdometry,
    pub(crate) pose_estimator: EkfPoseEstimator,
    pub(crate) gps_rx: watch::Receiver<GpsState>,
    pub(crate) lidar_rx: watch::Receiver<Option<lidar::PointCloud>>,
    pub(crate) imu_rx: watch::Receiver<Option<lidar::ImuData>>,
    pub(crate) lidar_cmd_tx: Option<tokio::sync::mpsc::Sender<LidarCommand>>,
    pub(crate) lidar_enabled: bool,
    pub(crate) lidar_status_rx: watch::Receiver<LidarStatus>,

    // -- SLAM --
    pub(crate) slam_scan_tx: Option<mpsc::Sender<(lidar::PointCloud, Pose, Option<slam::PreintegratedDelta>)>>,
    pub(crate) slam_state_rx: Option<watch::Receiver<SlamState>>,

    // -- navigation --
    pub(crate) navigation_controller: Option<NavigationController>,
    pub(crate) obstacle_tracker: ObstacleTracker,
    pub(crate) costmap_tx: watch::Sender<Option<CostmapSnapshot>>,
    pub(crate) obstacles_tx: watch::Sender<Option<Vec<costmap::TrackedObstacle>>>,
    pub(crate) path_tx: watch::Sender<Option<teleop::path_stream::PathSnapshot>>,

    // -- autonomous / policy --
    pub(crate) loaded_policy: Option<Policy>,
    pub(crate) autonomous_goal: Option<[f64; 2]>,
    pub(crate) autonomous_max_linear: f64,
    pub(crate) autonomous_max_angular: f64,
    pub(crate) norm_config: NormalizationConfig,

    // -- dispatch --
    pub(crate) dispatch_client: Option<DispatchClient>,
    pub(crate) dispatch_rx: Option<tokio::sync::broadcast::Receiver<DispatchEvent>>,
    pub(crate) current_dispatch_task: Arc<Mutex<Option<DispatchedTask>>>,

    // -- e-stop --
    pub(crate) estop_rx: mpsc::Receiver<EStopEvent>,
    pub(crate) estop_require_confirmation: bool,

    // -- recording / metrics --
    pub(crate) recorder: Recorder,
    pub(crate) recording_enabled: bool,
    pub(crate) rtc_metrics: Option<std::sync::Arc<RtcMetrics>>,

    // -- WebRTC peer tracking (for auto-sleep suppression) --
    pub(crate) peer_count: Arc<std::sync::atomic::AtomicU32>,

    // -- config-derived scalars --
    pub(crate) sleep_timeout_secs: u64,
    pub(crate) mount_pitch_rad: f32,
    pub(crate) discovery_enabled: bool,
    pub(crate) dispatch_enabled: bool,

    // -- tokio handle for spawning async work from the control thread --
    pub(crate) tokio_handle: tokio::runtime::Handle,

    // -----------------------------------------------------------------------
    // Loop-local mutable state (initialized once, mutated every iteration)
    // -----------------------------------------------------------------------
    pub(crate) loop_count: u64,
    pub(crate) heartbeat_counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub(crate) heartbeat_count: u32,
    pub(crate) last_heartbeat_log: Instant,
    pub(crate) tool_command: types::ToolCommand,
    pub(crate) idle_since: Instant,
    pub(crate) last_mode: Mode,
    pub(crate) sys_metrics: SystemMetricsCollector,
    pub(crate) last_sys_refresh: Instant,
    pub(crate) can_errors_active: bool,
    pub(crate) can_backoff_active: bool,
    pub(crate) can_error_count: u64,
    pub(crate) consecutive_can_errors: u32,
    pub(crate) last_policy_action: Option<policy::PolicyAction>,
    pub(crate) last_policy_action_time: Option<Instant>,
    pub(crate) dispatch_semaphore: Arc<Semaphore>,
    pub(crate) attitude_filter: AttitudeFilter,
    pub(crate) imu_preintegrator: slam::ImuPreintegrator,
    pub(crate) last_tick: Instant,
    pub(crate) control_period: Duration,
    pub(crate) hardware_estop_latched: bool,

    // -- control loop timing metrics --
    pub(crate) dt_min_ms: f64,
    pub(crate) dt_max_ms: f64,

    // -- subsystem liveness timestamps --
    pub(crate) last_gps_update: Option<Instant>,
    pub(crate) last_lidar_update: Option<Instant>,
    pub(crate) last_slam_update: Option<Instant>,
    pub(crate) last_subsystem_check: Instant,

    // -- dispatch/discovery connection tracking --
    pub(crate) dispatch_connected_live: bool,
    pub(crate) heartbeat_stalled: std::sync::Arc<std::sync::atomic::AtomicBool>,

    // -- sequence tracking (shared with teleop Server for ACK fields) --
    pub(crate) seq_tracker: std::sync::Arc<std::sync::Mutex<teleop::SequenceTracker>>,

    // -- Ethernet tool discovery --
    pub(crate) eth_discovery: Option<EthDiscovery>,
}

/// Run all initialization and return the context the control loop needs plus
/// the log guard (must outlive the daemon).
pub(crate) async fn initialize(
    args: Args,
) -> Result<(DaemonContext, tracing_appender::non_blocking::WorkerGuard)> {
    // Initialize logging with rolling file appender
    // The _guard must be held for the lifetime of the program to ensure logs are flushed
    let log_guard = crate::logging::init_logging(&args.log_dir, &args.log_level, args.console)?;

    // Load configuration file
    let file_config = FileConfig::load(&args.config)?;
    info!(path = %args.config.display(), "Loaded config");

    // Resolve rover_id from config file
    let rover_id = file_config.identity.rover_id.clone();

    // CAN interface: CLI arg > default "can0"
    let can_iface = args.can_interface.as_deref().unwrap_or("can0");

    let is_sim = args.sim || args.remote_can.is_some();

    if args.remote_can.is_some() {
        info!("Starting bvrd in REMOTE CAN mode (sim-bridge)");
    } else if args.vesc_serial.is_some() {
        info!("Starting bvrd in VESC SERIAL mode (USB CAN gateway)");
    } else if args.sim {
        info!("Starting bvrd in SIMULATION mode");
    } else {
        info!(config = ?args.config, can = %can_iface, rover = %rover_id, "Starting bvrd");
    }

    // Initialize telemetry recorder (paused - starts on Teleop/Autonomous entry)
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
            endpoint: args
                .metrics_endpoint
                .clone()
                .unwrap_or(file_config.metrics.endpoint.clone()),
            interval_hz: args
                .metrics_hz
                .unwrap_or(file_config.metrics.interval_hz),
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
        let discovery_endpoint = args
            .discovery_endpoint
            .clone()
            .unwrap_or(file_config.discovery.endpoint.clone());
        let discovery_rover_id = file_config
            .discovery
            .rover_id
            .clone()
            .unwrap_or(rover_id.clone());
        let discovery_rover_name = file_config
            .discovery
            .rover_name
            .clone()
            .unwrap_or_else(|| {
                discovery_rover_id
                    .replace("bvr-", "Beaver-")
                    .replace("frog-", "Frog-")
            });

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
            let policy_dir = args
                .policy_dir
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
    let navigation_controller: Option<NavigationController> = if navigation_enabled {
        let controller_type = match file_config.navigation.controller_type.as_str() {
            "mppi" => ControllerType::Mppi,
            _ => ControllerType::PurePursuit,
        };
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
            mppi: mppi::MppiConfig {
                num_samples: file_config.navigation.mppi_num_samples,
                horizon_steps: file_config.navigation.mppi_horizon_steps,
                dt: file_config.navigation.mppi_dt,
                temperature: file_config.navigation.mppi_temperature,
                max_linear_velocity: file_config.navigation.max_linear_vel,
                max_angular_velocity: file_config.navigation.max_angular_vel,
                ..mppi::MppiConfig::default()
            },
            controller_type,
            replan_distance: 0.5,
            blocked_timeout: file_config.navigation.blocked_timeout,
            smoother: Some(planner::SmootherConfig::default()),
            recovery: RecoveryConfig {
                backup_distance: file_config.navigation.recovery_backup_distance,
                backup_speed: file_config.navigation.recovery_backup_speed,
                spin_speed: file_config.navigation.recovery_spin_speed,
                wait_duration: file_config.navigation.recovery_wait_duration,
                max_recovery_cycles: file_config.navigation.max_recovery_cycles,
            },
        };
        let ctrl_name = match controller_type {
            ControllerType::PurePursuit => "pure pursuit",
            ControllerType::Mppi => "MPPI",
        };
        info!(
            costmap = format!(
                "{}x{}m @ {}m",
                nav_config.costmap_width, nav_config.costmap_height, nav_config.costmap_resolution
            ),
            controller = ctrl_name,
            "Classical navigation stack enabled (A* + {})",
            ctrl_name
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
    let auto_start_autonomous = navigation_controller
        .as_ref()
        .map(|nav| nav.current_task().is_some())
        .unwrap_or(false)
        || (autonomous_goal.is_some() && loaded_policy.is_some());

    // Initialize dispatch client for receiving mission waypoints
    let dispatch_enabled = !args.no_dispatch && file_config.dispatch.enabled;
    let dispatch_client = if dispatch_enabled {
        let dispatch_endpoint = args
            .dispatch_endpoint
            .clone()
            .unwrap_or(file_config.dispatch.endpoint.clone());

        info!(endpoint = %dispatch_endpoint, "Connecting to dispatch service");
        let client = DispatchClient::new(&dispatch_endpoint, &rover_id);
        Some(client)
    } else {
        info!("Dispatch service connection disabled");
        None
    };

    // Subscribe to dispatch events if enabled
    let dispatch_rx = dispatch_client.as_ref().map(|c| c.subscribe());

    // Current dispatched task state
    let current_dispatch_task: Arc<Mutex<Option<DispatchedTask>>> = Arc::new(Mutex::new(None));

    // Ethernet tool discovery (started early so CAN bridge can be auto-discovered)
    let eth_discovery = if file_config.eth_tools.enabled {
        let discovery = EthDiscovery::new(file_config.eth_tools.discovery_port);
        let eth_state = discovery.state();
        let port = file_config.eth_tools.discovery_port;
        tokio::spawn(EthDiscovery::run(eth_state, port));
        info!(port, "Ethernet tool discovery started");
        Some(discovery)
    } else {
        None
    };

    // Initialize CAN interface
    // Priority:
    //   1. --remote-can tcp://... (explicit CLI, for sim-bridge or manual bridge)
    //   2. --sim (SimBus)
    //   3. Auto-discover CanBridge via EthDiscovery (PoE ESP32 bridge)
    //   4. Fall back to SocketCAN can0
    let vesc_ids: [u8; 4] = args
        .vesc_ids
        .try_into()
        .expect("Need exactly 4 VESC IDs");
    let can_interface = if let Some(ref url) = args.remote_can {
        info!(url, "Using remote CAN (sim-bridge)");
        CanInterface::connect_remote(url).await?
    } else if let Some(ref port) = args.vesc_serial {
        let gateway_id = args.vesc_gateway_id.unwrap_or(vesc_ids[0]);
        info!(port, gateway_id, "Using VESC serial CAN gateway");
        CanInterface::open_serial(port, gateway_id).await?
    } else if args.sim {
        info!("Using simulated CAN bus");
        let sim_bus = SimBus::new(vesc_ids);
        CanInterface::Sim(Arc::new(Mutex::new(sim_bus)))
    } else if let Some(ref eth) = eth_discovery {
        // Wait up to 3s for a CAN bridge to appear via Ethernet discovery
        let bridge_url = {
            let mut found = None;
            for _ in 0..30 {
                if let Some(url) = eth.discover_can_bridge() {
                    found = Some(url);
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            found
        };
        if let Some(url) = bridge_url {
            info!(url = %url, "Auto-discovered CAN bridge via Ethernet");
            CanInterface::connect_remote(&url).await?
        } else {
            info!(interface = %can_iface, "No CAN bridge found, opening SocketCAN");
            CanInterface::Real(Bus::open(can_iface)?)
        }
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
    if is_sim {
        info!("Sim mode: booted into Idle");

        if auto_start_autonomous {
            state_machine.transition(Event::AutonomousRequest);
            info!("Auto-starting Autonomous mode (goal provided)");
        }
    }

    // Get initial mode before moving state_machine
    let initial_mode = state_machine.mode();

    let mut tool_registry = ToolRegistry::new();
    tool_registry.register_blower(&file_config.blower);

    let shared = Arc::new(Mutex::new(SharedState {
        state_machine,
        commanded_twist: Twist::default(),
        drivetrain,
        tool_registry,
        health: SubsystemHealth::default(),
        fault_code: FaultCode::None,
    }));

    // Channels
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);

    let initial_telemetry = Telemetry {
        sequence: 0,
        timestamp_us: 0,
        mode: initial_mode,
        pose: Pose::default(),
        power: PowerStatus {
            battery_voltage: 48.0, // Simulated full battery
            system_current: 0.0,
        },
        cmd_velocity: Twist::default(),
        meas_velocity: (0.0, 0.0),
        acceleration: (0.0, 0.0),
        motor_temps: [25.0; 4], // Ambient temp
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
        lidar_core_temp_c: None,
        lidar_work_state: 0,
        mode_changed_at: 0,
        policy_intention: None,
        fault_code: 0,
        active_tool_type: 0,
    };
    let (telemetry_tx, telemetry_rx) = watch::channel(initial_telemetry);

    // Video frame channels
    // mpsc for WebRTC: H.264 frames queued from all cameras
    let (video_tx_rtc, video_rx_rtc) = tokio::sync::mpsc::channel::<H264Frame>(32);
    // watch for UDP: backward compatibility for native CLI (shows latest JPEG frame only)
    let (_video_tx_udp, video_rx_udp) =
        watch::channel::<Option<teleop::video::VideoFrame>>(None);
    // watch for LiDAR point cloud streaming (created early for RtcServer, populated later)
    let (lidar_tx, lidar_rx) = watch::channel::<Option<lidar::PointCloud>>(None);
    // watch for costmap streaming (created early for RtcServer, populated from control loop)
    let (costmap_tx, costmap_rx) = watch::channel::<Option<CostmapSnapshot>>(None);
    // watch for obstacle streaming (tracked obstacles with stable IDs and velocity)
    let (obstacles_tx, obstacles_rx) =
        watch::channel::<Option<Vec<costmap::TrackedObstacle>>>(None);
    // watch for navigation path streaming (planned A* path + nav state)
    let (path_tx, path_rx) =
        watch::channel::<Option<teleop::path_stream::PathSnapshot>>(None);
    // Object tracker for stable IDs and velocity estimation
    let obstacle_tracker = ObstacleTracker::new(TrackerConfig::default());

    // Build camera mount from config (used by RTC server for camera info messages)
    let mount_defaults = camera::CameraMount::default();
    let camera_mount = camera::CameraMount {
        position: file_config
            .camera
            .mount_position
            .unwrap_or(mount_defaults.position),
        rotation: file_config
            .camera
            .mount_rotation
            .unwrap_or(mount_defaults.rotation),
        fov_h: file_config.camera.fov_h.unwrap_or(mount_defaults.fov_h),
        fov_v: file_config.camera.fov_v.unwrap_or(mount_defaults.fov_v),
    };

    // Spawn teleop server (UDP)
    let seq_tracker = std::sync::Arc::new(std::sync::Mutex::new(teleop::SequenceTracker::new()));
    let teleop_config = TeleopConfig::default();
    let teleop = TeleopServer::new(teleop_config, cmd_tx.clone(), telemetry_rx.clone(), seq_tracker.clone());

    tokio::spawn(async move {
        if let Err(e) = teleop.run().await {
            error!(?e, "Teleop server error");
        }
    });

    // Shared flag for PLI-triggered keyframe requests (camera <- RTC server)
    let keyframe_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Spawn WebRTC teleop server (primary browser teleop with video)
    let mut rtc_metrics: Option<std::sync::Arc<RtcMetrics>> = None;
    let mut rtc_peer_count: Option<Arc<std::sync::atomic::AtomicU32>> = None;
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
        // Wire keyframe flag for PLI handling (camera reads this to force IDR frames)
        rtc_server.set_keyframe_requestor(keyframe_flag.clone());
        // Enable costmap and obstacle streaming if navigation is enabled
        if navigation_enabled {
            rtc_server.set_costmap_rx(costmap_rx.clone());
            rtc_server.set_obstacles_rx(obstacles_rx.clone());
            rtc_server.set_path_rx(path_rx.clone());
        }
        // Provide camera mount info for MSG_CAMERA_INFO messages
        // Width/height are known from config (H.264 pipeline uses fixed resolution)
        let cam_defaults = camera::H264Config::default();
        let cam_w = file_config.camera.width.unwrap_or(cam_defaults.width);
        let cam_h = file_config.camera.height.unwrap_or(cam_defaults.height);
        rtc_server.set_camera_info(teleop::CameraInfo {
            camera_id: 0,
            width: cam_w as u16,
            height: cam_h as u16,
            position: camera_mount.position,
            rotation: camera_mount.rotation,
            fov_h: camera_mount.fov_h,
            fov_v: camera_mount.fov_v,
        });

        // Get metrics and peer_count handles before spawning
        rtc_metrics = Some(rtc_server.metrics());
        rtc_peer_count = Some(rtc_server.peer_count());

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
        let ui_config = UiConfig {
            port: args.ui_port,
        };
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
        // Resolve resolution: CLI --camera_resolution > bvr.toml > H264Config::default()
        let h264_defaults = camera::H264Config::default();
        let (width, height) = if let Some(res) = &args.camera_resolution {
            let parts: Vec<&str> = res.split('x').collect();
            if parts.len() == 2 {
                (
                    parts[0].parse().unwrap_or(h264_defaults.width),
                    parts[1].parse().unwrap_or(h264_defaults.height),
                )
            } else {
                (h264_defaults.width, h264_defaults.height)
            }
        } else {
            (
                file_config.camera.width.unwrap_or(h264_defaults.width),
                file_config.camera.height.unwrap_or(h264_defaults.height),
            )
        };

        let camera_config = camera::H264Config {
            width,
            height,
            fps: file_config
                .camera
                .fps
                .unwrap_or(args.camera_fps),
            bitrate_kbps: file_config
                .camera
                .h264_bitrate_kbps
                .unwrap_or(h264_defaults.bitrate_kbps),
            keyframe_interval: file_config
                .camera
                .h264_keyframe_interval
                .unwrap_or(h264_defaults.keyframe_interval),
        };

        // Always spawn UDP video server so hot-plugged cameras have somewhere to send frames
        let video_config = VideoConfig::default();
        let video_server = VideoServer::new(video_config.clone(), video_rx_udp.clone());
        info!(port = video_config.port, "UDP video server starting");
        tokio::spawn(async move {
            if let Err(e) = video_server.run().await {
                error!(?e, "UDP video server error");
            }
        });

        // Shared set of active camera stable IDs (dead cameras remove themselves)
        let active_cameras: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        // Monotonically increasing camera_id counter (informational only)
        let next_camera_id: Arc<AtomicU8> = Arc::new(AtomicU8::new(0));
        let keyframe_requestor = keyframe_flag.clone();

        let max_cameras = file_config.camera.max_cameras;

        // Initial camera scan
        info!("Detecting cameras...");
        let cameras = camera::detect_cameras();
        let cameras_to_start: Vec<_> = if max_cameras > 0 {
            cameras.into_iter().take(max_cameras).collect()
        } else {
            cameras
        };

        if cameras_to_start.is_empty() {
            info!("No cameras detected (hot-plug monitor will retry)");
        } else {
            info!(
                count = cameras_to_start.len(),
                max = max_cameras,
                "Cameras detected"
            );
            for cam in &cameras_to_start {
                info!(name = %cam.name, "  - {:?}", cam.camera_type);
            }

            let mut cameras_started = 0u8;
            for cam in &cameras_to_start {
                if crate::camera_init::start_camera(
                    cam,
                    camera_config.clone(),
                    &next_camera_id,
                    &active_cameras,
                    &video_tx_rtc,
                    &shared,
                    &keyframe_requestor,
                ) {
                    cameras_started += 1;
                }
            }

            if cameras_started > 0 {
                info!(count = cameras_started, "Camera capture started (GStreamer)");
            } else {
                warn!("No cameras could be started - hot-plug monitor will retry");
            }
        }

        // Spawn hot-plug monitor task
        let hot_plug_secs = file_config.camera.hot_plug_interval_secs;
        if hot_plug_secs > 0 {
            let monitor_active = active_cameras.clone();
            let monitor_next_id = next_camera_id.clone();
            let monitor_config = camera_config.clone();
            let monitor_rtc_tx = video_tx_rtc.clone();
            let monitor_shared = shared.clone();
            let monitor_kf_req = keyframe_requestor.clone();
            let monitor_interval = Duration::from_secs(hot_plug_secs);

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(monitor_interval).await;

                    let detected = camera::detect_cameras();
                    let active = monitor_active.lock().expect("active_cameras mutex poisoned").clone();
                    let active_count = active.len();

                    for cam in &detected {
                        // Respect max_cameras as concurrent limit
                        if max_cameras > 0 && active_count >= max_cameras {
                            break;
                        }

                        let id = cam.stable_id();
                        if active.contains(&id) {
                            continue;
                        }

                        info!(
                            stable_id = %id,
                            name = %cam.name,
                            "Hot-plugged camera detected, starting..."
                        );
                        crate::camera_init::start_camera(
                            cam,
                            monitor_config.clone(),
                            &monitor_next_id,
                            &monitor_active,
                            &monitor_rtc_tx,
                            &monitor_shared,
                            &monitor_kf_req,
                        );
                    }

                    // Update health flag based on active camera count
                    let has_cameras = !monitor_active.lock().expect("active_cameras mutex poisoned").is_empty();
                    monitor_shared.lock().expect("shared state mutex poisoned").health.camera_active = has_cameras;
                }
            });

            info!(
                interval_secs = hot_plug_secs,
                "Camera hot-plug monitor started"
            );
        }
    }

    // Initialize localization
    let odometry = WheelOdometry::new(chassis.clone(), args.pole_pairs);
    let ekf_config = EkfConfig {
        odom_linear_noise: file_config.localization.odom_linear_noise,
        odom_angular_noise: file_config.localization.odom_angular_noise,
        gps_noise_rtk_fixed: file_config.localization.gps_noise_rtk_fixed,
        gps_noise_rtk_float: file_config.localization.gps_noise_rtk_float,
        gps_noise_dgps: file_config.localization.gps_noise_dgps,
        gps_noise_standalone: file_config.localization.gps_noise_standalone,
        gps_heading_min_speed: file_config.localization.gps_heading_min_speed,
        imu_yaw_weight: file_config.localization.imu_yaw_weight,
        imu_yaw_speed_ramp: file_config.localization.imu_yaw_speed_ramp,
        imu_yaw_gyro_gate: file_config.localization.imu_yaw_gyro_gate,
        ..Default::default()
    };
    let pose_estimator = EkfPoseEstimator::with_config(ekf_config);

    // GPS state channel (updated by GPS reader thread)
    let (gps_tx, gps_rx) = watch::channel(GpsState::default());

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
    let lidar_rx_local = lidar_rx.clone();

    // LiDAR status channel (populated by driver's 10s polling)
    let (lidar_status_tx, lidar_status_rx) = watch::channel(LidarStatus::default());

    // Command channel for runtime lidar control (power save mode)
    let lidar_cmd_tx: Option<tokio::sync::mpsc::Sender<LidarCommand>> =
        if file_config.lidar.enabled {
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
                mounting_height_m: file_config.lidar.mounting_height_m,
                detect_mode: file_config.lidar.detect_mode,
                glass_heat: file_config.lidar.glass_heat,
                blind_spot_cm: file_config.lidar.blind_spot_cm,
                use_hardware_transform: file_config.lidar.use_hardware_transform,
                ..Default::default()
            };

            let lidar_reader = LidarReader::new(lidar_config);

            if file_config.lidar.power_save {
                // Power save mode: lidar starts immediately, only stops during Sleep
                let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<LidarCommand>(4);
                let _lidar_handle = lidar_reader.spawn_with_control(
                    lidar_tx,
                    imu_tx,
                    cmd_rx,
                    true,
                    lidar_status_tx,
                );
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
        Option<mpsc::Sender<(lidar::PointCloud, Pose, Option<slam::PreintegratedDelta>)>>,
        Option<watch::Receiver<SlamState>>,
    ) = if file_config.slam.enabled && file_config.lidar.enabled {
        let slam_config = SlamConfig {
            voxel_size: file_config.slam.voxel_size,
            max_iterations: file_config.slam.max_iterations,
            convergence_threshold: file_config.slam.convergence_threshold,
            max_correspondence_dist: file_config.slam.max_correspondence_dist,
            min_overlap: file_config.slam.min_overlap,
            keyframe_distance: file_config.slam.keyframe_distance,
            keyframe_rotation: file_config.slam.keyframe_rotation,
            loop_closure_threshold: file_config.slam.loop_closure_threshold,
            huber_threshold: file_config.slam.huber_threshold,
            loop_closure_max_disagreement_sigma: file_config
                .slam
                .loop_closure_max_disagreement_sigma,
            ..Default::default()
        };

        // Channel for sending scans to SLAM (bounded to prevent backpressure)
        let (scan_tx, mut scan_rx) =
            mpsc::channel::<(lidar::PointCloud, Pose, Option<slam::PreintegratedDelta>)>(4);
        // Watch channel for receiving SLAM state updates
        let (state_tx, state_rx) = watch::channel(SlamState::default());

        // Spawn SLAM on a dedicated thread (NOT tokio::spawn) to avoid blocking the async runtime.
        // SLAM's process_scan() is CPU-intensive (10-100ms+) and would starve tokio worker threads.
        // Using std::sync::mpsc for thread-safe channel from async to sync.
        let (slam_thread_tx, slam_thread_rx) = std::sync::mpsc::channel::<(
            lidar::PointCloud,
            Pose,
            Option<slam::PreintegratedDelta>,
        )>();

        // Bridge async mpsc to sync mpsc
        tokio::spawn(async move {
            while let Some(item) = scan_rx.recv().await {
                if slam_thread_tx.send(item).is_err() {
                    break; // SLAM thread stopped
                }
            }
        });

        // Map persistence config
        let map_path = std::path::PathBuf::from("/var/lib/bvr/slam_map.bin");
        let auto_save_keyframes: usize = 50;
        let load_prior_map = true;
        let rover_id_slam = file_config.identity.rover_id.clone();

        // Dedicated SLAM thread - completely isolated from tokio runtime
        std::thread::Builder::new()
            .name("slam".to_string())
            .spawn(move || {
                let mut slam = SlamProcessor::new(slam_config.clone());
                info!("SLAM thread started (dedicated, non-blocking)");

                // Load prior map if configured and available
                if load_prior_map && map_path.exists() {
                    match SlamProcessor::load_map(&map_path, &slam_config) {
                        Ok((keyframes, edges, sc_db)) => {
                            slam.restore_map(keyframes, edges, sc_db);
                            info!("Loaded prior SLAM map from {}", map_path.display());
                        }
                        Err(e) => {
                            warn!(?e, "Failed to load prior SLAM map, starting fresh");
                        }
                    }
                }

                let mut last_save_count: usize = slam.keyframe_count();
                let mut first_scan = slam.keyframe_count() == 0;
                let mut relocalized = !load_prior_map || slam.keyframe_count() == 0;

                while let Ok((scan, odom_pose, imu_delta)) = slam_thread_rx.recv() {
                    // Update odometry before processing scan
                    slam.update_odometry(&odom_pose);

                    // Attempt relocalization on first scan after loading a map
                    if !relocalized && !first_scan {
                        if let Some((_kf_id, pose)) = slam.relocalize(&scan, 0.5) {
                            info!(
                                x = pose.translation().x,
                                y = pose.translation().y,
                                "Relocalized against loaded map"
                            );
                        } else {
                            info!("Relocalization failed, continuing with loaded map poses");
                        }
                        relocalized = true;
                    }
                    first_scan = false;

                    // Set pre-integrated IMU delta for hybrid initial guess
                    slam.set_imu_delta(imu_delta);

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

                        // Auto-save every N keyframes
                        if auto_save_keyframes > 0
                            && slam.keyframes_since(last_save_count) >= auto_save_keyframes
                        {
                            if let Err(e) = slam.save_map(&map_path, &rover_id_slam) {
                                warn!(?e, "Failed to auto-save SLAM map");
                            } else {
                                last_save_count = slam.keyframe_count();
                            }
                        }

                        // Only broadcast SLAM state when scan matching produced
                        // a real correction. When process_scan returns None (scan
                        // match failed), the EKF pose may be drifted beyond the
                        // convergence basin — broadcasting it would reinforce
                        // drift in the main control loop's EKF.
                        let state = SlamState {
                            pose: slam.pose(),
                            keyframe_count: slam.keyframe_count() as u32,
                            loop_closure_count: slam.loop_closure_count() as u32,
                            keyframe_poses: slam.keyframe_poses(),
                            pose_covariance: slam.pose_covariance_array(),
                            match_score: update.match_score,
                        };
                        let _ = state_tx.send(state);
                    }
                }

                // Save map on clean shutdown
                if slam.keyframe_count() > 0 && slam.keyframes_since(last_save_count) > 0 {
                    if let Err(e) = slam.save_map(&map_path, &rover_id_slam) {
                        warn!(?e, "Failed to save SLAM map on shutdown");
                    }
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
    let slam_state_rx_local: Option<watch::Receiver<SlamState>> = slam_state_rx.clone();

    // Hardware e-stop GPIO monitoring (Jetson only, requires gpio feature)
    let (estop_tx, estop_rx) = mpsc::channel::<EStopEvent>(8);
    let hardware_estop_latched = false; // Latches until explicit release confirmation

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
                    warn!(
                        ?e,
                        "Failed to start hardware e-stop GPIO - continuing without hardware e-stop"
                    );
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

    // Pre-compute e-stop confirmation flag from file_config before it is consumed
    let estop_require_confirmation = file_config.estop.require_confirmation;

    // Pre-compute sleep timeout from file_config before it is consumed
    let sleep_timeout_secs = file_config.control.sleep_timeout_secs;

    // Pre-compute mounting pitch in radians (used for IMU transform in control loop)
    let mount_pitch_rad = file_config.lidar.mounting_pitch_deg.to_radians();

    // Control loop setup
    let mixer = DiffDriveMixer::new(chassis);
    let rate_limiter = RateLimiter::new(limits);
    let collision_monitor = CollisionMonitor::new(CollisionMonitorConfig {
        enabled: file_config.collision_guard.enabled,
        lookahead_time: file_config.collision_guard.lookahead_time,
        num_samples: file_config.collision_guard.num_samples,
        full_speed_distance: file_config.collision_guard.full_speed_distance,
        stop_distance: file_config.collision_guard.stop_distance,
        cost_threshold: file_config.collision_guard.cost_threshold,
        min_zone_scale: file_config.collision_guard.min_zone_scale,
        max_linear_velocity: file_config.collision_guard.max_linear_velocity,
        num_angular_samples: file_config.collision_guard.num_angular_samples,
        angular_spread: file_config.collision_guard.angular_spread,
        angular_cost_weight: file_config.collision_guard.angular_cost_weight,
    });
    if file_config.collision_guard.enabled {
        info!("Collision monitor enabled (velocity scaling for all modes)");
    }
    let mut watchdog = Watchdog::new(Duration::from_millis(file_config.control.watchdog_timeout_ms));

    // Feed watchdog immediately if starting in autonomous mode
    // (otherwise is_timed_out() returns true on first iteration before we can feed it)
    if initial_mode == Mode::Autonomous {
        watchdog.feed();
    }

    let control_period = Duration::from_millis(10); // 100 Hz
    let last_tick = Instant::now();

    // Heartbeat tracking for debugging watchdog issues
    let heartbeat_count: u32 = 0;
    let last_heartbeat_log = Instant::now();

    // Current tool command (from teleop)
    let tool_command = types::ToolCommand::default();

    // Auto-sleep after idle/disabled timeout
    let idle_since = Instant::now();

    // Track mode for change detection (for recording annotations)
    let last_mode = initial_mode;

    // Send initial LED command for current mode
    {
        let state = shared.lock().expect("shared state mutex poisoned");
        let led_cmd = state.state_machine.led_command();
        let led_frame = led_cmd.to_frame();
        if let Err(e) = can_interface.send(&led_frame) {
            debug!(?e, "Failed to send initial LED command");
        } else {
            info!(?initial_mode, "Initial LED state set");
        }
    }

    // Diagnostic counter for battery logging
    let loop_count: u64 = 0;

    // System metrics collector (CPU, memory, disk)
    let sys_metrics = SystemMetricsCollector::new();
    let last_sys_refresh = Instant::now();

    // CAN bus health tracking - avoid overwhelming driver when bus is unhealthy
    let can_errors_active = false;
    let can_backoff_active = false;
    let can_error_count: u64 = 0;
    let consecutive_can_errors: u32 = 0;

    // Policy execution timeout tracking for autonomous mode safety
    let last_policy_action: Option<policy::PolicyAction> = None;

    // Bounded task spawning for dispatch reports
    // Prevents unbounded task accumulation during long missions
    const MAX_DISPATCH_TASKS: usize = 100;
    let dispatch_semaphore = Arc::new(Semaphore::new(MAX_DISPATCH_TASKS));

    // Get tokio runtime handle for spawning async tasks from control loop thread
    let tokio_handle = tokio::runtime::Handle::current();

    // IMU attitude filter (fuses gyro + accelerometer for smooth roll/pitch)
    let attitude_filter = AttitudeFilter::new();

    // IMU pre-integrator for SLAM (accumulates gyro-Z between LiDAR scans)
    let imu_preintegrator =
        slam::ImuPreintegrator::new(slam::ImuPreintegrationConfig::default());

    // Heartbeat counter for deadlock detection (thread spawned in run_loop)
    let heartbeat_counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let heartbeat_stalled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Print info logs that come right before the control loop spawn
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

    let ctx = DaemonContext {
        can_interface,
        is_sim,
        shared,
        cmd_tx: cmd_tx.clone(),
        cmd_rx,
        telemetry_tx,
        metrics_tx,
        mixer,
        rate_limiter,
        collision_monitor,
        watchdog,
        odometry,
        pose_estimator,
        gps_rx,
        lidar_rx: lidar_rx_local,
        imu_rx,
        lidar_cmd_tx,
        lidar_enabled,
        lidar_status_rx,
        slam_scan_tx,
        slam_state_rx: slam_state_rx_local,
        navigation_controller,
        obstacle_tracker,
        costmap_tx,
        obstacles_tx,
        path_tx,
        loaded_policy,
        autonomous_goal,
        autonomous_max_linear,
        autonomous_max_angular,
        norm_config,
        dispatch_client,
        dispatch_rx,
        current_dispatch_task,
        estop_rx,
        estop_require_confirmation,
        recorder,
        recording_enabled,
        rtc_metrics,
        peer_count: rtc_peer_count.unwrap_or_else(|| Arc::new(std::sync::atomic::AtomicU32::new(0))),
        sleep_timeout_secs,
        mount_pitch_rad,
        discovery_enabled,
        dispatch_enabled,
        tokio_handle,
        loop_count,
        heartbeat_counter,
        heartbeat_count,
        last_heartbeat_log,
        tool_command,
        idle_since,
        last_mode,
        sys_metrics,
        last_sys_refresh,
        can_errors_active,
        can_backoff_active,
        can_error_count,
        consecutive_can_errors,
        last_policy_action,
        last_policy_action_time: None,
        dispatch_semaphore,
        attitude_filter,
        imu_preintegrator,
        last_tick,
        control_period,
        hardware_estop_latched,
        dt_min_ms: f64::MAX,
        dt_max_ms: 0.0,
        last_gps_update: None,
        last_lidar_update: None,
        last_slam_update: None,
        last_subsystem_check: Instant::now(),
        dispatch_connected_live: false,
        heartbeat_stalled,
        seq_tracker,
        eth_discovery,
    };

    Ok((ctx, log_guard))
}
