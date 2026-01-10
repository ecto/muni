"use client";

import { useRef, useEffect, Suspense } from "react";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { scaleReferences, type ScaleReference } from "./ModelCatalog";

interface ScaleRefModelProps {
  scaleRef: ScaleReference;
  modelMaxX: number;
  modelCenterZ: number;
  index: number;
}

function ScaleRefModel({ scaleRef, modelMaxX, modelCenterZ, index }: ScaleRefModelProps) {
  const { scene } = useGLTF(scaleRef.path);
  const groupRef = useRef<THREE.Group>(null);

  useEffect(() => {
    if (groupRef.current) {
      // Apply scale
      if (scaleRef.scale !== 1) {
        groupRef.current.scale.setScalar(scaleRef.scale);
      }

      // Apply rotation
      if (scaleRef.rotateY) {
        groupRef.current.rotation.y = scaleRef.rotateY;
      }

      // Position relative to main model
      groupRef.current.position.set(0, 0, 0);
      groupRef.current.updateMatrixWorld(true);

      const refBox = new THREE.Box3().setFromObject(groupRef.current);
      const refSize = refBox.getSize(new THREE.Vector3());

      groupRef.current.position.x = modelMaxX + 150 + refSize.x / 2 + index * 400;
      groupRef.current.position.z = modelCenterZ;

      groupRef.current.updateMatrixWorld(true);
      const newBox = new THREE.Box3().setFromObject(groupRef.current);
      groupRef.current.position.y -= newBox.min.y;
    }
  }, [scene, scaleRef.scale, scaleRef.rotateY, modelMaxX, modelCenterZ, index]);

  return (
    <group ref={groupRef}>
      <primitive object={scene.clone()} />
    </group>
  );
}

interface ScaleReferencesProps {
  activeScales: Set<string>;
  modelBounds: { size: THREE.Vector3; center: THREE.Vector3 } | null;
}

export function ScaleReferences({ activeScales, modelBounds }: ScaleReferencesProps) {
  if (!modelBounds || activeScales.size === 0) return null;

  const modelMaxX = modelBounds.center.x + modelBounds.size.x / 2;
  const modelCenterZ = modelBounds.center.z;

  const activeRefs = scaleReferences.filter((ref) => activeScales.has(ref.id));

  return (
    <group>
      {activeRefs.map((ref, index) => (
        <Suspense key={ref.id} fallback={null}>
          <ScaleRefModel
            scaleRef={ref}
            modelMaxX={modelMaxX}
            modelCenterZ={modelCenterZ}
            index={index}
          />
        </Suspense>
      ))}
    </group>
  );
}
