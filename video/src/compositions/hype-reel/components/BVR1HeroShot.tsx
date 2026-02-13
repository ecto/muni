// BVR1 3D hero shot compositions — orbital camera rigs with subtle motion
import { ThreeCanvas } from "@remotion/three";
import { useCurrentFrame, interpolate } from "remotion";
import { Suspense } from "react";
import { BVR1ModelSafe } from "../../../components/three/BVR1ModelSafe";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { SnowParticles, SnowSpray } from "../../../components/three/SnowParticles";
import { RainParticles, RainSplash } from "../../../components/three/RainParticles";
import { DebrisParticles, VacuumVortex } from "../../../components/three/DebrisParticles";
import { SafetyZones, LidarBeam, LidarPointCloud } from "../../../components/three/LidarSafetyViz";
import { REEL_CONFIG } from "../config";
import { DARK_BG } from "../../../lib/brand";
import * as THREE from "three";

const TARGET = new THREE.Vector3(0, 250, 0);

const CANVAS_STYLE: React.CSSProperties = { position: "absolute", inset: 0 };

const WRAPPER_STYLE: React.CSSProperties = {
  position: "absolute",
  inset: 0,
  backgroundColor: DARK_BG,
};

interface HeroShotProps {
  durationInFrames: number;
}

interface SnowActionProps extends HeroShotProps {
  variant?: 1 | 2;
}

// ---------------------------------------------------------------------------
// Front-right 3/4 view with subtle Ken Burns drift
// ---------------------------------------------------------------------------

export function BVR1HeroFront({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [Math.PI / 4, Math.PI / 4 + 0.05],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={azimuth}
            elevation={Math.PI / 10}
            distance={800}
            fov={45}
          />
          <Lighting intensity={1.2} />
          <Grid opacity={0.3} />
          <BVR1ModelSafe />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Pure side view with gentle pan
// ---------------------------------------------------------------------------

export function BVR1HeroSide({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [Math.PI / 2 - 0.03, Math.PI / 2 + 0.03],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 40, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={azimuth}
            elevation={Math.PI / 12}
            distance={900}
            fov={40}
          />
          <Lighting intensity={1.2} />
          <Grid opacity={0.3} />
          <BVR1ModelSafe />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Rear 3/4 view with slow zoom-in
// ---------------------------------------------------------------------------

export function BVR1HeroRear({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const distance = interpolate(
    frame,
    [0, durationInFrames],
    [800, 700],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={Math.PI + Math.PI / 6}
            elevation={Math.PI / 6}
            distance={distance}
            fov={45}
          />
          <Lighting intensity={1.2} />
          <Grid opacity={0.3} />
          <BVR1ModelSafe />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Orbiting action shot — two variants for cut variety
// ---------------------------------------------------------------------------

export function BVR1SnowAction({
  durationInFrames,
  variant = 1,
}: SnowActionProps) {
  const frame = useCurrentFrame();

  const startAzimuth = variant === 1 ? 0 : Math.PI;

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [startAzimuth, startAzimuth + Math.PI / 4],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={azimuth}
            elevation={Math.PI / 16}
            distance={850}
            fov={45}
          />
          <Lighting intensity={1.0} />
          <fog attach="fog" args={["#1a1a2e", 600, 3500]} />
          <Grid opacity={0.3} />
          <BVR1ModelSafe />
          <SnowParticles count={500} speed={3} size={3} opacity={0.6} />
          <SnowSpray
            count={250}
            origin={[0, 80, -250]}
            velocity={variant === 1 ? 12 : 18}
            spread={variant === 1 ? 1.0 : 1.5}
            lifetime={25}
            size={4}
            opacity={0.5}
          />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Rain action shot — rover in the rain (power washer / all-weather ops)
// ---------------------------------------------------------------------------

export function BVR1RainAction({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [Math.PI * 0.8, Math.PI * 0.8 + Math.PI / 5],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={azimuth}
            elevation={Math.PI / 14}
            distance={900}
            fov={45}
          />
          <ambientLight intensity={0.25} color="#667788" />
          <directionalLight position={[3, 6, 2]} intensity={0.6} color="#8899aa" />
          <directionalLight position={[-2, 4, -3]} intensity={0.3} color="#6677aa" />
          <fog attach="fog" args={["#0d1117", 500, 3000]} />
          <Grid opacity={0.2} cellColor="#334455" sectionColor="#4488aa" />
          <BVR1ModelSafe />
          <RainParticles count={800} speed={14} size={2} opacity={0.35} angle={0.2} />
          <RainSplash count={120} opacity={0.25} />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Debris vacuum action shot — particles sucked toward rover
// ---------------------------------------------------------------------------

export function BVR1DebrisAction({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [Math.PI * 1.4, Math.PI * 1.4 + Math.PI / 5],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={azimuth}
            elevation={Math.PI / 12}
            distance={900}
            fov={45}
          />
          <ambientLight intensity={0.35} color="#aa8866" />
          <directionalLight position={[4, 6, 2]} intensity={0.8} color="#ddaa77" />
          <directionalLight position={[-3, 3, -2]} intensity={0.3} color="#886644" />
          <fog attach="fog" args={["#1a1510", 600, 3200]} />
          <Grid opacity={0.25} cellColor="#554433" sectionColor="#aa8855" />
          <BVR1ModelSafe />
          <DebrisParticles count={300} radius={800} size={6} opacity={0.7} />
          <VacuumVortex count={60} opacity={0.2} />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// LiDAR safety visualization — overhead sweep with color-coded zones
// ---------------------------------------------------------------------------

export function BVR1LidarSafety({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  // Slow orbit from overhead
  const azimuth = interpolate(
    frame,
    [0, durationInFrames],
    [0, Math.PI / 6],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={{ ...WRAPPER_STYLE, backgroundColor: "#050808" }}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [0, 1200, 0] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={new THREE.Vector3(0, 0, 0)}
            azimuth={azimuth}
            elevation={Math.PI / 3.2}
            distance={1000}
            fov={50}
          />
          <ambientLight intensity={0.15} color="#224433" />
          <directionalLight position={[2, 8, 3]} intensity={0.3} color="#446655" />
          <fog attach="fog" args={["#050808", 800, 3000]} />
          <SafetyZones />
          <LidarBeam />
          <LidarPointCloud />
          <BVR1ModelSafe />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Close-up hero for founder / CTA sections — shallow angle, slow zoom
// ---------------------------------------------------------------------------

export function BVR1FounderShot({ durationInFrames }: HeroShotProps) {
  const frame = useCurrentFrame();

  const distance = interpolate(
    frame,
    [0, durationInFrames],
    [700, 600],
    { extrapolateRight: "clamp" },
  );

  return (
    <div style={WRAPPER_STYLE}>
      <ThreeCanvas
        width={REEL_CONFIG.width}
        height={REEL_CONFIG.height}
        camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
        style={CANVAS_STYLE}
      >
        <Suspense fallback={null}>
          <CameraRig
            target={TARGET}
            azimuth={Math.PI / 3}
            elevation={Math.PI / 14}
            distance={distance}
            fov={45}
          />
          <Lighting intensity={1.5} />
          <Grid opacity={0.3} />
          <BVR1ModelSafe />
        </Suspense>
      </ThreeCanvas>
    </div>
  );
}
