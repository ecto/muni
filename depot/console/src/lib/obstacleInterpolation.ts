/**
 * Client-side obstacle interpolation for smooth rendering.
 *
 * Uses one-frame-behind interpolation: renders at (now - delay) so we can
 * LERP between two known snapshots instead of snapping. Falls back to
 * velocity-based extrapolation when data is stale.
 *
 * Design principles:
 * - Global mutable state (no React involvement)
 * - LERP between consecutive snapshots for smooth motion
 * - Velocity extrapolation when past the newest snapshot
 * - Grace period before declaring tracks dead (suppresses single-frame dropout ghosts)
 * - Fade in new tracks, fade out confirmed-dead tracks
 * - Confidence filtering to suppress noisy detections
 */

import type { DecodedObstacle } from "./protocol";

/** An interpolated obstacle ready for rendering. */
export interface InterpolatedObstacle extends DecodedObstacle {
  /** Opacity for fade in/out (0-1). */
  opacity: number;
}

/** A snapshot of obstacles at a specific time. */
interface ObstacleSnapshot {
  receivedAt: number; // performance.now()
  obstacles: DecodedObstacle[];
}

/** Buffer capacity (4 slots at ~10Hz = 400ms history). */
const BUFFER_CAPACITY = 4;

/** Maximum gap before clearing buffer (ms). */
const MAX_GAP_MS = 300;

/** Render one snapshot behind for smooth LERP (ms). */
const INTERPOLATION_DELAY_MS = 100;

/** Maximum velocity extrapolation beyond newest snapshot (ms). */
const MAX_EXTRAPOLATION_MS = 150;

/** Minimum tracker confidence to render (0-255). Legacy formats use 255. */
const MIN_CONFIDENCE = 10;

/** Grace period before a missing track is declared dead (ms). */
const DEATH_GRACE_MS = 150;

/** Fade-in duration for new tracks (ms). */
const FADE_IN_MS = 200;

/** Fade-out duration for confirmed-dead tracks (ms). */
const FADE_OUT_MS = 200;

// ============================================================================
// Global mutable state
// ============================================================================

const buffer: (ObstacleSnapshot | null)[] = Array(BUFFER_CAPACITY).fill(null);
let head = 0;
let count = 0;

/** Tracks missing from recent snapshots, awaiting grace period. */
const pendingDeath = new Map<
  number,
  { missingSince: number; obstacle: DecodedObstacle }
>();

/** Tracks confirmed dead, being faded out. */
const dyingTracks = new Map<
  number,
  { diedAt: number; obstacle: DecodedObstacle }
>();

/** First-seen times for fade-in. */
const birthTimes = new Map<number, number>();

// ============================================================================
// Public API
// ============================================================================

/** Push a new obstacle snapshot. Called from WebRTC onmessage handler. */
export function pushObstacleSnapshot(obstacles: DecodedObstacle[]): void {
  const now = performance.now();

  // Large gap -> reset everything
  const prev = count > 0 ? buffer[head] : null;
  if (prev && now - prev.receivedAt > MAX_GAP_MS) {
    buffer.fill(null);
    head = 0;
    count = 0;
    pendingDeath.clear();
    dyingTracks.clear();
    birthTimes.clear();
  }

  // Write into ring buffer
  const newHead = (head + 1) % BUFFER_CAPACITY;
  buffer[newHead] = { receivedAt: now, obstacles };
  head = newHead;
  count = Math.min(count + 1, BUFFER_CAPACITY);

  // Current track IDs
  const currentIds = new Set<number>();
  for (const obs of obstacles) {
    currentIds.add(obs.trackId);
    if (!birthTimes.has(obs.trackId)) {
      birthTimes.set(obs.trackId, now);
    }
    // Reappeared — cancel any pending/dying state
    pendingDeath.delete(obs.trackId);
    dyingTracks.delete(obs.trackId);
  }

  // Tracks that disappeared since previous snapshot
  const prevSnap =
    count >= 2
      ? buffer[(newHead - 1 + BUFFER_CAPACITY) % BUFFER_CAPACITY]
      : null;
  if (prevSnap) {
    for (const obs of prevSnap.obstacles) {
      if (
        !currentIds.has(obs.trackId) &&
        !pendingDeath.has(obs.trackId) &&
        !dyingTracks.has(obs.trackId)
      ) {
        pendingDeath.set(obs.trackId, { missingSince: now, obstacle: obs });
      }
    }
  }

  // Promote pending deaths past grace period
  for (const [id, info] of pendingDeath) {
    if (now - info.missingSince >= DEATH_GRACE_MS) {
      dyingTracks.set(id, { diedAt: now, obstacle: info.obstacle });
      pendingDeath.delete(id);
    }
  }

  // Clean up expired dying tracks
  for (const [id, info] of dyingTracks) {
    if (now - info.diedAt > FADE_OUT_MS) {
      dyingTracks.delete(id);
      birthTimes.delete(id);
    }
  }
}

