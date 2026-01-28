import { useMemo } from "react";
import { Text, Billboard } from "@react-three/drei";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { Mode, type RoverInfo } from "@/lib/types";

interface FleetMarkersProps {
  /** ID of the rover to show with full model (others get simple markers) */
  selectedRoverId?: string | null;
  /** Currently active task ID for highlighting rovers with tasks */
  activeTaskRoverId?: string | null;
}

// Color by rover mode
const LABEL_FONT = "/BerkeleyMono-Regular.woff2";

const MODE_COLORS: Record<number, string> = {
  [Mode.Disabled]: "#6b7280", // gray
  [Mode.Idle]: "#9ca3af", // light gray
  [Mode.Teleop]: "#3b82f6", // blue
  [Mode.Autonomous]: "#22c55e", // green
  [Mode.EStop]: "#f59e0b", // amber
  [Mode.Fault]: "#ef4444", // red
};

/**
 * Renders markers for all rovers in the fleet on the 3D map.
 * Selected rover gets the full model, others get simplified cone markers.
 */
export function FleetMarkers({ selectedRoverId, activeTaskRoverId }: FleetMarkersProps) {
  const { rovers } = useConsoleStore();

  // Filter to online rovers that aren't the selected one
  const otherRovers = useMemo(
    () => rovers.filter((r) => r.id !== selectedRoverId && r.online),
    [rovers, selectedRoverId]
  );

  return (
    <group>
      {otherRovers.map((rover) => (
        <RoverMarker
          key={rover.id}
          rover={rover}
          isActive={rover.id === activeTaskRoverId}
        />
      ))}
    </group>
  );
}

interface RoverMarkerProps {
  rover: RoverInfo;
  isActive: boolean;
}

function RoverMarker({ rover, isActive }: RoverMarkerProps) {
  const color = MODE_COLORS[rover.mode] || MODE_COLORS[Mode.Disabled];

  // Convert from physics coords to Three.js
  // Physics: X=forward, Y=left, theta=0 facing +X
  // Three.js: -Z=forward, +X=right
  const position = useMemo(
    () => new THREE.Vector3(-rover.lastPose.y, 0.15, -rover.lastPose.x),
    [rover.lastPose.x, rover.lastPose.y]
  );

  return (
    <group position={position} rotation={[0, rover.lastPose.theta, 0]}>
      {/* Cone marker pointing in direction of travel */}
      <mesh rotation={[-Math.PI / 2, 0, Math.PI / 2]}>
        <coneGeometry args={[0.15, 0.35, 8]} />
        <meshStandardMaterial
          color={color}
          emissive={isActive ? color : "#000000"}
          emissiveIntensity={isActive ? 0.5 : 0}
        />
      </mesh>

      {/* Pulsing ring for active rovers */}
      {isActive && <ActivePulse color={color} />}

      {/* Rover ID label */}
      <Billboard position={[0, 0.5, 0]}>
        <Text
          font={LABEL_FONT}
          fontSize={0.2}
          color={color}
          anchorX="center"
          anchorY="middle"
          outlineWidth={0.02}
          outlineColor="#000000"
        >
          {rover.name || rover.id}
        </Text>
      </Billboard>
    </group>
  );
}

function ActivePulse({ color }: { color: string }) {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, 0.01, 0]}>
      <ringGeometry args={[0.25, 0.35, 16]} />
      <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} />
    </mesh>
  );
}
