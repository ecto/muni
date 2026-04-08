"use client";

import { useRef, useState, useEffect } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { Edges, Line } from "@react-three/drei";
import { useReducedMotion, useDarkMode } from "./hooks/useIsMobile";
import { useInView } from "./hooks/useInView";

/* ─── Parametric bracket: a shape that screams "this was designed in CAD" ─── */

function CadBracket({ time }: { time: React.MutableRefObject<number> }) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame(() => {
    if (!groupRef.current) return;
    groupRef.current.rotation.y = time.current * 0.08;
    groupRef.current.rotation.x = Math.sin(time.current * 0.03) * 0.1 + 0.3;
  });

  const edgeColor = "#e855a0";
  const matProps = {
    color: "#1a1a1a",
    roughness: 0.85,
    metalness: 0.15,
    transparent: true,
    opacity: 0.6,
  };

  return (
    <group ref={groupRef} scale={1.4}>
      {/* Base plate */}
      <mesh position={[0, -0.6, 0]}>
        <boxGeometry args={[3, 0.2, 2]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>

      {/* Left wall */}
      <mesh position={[-1.3, 0.4, 0]}>
        <boxGeometry args={[0.2, 1.8, 2]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>

      {/* Right wall */}
      <mesh position={[1.3, 0.4, 0]}>
        <boxGeometry args={[0.2, 1.8, 2]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>

      {/* Top bridge */}
      <mesh position={[0, 1.2, 0]}>
        <boxGeometry args={[2.8, 0.2, 2]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>

      {/* Center bore */}
      <mesh position={[0, 0.3, 0]} rotation={[Math.PI / 2, 0, 0]}>
        <cylinderGeometry args={[0.55, 0.55, 2.01, 32]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>

      {/* Mounting holes */}
      {[
        [-0.8, -0.6, 0.6],
        [0.8, -0.6, 0.6],
        [-0.8, -0.6, -0.6],
        [0.8, -0.6, -0.6],
      ].map((pos, i) => (
        <mesh key={i} position={pos as [number, number, number]} rotation={[0, 0, 0]}>
          <cylinderGeometry args={[0.12, 0.12, 0.21, 16]} />
          <meshStandardMaterial {...matProps} />
          <Edges threshold={15} color={edgeColor} lineWidth={2} />
        </mesh>
      ))}

      {/* Gussets / ribs */}
      <mesh position={[-1.1, -0.1, 0]} rotation={[0, 0, Math.PI / 4]}>
        <boxGeometry args={[0.12, 0.8, 1.6]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>
      <mesh position={[1.1, -0.1, 0]} rotation={[0, 0, -Math.PI / 4]}>
        <boxGeometry args={[0.12, 0.8, 1.6]} />
        <meshStandardMaterial {...matProps} />
        <Edges threshold={15} color={edgeColor} lineWidth={2} />
      </mesh>
    </group>
  );
}

/* ─── Floating dimension lines — the CAD UI vibe ─── */

function DimensionLines({ time }: { time: React.MutableRefObject<number> }) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame(() => {
    if (!groupRef.current) return;
    groupRef.current.rotation.y = time.current * 0.08;
    groupRef.current.rotation.x = Math.sin(time.current * 0.03) * 0.1 + 0.3;
  });

  const dims: [number, number, number][][] = [
    // Width dimension (bottom)
    [[-1.5, -1.0, 1.1], [1.5, -1.0, 1.1]],
    // Left tick
    [[-1.5, -1.15, 1.1], [-1.5, -0.85, 1.1]],
    // Right tick
    [[1.5, -1.15, 1.1], [1.5, -0.85, 1.1]],
    // Height dimension (left)
    [[-1.8, -0.7, 1.1], [-1.8, 1.3, 1.1]],
    // Top tick
    [[-1.95, 1.3, 1.1], [-1.65, 1.3, 1.1]],
    // Bottom tick
    [[-1.95, -0.7, 1.1], [-1.65, -0.7, 1.1]],
  ];

  return (
    <group ref={groupRef}>
      {dims.map((pts, i) => (
        <Line key={i} points={pts} color="#e855a0" lineWidth={2} transparent opacity={0.35} />
      ))}
    </group>
  );
}

/* ─── Subtle grid floor ─── */

function GridFloor() {
  return (
    <gridHelper
      args={[20, 40, "#e855a0", "#222222"]}
      position={[0, -0.75, 0]}
      material-transparent
      material-opacity={0.08}
    />
  );
}

/* ─── Scene composition ─── */

function Scene({ enableRotation }: { enableRotation: boolean }) {
  const time = useRef(0);

  useFrame((_, delta) => {
    if (enableRotation) {
      time.current += delta;
    }
  });

  return (
    <>
      <ambientLight intensity={0.3} />
      <directionalLight position={[5, 8, 5]} intensity={0.8} />
      <directionalLight position={[-3, 2, -5]} intensity={0.3} color="#334455" />
      <pointLight position={[0, 3, 0]} intensity={0.2} color="#e855a0" />

      <CadBracket time={time} />
      <DimensionLines time={time} />
      <GridFloor />
    </>
  );
}

/* ─── Responsive camera ─── */

function ResponsiveCamera() {
  const { viewport } = useThree();
  const { camera } = useThree();

  useEffect(() => {
    const aspect = viewport.width / viewport.height;
    const dist = aspect < 1 ? 7 : 5.5;
    camera.position.set(dist * 0.8, dist * 0.5, dist);
    camera.lookAt(0, 0.2, 0);
    camera.updateProjectionMatrix();
  }, [viewport.width, viewport.height, camera]);

  return null;
}

/* ─── Export ─── */

export function VcadViewer() {
  const [ref, inView] = useInView("400px");
  const prefersReducedMotion = useReducedMotion();
  const isDarkMode = useDarkMode();

  return (
    <div ref={ref} className="vcad-viewer">
      {inView && (
        <Canvas
          camera={{ fov: 30, near: 0.1, far: 100, position: [4.5, 2.5, 5.5] }}
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
