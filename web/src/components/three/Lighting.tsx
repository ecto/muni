"use client";

import { MUNI_ORANGE } from "@/lib/brand";

interface LightingProps {
  intensity?: number;
  tint?: string;
}

export function Lighting({ intensity = 1, tint }: LightingProps) {
  return (
    <>
      <ambientLight intensity={0.4 * intensity} />
      <directionalLight position={[200, 300, 200]} intensity={1.0 * intensity} castShadow />
      <directionalLight position={[-200, 100, -100]} intensity={0.5 * intensity} />
      <directionalLight
        position={[0, -100, -200]}
        intensity={0.3 * intensity}
        color={tint ?? MUNI_ORANGE}
      />
      <pointLight position={[0, 500, 0]} intensity={0.2 * intensity} />
    </>
  );
}
