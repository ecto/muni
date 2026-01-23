/**
 * Client-side pose interpolation for smooth rover rendering.
 *
 * Uses Hermite spline interpolation with server-provided velocity
 * for smooth motion between telemetry updates.
 *
 * Design principles:
 * - Pure functions, no side effects
 * - Server-authoritative: never diverge far from server state
 * - Short extrapolation only: max 100ms, then hold position
 */

import type { Pose } from "./types";

/** A snapshot of pose with velocity at a specific time. */
export interface PoseSnapshot {
  /** Server timestamp in microseconds */
  serverTimestamp: number;
  /** Local receive time (performance.now()) */
  receivedAt: number;
  /** Position and orientation */
  pose: Pose;
  /** Velocity (linear m/s, angular rad/s) */
  velocity: { linear: number; angular: number };
}

/** Ring buffer state for storing recent snapshots. */
export interface InterpolationState {
  snapshots: (PoseSnapshot | null)[];
  head: number;
  count: number;
}

/** Maximum extrapolation time in milliseconds. */
const MAX_EXTRAPOLATION_MS = 100;

/** Buffer capacity (8 slots = ~160ms at 50Hz). */
const BUFFER_CAPACITY = 8;

/** Maximum gap between snapshots before clearing buffer (ms). */
const MAX_SNAPSHOT_GAP_MS = 200;

/**
 * Global interpolation buffer - bypasses React for performance.
 * Updated at 50Hz from telemetry, read at 60Hz from useFrame.
 */
const globalBuffer: InterpolationState = {
  snapshots: Array(BUFFER_CAPACITY).fill(null),
  head: 0,
  count: 0,
};

/**
 * Get the global interpolation buffer for reading in useFrame.
 */
export function getInterpolationBuffer(): InterpolationState {
  return globalBuffer;
}

/**
 * Push a snapshot directly to the global buffer.
 * Called from telemetry handler - no React updates triggered.
 * Computes velocity from pose deltas for smooth interpolation.
 */
export function pushSnapshotDirect(snapshot: PoseSnapshot): void {
  // Check for large gap (backpressure recovery, tab switch, etc.)
  // Clear buffer to snap to current position instead of interpolating through stale data
  const prevIdx = globalBuffer.head;
  const prev = globalBuffer.snapshots[prevIdx];
  if (prev && globalBuffer.count > 0) {
    const gap = snapshot.receivedAt - prev.receivedAt;
    if (gap > MAX_SNAPSHOT_GAP_MS) {
      // Large gap - clear buffer, snap to current position
      globalBuffer.snapshots.fill(null);
      globalBuffer.head = 0;
      globalBuffer.count = 0;
    }
  }

  const head = (globalBuffer.head + 1) % BUFFER_CAPACITY;

  // Compute velocity from pose delta if we have a previous snapshot
  // (Re-fetch prev since we may have cleared the buffer)
  const currentPrev = globalBuffer.snapshots[globalBuffer.head];
  if (currentPrev && globalBuffer.count > 0) {
    const dt = (snapshot.receivedAt - currentPrev.receivedAt) / 1000; // seconds
    if (dt > 0.001 && dt < 0.5) { // Valid time delta (1ms to 500ms)
      const dx = snapshot.pose.x - currentPrev.pose.x;
      const dy = snapshot.pose.y - currentPrev.pose.y;
      const dtheta = normalizeAngle(snapshot.pose.theta - currentPrev.pose.theta);

      // Linear velocity is distance moved / time
      const distance = Math.sqrt(dx * dx + dy * dy);
      // Sign based on heading alignment with movement direction
      const moveAngle = Math.atan2(dy, dx);
      const headingDiff = Math.abs(normalizeAngle(moveAngle - currentPrev.pose.theta));
      const sign = headingDiff > Math.PI / 2 ? -1 : 1;

      snapshot.velocity = {
        linear: sign * distance / dt,
        angular: dtheta / dt,
      };
    }
  }

  globalBuffer.snapshots[head] = snapshot;
  globalBuffer.head = head;
  globalBuffer.count = Math.min(globalBuffer.count + 1, BUFFER_CAPACITY);
}

/**
 * Create initial interpolation state.
 * @deprecated Use getInterpolationBuffer() instead
 */
