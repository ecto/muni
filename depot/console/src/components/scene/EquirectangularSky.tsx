import { useRef, useEffect, useMemo, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { useConsoleStore } from "@/store";
import {
  PlaneGeometry,
  MeshBasicMaterial,
  CanvasTexture,
  SRGBColorSpace,
  DoubleSide,
  Euler,
  Quaternion,
  Vector3,
  type Texture,
  type Group,
} from "three";
import { computeRenderPose, getInterpolationBuffer } from "@/lib/interpolation";
import {
  getAllCameraFrames,
  getCameraFrame,
  getCameraCalibration,
  getFrameVersion,
  type CameraFrame,
  type CameraCalibration,
} from "@/lib/videoFrameStore";

// ============================================================================
// Frustum Projection Math
// ============================================================================

/** Default calibration matching old CameraMount::default() (pre-1080p rovers). */
const DEFAULT_CALIBRATION: CameraCalibration = {
  width: 640,
  height: 480,
  position: [0.25, 0.0, 0.25],   // [forward, left, up]
  rotation: [0.0, -0.175, 0.0],  // [roll, pitch, yaw]
  fovH: 1.05,                     // ~60 degrees
  fovV: 0.79,                     // ~45 degrees
};

/** Projection distance from camera to frustum plane (meters). */
const PROJECTION_DISTANCE = 3.0;

/**
 * Compute frustum plane dimensions and placement from camera calibration.
 *
 * Coordinate mapping (physics -> Three.js):
 *   Physics: X=forward, Y=left, Z=up
 *   Three.js: X=right, Y=up, Z=backward
 *
 *   Three.js x = -physics_y  (left -> right)
 *   Three.js y =  physics_z  (up -> up)
 *   Three.js z = -physics_x  (forward -> into screen)
 */
function computeFrustumPlacement(cal: CameraCalibration) {
  const d = PROJECTION_DISTANCE;

  // Frustum plane size at projection distance
  const planeW = 2 * d * Math.tan(cal.fovH / 2);
  const planeH = 2 * d * Math.tan(cal.fovV / 2);

  // Mount position in Three.js coordinates
  const mountPos = new Vector3(
    -cal.position[1],  // -left = right
    cal.position[2],   // up
    -cal.position[0],  // -forward = backward
  );

  // Mount rotation: physics Euler [roll, pitch, yaw] -> Three.js Euler('YXZ')
  const mountEuler = new Euler(
    -cal.rotation[1],  // -pitch (physics Y-left axis -> Three.js -X axis)
    cal.rotation[2],   //  yaw   (physics Z-up axis -> Three.js Y axis)
    -cal.rotation[0],  // -roll  (physics X-fwd axis -> Three.js -Z axis)
    "YXZ",
  );
  const mountQuat = new Quaternion().setFromEuler(mountEuler);

  // Forward offset in camera frame (looking into -Z in Three.js)
  const forwardOffset = new Vector3(0, 0, -d).applyQuaternion(mountQuat);

  // Plane center = mount position + forward offset
  const planeCenter = mountPos.clone().add(forwardOffset);

  return { planeW, planeH, planeCenter, mountQuat };
}

// ============================================================================
// CameraProjection Component
// ============================================================================

/**
 * Single camera projection plane.
 * Reads frame URL directly from the mutable store to avoid React re-renders.
 * Computes its own geometry and position from camera calibration.
 */
function CameraProjection({ cameraId }: { cameraId: number }) {
  const materialRef = useRef<MeshBasicMaterial | null>(null);
  const textureRef = useRef<Texture | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const geometryRef = useRef<PlaneGeometry | null>(null);
  const [hasTexture, setHasTexture] = useState(false);
  const lastLoadedUrlRef = useRef<string | null>(null);
  const pendingLoadRef = useRef<string | null>(null);

  // Compute frustum placement from calibration (or fallback)
  const { planeW, planeH, planeCenter, mountQuat } = useMemo(() => {
    const cal = getCameraCalibration(cameraId) ?? DEFAULT_CALIBRATION;
    return computeFrustumPlacement(cal);
  }, [cameraId]);

  // Create geometry matching frustum size
  const geometry = useMemo(() => {
    if (geometryRef.current) {
      geometryRef.current.dispose();
    }
    const geo = new PlaneGeometry(planeW, planeH);
    geometryRef.current = geo;
    return geo;
  }, [planeW, planeH]);

  // Poll store for new frames in useFrame (runs at 60fps, no React re-renders)
  useFrame(() => {
    const frame = getCameraFrame(cameraId);
    const frameUrl = frame?.url ?? null;

    if (!frameUrl || frameUrl === lastLoadedUrlRef.current || frameUrl === pendingLoadRef.current) {
      return;
    }

    pendingLoadRef.current = frameUrl;
    const img = new Image();
    img.onload = () => {
      if (pendingLoadRef.current !== frameUrl) return;

      if (!canvasRef.current) {
        canvasRef.current = document.createElement("canvas");
      }
      const canvas = canvasRef.current;
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.drawImage(img, 0, 0);

      if (!textureRef.current) {
        const newTexture = new CanvasTexture(canvas);
        newTexture.colorSpace = SRGBColorSpace;
        textureRef.current = newTexture;
      }
      textureRef.current.needsUpdate = true;

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

  // Cleanup on unmount
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
      if (geometryRef.current) {
        geometryRef.current.dispose();
        geometryRef.current = null;
      }
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
    <mesh
      geometry={geometry}
      position={[planeCenter.x, planeCenter.y, planeCenter.z]}
      quaternion={mountQuat}
    >
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

// ============================================================================
// EquirectangularSky (Camera Feed Container)
// ============================================================================

/**
 * Renders camera feeds as frustum-projected planes in front of the rover.
 * Creates a separate projection for each camera to avoid flickering.
 * The group follows the rover's position so planes are always in front.
 *
 * Reads video frames from the mutable store (videoFrameStore.ts) to avoid
 * React re-renders at 15fps per camera.
 */
export function EquirectangularSky() {
  const videoConnected = useConsoleStore((state) => state.videoConnected);
  const groupRef = useRef<Group>(null);

  const lastFrameVersionRef = useRef(0);
  const [cameraIds, setCameraIds] = useState<number[]>([]);
  const prevCameraIdsRef = useRef<string>("");

  // Check for new cameras from mutable store
  useFrame(() => {
    const currentVersion = getFrameVersion();
    if (currentVersion !== lastFrameVersionRef.current) {
      lastFrameVersionRef.current = currentVersion;
      const frames = getAllCameraFrames();
      const ids = Array.from(frames.keys()).sort((a, b) => a - b);
      const idsKey = ids.join(",");
      if (idsKey !== prevCameraIdsRef.current) {
        if (ids.length > 0 && cameraIds.length === 0) {
          console.log(`[EquirectangularSky] First frames received: ${ids.length} cameras`);
        }
        prevCameraIdsRef.current = idsKey;
        setCameraIds(ids);
      }
    }

    if (!groupRef.current) return;

    const result = computeRenderPose(getInterpolationBuffer(), performance.now(), 0);

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
      {cameraIds.map((cameraId) => (
        <CameraProjection key={cameraId} cameraId={cameraId} />
      ))}
    </group>
  );
}

// ============================================================================
// UI Components
// ============================================================================

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
 * Camera feed section for the teleop sidebar.
 * Displays raw video frames from all cameras as HTML images.
 * No card chrome — designed to sit inside a parent container.
 *
 * Polls the mutable video frame store at 15Hz (matching frame rate) to
 * update the display without excessive React re-renders.
 */
export function CameraFeedSection() {
  const videoConnected = useConsoleStore((s) => s.videoConnected);
  const videoFps = useConsoleStore((s) => s.videoFps);
  const cameraCount = useConsoleStore((s) => s.cameraCount);
  const [cameras, setCameras] = useState<Array<[number, CameraFrame]>>([]);
  const lastFrameVersionRef = useRef(0);

  // Poll the mutable store at 15Hz
  useEffect(() => {
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
  }, []);

  return (
    <div className="p-2 space-y-1">
      {/* Info row */}
      <div className="flex items-center justify-between text-[11px] text-muted-foreground">
        <span>Cameras ({cameraCount})</span>
        <span className={`font-mono ${videoConnected ? 'text-green-500' : 'text-red-500'}`}>
          {videoConnected ? `${videoFps} FPS` : 'OFFLINE'}
        </span>
      </div>

      {/* Camera feeds - vertical stack */}
      <div className="flex flex-col gap-1">
        {cameras.length > 0 ? (
          cameras.map(([cameraId, frame]) => (
            <CameraFeed key={cameraId} cameraId={cameraId} url={frame.url} />
          ))
        ) : (
          <div className="w-full h-24 flex items-center justify-center text-muted-foreground text-xs bg-black rounded">
            {videoConnected ? 'Waiting...' : 'No cameras'}
          </div>
        )}
      </div>
    </div>
  );
}
