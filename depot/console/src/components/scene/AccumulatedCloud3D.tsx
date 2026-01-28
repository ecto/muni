import { useRef, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import {
  getAccumulatedCloud,
  getAccumulatedCloudVersion,
} from "@/lib/accumulatedCloudStore";

/** Must match accumulatedCloudStore MAX_POINTS. */
const MAX_POINTS = 200_000;

/** Slightly smaller than live cloud so it reads as background context. */
const POINT_SIZE = 0.06;

/**
 * Renders the accumulated world-frame point cloud built up during teleop.
 *
 * Unlike PointCloud3D (rover-local, ephemeral), this sits at the world origin
 * with pre-transformed coordinates and points persist across frames.
 */
export function AccumulatedCloud3D() {
  const enabled = useConsoleStore((s) => s.accumulatedCloudEnabled);
  const lastVersionRef = useRef(0);
  const pointsRef = useRef<THREE.Points>(null);

  const [geometry] = useState(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(MAX_POINTS * 3), 3),
    );
    geo.setAttribute(
      "color",
      new THREE.BufferAttribute(new Float32Array(MAX_POINTS * 3), 3),
    );
    geo.setDrawRange(0, 0);
    return geo;
  });

  const [material] = useState(
    () =>
      new THREE.PointsMaterial({
        size: POINT_SIZE,
        vertexColors: true,
        sizeAttenuation: true,
        transparent: true,
        opacity: 0.7,
        depthWrite: false,
      }),
  );

  useEffect(() => {
    return () => {
      geometry.dispose();
      material.dispose();
    };
  }, [geometry, material]);

  useFrame(() => {
    if (!enabled) return;

    const version = getAccumulatedCloudVersion();
    if (version === lastVersionRef.current) return;
    lastVersionRef.current = version;

    const { positions, colors, count } = getAccumulatedCloud();
    if (count === 0) {
      geometry.setDrawRange(0, 0);
      return;
    }

    // Copy store data into GPU buffers
    const posAttr = geometry.getAttribute("position") as THREE.BufferAttribute;
    const colAttr = geometry.getAttribute("color") as THREE.BufferAttribute;
    (posAttr.array as Float32Array).set(positions.subarray(0, count * 3));
    (colAttr.array as Float32Array).set(colors.subarray(0, count * 3));
    posAttr.needsUpdate = true;
    colAttr.needsUpdate = true;
    geometry.setDrawRange(0, count);
    geometry.computeBoundingSphere();
  });

  return (
    <points
      ref={pointsRef}
      geometry={geometry}
      material={material}
      visible={enabled}
    />
  );
}
