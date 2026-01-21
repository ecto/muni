import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  Mode,
  type Telemetry,
  type GamepadInput,
  type Pose,
  type RoverInfo,
  type Session,
  type ServiceHealth,
  type GpsStatus,
  InputSource,
  CameraMode,
  defaultHealth,
} from "@/lib/types";
import {
  type InterpolationState,
  type PoseSnapshot,
  createInterpolationState,
  pushSnapshot,
} from "@/lib/interpolation";

export type Theme = "light" | "dark" | "system";

interface ConsoleState {
  // Fleet management
  rovers: RoverInfo[];
  selectedRoverId: string | null;
  setRovers: (rovers: RoverInfo[]) => void;
  updateRover: (id: string, updates: Partial<RoverInfo>) => void;
  selectRover: (id: string | null) => void;

  // Connection (for teleop)
  /** WebRTC signaling address for the selected rover */
  rtcAddress: string;
  connecting: boolean;
  connected: boolean;
  latencyMs: number;
  setRtcAddress: (address: string) => void;
  setConnecting: (connecting: boolean) => void;
  setConnected: (connected: boolean) => void;
  setLatency: (ms: number) => void;

  // Telemetry
  telemetry: Telemetry;
  updateTelemetry: (partial: Partial<Telemetry>) => void;

  // Interpolated pose for smooth rendering
  renderPose: Pose;
  isExtrapolating: boolean;
  interpolation: InterpolationState;
  setRenderPose: (pose: Pose, isExtrapolating?: boolean) => void;
  pushTelemetrySnapshot: (snapshot: PoseSnapshot) => void;

  // Input
  input: GamepadInput;
  inputSource: InputSource;
  setInput: (input: GamepadInput) => void;
  setInputSource: (source: InputSource) => void;

  // Camera
  cameraMode: CameraMode;
  setCameraMode: (mode: CameraMode) => void;

  // Toast notifications
  toast: string | null;
  showToast: (message: string, duration?: number) => void;

  // Video (multi-camera support)
  videoConnected: boolean;
  videoFps: number;
  /** @deprecated Use cameraFrames instead */
  videoFrame: string | null;
  videoTimestamp: number;
  /** Frames from each camera, keyed by camera ID */
  cameraFrames: Map<number, { url: string; timestamp: number }>;
  /** Number of active cameras */
  cameraCount: number;
  setVideoConnected: (connected: boolean) => void;
  setVideoFps: (fps: number) => void;
  setVideoFrame: (frame: string | null, timestamp: number) => void;
  setCameraFrame: (cameraId: number, url: string, timestamp: number) => void;

  // Sessions
  sessions: Session[];
  selectedSessionId: string | null;
  sessionsLoading: boolean;
  sessionsError: string | null;
  sessionRoverFilter: string | null;
  setSessions: (sessions: Session[]) => void;
  selectSession: (sessionId: string | null) => void;
  setSessionsLoading: (loading: boolean) => void;
  setSessionsError: (error: string | null) => void;
  setSessionRoverFilter: (roverId: string | null) => void;

  // Infrastructure status
  services: ServiceHealth[];
  gpsStatus: GpsStatus | null;
  setServices: (services: ServiceHealth[]) => void;
  setGpsStatus: (status: GpsStatus | null) => void;

