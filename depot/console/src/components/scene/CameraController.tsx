import { useRef, useEffect } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { Vector3 } from "three";
import { useConsoleStore } from "@/store";
import { CameraMode, Mode } from "@/lib/types";
import { computeRenderPose, getInterpolationBuffer } from "@/lib/interpolation";

export function CameraController() {
  const { camera, gl } = useThree();
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const telemetryMode = useConsoleStore((s) => s.telemetry.mode);

  const yawOffset = useRef(0);
  const pitch = useRef(0.4);
  const distance = useRef(3.5);

  // Skip lerp on first frame to avoid fly-in animation on mount
  const firstFrame = useRef(true);

  // Mouse drag state for orbit controls when not in teleop
  const isDragging = useRef(false);
  const lastMouse = useRef({ x: 0, y: 0 });

  // Check if teleop is active (only allow mouse orbit when NOT in teleop)
  const isTeleopActive = telemetryMode === Mode.Teleop;

  // Reset camera to behind rover when entering teleop mode
  const wasTeleopActive = useRef(false);
  useEffect(() => {
    if (isTeleopActive && !wasTeleopActive.current) {
      // Entering teleop: reset to default follow camera
      yawOffset.current = 0;
      pitch.current = 0.4;
      distance.current = 3.5;
    }
    wasTeleopActive.current = isTeleopActive;
  }, [isTeleopActive]);

  // Mouse event handlers for orbit controls
  useEffect(() => {
    const canvas = gl.domElement;

    const onMouseDown = (e: MouseEvent) => {
      if (isTeleopActive) return;
      isDragging.current = true;
      lastMouse.current = { x: e.clientX, y: e.clientY };
      canvas.style.cursor = "grabbing";
    };

    const onMouseMove = (e: MouseEvent) => {
      if (!isDragging.current || isTeleopActive) return;

      const dx = e.clientX - lastMouse.current.x;
      const dy = e.clientY - lastMouse.current.y;
      lastMouse.current = { x: e.clientX, y: e.clientY };

      // Sensitivity: pixels to radians
      const sensitivity = 0.005;
      yawOffset.current -= dx * sensitivity;
      pitch.current += dy * sensitivity;
      pitch.current = Math.max(0.1, Math.min(1.4, pitch.current));
    };

    const onMouseUp = () => {
      isDragging.current = false;
      canvas.style.cursor = "grab";
    };

    const onMouseLeave = () => {
      isDragging.current = false;
      canvas.style.cursor = "";
    };

    const onWheel = (e: WheelEvent) => {
      if (isTeleopActive) return;
      e.preventDefault();
      // Zoom: adjust distance
      const zoomSensitivity = 0.002;
      distance.current += e.deltaY * zoomSensitivity;
      distance.current = Math.max(1.5, Math.min(15, distance.current));
    };

    // Set initial cursor when not in teleop
    if (!isTeleopActive) {
      canvas.style.cursor = "grab"; // eslint-disable-line react-hooks/immutability -- DOM element, not hook return value
    }

    canvas.addEventListener("mousedown", onMouseDown);
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    canvas.addEventListener("mouseleave", onMouseLeave);
    canvas.addEventListener("wheel", onWheel, { passive: false });

    return () => {
      canvas.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
      canvas.removeEventListener("mouseleave", onMouseLeave);
      canvas.removeEventListener("wheel", onWheel);
      canvas.style.cursor = "";
    };
  }, [gl, isTeleopActive]);

  useFrame((_, delta) => {
    // Get latest input from store
    const { input } = useConsoleStore.getState();

    // Compute render pose from interpolation buffer (same source as RoverModel)
    const result = computeRenderPose(getInterpolationBuffer(), performance.now(), 0);
    const renderPose = result.pose;

    // Apply camera input
    const hasInput =
      Math.abs(input.cameraYaw) > 0.01 || Math.abs(input.cameraPitch) > 0.01;
    if (hasInput) {
      yawOffset.current += input.cameraYaw * delta * 2;
      pitch.current += input.cameraPitch * delta * 1.5;
      pitch.current = Math.max(0.1, Math.min(1.4, pitch.current));
    }

    // Auto-reset in third person when no input (only during teleop)
    if (!hasInput && cameraMode === CameraMode.ThirdPerson && isTeleopActive) {
      const returnSpeed =
        Math.abs(input.linear) > 0.1 || Math.abs(input.angular) > 0.1 ? 4 : 2;
      yawOffset.current *= 1 - returnSpeed * delta;
    }

    // Wrap yaw offset
    if (yawOffset.current > Math.PI) yawOffset.current -= Math.PI * 2;
    if (yawOffset.current < -Math.PI) yawOffset.current += Math.PI * 2;

    // Rover position in 3D (matching RoverModel coordinate mapping)
    // Physics: X=forward, Y=left; Three.js: -Z=forward, -X=left
    const roverPos = new Vector3(-renderPose.y, 0.15, -renderPose.x);

    switch (cameraMode) {
      case CameraMode.FirstPerson: {
        // Forward vector: physics (cos, sin) → Three.js (-sin, 0, -cos)
        const forward = new Vector3(
          -Math.sin(renderPose.theta),
          0,
          -Math.cos(renderPose.theta)
        );
        camera.position
          .copy(roverPos)
          .add(new Vector3(0, 0.3, 0))
          .add(forward.multiplyScalar(0.2));
        camera.lookAt(camera.position.clone().add(forward));
        break;
      }

      case CameraMode.ThirdPerson: {
        // Follow heading only during teleop/autonomous; idle SLAM drift is distracting
        const followHeading = isTeleopActive || telemetryMode === Mode.Autonomous;
        const yaw = (followHeading ? renderPose.theta : 0) + yawOffset.current;
        const horizontalDist = distance.current * Math.cos(pitch.current);
        const height = distance.current * Math.sin(pitch.current);

        // Offset: physics "behind" (-cos, -sin) → Three.js (sin, 0, cos)
        const targetPos = new Vector3(
          roverPos.x + horizontalDist * Math.sin(yaw),
          height + 0.3,
          roverPos.z + horizontalDist * Math.cos(yaw)
        );

        if (firstFrame.current) {
          camera.position.copy(targetPos);
        } else {
          camera.position.lerp(targetPos, Math.min(1, 8 * delta));
        }
        camera.lookAt(roverPos);
        break;
      }

      case CameraMode.FreeLook: {
        // Camera orbits at fixed world angle (doesn't follow rover heading)
        const horizontalDist = distance.current * Math.cos(pitch.current);
        const height = distance.current * Math.sin(pitch.current);

        const targetPos = new Vector3(
          roverPos.x + horizontalDist * Math.sin(yawOffset.current),
          height + 0.3,
          roverPos.z + horizontalDist * Math.cos(yawOffset.current)
        );

        if (firstFrame.current) {
          camera.position.copy(targetPos);
        } else {
          camera.position.lerp(targetPos, Math.min(1, 8 * delta));
        }
        camera.lookAt(roverPos);
        break;
      }
    }

    firstFrame.current = false;
  });

  return null;
}
