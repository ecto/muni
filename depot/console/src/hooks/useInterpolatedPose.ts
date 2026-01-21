/**
 * Hook for smooth client-side pose interpolation.
 *
 * Uses the snapshot buffer to compute a smooth render pose
 * at 60fps even when telemetry arrives at 50Hz.
 */

import { useFrame } from "@react-three/fiber";
import { useConsoleStore } from "@/store";
import { computeRenderPose } from "@/lib/interpolation";

/** Render delay in ms (helps smooth jitter but adds latency) */
const RENDER_DELAY_MS = 0;

/**
 * Hook that updates the render pose every frame using interpolation.
 *
 * Call this once in a component inside the R3F Canvas.
 * The interpolated pose is stored in the global store.
 */
export function useInterpolatedPose() {
  const interpolation = useConsoleStore((s) => s.interpolation);
  const setRenderPose = useConsoleStore((s) => s.setRenderPose);

  useFrame(() => {
    const result = computeRenderPose(
      interpolation,
      performance.now(),
      RENDER_DELAY_MS
    );
    setRenderPose(result.pose, result.isExtrapolating);
  });
}

/**
 * Hook to get the current render pose.
 * Use this in components that need to read the pose.
 */
export function useRenderPose() {
  return useConsoleStore((s) => ({
    pose: s.renderPose,
    isExtrapolating: s.isExtrapolating,
  }));
}
