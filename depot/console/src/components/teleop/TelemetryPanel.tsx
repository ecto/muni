import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
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
} from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { Mode, ModeLabels, CameraMode, WheelStatus } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";

const cameraModeLabels: Record<CameraMode, string> = {
  [CameraMode.ThirdPerson]: "3rd",
  [CameraMode.FirstPerson]: "1st",
  [CameraMode.FreeLook]: "Free",
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

function HealthIndicator({ label, healthy }: { label: string; healthy: boolean }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-muted-foreground">{label}</span>
      {healthy ? (
        <CheckCircle className="h-3 w-3 text-green-500" weight="fill" />
      ) : (
        <XCircle className="h-3 w-3 text-muted-foreground/50" weight="fill" />
      )}
    </div>
  );
}

function WheelStatusIndicator({ label, status }: { label: string; status: WheelStatus }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-muted-foreground">{label}</span>
      {status === WheelStatus.Online ? (
        <CheckCircle className="h-3 w-3 text-green-500" weight="fill" />
      ) : status === WheelStatus.Degraded ? (
        <Warning className="h-3 w-3 text-orange-500" weight="fill" />
      ) : (
        <XCircle className="h-3 w-3 text-muted-foreground/50" weight="fill" />
      )}
    </div>
  );
}

export function TelemetryPanel() {
  const { telemetry, connected, renderPose, cameraMode, setCameraMode } = useConsoleStore();
  const [collapsed, setCollapsed] = useState(false);

  const batteryPercent = getBatteryPercent(telemetry.power.battery_voltage);

  const cycleCameraMode = () => {
    const modes = [CameraMode.ThirdPerson, CameraMode.FirstPerson, CameraMode.FreeLook];
    const currentIndex = modes.indexOf(cameraMode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setCameraMode(modes[nextIndex]);
  };

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
          {connected ? 'CONNECTED' : 'OFFLINE'}
        </span>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-2 space-y-2">
          {/* Mode */}
          <div className="flex items-center justify-between text-xs">
            <span className="text-muted-foreground">Mode</span>
            <Badge variant={getModeVariant(telemetry.mode)} className="text-xs h-5 text-foreground">
              {ModeLabels[telemetry.mode]}
            </Badge>
          </div>

          {/* Battery */}
          <div className="space-y-1">
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

          {/* Current */}
          <div className="flex items-center justify-between text-xs">
            <span className="flex items-center gap-1.5 text-muted-foreground">
              <Lightning className="h-3.5 w-3.5" weight="fill" />
              Current
            </span>
            <span className="font-mono text-stone-900 dark:text-stone-100">
              {telemetry.power.system_current.toFixed(1)}A
            </span>
          </div>

          {/* Velocity */}
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Gauge className="h-3.5 w-3.5" weight="fill" />
              Velocity
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs text-stone-900 dark:text-stone-100">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Lin</span>
                <span className="font-mono">{telemetry.velocity.linear.toFixed(2)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Ang</span>
                <span className="font-mono">{telemetry.velocity.angular.toFixed(2)}</span>
              </div>
            </div>
          </div>

          {/* Position */}
          <div className="space-y-1 pt-1 border-t border-border">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Compass className="h-3.5 w-3.5" weight="fill" />
              Position
            </div>
            <div className="grid grid-cols-3 gap-1 text-xs text-stone-900 dark:text-stone-100">
              <div className="flex justify-between">
                <span className="text-muted-foreground">X</span>
                <span className="font-mono">{renderPose.x.toFixed(1)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Y</span>
                <span className="font-mono">{renderPose.y.toFixed(1)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">θ</span>
                <span className="font-mono">{((renderPose.theta * 180) / Math.PI).toFixed(0)}°</span>
              </div>
            </div>
          </div>

          {/* Camera mode */}
          <div className="flex items-center justify-between text-xs">
            <span className="flex items-center gap-1.5 text-muted-foreground">
              <Camera className="h-3.5 w-3.5" weight="fill" />
              Camera
            </span>
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
          </div>

          {/* Subsystem Health */}
          {connected && (
            <div className="space-y-1 pt-1 border-t border-border">
              <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <Heart className="h-3.5 w-3.5" weight="fill" />
                Health
              </div>
              <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
                <HealthIndicator label="CAN" healthy={telemetry.health.can_healthy} />
                <HealthIndicator label="Camera" healthy={telemetry.health.camera_active} />
                <HealthIndicator label="GPS" healthy={telemetry.health.gps_fix} />
                <HealthIndicator label="LiDAR" healthy={telemetry.health.lidar_active} />
                <HealthIndicator label="SLAM" healthy={telemetry.health.slam_running} />
                <HealthIndicator label="Rec" healthy={telemetry.health.recording_active} />
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
        </div>
      )}
    </div>
  );
}
