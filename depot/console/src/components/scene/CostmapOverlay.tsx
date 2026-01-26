import { useRef, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { getCostmapData, getCostmapVersion } from "@/lib/costmapStore";

/**
 * Renders the rover's costmap as a semi-transparent floor overlay.
 *
 * The costmap is a 2D occupancy grid in world coordinates. We render it
 * as a textured plane on the ground (Y=0.01 to avoid z-fighting).
 *
 * Color mapping:
 * - FREE (0): fully transparent
 * - Low cost (1-127): transparent (don't clutter)
 * - INSCRIBED (253): yellow, semi-transparent
 * - LETHAL (254): red, semi-transparent
 * - NO_INFORMATION (255): transparent
 */
export function CostmapOverlay() {
  const costmapEnabled = useConsoleStore((s) => s.costmapEnabled);

  const meshRef = useRef<THREE.Mesh>(null);
  const lastVersionRef = useRef<number>(0);
  const textureRef = useRef<THREE.DataTexture | null>(null);
  const geometryRef = useRef<THREE.PlaneGeometry | null>(null);
  const materialRef = useRef<THREE.MeshBasicMaterial | null>(null);

  // Texture data buffer — reused across updates
  const texDataRef = useRef<Uint8Array | null>(null);
  const texDimsRef = useRef({ width: 0, height: 0 });

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      textureRef.current?.dispose();
      textureRef.current = null;
      geometryRef.current?.dispose();
      geometryRef.current = null;
      materialRef.current?.dispose();
      materialRef.current = null;
    };
  }, []);

  // Reset when disabled
  useEffect(() => {
    if (!costmapEnabled && meshRef.current) {
      meshRef.current.visible = false;
      lastVersionRef.current = 0;
    }
  }, [costmapEnabled]);

  useFrame(() => {
    if (!costmapEnabled || !meshRef.current) return;

    const version = getCostmapVersion();
    if (version === lastVersionRef.current) return;
    lastVersionRef.current = version;

    const { cells, width, height, resolution, originX, originY } = getCostmapData();
    if (!cells || width === 0 || height === 0) {
      meshRef.current.visible = false;
      return;
    }

    // Lazily create the material
    if (!materialRef.current) {
      materialRef.current = new THREE.MeshBasicMaterial({
        transparent: true,
        depthWrite: false,
        side: THREE.DoubleSide,
        opacity: 1.0,
      });
      meshRef.current.material = materialRef.current;
    }

    // Reallocate texture buffer if dimensions changed
    if (texDimsRef.current.width !== width || texDimsRef.current.height !== height) {
      texDimsRef.current = { width, height };
      texDataRef.current = new Uint8Array(width * height * 4);

      // Recreate texture
      textureRef.current?.dispose();
      const tex = new THREE.DataTexture(
        texDataRef.current,
        width,
        height,
        THREE.RGBAFormat,
      );
      tex.minFilter = THREE.NearestFilter;
      tex.magFilter = THREE.NearestFilter;
      tex.flipY = false;
      // Rotate texture 90° CW so gx (world X) maps to Three.js -Z (forward)
      // and gy (world Y) maps to Three.js -X (left)
      tex.center.set(0.5, 0.5);
      tex.rotation = Math.PI / 2;
      textureRef.current = tex;
      materialRef.current.map = tex;
      materialRef.current.needsUpdate = true;

      // Recreate geometry to match grid world size
      geometryRef.current?.dispose();
      const worldWidth = width * resolution;
      const worldHeight = height * resolution;
      // Swap args: plane local X → Three.js X (world Y extent),
      // plane local Y → Three.js -Z (world X extent)
      const geo = new THREE.PlaneGeometry(worldHeight, worldWidth);
      geometryRef.current = geo;
      meshRef.current.geometry = geo;
    }

    const texData = texDataRef.current!;

    // Fill texture: map cost values to RGBA
    for (let i = 0; i < cells.length; i++) {
      const cost = cells[i];
      const offset = i * 4;

      if (cost >= 254) {
        // LETHAL — red
        texData[offset] = 239;
        texData[offset + 1] = 68;
        texData[offset + 2] = 68;
        texData[offset + 3] = 160;
      } else if (cost >= 253) {
        // INSCRIBED — yellow
        texData[offset] = 250;
        texData[offset + 1] = 204;
        texData[offset + 2] = 21;
        texData[offset + 3] = 120;
      } else if (cost >= 128) {
        // Medium cost — orange, fading alpha
        const t = (cost - 128) / (253 - 128);
        texData[offset] = 251;
        texData[offset + 1] = 146;
        texData[offset + 2] = 60;
        texData[offset + 3] = Math.round(t * 100);
      } else {
        // FREE / low cost — transparent
        texData[offset] = 0;
        texData[offset + 1] = 0;
        texData[offset + 2] = 0;
        texData[offset + 3] = 0;
      }
    }

    if (textureRef.current) {
      textureRef.current.needsUpdate = true;
    }

    // Position plane at costmap world origin (center of the grid)
    // Costmap origin is bottom-left corner in world frame
    const worldWidth = width * resolution;
    const worldHeight = height * resolution;
    // Three.js: X = -world_Y, Z = -world_X (same convention as rover model)
    meshRef.current.position.x = -(originY + worldHeight / 2);
    meshRef.current.position.y = 0.01; // slightly above ground to avoid z-fighting
    meshRef.current.position.z = -(originX + worldWidth / 2);

    meshRef.current.visible = true;
  });

  return (
    <mesh
      ref={meshRef}
      rotation={[-Math.PI / 2, 0, 0]}
      visible={false}
    >
      {/* Placeholder geometry — replaced in useFrame when data arrives */}
      <planeGeometry args={[1, 1]} />
      <meshBasicMaterial transparent opacity={0} />
    </mesh>
  );
}
