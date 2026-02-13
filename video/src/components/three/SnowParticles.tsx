// Deterministic snow particle system for Remotion (frame-based, no delta time)
import { useMemo } from "react";
import { useCurrentFrame } from "remotion";
import * as THREE from "three";

// Seeded PRNG (mulberry32) for deterministic particles across renders
function mulberry32(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

interface SnowParticlesProps {
  count?: number;
  area?: [number, number, number]; // width, height, depth
  speed?: number; // fall speed per frame
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
  const frame = useCurrentFrame();

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

  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const rawY = s.y - frame * speed * s.speedMult;
      const y = ((rawY % area[1]) + area[1]) % area[1] - area[1] / 2;
      positions[i * 3] = s.x + Math.sin(frame * 0.025 + s.driftPhase) * s.driftAmp;
      positions[i * 3 + 1] = y;
      positions[i * 3 + 2] = s.z + Math.cos(frame * 0.018 + s.driftPhase) * s.driftAmp * 0.7;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [frame, count, seeds, speed, area]);

  return (
    <points geometry={geometry}>
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

// Spray particles ejected from a point (plow throwing snow forward)
interface SprayProps {
  count?: number;
  origin?: [number, number, number]; // spray origin
  spread?: number; // cone angle
  velocity?: number; // initial speed
  gravity?: number;
  lifetime?: number; // frames before respawn
  size?: number;
  color?: string;
  opacity?: number;
}

export function SnowSpray({
  count = 200,
  origin = [0, 100, -300], // front of rover
  spread = 1.2,
  velocity = 15,
  gravity = 0.4,
  lifetime = 30,
  size = 5,
  color = "#e8f0ff",
  opacity = 0.6,
}: SprayProps) {
  const frame = useCurrentFrame();

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

  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % lifetime) / lifetime; // 0→1 within lifetime
      const v = velocity * s.speedMult;
      const x = origin[0] + Math.sin(s.angle) * v * t * lifetime + s.drift * t * 100;
      const y = origin[1] + Math.sin(s.upAngle) * v * t * lifetime - gravity * (t * lifetime) * (t * lifetime) * 0.5;
      const z = origin[2] - Math.cos(s.angle) * v * t * lifetime;

      // Fade out near end of lifetime
      positions[i * 3] = x;
      positions[i * 3 + 1] = Math.max(0, y); // don't go below ground
      positions[i * 3 + 2] = z;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [frame, count, seeds, origin, velocity, gravity, lifetime]);

  return (
    <points geometry={geometry}>
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
