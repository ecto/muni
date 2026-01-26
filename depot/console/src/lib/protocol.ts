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
 * Telemetry (Rover -> Operator) - 128 bytes (120 bytes for legacy):
 * - 0x11 Telemetry: [type:u8] [mode:u8] [seq:u16] [pad:4B] [pose:24B] [voltage:f64]
 *   [timestamp_us:u64] [cmd_vel:8B] [meas_vel:8B] [accel:8B] [temps:16B] [currents:16B]
 *   [health:2B] [odom_quality:2B] [dt_ms:f32] [ack_seq:u16] [ack_bits:u16]
 *   [roll:f32] [pitch:f32] [crc32:u32]
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
export const MSG_TELEMETRY = 0x11;
export const MSG_VIDEO_FRAME = 0x20;
export const MSG_POINT_CLOUD = 0x21;
export const MSG_COSTMAP = 0x22;

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
  const seq = buildHeader(view, MSG_SET_MODE, CMD_FLAG_MUST_ACK);
  view.setUint8(8, mode);
  // Debug: log the encoded bytes
  const bytes = new Uint8Array(buf);
  console.log(`[protocol] encodeSetMode(${mode}) seq=${seq} bytes=[${Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join(' ')}]`);
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

  // Roll/pitch from IMU [116-123] (new 128-byte format)
  // For backwards compatibility with 120-byte format, default to 0
  let roll = 0;
  let pitch = 0;
  if (data.byteLength >= 128) {
    roll = view.getFloat32(116, true);
    pitch = view.getFloat32(120, true);
    // CRC32 is at [124-127] for new format
  }
  // CRC32 [116-119] for old format, [124-127] for new - not verified

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
    modeChangedAt: Date.now(),
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
 * Format: [type:u8] [flags:u8] [sequence:u16 LE] [timestamp_ms:u32 LE] [enabled:u8]
 */
export function encodeLidarToggle(enabled: boolean): ArrayBuffer {
  const buf = new ArrayBuffer(CMD_HEADER_SIZE + 1);
  const view = new DataView(buf);
  buildHeader(view, MSG_LIDAR_TOGGLE);
  view.setUint8(CMD_HEADER_SIZE, enabled ? 1 : 0);
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
