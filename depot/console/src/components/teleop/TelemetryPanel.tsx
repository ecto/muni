import { useState, useMemo, useCallback } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { Sparkline, getChannelColor } from "@/components/ui/sparkline";
import {
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  Lightning,
  Gauge,
  Heart,
  CheckCircle,
  XCircle,
  Warning,
  Compass,
  Camera,
  CaretDown,
  CaretRight,
  Cpu,
  HardDrives,
  WifiHigh,
} from "@phosphor-icons/react";
import { useConsoleStore, type ChannelMetrics } from "@/store";
import { Mode, ModeLabels, CameraMode, WheelStatus } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";
import { useShallow } from "zustand/react/shallow";

const cameraModeLabels: Record<CameraMode, string> = {
  [CameraMode.ThirdPerson]: "3rd",
  [CameraMode.FirstPerson]: "1st",
  [CameraMode.FreeLook]: "Free",
};

const modeTooltips: Record<Mode, string> = {
  [Mode.Idle]: "Motors off, ready for commands",
  [Mode.Teleop]: "Manual control active",
  [Mode.Autonomous]: "Self-driving mode",
  [Mode.EStop]: "Emergency stop engaged",
  [Mode.Fault]: "System error detected",
  [Mode.Disabled]: "Teleop disabled",
};

function getModeVariant(
  mode: Mode
): "default" | "secondary" | "destructive" | "outline" {
  switch (mode) {
    case Mode.Teleop:
    case Mode.Autonomous:
      return "default";
    case Mode.EStop:
    case Mode.Fault:
      return "destructive";
    case Mode.Idle:
      return "secondary";
    default:
      return "outline";
  }
}

function BatteryIcon({ voltage }: { voltage: number }) {
  if (voltage < 42)
    return (
      <BatteryWarning className="h-3.5 w-3.5 text-destructive" weight="fill" />
    );
  if (voltage < 45)
    return <BatteryLow className="h-3.5 w-3.5 text-orange-500" weight="fill" />;
  return <BatteryFull className="h-3.5 w-3.5 text-green-500" weight="fill" />;
}

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

interface TelemetryPanelProps {
  onToggleLidar?: (enabled: boolean) => void;
}

// Channel configuration - defined outside component to avoid recreation
const CHANNEL_CONFIG = [
  { key: "telemetry", label: "Telem", tooltip: "Telemetry data channel (~30-60 msg/s)" },
  { key: "video", label: "Video", tooltip: "Video frames channel (~10-15 FPS per camera)" },
  { key: "pointcloud", label: "Cloud", tooltip: "LiDAR point cloud channel (~5 msg/s when enabled)" },
] as const;

