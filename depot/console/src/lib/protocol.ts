/**
 * Binary protocol matching bvrd teleop crate.
 *
 * Commands (Operator -> Rover) - All have 8-byte header:
 * [type:u8] [flags:u8] [sequence:u16 LE] [timestamp_ms:u32 LE] [payload...]
 *
 * - 0x01 Twist:        [header] [linear:f32 LE] [angular:f32 LE] [boost:u8]
 * - 0x02 E-Stop:       [header]
 * - 0x03 Heartbeat:    [header]
 * - 0x04 SetMode:      [header] [mode:u8]
 * - 0x05 Tool:         [header] [axis:f32 LE] [motor:f32 LE] [action_a:u8] [action_b:u8]
 * - 0x06 E-Stop Release: [header]
 *
 * Telemetry (Rover -> Operator) - 148 bytes (120/128/132/140 bytes for legacy):
 * - 0x11 Telemetry: [type:u8] [mode:u8] [seq:u16] [pad:4B] [pose:24B] [voltage:f64]
 *   [timestamp_us:u64] [cmd_vel:8B] [meas_vel:8B] [accel:8B] [temps:16B] [currents:16B]
 *   [health:2B] [odom_quality:2B] [dt_ms:f32] [ack_seq:u16] [ack_bits:u16]
 *   [roll:f32] [pitch:f32] [mode_changed_at:u32] [lidar_temp:f32] [lidar_state:u8]
 *   [policy:9B] [fault_code:u8] [reserved:1B] [crc32:u32]
 */

import {
  Mode,
  type Telemetry,
  type Pose,
  type Twist,
  type Power,
  type WheelStatus,
} from "./types";

// Message types
export const MSG_TWIST = 0x01;
export const MSG_ESTOP = 0x02;
export const MSG_HEARTBEAT = 0x03;
export const MSG_SET_MODE = 0x04;
export const MSG_TOOL = 0x05;
export const MSG_ESTOP_RELEASE = 0x06;
export const MSG_LIDAR_TOGGLE = 0x07;
export const MSG_SET_GOAL = 0x08;
export const MSG_TELEMETRY = 0x11;
export const MSG_VIDEO_FRAME = 0x20;
export const MSG_POINT_CLOUD = 0x21;
export const MSG_COSTMAP = 0x22;
export const MSG_OBSTACLES = 0x23;
export const MSG_CAMERA_INFO = 0x24;

// Command flags
export const CMD_FLAG_MUST_ACK = 0x01;
export const CMD_FLAG_RETRANSMIT = 0x02;

// Header size (all commands)
const CMD_HEADER_SIZE = 8;

// ============================================================================
// Sequence Number Management
// ============================================================================

let commandSequence = 0;

/** Get next command sequence number (wraps at u16 max). */
export function nextSequence(): number {
  const seq = commandSequence;
  commandSequence = (commandSequence + 1) & 0xffff;
  return seq;
}

/** Get current timestamp in milliseconds (truncated to u32). */
function nowMs(): number {
  return Date.now() & 0xffffffff;
}

// ============================================================================
// Command Encoding (Operator -> Rover)
// ============================================================================

/** Build command header. */
function buildHeader(
  view: DataView,
  msgType: number,
  flags: number = 0
): number {
  const seq = nextSequence();
  const ts = nowMs();
  view.setUint8(0, msgType);
  view.setUint8(1, flags);
  view.setUint16(2, seq, true);
  view.setUint32(4, ts, true);
  return seq;
}

export function encodeTwist(
  linear: number,
  angular: number,
  boost: boolean = false
): ArrayBuffer {
  // 8 (header) + 4 (linear) + 4 (angular) + 1 (boost) = 17 bytes
  const buf = new ArrayBuffer(17);
  const view = new DataView(buf);
  buildHeader(view, MSG_TWIST);
  view.setFloat32(8, linear, true);
  view.setFloat32(12, angular, true);
  view.setUint8(16, boost ? 1 : 0);
  return buf;
}

export function encodeEStop(): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE);
  const view = new DataView(buf);
  buildHeader(view, MSG_ESTOP, CMD_FLAG_MUST_ACK);
  return buf;
}

