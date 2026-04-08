"use client";

import { useRef, useMemo, useState, useEffect } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { useReducedMotion, useDarkMode } from "./hooks/useIsMobile";
import { useInView } from "./hooks/useInView";

const EDGE_COLOR = 0xff6600;
const MAT_PROPS = {
  color: "#1a1a1a",
  roughness: 0.85,
  metalness: 0.15,
  transparent: true,
  opacity: 0.5,
};

function RoverModel({ enableRotation }: { enableRotation: boolean }) {
  const { scene } = useGLTF("/models/bvr1_assembly.glb");
  const groupRef = useRef<THREE.Group>(null);
  const rotationRef = useRef<THREE.Group>(null);

  const processedScene = useMemo(() => {
    const clone = scene.clone(true);
    clone.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        mesh.material = new THREE.MeshStandardMaterial(MAT_PROPS);
        const edgeGeo = new THREE.EdgesGeometry(mesh.geometry, 20);
        const edgeMat = new THREE.LineBasicMaterial({
          color: EDGE_COLOR,
          transparent: true,
          opacity: 0.8,
        });
        mesh.add(new THREE.LineSegments(edgeGeo, edgeMat));
      }
    });
    return clone;
  }, [scene]);

  useEffect(() => {
    if (!groupRef.current) return;
    groupRef.current.rotation.x = -Math.PI / 2;
    groupRef.current.updateMatrixWorld(true);
    const box = new THREE.Box3().setFromObject(groupRef.current);
    const center = box.getCenter(new THREE.Vector3());
    groupRef.current.position.sub(center);
    groupRef.current.position.y += (box.max.y - box.min.y) * 0.1;
  }, [processedScene]);

  useFrame((_, delta) => {
    if (rotationRef.current && enableRotation) {
      rotationRef.current.rotation.y += delta * 0.1;
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
      <ambientLight intensity={0.25} />
      <directionalLight position={[5, 8, 5]} intensity={0.7} />
      <directionalLight position={[-3, 2, -5]} intensity={0.25} color="#334455" />
      <pointLight position={[0, 4, 0]} intensity={0.15} color="#ff6600" />
      <gridHelper
        args={[20, 40, "#ff6600", "#222222"]}
        position={[0, -1.5, 0]}
        material-transparent
        material-opacity={0.06}
      />
      <RoverModel enableRotation={enableRotation} />
    </>
  );
}

function ResponsiveCamera() {
  const { viewport, camera } = useThree();

  useEffect(() => {
    const aspect = viewport.width / viewport.height;
    const dist = aspect < 1 ? 550 : 420;
    camera.position.set(dist * 0.7, dist * 0.4, dist);
    camera.lookAt(0, 0, 0);
    camera.updateProjectionMatrix();
  }, [viewport.width, viewport.height, camera]);

  return null;
}

export function RoverViewer() {
  const [ref, inView] = useInView("400px");
  const prefersReducedMotion = useReducedMotion();
  const isDarkMode = useDarkMode();

  return (
    <div ref={ref} className="rover-viewer">
      {inView && (
        <Canvas
          camera={{ fov: 35, near: 1, far: 5000, position: [300, 180, 350] }}
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
