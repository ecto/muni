/**
 * Mutable costmap store - bypasses React for performance.
 *
 * Costmap data arrives at ~2Hz from the rover. Storing it in React state
 * would trigger unnecessary re-renders. The 3D overlay reads this
 * imperatively from useFrame.
 *
 * Follows the same pattern as pointCloudStore.ts.
 */

interface CostmapState {
  cells: Uint8Array | null;
  width: number;
  height: number;
  resolution: number;
  originX: number;
  originY: number;
  /** Version counter - incremented on every update for change detection */
  version: number;
  /** Timestamp of last update (performance.now()) */
  timestamp: number;
}

const globalStore: CostmapState = {
  cells: null,
  width: 0,
  height: 0,
  resolution: 0,
  originX: 0,
  originY: 0,
  version: 0,
  timestamp: 0,
};

/**
 * Store new costmap data. Called from WebRTC onmessage handler.
 */
export function setCostmapData(
  cells: Uint8Array,
  width: number,
  height: number,
  resolution: number,
  originX: number,
  originY: number,
): void {
  globalStore.cells = cells;
  globalStore.width = width;
  globalStore.height = height;
  globalStore.resolution = resolution;
  globalStore.originX = originX;
  globalStore.originY = originY;
  globalStore.version++;
  globalStore.timestamp = performance.now();
}

/**
 * Read current costmap data. Called from useFrame.
 */
export function getCostmapData(): {
  cells: Uint8Array | null;
  width: number;
  height: number;
  resolution: number;
  originX: number;
  originY: number;
} {
  return {
    cells: globalStore.cells,
    width: globalStore.width,
    height: globalStore.height,
    resolution: globalStore.resolution,
    originX: globalStore.originX,
    originY: globalStore.originY,
  };
}

/**
 * Get version counter for change detection in useFrame.
 */
export function getCostmapVersion(): number {
  return globalStore.version;
}

/**
 * Clear all costmap data. Called on disconnect.
 */
export function clearCostmapData(): void {
  globalStore.cells = null;
  globalStore.width = 0;
  globalStore.height = 0;
  globalStore.resolution = 0;
  globalStore.originX = 0;
  globalStore.originY = 0;
  globalStore.version++;
  globalStore.timestamp = performance.now();
}
