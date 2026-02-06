/**
 * Mutable path store - bypasses React for performance.
 *
 * Navigation path data arrives at ~2Hz from the rover. Storing it in React
 * state would trigger unnecessary re-renders. The 3D overlay reads this
 * imperatively from useFrame.
 *
 * Follows the same pattern as costmapStore.ts.
 */

import type { DecodedPathWaypoint } from "./protocol";

interface PathState {
  /** Navigation state (see NavState in protocol.ts). */
  state: number;
  /** Path waypoints in world meters (ENU). */
  waypoints: DecodedPathWaypoint[];
  /** Index of the current target waypoint. */
  currentWaypoint: number;
  /** Distance to current goal (meters). */
  distanceToGoal: number;
  /** Version counter - incremented on every update for change detection. */
  version: number;
  /** Timestamp of last update (performance.now()). */
  timestamp: number;
  /** MPPI best trajectory as (x, y) world positions. Null when inactive. */
  mppiTrajectory: [number, number][] | null;
  /** Version counter for MPPI trajectory change detection. */
  mppiVersion: number;
}

const globalStore: PathState = {
  state: 0,
  waypoints: [],
  currentWaypoint: 0,
  distanceToGoal: 0,
  version: 0,
  timestamp: 0,
  mppiTrajectory: null,
  mppiVersion: 0,
};

/**
 * Store new path data. Called from WebRTC onmessage handler.
 */
export function setPathData(
  state: number,
  waypoints: DecodedPathWaypoint[],
  currentWaypoint: number,
  distanceToGoal: number,
): void {
  globalStore.state = state;
  globalStore.waypoints = waypoints;
  globalStore.currentWaypoint = currentWaypoint;
  globalStore.distanceToGoal = distanceToGoal;
  globalStore.version++;
  globalStore.timestamp = performance.now();
}

/**
 * Read current path data. Called from useFrame.
 */
export function getPathData(): {
  state: number;
  waypoints: DecodedPathWaypoint[];
  currentWaypoint: number;
  distanceToGoal: number;
} {
  return {
    state: globalStore.state,
    waypoints: globalStore.waypoints,
    currentWaypoint: globalStore.currentWaypoint,
    distanceToGoal: globalStore.distanceToGoal,
  };
}

/**
 * Get version counter for change detection in useFrame.
 */
export function getPathVersion(): number {
  return globalStore.version;
}

/**
 * Store new MPPI trajectory data. Called from WebRTC onmessage handler.
 */
export function setMppiTrajectory(points: [number, number][]): void {
  globalStore.mppiTrajectory = points;
  globalStore.mppiVersion++;
}

/**
 * Read current MPPI trajectory data. Called from useFrame.
 */
export function getMppiTrajectory(): [number, number][] | null {
  return globalStore.mppiTrajectory;
}

/**
 * Get MPPI trajectory version counter for change detection in useFrame.
 */
export function getMppiVersion(): number {
  return globalStore.mppiVersion;
}

/**
 * Clear all path data. Called on disconnect.
 */
export function clearPathData(): void {
  globalStore.state = 0;
  globalStore.waypoints = [];
  globalStore.currentWaypoint = 0;
  globalStore.distanceToGoal = 0;
  globalStore.version++;
  globalStore.mppiTrajectory = null;
  globalStore.mppiVersion++;
  globalStore.timestamp = performance.now();
}