export function encodeEStopRelease(): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE);
  const view = new DataView(buf);
  buildHeader(view, MSG_ESTOP_RELEASE, CMD_FLAG_MUST_ACK);
  return buf;
}

export function encodeHeartbeat(): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE);
  const view = new DataView(buf);
  buildHeader(view, MSG_HEARTBEAT);
  return buf;
}

export function encodeSetMode(mode: Mode): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE + 1);
  const view = new DataView(buf);
  buildHeader(view, MSG_SET_MODE, CMD_FLAG_MUST_ACK);
  view.setUint8(8, mode);
  return buf;
}

export function encodeSetGoal(x: number, y: number): ArrayBuffer {
  // 8 (header) + 8 (x: f64 LE) + 8 (y: f64 LE) = 24 bytes
  const buf = new ArrayBuffer(24);
  const view = new DataView(buf);
  buildHeader(view, MSG_SET_GOAL, CMD_FLAG_MUST_ACK);
  view.setFloat64(8, x, true);
  view.setFloat64(16, y, true);
  return buf;
}

export function encodeTool(
  axis: number,
  motor: number,
  actionA: boolean,
  actionB: boolean
): ArrayBuffer {
  // 8 (header) + 4 (axis) + 4 (motor) + 1 (action_a) + 1 (action_b) = 18 bytes
  const buf = new ArrayBuffer(18);
  const view = new DataView(buf);
  buildHeader(view, MSG_TOOL);
  view.setFloat32(8, axis, true);
  view.setFloat32(12, motor, true);
  view.setUint8(16, actionA ? 1 : 0);
  view.setUint8(17, actionB ? 1 : 0);
  return buf;
}

// ============================================================================
// Telemetry Decoding (Rover -> Operator)
// ============================================================================

// Subsystem health flags (decoded from 2-byte bit field)
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

export interface DecodedTelemetry {
  sequence: number;
  mode: Mode;
  pose: Pose;
  power: Power;
  cmd_velocity: Twist;
  meas_velocity: { linear: number; angular: number };
  acceleration: { linear: number; angular: number };
  motor_temps: [number, number, number, number];
  motor_currents: [number, number, number, number];
  timestamp_us: number;
  health: SubsystemHealth;
  odometry_quality: number;
  dt_ms: number;
  last_cmd_seq: number;
  ack_bits: number;
  /** System CPU usage (0-100%) */
  cpu_percent: number;
  /** System memory usage (0-100%) */
  mem_percent: number;
  /** System disk usage (0-100%) */
  disk_percent: number;
  /** Epoch seconds when the current mode was entered (0 = unknown/legacy) */
  mode_changed_at: number;
  /** LiDAR core temperature (°C, 0 = unavailable) */
  lidar_core_temp_c: number;
  /** LiDAR work state (0=unknown, 1=sampling, 2=idle, 3=ready, 4=error) */
  lidar_work_state: number;
  /** Whether the BC policy is actively driving */
  policy_active: boolean;
  /** Policy intention X in world meters */
  intention_x: number;
  /** Policy intention Y in world meters */
  intention_y: number;
  /** Fault code (0 = no fault, 1 = watchdog timeout) */
  fault_code: number;
}

/**
 * Decode telemetry from 120-byte binary format.
 */
