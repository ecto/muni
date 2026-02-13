"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { mulberry32 } from "./mulberry32";

const COLORS = [
  [0.45, 0.35, 0.2],  // brown (dirt/leaves)
  [0.5, 0.5, 0.45],   // gray (paper/trash)
  [0.3, 0.4, 0.2],    // green (leaves)
];

interface DebrisParticlesProps {
  count?: number;
  radius?: number;
  suctionSpeed?: number;
  lifetime?: number;
  size?: number;
  opacity?: number;
}

export function DebrisParticles({
  count = 300,
  radius = 800,
  lifetime = 40,
  size = 6,
  opacity = 0.7,
}: DebrisParticlesProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const seeds = useMemo(() => {
    const rng = mulberry32(314);
    return Array.from({ length: count }, () => {
      const angle = rng() * Math.PI * 2;
      const dist = 200 + rng() * radius;
      return {
        startX: Math.cos(angle) * dist,
        startZ: Math.sin(angle) * dist,
        startY: rng() * 30,
        phase: Math.floor(rng() * lifetime),
        speedMult: 0.6 + rng() * 0.8,
        wobble: rng() * Math.PI * 2,
        wobbleAmp: 10 + rng() * 30,
        colorIdx: Math.floor(rng() * 3),
      };
    });
  }, [count, radius, lifetime]);

  const positions = useMemo(() => new Float32Array(count * 3), [count]);
  const colors = useMemo(() => {
    const c = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const col = COLORS[seeds[i].colorIdx];
      c[i * 3] = col[0];
      c[i * 3 + 1] = col[1];
      c[i * 3 + 2] = col[2];
    }
    return c;
  }, [count, seeds]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % lifetime) / lifetime;
      const ease = t * t;
      const x = s.startX + (0 - s.startX) * ease
        + Math.sin(frame * 0.05 + s.wobble) * s.wobbleAmp * (1 - ease);
      const y = s.startY + (100 - s.startY) * ease * 0.5
        + Math.abs(Math.sin(t * Math.PI)) * 40 * (1 - ease);
      const z = s.startZ + (0 - s.startZ) * ease
        + Math.cos(frame * 0.04 + s.wobble) * s.wobbleAmp * (1 - ease);
      positions[i * 3] = x;
      positions[i * 3 + 1] = y;
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
        <bufferAttribute attach="attributes-color" args={[colors, 3]} />
      </bufferGeometry>
      <pointsMaterial
        size={size}
        transparent
        opacity={opacity}
        sizeAttenuation
        depthWrite={false}
        vertexColors
      />
    </points>
  );
}

export function VacuumVortex({ count = 60, opacity = 0.25 }: { count?: number; opacity?: number }) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const rngSeeds = useMemo(() => {
    const rng = mulberry32(555);
    return Array.from({ length: count }, () => ({
      r1: 150 + rng() * 300,
      rDelta: 40 + rng() * 60,
      yOff: rng() * 80,
    }));
  }, [count]);

  const positions = useMemo(() => new Float32Array(count * 6), [count]);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = clockRef.current;
    for (let i = 0; i < count; i++) {
      const s = rngSeeds[i];
      const baseAngle = (i / count) * Math.PI * 2 + frame * 0.08;
      const r2 = s.r1 - s.rDelta;
      positions[i * 6] = Math.cos(baseAngle) * s.r1;
      positions[i * 6 + 1] = 20 + s.yOff;
      positions[i * 6 + 2] = Math.sin(baseAngle) * s.r1;
      positions[i * 6 + 3] = Math.cos(baseAngle + 0.15) * r2;
      positions[i * 6 + 4] = 40 + s.yOff;
      positions[i * 6 + 5] = Math.sin(baseAngle + 0.15) * r2;
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
      <lineBasicMaterial color="#887766" transparent opacity={opacity} depthWrite={false} />
    </lineSegments>
  );
}
