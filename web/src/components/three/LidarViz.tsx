"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { mulberry32 } from "./mulberry32";

const ZONE_STOP = new THREE.Color("#ff2244");
const ZONE_SLOW = new THREE.Color("#ffaa22");
const ZONE_CLEAR = new THREE.Color("#22ff66");

// --- Safety zone rings on the ground plane ---

interface SafetyZonesProps {
  pulseSpeed?: number;
}

export function SafetyZones({ pulseSpeed = 0.06 }: SafetyZonesProps) {
  const clockRef = useRef(0);
  const zonesRef = useRef<THREE.Group>(null);

  const zones = useMemo(
    () => [
      { inner: 0, outer: 150, color: ZONE_STOP, baseOpacity: 0.18, label: "STOP" },
      { inner: 150, outer: 350, color: ZONE_SLOW, baseOpacity: 0.12, label: "SLOW" },
      { inner: 350, outer: 600, color: ZONE_CLEAR, baseOpacity: 0.08, label: "CLEAR" },
    ],
    [],
  );

  const boundaries = useMemo(
    () => [
      { r: 150, color: ZONE_STOP },
      { r: 350, color: ZONE_SLOW },
      { r: 600, color: ZONE_CLEAR },
    ],
    [],
  );

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const pulse = 0.85 + 0.15 * Math.sin(clockRef.current * pulseSpeed);
    if (!zonesRef.current) return;
    zonesRef.current.children.forEach((child) => {
      if (child instanceof THREE.Mesh && child.material instanceof THREE.MeshBasicMaterial) {
        const userData = child.userData as { baseOpacity?: number; isBoundary?: boolean };
        if (userData.isBoundary) {
          child.material.opacity = 0.4 * pulse;
        } else if (userData.baseOpacity != null) {
          child.material.opacity = userData.baseOpacity * pulse;
        }
      }
    });
  });

  return (
    <group ref={zonesRef} rotation={[-Math.PI / 2, 0, 0]} position={[0, 2, 0]}>
      {zones.map((z) => (
        <mesh key={z.label} userData={{ baseOpacity: z.baseOpacity }}>
          <ringGeometry args={[z.inner, z.outer, 64]} />
          <meshBasicMaterial
            color={z.color}
            transparent
            opacity={z.baseOpacity}
            side={THREE.DoubleSide}
            depthWrite={false}
          />
        </mesh>
      ))}
      {boundaries.map((b) => (
        <mesh key={b.r} userData={{ isBoundary: true }}>
          <ringGeometry args={[b.r - 1.5, b.r + 1.5, 64]} />
          <meshBasicMaterial
            color={b.color}
            transparent
            opacity={0.4}
            side={THREE.DoubleSide}
            depthWrite={false}
          />
        </mesh>
      ))}
    </group>
  );
}

// --- Spinning LiDAR beam ---

export function LidarBeam({ speed = 0.12 }: { speed?: number }) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);
  const positions = useMemo(() => new Float32Array(6), []);

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const angle = clockRef.current * speed;
    positions[0] = 0;
    positions[1] = 60;
    positions[2] = 0;
    positions[3] = Math.cos(angle) * 700;
    positions[4] = 60;
    positions[5] = Math.sin(angle) * 700;
    if (geoRef.current) {
      geoRef.current.attributes.position.needsUpdate = true;
    }
  });

  return (
    <lineSegments>
      <bufferGeometry ref={geoRef}>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
      </bufferGeometry>
      <lineBasicMaterial color="#22ff66" transparent opacity={0.5} depthWrite={false} />
    </lineSegments>
  );
}

// --- Point cloud painted by sweeping beam ---

interface LidarPointCloudProps {
  maxPoints?: number;
  beamSpeed?: number;
  fadeFrames?: number;
}

export function LidarPointCloud({
  maxPoints = 2000,
  beamSpeed = 0.12,
  fadeFrames = 50,
}: LidarPointCloudProps) {
  const clockRef = useRef(0);
  const geoRef = useRef<THREE.BufferGeometry>(null);

  const obstacles = useMemo(() => {
    const rng = mulberry32(42);
    const obs: Array<{ x: number; z: number; r: number }> = [];
    for (let i = 0; i < 8; i++) {
      const oAngle = rng() * Math.PI * 2;
      const oDist = 200 + rng() * 350;
      obs.push({
        x: Math.cos(oAngle) * oDist,
        z: Math.sin(oAngle) * oDist,
        r: 20 + rng() * 40,
      });
    }
    return obs;
  }, []);

  const buffers = useMemo(
    () => ({
      positions: new Float32Array(maxPoints * 3),
      colors: new Float32Array(maxPoints * 3),
    }),
    [maxPoints],
  );

  useFrame((_, delta) => {
    clockRef.current += delta * 30;
    const frame = Math.floor(clockRef.current);

    let pointIdx = 0;
    const startFrame = Math.max(0, frame - fadeFrames);

    for (let f = startFrame; f <= frame && pointIdx < maxPoints; f++) {
      const angle = f * beamSpeed;
      const pointsPerFrame = 40;
      const subRng = mulberry32(f * 7919);

      for (let p = 0; p < pointsPerFrame && pointIdx < maxPoints; p++) {
        const dist = 80 + subRng() * 550;
        const scatter = (subRng() - 0.5) * 0.04;
        const a = angle + scatter;
        const x = Math.cos(a) * dist;
        const z = Math.sin(a) * dist;
        const y = 5 + subRng() * 15;

        let nearObstacle = false;
        for (const obs of obstacles) {
          const dx = x - obs.x;
          const dz = z - obs.z;
          if (dx * dx + dz * dz < obs.r * obs.r * 4) {
            nearObstacle = true;
            break;
          }
        }
        if (!nearObstacle && subRng() > 0.3) continue;

        const age = (frame - f) / fadeFrames;
        const alpha = 1 - age;

        let color: THREE.Color;
        if (dist < 150) color = ZONE_STOP;
        else if (dist < 350) color = ZONE_SLOW;
        else color = ZONE_CLEAR;

        buffers.positions[pointIdx * 3] = x;
        buffers.positions[pointIdx * 3 + 1] = y;
        buffers.positions[pointIdx * 3 + 2] = z;
        buffers.colors[pointIdx * 3] = color.r * alpha;
        buffers.colors[pointIdx * 3 + 1] = color.g * alpha;
        buffers.colors[pointIdx * 3 + 2] = color.b * alpha;
        pointIdx++;
      }
    }

    if (geoRef.current) {
      geoRef.current.setDrawRange(0, pointIdx);
      geoRef.current.attributes.position.needsUpdate = true;
      geoRef.current.attributes.color.needsUpdate = true;
    }
  });

  return (
    <points>
      <bufferGeometry ref={geoRef}>
        <bufferAttribute attach="attributes-position" args={[buffers.positions, 3]} />
        <bufferAttribute attach="attributes-color" args={[buffers.colors, 3]} />
      </bufferGeometry>
      <pointsMaterial
        size={4}
        transparent
        opacity={0.9}
        sizeAttenuation
        depthWrite={false}
        vertexColors
      />
    </points>
  );
}
