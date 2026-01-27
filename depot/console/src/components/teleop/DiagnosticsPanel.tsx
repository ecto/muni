import { useState, useEffect, useCallback } from "react";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { Sparkline } from "@/components/ui/sparkline";
import { getChannelColor } from "@/lib/channelColor";
import {
  CheckCircle,
  XCircle,
  Warning,
} from "@phosphor-icons/react";
import { useConsoleStore, type ChannelMetrics } from "@/store";
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
  { key: "video", label: "Video", tooltip: "Video frames channel (~10-15 FPS per camera)" },
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

interface DiagnosticsPanelProps {
  onToggleLidar?: (enabled: boolean) => void;
}

export function DiagnosticsPanel({ onToggleLidar }: DiagnosticsPanelProps) {
  const connected = useConsoleStore((s) => s.connected);
  const pointCloudEnabled = useConsoleStore((s) => s.pointCloudEnabled);
  const setPointCloudEnabled = useConsoleStore((s) => s.setPointCloudEnabled);
  const splatEnabled = useConsoleStore((s) => s.splatEnabled);
  const setSplatEnabled = useConsoleStore((s) => s.setSplatEnabled);
  const costmapEnabled = useConsoleStore((s) => s.costmapEnabled);
  const setCostmapEnabled = useConsoleStore((s) => s.setCostmapEnabled);
  const obstaclesEnabled = useConsoleStore((s) => s.obstaclesEnabled);
  const setObstaclesEnabled = useConsoleStore((s) => s.setObstaclesEnabled);
  const channelMetricsData = useConsoleStore(
    useShallow((s): (ChannelMetrics | undefined)[] =>
      CHANNEL_CONFIG.map(({ key }) => s.channelMetrics.get(key))
    )
  );

  // Poll telemetry at 4Hz (always — no collapsed state)
  const [telemetry, setTelemetry] = useState(useConsoleStore.getState().telemetry);
  useEffect(() => {
    const id = setInterval(() => {
      setTelemetry(useConsoleStore.getState().telemetry);
    }, 250);
    return () => clearInterval(id);
  }, []);

  const handleToggleLidar = useCallback(() => {
    const newEnabled = !pointCloudEnabled;
    setPointCloudEnabled(newEnabled);
    onToggleLidar?.(newEnabled);
  }, [pointCloudEnabled, setPointCloudEnabled, onToggleLidar]);

  const handleToggleSplat = useCallback(() => {
    setSplatEnabled(!splatEnabled);
  }, [splatEnabled, setSplatEnabled]);

  const handleToggleCostmap = useCallback(() => {
    setCostmapEnabled(!costmapEnabled);
  }, [costmapEnabled, setCostmapEnabled]);

  const handleToggleObstacles = useCallback(() => {
    setObstaclesEnabled(!obstaclesEnabled);
  }, [obstaclesEnabled, setObstaclesEnabled]);

  if (!connected) return null;

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
      </div>

      {/* Channel Metrics — no header */}
      <div className="pt-1 border-t border-border space-y-0.5">
        {CHANNEL_CONFIG.map(({ key, label, tooltip }, index) => {
          const metrics = channelMetricsData[index];
          const rate = metrics?.messagesPerSec ?? 0;
          const lastTime = metrics?.lastMessageTime ?? 0;
          const history = metrics?.history ?? [];
          const color = getChannelColor(rate, lastTime, connected);

          return (
            <Tooltip key={key}>
              <TooltipTrigger asChild>
                <div className="flex items-center justify-between text-xs cursor-help">
                  <span className="text-muted-foreground w-10">{label}</span>
                  <Sparkline data={history} color={color} width={48} height={14} />
                  <span className="font-mono text-stone-900 dark:text-stone-100 w-10 text-right" style={{ color }}>
                    {rate.toFixed(0)}/s
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
