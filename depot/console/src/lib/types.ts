// Mode enum matching bvrd
export const Mode = {
  Disabled: 0,
  Idle: 1,
  Teleop: 2,
  Autonomous: 3,
  EStop: 4,
  Fault: 5,
  Sleep: 6,
} as const;

export type Mode = (typeof Mode)[keyof typeof Mode];

export const ModeLabels: Record<Mode, string> = {
  [Mode.Disabled]: "Idle",
  [Mode.Idle]: "Idle",
  [Mode.Teleop]: "Teleop",
  [Mode.Autonomous]: "Autonomous",
  [Mode.EStop]: "E-Stop",
  [Mode.Fault]: "Fault",
  [Mode.Sleep]: "Sleep",
};

// Tool type enum matching bvrd tools::ToolType
export const ToolType = {
  Unknown: 0,
  SnowAuger: 1,
  Spreader: 2,
  Mower: 3,
  Plow: 4,
  Blower: 5,
  Lights: 6,
  Sensor: 7,
} as const;

export type ToolType = (typeof ToolType)[keyof typeof ToolType];

export const ToolTypeLabels: Record<number, string> = {
  [ToolType.Unknown]: "None",
  [ToolType.SnowAuger]: "Snow Auger",
  [ToolType.Spreader]: "Spreader",
  [ToolType.Mower]: "Mower",
  [ToolType.Plow]: "Plow",
  [ToolType.Blower]: "Blower",
  [ToolType.Lights]: "Headlights",
  [ToolType.Sensor]: "Sensor",
};

export interface Twist {
  linear: number;
  angular: number;
}

export interface Power {
  battery_voltage: number;
  system_current: number;
}

export interface Pose {
  x: number;
  y: number;
  theta: number;
  /** Roll angle from IMU (radians, 0 = level) */
  roll: number;
  /** Pitch angle from IMU (radians, 0 = level) */
  pitch: number;
}

export interface SlamStatus {
  pose: Pose;
  confidence: number;
  keyframeCount: number;
  loopClosureCount: number;
  mappingActive: boolean;
}

// Wheel status values: 0=offline, 1=degraded, 2=online
export const WheelStatus = {
  Offline: 0,
  Degraded: 1,
  Online: 2,
} as const;

export type WheelStatus = (typeof WheelStatus)[keyof typeof WheelStatus];

// Subsystem health flags from telemetry
export interface SubsystemHealth {
  can_healthy: boolean;
  recording_active: boolean;
  gps_fix: boolean;
  camera_active: boolean;
  dispatch_connected: boolean;
  discovery_connected: boolean;
  lidar_active: boolean;
  slam_running: boolean;
  /** Wheel status: [FL, FR, RL, RR], values: 0=offline, 1=degraded, 2=online */
  wheel_status: [WheelStatus, WheelStatus, WheelStatus, WheelStatus];
}

export const defaultHealth: SubsystemHealth = {
  can_healthy: false,
  recording_active: false,
  gps_fix: false,
  camera_active: false,
  dispatch_connected: false,
  discovery_connected: false,
  lidar_active: false,
  slam_running: false,
  wheel_status: [0, 0, 0, 0],
};

export interface LidarTelemetry {
  /** Core temperature in °C */
  temp_c: number;
  /** Work state: 0=unknown, 1=sampling, 2=idle, 3=ready, 4=error */
  work_state: number;
}

export interface Telemetry {
  mode: Mode;
  pose: Pose;
  power: Power;
  velocity: Twist;
  motor_temps: [number, number, number, number];
  connected: boolean;
  latency_ms: number;
  slamStatus?: SlamStatus;
  health: SubsystemHealth;
  /** System CPU usage (0-100%) */
  cpu_percent: number;
  /** System memory usage (0-100%) */
  mem_percent: number;
  /** System disk usage (0-100%) */
  disk_percent: number;
  /** Epoch ms when mode last changed (client-side tracking) */
  modeChangedAt: number;
  /** LiDAR sensor status (undefined when no data available) */
  lidar?: LidarTelemetry;
  /** Policy intention point in world coordinates (meters), null when inactive */
  policyIntention: { x: number; y: number } | null;
  /** Fault code (0 = no fault, 1 = watchdog timeout). Set when mode is Fault. */
  faultCode: number;
  /** Active tool type (0 = none, see ToolType enum) */
  activeToolType: ToolType;
}

export interface GamepadInput {
  linear: number;
  angular: number;
  toolAxis: number;
  actionA: boolean;
  actionB: boolean;
  estop: boolean;
  enable: boolean;
  disable: boolean;
  boost: boolean;
  cameraYaw: number;
  cameraPitch: number;
}

export const InputSource = {
  None: "none",
  Keyboard: "keyboard",
  Gamepad: "gamepad",
} as const;

export type InputSource = (typeof InputSource)[keyof typeof InputSource];

export const CameraMode = {
  ThirdPerson: "third-person",
  FirstPerson: "first-person",
  FreeLook: "free-look",
} as const;

export type CameraMode = (typeof CameraMode)[keyof typeof CameraMode];

// Session types (from recording crate's SessionMetadata)
export interface GpsSessionBounds {
  min_lat: number;
  max_lat: number;
  min_lon: number;
  max_lon: number;
}

export interface Session {
  session_id: string;
  rover_id: string;
  started_at: string;
  ended_at: string | null;
  duration_secs: number;
  gps_bounds: GpsSessionBounds | null;
  lidar_frames: number;
  camera_frames: number;
  pose_samples: number;
  session_file: string;
  session_dir: string;
}

// Map types (from map-api service)
export interface GpsBounds {
  minLat: number;
  maxLat: number;
  minLon: number;
  maxLon: number;
}

