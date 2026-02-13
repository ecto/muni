// Scene 6: Checklist, founder, CTA (0:33–0:48)
import { Sequence, useCurrentFrame } from "remotion";
import { interpolate } from "remotion";
import { ThreeCanvas } from "@remotion/three";
import { Suspense } from "react";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { BootSequence } from "../../../components/overlays/BootSequence";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { BVR1ModelSafe } from "../../../components/three/BVR1ModelSafe";
import { BVR1FounderShot } from "../components/BVR1HeroShot";
import { SnowParticles } from "../../../components/three/SnowParticles";
import { DARK_BG, MUNI_ORANGE, WHITE, FONT_MONO } from "../../../lib/brand";
import { FRAMES_PER_BEAT } from "../beat";

const section = REEL_CONFIG.sections.theClose;

export function TheClose() {
  const cuts = getCutsInRange(section.start, section.end);

  return (
    <>
      {cuts.map((cut) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        // Boot sequence checklist
        if (cut.label === "close-checklist") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BootSequence staggerFrames={10} />
            </Sequence>
          );
        }

        // Pink BVR1 — 3D model + grid tinted hot pink
        if (cut.label === "pink-bvr1") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <div style={{ position: "absolute", inset: 0, backgroundColor: "#0a0008" }}>
                <ThreeCanvas
                  width={REEL_CONFIG.width}
                  height={REEL_CONFIG.height}
                  camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
                  style={{ position: "absolute", inset: 0 }}
                >
                  <Suspense fallback={null}>
                    <PinkBVR1 durationInFrames={duration} />
                  </Suspense>
                </ThreeCanvas>
              </div>
            </Sequence>
          );
        }

        // Founder shot — 3D BVR1 close-up (replacing photo placeholder)
        if (cut.label === "close-founder") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1FounderShot durationInFrames={duration} />
            </Sequence>
          );
        }

        // Event promo — lines stagger in on the beat
        if (cut.label === "close-event") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <EventPromo />
            </Sequence>
          );
        }

        // Final 3D beauty shot
        if (cut.label === "close-bvr1-beauty") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <div style={{ position: "absolute", inset: 0, backgroundColor: DARK_BG }}>
                <ThreeCanvas
                  width={REEL_CONFIG.width}
                  height={REEL_CONFIG.height}
                  camera={{ fov: 45, near: 0.1, far: 5000, position: [600, 400, 600] }}
                  style={{ position: "absolute", inset: 0 }}
                >
                  <Suspense fallback={null}>
                    <BeautyShot durationInFrames={duration} />
                  </Suspense>
                </ThreeCanvas>
              </div>
            </Sequence>
          );
        }

        return (
          <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
            <ShotFrame shot={shot} durationInFrames={duration} />
            {shot.textOverlay && (
              <HypeTextCard
                text={shot.textOverlay}
                tag={shot.tag}
                bgColor={shot.bgColor}
                fontSize={cut.label === "close-url" ? 120 : 96}
              />
            )}
          </Sequence>
        );
      })}
    </>
  );
}

/** Pink BVR1 — hot pink tinted model + grid */
function PinkBVR1({ durationInFrames }: { durationInFrames: number }) {
  const frame = useCurrentFrame();
  const azimuth = interpolate(frame, [0, durationInFrames], [Math.PI / 3, Math.PI / 2.5], {
    extrapolateRight: "clamp",
  });

  return (
    <>
      <CameraRig distance={800} azimuth={azimuth} elevation={Math.PI / 8} fov={40} />
      <ambientLight intensity={0.3} color="#ff69b4" />
      <directionalLight position={[5, 8, 3]} intensity={1.5} color="#ff1493" />
      <directionalLight position={[-3, 4, -5]} intensity={0.6} color="#ff69b4" />
      <pointLight position={[0, 200, 0]} intensity={0.8} color="#ff1493" distance={2000} />
      <Grid opacity={0.7} cellColor="#ff1493" sectionColor="#ff69b4" />
      <BVR1ModelSafe />
    </>
  );
}

/** Final beauty shot — slow orbit with real BVR1 model */
function BeautyShot({ durationInFrames }: { durationInFrames: number }) {
  const frame = useCurrentFrame();
  const azimuth = interpolate(frame, [0, durationInFrames], [Math.PI / 4, Math.PI / 2], {
    extrapolateRight: "clamp",
  });
  const distance = interpolate(frame, [0, durationInFrames], [900, 750], {
    extrapolateRight: "clamp",
  });

  return (
    <>
      <CameraRig distance={distance} azimuth={azimuth} elevation={Math.PI / 10} fov={40} />
      <Lighting intensity={1.3} />
      <fog attach="fog" args={["#0a0a0a", 800, 3000]} />
      <Grid opacity={0.5} />
      <BVR1ModelSafe />
      <SnowParticles count={150} speed={1.5} size={2} opacity={0.3} area={[2500, 1500, 2500]} />
    </>
  );
}

const EVENT_LINES = [
  "SEE IT IN PERSON",
  "ONE NIGHT ONLY",
  "FORT MASON, SF",
  "FRIDAY THE 13TH, FEB 2026",
];

function EventPromo() {
  const frame = useCurrentFrame();
  const beatFrames = Math.round(FRAMES_PER_BEAT);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: "#0a0a0a",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 12,
      }}
    >
      {EVENT_LINES.map((line, i) => {
        const appearFrame = i * beatFrames;
        const visible = frame >= appearFrame;
        const age = frame - appearFrame;
        const opacity = visible
          ? interpolate(age, [0, 4], [0, 1], { extrapolateRight: "clamp" })
          : 0;
        const translateY = visible
          ? interpolate(age, [0, 4], [20, 0], { extrapolateRight: "clamp" })
          : 20;

        return (
          <div
            key={i}
            style={{
              fontFamily: FONT_MONO,
              fontSize: i === 3 ? 48 : 56,
              fontWeight: 700,
              color: i === 0 ? MUNI_ORANGE : WHITE,
              opacity,
              transform: `translateY(${translateY}px)`,
              letterSpacing: 3,
            }}
          >
            {line}
          </div>
        );
      })}
    </div>
  );
}
