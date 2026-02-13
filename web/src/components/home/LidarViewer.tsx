"use client";

import { Suspense } from "react";
import { Canvas } from "@react-three/fiber";
import { SafetyZones, LidarBeam, LidarPointCloud } from "@/components/three/LidarViz";
import { Grid } from "@/components/three/Grid";
import { useIsMobile, useReducedMotion } from "./hooks/useIsMobile";

export function LidarViewer() {
  const isMobile = useIsMobile();
  const reducedMotion = useReducedMotion();

  if (isMobile) {
    return (
      <div className="lidar-fallback">
        <div className="lidar-fallback-zones">
          <div className="lidar-zone lidar-zone-stop" />
          <div className="lidar-zone lidar-zone-slow" />
          <div className="lidar-zone lidar-zone-clear" />
        </div>
        <div className="lidar-fallback-labels">
          <span className="lidar-label-stop">STOP 0-1.5m</span>
          <span className="lidar-label-slow">SLOW 1.5-3.5m</span>
          <span className="lidar-label-clear">CLEAR 3.5-6m</span>
        </div>
      </div>
    );
  }

  return (
    <div className="lidar-canvas-wrap">
      <Canvas
        camera={{ fov: 45, near: 0.1, far: 5000, position: [0, 800, 0] }}
        style={{ width: "100%", height: "100%" }}
      >
        <ambientLight intensity={0.3} />
        <Grid opacity={0.15} fadeDistance={800} />
        <Suspense fallback={null}>
          <SafetyZones />
          {!reducedMotion && (
            <>
              <LidarBeam />
              <LidarPointCloud />
            </>
          )}
        </Suspense>
      </Canvas>
    </div>
  );
}