export function decodeTelemetry(data: ArrayBuffer): DecodedTelemetry | null {
  const view = new DataView(data);

  // Expect exactly 120 bytes
  if (data.byteLength < 120) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_TELEMETRY) {
    return null;
  }

  // Parse header
  const mode = view.getUint8(1) as Mode;
  const sequence = view.getUint16(2, true);
  // System metrics [4-6], reserved [7]
  const cpu_percent = view.getUint8(4);
  const mem_percent = view.getUint8(5);
  const disk_percent = view.getUint8(6);

  // Pose (3x f64 = 24 bytes) [8-31]
  const poseX = view.getFloat64(8, true);
  const poseY = view.getFloat64(16, true);
  const poseTheta = view.getFloat64(24, true);

  // Battery voltage + system current [32-39]
  // New format: f32 voltage [32-35] + f32 current [36-39]
  // Old format: f64 voltage [32-39] (no current)
  // Detect by checking if f32 read produces a sane voltage (0-100V range)
  const voltageF32 = view.getFloat32(32, true);
  const isNewFormat = voltageF32 > 0 && voltageF32 < 100;
  const batteryVoltage = isNewFormat ? voltageF32 : view.getFloat64(32, true);
  const systemCurrent = isNewFormat ? view.getFloat32(36, true) : 0;

  // Timestamp [40-47]
  const timestampLow = view.getUint32(40, true);
  const timestampHigh = view.getUint32(44, true);
  const timestamp_us = timestampLow + timestampHigh * 0x100000000;

  // Commanded velocity [48-55]
  const cmdLinear = view.getFloat32(48, true);
  const cmdAngular = view.getFloat32(52, true);

  // Measured velocity [56-63]
  const measLinear = view.getFloat32(56, true);
  const measAngular = view.getFloat32(60, true);

  // Acceleration [64-71]
  const accelLinear = view.getFloat32(64, true);
  const accelAngular = view.getFloat32(68, true);

  // Motor temps (4x f32 = 16 bytes) [72-87]
  const motor_temps: [number, number, number, number] = [
    view.getFloat32(72, true),
    view.getFloat32(76, true),
    view.getFloat32(80, true),
    view.getFloat32(84, true),
  ];

  // Motor currents (4x f32 = 16 bytes) [88-103]
  const motor_currents: [number, number, number, number] = [
    view.getFloat32(88, true),
    view.getFloat32(92, true),
    view.getFloat32(96, true),
    view.getFloat32(100, true),
  ];

  // Health bits [104-105]
  const healthBits = view.getUint16(104, true);
  const health: SubsystemHealth = {
    can_healthy: (healthBits & (1 << 0)) !== 0,
    recording_active: (healthBits & (1 << 1)) !== 0,
    gps_fix: (healthBits & (1 << 2)) !== 0,
    camera_active: (healthBits & (1 << 3)) !== 0,
    dispatch_connected: (healthBits & (1 << 4)) !== 0,
    discovery_connected: (healthBits & (1 << 5)) !== 0,
    lidar_active: (healthBits & (1 << 6)) !== 0,
    slam_running: (healthBits & (1 << 7)) !== 0,
    wheel_status: [
      ((healthBits >> 8) & 0x3) as WheelStatus,
      ((healthBits >> 10) & 0x3) as WheelStatus,
      ((healthBits >> 12) & 0x3) as WheelStatus,
      ((healthBits >> 14) & 0x3) as WheelStatus,
    ],
  };

  // Odometry quality [106-107]
  const odometry_quality = view.getUint16(106, true);

  // dt_ms [108-111]
  const dt_ms = view.getFloat32(108, true);

  // ACK fields [112-115]
  const last_cmd_seq = view.getUint16(112, true);
  const ack_bits = view.getUint16(114, true);

  // Roll/pitch from IMU [116-123] (128-byte+ format)
  // For backwards compatibility with 120-byte format, default to 0
  let roll = 0;
  let pitch = 0;
  if (data.byteLength >= 128) {
    roll = view.getFloat32(116, true);
    pitch = view.getFloat32(120, true);
    // CRC32 is at [124-127] for 128-byte format, [128-131] for 132-byte
  }

  // mode_changed_at [124-127] (132-byte format, epoch seconds)
  let mode_changed_at = 0;
  if (data.byteLength >= 132) {
    mode_changed_at = view.getUint32(124, true);
  }

  // LiDAR status [128-132] (140-byte format)
  let lidar_core_temp_c = 0;
  let lidar_work_state = 0;
  if (data.byteLength >= 140) {
    lidar_core_temp_c = view.getFloat32(128, true);
    lidar_work_state = view.getUint8(132);
  }

  // Policy intention [133-141] (148-byte format)
  let policy_active = false;
  let intention_x = 0;
  let intention_y = 0;
  let fault_code = 0;
  if (data.byteLength >= 148) {
    policy_active = view.getUint8(133) !== 0;
    intention_x = view.getFloat32(134, true);
    intention_y = view.getFloat32(138, true);
    fault_code = view.getUint8(142);
  }

  return {
    sequence,
    mode,
    pose: { x: poseX, y: poseY, theta: poseTheta, roll, pitch },
    power: { battery_voltage: batteryVoltage, system_current: systemCurrent },
    cmd_velocity: { linear: cmdLinear, angular: cmdAngular },
    meas_velocity: { linear: measLinear, angular: measAngular },
    acceleration: { linear: accelLinear, angular: accelAngular },
    motor_temps,
    motor_currents,
    timestamp_us,
    health,
    odometry_quality,
    dt_ms,
    last_cmd_seq,
    ack_bits,
    cpu_percent,
    mem_percent,
    disk_percent,
    mode_changed_at,
    lidar_core_temp_c,
    lidar_work_state,
    policy_active,
    intention_x,
    intention_y,
    fault_code,
  };
}

