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
      alphaHash
    />
  );
}

/**
 * 3D world container that manages map loading and rendering.
 */
export function World3D() {
  const { splatUrl, loading } = useMap3D();

  // Don't render anything if no splat URL available or still loading
  if (!splatUrl || (loading && !splatUrl)) {
    return null;
  }

  return (
    <group>
      <Suspense fallback={<LoadingRing />}>
        <SplatScene splatUrl={splatUrl} />
      </Suspense>
    </group>
  );
}
