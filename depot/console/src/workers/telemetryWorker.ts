/**
 * Web Worker for decoding telemetry and video frames off the main thread.
 *
 * This worker receives raw ArrayBuffer messages from WebRTC data channels
 * and decodes them into structured data, reducing main thread blocking.
 *
 * Uses Transferable objects for zero-copy ArrayBuffer transfer.
 */

// Web Worker global scope - use (self as any) for postMessage with transfer list
/* eslint-disable @typescript-eslint/no-explicit-any */
const workerSelf = self as any;

// Message types matching protocol.ts
const MSG_TELEMETRY = 0x11;
const MSG_VIDEO_FRAME = 0x20;
const MSG_POINT_CLOUD = 0x21;

// Point cloud constants
const POINT_CLOUD_HEADER_SIZE = 32;
const BYTES_PER_POINT = 8;

// ============================================================================
// Input/Output Message Types
// ============================================================================

/** Messages from main thread to worker */
export type WorkerInput =
  | { type: "telemetry"; data: ArrayBuffer }
  | { type: "video"; data: ArrayBuffer }
  | { type: "pointcloud"; data: ArrayBuffer };

/** Decoded telemetry output */
export interface DecodedTelemetryOutput {
  sequence: number;
  mode: number;
  pose: {
    x: number;
    y: number;
    theta: number;
    roll: number;
    pitch: number;
  };
  power: { battery_voltage: number; system_current: number };
  cmd_velocity: { linear: number; angular: number };
  meas_velocity: { linear: number; angular: number };
  acceleration: { linear: number; angular: number };
  motor_temps: [number, number, number, number];
  motor_currents: [number, number, number, number];
  timestamp_us: number;
  health: {
    can_healthy: boolean;
    recording_active: boolean;
    gps_fix: boolean;
    camera_active: boolean;
    dispatch_connected: boolean;
    discovery_connected: boolean;
    lidar_active: boolean;
    slam_running: boolean;
    wheel_status: [number, number, number, number];
  };
  odometry_quality: number;
  dt_ms: number;
  last_cmd_seq: number;
  ack_bits: number;
  cpu_percent: number;
  mem_percent: number;
  disk_percent: number;
}

/** Decoded video frame output (JPEG data is transferable) */
export interface DecodedVideoOutput {
  cameraId: number;
  timestamp_ms: number;
  width: number;
  height: number;
  /** JPEG data - transferred back as Uint8Array */
  jpegData: Uint8Array;
}

/** Decoded point cloud output */
export interface DecodedPointCloudOutput {
  frameId: number;
  points: Float32Array;
  reflectivity: Uint8Array;
  /** Tag values per point - contains confidence (bits 0-1) and return type (bits 2-3) */
  tag: Uint8Array;
}

/** Messages from worker to main thread */
export type WorkerOutput =
  | { type: "telemetry"; decoded: DecodedTelemetryOutput; receivedAt: number }
  | { type: "video"; decoded: DecodedVideoOutput; receivedAt: number }
  | { type: "pointcloud"; decoded: DecodedPointCloudOutput; receivedAt: number }
  | { type: "error"; channel: string; message: string };

// ============================================================================
// Decoding Functions (duplicated from protocol.ts for worker isolation)
// ============================================================================

function decodeTelemetry(data: ArrayBuffer): DecodedTelemetryOutput | null {
  const view = new DataView(data);

  if (data.byteLength < 120) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_TELEMETRY) {
    return null;
  }

  const mode = view.getUint8(1);
  const sequence = view.getUint16(2, true);
  const cpu_percent = view.getUint8(4);
  const mem_percent = view.getUint8(5);
  const disk_percent = view.getUint8(6);

  const poseX = view.getFloat64(8, true);
  const poseY = view.getFloat64(16, true);
  const poseTheta = view.getFloat64(24, true);

  // Detect old (f64 voltage) vs new (f32 voltage + f32 current) format
  const voltageF32 = view.getFloat32(32, true);
  const isNewFormat = voltageF32 > 0 && voltageF32 < 100;
  const batteryVoltage = isNewFormat ? voltageF32 : view.getFloat64(32, true);
  const systemCurrent = isNewFormat ? view.getFloat32(36, true) : 0;

  const timestampLow = view.getUint32(40, true);
  const timestampHigh = view.getUint32(44, true);
  const timestamp_us = timestampLow + timestampHigh * 0x100000000;

  const cmdLinear = view.getFloat32(48, true);
  const cmdAngular = view.getFloat32(52, true);

  const measLinear = view.getFloat32(56, true);
  const measAngular = view.getFloat32(60, true);

  const accelLinear = view.getFloat32(64, true);
  const accelAngular = view.getFloat32(68, true);

  const motor_temps: [number, number, number, number] = [
    view.getFloat32(72, true),
    view.getFloat32(76, true),
    view.getFloat32(80, true),
    view.getFloat32(84, true),
  ];

  const motor_currents: [number, number, number, number] = [
    view.getFloat32(88, true),
    view.getFloat32(92, true),
    view.getFloat32(96, true),
    view.getFloat32(100, true),
  ];

  const healthBits = view.getUint16(104, true);
  const health = {
    can_healthy: (healthBits & (1 << 0)) !== 0,
    recording_active: (healthBits & (1 << 1)) !== 0,
    gps_fix: (healthBits & (1 << 2)) !== 0,
    camera_active: (healthBits & (1 << 3)) !== 0,
    dispatch_connected: (healthBits & (1 << 4)) !== 0,
    discovery_connected: (healthBits & (1 << 5)) !== 0,
    lidar_active: (healthBits & (1 << 6)) !== 0,
    slam_running: (healthBits & (1 << 7)) !== 0,
    wheel_status: [
      (healthBits >> 8) & 0x3,
      (healthBits >> 10) & 0x3,
      (healthBits >> 12) & 0x3,
      (healthBits >> 14) & 0x3,
    ] as [number, number, number, number],
  };

  const odometry_quality = view.getUint16(106, true);
  const dt_ms = view.getFloat32(108, true);
  const last_cmd_seq = view.getUint16(112, true);
  const ack_bits = view.getUint16(114, true);

  let roll = 0;
  let pitch = 0;
  if (data.byteLength >= 128) {
    roll = view.getFloat32(116, true);
    pitch = view.getFloat32(120, true);
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
  };
}

