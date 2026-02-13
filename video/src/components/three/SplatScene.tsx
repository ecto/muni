// Gaussian splat renderer for hype reel transitions
// Uses @react-three/drei <Splat> component (same as depot console)
import { useCurrentFrame } from "remotion";
import { useThree } from "@react-three/fiber";
import { useEffect, Suspense } from "react";
import { interpolate } from "remotion";
import * as THREE from "three";

interface SplatSceneProps {
  /** Path to .splat file (relative to public/) */
  splatSrc?: string;
  /** Total frames this scene lasts */
  durationInFrames: number;
  /** Camera orbit speed (radians per full duration) */
  orbitRange?: number;
  /** Start azimuth */
  startAzimuth?: number;
}

/**
 * Placeholder splat scene — renders a particle cloud effect
 * when no .splat file is available. When a real .splat file
 * is provided, swap in drei's <Splat> component.
 */
export function SplatScene({
  splatSrc,
  durationInFrames,
  orbitRange = Math.PI * 0.5,
  startAzimuth = 0,
}: SplatSceneProps) {
  const frame = useCurrentFrame();
  const { camera } = useThree();

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [startAzimuth, startAzimuth + orbitRange],
    { extrapolateRight: "clamp" }
  );

  const distance = interpolate(frame, [0, durationInFrames], [8, 4], {
    extrapolateRight: "clamp",
  });

  const elevation = Math.PI / 8;

  useEffect(() => {
    const x = distance * Math.sin(azimuth) * Math.cos(elevation);
    const y = distance * Math.sin(elevation) + 1;
    const z = distance * Math.cos(azimuth) * Math.cos(elevation);
    camera.position.set(x, y, z);
    camera.lookAt(0, 0.5, 0);
  }, [camera, azimuth, distance, elevation]);

  // TODO: When .splat files are added, uncomment:
  // import { Splat } from "@react-three/drei";
  // return <Splat src={staticFile(splatSrc)} />;

  // Placeholder: particle cloud effect
  const particleCount = 2000;
  const positions = new Float32Array(particleCount * 3);
  const colors = new Float32Array(particleCount * 3);

  for (let i = 0; i < particleCount; i++) {
    const theta = Math.random() * Math.PI * 2;
    const phi = Math.acos(2 * Math.random() - 1);
    const r = 2 + Math.random() * 3;
    positions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
    positions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta) * 0.5 + 0.5;
    positions[i * 3 + 2] = r * Math.cos(phi);

    // Teal-orange color palette
    const t = Math.random();
    colors[i * 3] = t > 0.5 ? 1.0 : 0.0; // R
    colors[i * 3 + 1] = t > 0.5 ? 0.4 : 0.7; // G
    colors[i * 3 + 2] = t > 0.5 ? 0.0 : 0.7; // B
  }

  // Dissolve effect — particles scatter over time
  const dissolve = interpolate(frame, [0, durationInFrames * 0.8, durationInFrames], [0, 0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <>
      <ambientLight intensity={0.3} />
      <pointLight position={[5, 5, 5]} intensity={1} color="#ff6600" />
      <points>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={particleCount}
            array={positions}
            itemSize={3}
          />
          <bufferAttribute
            attach="attributes-color"
            count={particleCount}
            array={colors}
            itemSize={3}
          />
        </bufferGeometry>
        <pointsMaterial
          size={0.05 + dissolve * 0.1}
          vertexColors
          transparent
          opacity={1 - dissolve * 0.8}
          sizeAttenuation
        />
      </points>
    </>
  );
}
