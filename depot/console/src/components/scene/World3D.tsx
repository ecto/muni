/**
 * 3D world component that loads Gaussian splats from map-api.
 *
 * Auto-loads the appropriate map based on GPS position.
 * Renders the environment around the rover for teleop context.
 */

import { Suspense } from "react";
import { Splat } from "@react-three/drei";
import { useMap3D } from "@/hooks/useMap3D";


/**
 * Loading indicator while splat loads.
 */
function LoadingRing() {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]}>
      <ringGeometry args={[0.8, 1.0, 32]} />
      <meshBasicMaterial color="#3b82f6" transparent opacity={0.5} />
    </mesh>
  );
}

/**
 * Component that renders the actual splat.
 */
function SplatScene({ splatUrl }: { splatUrl: string }) {
  // drei's Splat component handles loading and rendering
  return (
    <Splat
      src={splatUrl}
      // Position the splat at the map origin
      // The coordinate transform positions the rover relative to this
      position={[0, 0, 0]}
    />
  );
}

/**
 * 3D world container that manages map loading and rendering.
 */
export function World3D() {
  const { manifest, splatUrl, loading, error } = useMap3D();

  // Don't render anything if no map is loaded
  if (!manifest || !splatUrl) {
    return null;
  }

  // Show error state (could add visual indicator)
  if (error) {
    console.warn("[World3D] Map error:", error);
    return null;
  }

  // Loading state is handled by Suspense fallback
  if (loading && !splatUrl) {
    return <LoadingRing />;
  }

  return (
    <group>
      <Suspense fallback={<LoadingRing />}>
        <SplatScene splatUrl={splatUrl} />
      </Suspense>
    </group>
  );
}
