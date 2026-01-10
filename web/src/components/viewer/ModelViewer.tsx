"use client";

import { Suspense, useRef, useEffect, useState, useCallback } from "react";
import { Canvas, useThree } from "@react-three/fiber";
import { OrbitControls, useGLTF } from "@react-three/drei";
import * as THREE from "three";
import { InfiniteGrid } from "./InfiniteGrid";
import { Hotspots } from "./Hotspots";
import { ScaleReferences } from "./ScaleReferences";
import { useTheme } from "./hooks/useTheme";
import type { ModelInfo, ComponentInfo } from "./ModelCatalog";

interface ModelProps {
  modelInfo: ModelInfo;
  onLoaded?: (size: THREE.Vector3, center: THREE.Vector3) => void;
  wireframe?: boolean;
}

function Model({ modelInfo, onLoaded, wireframe = false }: ModelProps) {
  const { scene } = useGLTF(modelInfo.path);
  const groupRef = useRef<THREE.Group>(null);

  useEffect(() => {
    if (groupRef.current) {
      // Rotate from Z-up to Y-up
      groupRef.current.rotation.x = -Math.PI / 2;

      // Update matrices
      groupRef.current.updateMatrixWorld(true);

      // Get bounding box
      const box = new THREE.Box3().setFromObject(groupRef.current);

      // Position on ground
      groupRef.current.position.y = -box.min.y;

      // Update again and get final dimensions
      groupRef.current.updateMatrixWorld(true);
      const finalBox = new THREE.Box3().setFromObject(groupRef.current);
      const size = finalBox.getSize(new THREE.Vector3());
      const center = finalBox.getCenter(new THREE.Vector3());

      if (onLoaded) {
        onLoaded(size, center);
      }
    }
  }, [scene, onLoaded]);

  useEffect(() => {
    scene.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
        materials.forEach((m) => {
          if (m && "wireframe" in m) {
            (m as THREE.MeshStandardMaterial).wireframe = wireframe;
          }
        });
      }
    });
  }, [scene, wireframe]);

  return (
    <group ref={groupRef}>
      <primitive object={scene.clone()} />
    </group>
  );
}

interface SceneProps {
  modelInfo: ModelInfo | null;
  wireframe: boolean;
  labelsVisible: boolean;
  selectedComponent: ComponentInfo | null;
  activeScales: Set<string>;
  modelBounds: { size: THREE.Vector3; center: THREE.Vector3 } | null;
  onModelLoaded?: (size: THREE.Vector3, center: THREE.Vector3) => void;
  onSelectComponent: (component: ComponentInfo) => void;
}

function Scene({
  modelInfo,
  wireframe,
  labelsVisible,
  selectedComponent,
  activeScales,
  modelBounds,
  onModelLoaded,
  onSelectComponent,
}: SceneProps) {
  const theme = useTheme();
  const { scene } = useThree();

  useEffect(() => {
    scene.background = new THREE.Color(theme.background);
  }, [scene, theme.background]);

  const showHotspots = modelInfo?.hasLabels && labelsVisible;

  return (
    <>
      {/* Lighting */}
      <ambientLight intensity={0.4} />
      <directionalLight position={[200, 300, 200]} intensity={1.0} />
      <directionalLight position={[-200, 100, -100]} intensity={0.5} />
      <directionalLight position={[0, -100, -200]} intensity={0.3} color="#ff6600" />

      {/* Grid */}
      <InfiniteGrid />

      {/* Model */}
      {modelInfo && (
        <Suspense fallback={null}>
          <Model modelInfo={modelInfo} onLoaded={onModelLoaded} wireframe={wireframe} />
        </Suspense>
      )}

      {/* Hotspots */}
      <Hotspots
        visible={showHotspots || false}
        selectedComponent={selectedComponent}
        modelBounds={modelBounds}
        onSelectComponent={onSelectComponent}
      />

      {/* Scale References */}
      <ScaleReferences activeScales={activeScales} modelBounds={modelBounds} />

      {/* Controls */}
      <OrbitControls enableDamping dampingFactor={0.05} />
    </>
  );
}

interface CameraControllerProps {
  target: THREE.Vector3 | null;
  position: THREE.Vector3 | null;
}

function CameraController({ target, position }: CameraControllerProps) {
  const { camera } = useThree();

  useEffect(() => {
    if (position) {
      camera.position.copy(position);
    }
  }, [camera, position]);

  return null;
}

export interface ViewerState {
  modelInfo: ModelInfo | null;
  selectedComponent: ComponentInfo | null;
  labelsVisible: boolean;
  wireframe: boolean;
  modelSize: THREE.Vector3 | null;
  modelCenter: THREE.Vector3 | null;
}

interface ModelViewerProps {
  state: ViewerState;
  activeScales: Set<string>;
  onModelLoaded: (size: THREE.Vector3, center: THREE.Vector3) => void;
  onSelectComponent: (component: ComponentInfo) => void;
}

export function ModelViewer({
  state,
  activeScales,
  onModelLoaded,
  onSelectComponent,
}: ModelViewerProps) {
  const [cameraTarget, setCameraTarget] = useState<THREE.Vector3 | null>(null);
  const [cameraPosition, setCameraPosition] = useState<THREE.Vector3 | null>(null);

  const handleModelLoaded = useCallback(
    (size: THREE.Vector3, center: THREE.Vector3) => {
      const maxDim = Math.max(size.x, size.y, size.z);
      setCameraPosition(
        new THREE.Vector3(center.x + maxDim, center.y + maxDim * 0.6, center.z + maxDim)
      );
      setCameraTarget(center);
      onModelLoaded(size, center);
    },
    [onModelLoaded]
  );

  const modelBounds =
    state.modelSize && state.modelCenter
      ? { size: state.modelSize, center: state.modelCenter }
      : null;

  return (
    <Canvas
      camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
      style={{ width: "100%", height: "100%" }}
    >
      <CameraController target={cameraTarget} position={cameraPosition} />
      <Scene
        modelInfo={state.modelInfo}
        wireframe={state.wireframe}
        labelsVisible={state.labelsVisible}
        selectedComponent={state.selectedComponent}
        activeScales={activeScales}
        modelBounds={modelBounds}
        onModelLoaded={handleModelLoaded}
        onSelectComponent={onSelectComponent}
      />
    </Canvas>
  );
}
