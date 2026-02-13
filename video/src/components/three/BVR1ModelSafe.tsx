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

export function BVR1ModelSafe({ positionY = 0, opacity = 1 }: BVR1ModelSafeProps) {
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

  useEffect(() => {
    if (!gltf) return;
    gltf.scene.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
        materials.forEach((m) => {
          if (m && "opacity" in m) {
            (m as THREE.MeshStandardMaterial).transparent = opacity < 1;
            (m as THREE.MeshStandardMaterial).opacity = opacity;
          }
        });
      }
    });
  }, [gltf, opacity]);

  const clonedScene = useMemo(() => gltf?.scene.clone(), [gltf]);

  if (failed || !clonedScene) return null;

  return (
    <group ref={groupRef}>
      <primitive object={clonedScene} />
    </group>
  );
}
