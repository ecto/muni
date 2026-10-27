"use client";

import { Suspense, useRef, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { useIsMobile, useReducedMotion } from "./hooks/useIsMobile";

const ACCENT = "#ff6600";
const ACCENT_DIM = "#ff660030";

function WireframeRover() {
  const { scene } = useGLTF("/models/bvr1_assembly_realistic.glb");
  const groupRef = useRef<THREE.Group>(null);
  const rotationRef = useRef<THREE.Group>(null);
  const reducedMotion = useReducedMotion();

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
    scene.traverse((child) => {
      if (child instanceof THREE.Mesh && child.material) {
        const mat = new THREE.MeshBasicMaterial({
          color: new THREE.Color(ACCENT),
          wireframe: true,
          transparent: true,
          opacity: 0.15,
        });
        child.material = mat;
      }
    });
  }, [scene]);

  useFrame((_, delta) => {
    if (rotationRef.current && !reducedMotion) {
      rotationRef.current.rotation.y += delta * 0.08;
    }
  });

  return (
    <group ref={rotationRef}>
      <group ref={groupRef}>
        <primitive object={scene} />
      </group>
    </group>
  );
}

function HeroLighting() {
  return (
    <>
      <ambientLight intensity={0.3} color={ACCENT} />
      <directionalLight position={[200, 300, 200]} intensity={0.4} color={ACCENT} />
      <directionalLight position={[-200, 100, -100]} intensity={0.2} color="#ffffff" />
    </>
  );
}

export function HeroRover() {
  const isMobile = useIsMobile(768);

  if (isMobile) return null;

  return (
    <div className="landing-hero-rover">
      <Canvas
        camera={{ fov: 30, near: 0.1, far: 5000, position: [900, 350, 900] }}
        style={{ width: "100%", height: "100%" }}
        gl={{ antialias: true, alpha: true }}
      >
        <HeroLighting />
        <Suspense fallback={null}>
          <WireframeRover />
        </Suspense>
      </Canvas>
    </div>
  );
}
