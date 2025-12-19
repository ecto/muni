import { useEffect, useRef } from "react";
import { useOperatorStore } from "@/store";
import { InputSource } from "@/lib/types";

function applyDeadzone(value: number, deadzone: number): number {
  if (Math.abs(value) < deadzone) return 0;
  // Rescale to 0-1 range after deadzone
  const sign = Math.sign(value);
  const magnitude = (Math.abs(value) - deadzone) / (1 - deadzone);
  return sign * magnitude;
}

export function useGamepad() {
  const { setInput, setInputSource, inputSource } = useOperatorStore();
  const frameRef = useRef<number | null>(null);

  useEffect(() => {
    const poll = () => {
      const gamepads = navigator.getGamepads();
      const gp = gamepads[0];

      if (gp && gp.connected) {
        // Switch to gamepad input source if any button/stick is active
        const hasActivity =
          gp.axes.some((a) => Math.abs(a) > 0.1) ||
          gp.buttons.some((b) => b.pressed || b.value > 0.1);

        if (hasActivity && inputSource !== InputSource.Gamepad) {
          setInputSource(InputSource.Gamepad);
        }

        // Only update if gamepad is the active source
        if (inputSource === InputSource.Gamepad || hasActivity) {
          // Left stick for movement
          // Stick up (negative Y) = forward (positive linear)
          // Stick right (positive X) = turn right (negative angular)
          const leftY = gp.axes[1] ?? 0;
          const leftX = gp.axes[0] ?? 0;
          const linear = applyDeadzone(-leftY, 0.1);
          const angular = applyDeadzone(-leftX, 0.1);

          // Right stick for camera (inverted)
          const rightX = gp.axes[2] ?? 0;
          const rightY = gp.axes[3] ?? 0;
          const cameraYaw = applyDeadzone(-rightX, 0.1) * 2;
          const cameraPitch = applyDeadzone(-rightY, 0.1) * 1.5;

          // Triggers for tool axis
          // RT (button 7 or axis 5) = positive, LT (button 6 or axis 4) = negative
          const rtButton = gp.buttons[7]?.value ?? 0;
          const ltButton = gp.buttons[6]?.value ?? 0;
          const toolAxis = rtButton - ltButton;

          // Buttons
          // A (South) or RB = action A
          const actionA =
            gp.buttons[0]?.pressed || gp.buttons[5]?.pressed || false;
          // B (East) or LB = action B
          const actionB =
            gp.buttons[1]?.pressed || gp.buttons[4]?.pressed || false;
          // Select/View = E-Stop
          const estop = gp.buttons[8]?.pressed || false;
          // Start/Menu = Enable
          const enable = gp.buttons[9]?.pressed || false;

          setInput({
            linear,
            angular,
            toolAxis,
            actionA,
            actionB,
            estop,
            enable,
            cameraYaw,
            cameraPitch,
          });

          if (hasActivity) {
            setInputSource(InputSource.Gamepad);
          }
        }
      }

      frameRef.current = requestAnimationFrame(poll);
    };

    frameRef.current = requestAnimationFrame(poll);

    // Handle gamepad connection events
    const onConnect = (_e: GamepadEvent) => {
      // Gamepad connected
    };

    const onDisconnect = (_e: GamepadEvent) => {
      if (inputSource === InputSource.Gamepad) {
        setInputSource(InputSource.None);
      }
    };

    window.addEventListener("gamepadconnected", onConnect);
    window.addEventListener("gamepaddisconnected", onDisconnect);

    return () => {
      if (frameRef.current) {
        cancelAnimationFrame(frameRef.current);
      }
      window.removeEventListener("gamepadconnected", onConnect);
      window.removeEventListener("gamepaddisconnected", onDisconnect);
    };
  }, [setInput, setInputSource, inputSource]);
}
