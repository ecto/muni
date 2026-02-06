//! Configuration file structures for bvr.toml.

use anyhow::Result;
use serde::Deserialize;
use tools::blower::BlowerConfig;
use tools::eth_protocol;
use tracing::warn;

/// Configuration file structure (bvr.toml).
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub(crate) struct FileConfig {
    pub identity: IdentityConfig,
    pub chassis: ChassisFileConfig,
    pub control: ControlFileConfig,
    pub metrics: MetricsFileConfig,
    pub discovery: DiscoveryFileConfig,
    pub autonomous: AutonomousFileConfig,
    pub dispatch: DispatchFileConfig,
    pub lidar: LidarFileConfig,
    pub localization: LocalizationFileConfig,
    pub slam: SlamFileConfig,
    pub navigation: NavigationFileConfig,
    pub estop: EStopFileConfig,
    pub camera: CameraFileConfig,
    pub blower: BlowerConfig,
    pub eth_tools: EthToolsFileConfig,
    pub collision_guard: CollisionGuardFileConfig,
}

/// Control loop configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct ControlFileConfig {
    /// Seconds of inactivity in Idle/Disabled before auto-transitioning to Sleep (0 = disabled)
    pub sleep_timeout_secs: u64,
    /// Watchdog timeout in milliseconds — if no heartbeat/command arrives within this window,
    /// teleop drops to Idle. 1500ms handles browser tab throttling (1Hz setInterval) with margin.
    pub watchdog_timeout_ms: u64,
}

impl Default for ControlFileConfig {
    fn default() -> Self {
        Self {
            sleep_timeout_secs: 0, // disabled by default
            watchdog_timeout_ms: 1500,
        }
    }
}

/// Collision monitor configuration (velocity scaling near obstacles, all modes).
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct CollisionGuardFileConfig {
    pub enabled: bool,
    pub lookahead_time: f64,
    pub num_samples: usize,
    pub full_speed_distance: f64,
    pub stop_distance: f64,
    pub cost_threshold: u8,
    pub min_zone_scale: f64,
    pub max_linear_velocity: f64,
    pub num_angular_samples: usize,
    pub angular_spread: f64,
    pub angular_cost_weight: f64,
}

impl Default for CollisionGuardFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookahead_time: 1.0,
            num_samples: 10,
            full_speed_distance: 1.0,
            stop_distance: 0.15,
            cost_threshold: 253,
            min_zone_scale: 0.5,
            max_linear_velocity: 1.0,
            num_angular_samples: 5,
            angular_spread: 0.5,
            angular_cost_weight: 0.3,
        }
    }
}

/// Camera configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct CameraFileConfig {
    /// Enable camera capture
    pub enabled: bool,
    /// Maximum number of cameras to initialize (0 = all detected)
    /// On Jetson, initializing multiple CSI cameras can hang GStreamer.
    /// Set to 1 if experiencing camera init hangs.
    pub max_cameras: usize,
    /// Hot-plug polling interval in seconds (0 = disabled).
    /// When enabled, bvrd periodically re-scans for new cameras and starts them.
    pub hot_plug_interval_secs: u64,
    /// Capture width in pixels
    pub width: Option<u32>,
    /// Capture height in pixels
    pub height: Option<u32>,
    /// Capture FPS
    pub fps: Option<u32>,
    /// JPEG quality (1-100)
    pub jpeg_quality: Option<u8>,
    /// H.264 encoding bitrate in kbit/sec (default: 4000 = 4 Mbps)
    pub h264_bitrate_kbps: Option<u32>,
    /// H.264 keyframe interval in frames (default: 30 = 1 keyframe/sec at 30fps)
    pub h264_keyframe_interval: Option<u32>,
    /// Camera mount position [forward, left, up] in meters
    pub mount_position: Option<[f32; 3]>,
    /// Camera mount rotation [roll, pitch, yaw] in radians
    pub mount_rotation: Option<[f32; 3]>,
    /// Horizontal field of view in radians
    pub fov_h: Option<f32>,
    /// Vertical field of view in radians
    pub fov_v: Option<f32>,
}

impl Default for CameraFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cameras: 1, // Default to 1 to avoid Jetson CSI init issues
            hot_plug_interval_secs: 3,
            width: None,
            height: None,
            fps: None,
            jpeg_quality: None,
            h264_bitrate_kbps: None,
            h264_keyframe_interval: None,
            mount_position: None,
            mount_rotation: None,
            fov_h: None,
            fov_v: None,
        }
    }
}

