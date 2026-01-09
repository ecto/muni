import { useRef, Suspense, useState, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";
import type { Group } from "three";
import * as THREE from "three";
import { useConsoleStore } from "@/store";

const MODEL_PATH = "/models/bvr1_assembly.glb";

/**
 * Fallback primitive rover model.
 * Used when glTF model fails to load or while loading.
 */
function PrimitiveRover() {
  const wheelPositions: [number, number, number][] = [
    [-0.28, 0.082, 0.28], // FL
    [0.28, 0.082, 0.28], // FR
    [-0.28, 0.082, -0.28], // RL
    [0.28, 0.082, -0.28], // RR
  ];

  return (
    <>
      {/* Main body */}
      <mesh position={[0, 0.15, 0]} castShadow>
        <boxGeometry args={[0.6, 0.2, 0.6]} />
        <meshStandardMaterial color="#1d4ed8" metalness={0.3} roughness={0.8} />
      </mesh>

      {/* Front indicator */}
      <mesh position={[0.28, 0.15, 0]} castShadow>
        <boxGeometry args={[0.08, 0.08, 0.15]} />
        <meshStandardMaterial
          color="#dc2626"
          emissive="#dc2626"
          emissiveIntensity={0.5}
        />
      </mesh>

      {/* Top panel */}
      <mesh position={[0, 0.26, 0]} castShadow>
        <boxGeometry args={[0.4, 0.02, 0.4]} />
        <meshStandardMaterial color="#0a0a0a" metalness={0.9} roughness={0.1} />
      </mesh>

      {/* Wheels */}
      {wheelPositions.map((pos, i) => (
        <mesh key={i} position={pos} rotation={[0, 0, Math.PI / 2]} castShadow>
          <cylinderGeometry args={[0.082, 0.082, 0.06, 16]} />
          <meshStandardMaterial color="#0c0a09" roughness={0.9} />
        </mesh>
      ))}
    </>
  );
}

/**
 * glTF rover model with CAD geometry and PBR materials.
 */
function GLTFRover() {
  const { scene } = useGLTF(MODEL_PATH);
  const clonedScene = scene.clone();

  // Apply shadows to all meshes
  clonedScene.traverse((child) => {
    if (child instanceof THREE.Mesh) {
      child.castShadow = true;
      child.receiveShadow = true;
    }
  });

  // Scale from mm to meters and rotate to Y-up
  return (
    <primitive
      object={clonedScene}
      scale={0.001}
      rotation={[-Math.PI / 2, 0, 0]}
      position={[0, 0, 0]}
    />
  );
}

/**
 * Rover model with telemetry-driven pose interpolation.
 * Loads glTF model with fallback to primitives.
 */
export function RoverModel() {
  const groupRef = useRef<Group>(null);
  const { telemetry, renderPose, setRenderPose } = useConsoleStore();
  const [useGltf, setUseGltf] = useState(true);

  // Preload the model
  useEffect(() => {
    useGLTF.preload(MODEL_PATH);
  }, []);

  // Interpolate toward target pose each frame
  useFrame((_, delta) => {
    if (!groupRef.current) return;

    const target = telemetry.pose;
    const current = renderPose;

    // Smooth interpolation
    const lerpFactor = Math.min(1, delta * 10);

    const newX = current.x + (target.x - current.x) * lerpFactor;
    const newY = current.y + (target.y - current.y) * lerpFactor;

    // Angle interpolation (handle wraparound)
    let angleDiff = target.theta - current.theta;
    while (angleDiff > Math.PI) angleDiff -= Math.PI * 2;
    while (angleDiff < -Math.PI) angleDiff += Math.PI * 2;
    const newTheta = current.theta + angleDiff * lerpFactor;

    setRenderPose({ x: newX, y: newY, theta: newTheta });

    // Apply to mesh (map 2D physics coords to 3D)
    // physics.x -> Three.js X
    // physics.y -> Three.js -Z
    groupRef.current.position.x = newX;
    groupRef.current.position.z = -newY;
    groupRef.current.rotation.y = newTheta;
  });

  return (
    <group ref={groupRef} position={[0, 0, 0]}>
      <Suspense fallback={<PrimitiveRover />}>
        {useGltf ? (
          <ErrorBoundary onError={() => setUseGltf(false)}>
            <GLTFRover />
          </ErrorBoundary>
        ) : (
          <PrimitiveRover />
        )}
      </Suspense>
    </group>
  );
}

/**
 * Simple error boundary for catching glTF loading errors.
 */
function ErrorBoundary({
  children,
  onError
}: {
  children: React.ReactNode;
  onError: () => void;
}) {
  const [hasError, setHasError] = useState(false);

  useEffect(() => {
    if (hasError) {
      onError();
    }
  }, [hasError, onError]);

  if (hasError) {
    return null;
  }

  return (
    <ErrorBoundaryInner onError={() => setHasError(true)}>
      {children}
    </ErrorBoundaryInner>
  );
}

import { Component, type ReactNode } from "react";

class ErrorBoundaryInner extends Component<
  { children: ReactNode; onError: () => void },
  { hasError: boolean }
> {
  state = { hasError: false };

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  componentDidCatch() {
    this.props.onError();
  }

  render() {
    if (this.state.hasError) {
      return null;
    }
    return this.props.children;
  }
}
