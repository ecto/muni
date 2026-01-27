/**
 * Client-side obstacle interpolation for smooth rendering.
 *
 * Ring buffer of obstacle snapshots with LERP/dead-reckoning between
 * server updates. Follows the same pattern as interpolation.ts.
 *
 * Design principles:
 * - Global mutable state (no React involvement)
 * - LERP between snapshots, dead-reckon with velocity when extrapolating
 * - Fade in new tracks, fade out dead tracks
 * - Clear buffer on large gaps
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

/** Buffer capacity (4 slots at 10Hz = 400ms history). */
const BUFFER_CAPACITY = 4;

/** Maximum gap before clearing buffer (ms). */
const MAX_GAP_MS = 300;

/** Maximum extrapolation time (ms). */
const MAX_EXTRAPOLATION_MS = 150;

/** Fade-in duration for new tracks (ms). */
const FADE_IN_MS = 200;

/** Fade-out duration for dead tracks (ms). */
const FADE_OUT_MS = 300;

// ============================================================================
// Global mutable state
// ============================================================================

const buffer: (ObstacleSnapshot | null)[] = Array(BUFFER_CAPACITY).fill(null);
let head = 0;
let count = 0;

/**
 * Map of trackId -> { lastSeenAt, lastObstacle } for fade-out tracking.
 * Tracks that disappear from snapshots get dead-reckoned and faded out.
 */
const dyingTracks = new Map<
  number,
  { lastSeenAt: number; obstacle: DecodedObstacle }
>();

/**
 * Map of trackId -> firstSeenAt for fade-in tracking.
 */
const birthTimes = new Map<number, number>();

// ============================================================================
// Public API
// ============================================================================

/** Push a new obstacle snapshot. Called from WebRTC onmessage handler. */
export function pushObstacleSnapshot(obstacles: DecodedObstacle[]): void {
  const now = performance.now();

  // Check for large gap -> clear
  const prevIdx = head;
  const prev = buffer[prevIdx];
  if (prev && count > 0) {
    const gap = now - prev.receivedAt;
    if (gap > MAX_GAP_MS) {
      buffer.fill(null);
      head = 0;
      count = 0;
      dyingTracks.clear();
      birthTimes.clear();
    }
  }

  const newHead = (head + 1) % BUFFER_CAPACITY;
  buffer[newHead] = { receivedAt: now, obstacles };
  head = newHead;
  count = Math.min(count + 1, BUFFER_CAPACITY);

  // Track births
  const currentIds = new Set<number>();
  for (const obs of obstacles) {
    currentIds.add(obs.trackId);
    if (!birthTimes.has(obs.trackId)) {
      birthTimes.set(obs.trackId, now);
    }
    // Remove from dying if it reappeared
    dyingTracks.delete(obs.trackId);
  }

  // Move disappeared tracks to dying
  const prevSnap = count >= 2 ? buffer[(newHead - 1 + BUFFER_CAPACITY) % BUFFER_CAPACITY] : null;
  if (prevSnap) {
    for (const obs of prevSnap.obstacles) {
      if (!currentIds.has(obs.trackId) && !dyingTracks.has(obs.trackId)) {
        dyingTracks.set(obs.trackId, { lastSeenAt: now, obstacle: obs });
      }
    }
  }

  // Clean up old dying tracks and stale birth entries
  for (const [id, info] of dyingTracks) {
    if (now - info.lastSeenAt > FADE_OUT_MS) {
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

  const timeSinceNewest = now - newest.receivedAt;
  const result: InterpolatedObstacle[] = [];

  // Build a map of newest obstacles by trackId
  const newestMap = new Map<number, DecodedObstacle>();
  for (const obs of newest.obstacles) {
    newestMap.set(obs.trackId, obs);
  }

  // Build a map of prev obstacles by trackId (for interpolation)
  const prevMap = new Map<number, DecodedObstacle>();
  if (prev) {
    for (const obs of prev.obstacles) {
      prevMap.set(obs.trackId, obs);
    }
  }

  if (timeSinceNewest <= 0 || !prev) {
    // At or before newest snapshot — use newest directly
    for (const obs of newest.obstacles) {
      result.push({
        ...obs,
        opacity: computeFadeIn(obs.trackId, now),
      });
    }
  } else if (prev) {
    const segmentDuration = newest.receivedAt - prev.receivedAt;

    if (segmentDuration > 0 && timeSinceNewest <= MAX_EXTRAPOLATION_MS) {
      // Extrapolate forward from newest using velocity
      for (const obs of newest.obstacles) {
        const dtSec = timeSinceNewest / 1000;
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
      // Beyond max extrapolation — hold position
      for (const obs of newest.obstacles) {
        result.push({
          ...obs,
          opacity: computeFadeIn(obs.trackId, now),
        });
      }
    }
  }

  // Add dying tracks (fade out)
  for (const [, info] of dyingTracks) {
    const elapsed = now - info.lastSeenAt;
    if (elapsed > FADE_OUT_MS) continue;

    const opacity = 1 - elapsed / FADE_OUT_MS;
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
      opacity: opacity * computeFadeIn(obs.trackId, now),
    });
  }

  return result;
}

/** Clear all interpolation state. Called on disconnect. */
export function clearObstacleInterpolation(): void {
  buffer.fill(null);
  head = 0;
  count = 0;
  dyingTracks.clear();
  birthTimes.clear();
}

// ============================================================================
// Internal
// ============================================================================

function computeFadeIn(trackId: number, now: number): number {
  const birth = birthTimes.get(trackId);
  if (birth === undefined) return 1;
  const age = now - birth;
  if (age >= FADE_IN_MS) return 1;
  return age / FADE_IN_MS;
}
