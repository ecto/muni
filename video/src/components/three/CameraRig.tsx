import { useThree } from "@react-three/fiber";
import { useEffect } from "react";
import * as THREE from "three";

interface CameraRigProps {
  // Target point to look at (usually model center)
  target?: THREE.Vector3;
  // Camera distance from target
  distance?: number;
  // Horizontal angle (radians, 0 = looking from +Z)
  azimuth?: number;
  // Vertical angle (radians, 0 = level, positive = looking down)
  elevation?: number;
  // Field of view
  fov?: number;
}

export function CameraRig({
  target = new THREE.Vector3(0, 250, 0),
  distance = 800,
  azimuth = Math.PI / 4,
  elevation = Math.PI / 8,
  fov = 45,
}: CameraRigProps) {
  const { camera } = useThree();

  useEffect(() => {
    // Calculate camera position from spherical coordinates
    const x = target.x + distance * Math.sin(azimuth) * Math.cos(elevation);
    const y = target.y + distance * Math.sin(elevation);
    const z = target.z + distance * Math.cos(azimuth) * Math.cos(elevation);

    camera.position.set(x, y, z);
    camera.lookAt(target);

    if ((camera as THREE.PerspectiveCamera).fov !== fov) {
      (camera as THREE.PerspectiveCamera).fov = fov;
      (camera as THREE.PerspectiveCamera).updateProjectionMatrix();
    }
  }, [camera, target, distance, azimuth, elevation, fov]);

  return null;
}
