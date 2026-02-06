import { useRef } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

const GROUND_Y = 0.01;
const CONE_HEIGHT = 0.3;
const CONE_RADIUS = 0.08;
const RING_RADIUS = 0.25;
const GOAL_COLOR = "#fbbf24"; // amber, matching ACTIVE_WAYPOINT_COLOR

/**
 * Renders a glowing amber beacon at the navigation goal position.
 * Visible when navigationGoal is set (click-to-navigate).
 * Pulsing ring on the ground + upright cone marker.
 */
export function GoalMarker() {
  const groupRef = useRef<THREE.Group>(null);
  const ringRef = useRef<THREE.Mesh>(null);
  const coneRef = useRef<THREE.Mesh>(null);

  useFrame(({ clock }) => {
    if (!groupRef.current) return;

    const goal = useConsoleStore.getState().navigationGoal;
    if (!goal) {
      groupRef.current.visible = false;
      return;
    }

    groupRef.current.visible = true;

    // ENU → Three.js: x stays, y → -z
    groupRef.current.position.set(goal.x, GROUND_Y, -goal.y);

    // Pulse the ring scale
    if (ringRef.current) {
      const pulse = 1.0 + 0.15 * Math.sin(clock.elapsedTime * 3.0);
      ringRef.current.scale.set(pulse, pulse, 1);
    }
  });

  return (
    <group ref={groupRef} visible={false}>
      {/* Ground ring */}
      <mesh ref={ringRef} rotation={[-Math.PI / 2, 0, 0]}>
        <ringGeometry args={[RING_RADIUS - 0.03, RING_RADIUS, 32]} />
        <meshStandardMaterial
          color={GOAL_COLOR}
          emissive={GOAL_COLOR}
          emissiveIntensity={0.6}
          transparent
          opacity={0.7}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Upright cone */}
      <mesh ref={coneRef} position={[0, CONE_HEIGHT / 2, 0]}>
        <coneGeometry args={[CONE_RADIUS, CONE_HEIGHT, 8]} />
        <meshStandardMaterial
          color={GOAL_COLOR}
          emissive={GOAL_COLOR}
          emissiveIntensity={0.8}
          transparent
          opacity={0.9}
        />
      </mesh>
    </group>
  );
}
