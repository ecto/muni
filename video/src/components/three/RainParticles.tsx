// Deterministic rain particle system — fast vertical streaks
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

interface RainParticlesProps {
  count?: number;
  area?: [number, number, number];
  speed?: number;
  size?: number;
  opacity?: number;
  color?: string;
  angle?: number; // wind angle in radians (0 = straight down)
}

export function RainParticles({
  count = 600,
  area = [2500, 1500, 2500],
  speed = 12,
  size = 2,
  opacity = 0.4,
  color = "#8899bb",
  angle = 0.15,
}: RainParticlesProps) {
  const frame = useCurrentFrame();

  const seeds = useMemo(() => {
    const rng = mulberry32(99);
    return Array.from({ length: count }, () => ({
      x: (rng() - 0.5) * area[0],
      y: rng() * area[1],
      z: (rng() - 0.5) * area[2],
      speedMult: 0.7 + rng() * 0.6,
    }));
  }, [count, area]);

  // Rain uses line segments (start + end per drop) for streak effect
  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 6); // 2 vertices per line
    const streakLen = speed * 3; // streak length in units

    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const rawY = s.y - frame * speed * s.speedMult;
      const y = ((rawY % area[1]) + area[1]) % area[1] - area[1] / 2;
      const x = s.x + Math.sin(angle) * frame * speed * s.speedMult * 0.3;
      const z = s.z;

      // Line start
      positions[i * 6] = x;
      positions[i * 6 + 1] = y;
      positions[i * 6 + 2] = z;
      // Line end (streak upward from current position)
      positions[i * 6 + 3] = x - Math.sin(angle) * streakLen * 0.3;
      positions[i * 6 + 4] = y + streakLen;
      positions[i * 6 + 5] = z;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [frame, count, seeds, speed, area, angle]);

  return (
    <lineSegments geometry={geometry}>
      <lineBasicMaterial
        color={color}
        transparent
        opacity={opacity}
        depthWrite={false}
      />
    </lineSegments>
  );
}

// Splash particles at ground level
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
  const frame = useCurrentFrame();

  const seeds = useMemo(() => {
    const rng = mulberry32(222);
    return Array.from({ length: count }, () => ({
      x: (rng() - 0.5) * area[0],
      z: (rng() - 0.5) * area[1],
      phase: Math.floor(rng() * 8),
      maxHeight: 5 + rng() * 15,
    }));
  }, [count, area]);

  const geometry = useMemo(() => {
    const positions = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const s = seeds[i];
      const t = ((frame + s.phase) % 8) / 8;
      const y = s.maxHeight * Math.sin(t * Math.PI); // arc up and down
      positions[i * 3] = s.x;
      positions[i * 3 + 1] = y;
      positions[i * 3 + 2] = s.z;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [frame, count, seeds]);

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
