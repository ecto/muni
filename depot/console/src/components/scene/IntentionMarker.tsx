import { useRef, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

const NUM_POINTS = 20;
const LOOKAHEAD = 2.0; // seconds
const GROUND_Y = 0.03; // just above trail (0.02)

/**
 * Visualizes the policy's intended trajectory as a curved arc on the ground.
 *
 * When the BC policy is actively driving, samples the forward trajectory
 * using differential drive kinematics and draws it as a fading cyan dashed arc
 * with a small arrowhead at the endpoint.
 *
 * Hidden when policyIntention is null (teleop, idle, classical nav).
 */
export function IntentionMarker() {
  const groupRef = useRef<THREE.Group>(null);

  // Pre-allocate geometry
  const [lineGeo] = useState(() => {
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array(NUM_POINTS * 3);
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const colors = new Float32Array(NUM_POINTS * 3);
    geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geo.setDrawRange(0, 0);
    return geo;
  });

  const [lineMat] = useState(
    () =>
      new THREE.LineDashedMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0.9,
        dashSize: 0.15,
        gapSize: 0.08,
        linewidth: 2,
      })
  );

  const [lineObj] = useState(() => {
    const l = new THREE.Line(lineGeo, lineMat);
    l.computeLineDistances();
    return l;
  });

  // Arrow cone at the endpoint
  const coneRef = useRef<THREE.Mesh>(null);

  // Reusable color
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
    const intention = state.telemetry.policyIntention;

    if (!intention) {
      groupRef.current.visible = false;
      return;
    }

    groupRef.current.visible = true;

    const { renderPose, telemetry } = state;
    const v = telemetry.velocity.linear;
    const omega = telemetry.velocity.angular;

    // Sample trajectory using differential drive kinematics
    const posAttr = lineGeo.getAttribute("position") as THREE.BufferAttribute;
    const colAttr = lineGeo.getAttribute("color") as THREE.BufferAttribute;

    const theta0 = renderPose.theta;
    const x0 = renderPose.x;
    const y0 = renderPose.y;

    for (let i = 0; i < NUM_POINTS; i++) {
      const t = (i / (NUM_POINTS - 1)) * LOOKAHEAD;
      let px: number, py: number;

      if (Math.abs(omega) > 0.01) {
        // Arc trajectory
        const theta_t = theta0 + omega * t;
        px = x0 + (v / omega) * (Math.sin(theta_t) - Math.sin(theta0));
        py = y0 + (v / omega) * (Math.cos(theta0) - Math.cos(theta_t));
      } else {
        // Straight line
        px = x0 + v * Math.cos(theta0) * t;
        py = y0 + v * Math.sin(theta0) * t;
      }

      // Map to Three.js coordinates: z = -x, x = -y
      posAttr.array[i * 3] = -py;
      posAttr.array[i * 3 + 1] = GROUND_Y;
      posAttr.array[i * 3 + 2] = -px;

      // Fade from bright cyan near rover to dim at endpoint
      const alpha = 1.0 - i / (NUM_POINTS - 1);
      tempColor.current.set("#22d3ee");
      tempColor.current.multiplyScalar(0.3 + alpha * 0.7);
      colAttr.array[i * 3] = tempColor.current.r;
      colAttr.array[i * 3 + 1] = tempColor.current.g;
      colAttr.array[i * 3 + 2] = tempColor.current.b;
    }

    posAttr.needsUpdate = true;
    colAttr.needsUpdate = true;
    lineGeo.setDrawRange(0, NUM_POINTS);
    lineGeo.computeBoundingSphere();
    lineObj.computeLineDistances();

    // Position arrowhead cone at the endpoint
    if (coneRef.current) {
      const lastIdx = (NUM_POINTS - 1) * 3;
      coneRef.current.position.set(
        posAttr.array[lastIdx],
        GROUND_Y,
        posAttr.array[lastIdx + 2]
      );

      // Orient cone along the trajectory direction
      const endTheta =
        Math.abs(omega) > 0.01
          ? theta0 + omega * LOOKAHEAD
          : theta0;
      coneRef.current.rotation.set(0, endTheta, 0);
    }
  });

  return (
    <group ref={groupRef} visible={false}>
      <primitive object={lineObj} />
      <mesh ref={coneRef} rotation={[-Math.PI / 2, 0, 0]}>
        <coneGeometry args={[0.04, 0.1, 6]} />
        <meshStandardMaterial
          color="#22d3ee"
          emissive="#22d3ee"
          emissiveIntensity={0.8}
          transparent
          opacity={0.85}
        />
      </mesh>
    </group>
  );
}
