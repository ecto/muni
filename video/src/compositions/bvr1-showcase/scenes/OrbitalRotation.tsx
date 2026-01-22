import { useCurrentFrame } from "remotion";
import { Suspense } from "react";
import * as THREE from "three";
import { BVR1Model } from "../../../components/three/BVR1Model";
import { CameraRig } from "../../../components/three/CameraRig";
import { Lighting } from "../../../components/three/Lighting";
import { Grid } from "../../../components/three/Grid";
import { orbitAngle, easeInOutExpo } from "../../../lib/easing";
import { SHOWCASE_CONFIG } from "../config";

export function OrbitalRotation() {
  const frame = useCurrentFrame();
  const { camera, model } = SHOWCASE_CONFIG;
  const duration = 300; // 10 seconds at 30fps

  // Full 360° rotation over the scene duration
  const azimuth = orbitAngle(
    frame,
    0,
    duration,
    Math.PI / 4, // Starting angle (from HeroReveal end)
    Math.PI / 4 + Math.PI * 2, // Full rotation
    easeInOutExpo
  );

  // Slight elevation change for interest
  const elevationVariation = Math.sin(frame / duration * Math.PI * 2) * 0.05;
  const elevation = camera.defaultElevation + elevationVariation;

  return (
    <>
      <CameraRig
        target={new THREE.Vector3(camera.defaultTarget.x, model.centerY, camera.defaultTarget.z)}
        distance={camera.defaultDistance}
        elevation={elevation}
        azimuth={azimuth}
        fov={camera.fov}
      />

      <Lighting />

      <Grid />

      <Suspense fallback={null}>
        <BVR1Model />
      </Suspense>
    </>
  );
}
