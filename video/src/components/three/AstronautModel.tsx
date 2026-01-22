import { useRef, useEffect } from "react";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";

interface AstronautModelProps {
  positionX?: number;
  positionZ?: number;
  opacity?: number;
}

export function AstronautModel({
  positionX = -400,
  positionZ = 0,
  opacity = 1,
}: AstronautModelProps) {
  const groupRef = useRef<THREE.Group>(null);
  const { scene } = useGLTF("/models/astronaut.glb");

  useEffect(() => {
    if (groupRef.current) {
      // The astronaut model needs to be scaled (scale: 1000 from ModelCatalog)
      groupRef.current.scale.set(1000, 1000, 1000);

      // Update matrices
      groupRef.current.updateMatrixWorld(true);

      // Get bounding box
      const box = new THREE.Box3().setFromObject(groupRef.current);

      // Position on ground
      groupRef.current.position.y = -box.min.y;
      groupRef.current.position.x = positionX;
      groupRef.current.position.z = positionZ;
    }
  }, [scene, positionX, positionZ]);

  // Apply opacity to all materials
  useEffect(() => {
    scene.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        const materials = Array.isArray(mesh.material)
          ? mesh.material
          : [mesh.material];
        materials.forEach((m) => {
          if (m && "opacity" in m) {
            (m as THREE.MeshStandardMaterial).transparent = opacity < 1;
            (m as THREE.MeshStandardMaterial).opacity = opacity;
          }
        });
      }
    });
  }, [scene, opacity]);

  return (
    <group ref={groupRef}>
      <primitive object={scene.clone()} />
    </group>
  );
}

// Preload the model
useGLTF.preload("/models/astronaut.glb");
