import { useState, useEffect, useMemo, useCallback, memo } from "react";
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

const BANK_TICKS = [0, 10, 20, 30, -10, -20, -30];
const PITCH_LINES = [-10, -5, 5, 10];

const AttitudeIndicator = memo(function AttitudeIndicator({
  roll,
  pitch,
}: {
  roll: number;
  pitch: number;
}) {
  const cx = 50;
  const cy = 50;
  const r = 38;
  // Convert radians to degrees for display; clamp pitch shift
  const rollDeg = (roll * 180) / Math.PI;
  const pitchShift = Math.max(-30, Math.min(30, ((pitch * 180) / Math.PI) * (r / 30)));

  return (
    <svg viewBox="0 0 100 100" className="w-20 h-20 mx-auto block">
      <defs>
        <clipPath id="ai-clip">
          <circle cx={cx} cy={cy} r={r} />
        </clipPath>
      </defs>

      {/* Background circle */}
      <circle cx={cx} cy={cy} r={r} className="fill-sky-900/30" />

      {/* Rotating group: ground + horizon + pitch marks */}
      <g
        clipPath="url(#ai-clip)"
        transform={`rotate(${-rollDeg}, ${cx}, ${cy})`}
      >
        {/* Ground half */}
        <rect
          x={cx - r - 5}
          y={cy + pitchShift}
          width={r * 2 + 10}
          height={r * 2 + 10}
          className="fill-amber-900/30"
        />
        {/* Horizon line */}
        <line
          x1={cx - r - 5}
          y1={cy + pitchShift}
          x2={cx + r + 5}
          y2={cy + pitchShift}
          className="stroke-emerald-400"
          strokeWidth={1.5}
        />
        {/* Pitch marks */}
        {PITCH_LINES.map((deg) => {
          const yOff = pitchShift - deg * (r / 30);
          const halfW = Math.abs(deg) === 10 ? 10 : 6;
          return (
            <line
              key={deg}
              x1={cx - halfW}
              y1={cy + yOff}
              x2={cx + halfW}
              y2={cy + yOff}
              className="stroke-stone-400/60"
              strokeWidth={0.75}
            />
          );
        })}
      </g>

      {/* Bank angle ticks (static, around top arc) */}
      {BANK_TICKS.map((deg) => {
        const angle = ((deg - 90) * Math.PI) / 180;
        const inner = r - 3;
        const outer = r + 1;
        return (
          <line
            key={deg}
            x1={cx + Math.cos(angle) * inner}
            y1={cy + Math.sin(angle) * inner}
            x2={cx + Math.cos(angle) * outer}
            y2={cy + Math.sin(angle) * outer}
            className={deg === 0 ? "stroke-emerald-400" : "stroke-stone-500/60"}
            strokeWidth={deg === 0 ? 1.5 : 0.75}
          />
        );
      })}

      {/* Center wings (fixed reference) */}
      <circle cx={cx} cy={cy} r={2} className="fill-emerald-400" />
      <line x1={cx - 16} y1={cy} x2={cx - 5} y2={cy} className="stroke-emerald-400" strokeWidth={1.5} />
      <line x1={cx + 5} y1={cy} x2={cx + 16} y2={cy} className="stroke-emerald-400" strokeWidth={1.5} />

      {/* Outer ring */}
      <circle cx={cx} cy={cy} r={r} fill="none" className="stroke-stone-600" strokeWidth={1} />
    </svg>
  );
});

export function StatusPanel() {
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const setCameraMode = useConsoleStore((s) => s.setCameraMode);

  // Poll high-frequency data at 4Hz instead of subscribing
  const [telemetry, setTelemetry] = useState(useConsoleStore.getState().telemetry);
  const [displayPose, setDisplayPose] = useState({ x: 0, y: 0, theta: 0, roll: 0, pitch: 0 });
  useEffect(() => {
    const id = setInterval(() => {
      const state = useConsoleStore.getState();
      setTelemetry(state.telemetry);
      setDisplayPose(state.renderPose);
    }, 250);
    return () => clearInterval(id);
  }, []);

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
    <div className="p-2 space-y-2">
      {/* Battery + Current merged row */}
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="space-y-1 cursor-help">
            <div className="flex items-center justify-between text-xs">
              <span className="flex items-center gap-1.5 text-muted-foreground">
                <BatteryIcon voltage={telemetry.power.battery_voltage} />
                {batteryPercent.toFixed(0)}%
                <span className="font-mono text-stone-900 dark:text-stone-100">
                  ({telemetry.power.battery_voltage.toFixed(1)}V)
                </span>
              </span>
              <span className="flex items-center gap-1 font-mono text-stone-900 dark:text-stone-100">
                <Lightning className="h-3 w-3 text-muted-foreground" weight="fill" />
                {telemetry.power.system_current.toFixed(1)}A
              </span>
            </div>
            <Progress value={batteryPercent} className="h-1.5" />
          </div>
        </TooltipTrigger>
        <TooltipContent side="right">12S LiPo. Critical &lt;42V, Warning &lt;45V, Full ~50.4V</TooltipContent>
      </Tooltip>

      {/* Velocity — no header */}
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

      {/* Position + Attitude — divider above */}
      <div className="pt-1 border-t border-border space-y-1.5">
        {/* X / Y / θ — vertical label + value */}
        <div className="grid grid-cols-3 gap-1 text-xs text-center text-stone-900 dark:text-stone-100">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help">
                <div className="text-muted-foreground">X</div>
                <div className="font-mono">{displayPose.x.toFixed(1)}</div>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">East-West position in meters from origin</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help">
                <div className="text-muted-foreground">Y</div>
                <div className="font-mono">{displayPose.y.toFixed(1)}</div>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">North-South position in meters from origin</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help">
                <div className="text-muted-foreground">&theta;</div>
                <div className="font-mono">{((displayPose.theta * 180) / Math.PI).toFixed(0)}&deg;</div>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Heading angle. 0&deg; = East, 90&deg; = North</TooltipContent>
          </Tooltip>
        </div>

        {/* Attitude indicator */}
        <AttitudeIndicator roll={displayPose.roll} pitch={displayPose.pitch} />

        {/* Roll / Pitch — vertical label + value, camera badge */}
        <div className="grid grid-cols-3 gap-1 text-xs text-center text-stone-900 dark:text-stone-100">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help">
                <div className="text-muted-foreground">Roll</div>
                <div className="font-mono">{((displayPose.roll * 180) / Math.PI).toFixed(1)}&deg;</div>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Side-to-side tilt from IMU. 0&deg; = level</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-help">
                <div className="text-muted-foreground">Pitch</div>
                <div className="font-mono">{((displayPose.pitch * 180) / Math.PI).toFixed(1)}&deg;</div>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Front-to-back tilt from IMU. 0&deg; = level</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center justify-center h-full">
                <Badge
                  variant="secondary"
                  className="text-[10px] h-4 px-1.5 cursor-pointer hover:bg-secondary/80 text-foreground"
                  onClick={(e) => {
                    e.stopPropagation();
                    cycleCameraMode();
                  }}
                >
                  📷{cameraModeLabels[cameraMode]}
                </Badge>
              </div>
            </TooltipTrigger>
            <TooltipContent side="right">Click to cycle: 3rd (chase), 1st (cockpit), Free (orbit)</TooltipContent>
          </Tooltip>
        </div>
      </div>
    </div>
  );
}
