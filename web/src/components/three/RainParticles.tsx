"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { mulberry32 } from "./mulberry32";

interface RainParticlesProps {
  count?: number;
  area?: [number, number, number];
  speed?: number;
  opacity?: number;
  color?: string;
  angle?: number;
}

export function RainParticles({
  count = 600,
  area = [2500, 1500, 2500],
  speed = 12,
  opacity = 0.4,
  color = "#8899bb",
  angle = 0.15,
}: RainParticlesProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const seeds = useMemo(() => {
    const rng = mulberry32(99);
    return Array.from({ length: count }, () => ({
      x: (rng() - 0.5) * area[0],
      y: rng() * area[1],
      z: (rng() - 0.5) * area[2],
      speedMult: 0.7 + rng() * 0.6,
    }));
  }, [count, area]);

  const positions = useMemo(() => new Float32Array(count * 6), [count]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    const streakLen = speed * 3;

    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const rawY = s.y - frame * speed * s.speedMult;
      const y = ((rawY % area[1]) + area[1]) % area[1] - area[1] / 2;
      const x = s.x + Math.sin(angle) * frame * speed * s.speedMult * 0.3;
      const z = s.z;

      positions[i * 6] = x;
      positions[i * 6 + 1] = y;
      positions[i * 6 + 2] = z;
      positions[i * 6 + 3] = x - Math.sin(angle) * streakLen * 0.3;
      positions[i * 6 + 4] = y + streakLen;
      positions[i * 6 + 5] = z;
    }
    if (geoRef.current) {
      geoRef.current.attributes.position.needsUpdate = true;
    }
  });

  return (
    <lineSegments>
      <bufferGeometry ref={geoRef}>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
      </bufferGeometry>
      <lineBasicMaterial color={color} transparent opacity={opacity} depthWrite={false} />
    </lineSegments>
  );
}

interface SplashProps {
  count?: number;
  area?: [number, number];
  size?: number;
  color?: string;
  opacity?: number;
}

export function RainSplash({
  count = 100,
  area = [1500, 1500],
  size = 3,
  color = "#aabbcc",
  opacity = 0.3,
}: SplashProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const seeds = useMemo(() => {
    const rng = mulberry32(222);
    return Array.from({ length: count }, () => ({
      x: (rng() - 0.5) * area[0],
      z: (rng() - 0.5) * area[1],
      phase: Math.floor(rng() * 8),
      maxHeight: 5 + rng() * 15,
    }));
  }, [count, area]);

  const positions = useMemo(() => new Float32Array(count * 3), [count]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % 8) / 8;
      positions[i * 3] = s.x;
      positions[i * 3 + 1] = s.maxHeight * Math.sin(t * Math.PI);
      positions[i * 3 + 2] = s.z;
    }
    if (geoRef.current) {
      geoRef.current.attributes.position.needsUpdate = true;
    }
  });

  return (
    <points>
      <bufferGeometry ref={geoRef}>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
      </bufferGeometry>
      <pointsMaterial
        color={color}
        size={size}
        transparent
        opacity={opacity}
        sizeAttenuation
        depthWrite={false}
      />
    </points>
  );
}
