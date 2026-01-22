import { useCurrentFrame } from "remotion";
import { Suspense } from "react";
import * as THREE from "three";
import { BVR1Model } from "../../../components/three/BVR1Model";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { smoothValue, easeOutQuart, easeOutExpo } from "../../../lib/easing";
import { SHOWCASE_CONFIG } from "../config";

export function HeroReveal() {
  const frame = useCurrentFrame();
  const { camera, model } = SHOWCASE_CONFIG;

  // Model rises from below (frames 0-120)
  const modelY = smoothValue(frame, 0, 120, -200, 0, easeOutQuart);

  // Camera pulls back and settles (frames 0-180)
  const cameraDistance = smoothValue(frame, 0, 180, 500, camera.defaultDistance, easeOutExpo);
  const cameraElevation = smoothValue(frame, 0, 180, Math.PI / 16, camera.defaultElevation, easeOutQuart);
  const cameraAzimuth = smoothValue(frame, 0, 180, Math.PI / 3, Math.PI / 4, easeOutQuart);

  // Fade in grid (frames 30-90)
  const gridOpacity = smoothValue(frame, 30, 90, 0, 1, easeOutQuart);

  // Fade in lighting (frames 0-60)
  const lightIntensity = smoothValue(frame, 0, 60, 0.3, 1, easeOutQuart);

  return (
    <>
      <CameraRig
        target={new THREE.Vector3(camera.defaultTarget.x, model.centerY, camera.defaultTarget.z)}
        distance={cameraDistance}
        elevation={cameraElevation}
        azimuth={cameraAzimuth}
        fov={camera.fov}
      />

      <Lighting intensity={lightIntensity} />

      <Grid opacity={gridOpacity} />

      <Suspense fallback={null}>
        <BVR1Model positionY={modelY} />
      </Suspense>
    </>
  );
}
