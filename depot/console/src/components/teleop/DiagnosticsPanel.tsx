import { useState, useEffect, useCallback, useRef } from "react";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { Slider } from "@/components/ui/slider";
import { Sparkline } from "@/components/ui/sparkline";
import { getChannelColor } from "@/lib/channelColor";
import {
  CheckCircle,
  XCircle,
  Warning,
} from "@phosphor-icons/react";
import { useConsoleStore, type ChannelMetrics } from "@/store";
import { clearAccumulatedCloud } from "@/lib/accumulatedCloudStore";
import { WheelStatus } from "@/lib/types";
import { useShallow } from "zustand/react/shallow";

function HealthIndicator({ label, healthy, tooltip }: { label: string; healthy: boolean; tooltip: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className="flex items-center justify-between text-xs cursor-help">
          <span className="text-muted-foreground">{label}</span>
          {healthy ? (
            <CheckCircle className="h-3 w-3 text-green-500" weight="fill" />
          ) : (
            <XCircle className="h-3 w-3 text-muted-foreground/50" weight="fill" />
          )}
        </div>
      </TooltipTrigger>
      <TooltipContent side="right">{tooltip}</TooltipContent>
    </Tooltip>
  );
}

const wheelLabels = ["FL", "FR", "RL", "RR"] as const;
const wheelTooltips: Record<string, string> = {
  FL: "Front-Left wheel motor",
  FR: "Front-Right wheel motor",
  RL: "Rear-Left wheel motor",
  RR: "Rear-Right wheel motor",
};

function WheelDot({ label, status }: { label: string; status: WheelStatus }) {
  const color =
    status === WheelStatus.Online
      ? "text-green-500"
      : status === WheelStatus.Degraded
        ? "text-orange-500"
        : "text-muted-foreground/50";
  const icon =
    status === WheelStatus.Online ? (
      <CheckCircle className="h-3 w-3" weight="fill" />
    ) : status === WheelStatus.Degraded ? (
      <Warning className="h-3 w-3" weight="fill" />
    ) : (
      <XCircle className="h-3 w-3" weight="fill" />
    );

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className={`flex items-center gap-0.5 text-xs cursor-help ${color}`}>
          {label}{icon}
        </span>
      </TooltipTrigger>
      <TooltipContent side="bottom">{wheelTooltips[label]}</TooltipContent>
    </Tooltip>
  );
}

const CHANNEL_CONFIG = [
  { key: "telemetry", label: "Telem", tooltip: "Telemetry data channel (~30-60 msg/s)" },
  { key: "video", label: "FPS", tooltip: "Video decode frame rate (~10-15 FPS per camera)" },
  { key: "pointcloud", label: "Cloud", tooltip: "LiDAR point cloud channel (~5 msg/s when enabled)" },
  { key: "costmap", label: "Costmap", tooltip: "Occupancy costmap channel (~5 msg/s when enabled)" },
  { key: "obstacles", label: "Objects", tooltip: "Obstacle detections channel (~5 msg/s when enabled)" },
] as const;

const LIDAR_WORK_STATE_LABELS: Record<number, string> = {
  1: "Sampling",
  2: "Idle",
  3: "Ready",
  4: "Error",
};

function lidarWorkStateLabel(state: number): string {
  return LIDAR_WORK_STATE_LABELS[state] ?? "\u2013";
}

function lidarTempColor(temp: number): string {
  if (temp > 70) return "text-red-500";
  if (temp >= 50) return "text-yellow-500";
  return "text-green-500";
}

function lidarIconColor(state: number): string {
  if (state === 1) return "text-green-500";   // Sampling (active)
  if (state === 3) return "text-green-500";   // Ready
  if (state === 2) return "text-yellow-500";  // Idle
  if (state === 4) return "text-red-500";     // Error
  return "text-muted-foreground/50";
}

function getLatencyColor(ms: number): string {
  if (ms === 0) return "#6b7280";
  if (ms > 200) return "#ef4444";
  if (ms > 100) return "#f59e0b";
  return "#22c55e";
}

function getFpsColor(fps: number): string {
  if (fps === 0) return "#6b7280";
  if (fps < 5) return "#ef4444";
  if (fps < 10) return "#f59e0b";
  return "#22c55e";
}

const LATENCY_HISTORY_LEN = 30;

interface DiagnosticsPanelProps {
  onToggleLidar?: (enabled: boolean) => void;
}