// ============================================================================
// Full Telemetry with computed fields
// ============================================================================

export function telemetryFromDecoded(decoded: DecodedTelemetry): Telemetry {
  return {
    mode: decoded.mode,
    pose: decoded.pose,
    power: {
      battery_voltage: decoded.power.battery_voltage,
      system_current: decoded.power.system_current,
    },
    velocity: decoded.cmd_velocity,
    motor_temps: decoded.motor_temps,
    connected: true,
    latency_ms: 0, // Computed by caller
    health: decoded.health,
    cpu_percent: decoded.cpu_percent,
    mem_percent: decoded.mem_percent,
    disk_percent: decoded.disk_percent,
    modeChangedAt: decoded.mode_changed_at > 0 ? decoded.mode_changed_at * 1000 : Date.now(),
    lidar:
      decoded.lidar_core_temp_c > 0 || decoded.lidar_work_state > 0
        ? { temp_c: decoded.lidar_core_temp_c, work_state: decoded.lidar_work_state }
        : undefined,
    policyIntention: decoded.policy_active
      ? { x: decoded.intention_x, y: decoded.intention_y }
      : null,
    faultCode: decoded.fault_code,
  };
}

// ============================================================================
// Video Frame Decoding (Rover -> Operator)
// ============================================================================

export interface DecodedVideoFrame {
  cameraId: number;
  timestamp_ms: number;
  width: number;
  height: number;
  jpegData: Uint8Array;
}

/**
 * Decode a video frame message.
 * Format: [type:u8] [camera_id:u8] [timestamp:u64 LE] [width:u16 LE] [height:u16 LE] [jpeg_data:...]
 */
export function decodeVideoFrame(data: ArrayBuffer): DecodedVideoFrame | null {
  const view = new DataView(data);

  // Minimum size: 1 (type) + 1 (camera_id) + 8 (timestamp) + 2 (width) + 2 (height) + some data
  if (data.byteLength < 15) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_VIDEO_FRAME) {
    return null;
  }

  // Camera ID
  const cameraId = view.getUint8(1);

  // Timestamp (u64 LE)
  const timestampLow = view.getUint32(2, true);
  const timestampHigh = view.getUint32(6, true);
  const timestamp_ms = timestampLow + timestampHigh * 0x100000000;

  // Dimensions
  const width = view.getUint16(10, true);
  const height = view.getUint16(12, true);

  // JPEG data (rest of buffer)
  const jpegData = new Uint8Array(data, 14);

  return {
    cameraId,
    timestamp_ms,
    width,
    height,
    jpegData,
  };
}

/**
 * Convert JPEG data to a blob URL for use in textures.
 * We use blob URLs but the caller should NOT revoke them - the texture
 * loader needs time to load them asynchronously.
 */
export function videoFrameToBlobUrl(frame: DecodedVideoFrame): string {
  // Create blob directly from the Uint8Array
  // Use type assertion since TypeScript 5.7+ is overly strict about ArrayBufferLike vs ArrayBuffer
  const blob = new Blob([frame.jpegData as BlobPart], { type: "image/jpeg" });
  return URL.createObjectURL(blob);
}

// ============================================================================
// Camera Info Decoding (Rover -> Operator)
// ============================================================================

export interface DecodedCameraInfo {
  cameraId: number;
  width: number;
  height: number;
  /** Mount position [forward, left, up] in meters */
  position: [number, number, number];
  /** Mount rotation [roll, pitch, yaw] in radians */
  rotation: [number, number, number];
  /** Horizontal field of view (radians) */
  fovH: number;
  /** Vertical field of view (radians) */
  fovV: number;
}

