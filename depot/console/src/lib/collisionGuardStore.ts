/**
 * Mutable collision guard store - bypasses React for performance.
 *
 * The 3D CollisionGuardArc computes obstacle distance each frame and writes
 * it here. The 2D StatusPanel reads it at 4Hz via polling. This avoids
 * coupling the two components through React state.
 *
 * Follows the same pattern as costmapStore.ts.
 */

interface GuardState {
  /** Distance to nearest obstacle along the projected arc (meters) */
  distance: number;
  /** Whether the guard is actively computing (teleop mode + costmap + moving) */
  active: boolean;
  /** Version counter for change detection */
  version: number;
}

const state: GuardState = {
  distance: 0,
  active: false,
  version: 0,
};

export function setGuardState(distance: number, active: boolean): void {
  state.distance = distance;
  state.active = active;
  state.version++;
}

export function getGuardState(): { distance: number; active: boolean } {
  return { distance: state.distance, active: state.active };
}