/// Hardware e-stop configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct EStopFileConfig {
    /// Enable hardware e-stop GPIO monitoring
    pub enabled: bool,
    /// GPIO chip device name (e.g., "gpiochip0")
    pub gpio_chip: String,
    /// GPIO line number (e.g., 16 for Pin 36 on Jetson 40-pin header)
    pub gpio_line: u32,
    /// Active-low logic (true = button connects GPIO to GND when pressed)
    pub active_low: bool,
    /// Debounce time in milliseconds
    pub debounce_ms: u64,
    /// Require operator confirmation to release e-stop (not just button release)
    pub require_confirmation: bool,
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
pub(crate) struct DispatchFileConfig {
    /// Enable dispatch service connection
    pub enabled: bool,
    /// Dispatch service endpoint (ws://host:port/ws)
    pub endpoint: String,
}

impl Default for DispatchFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "ws://depot:4890/ws".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct LidarFileConfig {
    /// Enable LiDAR sensor
    pub enabled: bool,
    /// LiDAR IP address (Livox Mid360)
    pub lidar_ip: String,
    /// Host IP address for receiving data (must match rover's eth interface)
    pub host_ip: String,
    /// Point cloud UDP port (default 56301)
    pub point_cloud_port: u16,
    /// IMU data UDP port (default 56401, 0 to disable)
    pub imu_port: u16,
    /// Voxel size for point cloud downsampling (meters). Larger = fewer points.
    pub stream_voxel_size: f32,
    /// Maximum points per frame for streaming. Higher = more detail but more bandwidth.
    pub stream_max_points: usize,
    /// Mounting pitch angle in degrees (positive = sensor tilted forward/down).
    /// Applied to both point cloud transform and IMU readings.
    pub mounting_pitch_deg: f32,
    /// Mounting height above ground in meters.
    /// Translates point cloud Z so ground ≈ 0 after pitch rotation.
    pub mounting_height_m: f32,
    /// Power save mode: only run lidar during Teleop/Autonomous, stop in Idle.
    /// Reduces heat and power consumption when rover is not active.
    pub power_save: bool,
    /// Detection sensitivity: 0=normal, 1=sensitive (better for dark surfaces).
    pub detect_mode: u8,
    /// Enable lens heating (prevents fog/ice in winter).
    pub glass_heat: bool,
    /// Minimum detection range in cm (50-200, 0 = device default).
    pub blind_spot_cm: u16,
    /// Offload mounting pitch to lidar firmware (instead of software transform).
    pub use_hardware_transform: bool,
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
            mounting_height_m: 0.0,
            power_save: true,
            detect_mode: 0,
            glass_heat: false,
            blind_spot_cm: 0,
            use_hardware_transform: false,
        }
    }
}

/// Localization (EKF) configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct LocalizationFileConfig {
    pub odom_linear_noise: f64,
    pub odom_angular_noise: f64,
    pub gps_noise_rtk_fixed: f64,
    pub gps_noise_rtk_float: f64,
    pub gps_noise_dgps: f64,
    pub gps_noise_standalone: f64,
    pub gps_heading_min_speed: f64,
    pub imu_yaw_weight: f64,
    pub imu_yaw_speed_ramp: f64,
    pub imu_yaw_gyro_gate: f64,
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
            imu_yaw_speed_ramp: 0.1,
            imu_yaw_gyro_gate: 0.5,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct SlamFileConfig {
    /// Enable SLAM processing
    pub enabled: bool,
    /// Voxel downsampling resolution for GICP (meters)
    pub voxel_size: f64,
    /// Maximum GICP iterations
    pub max_iterations: usize,
    /// GICP convergence threshold (pose delta norm)
    pub convergence_threshold: f64,
    /// Maximum correspondence distance for outlier rejection (meters)
    pub max_correspondence_dist: f64,
    /// Minimum fraction of points with valid correspondences
    pub min_overlap: f64,
    /// Distance threshold for inserting new keyframe (meters)
    pub keyframe_distance: f64,
    /// Rotation threshold for inserting new keyframe (radians)
    pub keyframe_rotation: f64,
    /// Score threshold for accepting loop closure match
    pub loop_closure_threshold: f64,
    /// Huber loss threshold for robust loop closure weighting (meters)
    pub huber_threshold: f64,
    /// Maximum disagreement between loop closure and odometry (in σ)
    pub loop_closure_max_disagreement_sigma: f64,
}

