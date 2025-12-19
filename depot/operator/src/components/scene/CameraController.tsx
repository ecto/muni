import { useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { Vector3 } from "three";
import { useOperatorStore } from "@/store";
import { CameraMode } from "@/lib/types";

export function CameraController() {
  const { camera } = useThree();
  const { renderPose, cameraMode, input } = useOperatorStore();

  const yawOffset = useRef(0);
  const pitch = useRef(0.4);
  const distance = useRef(3.5);

  useFrame((_, delta) => {
    // Apply camera input
    const hasInput =
      Math.abs(input.cameraYaw) > 0.01 || Math.abs(input.cameraPitch) > 0.01;
    if (hasInput) {
      yawOffset.current += input.cameraYaw * delta * 2;
      pitch.current += input.cameraPitch * delta * 1.5;
      pitch.current = Math.max(0.1, Math.min(1.4, pitch.current));
    }

    // Auto-reset in third person when no input
    if (!hasInput && cameraMode === CameraMode.ThirdPerson) {
      const returnSpeed =
        Math.abs(input.linear) > 0.1 || Math.abs(input.angular) > 0.1 ? 4 : 2;
      yawOffset.current *= 1 - returnSpeed * delta;
    }

    // Wrap yaw offset
    if (yawOffset.current > Math.PI) yawOffset.current -= Math.PI * 2;
    if (yawOffset.current < -Math.PI) yawOffset.current += Math.PI * 2;

    // Rover position in 3D
    const roverPos = new Vector3(renderPose.x, 0.15, -renderPose.y);

    switch (cameraMode) {
      case CameraMode.FirstPerson: {
        const forward = new Vector3(
          Math.cos(renderPose.theta),
          0,
          -Math.sin(renderPose.theta)
        );
        camera.position
          .copy(roverPos)
          .add(new Vector3(0, 0.3, 0))
          .add(forward.multiplyScalar(0.2));
        camera.lookAt(camera.position.clone().add(forward));
        break;
      }

      case CameraMode.ThirdPerson: {
        const totalYaw = renderPose.theta + yawOffset.current + Math.PI;
        const horizontalDist = distance.current * Math.cos(pitch.current);
        const height = distance.current * Math.sin(pitch.current);

        const targetPos = new Vector3(
          renderPose.x + horizontalDist * Math.cos(totalYaw),
          height + 0.3,
          -renderPose.y - horizontalDist * Math.sin(totalYaw)
        );

        camera.position.lerp(targetPos, 8 * delta);
        camera.lookAt(roverPos);
        break;
      }

      case CameraMode.FreeLook: {
        const horizontalDist = distance.current * Math.cos(pitch.current);
        const height = distance.current * Math.sin(pitch.current);

        const targetPos = new Vector3(
          renderPose.x + horizontalDist * Math.cos(yawOffset.current),
          height + 0.3,
          -renderPose.y - horizontalDist * Math.sin(yawOffset.current)
        );

        camera.position.lerp(targetPos, 8 * delta);
        camera.lookAt(roverPos);
        break;
      }
    }
  });

  return null;
}
