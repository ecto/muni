"use client";

import { useRef, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";

interface BVR1ModelProps {
  rotationSpeed?: number;
  wireframe?: boolean;
  opacity?: number;
  tint?: string;
}

export function BVR1Model({
  rotationSpeed = 0.1,
  wireframe = false,
  opacity = 1,
  tint,
}: BVR1ModelProps) {
  const { scene } = useGLTF("/models/bvr1_assembly_realistic.glb");
  const groupRef = useRef<THREE.Group>(null);
  const rotationRef = useRef<THREE.Group>(null);

  useEffect(() => {
    if (!groupRef.current) return;
    // CAD Z-up to Three.js Y-up
    groupRef.current.rotation.x = -Math.PI / 2;
    groupRef.current.updateMatrixWorld(true);

    // Ground the model
    const box = new THREE.Box3().setFromObject(groupRef.current);
    groupRef.current.position.y = -box.min.y;
  }, [scene]);

  useEffect(() => {
    const clone = scene.clone(true);
    clone.traverse((child) => {
      if (child instanceof THREE.Mesh && child.material) {
        const mat = (child.material as THREE.MeshStandardMaterial).clone();
        if (wireframe) mat.wireframe = true;
        if (opacity < 1) {
          mat.transparent = true;
          mat.opacity = opacity;
        }
        if (tint) {
          mat.color = new THREE.Color(tint);
        }
        child.material = mat;
      }
    });
  }, [scene, wireframe, opacity, tint]);

  useFrame((_, delta) => {
    if (rotationRef.current) {
      rotationRef.current.rotation.y += delta * rotationSpeed;
    }
  });

  return (
    <group ref={rotationRef}>
      <group ref={groupRef}>
        <primitive object={scene.clone()} />
      </group>
    </group>
  );
}

useGLTF.preload("/models/bvr1_assembly_realistic.glb");
