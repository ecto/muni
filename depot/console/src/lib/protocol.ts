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
 * Telemetry (Rover -> Operator) - 120 bytes:
 * - 0x11 Telemetry: [type:u8] [mode:u8] [seq:u16] [pad:4B] [pose:24B] [voltage:f64]
 *   [timestamp_us:u64] [cmd_vel:8B] [meas_vel:8B] [accel:8B] [temps:16B] [currents:16B]
 *   [health:2B] [odom_quality:2B] [dt_ms:f32] [ack_seq:u16] [ack_bits:u16] [crc32:u32]
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
export const MSG_TELEMETRY = 0x11;
export const MSG_VIDEO_FRAME = 0x20;

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

  // Battery voltage [32-39]
  const batteryVoltage = view.getFloat64(32, true);

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

  // CRC32 [116-119] - could verify but we trust UDP/WebRTC for now

  return {
    sequence,
    mode,
    pose: { x: poseX, y: poseY, theta: poseTheta },
    power: { battery_voltage: batteryVoltage, system_current: 0 },
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
  // Compute system current as sum of motor currents
  const systemCurrent = decoded.motor_currents.reduce((a, b) => a + b, 0);

  return {
    mode: decoded.mode,
    pose: decoded.pose,
    power: {
      battery_voltage: decoded.power.battery_voltage,
      system_current: systemCurrent,
    },
    velocity: decoded.cmd_velocity,
    motor_temps: decoded.motor_temps,
    connected: true,
    latency_ms: 0, // Computed by caller
    health: decoded.health,
    cpu_percent: decoded.cpu_percent,
    mem_percent: decoded.mem_percent,
    disk_percent: decoded.disk_percent,
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
