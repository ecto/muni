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
import {
  getAllCameraFrames,
  getFrameVersion,
  type CameraFrame,
} from "@/lib/videoFrameStore";

// Shared geometry for all camera projections (4:3 aspect ratio, 2m tall)
const CAMERA_PLANE_GEOMETRY = new PlaneGeometry(2 * (640 / 480), 2);

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
  const materialRef = useRef<MeshBasicMaterial | null>(null);
  const textureRef = useRef<Texture | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [hasTexture, setHasTexture] = useState(false);
  // Track mount generation to force texture reload on remount
  const [mountId] = useState(() => Math.random());

  // Load frame and update texture
  // Include mountId to ensure effect runs on remount even if frameUrl unchanged
  useEffect(() => {
    if (!frameUrl) {
      console.log(`[CameraProjection] No frameUrl provided`);
      return;
    }

    const img = new Image();
    img.onerror = (e) => {
      console.error(`[CameraProjection] Failed to load image:`, e);
    };
    img.onload = () => {
      console.log(`[CameraProjection] Image loaded: ${img.width}x${img.height}`);

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
        console.log(`[CameraProjection] Created new texture`);
      }
      // Mark texture as needing update (canvas content changed)
      textureRef.current.needsUpdate = true;

      // Update material - always reattach texture in case material was recreated
      if (materialRef.current) {
        materialRef.current.map = textureRef.current;
        materialRef.current.needsUpdate = true;
      }

      setHasTexture(true);
    };
    img.src = frameUrl;
  }, [frameUrl, mountId]);

  // Cleanup on unmount - dispose texture, material, and canvas
  useEffect(() => {
    return () => {
      if (textureRef.current) {
        textureRef.current.dispose();
        textureRef.current = null;
      }
      if (materialRef.current) {
        materialRef.current.dispose();
        materialRef.current = null;
      }
      // Clear canvas to free memory
      if (canvasRef.current) {
        const ctx = canvasRef.current.getContext("2d");
        if (ctx) {
          ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
        }
        canvasRef.current.width = 0;
        canvasRef.current.height = 0;
        canvasRef.current = null;
      }
    };
  }, []);

  return (
    <mesh geometry={CAMERA_PLANE_GEOMETRY} position={position}>
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
 *
 * Reads video frames from the mutable store (videoFrameStore.ts) to avoid
 * React re-renders at 15fps per camera.
 */
export function EquirectangularSky() {
  const videoConnected = useConsoleStore((state) => state.videoConnected);
  const groupRef = useRef<Group>(null);

  // Track frame version to detect changes from mutable store
  const lastFrameVersionRef = useRef(0);
  // Local state for cameras - updated when frame version changes
  const [cameras, setCameras] = useState<Array<[number, CameraFrame]>>([]);

  // Memoize camera positions to prevent unnecessary remounts
  // Each camera gets a stable position tuple based on its index and total count
  const positions = useMemo(() => {
    const total = cameras.length;
    if (total === 0) return [];
    return cameras.map((_, index) => {
      if (total === 1) return [0, 1.5, -3] as [number, number, number];
      // Spread cameras horizontally in front of rover (planes are ~2.67m wide)
      const spacing = 3;
      const offset = ((total - 1) / 2) * spacing;
      const x = index * spacing - offset;
      return [x, 1.5, -3] as [number, number, number];
    });
  }, [cameras]);

  // Check for frame updates from mutable store (runs even when not rendering)
  // This must run before the early return to ensure cameras state gets updated
  useFrame(() => {
    // Always check for frame updates, even if we're not rendering yet
    const currentVersion = getFrameVersion();
    if (currentVersion !== lastFrameVersionRef.current) {
      lastFrameVersionRef.current = currentVersion;
      // Read frames from mutable store and update local state
      const frames = getAllCameraFrames();
      const sorted = Array.from(frames.entries()).sort((a, b) => a[0] - b[0]);
      if (sorted.length > 0 && cameras.length === 0) {
        console.log(`[EquirectangularSky] First frames received: ${sorted.length} cameras, version=${currentVersion}`);
      }
      setCameras(sorted);
    }

    // Update group position if we have a ref
    if (!groupRef.current) return;

    const result = computeRenderPose(getInterpolationBuffer(), performance.now(), 0);

    // Apply pose to group (same coordinate mapping as RoverModel)
    // Physics: X=forward, Y=left; Three.js: -Z=forward, -X=left
    groupRef.current.position.z = -result.pose.x;
    groupRef.current.position.x = -result.pose.y;
    groupRef.current.rotation.y = result.pose.theta;
  });

  // Debug: log render state
  if (videoConnected && cameras.length === 0) {
    // Video is connected but we have no frames yet - check store directly
    const storeVersion = getFrameVersion();
    const storeFrames = getAllCameraFrames();
    if (storeFrames.size > 0) {
      console.warn(`[EquirectangularSky] Store has ${storeFrames.size} frames (v${storeVersion}) but cameras state is empty`);
    }
  }

  if (!videoConnected || cameras.length === 0) {
    return null;
  }

  console.log(`[EquirectangularSky] Rendering ${cameras.length} cameras`);

  return (
    <group ref={groupRef}>
      {cameras.map(([cameraId, frame], index) => (
        <CameraProjection
          key={cameraId}
          frameUrl={frame.url}
          position={positions[index]}
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
 *
 * Polls the mutable video frame store at 15Hz (matching frame rate) to
 * update the display without excessive React re-renders.
 */
export function CameraFeedCard() {
  const { videoConnected, videoFps, cameraCount } = useConsoleStore();
  const [collapsed, setCollapsed] = useState(false);
  const [cameras, setCameras] = useState<Array<[number, CameraFrame]>>([]);
  const lastFrameVersionRef = useRef(0);

  // Poll the mutable store at 15Hz when not collapsed
  useEffect(() => {
    if (collapsed) return;

    const interval = setInterval(() => {
      const currentVersion = getFrameVersion();
      if (currentVersion !== lastFrameVersionRef.current) {
        lastFrameVersionRef.current = currentVersion;
        const frames = getAllCameraFrames();
        const sorted = Array.from(frames.entries()).sort((a, b) => a[0] - b[0]);
        setCameras(sorted);
      }
    }, 67); // ~15Hz polling

    return () => clearInterval(interval);
  }, [collapsed]);

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
