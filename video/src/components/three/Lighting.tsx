import { MUNI_ORANGE } from "../../lib/brand";

interface LightingProps {
  intensity?: number;
}

export function Lighting({ intensity = 1 }: LightingProps) {
  return (
    <>
      {/* Ambient fill */}
      <ambientLight intensity={0.4 * intensity} />

      {/* Key light - front right */}
      <directionalLight
        position={[200, 300, 200]}
        intensity={1.0 * intensity}
        castShadow
      />

      {/* Fill light - front left */}
      <directionalLight
        position={[-200, 100, -100]}
        intensity={0.5 * intensity}
      />

      {/* Rim light - back with orange tint */}
      <directionalLight
        position={[0, -100, -200]}
        intensity={0.3 * intensity}
        color={MUNI_ORANGE}
      />

      {/* Top accent */}
      <pointLight position={[0, 500, 0]} intensity={0.2 * intensity} />
    </>
  );
}
