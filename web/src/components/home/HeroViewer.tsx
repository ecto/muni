"use client";

import { Suspense, useRef, useEffect, useState } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { useGLTF, AsciiRenderer } from "@react-three/drei";
import * as THREE from "three";
import { useIsMobile, useReducedMotion } from "./hooks/useIsMobile";

interface AutoRotatingModelProps {
  path: string;
  rotationSpeed?: number;
  enableRotation?: boolean;
  onLoaded?: () => void;
}

function AutoRotatingModel({
  path,
  rotationSpeed = 0.15,
  enableRotation = true,
  onLoaded,
}: AutoRotatingModelProps) {
  const { scene } = useGLTF(path);
  const groupRef = useRef<THREE.Group>(null);
  const rotationRef = useRef<THREE.Group>(null);

  useEffect(() => {
    if (groupRef.current) {
      // Rotate from Z-up to Y-up (CAD convention)
      groupRef.current.rotation.x = -Math.PI / 2;
      groupRef.current.updateMatrixWorld(true);

      // Position on ground
      const box = new THREE.Box3().setFromObject(groupRef.current);
      groupRef.current.position.y = -box.min.y;
    }
    // Signal that model is loaded
    onLoaded?.();
  }, [scene, onLoaded]);

  useFrame((_, delta) => {
    if (rotationRef.current && enableRotation) {
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

interface SceneProps {
  enableRotation: boolean;
  onLoaded?: () => void;
}

function Scene({ enableRotation, onLoaded }: SceneProps) {
  return (
    <>
      {/* Lighting - higher contrast for ASCII */}
      <ambientLight intensity={0.3} />
      <directionalLight position={[300, 400, 200]} intensity={1.2} />
      <directionalLight position={[-200, 100, -100]} intensity={0.4} />
      <directionalLight
        position={[0, -100, -200]}
        intensity={0.2}
        color="#ff6600"
      />

      {/* Model */}
      <Suspense fallback={null}>
        <AutoRotatingModel
          path="/models/bvr1_assembly.glb"
          enableRotation={enableRotation}
          onLoaded={onLoaded}
        />
      </Suspense>

      {/* ASCII Effect */}
      <AsciiRenderer
        fgColor="#ff6600"
        bgColor="transparent"
        characters={" .'`^,:;Il!i><~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"}
        invert
        resolution={0.18}
      />
    </>
  );
}

export function HeroViewer() {
  const [isClient, setIsClient] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);
  const isMobile = useIsMobile();
  const prefersReducedMotion = useReducedMotion();

  useEffect(() => {
    setIsClient(true);
  }, []);

  // On mobile, show nothing (or could show static image)
  if (!isClient) {
    return <div className="hero-viewer hero-viewer-loading" />;
  }

  if (isMobile) {
    // Mobile: show static image
    return (
      <div className="hero-viewer">
        <img
          src="/images/bvr1.png"
          alt="BVR1 Production Rover"
          className="hero-fallback-image"
        />
      </div>
    );
  }

  const enableRotation = !prefersReducedMotion;

  return (
    <div className={`hero-viewer ${isLoaded ? "hero-viewer-ready" : "hero-viewer-loading"}`}>
      <Canvas
        camera={{
          fov: 35,
          near: 0.1,
          far: 5000,
          position: [800, 400, 800],
        }}
        style={{ width: "100%", height: "100%" }}
      >
        <Scene enableRotation={enableRotation} onLoaded={() => setIsLoaded(true)} />
      </Canvas>
    </div>
  );
}