/**
 * Decode a camera info message (38 bytes).
 * Format: [type:u8] [camera_id:u8] [width:u16 LE] [height:u16 LE]
 *         [pos_fwd:f32] [pos_left:f32] [pos_up:f32]
 *         [rot_roll:f32] [rot_pitch:f32] [rot_yaw:f32]
 *         [fov_h:f32] [fov_v:f32]
 */
export function decodeCameraInfo(data: ArrayBuffer): DecodedCameraInfo | null {
  if (data.byteLength < 38) {
    return null;
  }

  const view = new DataView(data);
  if (view.getUint8(0) !== MSG_CAMERA_INFO) {
    return null;
  }

  return {
    cameraId: view.getUint8(1),
    width: view.getUint16(2, true),
    height: view.getUint16(4, true),
    position: [
      view.getFloat32(6, true),
      view.getFloat32(10, true),
      view.getFloat32(14, true),
    ],
    rotation: [
      view.getFloat32(18, true),
      view.getFloat32(22, true),
      view.getFloat32(26, true),
    ],
    fovH: view.getFloat32(30, true),
    fovV: view.getFloat32(34, true),
  };
}

// ============================================================================
// Point Cloud Protocol (Rover -> Operator)
// ============================================================================

/** Header size for point cloud message. */
const POINT_CLOUD_HEADER_SIZE = 32;
/** Bytes per point in point cloud message. */
const BYTES_PER_POINT = 8;

export interface DecodedPointCloud {
  frameId: number;
  /** Flat array of x,y,z coordinates: [x0, y0, z0, x1, y1, z1, ...] */
  points: Float32Array;
  /** Reflectivity values per point (0-255) */
  reflectivity: Uint8Array;
  /** Tag values per point - contains confidence (bits 0-1) and return type (bits 2-3) */
  tag: Uint8Array;
}

/**
 * Decode a point cloud message from binary format.
 *
 * Binary format:
 * [0]      u8    MSG_POINT_CLOUD (0x21)
 * [1]      u8    flags (reserved)
 * [2-3]    u16   point_count (LE)
 * [4-7]    u32   frame_id (LE)
 * [8-11]   f32   scale_x (LE)
 * [12-15]  f32   scale_y (LE)
 * [16-19]  f32   scale_z (LE)
 * [20-23]  f32   offset_x (LE)
 * [24-27]  f32   offset_y (LE)
 * [28-31]  f32   offset_z (LE)
 * [32...]  per-point: [x:i16 LE][y:i16 LE][z:i16 LE][reflectivity:u8][tag:u8]
 */
export function decodePointCloud(data: ArrayBuffer): DecodedPointCloud | null {
  if (data.byteLength < POINT_CLOUD_HEADER_SIZE) {
    return null;
  }

  const view = new DataView(data);
  const msgType = view.getUint8(0);

  if (msgType !== MSG_POINT_CLOUD) {
    return null;
  }

  const pointCount = view.getUint16(2, true);
  const frameId = view.getUint32(4, true);

  const expectedSize = POINT_CLOUD_HEADER_SIZE + pointCount * BYTES_PER_POINT;
  if (data.byteLength < expectedSize) {
    return null;
  }

  // Read scale and offset
  const scaleX = view.getFloat32(8, true);
  const scaleY = view.getFloat32(12, true);
  const scaleZ = view.getFloat32(16, true);
  const offsetX = view.getFloat32(20, true);
  const offsetY = view.getFloat32(24, true);
  const offsetZ = view.getFloat32(28, true);

  // Decode points
  const points = new Float32Array(pointCount * 3);
  const reflectivity = new Uint8Array(pointCount);
  const tag = new Uint8Array(pointCount);

  for (let i = 0; i < pointCount; i++) {
    const offset = POINT_CLOUD_HEADER_SIZE + i * BYTES_PER_POINT;

    // Read quantized coordinates
    const qx = view.getInt16(offset, true);
    const qy = view.getInt16(offset + 2, true);
    const qz = view.getInt16(offset + 4, true);
    const refl = view.getUint8(offset + 6);
    const pointTag = view.getUint8(offset + 7);

    // Dequantize: raw = quantized * scale + offset
    points[i * 3] = qx * scaleX + offsetX;
    points[i * 3 + 1] = qy * scaleY + offsetY;
    points[i * 3 + 2] = qz * scaleZ + offsetZ;
    reflectivity[i] = refl;
    tag[i] = pointTag;
  }

  return { frameId, points, reflectivity, tag };
}

