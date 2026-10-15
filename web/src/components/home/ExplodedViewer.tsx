"use client";

import { useRef, useMemo, useState, useEffect } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { useReducedMotion, useDarkMode } from "./hooks/useIsMobile";
import { useInView } from "./hooks/useInView";

const EDGE_COLOR = 0x22c55e;
const EXPLODE_FACTOR = 1.8;

function ExplodedModel({ enableRotation }: { enableRotation: boolean }) {
  const { scene } = useGLTF("/models/bvr1_assembly.glb");
  const groupRef = useRef<THREE.Group>(null);
  const rotationRef = useRef<THREE.Group>(null);

  const processedScene = useMemo(() => {
    const clone = scene.clone(true);

    clone.updateMatrixWorld(true);
    const center = new THREE.Vector3();
    let count = 0;
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const pos = new THREE.Vector3();
        child.getWorldPosition(pos);
        center.add(pos);
        count++;
      }
    });
    if (count > 0) center.divideScalar(count);

    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;

        mesh.material = new THREE.MeshStandardMaterial({
          color: "#1a1a1a",
          roughness: 0.85,
          metalness: 0.15,
          transparent: true,
          opacity: 0.4,
        });

        const edgeGeo = new THREE.EdgesGeometry(mesh.geometry, 20);
        const edgeMat = new THREE.LineBasicMaterial({
          color: EDGE_COLOR,
          transparent: true,
          opacity: 0.7,
        });
        mesh.add(new THREE.LineSegments(edgeGeo, edgeMat));

        const worldPos = new THREE.Vector3();
        mesh.getWorldPosition(worldPos);
        const dir = worldPos.clone().sub(center);
        if (dir.length() > 0.01) {
          mesh.position.add(
            dir.normalize().multiplyScalar(dir.length() * EXPLODE_FACTOR)
          );
        }
      }
    });

    return clone;
  }, [scene]);

  useEffect(() => {
    if (!groupRef.current) return;
    groupRef.current.rotation.x = -Math.PI / 2;
    groupRef.current.updateMatrixWorld(true);
    const box = new THREE.Box3().setFromObject(groupRef.current);
    const c = box.getCenter(new THREE.Vector3());
    groupRef.current.position.sub(c);
  }, [processedScene]);

  useFrame((_, delta) => {
    if (rotationRef.current && enableRotation) {
      rotationRef.current.rotation.y += delta * 0.06;
    }
  });

  return (
    <group ref={rotationRef}>
      <group ref={groupRef}>
        <primitive object={processedScene} />
      </group>
    </group>
  );
}

function Scene({ enableRotation }: { enableRotation: boolean }) {
  return (
    <>
      <ambientLight intensity={0.2} />
      <directionalLight position={[5, 8, 5]} intensity={0.6} />
      <directionalLight position={[-3, 2, -5]} intensity={0.2} color="#334455" />
      <pointLight position={[0, 4, 0]} intensity={0.15} color="#22c55e" />
      <ExplodedModel enableRotation={enableRotation} />
    </>
  );
}

function ResponsiveCamera() {
  const { viewport, camera } = useThree();

  useEffect(() => {
    const aspect = viewport.width / viewport.height;
    const dist = aspect < 1 ? 850 : 700;
    camera.position.set(dist * 0.7, dist * 0.5, dist);
    camera.lookAt(0, 20, 0);
    camera.updateProjectionMatrix();
  }, [viewport.width, viewport.height, camera]);

  return null;
}

export function ExplodedViewer() {
  const [ref, inView] = useInView("400px");
  const prefersReducedMotion = useReducedMotion();
  const isDarkMode = useDarkMode();

  return (
    <div ref={ref} className="exploded-viewer">
      {inView && (
        <Canvas
          camera={{ fov: 35, near: 1, far: 5000, position: [490, 350, 700] }}
          style={{ width: "100%", height: "100%" }}
          gl={{ antialias: true, alpha: true }}
          dpr={[1, 1.5]}
        >
          <color attach="background" args={[isDarkMode ? "#0a0a0b" : "#f8f8f8"]} />
          <ResponsiveCamera />
          <Scene enableRotation={!prefersReducedMotion} />
        </Canvas>
      )}
    </div>
  );
}