/** Compute interpolated obstacles for the current frame. */
export function computeInterpolatedObstacles(
  now: number,
): InterpolatedObstacle[] {
  if (count === 0) return [];

  const newest = buffer[head]!;
  const prevIdx = (head - 1 + BUFFER_CAPACITY) % BUFFER_CAPACITY;
  const prev = count >= 2 ? buffer[prevIdx] : null;

  const renderTime = now - INTERPOLATION_DELAY_MS;
  const result: InterpolatedObstacle[] = [];

  if (!prev) {
    // Single snapshot — use directly
    for (const obs of newest.obstacles) {
      if (obs.confidence < MIN_CONFIDENCE) continue;
      result.push({ ...obs, opacity: computeFadeIn(obs.trackId, now) });
    }
    appendDyingTracks(result, now);
    return result;
  }

  const segmentDuration = newest.receivedAt - prev.receivedAt;

  // Build prev map for LERP matching
  const prevMap = new Map<number, DecodedObstacle>();
  for (const obs of prev.obstacles) {
    prevMap.set(obs.trackId, obs);
  }

  if (
    segmentDuration > 0 &&
    renderTime >= prev.receivedAt &&
    renderTime <= newest.receivedAt
  ) {
    // Happy path: LERP between prev and newest
    const t = (renderTime - prev.receivedAt) / segmentDuration;
    for (const obs of newest.obstacles) {
      if (obs.confidence < MIN_CONFIDENCE) continue;
      const p = prevMap.get(obs.trackId);
      if (p) {
        result.push({
          ...obs,
          centroidX: lerp(p.centroidX, obs.centroidX, t),
          centroidY: lerp(p.centroidY, obs.centroidY, t),
          bboxMinX: lerp(p.bboxMinX, obs.bboxMinX, t),
          bboxMinY: lerp(p.bboxMinY, obs.bboxMinY, t),
          bboxMaxX: lerp(p.bboxMaxX, obs.bboxMaxX, t),
          bboxMaxY: lerp(p.bboxMaxY, obs.bboxMaxY, t),
          opacity: computeFadeIn(obs.trackId, now),
        });
      } else {
        // New track not in prev — show at newest position
        result.push({ ...obs, opacity: computeFadeIn(obs.trackId, now) });
      }
    }
  } else if (renderTime > newest.receivedAt) {
    // Past newest — extrapolate with velocity, clamped
    const dtSec =
      Math.min(renderTime - newest.receivedAt, MAX_EXTRAPOLATION_MS) / 1000;
    for (const obs of newest.obstacles) {
      if (obs.confidence < MIN_CONFIDENCE) continue;
      result.push({
        ...obs,
        centroidX: obs.centroidX + obs.velocityX * dtSec,
        centroidY: obs.centroidY + obs.velocityY * dtSec,
        bboxMinX: obs.bboxMinX + obs.velocityX * dtSec,
        bboxMinY: obs.bboxMinY + obs.velocityY * dtSec,
        bboxMaxX: obs.bboxMaxX + obs.velocityX * dtSec,
        bboxMaxY: obs.bboxMaxY + obs.velocityY * dtSec,
        opacity: computeFadeIn(obs.trackId, now),
      });
    }
  } else {
    // Before prev (shouldn't happen normally) — use newest
    for (const obs of newest.obstacles) {
      if (obs.confidence < MIN_CONFIDENCE) continue;
      result.push({ ...obs, opacity: computeFadeIn(obs.trackId, now) });
    }
  }

  appendDyingTracks(result, now);
  return result;
}

/** Clear all interpolation state. Called on disconnect. */
export function clearObstacleInterpolation(): void {
  buffer.fill(null);
  head = 0;
  count = 0;
  pendingDeath.clear();
  dyingTracks.clear();
  birthTimes.clear();
}

// ============================================================================
// Internal
// ============================================================================

function appendDyingTracks(
  result: InterpolatedObstacle[],
  now: number,
): void {
  for (const [, info] of dyingTracks) {
    const elapsed = now - info.diedAt;
    if (elapsed > FADE_OUT_MS) continue;
    const fadeOut = 1 - elapsed / FADE_OUT_MS;
    const dtSec = elapsed / 1000;
    const obs = info.obstacle;
    result.push({
      ...obs,
      centroidX: obs.centroidX + obs.velocityX * dtSec,
      centroidY: obs.centroidY + obs.velocityY * dtSec,
      bboxMinX: obs.bboxMinX + obs.velocityX * dtSec,
      bboxMinY: obs.bboxMinY + obs.velocityY * dtSec,
      bboxMaxX: obs.bboxMaxX + obs.velocityX * dtSec,
      bboxMaxY: obs.bboxMaxY + obs.velocityY * dtSec,
      opacity: fadeOut,
    });
  }
}

function computeFadeIn(trackId: number, now: number): number {
  const birth = birthTimes.get(trackId);
  if (birth === undefined) return 1;
  const age = now - birth;
  if (age >= FADE_IN_MS) return 1;
  return age / FADE_IN_MS;
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}
