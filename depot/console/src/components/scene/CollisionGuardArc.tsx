import { useRef, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { getCostmapData } from "@/lib/costmapStore";
import { setGuardState } from "@/lib/collisionGuardStore";
import { Mode } from "@/lib/types";

const NUM_SAMPLES = 10;
const LOOKAHEAD_TIME = 1.0; // seconds
const FULL_SPEED_DIST = 1.0; // meters — beyond this, no scaling
const STOP_DIST = 0.15; // meters — closer than this, velocity zeroed
const COST_THRESHOLD = 253; // occupancy value considered obstacle
const GROUND_Y = 0.025; // between trail (0.02) and IntentionMarker (0.03)

const COLOR_GREEN = new THREE.Color("#22c55e");
const COLOR_YELLOW = new THREE.Color("#eab308");
const COLOR_RED = new THREE.Color("#ef4444");

/**
 * Visualizes the collision guard's projected trajectory as a colored arc
 * on the ground plane.
 *
 * Green segments = clear, yellow = approaching obstacle, red = obstacle hit.
 * The arc is truncated at the first obstacle. Only visible in Teleop mode
 * when a costmap is available and the rover is moving.
 */
export function CollisionGuardArc() {
  const groupRef = useRef<THREE.Group>(null);

  const [lineGeo] = useState(() => {
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array((NUM_SAMPLES + 1) * 3);
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const colors = new Float32Array((NUM_SAMPLES + 1) * 3);
    geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geo.setDrawRange(0, 0);
    return geo;
  });

  const [lineMat] = useState(
    () =>
      new THREE.LineBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0.9,
        linewidth: 2,
      })
  );

  const [lineObj] = useState(() => new THREE.Line(lineGeo, lineMat));

  const tempColor = useRef(new THREE.Color());

  useEffect(() => {
    return () => {
      lineGeo.dispose();
      lineMat.dispose();
    };
  }, [lineGeo, lineMat]);

  useFrame(() => {
    if (!groupRef.current) return;

    const state = useConsoleStore.getState();
    const mode = state.telemetry.mode;

    // Only show in Teleop mode
    if (mode !== Mode.Teleop) {
      groupRef.current.visible = false;
      setGuardState(0, false);
      return;
    }

    const { renderPose, telemetry } = state;
    const v = telemetry.velocity.linear;
    const omega = telemetry.velocity.angular;

    const costmap = getCostmapData();

    // Hide if no costmap or near-zero linear velocity
    if (!costmap.cells || Math.abs(v) < 0.01) {
      groupRef.current.visible = false;
      setGuardState(0, false);
      return;
    }

    groupRef.current.visible = true;

    const posAttr = lineGeo.getAttribute("position") as THREE.BufferAttribute;
    const colAttr = lineGeo.getAttribute("color") as THREE.BufferAttribute;

    const theta0 = renderPose.theta;
    const x0 = renderPose.x;
    const y0 = renderPose.y;

    const { cells, width, height, resolution, originX, originY } = costmap;

    let drawCount = NUM_SAMPLES + 1;
    let minObstacleDist = Infinity;

    for (let i = 0; i <= NUM_SAMPLES; i++) {
      const t = (i / NUM_SAMPLES) * LOOKAHEAD_TIME;

      // Differential drive kinematics in body frame, then transform to world
      let px: number, py: number;
      if (Math.abs(omega) < 0.001) {
        // Straight line
        px = x0 + v * Math.cos(theta0) * t;
        py = y0 + v * Math.sin(theta0) * t;
      } else {
        // Circular arc
        const R = v / omega;
        const thetaT = theta0 + omega * t;
        px = x0 + R * (Math.sin(thetaT) - Math.sin(theta0));
        py = y0 + R * (Math.cos(theta0) - Math.cos(thetaT));
      }

      // Compute distance from rover along the arc
      const dx = px - x0;
      const dy = py - y0;
      const dist = Math.sqrt(dx * dx + dy * dy);

      // Look up costmap cost at this world position
      const gx = Math.floor((px - originX) / resolution);
      const gy = Math.floor((py - originY) / resolution);
      let cost = 0;
      if (gx >= 0 && gx < width && gy >= 0 && gy < height) {
        cost = cells[gy * width + gx];
      }

      const isObstacle = cost >= COST_THRESHOLD;

      if (isObstacle && dist < minObstacleDist) {
        minObstacleDist = dist;
      }

      // Color based on proximity to obstacle
      if (isObstacle) {
        // Hit obstacle — red, truncate after this point
        tempColor.current.copy(COLOR_RED);
        // Set position for this point, then truncate
        posAttr.array[i * 3] = -py; // three.x = -physics.y
        posAttr.array[i * 3 + 1] = GROUND_Y;
        posAttr.array[i * 3 + 2] = -px; // three.z = -physics.x
        colAttr.array[i * 3] = tempColor.current.r;
        colAttr.array[i * 3 + 1] = tempColor.current.g;
        colAttr.array[i * 3 + 2] = tempColor.current.b;
        drawCount = i + 1;
        break;
      }

      // Color based on distance to nearest obstacle found so far
      // We color based on how close this point is to the obstacle,
      // using the scaling zone thresholds
      let color: THREE.Color;
      if (minObstacleDist <= STOP_DIST) {
        color = COLOR_RED;
      } else if (minObstacleDist < FULL_SPEED_DIST) {
        // Lerp green → yellow → red based on distance within scaling zone
        const ratio =
          (minObstacleDist - STOP_DIST) / (FULL_SPEED_DIST - STOP_DIST);
        if (ratio > 0.5) {
          tempColor.current.copy(COLOR_GREEN).lerp(COLOR_YELLOW, 1 - (ratio - 0.5) * 2);
          color = tempColor.current;
        } else {
          tempColor.current.copy(COLOR_YELLOW).lerp(COLOR_RED, 1 - ratio * 2);
          color = tempColor.current;
        }
      } else {
        color = COLOR_GREEN;
      }

      // Map to Three.js coords: three.x = -physics.y, three.z = -physics.x
      posAttr.array[i * 3] = -py;
      posAttr.array[i * 3 + 1] = GROUND_Y;
      posAttr.array[i * 3 + 2] = -px;

      colAttr.array[i * 3] = color.r;
      colAttr.array[i * 3 + 1] = color.g;
      colAttr.array[i * 3 + 2] = color.b;
    }

    posAttr.needsUpdate = true;
    colAttr.needsUpdate = true;
    lineGeo.setDrawRange(0, drawCount);
    lineGeo.computeBoundingSphere();

    // Publish guard state for StatusPanel
    const finalDist = minObstacleDist === Infinity ? FULL_SPEED_DIST : minObstacleDist;
    setGuardState(finalDist, true);
  });

  return (
    <group ref={groupRef} visible={false}>
      <primitive object={lineObj} />
    </group>
  );
}
