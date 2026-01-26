/**
 * Mutable point cloud store - bypasses React for performance.
 *
 * Point cloud data arrives at ~5Hz from the LiDAR sensor. Storing it in
 * React state triggers re-renders + an expensive useEffect that transforms
 * 1000+ points. The actual 3D rendering happens in useFrame (no React needed).
 *
 * This module follows the same pattern as videoFrameStore.ts: a plain JS
 * object updated from the WebRTC handler, read imperatively from useFrame.
 */

/** Internal store state. */
interface PointCloudState {
  points: Float32Array | null;
  reflectivity: Uint8Array | null;
  tag: Uint8Array | null;
  /** Version counter - incremented on every update for change detection */
  version: number;
  /** Timestamp of last update (performance.now()) */
  timestamp: number;
}

const globalStore: PointCloudState = {
  points: null,
  reflectivity: null,
  tag: null,
  version: 0,
  timestamp: 0,
};

/**
 * Store new point cloud data. Called from WebRTC onmessage handler.
 */
export function setPointCloudData(
  points: Float32Array,
  reflectivity: Uint8Array,
  tag: Uint8Array
): void {
  globalStore.points = points;
  globalStore.reflectivity = reflectivity;
  globalStore.tag = tag;
  globalStore.version++;
  globalStore.timestamp = performance.now();
}

/**
 * Read current point cloud data. Called from useFrame.
 */
export function getPointCloudData(): {
  points: Float32Array | null;
  reflectivity: Uint8Array | null;
  tag: Uint8Array | null;
} {
  return {
    points: globalStore.points,
    reflectivity: globalStore.reflectivity,
    tag: globalStore.tag,
  };
}

/**
 * Get version counter for change detection in useFrame.
 */
export function getPointCloudVersion(): number {
  return globalStore.version;
}

/**
 * Clear all point cloud data. Called on disconnect.
 */
export function clearPointCloudData(): void {
  globalStore.points = null;
  globalStore.reflectivity = null;
  globalStore.tag = null;
  globalStore.version++;
  globalStore.timestamp = performance.now();
}
