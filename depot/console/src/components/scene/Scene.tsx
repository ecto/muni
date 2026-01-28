import { useCallback, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import type { RootState } from "@react-three/fiber";
import { Ground } from "./Ground";
import { RoverModel } from "./RoverModel";
import { RoverTrail } from "./RoverTrail";
import { MapOverlay } from "./MapOverlay";
import { FleetMarkers } from "./FleetMarkers";
import { CameraController } from "./CameraController";
import { EquirectangularSky } from "./EquirectangularSky";
import { World3D } from "./World3D";
import { PointCloud3D } from "./PointCloud3D";
import { CostmapOverlay } from "./CostmapOverlay";
import { ObstacleOverlay } from "./ObstacleOverlay";
import { IntentionMarker } from "./IntentionMarker";
import { GoalMarker } from "./GoalMarker";
import { CollisionGuardArc } from "./CollisionGuardArc";
import { useConsoleStore } from "@/store";
import { useMapData } from "@/hooks/useMapData";
import { threeToEnu } from "@/lib/coordinates";
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
  /** Callback when the ground is right-clicked to set a navigation goal (ENU x, y) */
  onGoalClick?: (x: number, y: number) => void;
}

export function Scene({ mode = "teleop", zones: propZones, activeTasks, onZoneClick, onGoalClick }: SceneProps) {
  const videoConnected = useConsoleStore((s) => s.videoConnected);
  const selectedRoverId = useConsoleStore((s) => s.selectedRoverId);
  const splatEnabled = useConsoleStore((s) => s.splatEnabled);
  const { zones: fetchedZones } = useMapData();

  // Track WebGL context state for recovery
  const glRef = useRef<WebGLRenderingContext | null>(null);
  const [contextLost, setContextLost] = useState(false);

  // Use provided zones or fetch them
  const zones = propZones ?? fetchedZones;

  // In dispatch mode, show fleet markers instead of single rover
  const showFleetMarkers = mode === "dispatch";
  const showSingleRover = mode === "teleop";

  // Performance optimization: when video is active in teleop mode, disable
  // expensive rendering (shadows, World3D) since they're not visible anyway
  const videoActive = videoConnected && showSingleRover;

  // Reduce shadow map size when video not active (still useful for dispatch mode)
  const shadowMapSize = videoActive ? 512 : 1024;

  // Handle WebGL context events
  const handleCreated = useCallback((state: RootState) => {
    const gl = state.gl.getContext();
    glRef.current = gl as WebGLRenderingContext;

    const canvas = state.gl.domElement;

    const handleContextLost = (event: Event) => {
      event.preventDefault();
      console.warn("[Scene] WebGL context lost");
      setContextLost(true);
    };

    const handleContextRestored = () => {
      console.info("[Scene] WebGL context restored");
      setContextLost(false);
    };

    canvas.addEventListener("webglcontextlost", handleContextLost);
    canvas.addEventListener("webglcontextrestored", handleContextRestored);

    // Return cleanup function via onPointerMissed trick is not available,
    // but R3F handles canvas cleanup automatically
  }, []);

  // Show fallback when context is lost
  if (contextLost) {
    return (
      <div className="absolute inset-0 flex items-center justify-center bg-stone-900 text-stone-400">
        <div className="text-center">
          <p className="text-lg font-medium">WebGL context lost</p>
          <p className="text-sm">Recovering...</p>
        </div>
      </div>
    );
  }

  return (
    <Canvas
      shadows={!videoActive}
      camera={{ position: [3, 2.5, 3], fov: 60 }}
      gl={{
        antialias: true,
        powerPreference: "high-performance",
      }}
      // Limit pixel ratio to reduce GPU memory on high-DPI displays
      dpr={[1, 2]}
      onCreated={handleCreated}
      className="absolute inset-0"
      onContextMenu={(e) => e.preventDefault()}
    >
      {/* 360 video skybox (when connected) - only in teleop mode */}
      {showSingleRover && <EquirectangularSky />}

      {/* Lighting: reduce when video is active for better visibility */}
      <ambientLight intensity={videoActive ? 0.5 : 0.3} />
      <directionalLight
        position={[10, 20, 10]}
        intensity={videoActive ? 0.8 : 1.5}
        castShadow={!videoActive}
        shadow-mapSize={[shadowMapSize, shadowMapSize]}
        shadow-camera-far={50}
        shadow-camera-left={-20}
        shadow-camera-right={20}
        shadow-camera-top={20}
        shadow-camera-bottom={-20}
      />

      {/* Invisible ground plane for click-to-navigate (right-click) */}
      {showSingleRover && onGoalClick && (
        <mesh
          rotation={[-Math.PI / 2, 0, 0]}
          position={[0, 0, 0]}
          onContextMenu={(e) => {
            e.stopPropagation();
            const enu = threeToEnu({ x: e.point.x, y: e.point.y, z: e.point.z });
            onGoalClick(enu.x, enu.y);
          }}
        >
          <planeGeometry args={[200, 200]} />
          <meshBasicMaterial visible={false} />
        </mesh>
      )}

      {/* Scene elements */}
      <Ground />
      {/* Gaussian splat 3D map - can be toggled off for performance */}
      {splatEnabled && <World3D />}
      <MapOverlay zones={zones} activeTasks={activeTasks} onZoneClick={onZoneClick} />

      {/* Rover visualization depends on mode */}
      {showSingleRover && (
        <>
          <RoverTrail />
          <RoverModel />
          <IntentionMarker />
          <GoalMarker />
          <CollisionGuardArc />
          <PointCloud3D />
          <CostmapOverlay />
          <ObstacleOverlay />
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
