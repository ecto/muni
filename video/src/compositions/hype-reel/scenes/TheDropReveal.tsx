// Scene 2: Beat drop, BVR1 reveal, "no gas" stats (0:12–0:18)
import { Sequence, useCurrentFrame } from "remotion";
import { interpolate } from "remotion";
import { ThreeCanvas } from "@remotion/three";
import { Suspense } from "react";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration, beatToFrame } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { BVR1ModelSafe } from "../../../components/three/BVR1ModelSafe";
import { BVR1HeroFront, BVR1HeroSide, BVR1HeroRear } from "../components/BVR1HeroShot";
import { MUNI_ORANGE, WHITE, FONT_MONO, DARK_BG } from "../../../lib/brand";

const section = REEL_CONFIG.sections.theDropReveal;
const FONT_DISPLAY = '"Inter", "Helvetica Neue", Arial, sans-serif';

/** Stats that appear one-by-one on beats when A$AP says "no gas" */
function StatsReveal() {
  const frame = useCurrentFrame();

  const stats = [
    { label: "48V SYSTEM", beat: 0 },
    { label: "4x VESC 75/300", beat: 2 },
    { label: "12 kW PEAK", beat: 4 },
    { label: "ZERO EMISSIONS", beat: 6 },
  ];

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: "#050505",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 20,
      }}
    >
      <div
        style={{
          fontFamily: FONT_DISPLAY,
          fontSize: 24,
          fontWeight: 900,
          color: MUNI_ORANGE,
          letterSpacing: 8,
          marginBottom: 16,
        }}
      >
        NO GAS
      </div>
      {stats.map((stat, i) => {
        // Each stat appears on its beat (1-beat spacing ≈ 13 frames, snappier stagger)
        const appearFrame = i * 10;
        const visible = frame >= appearFrame;
        const age = frame - appearFrame;
        const opacity = visible
          ? interpolate(age, [0, 4], [0, 1], {
              extrapolateRight: "clamp",
            })
          : 0;
        const translateY = visible
          ? interpolate(age, [0, 4], [20, 0], {
              extrapolateRight: "clamp",
            })
          : 20;

        return (
          <div
            key={stat.label}
            style={{
              fontFamily: FONT_MONO,
              fontSize: 56,
              fontWeight: 700,
              color: i % 2 === 0 ? MUNI_ORANGE : WHITE,
              opacity,
              transform: `translateY(${translateY}px)`,
              letterSpacing: 2,
            }}
          >
            {stat.label}
          </div>
        );
      })}
    </div>
  );
}

export function TheDropReveal() {
  const frame = useCurrentFrame();
  const cuts = getCutsInRange(section.start, section.end);

  // Flash white on beat drop (first 3 frames)
  const flashOpacity = interpolate(frame, [0, 3], [0.8, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <>
      {cuts.map((cut) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        // BVR1 hero shots — 3D rendered
        if (cut.label === "bvr1-hero-front") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1HeroFront durationInFrames={duration} />
              {shot.textOverlay && (
                <HypeTextCard text={shot.textOverlay} fontSize={120} />
              )}
            </Sequence>
          );
        }
        if (cut.label === "bvr1-hero-side") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1HeroSide durationInFrames={duration} />
            </Sequence>
          );
        }
        if (cut.label === "bvr1-hero-rear") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1HeroRear durationInFrames={duration} />
            </Sequence>
          );
        }

        // 3D wireframe spin — campedersen.com aesthetic
        if (cut.label === "bvr1-3d-spin") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <div style={{ position: "absolute", inset: 0, backgroundColor: "#050208" }}>
                <ThreeCanvas
                  width={REEL_CONFIG.width}
                  height={REEL_CONFIG.height}
                  camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
                  style={{ position: "absolute", inset: 0 }}
                >
                  <Suspense fallback={null}>
                    <SpinningBVR1 durationInFrames={duration} />
                  </Suspense>
                </ThreeCanvas>
              </div>
            </Sequence>
          );
        }

        // "No gas" stats — render as a single combined sequence
        if (cut.label === "no-gas-48v") {
          const statsEnd = section.end;
          const noGasStart = cut.frame - section.start;
          // Find the end of the last no-gas cut
          const noGasCuts = cuts.filter((c) => c.label.startsWith("no-gas-"));
          const lastNoGas = noGasCuts[noGasCuts.length - 1];
          const noGasDuration = lastNoGas
            ? lastNoGas.frame - section.start + getCutDuration(lastNoGas, section.end) - noGasStart
            : duration;

          return (
            <Sequence key={cut.label} from={noGasStart} durationInFrames={noGasDuration}>
              <StatsReveal />
            </Sequence>
          );
        }

        // Skip individual no-gas cuts (handled above as group)
        if (cut.label.startsWith("no-gas-") && cut.label !== "no-gas-48v") {
          return null;
        }

        return (
          <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
            <ShotFrame shot={shot} durationInFrames={duration} />
            {shot.textOverlay && (
              <HypeTextCard
                text={shot.textOverlay}
                tag={shot.tag}
                bgColor={shot.bgColor}
                fontSize={120}
              />
            )}
          </Sequence>
        );
      })}

      {/* Beat drop flash */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundColor: WHITE,
          opacity: flashOpacity,
          pointerEvents: "none",
        }}
      />
    </>
  );
}

/** Wireframe BVR1 spin — campedersen.com aesthetic */
function SpinningBVR1({ durationInFrames }: { durationInFrames: number }) {
  const frame = useCurrentFrame();
  const azimuth = interpolate(frame, [0, durationInFrames], [0, Math.PI * 2], {
    extrapolateRight: "clamp",
  });

  return (
    <>
      <CameraRig distance={800} azimuth={azimuth} elevation={Math.PI / 10} fov={45} />
      <ambientLight intensity={0.3} color="#a855f7" />
      <directionalLight position={[5, 8, 3]} intensity={0.8} color="#a855f7" />
      <directionalLight position={[-3, 4, -5]} intensity={0.4} color="#f97316" />
      <Grid opacity={0.4} cellColor="#7c3aed" sectionColor="#a855f7" />
      <BVR1ModelSafe wireframe tint="#a855f7" />
    </>
  );
}
