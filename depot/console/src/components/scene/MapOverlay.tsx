import { useMemo, useRef } from "react";
import { Line, Text, Billboard } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import type { Zone, Waypoint, Task } from "@/lib/types";
import { TaskStatus } from "@/lib/types";

interface MapOverlayProps {
  zones: Zone[];
  /** Active tasks for highlighting waypoints */
  activeTasks?: Task[];
  /** Callback when a zone is clicked */
  onZoneClick?: (zoneId: string) => void;
}

// Colors for different zone types
const ZONE_COLORS: Record<string, string> = {
  route: "#3b82f6", // blue
  area: "#22c55e", // green
  exclusion: "#ef4444", // red
  default: "#a855f7", // purple
};

const ACTIVE_ZONE_COLOR = "#22c55e"; // green for active zones
const ACTIVE_WAYPOINT_COLOR = "#fbbf24"; // amber for current waypoint

/**
 * Renders zones and waypoints from the dispatch service on the ground plane.
 */
export function MapOverlay({ zones, activeTasks, onZoneClick }: MapOverlayProps) {
  return (
    <group>
      {zones.map((zone) => (
        <ZoneVisualization
          key={zone.id}
          zone={zone}
          activeTask={activeTasks?.find((t) =>
            t.status === TaskStatus.Active || t.status === TaskStatus.Assigned
          )}
          onClick={onZoneClick ? () => onZoneClick(zone.id) : undefined}
        />
      ))}
    </group>
  );
}

interface ZoneVisualizationProps {
  zone: Zone;
  activeTask?: Task;
  onClick?: () => void;
}

function ZoneVisualization({ zone, activeTask, onClick }: ZoneVisualizationProps) {
  const isActive = !!activeTask;
  const baseColor = ZONE_COLORS[zone.zoneType] || ZONE_COLORS.default;
  const color = isActive ? ACTIVE_ZONE_COLOR : baseColor;
  const waypoints = zone.waypoints;

  // Current waypoint index from active task
  const currentWaypointIndex = activeTask?.waypoint ?? -1;

  // Create line points for route visualization
  const linePoints = useMemo(() => {
    if (waypoints.length < 2) return null;
    return waypoints.map((wp) => new THREE.Vector3(wp.x, 0.03, -wp.y));
  }, [waypoints]);

  // Create closed polygon if zone type is area/exclusion
  const polygonPoints = useMemo(() => {
    if (zone.zoneType === "route" || waypoints.length < 3) return null;
    const points = waypoints.map((wp) => new THREE.Vector3(wp.x, 0.02, -wp.y));
    // Close the polygon
    points.push(points[0].clone());
    return points;
  }, [waypoints, zone.zoneType]);

  return (
    <group onClick={onClick}>
      {/* Route line */}
      {linePoints && (
        <Line
          points={linePoints}
          color={color}
          lineWidth={isActive ? 3 : 2}
          dashed={zone.zoneType === "exclusion"}
          dashSize={0.3}
          gapSize={0.15}
        />
      )}

      {/* Area polygon outline */}
      {polygonPoints && (
        <Line
          points={polygonPoints}
          color={color}
          lineWidth={isActive ? 4 : 3}
          dashed={zone.zoneType === "exclusion"}
          dashSize={0.3}
          gapSize={0.15}
        />
      )}

      {/* Waypoint markers */}
      {waypoints.map((wp, i) => (
        <WaypointMarker
          key={i}
          waypoint={wp}
          index={i}
          color={color}
          showLabel={waypoints.length <= 20}
          isActive={i === currentWaypointIndex}
        />
      ))}

      {/* Zone name label at centroid */}
      {waypoints.length > 0 && (
        <ZoneLabel zone={zone} color={color} isActive={isActive} />
      )}
    </group>
  );
}

// Memoized geometry instances to avoid recreation
const SPHERE_GEOMETRY_ACTIVE = new THREE.SphereGeometry(0.12, 16, 16);
const SPHERE_GEOMETRY_INACTIVE = new THREE.SphereGeometry(0.08, 16, 16);
const CONE_GEOMETRY = new THREE.ConeGeometry(0.05, 0.1, 8);

// Constant position tuples
const LABEL_POSITION: [number, number, number] = [0, 0.35, 0];