export function TelemetryPanel({ onToggleLidar }: TelemetryPanelProps) {
  // Split selectors for granular updates
  const telemetry = useConsoleStore((s) => s.telemetry);
  const connected = useConsoleStore((s) => s.connected);
  const latencyMs = useConsoleStore((s) => s.latencyMs);
  const renderPose = useConsoleStore((s) => s.renderPose);
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const setCameraMode = useConsoleStore((s) => s.setCameraMode);
  const pointCloudEnabled = useConsoleStore((s) => s.pointCloudEnabled);
  const setPointCloudEnabled = useConsoleStore((s) => s.setPointCloudEnabled);
  const splatEnabled = useConsoleStore((s) => s.splatEnabled);
  const setSplatEnabled = useConsoleStore((s) => s.setSplatEnabled);
  // Select only the metrics values we need, not the whole Map
  // useShallow compares array elements by reference to avoid rerenders when Map updates
  const channelMetricsData = useConsoleStore(
    useShallow((s): (ChannelMetrics | undefined)[] =>
      CHANNEL_CONFIG.map(({ key }) => s.channelMetrics.get(key))
    )
  );

  const [collapsed, setCollapsed] = useState(false);

  const batteryPercent = useMemo(
    () => getBatteryPercent(telemetry.power.battery_voltage),
    [telemetry.power.battery_voltage]
  );

  const cycleCameraMode = useCallback(() => {
    const modes = [CameraMode.ThirdPerson, CameraMode.FirstPerson, CameraMode.FreeLook];
    const currentIndex = modes.indexOf(cameraMode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setCameraMode(modes[nextIndex]);
  }, [cameraMode, setCameraMode]);

  const handleToggleLidar = useCallback(() => {
    const newEnabled = !pointCloudEnabled;
    setPointCloudEnabled(newEnabled);
    onToggleLidar?.(newEnabled);
  }, [pointCloudEnabled, setPointCloudEnabled, onToggleLidar]);

  const handleToggleSplat = useCallback(() => {
    setSplatEnabled(!splatEnabled);
  }, [splatEnabled, setSplatEnabled]);

  return (
    <div className="w-56 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div
        className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          {collapsed ? <CaretRight className="h-3 w-3" /> : <CaretDown className="h-3 w-3" />}
          Telemetry
        </span>
        <span className={`text-xs font-mono ${connected ? 'text-green-500' : 'text-red-500'}`}>
          {connected ? `${latencyMs}ms` : 'OFFLINE'}
        </span>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-2 space-y-2">
          {/* Mode */}
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center justify-between text-xs cursor-help">
                <span className="text-muted-foreground">Mode</span>
                <Badge variant={getModeVariant(telemetry.mode)} className="text-xs h-5 text-foreground">
                  {ModeLabels[telemetry.mode]}
                </Badge>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">{modeTooltips[telemetry.mode]}</TooltipContent>
          </Tooltip>

          {/* Battery */}
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="space-y-1 cursor-help">
                <div className="flex items-center justify-between text-xs">
                  <span className="flex items-center gap-1.5 text-muted-foreground">
                    <BatteryIcon voltage={telemetry.power.battery_voltage} />
                    Battery
                  </span>
                  <span className="font-mono text-stone-900 dark:text-stone-100">
                    {batteryPercent.toFixed(0)}%
                    <span className="text-muted-foreground ml-1">
                      ({telemetry.power.battery_voltage.toFixed(1)}V)
                    </span>
                  </span>
                </div>
                <Progress value={batteryPercent} className="h-1.5" />
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">12S LiPo pack. Critical &lt;42V, Warning &lt;45V, Full ~50.4V</TooltipContent>
          </Tooltip>

          {/* Current */}
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center justify-between text-xs cursor-help">
                <span className="flex items-center gap-1.5 text-muted-foreground">
                  <Lightning className="h-3.5 w-3.5" weight="fill" />
                  Current
                </span>
                <span className="font-mono text-stone-900 dark:text-stone-100">
                  {telemetry.power.system_current.toFixed(1)}A
                </span>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Total system current draw from all motors and electronics</TooltipContent>
          </Tooltip>

          {/* Velocity */}
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Gauge className="h-3.5 w-3.5" weight="fill" />
              Velocity
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs text-stone-900 dark:text-stone-100">
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Lin</span>
                    <span className="font-mono">{telemetry.velocity.linear.toFixed(2)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Forward/backward speed in m/s</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Ang</span>
                    <span className="font-mono">{telemetry.velocity.angular.toFixed(2)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Rotation rate in rad/s (positive = CCW)</TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* Position */}
          <div className="space-y-1 pt-1 border-t border-border">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Compass className="h-3.5 w-3.5" weight="fill" />
              Position
            </div>
            <div className="grid grid-cols-3 gap-1 text-xs text-stone-900 dark:text-stone-100">
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">X</span>
                    <span className="font-mono">{renderPose.x.toFixed(1)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">East-West position in meters from origin</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Y</span>
                    <span className="font-mono">{renderPose.y.toFixed(1)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">North-South position in meters from origin</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">θ</span>
                    <span className="font-mono">{((renderPose.theta * 180) / Math.PI).toFixed(0)}°</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Heading angle. 0° = East, 90° = North</TooltipContent>
              </Tooltip>
            </div>
            {/* IMU Orientation (roll/pitch) */}
            <div className="grid grid-cols-2 gap-1 text-xs text-stone-900 dark:text-stone-100 mt-1">
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Roll</span>
                    <span className="font-mono">{((renderPose.roll * 180) / Math.PI).toFixed(1)}°</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Side-to-side tilt from IMU. 0° = level</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Pitch</span>
                    <span className="font-mono">{((renderPose.pitch * 180) / Math.PI).toFixed(1)}°</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Front-to-back tilt from IMU. 0° = level</TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* Camera mode */}
          <div className="flex items-center justify-between text-xs">
            <span className="flex items-center gap-1.5 text-muted-foreground">
              <Camera className="h-3.5 w-3.5" weight="fill" />
              Camera
            </span>
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant="secondary"
                  className="text-[10px] h-4 px-1.5 cursor-pointer hover:bg-secondary/80 text-foreground"
                  onClick={(e) => {
                    e.stopPropagation();
                    cycleCameraMode();
                  }}
                >
                  {cameraModeLabels[cameraMode]}
                </Badge>
              </TooltipTrigger>
              <TooltipContent side="right">Click to cycle: 3rd (chase), 1st (cockpit), Free (orbit)</TooltipContent>
            </Tooltip>
          </div>

          {/* Subsystem Health */}
          {connected && (
            <div className="space-y-1 pt-1 border-t border-border">
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
              </div>
            </div>
          )}

          {/* Channel Metrics */}
          {connected && (
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
          )}

          {/* Wheel Status */}
          {connected && (
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
          )}

          {/* System Resources */}
          {connected && (
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
          )}
        </div>
      )}
    </div>
  );
}