impl Default for SlamFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            voxel_size: 0.2,
            max_iterations: 30,
            convergence_threshold: 1e-4,
            max_correspondence_dist: 1.0,
            min_overlap: 0.3,
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
pub(crate) struct NavigationFileConfig {
    /// Enable classical navigation (A* + pure pursuit) instead of policy
    pub enabled: bool,
    /// Costmap width in meters
    pub costmap_width: f64,
    /// Costmap height in meters
    pub costmap_height: f64,
    /// Costmap resolution (meters per cell)
    pub costmap_resolution: f64,
    /// Inflation radius around obstacles (meters)
    pub inflation_radius: f64,
    /// Robot inscribed radius (meters)
    pub inscribed_radius: f64,
    /// Lookahead distance for pure pursuit (meters)
    pub lookahead_distance: f64,
    /// Goal tolerance (meters)
    pub goal_tolerance: f64,
    /// Maximum linear velocity (m/s)
    pub max_linear_vel: f64,
    /// Maximum angular velocity (rad/s)
    pub max_angular_vel: f64,
    /// Blocked timeout before starting recovery (seconds)
    pub blocked_timeout: f64,
    /// Recovery: distance to back up (meters)
    pub recovery_backup_distance: f64,
    /// Recovery: backup speed (m/s)
    pub recovery_backup_speed: f64,
    /// Recovery: spin angular velocity (rad/s)
    pub recovery_spin_speed: f64,
    /// Recovery: wait duration between cycles (seconds)
    pub recovery_wait_duration: f64,
    /// Recovery: max cycles before failure
    pub max_recovery_cycles: u32,
    /// Controller type: "pure_pursuit" (default) or "mppi"
    pub controller_type: String,
    /// MPPI: number of candidate trajectories
    pub mppi_num_samples: usize,
    /// MPPI: prediction horizon steps
    pub mppi_horizon_steps: usize,
    /// MPPI: time step per horizon step (seconds)
    pub mppi_dt: f64,
    /// MPPI: temperature for soft-max weighting
    pub mppi_temperature: f64,
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
            recovery_backup_distance: 0.3,
            recovery_backup_speed: 0.2,
            recovery_spin_speed: 0.5,
            recovery_wait_duration: 10.0,
            max_recovery_cycles: 3,
            controller_type: "pure_pursuit".to_string(),
            mppi_num_samples: 512,
            mppi_horizon_steps: 30,
            mppi_dt: 0.1,
            mppi_temperature: 0.5,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct ChassisFileConfig {
    /// Wheel diameter in meters
    pub wheel_diameter_m: f64,
    /// Distance between left and right wheels (track width)
    pub track_width_m: f64,
    /// Distance between front and rear axles
    pub wheelbase_m: f64,
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
pub(crate) struct AutonomousFileConfig {
    /// Enable autonomous mode support
    pub enabled: bool,
    /// Directory containing policy files
    pub policy_dir: String,
    /// Specific policy file to load (overrides default)
    pub policy_file: Option<String>,
    /// Maximum linear velocity in autonomous mode (m/s)
    pub max_linear_vel: f64,
    /// Maximum angular velocity in autonomous mode (rad/s)
    pub max_angular_vel: f64,
    /// Goal waypoint [x, y] in local frame (for testing)
    pub goal: Option<[f64; 2]>,
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
pub(crate) struct IdentityConfig {
    pub rover_id: String,
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
pub(crate) struct MetricsFileConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval_hz: u32,
}

impl Default for MetricsFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "depot:8089".to_string(),
            interval_hz: 1,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct DiscoveryFileConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub rover_id: Option<String>,
    pub rover_name: Option<String>,
    /// WebRTC signaling port to advertise (default: 4852)
    pub rtc_port: u16,
    pub heartbeat_secs: u32,
    /// Hostname to advertise instead of auto-detected IP (e.g., "frog-0" for Tailscale)
    pub advertise_host: Option<String>,
}

impl Default for DiscoveryFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "depot:4860".to_string(),
            rover_id: None,
            rover_name: None,
            rtc_port: 4852,
            heartbeat_secs: 2,
            advertise_host: None,
        }
    }
}

/// Ethernet tool MCU discovery configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct EthToolsFileConfig {
    /// Enable Ethernet tool discovery
    pub enabled: bool,
    /// UDP port to listen for MCU broadcasts
    pub discovery_port: u16,
}

impl Default for EthToolsFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_port: eth_protocol::DISCOVERY_PORT,
        }
    }
}

impl FileConfig {
    pub(crate) fn load(path: &std::path::Path) -> Result<Self> {
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