  // Theme
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

const defaultTelemetry: Telemetry = {
  mode: Mode.Disabled,
  pose: { x: 0, y: 0, theta: 0 },
  power: { battery_voltage: 0, system_current: 0 },
  velocity: { linear: 0, angular: 0 },
  motor_temps: [0, 0, 0, 0],
  connected: false,
  latency_ms: 0,
  health: defaultHealth,
};

const defaultInput: GamepadInput = {
  linear: 0,
  angular: 0,
  toolAxis: 0,
  actionA: false,
  actionB: false,
  estop: false,
  enable: false,
  disable: false,
  boost: false,
  cameraYaw: 0,
  cameraPitch: 0,
};

export const useConsoleStore = create<ConsoleState>()(
  persist(
    (set) => ({
  // Fleet management
  rovers: [],
  selectedRoverId: null,
  setRovers: (rovers) => set({ rovers }),
  updateRover: (id, updates) =>
    set((state) => ({
      rovers: state.rovers.map((r) => (r.id === id ? { ...r, ...updates } : r)),
    })),
  selectRover: (id) => {
    set((state) => {
      const rover = state.rovers.find((r) => r.id === id);
      if (rover) {
        return {
          selectedRoverId: id,
          rtcAddress: rover.rtcAddress,
        };
      }
      return { selectedRoverId: id };
    });
  },

  // Connection
  rtcAddress: "ws://localhost:4852",
  connecting: false,
  connected: false,
  latencyMs: 0,
  setRtcAddress: (address) => set({ rtcAddress: address }),
  setConnecting: (connecting) => set({ connecting }),
  setConnected: (connected) => set({ connected, connecting: false }),
  setLatency: (ms) =>
    set((state) => {
      if (ms === 0) return { latencyMs: 0 };
      // Exponential moving average for smooth display
      const alpha = 0.15;
      const smoothed = alpha * ms + (1 - alpha) * state.latencyMs;
      return { latencyMs: Math.round(smoothed) };
    }),

  // Telemetry
  telemetry: defaultTelemetry,
  updateTelemetry: (partial) =>
    set((state) => ({
      telemetry: { ...state.telemetry, ...partial },
    })),

  // Render pose and interpolation
  renderPose: { x: 0, y: 0, theta: 0 },
  isExtrapolating: false,
  interpolation: createInterpolationState(),
  setRenderPose: (pose, isExtrapolating = false) =>
    set({ renderPose: pose, isExtrapolating }),
  pushTelemetrySnapshot: (snapshot) =>
    set((state) => ({
      interpolation: pushSnapshot(state.interpolation, snapshot),
    })),

  // Input
  input: defaultInput,
  inputSource: InputSource.Keyboard, // Default to keyboard (gamepad overrides on activity)
  setInput: (input) => set({ input }),
  setInputSource: (source) => set({ inputSource: source }),

  // Camera
  cameraMode: CameraMode.ThirdPerson,
  setCameraMode: (mode) => set({ cameraMode: mode }),

  // Toast
  toast: null,
  showToast: (message, duration = 1500) => {
    set({ toast: message });
    setTimeout(() => set({ toast: null }), duration);
  },

  // Video (multi-camera support)
  videoConnected: false,
  videoFps: 0,
  videoFrame: null,
  videoTimestamp: 0,
  cameraFrames: new Map(),
  cameraCount: 0,
  setVideoConnected: (connected) => set({ videoConnected: connected }),
  setVideoFps: (fps) => set({ videoFps: fps }),
  setVideoFrame: (frame, timestamp) =>
    set({ videoFrame: frame, videoTimestamp: timestamp }),
  setCameraFrame: (cameraId, url, timestamp) =>
    set((state) => {
      const newFrames = new Map(state.cameraFrames);
      newFrames.set(cameraId, { url, timestamp });
      return {
        cameraFrames: newFrames,
        cameraCount: newFrames.size,
        // Keep videoFrame updated with the latest frame for backwards compatibility
        videoFrame: url,
        videoTimestamp: timestamp,
      };
    }),

  // Sessions
  sessions: [],
  selectedSessionId: null,
  sessionsLoading: false,
  sessionsError: null,
  sessionRoverFilter: null,
  setSessions: (sessions) => set({ sessions, sessionsError: null }),
  selectSession: (sessionId) => set({ selectedSessionId: sessionId }),
  setSessionsLoading: (loading) => set({ sessionsLoading: loading }),
  setSessionsError: (error) => set({ sessionsError: error }),
  setSessionRoverFilter: (roverId) => set({ sessionRoverFilter: roverId }),

  // Infrastructure
  services: [],
  gpsStatus: null,
  setServices: (services) => set({ services }),
  setGpsStatus: (status) => set({ gpsStatus: status }),

  // Theme
  theme: "system",
  setTheme: (theme) => set({ theme }),
    }),
    {
      name: "console-storage",
      partialize: (state) => ({ theme: state.theme }),
    }
  )
);
