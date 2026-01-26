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
  Heart,
  CheckCircle,
  XCircle,
  Warning,
  CaretDown,
  CaretRight,
  Cpu,
  HardDrives,
  WifiHigh,
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

const wheelTooltips: Record<string, string> = {
  FL: "Front-Left wheel motor",
  FR: "Front-Right wheel motor",
  RL: "Rear-Left wheel motor",
  RR: "Rear-Right wheel motor",
};

function WheelStatusIndicator({ label, status }: { label: string; status: WheelStatus }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className="flex items-center justify-between text-xs cursor-help">
          <span className="text-muted-foreground">{label}</span>
          {status === WheelStatus.Online ? (
            <CheckCircle className="h-3 w-3 text-green-500" weight="fill" />
          ) : status === WheelStatus.Degraded ? (
            <Warning className="h-3 w-3 text-orange-500" weight="fill" />
          ) : (
            <XCircle className="h-3 w-3 text-muted-foreground/50" weight="fill" />
          )}
        </div>
      </TooltipTrigger>
      <TooltipContent side="right">{wheelTooltips[label]}</TooltipContent>
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

  // Collapsed by default — diagnostics are on-demand
  const [collapsed, setCollapsed] = useState(false);

  // Poll telemetry at 4Hz only when expanded (saves work when collapsed)
  const [telemetry, setTelemetry] = useState(useConsoleStore.getState().telemetry);
  useEffect(() => {
    if (collapsed) return;
    const id = setInterval(() => {
      setTelemetry(useConsoleStore.getState().telemetry);
    }, 250);
    return () => clearInterval(id);
  }, [collapsed]);

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

  return (
    <div className="w-56 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div
        className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          {collapsed ? <CaretRight className="h-3 w-3" /> : <CaretDown className="h-3 w-3" />}
          Diagnostics
        </span>
      </div>

      {/* Content — only rendered when expanded and connected */}
      {!collapsed && connected && (
        <div className="p-2 space-y-2">
          {/* Health */}
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Heart className="h-3.5 w-3.5" weight="fill" />
              Health
            </div>
            <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
              <HealthIndicator label="CAN" healthy={telemetry.health.can_healthy} tooltip="CAN bus communication with motor controllers" />
              <HealthIndicator label="Camera" healthy={telemetry.health.camera_active} tooltip="360° camera feed active" />
              <HealthIndicator label="GPS" healthy={telemetry.health.gps_fix} tooltip="RTK GPS has valid fix" />
              {/* LiDAR with toggle when active */}
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
                <HealthIndicator label="LiDAR" healthy={false} tooltip="LiDAR sensor not active" />
              )}
              <HealthIndicator label="SLAM" healthy={telemetry.health.slam_running} tooltip="Simultaneous Localization and Mapping running" />
              <HealthIndicator label="Rec" healthy={telemetry.health.recording_active} tooltip="Session recording to .rrd file" />
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
          </div>

          {/* Channel Metrics */}
          <div className="space-y-1 pt-1 border-t border-border">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <WifiHigh className="h-3.5 w-3.5" weight="fill" />
              Channels
            </div>
            <div className="space-y-0.5">
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
          </div>

          {/* Wheel Status */}
          <div className="space-y-1 pt-1 border-t border-border">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              Wheels
            </div>
            <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
              <WheelStatusIndicator label="FL" status={telemetry.health.wheel_status[0]} />
              <WheelStatusIndicator label="FR" status={telemetry.health.wheel_status[1]} />
              <WheelStatusIndicator label="RL" status={telemetry.health.wheel_status[2]} />
              <WheelStatusIndicator label="RR" status={telemetry.health.wheel_status[3]} />
            </div>
          </div>

          {/* System Resources */}
          <div className="space-y-1 pt-1 border-t border-border">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Cpu className="h-3.5 w-3.5" weight="fill" />
              System
            </div>
            <div className="space-y-1">
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex items-center justify-between text-xs cursor-help">
                    <span className="text-muted-foreground">CPU</span>
                    <span className={`font-mono ${telemetry.cpu_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
                      {telemetry.cpu_percent}%
                    </span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Jetson CPU utilization</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex items-center justify-between text-xs cursor-help">
                    <span className="text-muted-foreground">Mem</span>
                    <span className={`font-mono ${telemetry.mem_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
                      {telemetry.mem_percent}%
                    </span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">System memory usage</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex items-center justify-between text-xs cursor-help">
                    <span className="flex items-center gap-1 text-muted-foreground">
                      <HardDrives className="h-3 w-3" />
                      Disk
                    </span>
                    <span className={`font-mono ${telemetry.disk_percent > 90 ? 'text-red-500' : telemetry.disk_percent > 80 ? 'text-orange-500' : 'text-stone-900 dark:text-stone-100'}`}>
                      {telemetry.disk_percent}%
                    </span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Root filesystem usage</TooltipContent>
              </Tooltip>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