function decodeVideoFrame(data: ArrayBuffer): DecodedVideoOutput | null {
  const view = new DataView(data);

  if (data.byteLength < 15) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_VIDEO_FRAME) {
    return null;
  }

  const cameraId = view.getUint8(1);

  const timestampLow = view.getUint32(2, true);
  const timestampHigh = view.getUint32(6, true);
  const timestamp_ms = timestampLow + timestampHigh * 0x100000000;

  const width = view.getUint16(10, true);
  const height = view.getUint16(12, true);

  // Copy JPEG data to a new Uint8Array that can be transferred
  const jpegData = new Uint8Array(data.byteLength - 14);
  jpegData.set(new Uint8Array(data, 14));

  return {
    cameraId,
    timestamp_ms,
    width,
    height,
    jpegData,
  };
}

function decodePointCloud(data: ArrayBuffer): DecodedPointCloudOutput | null {
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

  const scaleX = view.getFloat32(8, true);
  const scaleY = view.getFloat32(12, true);
  const scaleZ = view.getFloat32(16, true);
  const offsetX = view.getFloat32(20, true);
  const offsetY = view.getFloat32(24, true);
  const offsetZ = view.getFloat32(28, true);

  const points = new Float32Array(pointCount * 3);
  const reflectivity = new Uint8Array(pointCount);
  const tag = new Uint8Array(pointCount);

  for (let i = 0; i < pointCount; i++) {
    const offset = POINT_CLOUD_HEADER_SIZE + i * BYTES_PER_POINT;

    const qx = view.getInt16(offset, true);
    const qy = view.getInt16(offset + 2, true);
    const qz = view.getInt16(offset + 4, true);
    const refl = view.getUint8(offset + 6);
    const pointTag = view.getUint8(offset + 7);

    points[i * 3] = qx * scaleX + offsetX;
    points[i * 3 + 1] = qy * scaleY + offsetY;
    points[i * 3 + 2] = qz * scaleZ + offsetZ;
    reflectivity[i] = refl;
    tag[i] = pointTag;
  }

  return { frameId, points, reflectivity, tag };
}

// ============================================================================
// Message Handler
// ============================================================================

workerSelf.onmessage = (event: MessageEvent<WorkerInput>) => {
  const { type, data } = event.data;
  const receivedAt = performance.now();

  switch (type) {
    case "telemetry": {
      const decoded = decodeTelemetry(data);
      if (decoded) {
        const output: WorkerOutput = { type: "telemetry", decoded, receivedAt };
        workerSelf.postMessage(output);
      } else {
        const output: WorkerOutput = {
          type: "error",
          channel: "telemetry",
          message: `Failed to decode telemetry, size: ${data.byteLength}`,
        };
        workerSelf.postMessage(output);
      }
      break;
    }

    case "video": {
      const decoded = decodeVideoFrame(data);
      if (decoded) {
        const output: WorkerOutput = { type: "video", decoded, receivedAt };
        // Transfer the jpegData buffer back to main thread
        workerSelf.postMessage(output, [decoded.jpegData.buffer]);
      } else {
        const output: WorkerOutput = {
          type: "error",
          channel: "video",
          message: `Failed to decode video frame, size: ${data.byteLength}`,
        };
        workerSelf.postMessage(output);
      }
      break;
    }

    case "pointcloud": {
      const decoded = decodePointCloud(data);
      if (decoded) {
        const output: WorkerOutput = { type: "pointcloud", decoded, receivedAt };
        // Transfer the typed arrays back to main thread
        workerSelf.postMessage(output, [
          decoded.points.buffer,
          decoded.reflectivity.buffer,
        ]);
      } else {
        const output: WorkerOutput = {
          type: "error",
          channel: "pointcloud",
          message: `Failed to decode point cloud, size: ${data.byteLength}`,
        };
        workerSelf.postMessage(output);
      }
      break;
    }
  }
};
