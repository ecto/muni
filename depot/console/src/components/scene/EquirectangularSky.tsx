import { useRef, useEffect, useMemo } from "react";
import { useFrame, useLoader } from "@react-three/fiber";
import { useConsoleStore } from "@/store";
import {
  BackSide,
  SphereGeometry,
  MeshBasicMaterial,
  TextureLoader,
  SRGBColorSpace,
  type Mesh,
  type Texture,
} from "three";

/**
 * Renders the 360 video feed from the Insta360 X4 as an environment skybox.
 *
 * The equirectangular image is mapped onto the inside of a large sphere,
 * creating an immersive view of the rover's surroundings.
 *
 * When no video is available, displays a neutral gray gradient.
 */
export function EquirectangularSky() {
  const meshRef = useRef<Mesh>(null);
  const materialRef = useRef<MeshBasicMaterial>(null);
  const textureRef = useRef<Texture | null>(null);

  const { videoFrame, videoConnected, renderPose } = useConsoleStore();

  // Create sphere geometry once (inside-out)
  const geometry = useMemo(() => {
    const geo = new SphereGeometry(500, 64, 32);
    // Flip UVs horizontally for correct orientation
    const uvs = geo.attributes.uv;
    for (let i = 0; i < uvs.count; i++) {
      uvs.setX(i, 1 - uvs.getX(i));
    }
    return geo;
  }, []);

  // Load placeholder texture for when video is disconnected
  const placeholderTexture = useLoader(
    TextureLoader,
    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
  );

  // Update texture when video frame changes
  useEffect(() => {
    if (!materialRef.current) return;

    if (videoFrame && videoConnected) {
      // Load new texture from blob URL
      const loader = new TextureLoader();
      loader.load(
        videoFrame,
        (texture) => {
          texture.colorSpace = SRGBColorSpace;

          // Dispose old texture
          if (textureRef.current && textureRef.current !== placeholderTexture) {
            textureRef.current.dispose();
          }

          textureRef.current = texture;
          materialRef.current!.map = texture;
          materialRef.current!.needsUpdate = true;
        },
        undefined,
        () => {
          // Failed to load video frame texture: silently ignore
        }
      );
    } else {
      // Use placeholder
      if (textureRef.current && textureRef.current !== placeholderTexture) {
        textureRef.current.dispose();
      }
      textureRef.current = null;
      materialRef.current.map = null;
      materialRef.current.color.setHex(0x1a1a1a);
      materialRef.current.needsUpdate = true;
    }
  }, [videoFrame, videoConnected, placeholderTexture]);

  // Rotate sky sphere opposite to rover orientation so the world appears fixed
  useFrame(() => {
    if (!meshRef.current) return;

    // The sky rotates opposite to the rover's heading
    // This creates the effect of the rover moving through a fixed world
    meshRef.current.rotation.y = -renderPose.theta;
  });

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (textureRef.current && textureRef.current !== placeholderTexture) {
        textureRef.current.dispose();
      }
    };
  }, [placeholderTexture]);

  return (
    <mesh ref={meshRef} geometry={geometry}>
      <meshBasicMaterial
        ref={materialRef}
        side={BackSide}
        color={videoConnected ? 0xffffff : 0x1a1a1a}
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
      360 {videoFps} FPS
    </div>
  );
}
