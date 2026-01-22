import { useCurrentFrame } from "remotion";
import { Suspense } from "react";
import * as THREE from "three";
import { Html } from "@react-three/drei";
import { BVR1Model } from "../../../components/three/BVR1Model";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { smoothValue, easeOutQuart, easeInOutExpo } from "../../../lib/easing";
import { SHOWCASE_CONFIG, CALLOUT_TIMINGS } from "../config";
import { videoHighlightComponents } from "../../../lib/bvr1-components";
import { MUNI_ORANGE, FONT_MONO, WHITE } from "../../../lib/brand";

// Inline callout label component for 3D space
function CalloutLabel({
  name,
  desc,
  specs,
  opacity,
}: {
  name: string;
  desc: string;
  specs: string;
  opacity: number;
}) {
  if (opacity < 0.01) return null;

  return (
    <Html
      center
      style={{
        opacity,
        pointerEvents: "none",
        transform: "translateY(-100%)",
      }}
    >
      <div
        style={{
          background: "rgba(0, 0, 0, 0.9)",
          border: `2px solid ${MUNI_ORANGE}`,
          borderRadius: 8,
          padding: "16px 24px",
          fontFamily: FONT_MONO,
          color: WHITE,
          textAlign: "center",
          minWidth: 200,
          whiteSpace: "nowrap",
        }}
      >
        <div style={{ fontSize: 20, fontWeight: 700, color: MUNI_ORANGE, marginBottom: 6 }}>
          {name}
        </div>
        <div style={{ fontSize: 14, opacity: 0.8, marginBottom: 4 }}>{desc}</div>
        <div style={{ fontSize: 13, opacity: 0.6 }}>{specs}</div>
      </div>
      {/* Connector line */}
      <div
        style={{
          width: 2,
          height: 60,
          background: MUNI_ORANGE,
          margin: "0 auto",
        }}
      />
      {/* Anchor dot */}
      <div
        style={{
          width: 14,
          height: 14,
          borderRadius: "50%",
          background: MUNI_ORANGE,
          border: `2px solid ${WHITE}`,
          margin: "0 auto",
        }}
      />
    </Html>
  );
}

export function ComponentCallouts() {
  const frame = useCurrentFrame();
  const { camera, model } = SHOWCASE_CONFIG;

  // Determine which callout is active
  const callouts = [
    { timing: CALLOUT_TIMINGS.lidar, component: videoHighlightComponents[0], azimuth: Math.PI / 4 },
    { timing: CALLOUT_TIMINGS.camera, component: videoHighlightComponents[1], azimuth: Math.PI / 3 },
    { timing: CALLOUT_TIMINGS.motors, component: videoHighlightComponents[2], azimuth: -Math.PI / 4 },
    { timing: CALLOUT_TIMINGS.battery, component: videoHighlightComponents[3], azimuth: Math.PI / 2 },
  ];

  // Find current callout index
  let currentIndex = 0;
  for (let i = 0; i < callouts.length; i++) {
    if (frame >= callouts[i].timing.start && frame < callouts[i].timing.end) {
      currentIndex = i;
      break;
    }
    if (frame >= callouts[i].timing.end) {
      currentIndex = i;
    }
  }

  const currentCallout = callouts[currentIndex];
  const prevCallout = currentIndex > 0 ? callouts[currentIndex - 1] : currentCallout;

  // Camera transitions between callout positions
  const transitionDuration = 30; // 1 second transition
  const localFrame = frame - currentCallout.timing.start;

  const cameraAzimuth =
    localFrame < transitionDuration
      ? smoothValue(localFrame, 0, transitionDuration, prevCallout.azimuth, currentCallout.azimuth, easeInOutExpo)
      : currentCallout.azimuth;

  // Zoom in slightly for callouts
  const cameraDistance = smoothValue(frame, 0, 60, camera.defaultDistance, 700, easeOutQuart);

  return (
    <>
      <CameraRig
        target={new THREE.Vector3(camera.defaultTarget.x, model.centerY, camera.defaultTarget.z)}
        distance={cameraDistance}
        elevation={camera.defaultElevation * 1.2}
        azimuth={cameraAzimuth}
        fov={camera.fov}
      />

      <Lighting />

      <Grid opacity={0.5} />

      <Suspense fallback={null}>
        <BVR1Model />

        {/* Render all callout labels with appropriate opacity */}
        {callouts.map((callout, i) => {
          const isActive = i === currentIndex;
          const calloutFrame = frame - callout.timing.start;

          // Fade in/out logic
          let labelOpacity = 0;
          if (isActive) {
            // Fade in over first 20 frames, fade out over last 20 frames
            const fadeInEnd = 20;
            const fadeOutStart = callout.timing.end - callout.timing.start - 20;
            const fadeOutEnd = callout.timing.end - callout.timing.start;

            labelOpacity = smoothValue(calloutFrame, 0, fadeInEnd, 0, 1, easeOutQuart);
            if (calloutFrame > fadeOutStart) {
              labelOpacity *= smoothValue(calloutFrame, fadeOutStart, fadeOutEnd, 1, 0, easeOutQuart);
            }
          }

          const pos = callout.component.position;

          return (
            <group key={callout.component.id} position={[pos.x, pos.z, -pos.y]}>
              <CalloutLabel
                name={callout.component.name}
                desc={callout.component.desc}
                specs={callout.component.specs}
                opacity={labelOpacity}
              />
            </group>
          );
        })}
      </Suspense>
    </>
  );
}
