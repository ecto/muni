/**
 * Binary protocol matching bvrd teleop crate.
 *
 * Commands (Operator → Rover):
 * - 0x01 Twist: [type:u8] [linear:f64 LE] [angular:f64 LE]
 * - 0x02 E-Stop: [type:u8]
 * - 0x03 Heartbeat: [type:u8]
 * - 0x04 SetMode: [type:u8] [mode:u8]
 * - 0x05 Tool: [type:u8] [axis:f32 LE] [motor:f32 LE] [action_a:u8] [action_b:u8]
 * - 0x06 E-Stop Release: [type:u8]
 *
 * Telemetry (Rover → Operator):
 * - 0x10 Telemetry: [type:u8] [mode:u8] [pose:24B] [voltage:f64] [timestamp:u64] [velocity:16B] [temps:16B] [currents:16B]
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
// Command Encoding (Operator → Rover)
// ============================================================================

export function encodeTwist(linear: number, angular: number): ArrayBuffer {
  const buf = new ArrayBuffer(17);
  const view = new DataView(buf);
  view.setUint8(0, MSG_TWIST);
  view.setFloat64(1, linear, true); // little-endian
  view.setFloat64(9, angular, true);
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
// Telemetry Decoding (Rover → Operator)
// ============================================================================

export interface DecodedTelemetry {
  mode: Mode;
  pose: Pose;
  power: Power;
  velocity: Twist;
  motor_temps: [number, number, number, number];
  motor_currents: [number, number, number, number];
  timestamp_ms: number;
}

export function decodeTelemetry(data: ArrayBuffer): DecodedTelemetry | null {
  const view = new DataView(data);

  // Minimum size check
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

  return {
    mode,
    pose: { x: poseX, y: poseY, theta: poseTheta },
    power: { battery_voltage: batteryVoltage, system_current: 0 }, // Current not in this packet
    velocity: { linear: velocityLinear, angular: velocityAngular },
    motor_temps,
    motor_currents,
    timestamp_ms,
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
  };
}

// ============================================================================
// Video Frame Decoding (Rover → Operator)
// ============================================================================

export interface DecodedVideoFrame {
  timestamp_ms: number;
  width: number;
  height: number;
  jpegData: Uint8Array;
}

/**
 * Decode a video frame message.
 * Format: [type:u8] [timestamp:u64 LE] [width:u16 LE] [height:u16 LE] [jpeg_data:...]
 */
export function decodeVideoFrame(data: ArrayBuffer): DecodedVideoFrame | null {
  const view = new DataView(data);

  // Minimum size: 1 (type) + 8 (timestamp) + 2 (width) + 2 (height) + some data
  if (data.byteLength < 14) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_VIDEO_FRAME) {
    return null;
  }

  // Timestamp (u64 LE)
  const timestampLow = view.getUint32(1, true);
  const timestampHigh = view.getUint32(5, true);
  const timestamp_ms = timestampLow + timestampHigh * 0x100000000;

  // Dimensions
  const width = view.getUint16(9, true);
  const height = view.getUint16(11, true);

  // JPEG data (rest of buffer)
  const jpegData = new Uint8Array(data, 13);

  return {
    timestamp_ms,
    width,
    height,
    jpegData,
  };
}

/**
 * Convert JPEG data to a blob URL for use in textures.
 */
export function videoFrameToBlobUrl(frame: DecodedVideoFrame): string {
  // Create a new Uint8Array to ensure we have a proper ArrayBuffer (not SharedArrayBuffer)
  const data = new Uint8Array(frame.jpegData);
  const blob = new Blob([data], { type: "image/jpeg" });
  return URL.createObjectURL(blob);
}