function WaypointMarker({
  waypoint,
  index,
  color,
  showLabel,
  isActive = false,
}: {
  waypoint: Waypoint;
  index: number;
  color: string;
  showLabel: boolean;
  isActive?: boolean;
}) {
  const hasHeading = waypoint.theta !== undefined;
  const displayColor = isActive ? ACTIVE_WAYPOINT_COLOR : color;

  // Memoize position to avoid recreating array on every render
  const position = useMemo(
    (): [number, number, number] => [waypoint.x, 0.05, -waypoint.y],
    [waypoint.x, waypoint.y]
  );

  // Memoize direction indicator position
  const directionPosition = useMemo(
    (): [number, number, number] | null =>
      hasHeading
        ? [Math.cos(waypoint.theta!) * 0.15, 0, -Math.sin(waypoint.theta!) * 0.15]
        : null,
    [hasHeading, waypoint.theta]
  );

  return (
    <group position={position}>
      {/* Marker sphere */}
      <mesh geometry={isActive ? SPHERE_GEOMETRY_ACTIVE : SPHERE_GEOMETRY_INACTIVE}>
        <meshStandardMaterial
          color={displayColor}
          emissive={isActive ? displayColor : "#000000"}
          emissiveIntensity={isActive ? 0.8 : 0}
        />
      </mesh>

      {/* Pulsing ring for active waypoint */}
      {isActive && <PulsingRing color={displayColor} />}

      {/* Direction indicator if theta is set */}
      {directionPosition && (
        <mesh position={directionPosition} geometry={CONE_GEOMETRY}>
          <meshBasicMaterial color={displayColor} />
        </mesh>
      )}

      {/* Index label */}
      {showLabel && (
        <Billboard position={LABEL_POSITION}>
          <Text

            fontSize={isActive ? 0.2 : 0.15}
            color={displayColor}
            anchorX="center"
            anchorY="middle"
            outlineWidth={0.02}
            outlineColor="#000000"
          >
            {index + 1}
          </Text>
        </Billboard>
      )}
    </group>
  );
}

// Constant geometry and transforms for PulsingRing
const RING_GEOMETRY = new THREE.RingGeometry(0.15, 0.25, 24);
const RING_ROTATION: [number, number, number] = [-Math.PI / 2, 0, 0];
const RING_POSITION: [number, number, number] = [0, 0.01, 0];

/** Pulsing ring effect for active waypoints */
function PulsingRing({ color }: { color: string }) {
  const ringRef = useRef<THREE.Mesh>(null);

  useFrame(({ clock }) => {
    if (!ringRef.current) return;
    // Pulse scale between 1.0 and 1.5
    const scale = 1.0 + 0.5 * Math.sin(clock.elapsedTime * 3);
    ringRef.current.scale.setScalar(scale);
    // Pulse opacity between 0.3 and 0.8
    const mat = ringRef.current.material as THREE.MeshBasicMaterial;
    mat.opacity = 0.3 + 0.5 * (1 - Math.sin(clock.elapsedTime * 3));
  });

  return (
    <mesh
      ref={ringRef}
      rotation={RING_ROTATION}
      position={RING_POSITION}
      geometry={RING_GEOMETRY}
    >
      <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} />
    </mesh>
  );
}

function ZoneLabel({ zone, color, isActive = false }: { zone: Zone; color: string; isActive?: boolean }) {
  // Calculate centroid of waypoints
  const centroid = useMemo(() => {
    const wps = zone.waypoints;
    if (wps.length === 0) return new THREE.Vector3(0, 0.5, 0);
    const sum = wps.reduce(
      (acc, wp) => ({ x: acc.x + wp.x, y: acc.y + wp.y }),
      { x: 0, y: 0 }
    );
    return new THREE.Vector3(
      sum.x / wps.length,
      0.5,
      -sum.y / wps.length
    );
  }, [zone.waypoints]);

  return (
    <Billboard position={centroid}>
      <Text
        fontSize={isActive ? 0.3 : 0.25}
        color={color}
        anchorX="center"
        anchorY="middle"
        outlineWidth={0.03}
        outlineColor="#000000"
      >
        {zone.name}
        {isActive && " (Active)"}
      </Text>
    </Billboard>
  );
}
