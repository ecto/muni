"use client";

import { Suspense, useRef, useEffect, useState } from "react";
import { Canvas, useThree } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { useTheme } from "../viewer/hooks/useTheme";
import { useIsMobile } from "./hooks/useIsMobile";

export type HowItWorksStep = "connect" | "drive" | "safety";

interface StepCameraConfig {
  position: [number, number, number];
  target: [number, number, number];
}

const stepCameras: Record<HowItWorksStep, StepCameraConfig> = {
  connect: {
    // Isometric overview (top-down-ish)
    position: [600, 600, 600],
    target: [0, 200, 0],
  },
  drive: {
    // Side profile
    position: [900, 200, 0],
    target: [0, 200, 0],
  },
  safety: {
    // Front angle (emphasizing LiDAR)
    position: [0, 300, 800],
    target: [0, 250, 0],
  },
};

interface ModelProps {
  path: string;
}

function Model({ path }: ModelProps) {
  const { scene } = useGLTF(path);
  const groupRef = useRef<THREE.Group>(null);

  useEffect(() => {
    if (groupRef.current) {
      // Rotate from Z-up to Y-up
      groupRef.current.rotation.x = -Math.PI / 2;
      groupRef.current.updateMatrixWorld(true);

      // Position on ground
      const box = new THREE.Box3().setFromObject(groupRef.current);
      groupRef.current.position.y = -box.min.y;
    }
  }, [scene]);

  return (
    <group ref={groupRef}>
      <primitive object={scene.clone()} />
    </group>
  );
}

interface CameraControllerProps {
  step: HowItWorksStep;
}

function CameraController({ step }: CameraControllerProps) {
  const { camera } = useThree();
  const targetRef = useRef(new THREE.Vector3());

  useEffect(() => {
    const config = stepCameras[step];
    const targetPosition = new THREE.Vector3(...config.position);
    const targetLookAt = new THREE.Vector3(...config.target);

    // Animate camera position
    const startPosition = camera.position.clone();
    const startTime = performance.now();
    const duration = 800;

    function animate() {
      const elapsed = performance.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      // Ease out cubic
      const eased = 1 - Math.pow(1 - progress, 3);

      camera.position.lerpVectors(startPosition, targetPosition, eased);
      targetRef.current.lerpVectors(targetRef.current, targetLookAt, eased * 0.1);
      camera.lookAt(targetLookAt);

      if (progress < 1) {
        requestAnimationFrame(animate);
      }
    }

    animate();
  }, [step, camera]);

  return null;
}

interface SceneProps {
  step: HowItWorksStep;
}

function Scene({ step }: SceneProps) {
  const theme = useTheme();
  const { scene } = useThree();

  useEffect(() => {
    scene.background = new THREE.Color(theme.background);
  }, [scene, theme.background]);

  return (
    <>
      {/* Lighting */}
      <ambientLight intensity={0.5} />
      <directionalLight position={[200, 300, 200]} intensity={1.0} />
      <directionalLight position={[-200, 100, -100]} intensity={0.5} />
      <directionalLight
        position={[0, -100, -200]}
        intensity={0.3}
        color="#ff6600"
      />

      {/* Camera Controller */}
      <CameraController step={step} />

      {/* Model */}
      <Suspense fallback={null}>
        <Model path="/models/bvr1_assembly.glb" />
      </Suspense>
    </>
  );
}

function ViewerFallback() {
  return (
    <div className="how-it-works-fallback">
      <img
        src="/images/bvr1.png"
        alt="BVR1 Rover"
        className="how-it-works-fallback-image"
      />
    </div>
  );
}

interface HowItWorksViewerProps {
  step: HowItWorksStep;
}

export function HowItWorksViewer({ step }: HowItWorksViewerProps) {
  const [isClient, setIsClient] = useState(false);
  const isMobile = useIsMobile();

  useEffect(() => {
    setIsClient(true);
  }, []);

  // On mobile, show static fallback to save resources
  if (!isClient || isMobile) {
    return <ViewerFallback />;
  }

  const initialCamera = stepCameras[step];

  return (
    <div className="how-it-works-viewer">
      <Suspense fallback={<ViewerFallback />}>
        <Canvas
          camera={{
            fov: 35,
            near: 0.1,
            far: 5000,
            position: initialCamera.position,
          }}
          style={{ width: "100%", height: "100%" }}
        >
          <Scene step={step} />
        </Canvas>
      </Suspense>
    </div>
  );
}
