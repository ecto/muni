import { useRef, useEffect, useMemo, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { useConsoleStore } from "@/store";
import {
  PlaneGeometry,
  MeshBasicMaterial,
  VideoTexture,
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
  getAllVideoStreams,
  getCameraCalibration,
  getFrameVersion,
  getVideoElement,
  setVideoElement,
  removeVideoElement,
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
 * Shares the sidebar's actively-decoding <video> element for VideoTexture.
 * Safari won't decode frames for offscreen/hidden video elements, so we
 * reuse the visible CameraFeed element registered in videoFrameStore.
 */
function CameraProjection({ cameraId }: { cameraId: number }) {
  const materialRef = useRef<MeshBasicMaterial | null>(null);
  const textureRef = useRef<Texture | null>(null);
  const geometryRef = useRef<PlaneGeometry | null>(null);
  const [hasTexture, setHasTexture] = useState(false);
  const boundVideoRef = useRef<HTMLVideoElement | null>(null);

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

  // Poll for the sidebar's video element and create VideoTexture from it
  useFrame(() => {
    const video = getVideoElement(cameraId);
    if (!video) return;

    // If the video element changed (e.g. remount), recreate texture
    if (boundVideoRef.current !== video) {
      if (textureRef.current) {
        textureRef.current.dispose();
        textureRef.current = null;
      }
      boundVideoRef.current = video;
    }

    // Create VideoTexture once video has data
    if (video.readyState >= video.HAVE_CURRENT_DATA && !textureRef.current) {
      const tex = new VideoTexture(video);
      tex.colorSpace = SRGBColorSpace;
      textureRef.current = tex;

      if (materialRef.current) {
        materialRef.current.map = tex;
        materialRef.current.needsUpdate = true;
      }
      if (!hasTexture) setHasTexture(true);
    }

    // Force texture update every frame
    if (textureRef.current) {
      textureRef.current.needsUpdate = true;
    }
  });

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (textureRef.current) {
        textureRef.current.dispose();
        textureRef.current = null;
      }
      if (geometryRef.current) {
        geometryRef.current.dispose();
        geometryRef.current = null;
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

  // Check for new cameras from video stream store
  useFrame(() => {
    const currentVersion = getFrameVersion();
    if (currentVersion !== lastFrameVersionRef.current) {
      lastFrameVersionRef.current = currentVersion;
      const streams = getAllVideoStreams();
      const ids = Array.from(streams.keys()).sort((a, b) => a - b);
      const idsKey = ids.join(",");
      if (idsKey !== prevCameraIdsRef.current) {
        if (ids.length > 0 && cameraIds.length === 0) {
          console.log(`[EquirectangularSky] Video streams received: ${ids.length} cameras`);
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
 * Single camera feed display - renders video stream at native resolution, scaled down.
 * Registers its <video> element in the store so CameraProjection can share it
 * for VideoTexture (Safari won't decode offscreen video elements).
 */
function CameraFeed({ cameraId, stream }: { cameraId: number; stream: MediaStream | null }) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const setVideoFps = useConsoleStore((s) => s.setVideoFps);

  useEffect(() => {
    if (videoRef.current && stream) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  // Register video element for 3D projection sharing
  useEffect(() => {
    if (videoRef.current) {
      setVideoElement(cameraId, videoRef.current);
    }
    return () => {
      removeVideoElement(cameraId);
    };
  }, [cameraId]);

  // FPS counting via requestVideoFrameCallback
  useEffect(() => {
    const video = videoRef.current;
    if (!video || !("requestVideoFrameCallback" in video)) return;

    let frameCount = 0;
    let lastFpsTime = performance.now();
    let callbackId: number;

    const onFrame = () => {
      frameCount++;
      const now = performance.now();
      if (now - lastFpsTime >= 1000) {
        const fps = Math.round((frameCount / (now - lastFpsTime)) * 1000);
        setVideoFps(fps);
        frameCount = 0;
        lastFpsTime = now;
      }
      callbackId = video.requestVideoFrameCallback(onFrame);
    };
    callbackId = video.requestVideoFrameCallback(onFrame);

    return () => {
      video.cancelVideoFrameCallback(callbackId);
    };
  }, [stream, setVideoFps]);

  return (
    <div className="relative bg-black">
      {stream ? (
        <video
          ref={videoRef}
          autoPlay
          muted
          playsInline
          className="block max-h-24"
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
  const [cameras, setCameras] = useState<Array<[number, MediaStream]>>([]);
  const lastFrameVersionRef = useRef(0);

  // Poll the video stream store at 15Hz
  useEffect(() => {
    const interval = setInterval(() => {
      const currentVersion = getFrameVersion();
      if (currentVersion !== lastFrameVersionRef.current) {
        lastFrameVersionRef.current = currentVersion;
        const streams = getAllVideoStreams();
        const sorted = Array.from(streams.entries()).sort((a, b) => a[0] - b[0]);
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
          cameras.map(([cameraId, stream]) => (
            <CameraFeed key={cameraId} cameraId={cameraId} stream={stream} />
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