export function createInterpolationState(): InterpolationState {
  return {
    snapshots: Array(BUFFER_CAPACITY).fill(null),
    head: 0,
    count: 0,
  };
}

/**
 * Push a new snapshot into the buffer.
 * @deprecated Use pushSnapshotDirect() instead
 */
export function pushSnapshot(
  state: InterpolationState,
  snapshot: PoseSnapshot
): InterpolationState {
  const head = (state.head + 1) % BUFFER_CAPACITY;
  state.snapshots[head] = snapshot;
  state.head = head;
  state.count = Math.min(state.count + 1, BUFFER_CAPACITY);
  return state;
}

/**
 * Get snapshots in chronological order (oldest to newest).
 */
export function getOrderedSnapshots(state: InterpolationState): PoseSnapshot[] {
  const result: PoseSnapshot[] = [];
  if (state.count === 0) return result;

  // Start from oldest (head - count + 1) and go to head
  for (let i = 0; i < state.count; i++) {
    const idx =
      (state.head - state.count + 1 + i + BUFFER_CAPACITY) % BUFFER_CAPACITY;
    const snapshot = state.snapshots[idx];
    if (snapshot) {
      result.push(snapshot);
    }
  }

  return result;
}

/**
 * Get the two most recent snapshots for interpolation.
 */
function getRecentSnapshots(
  state: InterpolationState
): [PoseSnapshot | null, PoseSnapshot | null] {
  if (state.count === 0) return [null, null];
  if (state.count === 1) {
    return [state.snapshots[state.head], null];
  }

  const newest = state.snapshots[state.head];
  const prevIdx = (state.head - 1 + BUFFER_CAPACITY) % BUFFER_CAPACITY;
  const prev = state.snapshots[prevIdx];

  return [newest, prev];
}

/**
 * Hermite spline interpolation between two poses.
 *
 * Uses cubic Hermite spline with position and velocity at each endpoint
 * for smooth C1-continuous interpolation.
 *
 * @param p0 Start position
 * @param v0 Start velocity (tangent)
 * @param p1 End position
 * @param v1 End velocity (tangent)
 * @param t Parameter in [0, 1]
 */
function hermite(p0: number, v0: number, p1: number, v1: number, t: number): number {
  const t2 = t * t;
  const t3 = t2 * t;

  // Hermite basis functions
  const h00 = 2 * t3 - 3 * t2 + 1;
  const h10 = t3 - 2 * t2 + t;
  const h01 = -2 * t3 + 3 * t2;
  const h11 = t3 - t2;

  return h00 * p0 + h10 * v0 + h01 * p1 + h11 * v1;
}

/**
 * Normalize angle to [-PI, PI].
 */
function normalizeAngle(angle: number): number {
  while (angle > Math.PI) angle -= 2 * Math.PI;
  while (angle < -Math.PI) angle += 2 * Math.PI;
  return angle;
}

/**
 * Interpolate between two poses using Hermite spline.
 */
function interpolatePoses(
  from: PoseSnapshot,
  to: PoseSnapshot,
  t: number,
  dtSeconds: number
): Pose {
  // Convert velocities to position deltas for tangent scaling
  const fromLinearTangent = from.velocity.linear * dtSeconds;
  const toLinearTangent = to.velocity.linear * dtSeconds;
  const fromAngularTangent = from.velocity.angular * dtSeconds;
  const toAngularTangent = to.velocity.angular * dtSeconds;

  // For x/y, we need to account for heading
  // Approximate: tangent in world frame based on heading
  const fromHeading = from.pose.theta;
  const toHeading = to.pose.theta;

  const fromTangentX = fromLinearTangent * Math.cos(fromHeading);
  const fromTangentY = fromLinearTangent * Math.sin(fromHeading);
  const toTangentX = toLinearTangent * Math.cos(toHeading);
  const toTangentY = toLinearTangent * Math.sin(toHeading);

  const x = hermite(from.pose.x, fromTangentX, to.pose.x, toTangentX, t);
  const y = hermite(from.pose.y, fromTangentY, to.pose.y, toTangentY, t);

  // For theta, interpolate through shortest path
  const thetaDiff = normalizeAngle(to.pose.theta - from.pose.theta);
  const theta = hermite(
    from.pose.theta,
    fromAngularTangent,
    from.pose.theta + thetaDiff,
    toAngularTangent,
    t
  );

  // Lerp roll/pitch for smooth visual transitions (handles angle wrapping)
  const rollDiff = normalizeAngle(to.pose.roll - from.pose.roll);
  const pitchDiff = normalizeAngle(to.pose.pitch - from.pose.pitch);
  const roll = normalizeAngle(from.pose.roll + rollDiff * t);
  const pitch = normalizeAngle(from.pose.pitch + pitchDiff * t);
  return { x, y, theta: normalizeAngle(theta), roll, pitch };
}

