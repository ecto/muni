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
  getCameraFrame,
  getFrameVersion,
  type CameraFrame,
} from "@/lib/videoFrameStore";

// Shared geometry for all camera projections (4:3 aspect ratio, 2m tall)
const CAMERA_PLANE_GEOMETRY = new PlaneGeometry(2 * (640 / 480), 2);

/**
 * Single camera projection plane.
 * Reads frame URL directly from the mutable store to avoid React re-renders.
 */
function CameraProjection({
  cameraId,
  position
}: {
  cameraId: number;
  position: [number, number, number];
}) {
  const materialRef = useRef<MeshBasicMaterial | null>(null);
  const textureRef = useRef<Texture | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [hasTexture, setHasTexture] = useState(false);
  // Track last loaded URL to avoid reloading same frame
  const lastLoadedUrlRef = useRef<string | null>(null);
  // Track pending image load to avoid race conditions
  const pendingLoadRef = useRef<string | null>(null);

  // Poll store for new frames in useFrame (runs at 60fps, no React re-renders)
  useFrame(() => {
    const frame = getCameraFrame(cameraId);
    const frameUrl = frame?.url ?? null;

    // Skip if no URL, already loaded, or load in progress
    if (!frameUrl || frameUrl === lastLoadedUrlRef.current || frameUrl === pendingLoadRef.current) {
      return;
    }

    // Start loading new frame
    pendingLoadRef.current = frameUrl;
    const img = new Image();
    img.onload = () => {
      // Verify this is still the frame we want (not stale)
      if (pendingLoadRef.current !== frameUrl) return;

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
      textureRef.current.needsUpdate = true;

      // Update material
      if (materialRef.current) {
        materialRef.current.map = textureRef.current;
        materialRef.current.needsUpdate = true;
      }

      lastLoadedUrlRef.current = frameUrl;
      pendingLoadRef.current = null;
      if (!hasTexture) setHasTexture(true);
    };
    img.onerror = () => {
      pendingLoadRef.current = null;
    };
    img.src = frameUrl;
  });

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
  // Only track camera IDs - frames are read directly from store by CameraProjection
  const [cameraIds, setCameraIds] = useState<number[]>([]);
  // Track previous camera IDs to avoid unnecessary state updates
  const prevCameraIdsRef = useRef<string>("");

  // Memoize camera positions to prevent unnecessary remounts
  // Each camera gets a stable position tuple based on its index and total count
  const positions = useMemo(() => {
    const total = cameraIds.length;
    if (total === 0) return [];
    return cameraIds.map((_, index) => {
      if (total === 1) return [0, 1.5, -3] as [number, number, number];
      // Spread cameras horizontally in front of rover (planes are ~2.67m wide)
      const spacing = 3;
      const offset = ((total - 1) / 2) * spacing;
      const x = index * spacing - offset;
      return [x, 1.5, -3] as [number, number, number];
    });
  }, [cameraIds]);

  // Check for new cameras from mutable store (runs even when not rendering)
  // Only triggers re-render when the set of camera IDs changes, not on every frame
  useFrame(() => {
    const currentVersion = getFrameVersion();
    if (currentVersion !== lastFrameVersionRef.current) {
      lastFrameVersionRef.current = currentVersion;
      // Get current camera IDs from store
      const frames = getAllCameraFrames();
      const ids = Array.from(frames.keys()).sort((a, b) => a - b);
      const idsKey = ids.join(",");
      // Only update state if camera set changed (not on every frame)
      if (idsKey !== prevCameraIdsRef.current) {
        if (ids.length > 0 && cameraIds.length === 0) {
          console.log(`[EquirectangularSky] First frames received: ${ids.length} cameras`);
        }
        prevCameraIdsRef.current = idsKey;
        setCameraIds(ids);
      }
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

  if (!videoConnected || cameraIds.length === 0) {
    return null;
  }

  return (
    <group ref={groupRef}>
      {cameraIds.map((cameraId, index) => (
        <CameraProjection
          key={cameraId}
          cameraId={cameraId}
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
  const videoConnected = useConsoleStore((s) => s.videoConnected);
  const videoFps = useConsoleStore((s) => s.videoFps);

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
  const videoConnected = useConsoleStore((s) => s.videoConnected);
  const videoFps = useConsoleStore((s) => s.videoFps);
  const cameraCount = useConsoleStore((s) => s.cameraCount);
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
