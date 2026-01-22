import { useCurrentFrame } from "remotion";
import { Suspense } from "react";
import * as THREE from "three";
import { Html } from "@react-three/drei";
import { BVR1Model } from "../../../components/three/BVR1Model";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { smoothValue, easeOutQuart, easeInOutExpo } from "../../../lib/easing";
import { SHOWCASE_CONFIG, BVR1_SPECS } from "../config";
import { MUNI_ORANGE, FONT_MONO, WHITE, DARK_BG } from "../../../lib/brand";

function SpecsOverlay({ opacity }: { opacity: number }) {
  if (opacity < 0.01) return null;

  return (
    <Html
      fullscreen
      style={{
        opacity,
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          position: "absolute",
          right: 80,
          top: "50%",
          transform: "translateY(-50%)",
          background: `linear-gradient(135deg, ${DARK_BG} 0%, rgba(26, 26, 26, 0.95) 100%)`,
          border: `2px solid ${MUNI_ORANGE}`,
          borderRadius: 16,
          padding: "40px 48px",
          fontFamily: FONT_MONO,
          color: WHITE,
          minWidth: 380,
          backdropFilter: "blur(10px)",
        }}
      >
        <div
          style={{
            fontSize: 32,
            fontWeight: 700,
            color: MUNI_ORANGE,
            marginBottom: 32,
            textTransform: "uppercase",
            letterSpacing: 3,
          }}
        >
          BVR1 Specs
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          {BVR1_SPECS.map((spec, i) => (
            <div
              key={i}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 48,
                borderBottom: i < BVR1_SPECS.length - 1 ? "1px solid rgba(255,102,0,0.2)" : "none",
                paddingBottom: i < BVR1_SPECS.length - 1 ? 16 : 0,
              }}
            >
              <span style={{ fontSize: 17, opacity: 0.7 }}>{spec.label}</span>
              <span style={{ fontSize: 17, fontWeight: 700 }}>{spec.value}</span>
            </div>
          ))}
        </div>
      </div>
    </Html>
  );
}

function LogoOverlay({ opacity }: { opacity: number }) {
  if (opacity < 0.01) return null;

  return (
    <Html
      fullscreen
      style={{
        opacity,
        pointerEvents: "none",
      }}
    >
      <div
        style={{
          position: "absolute",
          left: 60,
          bottom: 60,
          display: "flex",
          alignItems: "center",
          gap: 20,
        }}
      >
        <img
          src="/images/muni-logo-light.svg"
          alt="Muni"
          style={{ width: 60, height: 60 }}
        />
        <div
          style={{
            fontFamily: FONT_MONO,
            fontSize: 36,
            fontWeight: 700,
            color: WHITE,
            letterSpacing: 2,
          }}
        >
          MUNI
        </div>
      </div>
    </Html>
  );
}

export function SpecsClose() {
  const frame = useCurrentFrame();
  const { camera, model } = SHOWCASE_CONFIG;
  const duration = 240; // 8 seconds

  // Camera slowly pulls back
  const cameraDistance = smoothValue(frame, 0, duration, 700, 900, easeOutQuart);

  // Slow rotation
  const cameraAzimuth = smoothValue(frame, 0, duration, Math.PI / 2, Math.PI / 2 + Math.PI / 8, easeInOutExpo);

  // Specs fade in (frames 0-40)
  const specsOpacity = smoothValue(frame, 0, 40, 0, 1, easeOutQuart);

  // Logo fade in (frames 30-70)
  const logoOpacity = smoothValue(frame, 30, 70, 0, 1, easeOutQuart);

  // Final fade out (frames 200-240)
  const fadeOut = frame > 200 ? smoothValue(frame, 200, duration, 1, 0, easeOutQuart) : 1;

  return (
    <>
      <CameraRig
        target={new THREE.Vector3(camera.defaultTarget.x, model.centerY, camera.defaultTarget.z)}
        distance={cameraDistance}
        elevation={camera.defaultElevation}
        azimuth={cameraAzimuth}
        fov={camera.fov}
      />

      <Lighting intensity={fadeOut} />

      <Grid opacity={0.4 * fadeOut} />

      <Suspense fallback={null}>
        <group>
          <BVR1Model opacity={fadeOut} />
        </group>
      </Suspense>

      <SpecsOverlay opacity={specsOpacity * fadeOut} />
      <LogoOverlay opacity={logoOpacity * fadeOut} />
    </>
  );
}
