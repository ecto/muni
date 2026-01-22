import { useRef, useEffect, useMemo, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { useConsoleStore } from "@/store";
import { CaretDown, CaretRight } from "@phosphor-icons/react";
import {
  PlaneGeometry,
  MeshBasicMaterial,
  CanvasTexture,
  SRGBColorSpace,
  DoubleSide,
  type Texture,
  type Group,
} from "three";
import { computeRenderPose, getInterpolationBuffer } from "@/lib/interpolation";

/**
 * Single camera projection plane.
 */
function CameraProjection({
  frameUrl,
  position
}: {
  frameUrl: string | null;
  position: [number, number, number];
}) {
  const materialRef = useRef<MeshBasicMaterial>(null);
  const textureRef = useRef<Texture | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [hasTexture, setHasTexture] = useState(false);

  // Create plane geometry sized for 4:3 aspect ratio
  const geometry = useMemo(() => {
    const aspectRatio = 640 / 480;
    const height = 2;  // 2 meters tall
    const width = height * aspectRatio;
    return new PlaneGeometry(width, height);
  }, []);

  // Load frame and update texture
  useEffect(() => {
    if (!frameUrl) return;

    const img = new Image();
    img.onload = () => {
      // Create or reuse canvas
      if (!canvasRef.current) {
        canvasRef.current = document.createElement("canvas");
      }
      const canvas = canvasRef.current;
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.drawImage(img, 0, 0);

      // Create or update texture
      if (!textureRef.current) {
        const newTexture = new CanvasTexture(canvas);
        newTexture.colorSpace = SRGBColorSpace;
        textureRef.current = newTexture;
      }
      // Mark texture as needing update (canvas content changed)
      textureRef.current.needsUpdate = true;

      // Update material
      if (materialRef.current) {
        if (!materialRef.current.map) {
          materialRef.current.map = textureRef.current;
        }
        materialRef.current.needsUpdate = true;
      }

      setHasTexture(true);
    };
    img.src = frameUrl;
  }, [frameUrl]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (textureRef.current) {
        textureRef.current.dispose();
        textureRef.current = null;
      }
    };
  }, []);

  return (
    <mesh geometry={geometry} position={position}>
      <meshBasicMaterial
        ref={materialRef}
        color={hasTexture ? 0xffffff : 0xff0000}
        side={DoubleSide}
        fog={false}
        toneMapped={false}
      />
    </mesh>
  );
}

/**
 * Renders camera feeds as video planes positioned in front of the rover.
 * Creates a separate projection for each camera to avoid flickering.
 * The group follows the rover's position so planes are always in front.
 */
export function EquirectangularSky() {
  const cameraFrames = useConsoleStore((state) => state.cameraFrames);
  const videoConnected = useConsoleStore((state) => state.videoConnected);
  const groupRef = useRef<Group>(null);

  // Convert Map to sorted array - need to track size for reactivity
  const cameraCount = cameraFrames.size;
  const cameras = useMemo(() => {
    return Array.from(cameraFrames.entries()).sort((a, b) => a[0] - b[0]);
  }, [cameraFrames, cameraCount]);

  // Follow the rover's position using the same interpolation as RoverModel
  useFrame(() => {
    if (!groupRef.current) return;

    const result = computeRenderPose(getInterpolationBuffer(), performance.now(), 0);

    // Apply pose to group (same coordinate mapping as RoverModel)
    // Physics: X=forward, Y=left; Three.js: -Z=forward, -X=left
    groupRef.current.position.z = -result.pose.x;
    groupRef.current.position.x = -result.pose.y;
    groupRef.current.rotation.y = result.pose.theta;
  });

  if (!videoConnected || cameras.length === 0) {
    return null;
  }

  // Position cameras relative to the rover (local coordinates)
  // Planes are positioned in front of the rover in local space
  const getPosition = (index: number, total: number): [number, number, number] => {
    if (total === 1) return [0, 1.5, -3];
    // Spread cameras horizontally in front of rover (planes are ~2.67m wide)
    const spacing = 3;
    const offset = ((total - 1) / 2) * spacing;
    const x = index * spacing - offset;
    return [x, 1.5, -3];
  };

  return (
    <group ref={groupRef}>
      {cameras.map(([cameraId, frame], index) => (
        <CameraProjection
          key={cameraId}
          frameUrl={frame.url}
          position={getPosition(index, cameras.length)}
        />
      ))}
    </group>
  );
}

/**
 * Video status indicator overlay.
 * Shows connection status and FPS in the corner of the screen.
 */
export function VideoStatusBadge() {
  const { videoConnected, videoFps } = useConsoleStore();

  if (!videoConnected) {
    return (
      <div className="bg-destructive/80 text-destructive-foreground px-2 py-1 rounded text-xs font-mono">
        VIDEO OFFLINE
      </div>
    );
  }

  return (
    <div className="bg-muted/80 text-muted-foreground px-2 py-1 rounded text-xs font-mono">
      CAM {videoFps} FPS
    </div>
  );
}

/**
 * Single camera feed display - renders at native resolution, scaled down.
 */
function CameraFeed({ cameraId, url }: { cameraId: number; url: string | null }) {
  return (
    <div className="relative bg-black">
      {url ? (
        <img
          src={url}
          alt={`Camera ${cameraId}`}
          className="block max-h-24"
          style={{ imageRendering: "pixelated" }}
        />
      ) : (
        <div className="w-20 h-24 flex items-center justify-center text-muted-foreground text-xs">
          No signal
        </div>
      )}
      <div className="absolute bottom-0 left-0 px-1 bg-black/60 text-white text-xs">
        CAM {cameraId}
      </div>
    </div>
  );
}

/**
 * Picture-in-picture camera feed card.
 * Displays raw video frames from all cameras as HTML images for debugging
 * and as an always-visible camera view for operators.
 */
export function CameraFeedCard() {
  const { videoConnected, videoFps, cameraFrames, cameraCount } = useConsoleStore();
  const [collapsed, setCollapsed] = useState(false);

  // Convert Map to array for rendering
  const cameras = Array.from(cameraFrames.entries()).sort((a, b) => a[0] - b[0]);

  return (
    <div className="bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div
        className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          {collapsed ? <CaretRight className="h-3 w-3" /> : <CaretDown className="h-3 w-3" />}
          Cameras ({cameraCount})
        </span>
        <span className={`text-xs font-mono ${videoConnected ? 'text-green-500' : 'text-red-500'}`}>
          {videoConnected ? `${videoFps} FPS` : 'OFFLINE'}
        </span>
      </div>

      {/* Camera feeds - vertical stack */}
      {!collapsed && (
        <div className="flex flex-col gap-1 p-1">
          {cameras.length > 0 ? (
            cameras.map(([cameraId, frame]) => (
              <CameraFeed key={cameraId} cameraId={cameraId} url={frame.url} />
            ))
          ) : (
            <div className="w-20 h-24 flex items-center justify-center text-muted-foreground text-xs bg-black">
              {videoConnected ? 'Waiting...' : 'No cameras'}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
