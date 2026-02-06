import { useRef, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { getMppiTrajectory, getMppiVersion } from "@/lib/pathStore";

const GROUND_Y = 0.05; // just above path overlay (0.04)
const TRAJ_COLOR = "#06b6d4"; // cyan-500
const MAX_POINTS = 128;

/**
 * Renders the MPPI best trajectory as a line on the ground.
 *
 * Visible only when the MPPI controller is active and producing
 * trajectory data (autonomous mode with MPPI controller type).
 */
export function MppiOverlay() {
  const groupRef = useRef<THREE.Group>(null);
  const lastVersionRef = useRef(0);

  const [lineGeo] = useState(() => {
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array(MAX_POINTS * 3);
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geo.setDrawRange(0, 0);
    return geo;
  });

  const [lineMat] = useState(
    () =>
      new THREE.LineBasicMaterial({
        color: TRAJ_COLOR,
        transparent: true,
        opacity: 0.8,
        linewidth: 2,
      })
  );

  const [lineObj] = useState(() => new THREE.Line(lineGeo, lineMat));

  useEffect(() => {
    return () => {
      lineGeo.dispose();
      lineMat.dispose();
    };
  }, [lineGeo, lineMat]);

  useFrame(() => {
    if (!groupRef.current) return;

    const version = getMppiVersion();
    if (version === lastVersionRef.current) return;
    lastVersionRef.current = version;

    const traj = getMppiTrajectory();

    if (!traj || traj.length < 2) {
      groupRef.current.visible = false;
      return;
    }

    const posAttr = lineGeo.getAttribute("position") as THREE.BufferAttribute;
    const count = Math.min(traj.length, MAX_POINTS);

    for (let i = 0; i < count; i++) {
      posAttr.array[i * 3] = traj[i][0];       // x (east)
      posAttr.array[i * 3 + 1] = GROUND_Y;     // slight elevation
      posAttr.array[i * 3 + 2] = -traj[i][1];  // -y (north -> -z)
    }

    posAttr.needsUpdate = true;
    lineGeo.setDrawRange(0, count);
    lineGeo.computeBoundingSphere();

    groupRef.current.visible = true;
  });

  return (
    <group ref={groupRef} visible={false}>
      <primitive object={lineObj} />
    </group>
  );
}
