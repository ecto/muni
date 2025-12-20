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
} as const;

export type View = (typeof View)[keyof typeof View];

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
