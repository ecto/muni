import { describe, it, expect, beforeEach } from "vitest";
import { useConsoleStore } from "./store";
import { Mode, InputSource } from "@/lib/types";

describe("ConsoleStore", () => {
  beforeEach(() => {
    // Reset store between tests
    useConsoleStore.setState(useConsoleStore.getInitialState());
  });

  describe("fleet management", () => {
    it("starts with no rovers", () => {
      const state = useConsoleStore.getState();
      expect(state.rovers).toEqual([]);
      expect(state.selectedRoverId).toBeNull();
    });

    it("setRovers updates the rover list", () => {
      const rovers = [
        { id: "frog-0", name: "frog-0", address: "10.0.0.1", port: 4852, lastSeen: Date.now() },
      ];
      useConsoleStore.getState().setRovers(rovers as any);
      expect(useConsoleStore.getState().rovers).toHaveLength(1);
      expect(useConsoleStore.getState().rovers[0].id).toBe("frog-0");
    });

    it("selectRover sets selectedRoverId", () => {
      useConsoleStore.getState().selectRover("frog-0");
      expect(useConsoleStore.getState().selectedRoverId).toBe("frog-0");
    });

    it("selectRover with null deselects", () => {
      useConsoleStore.getState().selectRover("frog-0");
      useConsoleStore.getState().selectRover(null);
      expect(useConsoleStore.getState().selectedRoverId).toBeNull();
    });
  });

  describe("connection state", () => {
    it("starts disconnected", () => {
      const state = useConsoleStore.getState();
      expect(state.connected).toBe(false);
      expect(state.connecting).toBe(false);
    });

    it("setConnected updates connection state", () => {
      useConsoleStore.getState().setConnected(true);
      expect(useConsoleStore.getState().connected).toBe(true);
    });

    it("setLatency applies exponential moving average", () => {
      useConsoleStore.getState().setLatency(100);
      const latency = useConsoleStore.getState().latencyMs;
      // EMA: 0.15 * 100 + 0.85 * 0 = 15
      expect(latency).toBe(15);
    });

    it("setLatency resets on zero", () => {
      useConsoleStore.getState().setLatency(100);
      useConsoleStore.getState().setLatency(0);
      expect(useConsoleStore.getState().latencyMs).toBe(0);
    });
  });

  describe("telemetry", () => {
    it("starts in Idle mode", () => {
      const state = useConsoleStore.getState();
      expect(state.telemetry.mode).toBe(Mode.Idle);
    });

    it("updateTelemetry merges partial state", () => {
      useConsoleStore.getState().updateTelemetry({ mode: Mode.Teleop });
      expect(useConsoleStore.getState().telemetry.mode).toBe(Mode.Teleop);
    });
  });

  describe("input", () => {
    it("setInputSource updates source", () => {
      useConsoleStore.getState().setInputSource(InputSource.Gamepad);
      expect(useConsoleStore.getState().inputSource).toBe(InputSource.Gamepad);
    });
  });

  describe("toast", () => {
    it("showToast sets toast message", () => {
      useConsoleStore.getState().showToast("test message");
      expect(useConsoleStore.getState().toast).toBe("test message");
    });
  });
});
