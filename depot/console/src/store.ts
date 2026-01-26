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
  type Alert,
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

// Channel health metrics for WebRTC diagnostics
export interface ChannelMetrics {
  name: string;
  messagesPerSec: number;
  lastMessageTime: number; // performance.now()
  history: number[]; // Last 30 samples of messagesPerSec
}

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
  // Note: Video frames are stored in a mutable store (videoFrameStore.ts) to avoid
  // React re-renders at 15fps per camera. Use getAllCameraFrames() from videoFrameStore
  // to read frames in useFrame or polling loops.
  videoConnected: boolean;
  videoFps: number;
  /** @deprecated Use getLatestFrame() from videoFrameStore instead */
  videoFrame: string | null;
  videoTimestamp: number;
  /** @deprecated Use getAllCameraFrames() from videoFrameStore instead */
  cameraFrames: Map<number, { url: string; timestamp: number }>;
  /** Number of active cameras (updated infrequently when cameras connect/disconnect) */
  cameraCount: number;
  setVideoConnected: (connected: boolean) => void;
  setVideoFps: (fps: number) => void;
  setVideoFrame: (frame: string | null, timestamp: number) => void;
  /** @deprecated Use setCameraFrame from videoFrameStore instead */
  setCameraFrame: (cameraId: number, url: string, timestamp: number) => void;
  /** Update camera count (called when cameras connect/disconnect, not on every frame) */
  setCameraCount: (count: number) => void;

  // Point cloud
  // Note: Point cloud data (points, reflectivity, tag) is stored in a mutable store
  // (pointCloudStore.ts) to avoid React re-renders at 5Hz. Use getPointCloudData()
  // from pointCloudStore to read data in useFrame or polling loops.
  pointCloudEnabled: boolean;
  setPointCloudEnabled: (enabled: boolean) => void;

  // Costmap overlay
  // Note: Costmap data is stored in a mutable store (costmapStore.ts) to avoid
  // React re-renders. Use getCostmapData() from costmapStore in useFrame.
  costmapEnabled: boolean;
  setCostmapEnabled: (enabled: boolean) => void;

  // Obstacle overlay
  // Note: Obstacle data is stored in a mutable store (obstacleStore.ts) to avoid
  // React re-renders. Use getObstacleData() from obstacleStore in useFrame.
  obstaclesEnabled: boolean;
  setObstaclesEnabled: (enabled: boolean) => void;

  // Gaussian splat (3D map)
  splatEnabled: boolean;
  setSplatEnabled: (enabled: boolean) => void;

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

  // Alerts (server-backed)
  alerts: Alert[];
  setAlerts: (alerts: Alert[]) => void;
  addAlert: (alert: Alert) => void;
  updateAlert: (id: string, updates: Partial<Alert>) => void;
  removeAlert: (id: string) => void;

  // Theme
  theme: Theme;
  setTheme: (theme: Theme) => void;

  // Channel metrics (WebRTC diagnostics)
  channelMetrics: Map<string, ChannelMetrics>;
  updateChannelMetrics: (name: string, metrics: Partial<ChannelMetrics>) => void;
  setAllChannelMetrics: (updates: Map<string, Partial<ChannelMetrics>>) => void;
  resetChannelMetrics: () => void;
}

