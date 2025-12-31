// Mode enum matching bvrd
export const Mode = {
  Disabled: 0,
  Idle: 1,
  Teleop: 2,
  Autonomous: 3,
  EStop: 4,
  Fault: 5,
} as const;

export type Mode = (typeof Mode)[keyof typeof Mode];

export const ModeLabels: Record<Mode, string> = {
  [Mode.Disabled]: "Disabled",
  [Mode.Idle]: "Idle",
  [Mode.Teleop]: "Teleop",
  [Mode.Autonomous]: "Autonomous",
  [Mode.EStop]: "E-Stop",
  [Mode.Fault]: "Fault",
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
}

export interface Telemetry {
  mode: Mode;
  pose: Pose;
  power: Power;
  velocity: Twist;
  motor_temps: [number, number, number, number];
  connected: boolean;
  latency_ms: number;
}

export interface GamepadInput {
  linear: number;
  angular: number;
  toolAxis: number;
  actionA: boolean;
  actionB: boolean;
  estop: boolean;
  enable: boolean;
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
  address: string;
  videoAddress: string;
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

// GPS/Base station status
export interface GpsStatus {
  connected: boolean;
  mode: string;
  fixQuality: string;
  satellites: number;
  latitude?: number;
  longitude?: number;
  altitude?: number;
  hdop?: number;
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
