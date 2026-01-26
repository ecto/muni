import { useState, useEffect, useMemo, useCallback } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import {
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  Lightning,
  Gauge,
  Compass,
  Camera,
  CaretDown,
  CaretRight,
} from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { CameraMode } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";

const cameraModeLabels: Record<CameraMode, string> = {
  [CameraMode.ThirdPerson]: "3rd",
  [CameraMode.FirstPerson]: "1st",
  [CameraMode.FreeLook]: "Free",
};

function BatteryIcon({ voltage }: { voltage: number }) {
  if (voltage < 42)
    return <BatteryWarning className="h-3.5 w-3.5 text-destructive" weight="fill" />;
  if (voltage < 45)
    return <BatteryLow className="h-3.5 w-3.5 text-orange-500" weight="fill" />;
  return <BatteryFull className="h-3.5 w-3.5 text-green-500" weight="fill" />;
}

export function StatusPanel() {
  const connected = useConsoleStore((s) => s.connected);
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const setCameraMode = useConsoleStore((s) => s.setCameraMode);

  // Poll high-frequency data at 4Hz instead of subscribing
  const [telemetry, setTelemetry] = useState(useConsoleStore.getState().telemetry);
  const [latencyMs, setLatencyMs] = useState(0);
  const [displayPose, setDisplayPose] = useState({ x: 0, y: 0, theta: 0, roll: 0, pitch: 0 });
  useEffect(() => {
    const id = setInterval(() => {
      const state = useConsoleStore.getState();
      setTelemetry(state.telemetry);
      setLatencyMs(state.latencyMs);
      setDisplayPose(state.renderPose);
    }, 250);
    return () => clearInterval(id);
  }, []);

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

  return (
    <div className="w-56 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div
        className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          {collapsed ? <CaretRight className="h-3 w-3" /> : <CaretDown className="h-3 w-3" />}
          Status
        </span>
        <span className={`text-xs font-mono ${connected ? 'text-green-500' : 'text-red-500'}`}>
          {connected ? `${latencyMs}ms` : 'OFFLINE'}
        </span>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-2 space-y-2">
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
                    <span className="font-mono">{displayPose.x.toFixed(1)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">East-West position in meters from origin</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Y</span>
                    <span className="font-mono">{displayPose.y.toFixed(1)}</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">North-South position in meters from origin</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">θ</span>
                    <span className="font-mono">{((displayPose.theta * 180) / Math.PI).toFixed(0)}°</span>
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
                    <span className="font-mono">{((displayPose.roll * 180) / Math.PI).toFixed(1)}°</span>
                  </div>
                </TooltipTrigger>
                <TooltipContent side="right">Side-to-side tilt from IMU. 0° = level</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="flex justify-between cursor-help">
                    <span className="text-muted-foreground">Pitch</span>
                    <span className="font-mono">{((displayPose.pitch * 180) / Math.PI).toFixed(1)}°</span>
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
        </div>
      )}
    </div>
  );
}
