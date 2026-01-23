import { useRef, useEffect, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

const MAX_POINTS = 500;
const MAX_POINTS_VIDEO_ACTIVE = 100; // Reduced trail when video is active for perf
const MIN_DISTANCE = 0.05; // Minimum distance between points (meters)

/**
 * Trail visualization showing the rover's path.
 * Renders as a fading line behind the rover using a reusable BufferGeometry.
 */
export function RoverTrail() {
  const { renderPose, connected, videoConnected } = useConsoleStore();
  const pointsRef = useRef<THREE.Vector3[]>([]);
  const lastPosRef = useRef<THREE.Vector3 | null>(null);
  // Reusable Vector3 to avoid allocations in useFrame
  const tempVec = useRef(new THREE.Vector3());
  // Track current point count for conditional rendering
  const countRef = useRef(0);

  // Use reduced trail length when video is active to reduce GPU work
  const maxPoints = videoConnected ? MAX_POINTS_VIDEO_ACTIVE : MAX_POINTS;

  // Pre-allocate geometry with max capacity
  const geometryRef = useRef<THREE.BufferGeometry | null>(null);
  const geometry = useMemo(() => {
    if (geometryRef.current) {
      geometryRef.current.dispose();
    }
    const geo = new THREE.BufferGeometry();
    // Pre-allocate position buffer (use full MAX_POINTS to avoid reallocation)
    const positions = new Float32Array(MAX_POINTS * 3);
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    // Pre-allocate color buffer
    const colors = new Float32Array(MAX_POINTS * 3);
    geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geo.setDrawRange(0, 0);
    geometryRef.current = geo;
    return geo;
  }, []);

  // Material with vertex colors
  const materialRef = useRef<THREE.LineBasicMaterial | null>(null);
  const material = useMemo(() => {
    if (materialRef.current) {
      materialRef.current.dispose();
    }
    const mat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.8,
      linewidth: 2,
    });
    materialRef.current = mat;
    return mat;
  }, []);

  // Reusable Color object to avoid allocations
  const tempColor = useRef(new THREE.Color());

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (geometryRef.current) {
        geometryRef.current.dispose();
        geometryRef.current = null;
      }
      if (materialRef.current) {
        materialRef.current.dispose();
        materialRef.current = null;
      }
      pointsRef.current = [];
    };
  }, []);

  // Clear trail on disconnect
  useEffect(() => {
    if (!connected) {
      pointsRef.current = [];
      lastPosRef.current = null;
      countRef.current = 0;
      geometry.setDrawRange(0, 0);
    }
  }, [connected, geometry]);

  // Add points and update geometry each frame
  useFrame(() => {
    tempVec.current.set(renderPose.x, 0.02, -renderPose.y);
    const lastPos = lastPosRef.current;
    let needsUpdate = false;

    // Only add point if moved far enough
    if (!lastPos || tempVec.current.distanceTo(lastPos) >= MIN_DISTANCE) {
      pointsRef.current.push(tempVec.current.clone());
      lastPosRef.current = tempVec.current.clone();

      // Trim to max points (reduced when video is active)
      while (pointsRef.current.length > maxPoints) {
        pointsRef.current.shift();
      }

      needsUpdate = true;
    }

    const points = pointsRef.current;
    const count = points.length;

    if (needsUpdate && count >= 2) {
      const positionAttr = geometry.getAttribute(
        "position"
      ) as THREE.BufferAttribute;
      const colorAttr = geometry.getAttribute("color") as THREE.BufferAttribute;

      for (let i = 0; i < count; i++) {
        const pt = points[i];
        positionAttr.array[i * 3] = pt.x;
        positionAttr.array[i * 3 + 1] = pt.y;
        positionAttr.array[i * 3 + 2] = pt.z;

        // Gradient color: fade from dark to cyan
        const alpha = i / count;
        tempColor.current.setHSL(0.5, 0.8, 0.3 + alpha * 0.3);
        colorAttr.array[i * 3] = tempColor.current.r;
        colorAttr.array[i * 3 + 1] = tempColor.current.g;
        colorAttr.array[i * 3 + 2] = tempColor.current.b;
      }

      positionAttr.needsUpdate = true;
      colorAttr.needsUpdate = true;
      geometry.setDrawRange(0, count);
      geometry.computeBoundingSphere();
      countRef.current = count;
    }
  });

  // Create a persistent Line object that we update
  const lineObj = useMemo(() => {
    return new THREE.Line(geometry, material);
  }, [geometry, material]);

  // Don't render if not enough points
  if (countRef.current < 2) {
    return null;
  }

  return <primitive object={lineObj} />;
}