export interface MapSummary {
  id: string;
  name: string;
  bounds: GpsBounds;
  version: number;
  updatedAt: string;
  sessionCount: number;
  hasSplat: boolean;
  thumbnailUrl: string | null;
}

export interface MapAssets {
  splat: string | null;
  pointcloud: string | null;
  mesh: string | null;
  thumbnail: string | null;
}

export interface MapSessionRef {
  sessionId: string;
  roverId: string;
  date: string;
}

export interface MapStats {
  totalPoints: number;
  totalSplats: number;
  coveragePct: number;
}

export interface MapManifest {
  id: string;
  name: string;
  description: string | null;
  bounds: GpsBounds;
  version: number;
  createdAt: string;
  updatedAt: string;
  assets: MapAssets;
  sessions: MapSessionRef[];
  stats: MapStats;
}

// Rover info for fleet management
export interface RoverInfo {
  id: string;
  name: string;
  /** WebRTC signaling address (e.g., "ws://192.168.1.100:4852") */
  rtcAddress: string;
  online: boolean;
  batteryVoltage: number;
  lastPose: Pose;
  mode: Mode;
  lastSeen: number;
}

// Service health for infrastructure monitoring
export interface ServiceHealth {
  name: string;
  status: "healthy" | "unhealthy" | "unknown";
  url?: string;
  lastCheck: number;
  details?: string;
}

// =============================================================================
// Dispatch Types (zones, missions, tasks)
// =============================================================================

export interface Waypoint {
  x: number;
  y: number;
  theta?: number;
}

export interface GpsCoord {
  lat: number;
  lon: number;
}

export interface CoverageConfig {
  toolWidth: number;
  overlapPct: number;
  swathAngle?: number | null;
}

export interface Zone {
  id: string;
  name: string;
  zoneType: string;
  waypoints: Waypoint[];
  polygon?: GpsCoord[];
  /** Lat/lng polygon for map display */
  polygonLatlng?: [number, number][];
  mapId?: string;
  /** Coverage tool configuration (for coverage zones) */
  coverageConfig?: CoverageConfig | null;
  /** Generated coverage waypoints (boustrophedon sweep path) */
  coverageWaypoints?: Waypoint[] | null;
  createdAt: string;
  updatedAt: string;
}

export interface Schedule {
  trigger: "manual" | "once" | "cron";
  cron?: string;
  loop: boolean;
  /** Active window start (HH:MM format) */
  windowStart?: string;
  /** Active window end (HH:MM format) */
  windowEnd?: string;
}

export interface Mission {
  id: string;
  name: string;
  zoneId: string;
  roverId?: string;
  schedule: Schedule;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export const TaskStatus = {
  Pending: "pending",
  Assigned: "assigned",
  Active: "active",
  Done: "done",
  Failed: "failed",
  Cancelled: "cancelled",
} as const;

export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

export interface Task {
  id: string;
  missionId: string;
  roverId: string;
  status: TaskStatus;
  progress: number;
  waypoint: number;
  lap: number;
  error?: string;
  createdAt: string;
  startedAt?: string;
  endedAt?: string;
}

export interface ConnectedRover {
  roverId: string;
  connected: boolean;
  taskId?: string;
}

// =============================================================================
// Alert Types (client-side alerting)
// =============================================================================

export const AlertSeverity = {
  Critical: "critical",
  Warning: "warning",
  Info: "info",
} as const;
export type AlertSeverity = (typeof AlertSeverity)[keyof typeof AlertSeverity];

export interface Alert {
  id: string;
  severity: AlertSeverity;
  source: "rover" | "service" | "dispatch" | "gps" | "weather";
  sourceId: string;
  message: string;
  /** ISO timestamp (server-backed) */
  createdAt: string;
  acknowledgedAt?: string;
  acknowledgedBy?: string;
  clearedAt?: string;
  metadata?: Record<string, unknown>;
}

// =============================================================================
// Weather Types
// =============================================================================

export interface WeatherResponse {
  current: CurrentConditions | null;
  forecast: ForecastPeriod[];
  updatedAt: string | null;
}

export interface CurrentConditions {
  temperatureF: number;
  windSpeedMph: number;
  windDirection: string;
  shortForecast: string;
  humidityPct: number | null;
  isSnowing: boolean;
  iconUrl: string | null;
}

export interface ForecastPeriod {
  startTime: string;
  endTime: string;
  temperatureF: number;
  windSpeed: string;
  windDirection: string;
  shortForecast: string;
  precipitationProbability: number | null;
  isSnow: boolean;
}

// GPS/Base station status
export type Constellation = "gps" | "glonass" | "galileo" | "beidou" | "qzss" | "unknown";

export interface SatelliteInfo {
  prn: number;
  constellation: Constellation;
  elevation?: number;
  azimuth?: number;
  snr?: number;
  used: boolean;
}

export interface HistoryPoint {
  timestamp: number;
  hdop?: number;
  satellites: number;
  fixQuality: string;
}

export interface GpsStatus {
  connected: boolean;
  mode: string;
  fixQuality: string;
  satellites: number;
  latitude?: number;
  longitude?: number;
  altitude?: number;
  hdop?: number;
  vdop?: number;
  pdop?: number;
  satelliteInfo?: SatelliteInfo[];
  history?: HistoryPoint[];
  // Base station specific
  surveyIn?: {
    active: boolean;
    valid: boolean;
    duration: number;
    accuracy: number;
  };
  rtcmMessages?: {
    type: number;
    count: number;
    lastSeen: number;
  }[];
  clients?: number;
  lastUpdate: number;
}
