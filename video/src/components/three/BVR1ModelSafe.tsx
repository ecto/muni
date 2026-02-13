// BVR1 model loader — GLB inlined as base64 data URL via webpack asset/inline
import { useRef, useEffect, useMemo, useState } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three-stdlib";

// Webpack asset/inline — embeds the GLB as a base64 data URL in the bundle
// @ts-expect-error webpack asset import
import glbDataUrl from "../../../public/models/bvr1_assembly_realistic.glb";

interface BVR1ModelSafeProps {
  positionY?: number;
  opacity?: number;
  wireframe?: boolean;
  tint?: string; // hex color to override all materials
}

// Module-level cache
let gltfCache: { scene: THREE.Group } | null = null;
let gltfPromise: Promise<{ scene: THREE.Group }> | null = null;

function loadModel(): Promise<{ scene: THREE.Group }> {
  if (gltfCache) return Promise.resolve(gltfCache);
  if (gltfPromise) return gltfPromise;

  gltfPromise = fetch(glbDataUrl as string)
    .then((r) => r.arrayBuffer())
    .then(
      (buf) =>
        new Promise<{ scene: THREE.Group }>((resolve, reject) => {
          new GLTFLoader().parse(buf, "", (gltf) => {
            gltfCache = gltf as { scene: THREE.Group };
            resolve(gltfCache);
          }, reject);
        }),
    );

  return gltfPromise;
}

export function BVR1ModelSafe({ positionY = 0, opacity = 1, wireframe = false, tint }: BVR1ModelSafeProps) {
  const groupRef = useRef<THREE.Group>(null);
  const [gltf, setGltf] = useState<{ scene: THREE.Group } | null>(gltfCache);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (gltf) return;
    loadModel().then(setGltf).catch(() => setFailed(true));
  }, [gltf]);

  useEffect(() => {
    if (!gltf || !groupRef.current) return;
    groupRef.current.rotation.x = -Math.PI / 2;
    groupRef.current.updateMatrixWorld(true);
    const box = new THREE.Box3().setFromObject(groupRef.current);
    groupRef.current.position.y = -box.min.y + positionY;
  }, [gltf, positionY]);

  const clonedScene = useMemo(() => {
    if (!gltf) return null;
    const clone = gltf.scene.clone(true);
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        // Clone materials so we don't mutate cached originals
        const srcMats = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
        const newMats = srcMats.map((m) => {
          const mat = m.clone();
          if (mat instanceof THREE.MeshStandardMaterial) {
            mat.wireframe = wireframe;
            mat.transparent = opacity < 1 || wireframe;
            mat.opacity = wireframe ? 0.85 : opacity;
            if (tint) {
              mat.color.set(tint);
              mat.emissive.set(tint);
              mat.emissiveIntensity = 0.15;
            }
          }
          return mat;
        });
        mesh.material = newMats.length === 1 ? newMats[0] : newMats;
      }
    });
    return clone;
  }, [gltf, opacity, wireframe, tint]);

  if (failed || !clonedScene) return null;

  return (
    <group ref={groupRef}>
      <primitive object={clonedScene} />
    </group>
  );
}
