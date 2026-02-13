// Debris particles being vacuumed toward the rover
import { useMemo } from "react";
import { useCurrentFrame } from "remotion";
import * as THREE from "three";

function mulberry32(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

interface DebrisParticlesProps {
  count?: number;
  radius?: number; // spawn radius around rover
  suctionSpeed?: number;
  lifetime?: number;
  size?: number;
  opacity?: number;
}

export function DebrisParticles({
  count = 300,
  radius = 800,
  suctionSpeed = 8,
  lifetime = 40,
  size = 6,
  opacity = 0.7,
}: DebrisParticlesProps) {
  const frame = useCurrentFrame();

  const seeds = useMemo(() => {
    const rng = mulberry32(314);
    return Array.from({ length: count }, () => {
      const angle = rng() * Math.PI * 2;
      const dist = 200 + rng() * radius;
      return {
        startX: Math.cos(angle) * dist,
        startZ: Math.sin(angle) * dist,
        startY: rng() * 30, // ground level with slight variation
        phase: Math.floor(rng() * lifetime),
        speedMult: 0.6 + rng() * 0.8,
        wobble: rng() * Math.PI * 2,
        wobbleAmp: 10 + rng() * 30,
        // Color variation: browns, grays, greens (leaf/trash tones)
        colorIdx: Math.floor(rng() * 3),
      };
    });
  }, [count, radius, lifetime]);

  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 3);
    const colors = new Float32Array(count * 3);
    const COLORS = [
      [0.45, 0.35, 0.2],  // brown (dirt/leaves)
      [0.5, 0.5, 0.45],   // gray (paper/trash)
      [0.3, 0.4, 0.2],    // green (leaves)
    ];

    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % lifetime) / lifetime; // 0→1 within cycle
      const ease = t * t; // accelerate as it gets closer (vacuum effect)

      // Lerp from start position toward rover center (0, 100, 0)
      const targetX = 0;
      const targetY = 100;
      const targetZ = 0;

      const x = s.startX + (targetX - s.startX) * ease
        + Math.sin(frame * 0.05 + s.wobble) * s.wobbleAmp * (1 - ease);
      const y = s.startY + (targetY - s.startY) * ease * 0.5
        + Math.abs(Math.sin(t * Math.PI)) * 40 * (1 - ease); // arc upward then down
      const z = s.startZ + (targetZ - s.startZ) * ease
        + Math.cos(frame * 0.04 + s.wobble) * s.wobbleAmp * (1 - ease);

      positions[i * 3] = x;
      positions[i * 3 + 1] = y;
      positions[i * 3 + 2] = z;

      const c = COLORS[s.colorIdx];
      colors[i * 3] = c[0];
      colors[i * 3 + 1] = c[1];
      colors[i * 3 + 2] = c[2];
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
    return geo;
  }, [frame, count, seeds, lifetime]);

  return (
    <points geometry={geometry}>
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

// Swirl vortex lines around the vacuum intake
export function VacuumVortex({ count = 60, opacity = 0.25 }: { count?: number; opacity?: number }) {
  const frame = useCurrentFrame();

  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 6); // line segments
    const rng = mulberry32(555);

    for (let i = 0; i < count; i++) {
      const baseAngle = (i / count) * Math.PI * 2 + frame * 0.08;
      const r1 = 150 + rng() * 300;
      const r2 = r1 - 40 - rng() * 60;
      const yOff = rng() * 80;

      positions[i * 6] = Math.cos(baseAngle) * r1;
      positions[i * 6 + 1] = 20 + yOff;
      positions[i * 6 + 2] = Math.sin(baseAngle) * r1;

      positions[i * 6 + 3] = Math.cos(baseAngle + 0.15) * r2;
      positions[i * 6 + 4] = 40 + yOff;
      positions[i * 6 + 5] = Math.sin(baseAngle + 0.15) * r2;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [frame, count]);

  return (
    <lineSegments geometry={geometry}>
      <lineBasicMaterial color="#88776655" transparent opacity={opacity} depthWrite={false} />
    </lineSegments>
  );
}
