/**
 * Accumulated point cloud store - builds a persistent world-frame map
 * from LiDAR frames during teleop.
 *
 * Same mutable-store pattern as pointCloudStore.ts / costmapStore.ts:
 * written from WebRTC handler, read imperatively from useFrame.
 *
 * Points are transformed from sensor frame to ENU world frame using the
 * rover pose, then to Three.js coordinates matching the teleop scene
 * convention (threeX = -enuY, threeY = enuZ, threeZ = -enuX).
 *
 * Voxel deduplication (10cm grid) prevents redundant points from
 * overlapping scans as the rover drives back over the same area.
 */

/** Voxel grid resolution in meters. */
const VOXEL_SIZE = 0.10;

/** Maximum accumulated points (~5MB positions + colors). */
const MAX_POINTS = 200_000;

/** Internal store state. */
interface AccumulatedCloudState {
  /** Pre-allocated xyz buffer (MAX_POINTS * 3). Three.js coords. */
  positions: Float32Array;
  /** Pre-allocated rgb buffer (MAX_POINTS * 3). Height-based coloring. */
  colors: Float32Array;
  /** Current point count (write head). */
  count: number;
  /** Version counter for change detection in useFrame. */
  version: number;
  /** Spatial hash → index for voxel deduplication. */
  voxelMap: Map<number, number>;
}

const store: AccumulatedCloudState = {
  positions: new Float32Array(MAX_POINTS * 3),
  colors: new Float32Array(MAX_POINTS * 3),
  count: 0,
  version: 0,
  voxelMap: new Map(),
};

/**
 * Spatial hash for a voxel coordinate. Uses a simple Cantor-like hash
 * to combine 3 integers into a single number for Map lookup.
 */
function voxelHash(vx: number, vy: number, vz: number): number {
  // Offset to handle negatives (shift by large number)
  const ox = vx + 500000;
  const oy = vy + 500000;
  const oz = vz + 500000;
  // Combine with prime multipliers — collisions are rare at 10cm resolution
  return (ox * 73856093) ^ (oy * 19349663) ^ (oz * 83492791);
}

/**
 * Height-based coloring: blue (ground) -> cyan -> green -> yellow -> red (high).
 * Same gradient as PointCloud3D.tsx.
 */
function heightColor(y: number, out: Float32Array, offset: number) {
  const t = Math.max(0, Math.min(1, (y + 0.2) / 2.0));

  if (t < 0.25) {
    const s = t / 0.25;
    out[offset] = 0.0;
    out[offset + 1] = s;
    out[offset + 2] = 1.0;
  } else if (t < 0.5) {
    const s = (t - 0.25) / 0.25;
    out[offset] = 0.0;
    out[offset + 1] = 1.0;
    out[offset + 2] = 1.0 - s;
  } else if (t < 0.75) {
    const s = (t - 0.5) / 0.25;
    out[offset] = s;
    out[offset + 1] = 1.0;
    out[offset + 2] = 0.0;
  } else {
    const s = (t - 0.75) / 0.25;
    out[offset] = 1.0;
    out[offset + 1] = 1.0 - s;
    out[offset + 2] = 0.0;
  }
}

/**
 * Ingest a LiDAR frame into the accumulated cloud.
 *
 * @param sensorPoints - Float32Array of sensor-frame points (x=forward, y=left, z=up), length N*3
 * @param pose - Rover pose in ENU frame { x: East, y: North, theta: heading from East CCW+ }
 */
export function ingestFrame(
  sensorPoints: Float32Array,
  pose: { x: number; y: number; theta: number },
): void {
  const cosT = Math.cos(pose.theta);
  const sinT = Math.sin(pose.theta);
  const incoming = sensorPoints.length / 3;

  for (let i = 0; i < incoming; i++) {
    const sx = sensorPoints[i * 3];
    const sy = sensorPoints[i * 3 + 1];
    const sz = sensorPoints[i * 3 + 2];

    // Sensor -> ENU world
    const enuX = pose.x + sx * cosT - sy * sinT;
    const enuY = pose.y + sx * sinT + sy * cosT;
    const enuZ = sz;

    // Voxel dedup (ENU coordinates, 10cm grid)
    const vx = Math.floor(enuX / VOXEL_SIZE);
    const vy = Math.floor(enuY / VOXEL_SIZE);
    const vz = Math.floor(enuZ / VOXEL_SIZE);
    const hash = voxelHash(vx, vy, vz);

    if (store.voxelMap.has(hash)) continue;

    // Capacity check — stop accumulating when full
    if (store.count >= MAX_POINTS) continue;

    const idx = store.count;
    store.voxelMap.set(hash, idx);

    // ENU -> teleop Three.js (matches RoverModel/PointCloud3D convention)
    const threeX = -enuY;
    const threeY = enuZ;
    const threeZ = -enuX;

    store.positions[idx * 3] = threeX;
    store.positions[idx * 3 + 1] = threeY;
    store.positions[idx * 3 + 2] = threeZ;

    // Height-based color (threeY = height)
    heightColor(threeY, store.colors, idx * 3);

    store.count++;
  }

  store.version++;
}

/**
 * Get accumulated cloud data for GPU upload.
 */
export function getAccumulatedCloud(): {
  positions: Float32Array;
  colors: Float32Array;
  count: number;
} {
  return {
    positions: store.positions,
    colors: store.colors,
    count: store.count,
  };
}

/**
 * Version counter for change detection in useFrame.
 */
export function getAccumulatedCloudVersion(): number {
  return store.version;
}

/**
 * Clear all accumulated data. Called on disconnect or user action.
 */
export function clearAccumulatedCloud(): void {
  store.count = 0;
  store.version++;
  store.voxelMap.clear();
}
