/**
 * Mutable obstacle store - bypasses React for performance.
 *
 * Obstacle data arrives at ~10Hz from the rover. Storing it in React state
 * would trigger unnecessary re-renders. The 3D overlay reads this
 * imperatively from useFrame via interpolation.
 *
 * Follows the same pattern as costmapStore.ts and pointCloudStore.ts.
 */

import type { DecodedObstacle } from "./protocol";
import {
  pushObstacleSnapshot,
  computeInterpolatedObstacles,
  clearObstacleInterpolation,
  type InterpolatedObstacle,
} from "./obstacleInterpolation";

interface ObstacleState {
  obstacles: DecodedObstacle[];
  /** Version counter - incremented on every update for change detection */
  version: number;
  /** Timestamp of last update (performance.now()) */
  timestamp: number;
}

const globalStore: ObstacleState = {
  obstacles: [],
  version: 0,
  timestamp: 0,
};

/**
 * Store new obstacle data. Called from WebRTC onmessage handler.
 * Pushes into interpolation buffer for smooth rendering.
 */
export function setObstacleData(obstacles: DecodedObstacle[]): void {
  globalStore.obstacles = obstacles;
  globalStore.version++;
  globalStore.timestamp = performance.now();
  pushObstacleSnapshot(obstacles);
}

/**
 * Read current obstacle data (raw, not interpolated). Called from useFrame.
 */
export function getObstacleData(): DecodedObstacle[] {
  return globalStore.obstacles;
}

/**
 * Get interpolated obstacles for smooth rendering.
 * Returns fresh values every frame (LERP/dead-reckoned).
 */
export function getInterpolatedObstacles(): InterpolatedObstacle[] {
  return computeInterpolatedObstacles(performance.now());
}

/**
 * Get version counter for change detection in useFrame.
 */
export function getObstacleVersion(): number {
  return globalStore.version;
}

/**
 * Clear all obstacle data. Called on disconnect.
 */
export function clearObstacleData(): void {
  globalStore.obstacles = [];
  globalStore.version++;
  globalStore.timestamp = performance.now();
  clearObstacleInterpolation();
}