const defaultTelemetry: Telemetry = {
  mode: Mode.Idle,
  pose: { x: 0, y: 0, theta: 0, roll: 0, pitch: 0 },
  power: { battery_voltage: 0, system_current: 0 },
  velocity: { linear: 0, angular: 0 },
  motor_temps: [0, 0, 0, 0],
  connected: false,
  latency_ms: 0,
  health: defaultHealth,
  cpu_percent: 0,
  mem_percent: 0,
  disk_percent: 0,
  modeChangedAt: Date.now(),
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
  setRovers: (newRovers) =>
    set((state) => {
      // If WebRTC is connected, the connected rover's telemetry is fresher
      // than discovery heartbeats — preserve all WebRTC-synced fields.
      if (state.connected && state.selectedRoverId) {
        const existing = state.rovers.find((r) => r.id === state.selectedRoverId);
        if (existing) {
          return {
            rovers: newRovers.map((r) =>
              r.id === state.selectedRoverId
                ? { ...r, online: true, mode: existing.mode, batteryVoltage: existing.batteryVoltage, lastSeen: existing.lastSeen }
                : r
            ),
          };
        }
      }
      return { rovers: newRovers };
    }),
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
    set((state) => {
      const next = { ...state.telemetry, ...partial };
      const prevMode = state.telemetry.mode;
      const nextMode = next.mode;

      // Disable overlays when entering sleep (sensors are powered down)
      if (nextMode === Mode.Sleep && prevMode !== Mode.Sleep) {
        return {
          telemetry: next,
          pointCloudEnabled: false,
          costmapEnabled: false,
          obstaclesEnabled: false,
        };
      }

      // Re-enable overlays when waking from sleep
      if (prevMode === Mode.Sleep && nextMode !== Mode.Sleep) {
        return {
          telemetry: next,
          pointCloudEnabled: true,
          costmapEnabled: true,
          obstaclesEnabled: true,
        };
      }

      return { telemetry: next };
    }),

  // Render pose and interpolation
  renderPose: { x: 0, y: 0, theta: 0, roll: 0, pitch: 0 },
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
  /** @deprecated Use setCameraFrame from videoFrameStore instead */
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
  setCameraCount: (count) => set({ cameraCount: count }),

  // Point cloud
  pointCloudEnabled: false,
  setPointCloudEnabled: (enabled) => set({ pointCloudEnabled: enabled }),

  // Costmap overlay
  costmapEnabled: true,
  setCostmapEnabled: (enabled) => set({ costmapEnabled: enabled }),

  // Obstacle overlay
  obstaclesEnabled: true,
  setObstaclesEnabled: (enabled) => set({ obstaclesEnabled: enabled }),

  // Gaussian splat
  splatEnabled: false,
  setSplatEnabled: (enabled) => set({ splatEnabled: enabled }),

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

  // Alerts (server-backed)
  alerts: [],
  setAlerts: (alerts) => set({ alerts }),
  addAlert: (alert) =>
    set((state) => {
      // Deduplicate by id
      if (state.alerts.some((a) => a.id === alert.id)) return state;
      return { alerts: [alert, ...state.alerts].slice(0, 200) };
    }),
  updateAlert: (id, updates) =>
    set((state) => ({
      alerts: state.alerts.map((a) =>
        a.id === id ? { ...a, ...updates } : a
      ),
    })),
  removeAlert: (id) =>
    set((state) => ({
      alerts: state.alerts.filter((a) => a.id !== id),
    })),

  // Theme
  theme: "system",
  setTheme: (theme) => set({ theme }),

  // Channel metrics
  channelMetrics: new Map([
    ["telemetry", { name: "telemetry", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
    ["video", { name: "video", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
    ["pointcloud", { name: "pointcloud", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
    ["costmap", { name: "costmap", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
    ["obstacles", { name: "obstacles", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
  ]),
  updateChannelMetrics: (name, metrics) =>
    set((state) => {
      const newMap = new Map(state.channelMetrics);
      const existing = newMap.get(name) || { name, messagesPerSec: 0, lastMessageTime: 0, history: [] };
      newMap.set(name, { ...existing, ...metrics });
      return { channelMetrics: newMap };
    }),
  setAllChannelMetrics: (updates) =>
    set((state) => {
      const newMap = new Map(state.channelMetrics);
      updates.forEach((metrics, name) => {
        const existing = newMap.get(name) || { name, messagesPerSec: 0, lastMessageTime: 0, history: [] };
        newMap.set(name, { ...existing, ...metrics });
      });
      return { channelMetrics: newMap };
    }),
  resetChannelMetrics: () =>
    set({
      channelMetrics: new Map([
        ["telemetry", { name: "telemetry", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
        ["video", { name: "video", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
        ["pointcloud", { name: "pointcloud", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
        ["costmap", { name: "costmap", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
        ["obstacles", { name: "obstacles", messagesPerSec: 0, lastMessageTime: 0, history: [] }],
      ]),
    }),
    }),
    {
      name: "console-storage",
      partialize: (state) => ({ theme: state.theme }),
    }
  )
);
