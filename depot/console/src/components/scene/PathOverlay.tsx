import { useRef, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { getPathData, getPathVersion } from "@/lib/pathStore";
import { NavState } from "@/lib/protocol";

const GROUND_Y = 0.04; // just above trail (0.02) and costmap (0.03)
const WAYPOINT_RADIUS = 0.06;
const CURRENT_WAYPOINT_RADIUS = 0.10;
const PATH_COLOR = "#10b981"; // emerald-500
const CURRENT_WAYPOINT_COLOR = "#fbbf24"; // amber — matching GoalMarker

// Pre-allocated geometries
const SPHERE_GEO = new THREE.SphereGeometry(WAYPOINT_RADIUS, 12, 12);
const SPHERE_GEO_ACTIVE = new THREE.SphereGeometry(CURRENT_WAYPOINT_RADIUS, 16, 16);

/**
 * Renders the A* planned navigation path as a line on the ground
 * with sphere markers at each waypoint.
 *
 * Visible only when the rover is in autonomous mode and actively
 * following or planning a path.
 */
export function PathOverlay() {
  const groupRef = useRef<THREE.Group>(null);
  const lastVersionRef = useRef(0);

  // Pre-allocate line geometry
  const [lineGeo] = useState(() => {
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array(512 * 3); // up to 512 waypoints
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geo.setDrawRange(0, 0);
    return geo;
  });

  const [lineMat] = useState(
    () =>
      new THREE.LineBasicMaterial({
        color: PATH_COLOR,
        transparent: true,
        opacity: 0.8,
        linewidth: 2,
      })
  );

  const [lineObj] = useState(() => new THREE.Line(lineGeo, lineMat));

  // Waypoint sphere pool
  const spherePoolRef = useRef<THREE.Mesh[]>([]);
  const pulseRingRef = useRef<THREE.Mesh>(null);

  useEffect(() => {
    return () => {
      lineGeo.dispose();
      lineMat.dispose();
    };
  }, [lineGeo, lineMat]);

  useFrame(({ clock }) => {
    if (!groupRef.current) return;

    const version = getPathVersion();

    // Only update geometry when data changes
    if (version !== lastVersionRef.current) {
      lastVersionRef.current = version;

      const { state, waypoints, currentWaypoint } = getPathData();

      // Only show when actively navigating
      const visible =
        state === NavState.Following ||
        state === NavState.Planning ||
        state === NavState.Replanning ||
        state === NavState.ObstacleStopped ||
        state === NavState.Recovering;

      if (!visible || waypoints.length === 0) {
        groupRef.current.visible = false;
        return;
      }

      groupRef.current.visible = true;

      // Update line geometry
      const posAttr = lineGeo.getAttribute("position") as THREE.BufferAttribute;
      const count = Math.min(waypoints.length, 512);

      for (let i = 0; i < count; i++) {
        const wp = waypoints[i];
        posAttr.array[i * 3] = wp.x;
        posAttr.array[i * 3 + 1] = GROUND_Y;
        posAttr.array[i * 3 + 2] = -wp.y;
      }

      posAttr.needsUpdate = true;
      lineGeo.setDrawRange(0, count);
      lineGeo.computeBoundingSphere();

      // Update waypoint spheres
      const pool = spherePoolRef.current;

      // Hide excess spheres
      for (let i = count; i < pool.length; i++) {
        pool[i].visible = false;
      }

      // Create or update spheres
      for (let i = 0; i < count; i++) {
        let sphere = pool[i];
        if (!sphere) {
          const mat = new THREE.MeshStandardMaterial({
            color: PATH_COLOR,
            emissive: PATH_COLOR,
            emissiveIntensity: 0.3,
            transparent: true,
            opacity: 0.7,
          });
          sphere = new THREE.Mesh(SPHERE_GEO, mat);
          pool[i] = sphere;
          groupRef.current.add(sphere);
        }

        const wp = waypoints[i];
        sphere.position.set(wp.x, GROUND_Y, -wp.y);
        sphere.visible = true;

        // Highlight current target waypoint
        const isCurrent = i === currentWaypoint;
        sphere.geometry = isCurrent ? SPHERE_GEO_ACTIVE : SPHERE_GEO;
        const mat = sphere.material as THREE.MeshStandardMaterial;
        mat.color.set(isCurrent ? CURRENT_WAYPOINT_COLOR : PATH_COLOR);
        mat.emissive.set(isCurrent ? CURRENT_WAYPOINT_COLOR : PATH_COLOR);
        mat.emissiveIntensity = isCurrent ? 0.6 : 0.3;
        mat.opacity = isCurrent ? 0.9 : 0.7;
      }

      // Position the pulse ring at the current waypoint
      if (pulseRingRef.current && currentWaypoint < waypoints.length) {
        const wp = waypoints[currentWaypoint];
        pulseRingRef.current.position.set(wp.x, GROUND_Y + 0.005, -wp.y);
        pulseRingRef.current.visible = true;
      } else if (pulseRingRef.current) {
        pulseRingRef.current.visible = false;
      }
    }

    // Animate pulse ring even when data hasn't changed
    if (pulseRingRef.current && pulseRingRef.current.visible) {
      const pulse = 1.0 + 0.2 * Math.sin(clock.elapsedTime * 3.0);
      pulseRingRef.current.scale.set(pulse, pulse, 1);
    }
  });

  return (
    <group ref={groupRef} visible={false}>
      <primitive object={lineObj} />
      {/* Pulse ring at current waypoint */}
      <mesh ref={pulseRingRef} rotation={[-Math.PI / 2, 0, 0]} visible={false}>
        <ringGeometry args={[CURRENT_WAYPOINT_RADIUS - 0.02, CURRENT_WAYPOINT_RADIUS + 0.02, 32]} />
        <meshStandardMaterial
          color={CURRENT_WAYPOINT_COLOR}
          emissive={CURRENT_WAYPOINT_COLOR}
          emissiveIntensity={0.5}
          transparent
          opacity={0.6}
          side={THREE.DoubleSide}
        />
      </mesh>
    </group>
  );
}
