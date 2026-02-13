// Animated LiDAR sweep with safety zone rings
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

// Colors for safety zones
const ZONE_STOP = new THREE.Color("#ff2244");
const ZONE_SLOW = new THREE.Color("#ffaa22");
const ZONE_CLEAR = new THREE.Color("#22ff66");

// --- Safety zone rings on the ground plane ---

interface SafetyZonesProps {
  pulseSpeed?: number;
}

export function SafetyZones({ pulseSpeed = 0.06 }: SafetyZonesProps) {
  const frame = useCurrentFrame();

  const zones = useMemo(
    () => [
      { inner: 0, outer: 150, color: ZONE_STOP, baseOpacity: 0.18, label: "STOP" },
      { inner: 150, outer: 350, color: ZONE_SLOW, baseOpacity: 0.12, label: "SLOW" },
      { inner: 350, outer: 600, color: ZONE_CLEAR, baseOpacity: 0.08, label: "CLEAR" },
    ],
    [],
  );

  const pulse = 0.85 + 0.15 * Math.sin(frame * pulseSpeed);

  return (
    <group rotation={[-Math.PI / 2, 0, 0]} position={[0, 2, 0]}>
      {zones.map((z) => (
        <mesh key={z.label}>
          <ringGeometry args={[z.inner, z.outer, 64]} />
          <meshBasicMaterial
            color={z.color}
            transparent
            opacity={z.baseOpacity * pulse}
            side={THREE.DoubleSide}
            depthWrite={false}
          />
        </mesh>
      ))}
      {/* Zone boundary rings */}
      {[150, 350, 600].map((r, i) => (
        <mesh key={r}>
          <ringGeometry args={[r - 1.5, r + 1.5, 64]} />
          <meshBasicMaterial
            color={[ZONE_STOP, ZONE_SLOW, ZONE_CLEAR][i]}
            transparent
            opacity={0.4 * pulse}
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
  const frame = useCurrentFrame();
  const angle = frame * speed;

  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array([
      0, 60, 0,
      Math.cos(angle) * 700, 60, Math.sin(angle) * 700,
    ]);
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    return geo;
  }, [angle]);

  return (
    <lineSegments geometry={geometry}>
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
  const frame = useCurrentFrame();

  const geometry = useMemo(() => {
    const rng = mulberry32(42);
    const positions = new Float32Array(maxPoints * 3);
    const colors = new Float32Array(maxPoints * 3);

    // Generate obstacle layout (deterministic)
    const obstacles: Array<{ x: number; z: number; r: number }> = [];
    for (let i = 0; i < 8; i++) {
      const oAngle = rng() * Math.PI * 2;
      const oDist = 200 + rng() * 350;
      obstacles.push({
        x: Math.cos(oAngle) * oDist,
        z: Math.sin(oAngle) * oDist,
        r: 20 + rng() * 40,
      });
    }

    let pointIdx = 0;
    // Each frame the beam sweeps — paint points for recent frames
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

        // Check if near obstacle — cluster more points there
        let nearObstacle = false;
        for (const obs of obstacles) {
          const dx = x - obs.x;
          const dz = z - obs.z;
          if (dx * dx + dz * dz < obs.r * obs.r * 4) {
            nearObstacle = true;
            break;
          }
        }

        if (!nearObstacle && subRng() > 0.3) continue; // sparse in open areas

        const age = (frame - f) / fadeFrames;
        const alpha = 1 - age;

        // Color by distance from rover
        let color: THREE.Color;
        if (dist < 150) {
          color = ZONE_STOP;
        } else if (dist < 350) {
          color = ZONE_SLOW;
        } else {
          color = ZONE_CLEAR;
        }

        positions[pointIdx * 3] = x;
        positions[pointIdx * 3 + 1] = y;
        positions[pointIdx * 3 + 2] = z;
        colors[pointIdx * 3] = color.r * alpha;
        colors[pointIdx * 3 + 1] = color.g * alpha;
        colors[pointIdx * 3 + 2] = color.b * alpha;
        pointIdx++;
      }
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions.slice(0, pointIdx * 3), 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(colors.slice(0, pointIdx * 3), 3));
    return geo;
  }, [frame, maxPoints, beamSpeed, fadeFrames]);

  return (
    <points geometry={geometry}>
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
