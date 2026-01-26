import { useEffect, useCallback, useRef } from "react";
import { useConsoleStore } from "@/store";
import { InputSource, CameraMode } from "@/lib/types";

const KEYS: Record<string, readonly string[]> = {
  forward: ["KeyW", "ArrowUp"],
  backward: ["KeyS", "ArrowDown"],
  left: ["KeyA", "ArrowLeft"],
  right: ["KeyD", "ArrowRight"],
  toolUp: ["KeyE"],
  toolDown: ["KeyQ"],
  actionA: ["Space"],
  actionB: ["KeyF"],
  estop: ["Escape"],
  enable: ["Enter"],
  disable: ["Backspace"], // Release control gracefully
  boost: ["ShiftLeft", "ShiftRight"], // Hold Shift for full speed
  cameraToggle: ["KeyC"],
  cameraFree: ["KeyV"],
};

// Track pressed keys (module-level singleton)
const pressedKeys = new Set<string>();

// Module-level key tracking (never removed)
function globalKeyDown(e: KeyboardEvent) {
  pressedKeys.add(e.code);
}

function globalKeyUp(e: KeyboardEvent) {
  pressedKeys.delete(e.code);
}

// Clear all keys on blur (prevents stuck keys)
function globalBlur() {
  pressedKeys.clear();
}

// Initialize key tracking once at module load
if (typeof window !== "undefined") {
  window.addEventListener("keydown", globalKeyDown);
  window.addEventListener("keyup", globalKeyUp);
  window.addEventListener("blur", globalBlur);
}

export function useKeyboard() {
  const setInput = useConsoleStore((s) => s.setInput);
  const setInputSource = useConsoleStore((s) => s.setInputSource);
  const inputSource = useConsoleStore((s) => s.inputSource);
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const setCameraMode = useConsoleStore((s) => s.setCameraMode);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Don't capture if typing in an input
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      // Camera mode toggles
      if (KEYS.cameraToggle.includes(e.code)) {
        e.preventDefault();
        const modes = [
          CameraMode.ThirdPerson,
          CameraMode.FirstPerson,
          CameraMode.FreeLook,
        ];
        const modeLabels = ["3rd Person", "1st Person", "Free Look"];
        const currentIndex = modes.indexOf(cameraMode);
        const nextIndex = (currentIndex + 1) % modes.length;
        setCameraMode(modes[nextIndex]);
        useConsoleStore
          .getState()
          .showToast(`Camera: ${modeLabels[nextIndex]}`);
        return;
      }

      if (KEYS.cameraFree.includes(e.code)) {
        e.preventDefault();
        const newMode =
          cameraMode === CameraMode.FreeLook
            ? CameraMode.ThirdPerson
            : CameraMode.FreeLook;
        setCameraMode(newMode);
        useConsoleStore
          .getState()
          .showToast(
            `Camera: ${
              newMode === CameraMode.FreeLook ? "Free Look" : "3rd Person"
            }`
          );
        return;
      }

      // Set keyboard as input source
      if (inputSource !== InputSource.Keyboard) {
        setInputSource(InputSource.Keyboard);
      }
    },
    [inputSource, cameraMode, setInputSource, setCameraMode]
  );

  // Track previous input to skip redundant Zustand set() calls
  const prevInputRef = useRef({
    linear: 0, angular: 0, toolAxis: 0,
    actionA: false, actionB: false, estop: false,
    enable: false, disable: false, boost: false,
  });

  const updateInput = useCallback(() => {
    // Only update if keyboard is the active source
    const store = useConsoleStore.getState();
    if (store.inputSource !== InputSource.Keyboard) {
      return;
    }

    // Get currently pressed keys
    const pressed = pressedKeys;

    // Calculate movement
    let linear = 0;
    let angular = 0;
    let toolAxis = 0;

    if (KEYS.forward.some((k) => pressed.has(k))) linear += 1;
    if (KEYS.backward.some((k) => pressed.has(k))) linear -= 1;
    if (KEYS.left.some((k) => pressed.has(k))) angular += 1; // Left = positive (CCW)
    if (KEYS.right.some((k) => pressed.has(k))) angular -= 1; // Right = negative (CW)
    if (KEYS.toolUp.some((k) => pressed.has(k))) toolAxis += 1;
    if (KEYS.toolDown.some((k) => pressed.has(k))) toolAxis -= 1;

    // Buttons
    const actionA = KEYS.actionA.some((k) => pressed.has(k));
    const actionB = KEYS.actionB.some((k) => pressed.has(k));
    const estop = KEYS.estop.some((k) => pressed.has(k));
    const enable = KEYS.enable.some((k) => pressed.has(k));
    const disable = KEYS.disable.some((k) => pressed.has(k));
    const boost = KEYS.boost.some((k) => pressed.has(k));

    // Skip Zustand set() if nothing changed (avoids 60Hz state churn when idle)
    const prev = prevInputRef.current;
    if (
      prev.linear === linear && prev.angular === angular && prev.toolAxis === toolAxis &&
      prev.actionA === actionA && prev.actionB === actionB && prev.estop === estop &&
      prev.enable === enable && prev.disable === disable && prev.boost === boost
    ) {
      return;
    }
    prevInputRef.current = { linear, angular, toolAxis, actionA, actionB, estop, enable, disable, boost };

    setInput({
      linear,
      angular,
      toolAxis,
      actionA,
      actionB,
      estop,
      enable,
      disable,
      boost,
      cameraYaw: 0, // Mouse handled separately
      cameraPitch: 0,
    });
  }, [setInput]);

  useEffect(() => {
    // Only add the component-level keydown handler (for input source switching)
    // Key tracking is handled by module-level globalKeyDown/globalKeyUp
    window.addEventListener("keydown", handleKeyDown);

    // Poll at 60Hz for smooth input
    const interval = setInterval(updateInput, 16);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      clearInterval(interval);
    };
  }, [handleKeyDown, updateInput]);
}
