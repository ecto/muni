import { useCurrentFrame } from "remotion";
import { Suspense } from "react";
import * as THREE from "three";
import { BVR1Model } from "../../../components/three/BVR1Model";
import { AstronautModel } from "../../../components/three/AstronautModel";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { smoothValue, easeOutQuart, easeInOutExpo } from "../../../lib/easing";
import { SHOWCASE_CONFIG } from "../config";

export function ScaleComparison() {
  const frame = useCurrentFrame();
  const { camera, model } = SHOWCASE_CONFIG;

  // Camera adjusts to show both model and astronaut
  const cameraAzimuth = smoothValue(frame, 0, 60, Math.PI / 4, Math.PI / 2.5, easeInOutExpo);
  const cameraDistance = smoothValue(frame, 0, 60, 800, 1000, easeOutQuart);

  // Astronaut walks in from left (frames 30-150)
  const astronautX = smoothValue(frame, 30, 150, -800, -350, easeOutQuart);

  // Astronaut fades in (frames 30-60)
  const astronautOpacity = smoothValue(frame, 30, 60, 0, 1, easeOutQuart);

  return (
    <>
      <CameraRig
        target={new THREE.Vector3(-100, model.centerY, camera.defaultTarget.z)}
        distance={cameraDistance}
        elevation={camera.defaultElevation}
        azimuth={cameraAzimuth}
        fov={camera.fov}
      />

      <Lighting />

      <Grid />

      <Suspense fallback={null}>
        <BVR1Model />
        <AstronautModel
          positionX={astronautX}
          positionZ={50}
          opacity={astronautOpacity}
        />
      </Suspense>
    </>
  );
}
