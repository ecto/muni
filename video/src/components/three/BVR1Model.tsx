import { useRef, useEffect } from "react";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";

interface BVR1ModelProps {
  positionY?: number;
  opacity?: number;
}

export function BVR1Model({ positionY = 0, opacity = 1 }: BVR1ModelProps) {
  const groupRef = useRef<THREE.Group>(null);
  const { scene } = useGLTF("/models/bvr1_assembly_realistic.glb");

  useEffect(() => {
    if (groupRef.current) {
      // Rotate from Z-up (CAD) to Y-up (Three.js)
      groupRef.current.rotation.x = -Math.PI / 2;

      // Update matrices
      groupRef.current.updateMatrixWorld(true);

      // Get bounding box
      const box = new THREE.Box3().setFromObject(groupRef.current);

      // Position on ground (base at y=0)
      groupRef.current.position.y = -box.min.y + positionY;
    }
  }, [scene, positionY]);

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
useGLTF.preload("/models/bvr1_assembly_realistic.glb");
