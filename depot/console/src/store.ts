import { create } from "zustand";
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
} from "@/lib/types";

interface ConsoleState {
  // Fleet management
  rovers: RoverInfo[];
  selectedRoverId: string | null;
  setRovers: (rovers: RoverInfo[]) => void;
  updateRover: (id: string, updates: Partial<RoverInfo>) => void;
  selectRover: (id: string | null) => void;

  // Connection (for teleop)
  roverAddress: string;
  videoAddress: string;
  connected: boolean;
  latencyMs: number;
  setRoverAddress: (address: string) => void;
  setVideoAddress: (address: string) => void;
  setConnected: (connected: boolean) => void;
  setLatency: (ms: number) => void;

  // Telemetry
  telemetry: Telemetry;
  updateTelemetry: (partial: Partial<Telemetry>) => void;

  // Interpolated pose for smooth rendering
  renderPose: Pose;
  setRenderPose: (pose: Pose) => void;

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

  // Video
  videoConnected: boolean;
  videoFps: number;
  videoFrame: string | null;
  videoTimestamp: number;
  setVideoConnected: (connected: boolean) => void;
  setVideoFps: (fps: number) => void;
  setVideoFrame: (frame: string | null, timestamp: number) => void;

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
}

const defaultTelemetry: Telemetry = {
  mode: Mode.Disabled,
  pose: { x: 0, y: 0, theta: 0 },
  power: { battery_voltage: 0, system_current: 0 },
  velocity: { linear: 0, angular: 0 },
  motor_temps: [0, 0, 0, 0],
  connected: false,
  latency_ms: 0,
};

const defaultInput: GamepadInput = {
  linear: 0,
  angular: 0,
  toolAxis: 0,
  actionA: false,
  actionB: false,
  estop: false,
  enable: false,
  boost: false,
  cameraYaw: 0,
  cameraPitch: 0,
};

export const useConsoleStore = create<ConsoleState>((set) => ({
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
          roverAddress: rover.address,
          videoAddress: rover.videoAddress,
        };
      }
      return { selectedRoverId: id };
    });
  },

  // Connection
  roverAddress: "ws://localhost:4850",
  videoAddress: "ws://localhost:4851",
  connected: false,
  latencyMs: 0,
  setRoverAddress: (address) => set({ roverAddress: address }),
  setVideoAddress: (address) => set({ videoAddress: address }),
  setConnected: (connected) => set({ connected }),
  setLatency: (ms) => set({ latencyMs: ms }),

  // Telemetry
  telemetry: defaultTelemetry,
  updateTelemetry: (partial) =>
    set((state) => ({
      telemetry: { ...state.telemetry, ...partial },
    })),

  // Render pose
  renderPose: { x: 0, y: 0, theta: 0 },
  setRenderPose: (pose) => set({ renderPose: pose }),

  // Input
  input: defaultInput,
  inputSource: InputSource.None,
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

  // Video
  videoConnected: false,
  videoFps: 0,
  videoFrame: null,
  videoTimestamp: 0,
  setVideoConnected: (connected) => set({ videoConnected: connected }),
  setVideoFps: (fps) => set({ videoFps: fps }),
  setVideoFrame: (frame, timestamp) =>
    set({ videoFrame: frame, videoTimestamp: timestamp }),

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
}));
