import { useRef, useMemo, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

/** Maximum accumulated points to render. */
const MAX_POINTS = 5000;

/** Reduced point count when video is active for performance. */
const MAX_POINTS_VIDEO_ACTIVE = 2000;

/** Point lifetime in seconds before fully faded. */
const POINT_LIFETIME = 3.0;

/** Point size in meters. */
const POINT_SIZE = 0.06;

/** Accumulated point with world position and age. */
interface AccumulatedPoint {
  // World position (Three.js coordinates)
  x: number;
  y: number;
  z: number;
  // Original reflectivity (0-255)
  reflectivity: number;
  // Time when point was added (performance.now() / 1000)
  timestamp: number;
}

/**
 * Renders point cloud data from LiDAR sensor with temporal accumulation.
 *
 * Points persist over time, fading out gradually to reveal structure.
 * New points are transformed to world coordinates using the rover's pose.
 */
export function PointCloud3D() {
  const { pointCloud, pointCloudReflectivity, pointCloudEnabled, renderPose, videoConnected } =
    useConsoleStore();

  // Use reduced point count when video is active to reduce GPU work
  const maxPoints = videoConnected ? MAX_POINTS_VIDEO_ACTIVE : MAX_POINTS;

  const pointsRef = useRef<THREE.Points>(null);

  // Accumulated points buffer (persists across renders)
  const accumulatedRef = useRef<AccumulatedPoint[]>([]);

  // Track last processed frame to avoid duplicate processing
  const lastFrameRef = useRef<Float32Array | null>(null);

  // Pre-allocate buffer geometry with cleanup
  const geometryRef = useRef<THREE.BufferGeometry | null>(null);
  const geometry = useMemo(() => {
    // Dispose previous geometry if it exists (shouldn't happen with stable useMemo, but safety)
    if (geometryRef.current) {
      geometryRef.current.dispose();
    }

    const geo = new THREE.BufferGeometry();

    // Position buffer (x, y, z per point)
    const positions = new Float32Array(MAX_POINTS * 3);
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));

    // Color buffer (r, g, b per point) - alpha encoded in brightness
    const colors = new Float32Array(MAX_POINTS * 3);
    geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));

    geo.setDrawRange(0, 0);
    geometryRef.current = geo;
    return geo;
  }, []);

  // Point material with cleanup
  const materialRef = useRef<THREE.PointsMaterial | null>(null);
  const material = useMemo(() => {
    // Dispose previous material if it exists
    if (materialRef.current) {
      materialRef.current.dispose();
    }

    const mat = new THREE.PointsMaterial({
      size: POINT_SIZE,
      vertexColors: true,
      sizeAttenuation: true,
      transparent: true,
      opacity: 0.85,
      depthWrite: false, // Better blending for overlapping points
    });
    materialRef.current = mat;
    return mat;
  }, []);

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
      // Clear accumulated points
      accumulatedRef.current = [];
    };
  }, []);

  // Add new points when data arrives
  useEffect(() => {
    if (!pointCloud || !pointCloudReflectivity || !pointCloudEnabled) {
      return;
    }

    // Skip if same frame data
    if (lastFrameRef.current === pointCloud) {
      return;
    }
    lastFrameRef.current = pointCloud;

    const now = performance.now() / 1000;
    const pointCount = pointCloud.length / 3;
    const accumulated = accumulatedRef.current;

    // Transform new points from sensor frame to world frame
    // Sensor: X forward, Y left, Z up (relative to rover)
    // World: X east, Y up, Z south (Three.js)
    const cosTheta = Math.cos(renderPose.theta);
    const sinTheta = Math.sin(renderPose.theta);

    for (let i = 0; i < pointCount; i++) {
      const sx = pointCloud[i * 3];     // sensor X (forward)
      const sy = pointCloud[i * 3 + 1]; // sensor Y (left)
      const sz = pointCloud[i * 3 + 2]; // sensor Z (up)

      // Transform sensor coords to rover-local Three.js coords
      // Rover-local: -sy = right, sz = up, -sx = back
      const localX = -sy;
      const localY = sz;
      const localZ = -sx;

      // Rotate by rover heading around Y axis, then translate to world position
      // World X = localX * cos(theta) - localZ * sin(theta) + rover.x
      // World Z = localX * sin(theta) + localZ * cos(theta) - rover.y
      const worldX = localX * cosTheta - localZ * sinTheta + renderPose.x;
      const worldY = localY; // Y is up, no rotation needed
      const worldZ = localX * sinTheta + localZ * cosTheta - renderPose.y;

      accumulated.push({
        x: worldX,
        y: worldY,
        z: worldZ,
        reflectivity: pointCloudReflectivity[i],
        timestamp: now,
      });
    }

    // Trim to max points (reduced when video is active for performance)
    if (accumulated.length > maxPoints) {
      accumulated.splice(0, accumulated.length - maxPoints);
    }
  }, [pointCloud, pointCloudReflectivity, pointCloudEnabled, renderPose, maxPoints]);

  // Clear accumulated points when disabled
  useEffect(() => {
    if (!pointCloudEnabled) {
      accumulatedRef.current = [];
      geometry.setDrawRange(0, 0);
    }
  }, [pointCloudEnabled, geometry]);

  // Update geometry each frame (fade and cull old points)
  useFrame(() => {
    if (!pointCloudEnabled) return;

    const now = performance.now() / 1000;
    const accumulated = accumulatedRef.current;

    // Remove expired points
    const cutoff = now - POINT_LIFETIME;
    while (accumulated.length > 0 && accumulated[0].timestamp < cutoff) {
      accumulated.shift();
    }

    const pointCount = accumulated.length;
    if (pointCount === 0) {
      geometry.setDrawRange(0, 0);
      return;
    }

    const positionAttr = geometry.getAttribute("position") as THREE.BufferAttribute;
    const colorAttr = geometry.getAttribute("color") as THREE.BufferAttribute;

    // Update positions and colors with age-based fading
    for (let i = 0; i < pointCount; i++) {
      const point = accumulated[i];

      // Position (already in world coordinates)
      positionAttr.array[i * 3] = point.x;
      positionAttr.array[i * 3 + 1] = point.y;
      positionAttr.array[i * 3 + 2] = point.z;

      // Age-based fade (1.0 = new, 0.0 = expired)
      const age = now - point.timestamp;
      const fade = Math.max(0, 1 - age / POINT_LIFETIME);

      // Color: blue-to-white gradient based on reflectivity, faded by age
      const refl = point.reflectivity / 255;
      const brightness = fade * fade; // Quadratic fade for smoother falloff

      colorAttr.array[i * 3] = (0.3 + refl * 0.7) * brightness;     // R
      colorAttr.array[i * 3 + 1] = (0.5 + refl * 0.5) * brightness; // G
      colorAttr.array[i * 3 + 2] = 1.0 * brightness;                 // B
    }

    positionAttr.needsUpdate = true;
    colorAttr.needsUpdate = true;
    geometry.setDrawRange(0, pointCount);
    geometry.computeBoundingSphere();
  });

  return (
    <points
      ref={pointsRef}
      geometry={geometry}
      material={material}
      visible={pointCloudEnabled}
    />
  );
}
