import { useRef, useEffect, useMemo, useState } from "react";
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
  // Reusable Vector3 to avoid allocations in useFrame
  const tempVec = useRef(new THREE.Vector3());
  // Track point count changes to trigger re-render for colors
  const [pointCount, setPointCount] = useState(0);

  // Clear trail on disconnect
  useEffect(() => {
    if (!connected) {
      pointsRef.current = [];
      lastPosRef.current = null;
      setPointCount(0);
    }
  }, [connected]);

  // Add points as rover moves
  useFrame(() => {
    tempVec.current.set(renderPose.x, 0.02, -renderPose.y);
    const lastPos = lastPosRef.current;

    // Only add point if moved far enough
    if (!lastPos || tempVec.current.distanceTo(lastPos) >= MIN_DISTANCE) {
      pointsRef.current.push(tempVec.current.clone());
      lastPosRef.current = tempVec.current.clone();

      // Trim to max points
      if (pointsRef.current.length > MAX_POINTS) {
        pointsRef.current.shift();
      }

      // Trigger re-render only when point count changes significantly
      // (batched to avoid excessive renders)
      const currentCount = pointsRef.current.length;
      if (Math.abs(currentCount - pointCount) >= 5 || currentCount < 5) {
        setPointCount(currentCount);
      }
    }
  });

  const points = pointsRef.current;

  // Memoize gradient colors - only recalculate when point count changes
  const colors = useMemo(() => {
    return points.map((_, i) => {
      const alpha = i / points.length;
      // Fade from transparent to cyan
      return new THREE.Color().setHSL(0.5, 0.8, 0.3 + alpha * 0.3);
    });
  }, [pointCount, points.length]);

  if (points.length < 2) {
    return null;
  }

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
