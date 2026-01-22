import { Canvas } from "@react-three/fiber";
import { Ground } from "./Ground";
import { RoverModel } from "./RoverModel";
import { RoverTrail } from "./RoverTrail";
import { MapOverlay } from "./MapOverlay";
import { FleetMarkers } from "./FleetMarkers";
import { CameraController } from "./CameraController";
import { EquirectangularSky } from "./EquirectangularSky";
import { World3D } from "./World3D";
import { PointCloud3D } from "./PointCloud3D";
import { useConsoleStore } from "@/store";
import { useMapData } from "@/hooks/useMapData";
import type { Zone, Task } from "@/lib/types";

export type SceneMode = "teleop" | "dispatch";

interface SceneProps {
  /** Mode determines which elements to show: teleop (default) or dispatch */
  mode?: SceneMode;
  /** Zones to render (if not provided, fetched via useMapData) */
  zones?: Zone[];
  /** Active tasks for highlighting waypoints */
  activeTasks?: Task[];
  /** Callback when a zone is clicked */
  onZoneClick?: (zoneId: string) => void;
}

export function Scene({ mode = "teleop", zones: propZones, activeTasks, onZoneClick }: SceneProps) {
  const { videoConnected, selectedRoverId } = useConsoleStore();
  const { zones: fetchedZones } = useMapData();

  // Use provided zones or fetch them
  const zones = propZones ?? fetchedZones;

  // In dispatch mode, show fleet markers instead of single rover
  const showFleetMarkers = mode === "dispatch";
  const showSingleRover = mode === "teleop";

  return (
    <Canvas
      shadows
      camera={{ position: [3, 2.5, 3], fov: 60 }}
      gl={{ antialias: true }}
      className="absolute inset-0"
    >
      {/* 360 video skybox (when connected) - only in teleop mode */}
      {showSingleRover && <EquirectangularSky />}

      {/* Lighting: reduce when video is active for better visibility */}
      <ambientLight intensity={videoConnected && showSingleRover ? 0.5 : 0.3} />
      <directionalLight
        position={[10, 20, 10]}
        intensity={videoConnected && showSingleRover ? 0.8 : 1.5}
        castShadow
        shadow-mapSize={[2048, 2048]}
        shadow-camera-far={50}
        shadow-camera-left={-20}
        shadow-camera-right={20}
        shadow-camera-top={20}
        shadow-camera-bottom={-20}
      />

      {/* Scene elements */}
      <Ground />
      <World3D />
      <MapOverlay zones={zones} activeTasks={activeTasks} onZoneClick={onZoneClick} />

      {/* Rover visualization depends on mode */}
      {showSingleRover && (
        <>
          <RoverTrail />
          <RoverModel />
          <PointCloud3D />
        </>
      )}
      {showFleetMarkers && (
        <>
          <FleetMarkers
            selectedRoverId={selectedRoverId}
            activeTaskRoverId={activeTasks?.[0]?.roverId}
          />
          {/* Show selected rover with full model if there is one */}
          {selectedRoverId && <RoverModel />}
          {selectedRoverId && <RoverTrail />}
        </>
      )}

      <CameraController />

      {/* Fog for depth: disabled when video is active */}
      {!(videoConnected && showSingleRover) && <fog attach="fog" args={["#0c0a09", 20, 80]} />}
    </Canvas>
  );
}