/**
 * Encode a LiDAR toggle command.
 * Format: [type:u8] [flags:u8] [sequence:u16 LE] [timestamp_ms:u32 LE]
 *         [enabled:u8] [rate_hz:u8] [max_points:u16 LE]
 */
export function encodeLidarToggle(
  enabled: boolean,
  rateHz: number = 5,
  maxPoints: number = 1000,
): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE + 4);
  const view = new DataView(buf);
  buildHeader(view, MSG_LIDAR_TOGGLE);
  view.setUint8(CMD_HEADER_SIZE, enabled ? 1 : 0);
  view.setUint8(CMD_HEADER_SIZE + 1, Math.max(1, Math.min(10, rateHz)));
  view.setUint16(CMD_HEADER_SIZE + 2, Math.max(100, Math.min(10000, maxPoints)), true);
  return buf;
}

// ============================================================================
// Costmap Decoding (Rover -> Operator)
// ============================================================================

/** Costmap header size in bytes. */
const COSTMAP_HEADER_SIZE = 18;

/** Encoding types. */
const COSTMAP_ENCODING_RAW = 0;
const COSTMAP_ENCODING_RLE = 1;

export interface DecodedCostmap {
  width: number;
  height: number;
  resolution: number;
  originX: number;
  originY: number;
  /** Cost per cell: 0=free, 253=inscribed, 254=lethal */
  cells: Uint8Array;
}

/**
 * Decode a costmap message from binary format.
 *
 * Header (18 bytes):
 *   [0]      u8    MSG_COSTMAP (0x22)
 *   [1]      u8    encoding (0=raw, 1=RLE)
 *   [2-3]    u16   width (cells, LE)
 *   [4-5]    u16   height (cells, LE)
 *   [6-9]    f32   resolution (m, LE)
 *   [10-13]  f32   origin_x (m, LE)
 *   [14-17]  f32   origin_y (m, LE)
 *   [18...]  cell data (raw or RLE)
 */
export function decodeCostmap(data: ArrayBuffer): DecodedCostmap | null {
  if (data.byteLength < COSTMAP_HEADER_SIZE) {
    return null;
  }

  const view = new DataView(data);
  const msgType = view.getUint8(0);
  if (msgType !== MSG_COSTMAP) {
    return null;
  }

  const encoding = view.getUint8(1);
  const width = view.getUint16(2, true);
  const height = view.getUint16(4, true);
  const resolution = view.getFloat32(6, true);
  const originX = view.getFloat32(10, true);
  const originY = view.getFloat32(14, true);

  const payload = new Uint8Array(data, COSTMAP_HEADER_SIZE);
  const expectedLen = width * height;

  let cells: Uint8Array;
  if (encoding === COSTMAP_ENCODING_RAW) {
    if (payload.length < expectedLen) {
      return null;
    }
    cells = payload.slice(0, expectedLen);
  } else if (encoding === COSTMAP_ENCODING_RLE) {
    const decoded = rleDecodeU8(payload, expectedLen);
    if (!decoded) {
      return null;
    }
    cells = decoded;
  } else {
    return null;
  }

  return { width, height, resolution, originX, originY, cells };
}

/** RLE decode: [count:u8][value:u8] pairs → flat Uint8Array. */
function rleDecodeU8(encoded: Uint8Array, expectedLen: number): Uint8Array | null {
  const out = new Uint8Array(expectedLen);
  let outIdx = 0;

  for (let i = 0; i + 1 < encoded.length; i += 2) {
    const count = encoded[i];
    const value = encoded[i + 1];
    for (let j = 0; j < count; j++) {
      if (outIdx >= expectedLen) {
        return null; // overflow
      }
      out[outIdx++] = value;
    }
  }

  return outIdx === expectedLen ? out : null;
}

