import { useRef, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import { Line } from "@react-three/drei";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

const MAX_POINTS = 500;
const MIN_DISTANCE = 0.05; // Minimum distance between points (meters)

/**
 * Trail visualization showing the rover's path.
 * Renders as a fading line behind the rover.
 */
export function RoverTrail() {
  const { renderPose, connected } = useConsoleStore();
  const pointsRef = useRef<THREE.Vector3[]>([]);
  const lastPosRef = useRef<THREE.Vector3 | null>(null);

  // Clear trail on disconnect
  useEffect(() => {
    if (!connected) {
      pointsRef.current = [];
      lastPosRef.current = null;
    }
  }, [connected]);

  // Add points as rover moves
  useFrame(() => {
    const pos = new THREE.Vector3(renderPose.x, 0.02, -renderPose.y);
    const lastPos = lastPosRef.current;

    // Only add point if moved far enough
    if (!lastPos || pos.distanceTo(lastPos) >= MIN_DISTANCE) {
      pointsRef.current.push(pos.clone());
      lastPosRef.current = pos.clone();

      // Trim to max points
      if (pointsRef.current.length > MAX_POINTS) {
        pointsRef.current.shift();
      }
    }
  });

  const points = pointsRef.current;

  if (points.length < 2) {
    return null;
  }

  // Create gradient colors for fading effect
  const colors = points.map((_, i) => {
    const alpha = i / points.length;
    // Fade from transparent to cyan
    return new THREE.Color().setHSL(0.5, 0.8, 0.3 + alpha * 0.3);
  });

  return (
    <Line
      points={points}
      vertexColors={colors}
      lineWidth={2}
      transparent
      opacity={0.8}
    />
  );
}
