import { useRef, useMemo, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { getPointCloudData, getPointCloudVersion } from "@/lib/pointCloudStore";

/** Maximum points in the ring buffer. */
const MAX_POINTS = 6000;

/** How long points stay visible (seconds). */
const POINT_LIFETIME = 1.5;

/** Point size in meters — large enough to read at teleop camera distance. */
const POINT_SIZE = 0.09;

/**
 * Map a height (Y in Three.js, meters) to an RGB color.
 * Blue (ground) → cyan → green → yellow → red (high).
 * Standard approach for LiDAR visualization.
 */
function heightColor(y: number, out: Float32Array, offset: number) {
  // Clamp to reasonable range and normalize to 0–1
  const t = Math.max(0, Math.min(1, (y + 0.2) / 2.0));

  // 5-stop gradient: blue → cyan → green → yellow → red
  if (t < 0.25) {
    const s = t / 0.25;
    out[offset] = 0.0;
    out[offset + 1] = s;
    out[offset + 2] = 1.0;
  } else if (t < 0.5) {
    const s = (t - 0.25) / 0.25;
    out[offset] = 0.0;
    out[offset + 1] = 1.0;
    out[offset + 2] = 1.0 - s;
  } else if (t < 0.75) {
    const s = (t - 0.5) / 0.25;
    out[offset] = s;
    out[offset + 1] = 1.0;
    out[offset + 2] = 0.0;
  } else {
    const s = (t - 0.75) / 0.25;
    out[offset] = 1.0;
    out[offset + 1] = 1.0 - s;
    out[offset + 2] = 0.0;
  }
}

/**
 * Renders LiDAR point cloud in rover-local coordinates with short
 * temporal accumulation so the sparse Livox non-repetitive scan
 * builds enough density to be readable. Height-based coloring gives
 * instant depth perception. The <points> object tracks the rover's
 * pose each frame.
 */
export function PointCloud3D() {
  const pointCloudEnabled = useConsoleStore((s) => s.pointCloudEnabled);

  const pointsRef = useRef<THREE.Points>(null);
  const lastVersionRef = useRef<number>(0);

  // Ring buffer — fixed-size, no allocations after init
  const ringRef = useRef({
    pos: new Float32Array(MAX_POINTS * 3),
    time: new Float32Array(MAX_POINTS),
    head: 0,
    count: 0,
  });

  const geometryRef = useRef<THREE.BufferGeometry | null>(null);
  const geometry = useMemo(() => {
    if (geometryRef.current) geometryRef.current.dispose();
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(MAX_POINTS * 3), 3));
    geo.setAttribute("color", new THREE.BufferAttribute(new Float32Array(MAX_POINTS * 3), 3));
    geo.setDrawRange(0, 0);
    geometryRef.current = geo;
    return geo;
  }, []);

  const materialRef = useRef<THREE.PointsMaterial | null>(null);
  const material = useMemo(() => {
    if (materialRef.current) materialRef.current.dispose();
    const mat = new THREE.PointsMaterial({
      size: POINT_SIZE,
      vertexColors: true,
      sizeAttenuation: true,
      transparent: true,
      opacity: 0.9,
      depthWrite: false,
    });
    materialRef.current = mat;
    return mat;
  }, []);

  useEffect(() => {
    return () => {
      geometryRef.current?.dispose();
      geometryRef.current = null;
      materialRef.current?.dispose();
      materialRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!pointCloudEnabled) {
      const ring = ringRef.current;
      ring.head = 0;
      ring.count = 0;
      lastVersionRef.current = 0;
      geometry.setDrawRange(0, 0);
    }
  }, [pointCloudEnabled, geometry]);

  useFrame(() => {
    if (!pointCloudEnabled) return;

    const now = performance.now() / 1000;
    const ring = ringRef.current;
    let dirty = false;

    // Ingest new data into ring buffer
    const version = getPointCloudVersion();
    if (version !== lastVersionRef.current) {
      lastVersionRef.current = version;
      dirty = true;

      const { points } = getPointCloudData();
      if (points) {
        const incoming = points.length / 3;
        for (let i = 0; i < incoming; i++) {
          const idx = ring.head;
          // Sensor → rover-local Three.js: -sy right, sz up, -sx back
          ring.pos[idx * 3] = -points[i * 3 + 1];
          ring.pos[idx * 3 + 1] = points[i * 3 + 2];
          ring.pos[idx * 3 + 2] = -points[i * 3];
          ring.time[idx] = now;
          ring.head = (idx + 1) % MAX_POINTS;
          if (ring.count < MAX_POINTS) ring.count++;
        }
      }
    }

    // Rebuild GPU buffers — on new data or periodically for fade
    if (!dirty) {
      // Sync pose even when no new data
      if (pointsRef.current) {
        const rp = useConsoleStore.getState().renderPose;
        pointsRef.current.position.x = -rp.y;
        pointsRef.current.position.z = -rp.x;
        pointsRef.current.rotation.y = rp.theta;
      }
      return;
    }

    const posArr = (geometry.getAttribute("position") as THREE.BufferAttribute).array as Float32Array;
    const colArr = (geometry.getAttribute("color") as THREE.BufferAttribute).array as Float32Array;
    const cutoff = now - POINT_LIFETIME;
    let out = 0;

    for (let i = 0; i < ring.count; i++) {
      const idx = (ring.head - ring.count + i + MAX_POINTS) % MAX_POINTS;
      if (ring.time[idx] < cutoff) continue;

      const px = ring.pos[idx * 3];
      const py = ring.pos[idx * 3 + 1];
      const pz = ring.pos[idx * 3 + 2];

      posArr[out * 3] = px;
      posArr[out * 3 + 1] = py;
      posArr[out * 3 + 2] = pz;

      // Age fade: dim older points
      const fade = (ring.time[idx] - cutoff) / POINT_LIFETIME;

      // Height-based color, scaled by age
      heightColor(py, colArr, out * 3);
      colArr[out * 3] *= fade;
      colArr[out * 3 + 1] *= fade;
      colArr[out * 3 + 2] *= fade;

      out++;
    }

    (geometry.getAttribute("position") as THREE.BufferAttribute).needsUpdate = true;
    (geometry.getAttribute("color") as THREE.BufferAttribute).needsUpdate = true;
    geometry.setDrawRange(0, out);
    if (out > 0) geometry.computeBoundingSphere();

    // Sync pose with rover
    if (pointsRef.current) {
      const rp = useConsoleStore.getState().renderPose;
      pointsRef.current.position.x = -rp.y;
      pointsRef.current.position.z = -rp.x;
      pointsRef.current.rotation.y = rp.theta;
    }
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
