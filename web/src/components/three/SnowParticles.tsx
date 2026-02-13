"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { mulberry32 } from "./mulberry32";

interface SnowParticlesProps {
  count?: number;
  area?: [number, number, number];
  speed?: number;
  size?: number;
  opacity?: number;
  color?: string;
}

export function SnowParticles({
  count = 400,
  area = [2000, 1200, 2000],
  speed = 2.5,
  size = 4,
  opacity = 0.7,
  color = "#ffffff",
}: SnowParticlesProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const seeds = useMemo(() => {
    const rng = mulberry32(42);
    return Array.from({ length: count }, () => ({
      x: (rng() - 0.5) * area[0],
      y: rng() * area[1],
      z: (rng() - 0.5) * area[2],
      speedMult: 0.6 + rng() * 0.8,
      driftPhase: rng() * Math.PI * 2,
      driftAmp: 5 + rng() * 15,
    }));
  }, [count, area]);

  const positions = useMemo(() => new Float32Array(count * 3), [count]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const rawY = s.y - frame * speed * s.speedMult;
      const y = ((rawY % area[1]) + area[1]) % area[1] - area[1] / 2;
      positions[i * 3] = s.x + Math.sin(frame * 0.025 + s.driftPhase) * s.driftAmp;
      positions[i * 3 + 1] = y;
      positions[i * 3 + 2] = s.z + Math.cos(frame * 0.018 + s.driftPhase) * s.driftAmp * 0.7;
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

interface SprayProps {
  count?: number;
  origin?: [number, number, number];
  spread?: number;
  velocity?: number;
  gravity?: number;
  lifetime?: number;
  size?: number;
  color?: string;
  opacity?: number;
}

export function SnowSpray({
  count = 200,
  origin = [0, 100, -300],
  spread = 1.2,
  velocity = 15,
  gravity = 0.4,
  lifetime = 30,
  size = 5,
  color = "#e8f0ff",
  opacity = 0.6,
}: SprayProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const seeds = useMemo(() => {
    const rng = mulberry32(137);
    return Array.from({ length: count }, () => ({
      angle: (rng() - 0.5) * spread,
      upAngle: rng() * 0.5 + 0.2,
      speedMult: 0.5 + rng() * 1.0,
      phase: Math.floor(rng() * lifetime),
      drift: (rng() - 0.5) * 0.3,
    }));
  }, [count, spread, lifetime]);

  const positions = useMemo(() => new Float32Array(count * 3), [count]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % lifetime) / lifetime;
      const v = velocity * s.speedMult;
      const x = origin[0] + Math.sin(s.angle) * v * t * lifetime + s.drift * t * 100;
      const y = origin[1] + Math.sin(s.upAngle) * v * t * lifetime - gravity * (t * lifetime) * (t * lifetime) * 0.5;
      const z = origin[2] - Math.cos(s.angle) * v * t * lifetime;
      positions[i * 3] = x;
      positions[i * 3 + 1] = Math.max(0, y);
      positions[i * 3 + 2] = z;
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
        blending={THREE.AdditiveBlending}
      />
    </points>
  );
}