/**
 * Extrapolate pose forward using velocity.
 */
function extrapolatePose(snapshot: PoseSnapshot, dtSeconds: number): Pose {
  const dx = snapshot.velocity.linear * dtSeconds * Math.cos(snapshot.pose.theta);
  const dy = snapshot.velocity.linear * dtSeconds * Math.sin(snapshot.pose.theta);
  const dtheta = snapshot.velocity.angular * dtSeconds;

  // Roll/pitch are passed through (high-frequency IMU data, no extrapolation)
  return {
    x: snapshot.pose.x + dx,
    y: snapshot.pose.y + dy,
    theta: normalizeAngle(snapshot.pose.theta + dtheta),
    roll: snapshot.pose.roll,
    pitch: snapshot.pose.pitch,
  };
}

export interface RenderPoseResult {
  pose: Pose;
  isExtrapolating: boolean;
  /** Age of newest snapshot in ms */
  snapshotAge: number;
}

/**
 * Compute the render pose for the current frame.
 *
 * This is the main interpolation function called every frame.
 *
 * @param state Interpolation buffer state
 * @param clientNow Current client time (performance.now())
 * @param renderDelay How far behind real-time to render (ms, for jitter buffer)
 */
export function computeRenderPose(
  state: InterpolationState,
  clientNow: number,
  renderDelay: number = 0
): RenderPoseResult {
  const [newest, prev] = getRecentSnapshots(state);

  // No data yet - return origin
  if (!newest) {
    return {
      pose: { x: 0, y: 0, theta: 0, roll: 0, pitch: 0 },
      isExtrapolating: false,
      snapshotAge: Infinity,
    };
  }

  const snapshotAge = clientNow - newest.receivedAt;
  const targetTime = clientNow - renderDelay;

  // Only one snapshot - extrapolate from it
  if (!prev) {
    const dt = (targetTime - newest.receivedAt) / 1000;

    if (dt > MAX_EXTRAPOLATION_MS / 1000) {
      // Beyond max extrapolation - hold position
      return {
        pose: newest.pose,
        isExtrapolating: true,
        snapshotAge,
      };
    }

    if (dt <= 0) {
      // Target time is before snapshot - use snapshot directly
      return {
        pose: newest.pose,
        isExtrapolating: false,
        snapshotAge,
      };
    }

    // Extrapolate forward
    return {
      pose: extrapolatePose(newest, dt),
      isExtrapolating: true,
      snapshotAge,
    };
  }

  // Two snapshots available - interpolate or extrapolate
  const newestTime = newest.receivedAt;
  const prevTime = prev.receivedAt;
  const segmentDuration = newestTime - prevTime;

  // Avoid division by zero
  if (segmentDuration <= 0) {
    return {
      pose: newest.pose,
      isExtrapolating: false,
      snapshotAge,
    };
  }

  // Where does targetTime fall relative to our snapshots?
  if (targetTime >= newestTime) {
    // Target is after newest - extrapolate
    const dt = (targetTime - newestTime) / 1000;

    if (dt > MAX_EXTRAPOLATION_MS / 1000) {
      // Beyond max extrapolation - hold position
      return {
        pose: newest.pose,
        isExtrapolating: true,
        snapshotAge,
      };
    }

    return {
      pose: extrapolatePose(newest, dt),
      isExtrapolating: true,
      snapshotAge,
    };
  }

  if (targetTime <= prevTime) {
    // Target is before prev - use prev (shouldn't happen normally)
    return {
      pose: prev.pose,
      isExtrapolating: false,
      snapshotAge,
    };
  }

  // Target is between prev and newest - interpolate
  const t = (targetTime - prevTime) / segmentDuration;
  const dtSeconds = segmentDuration / 1000;

  return {
    pose: interpolatePoses(prev, newest, t, dtSeconds),
    isExtrapolating: false,
    snapshotAge,
  };
}
