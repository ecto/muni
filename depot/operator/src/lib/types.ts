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
  boost: boolean; // L3 or Shift for full power mode
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

// View state
export const View = {
  Home: "home",
  Teleop: "teleop",
  Maps: "maps",
  Sessions: "sessions",
  SessionPlayback: "session-playback",
} as const;

export type View = (typeof View)[keyof typeof View];

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
  started_at: string; // ISO timestamp
  ended_at: string | null;
  duration_secs: number;
  gps_bounds: GpsSessionBounds | null;
  lidar_frames: number;
  camera_frames: number;
  pose_samples: number;
  session_file: string; // "session.rrd"
  // Computed/added by API
  session_dir: string; // Directory name (timestamp format)
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
  lastSeen: number; // timestamp
}
