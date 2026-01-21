import { useRef, useEffect, useMemo, useState } from "react";
import { useConsoleStore } from "@/store";
import { CaretDown, CaretRight } from "@phosphor-icons/react";
import {
  PlaneGeometry,
  MeshBasicMaterial,
  CanvasTexture,
  SRGBColorSpace,
  DoubleSide,
  type Mesh,
  type Texture,
} from "three";

/**
 * Renders the camera feed as a video plane positioned in front of the rover.
 *
 * The video is displayed on a plane that acts as a "windshield" view,
 * showing what the rover's cameras see ahead.
 */
export function EquirectangularSky() {
  const meshRef = useRef<Mesh>(null);
  const materialRef = useRef<MeshBasicMaterial>(null);
  const textureRef = useRef<Texture | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const ctxRef = useRef<CanvasRenderingContext2D | null>(null);
  const currentUrlRef = useRef<string | null>(null);
  const firstFrameLoadedRef = useRef(false);

  const { videoFrame, videoConnected } = useConsoleStore();

  // Create plane geometry sized for 4:3 aspect ratio
  const geometry = useMemo(() => {
    const aspectRatio = 640 / 480;
    const height = 8;
    const width = height * aspectRatio;
    return new PlaneGeometry(width, height);
  }, []);

  // Initialize canvas for texture
  useEffect(() => {
    canvasRef.current = document.createElement("canvas");
    canvasRef.current.width = 640;
    canvasRef.current.height = 480;
    ctxRef.current = canvasRef.current.getContext("2d");

    return () => {
      if (textureRef.current) {
        textureRef.current.dispose();
      }
    };
  }, []);

  // Update texture when video frame changes
  useEffect(() => {
    if (!materialRef.current || !canvasRef.current || !ctxRef.current) return;
    if (!videoFrame || !videoConnected) {
      if (materialRef.current) {
        materialRef.current.map = null;
        materialRef.current.color.setHex(0x333333);
        materialRef.current.needsUpdate = true;
      }
      firstFrameLoadedRef.current = false;
      return;
    }

    const canvas = canvasRef.current;
    const ctx = ctxRef.current;
    const frameUrl = videoFrame;

    // Track which URL we're currently loading
    currentUrlRef.current = frameUrl;

    // Use Image element for loading - handles blob URLs correctly
    const img = new Image();
    img.onload = () => {
      // Check if this is still the current frame (not stale)
      if (currentUrlRef.current !== frameUrl) {
        return;
      }

      // Resize canvas if needed
      if (canvas.width !== img.width || canvas.height !== img.height) {
        canvas.width = img.width;
        canvas.height = img.height;
      }

      // Draw to canvas
      ctx.drawImage(img, 0, 0);

      // Create or update texture
      if (!textureRef.current) {
        textureRef.current = new CanvasTexture(canvas);
        textureRef.current.colorSpace = SRGBColorSpace;
        if (materialRef.current) {
          materialRef.current.map = textureRef.current;
          materialRef.current.needsUpdate = true;
        }
      } else {
        textureRef.current.needsUpdate = true;
      }

      if (!firstFrameLoadedRef.current) {
        console.log(`[VideoPlane] First texture loaded: ${canvas.width}x${canvas.height}`);
        firstFrameLoadedRef.current = true;
      }
    };

    img.onerror = () => {
      // Only log if this is still the current URL (not a stale revoked URL)
      if (currentUrlRef.current === frameUrl && !firstFrameLoadedRef.current) {
        console.error("[VideoPlane] Failed to load frame");
      }
    };

    // Start loading
    img.src = frameUrl;
  }, [videoFrame, videoConnected]);

  return (
    <mesh ref={meshRef} geometry={geometry} position={[0, 4, -10]}>
      <meshBasicMaterial
        ref={materialRef}
        side={DoubleSide}
        color={0xffffff}
        fog={false}
      />
    </mesh>
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
 * Single camera feed display.
 */
function CameraFeed({ cameraId, url }: { cameraId: number; url: string | null }) {
  return (
    <div className="relative w-40 h-30 bg-black">
      {url ? (
        <img
          src={url}
          alt={`Camera ${cameraId}`}
          className="w-full h-full object-contain"
        />
      ) : (
        <div className="absolute inset-0 flex items-center justify-center text-muted-foreground text-xs">
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

      {/* Camera feeds grid */}
      {!collapsed && (
        <div className="flex flex-wrap gap-1 p-1">
          {cameras.length > 0 ? (
            cameras.map(([cameraId, frame]) => (
              <CameraFeed key={cameraId} cameraId={cameraId} url={frame.url} />
            ))
          ) : (
            <div className="w-40 h-30 flex items-center justify-center text-muted-foreground text-sm bg-black">
              {videoConnected ? 'Waiting for frames...' : 'No cameras'}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
