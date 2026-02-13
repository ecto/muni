"use client";

import { Suspense, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { Grid } from "@/components/three/Grid";
import { Lighting } from "@/components/three/Lighting";
import { BVR1Model } from "@/components/three/BVR1Model";
import { SnowParticles, SnowSpray } from "@/components/three/SnowParticles";
import { DebrisParticles, VacuumVortex } from "@/components/three/DebrisParticles";
import { RainParticles, RainSplash } from "@/components/three/RainParticles";
import { PLATFORM_MODES, type ModeConfig } from "./platformModes";
import { useIsMobile, useReducedMotion } from "./hooks/useIsMobile";

function ParticlesForMode({ mode, paused }: { mode: ModeConfig["id"]; paused: boolean }) {
  if (paused) return null;
  switch (mode) {
    case "snow":
      return (
        <>
          <SnowParticles />
          <SnowSpray />
        </>
      );
    case "sweep":
      return (
        <>
          <DebrisParticles />
          <VacuumVortex />
        </>
      );
    case "wash":
      return (
        <>
          <RainParticles />
          <RainSplash />
        </>
      );
  }
}

export function PlatformViewer() {
  const [activeMode, setActiveMode] = useState<ModeConfig["id"]>("snow");
  const isMobile = useIsMobile();
  const reducedMotion = useReducedMotion();
  const config = PLATFORM_MODES.find((m) => m.id === activeMode)!;

  return (
    <div className="platform-viewer">
      {/* Mode tabs */}
      <div className="platform-tabs">
        {PLATFORM_MODES.map((mode) => (
          <button
            key={mode.id}
            className={`platform-tab ${mode.id === activeMode ? "platform-tab-active" : ""}`}
            onClick={() => setActiveMode(mode.id)}
          >
            {mode.label}
          </button>
        ))}
      </div>

      {/* 3D scene (desktop) or fallback (mobile) */}
      <div className="platform-canvas-wrap">
        {isMobile ? (
          <div className="platform-fallback">
            <img
              src="/images/bvr1.png"
              alt="BVR1 Rover"
              className="platform-fallback-img"
            />
          </div>
        ) : (
          <Canvas
            camera={{ fov: 35, near: 0.1, far: 5000, position: [800, 400, 800] }}
            style={{ width: "100%", height: "100%" }}
          >
            <fog attach="fog" args={[config.fogColor, 500, 3000]} />
            <Lighting intensity={config.lightIntensity} tint={config.tint} />
            <Grid opacity={0.3} />
            <Suspense fallback={null}>
              <BVR1Model />
            </Suspense>
            <ParticlesForMode mode={activeMode} paused={reducedMotion} />
          </Canvas>
        )}
      </div>

      {/* Stats row */}
      <div className="platform-stats">
        {config.stats.map((stat) => (
          <div key={stat.label} className="platform-stat">
            <div className="platform-stat-value">{stat.value}</div>
            <div className="platform-stat-label">{stat.label}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
