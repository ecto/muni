/**
 * Binary protocol matching bvrd teleop crate.
 *
 * Commands (Operator -> Rover):
 * - 0x01 Twist: [type:u8] [linear:f64 LE] [angular:f64 LE] [boost:u8] [timestamp:u64 LE]
 * - 0x02 E-Stop: [type:u8]
 * - 0x03 Heartbeat: [type:u8]
 * - 0x04 SetMode: [type:u8] [mode:u8]
 * - 0x05 Tool: [type:u8] [axis:f32 LE] [motor:f32 LE] [action_a:u8] [action_b:u8]
 * - 0x06 E-Stop Release: [type:u8]
 *
 * Telemetry (Rover -> Operator):
 * - 0x10 Telemetry: [type:u8] [mode:u8] [pose:24B] [voltage:f64] [timestamp:u64] [velocity:16B] [temps:16B] [currents:16B] [health:2B]
 */

import {
  Mode,
  type Telemetry,
  type Pose,
  type Twist,
  type Power,
} from "./types";

// Message types
export const MSG_TWIST = 0x01;
export const MSG_ESTOP = 0x02;
export const MSG_HEARTBEAT = 0x03;
export const MSG_SET_MODE = 0x04;
export const MSG_TOOL = 0x05;
export const MSG_ESTOP_RELEASE = 0x06;
export const MSG_TELEMETRY = 0x10;
export const MSG_VIDEO_FRAME = 0x20;

// ============================================================================
// Command Encoding (Operator -> Rover)
// ============================================================================

export function encodeTwist(
  linear: number,
  angular: number,
  boost: boolean = false
): ArrayBuffer {
  // 1 (type) + 8 (linear) + 8 (angular) + 1 (boost) + 8 (timestamp) = 26 bytes
  const buf = new ArrayBuffer(26);
  const view = new DataView(buf);
  view.setUint8(0, MSG_TWIST);
  view.setFloat64(1, linear, true); // little-endian
  view.setFloat64(9, angular, true);
  view.setUint8(17, boost ? 1 : 0);
  // Timestamp in milliseconds since epoch
  const timestamp = BigInt(Date.now());
  view.setBigUint64(18, timestamp, true);
  return buf;
}

export function encodeEStop(): ArrayBuffer {
  const buf = new ArrayBuffer(1);
  const view = new DataView(buf);
  view.setUint8(0, MSG_ESTOP);
  return buf;
}

export function encodeEStopRelease(): ArrayBuffer {
  const buf = new ArrayBuffer(1);
  const view = new DataView(buf);
  view.setUint8(0, MSG_ESTOP_RELEASE);
  return buf;
}

export function encodeHeartbeat(): ArrayBuffer {
  const buf = new ArrayBuffer(1);
  const view = new DataView(buf);
  view.setUint8(0, MSG_HEARTBEAT);
  return buf;
}

export function encodeSetMode(mode: Mode): ArrayBuffer {
  const buf = new ArrayBuffer(2);
  const view = new DataView(buf);
  view.setUint8(0, MSG_SET_MODE);
  view.setUint8(1, mode);
  return buf;
}

export function encodeTool(
  axis: number,
  motor: number,
  actionA: boolean,
  actionB: boolean
): ArrayBuffer {
  const buf = new ArrayBuffer(11);
  const view = new DataView(buf);
  view.setUint8(0, MSG_TOOL);
  view.setFloat32(1, axis, true);
  view.setFloat32(5, motor, true);
  view.setUint8(9, actionA ? 1 : 0);
  view.setUint8(10, actionB ? 1 : 0);
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
}

export interface DecodedTelemetry {
  mode: Mode;
  pose: Pose;
  power: Power;
  velocity: Twist;
  motor_temps: [number, number, number, number];
  motor_currents: [number, number, number, number];
  timestamp_ms: number;
  health: SubsystemHealth;
}

export function decodeTelemetry(data: ArrayBuffer): DecodedTelemetry | null {
  const view = new DataView(data);

  // Minimum size check: 90 bytes base, optionally +2 bytes for health
  if (data.byteLength < 90) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_TELEMETRY) {
    return null;
  }

  let offset = 1;

  // Mode (u8)
  const modeValue = view.getUint8(offset);
  offset += 1;
  const mode = modeValue as Mode;

  // Pose (3x f64 = 24 bytes)
  const poseX = view.getFloat64(offset, true);
  offset += 8;
  const poseY = view.getFloat64(offset, true);
  offset += 8;
  const poseTheta = view.getFloat64(offset, true);
  offset += 8;

  // Battery voltage (f64)
  const batteryVoltage = view.getFloat64(offset, true);
  offset += 8;

  // Timestamp (u64)
  const timestampLow = view.getUint32(offset, true);
  const timestampHigh = view.getUint32(offset + 4, true);
  const timestamp_ms = timestampLow + timestampHigh * 0x100000000;
  offset += 8;

  // Velocity (2x f64 = 16 bytes)
  const velocityLinear = view.getFloat64(offset, true);
  offset += 8;
  const velocityAngular = view.getFloat64(offset, true);
  offset += 8;

  // Motor temps (4x f32 = 16 bytes)
  const motor_temps: [number, number, number, number] = [
    view.getFloat32(offset, true),
    view.getFloat32(offset + 4, true),
    view.getFloat32(offset + 8, true),
    view.getFloat32(offset + 12, true),
  ];
  offset += 16;

  // Motor currents (4x f32 = 16 bytes)
  const motor_currents: [number, number, number, number] = [
    view.getFloat32(offset, true),
    view.getFloat32(offset + 4, true),
    view.getFloat32(offset + 8, true),
    view.getFloat32(offset + 12, true),
  ];
  offset += 16;

  // Subsystem health (2 bytes, little-endian u16) - optional for backwards compatibility
  let health: SubsystemHealth;
  if (data.byteLength >= offset + 2) {
    const healthBits = view.getUint16(offset, true);
    health = {
      can_healthy: (healthBits & (1 << 0)) !== 0,
      recording_active: (healthBits & (1 << 1)) !== 0,
      gps_fix: (healthBits & (1 << 2)) !== 0,
      camera_active: (healthBits & (1 << 3)) !== 0,
      dispatch_connected: (healthBits & (1 << 4)) !== 0,
      discovery_connected: (healthBits & (1 << 5)) !== 0,
      lidar_active: (healthBits & (1 << 6)) !== 0,
      slam_running: (healthBits & (1 << 7)) !== 0,
    };
  } else {
    // Default health for older firmware without health field
    health = {
      can_healthy: false,
      recording_active: false,
      gps_fix: false,
      camera_active: false,
      dispatch_connected: false,
      discovery_connected: false,
      lidar_active: false,
      slam_running: false,
    };
  }

  return {
    mode,
    pose: { x: poseX, y: poseY, theta: poseTheta },
    power: { battery_voltage: batteryVoltage, system_current: 0 }, // Current not in this packet
    velocity: { linear: velocityLinear, angular: velocityAngular },
    motor_temps,
    motor_currents,
    timestamp_ms,
    health,
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
    velocity: decoded.velocity,
    motor_temps: decoded.motor_temps,
    connected: true,
    latency_ms: 0, // Computed by caller
    health: decoded.health,
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