export function DiagnosticsPanel({ onToggleLidar }: DiagnosticsPanelProps) {
  const connected = useConsoleStore((s) => s.connected);
  const pointCloudEnabled = useConsoleStore((s) => s.pointCloudEnabled);
  const setPointCloudEnabled = useConsoleStore((s) => s.setPointCloudEnabled);
  const lidarStreamRate = useConsoleStore((s) => s.lidarStreamRate);
  const setLidarStreamRate = useConsoleStore((s) => s.setLidarStreamRate);
  const lidarMaxPoints = useConsoleStore((s) => s.lidarMaxPoints);
  const setLidarMaxPoints = useConsoleStore((s) => s.setLidarMaxPoints);
  const splatEnabled = useConsoleStore((s) => s.splatEnabled);
  const setSplatEnabled = useConsoleStore((s) => s.setSplatEnabled);
  const costmapEnabled = useConsoleStore((s) => s.costmapEnabled);
  const setCostmapEnabled = useConsoleStore((s) => s.setCostmapEnabled);
  const obstaclesEnabled = useConsoleStore((s) => s.obstaclesEnabled);
  const setObstaclesEnabled = useConsoleStore((s) => s.setObstaclesEnabled);
  const accumulatedCloudEnabled = useConsoleStore((s) => s.accumulatedCloudEnabled);
  const setAccumulatedCloudEnabled = useConsoleStore((s) => s.setAccumulatedCloudEnabled);
  const channelMetricsData = useConsoleStore(
    useShallow((s): (ChannelMetrics | undefined)[] =>
      CHANNEL_CONFIG.map(({ key }) => s.channelMetrics.get(key))
    )
  );

  // Poll telemetry + latency at 4Hz (always — no collapsed state)
  const [telemetry, setTelemetry] = useState(useConsoleStore.getState().telemetry);
  const [latencyMs, setLatencyMs] = useState(useConsoleStore.getState().latencyMs);
  const latencyHistoryRef = useRef<number[]>([]);
  const [latencyHistory, setLatencyHistory] = useState<number[]>([]);
  useEffect(() => {
    const id = setInterval(() => {
      const state = useConsoleStore.getState();
      setTelemetry(state.telemetry);
      setLatencyMs(state.latencyMs);
      const h = latencyHistoryRef.current;
      h.push(state.latencyMs);
      if (h.length > LATENCY_HISTORY_LEN) h.shift();
      setLatencyHistory([...h]);
    }, 250);
    return () => clearInterval(id);
  }, []);

  const handleToggleLidar = useCallback(() => {
    const newEnabled = !pointCloudEnabled;
    setPointCloudEnabled(newEnabled);
    onToggleLidar?.(newEnabled);
  }, [pointCloudEnabled, setPointCloudEnabled, onToggleLidar]);

  const handleLidarRateChange = useCallback((value: number[]) => {
    setLidarStreamRate(value[0]);
    onToggleLidar?.(true);
  }, [setLidarStreamRate, onToggleLidar]);

  const handleLidarDensityChange = useCallback((value: number[]) => {
    setLidarMaxPoints(value[0]);
    onToggleLidar?.(true);
  }, [setLidarMaxPoints, onToggleLidar]);

  const handleToggleSplat = useCallback(() => {
    setSplatEnabled(!splatEnabled);
  }, [splatEnabled, setSplatEnabled]);

  const handleToggleCostmap = useCallback(() => {
    setCostmapEnabled(!costmapEnabled);
  }, [costmapEnabled, setCostmapEnabled]);

  const handleToggleObstacles = useCallback(() => {
    setObstaclesEnabled(!obstaclesEnabled);
  }, [obstaclesEnabled, setObstaclesEnabled]);

  const handleToggleAccumulatedCloud = useCallback(() => {
    setAccumulatedCloudEnabled(!accumulatedCloudEnabled);
  }, [accumulatedCloudEnabled, setAccumulatedCloudEnabled]);

  const handleClearAccumulatedCloud = useCallback(() => {
    clearAccumulatedCloud();
  }, []);

  return (
    <div className="p-2 space-y-2">
      {/* Health grid — no header */}
      <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
        <HealthIndicator label="CAN" healthy={telemetry.health.can_healthy} tooltip="CAN bus communication with motor controllers" />
        <HealthIndicator label="Camera" healthy={telemetry.health.camera_active} tooltip="360\u00b0 camera feed active" />
        <HealthIndicator label="GPS" healthy={telemetry.health.gps_fix} tooltip="RTK GPS has valid fix" />
        {/* LiDAR — icon color encodes work state, temp inline */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center justify-between text-xs cursor-help">
              <span className="text-muted-foreground">LiDAR</span>
              <span className="flex items-center gap-1">
                {telemetry.health.lidar_active && telemetry.lidar && (
                  <span className={`text-[11px] font-mono ${lidarTempColor(telemetry.lidar.temp_c)}`}>{telemetry.lidar.temp_c.toFixed(0)}&deg;</span>
                )}
                {telemetry.health.lidar_active ? (
                  <CheckCircle className={`h-3 w-3 ${telemetry.lidar ? lidarIconColor(telemetry.lidar.work_state) : 'text-green-500'}`} weight="fill" />
                ) : (
                  <XCircle className="h-3 w-3 text-muted-foreground/50" weight="fill" />
                )}
              </span>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">
            {telemetry.health.lidar_active
              ? `LiDAR ${telemetry.lidar ? lidarWorkStateLabel(telemetry.lidar.work_state) : 'active'}${telemetry.lidar ? ` \u00b7 ${telemetry.lidar.temp_c.toFixed(0)}\u00b0C` : ''}`
              : "LiDAR sensor not active"}
          </TooltipContent>
        </Tooltip>
        <HealthIndicator label="SLAM" healthy={telemetry.health.slam_running} tooltip="Simultaneous Localization and Mapping running" />
        <HealthIndicator label="Rec" healthy={telemetry.health.recording_active} tooltip="Session recording to .rrd file" />
      </div>

      {/* Layer toggles */}
      <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
        {/* LiDAR toggle (only when sensor active) */}
        {telemetry.health.lidar_active ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <div
                className="flex items-center justify-between text-xs cursor-pointer hover:bg-muted/50 rounded px-0.5 -mx-0.5"
                onClick={handleToggleLidar}
              >
                <span className="text-muted-foreground">LiDAR</span>
                <Badge
                  variant={pointCloudEnabled ? "default" : "secondary"}
                  className="text-[10px] h-4 px-1.5 text-foreground"
                >
                  {pointCloudEnabled ? "ON" : "OFF"}
                </Badge>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Click to toggle point cloud streaming</TooltipContent>
          </Tooltip>
        ) : (
          <div />
        )}
        {/* Splat toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className="flex items-center justify-between text-xs cursor-pointer hover:bg-muted/50 rounded px-0.5 -mx-0.5"
              onClick={handleToggleSplat}
            >
              <span className="text-muted-foreground">Splat</span>
              <Badge
                variant={splatEnabled ? "default" : "secondary"}
                className="text-[10px] h-4 px-1.5 text-foreground"
              >
                {splatEnabled ? "ON" : "OFF"}
              </Badge>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Click to toggle 3D Gaussian splat map</TooltipContent>
        </Tooltip>
        {/* Costmap toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className="flex items-center justify-between text-xs cursor-pointer hover:bg-muted/50 rounded px-0.5 -mx-0.5"
              onClick={handleToggleCostmap}
            >
              <span className="text-muted-foreground">Costmap</span>
              <Badge
                variant={costmapEnabled ? "default" : "secondary"}
                className="text-[10px] h-4 px-1.5 text-foreground"
              >
                {costmapEnabled ? "ON" : "OFF"}
              </Badge>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Click to toggle costmap overlay</TooltipContent>
        </Tooltip>
        {/* Obstacles toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className="flex items-center justify-between text-xs cursor-pointer hover:bg-muted/50 rounded px-0.5 -mx-0.5"
              onClick={handleToggleObstacles}
            >
              <span className="text-muted-foreground">Objects</span>
              <Badge
                variant={obstaclesEnabled ? "default" : "secondary"}
                className="text-[10px] h-4 px-1.5 text-foreground"
              >
                {obstaclesEnabled ? "ON" : "OFF"}
              </Badge>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Click to toggle obstacle overlay</TooltipContent>
        </Tooltip>
        {/* Map Cloud toggle + clear */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className="flex items-center justify-between text-xs cursor-pointer hover:bg-muted/50 rounded px-0.5 -mx-0.5"
              onClick={handleToggleAccumulatedCloud}
              onContextMenu={(e) => {
                e.preventDefault();
                handleClearAccumulatedCloud();
              }}
            >
              <span className="text-muted-foreground">Map</span>
              <Badge
                variant={accumulatedCloudEnabled ? "default" : "secondary"}
                className="text-[10px] h-4 px-1.5 text-foreground"
              >
                {accumulatedCloudEnabled ? "ON" : "OFF"}
              </Badge>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">Accumulated LiDAR map cloud (right-click to clear)</TooltipContent>
        </Tooltip>
      </div>

      {/* LiDAR stream config (visible when streaming) */}
      {pointCloudEnabled && telemetry.health.lidar_active && (
        <div className="space-y-1.5">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2 text-xs cursor-help">
                <span className="text-muted-foreground w-8">{lidarStreamRate}Hz</span>
                <Slider
                  min={1} max={10} step={1}
                  value={[lidarStreamRate]}
                  onValueChange={handleLidarRateChange}
                  className="[&_[data-slot=slider-track]]:h-0.5 [&_[data-slot=slider-thumb]]:h-2.5 [&_[data-slot=slider-thumb]]:w-2.5 [&_[data-slot=slider-range]]:bg-green-500"
                />
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Streaming rate (1–10 Hz)</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2 text-xs cursor-help">
                <span className="text-muted-foreground w-8">{(lidarMaxPoints / 1000).toFixed(1)}k</span>
                <Slider
                  min={500} max={5000} step={500}
                  value={[lidarMaxPoints]}
                  onValueChange={handleLidarDensityChange}
                  className="[&_[data-slot=slider-track]]:h-0.5 [&_[data-slot=slider-thumb]]:h-2.5 [&_[data-slot=slider-thumb]]:w-2.5 [&_[data-slot=slider-range]]:bg-green-500"
                />
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Max points per frame (500–5000)</TooltipContent>
          </Tooltip>
        </div>
      )}

      {/* Channel Metrics — no header */}
      <div className="pt-1 border-t border-border space-y-0.5">
        {/* Latency */}
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center justify-between text-xs cursor-help">
              <span className="text-muted-foreground w-10">Ping</span>
              <Sparkline data={latencyHistory} color={getLatencyColor(latencyMs)} width={48} height={14} />
              <span className="font-mono w-10 text-right" style={{ color: getLatencyColor(latencyMs) }}>
                {latencyMs}ms
              </span>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">WebSocket round-trip latency</TooltipContent>
        </Tooltip>
        {CHANNEL_CONFIG.map(({ key, label, tooltip }, index) => {
          const metrics = channelMetricsData[index];
          const rate = metrics?.messagesPerSec ?? 0;
          const lastTime = metrics?.lastMessageTime ?? 0;
          const history = metrics?.history ?? [];
          const color = key === "video" ? getFpsColor(rate) : getChannelColor(rate, lastTime, connected);

          return (
            <Tooltip key={key}>
              <TooltipTrigger asChild>
                <div className="flex items-center justify-between text-xs cursor-help">
                  <span className="text-muted-foreground w-10">{label}</span>
                  <Sparkline data={history} color={color} width={48} height={14} />
                  <span className="font-mono text-stone-900 dark:text-stone-100 w-10 text-right" style={{ color }}>
                    {rate.toFixed(0)}{key === "video" ? "fps" : "/s"}
                  </span>
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">{tooltip}</TooltipContent>
            </Tooltip>
          );
        })}
      </div>

      {/* Wheels — single inline row */}
      <div className="pt-1 border-t border-border flex items-center justify-between">
        {wheelLabels.map((label, i) => (
          <WheelDot key={label} label={label} status={telemetry.health.wheel_status[i]} />
        ))}
      </div>

      {/* System — single row, 3 cols */}
      <div className="grid grid-cols-3 gap-1 text-xs">
        <Tooltip>
          <TooltipTrigger asChild>
            <span className={`font-mono cursor-help ${telemetry.cpu_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
              <span className="text-muted-foreground">CPU </span>{telemetry.cpu_percent}%
            </span>
          </TooltipTrigger>
          <TooltipContent side="bottom">Jetson CPU utilization</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <span className={`font-mono cursor-help ${telemetry.mem_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
              <span className="text-muted-foreground">Mem </span>{telemetry.mem_percent}%
            </span>
          </TooltipTrigger>
          <TooltipContent side="bottom">System memory usage</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <span className={`font-mono cursor-help ${telemetry.disk_percent > 90 ? 'text-red-500' : telemetry.disk_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
              <span className="text-muted-foreground">Disk </span>{telemetry.disk_percent}%
            </span>
          </TooltipTrigger>
          <TooltipContent side="bottom">Root filesystem usage</TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