// ============================================================================
// Obstacle Decoding (MSG_OBSTACLES = 0x23)
// ============================================================================

/** Header size for obstacle messages. */
const OBSTACLES_HEADER_SIZE = 4;

/** Per-obstacle record size in bytes (v3, tracked with rotation). */
const OBSTACLE_RECORD_SIZE = 60;

/** A tracked obstacle with bounding box and centroid in world coordinates. */
export interface DecodedObstacle {
  /** Stable track ID. */
  trackId: number;
  /** Obstacle class: 0=Unknown, 1=Pole, 2=Vehicle, 3=Pedestrian, 4=Wall, 5=Debris. */
  obstacleClass: number;
  /** Tracker confidence (0-255). */
  confidence: number;
  /** Track age in frames. */
  age: number;
  centroidX: number;
  centroidY: number;
  bboxMinX: number;
  bboxMinY: number;
  bboxMaxX: number;
  bboxMaxY: number;
  cellCount: number;
  /** Area in square meters. */
  area: number;
  /** Minimum observed Z (height) in meters. */
  minZ: number;
  /** Maximum observed Z (height) in meters. */
  maxZ: number;
  /** PCA principal axis rotation in radians. */
  rotation: number;
  /** Velocity X in m/s. */
  velocityX: number;
  /** Velocity Y in m/s. */
  velocityY: number;
}

/**
 * Decode an obstacles message from binary v3 format.
 *
 * Header (4 bytes):
 *   [0]      u8    MSG_OBSTACLES (0x23)
 *   [1]      u8    obstacle_count
 *   [2-3]    u16   version (3)
 *
 * Per obstacle (60 bytes):
 *   [0-3]    u32   track_id
 *   [4]      u8    class
 *   [5]      u8    confidence
 *   [6-7]    u16   age
 *   [8-11]   f32   centroid_x
 *   [12-15]  f32   centroid_y
 *   [16-19]  f32   bbox_min_x
 *   [20-23]  f32   bbox_min_y
 *   [24-27]  f32   bbox_max_x
 *   [28-31]  f32   bbox_max_y
 *   [32-35]  u32   cell_count
 *   [36-39]  f32   area
 *   [40-43]  f32   min_z
 *   [44-47]  f32   max_z
 *   [48-51]  f32   rotation
 *   [52-55]  f32   velocity_x
 *   [56-59]  f32   velocity_y
 */
export function decodeObstacles(data: ArrayBuffer): DecodedObstacle[] | null {
  if (data.byteLength < OBSTACLES_HEADER_SIZE) {
    return null;
  }

  const view = new DataView(data);
  if (view.getUint8(0) !== MSG_OBSTACLES) {
    return null;
  }

  const count = view.getUint8(1);
  const expectedLen = OBSTACLES_HEADER_SIZE + count * OBSTACLE_RECORD_SIZE;
  if (data.byteLength < expectedLen) {
    return null;
  }

  const obstacles: DecodedObstacle[] = [];
  for (let i = 0; i < count; i++) {
    const offset = OBSTACLES_HEADER_SIZE + i * OBSTACLE_RECORD_SIZE;
    obstacles.push({
      trackId: view.getUint32(offset, true),
      obstacleClass: view.getUint8(offset + 4),
      confidence: view.getUint8(offset + 5),
      age: view.getUint16(offset + 6, true),
      centroidX: view.getFloat32(offset + 8, true),
      centroidY: view.getFloat32(offset + 12, true),
      bboxMinX: view.getFloat32(offset + 16, true),
      bboxMinY: view.getFloat32(offset + 20, true),
      bboxMaxX: view.getFloat32(offset + 24, true),
      bboxMaxY: view.getFloat32(offset + 28, true),
      cellCount: view.getUint32(offset + 32, true),
      area: view.getFloat32(offset + 36, true),
      minZ: view.getFloat32(offset + 40, true),
      maxZ: view.getFloat32(offset + 44, true),
      rotation: view.getFloat32(offset + 48, true),
      velocityX: view.getFloat32(offset + 52, true),
      velocityY: view.getFloat32(offset + 56, true),
    });
  }

  return obstacles;
}
