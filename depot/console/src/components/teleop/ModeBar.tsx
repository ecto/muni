import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { Play, Stop, CircleNotch, Robot, Moon, Sun, ArrowCounterClockwise } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { Mode, ModeLabels } from "@/lib/types";

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

const modeTooltips: Record<Mode, string> = {
  [Mode.Idle]: "Motors off, ready for commands",
  [Mode.Teleop]: "Manual control active",
  [Mode.Autonomous]: "Self-driving mode",
  [Mode.EStop]: "Emergency stop engaged",
  [Mode.Fault]: "System error detected",
  [Mode.Disabled]: "Idle",
  [Mode.Sleep]: "Sensors powered down",
};

/** Map fault code to human-readable description. */
const faultDescriptions: Record<number, string> = {
  1: "Watchdog timeout — control loop or policy hung",
};

interface ModeBarProps {
  sendSleep: () => void;
  sendWake: () => void;
  sendEnable: () => void;
  sendDisable: () => void;
  sendAutonomous: () => void;
  sendStopAutonomy: () => void;
  sendClearFault: () => void;
}

export function ModeBar({
  sendSleep,
  sendWake,
  sendEnable,
  sendDisable,
  sendAutonomous,
  sendStopAutonomy,
  sendClearFault,
}: ModeBarProps) {
  const telemetryMode = useConsoleStore((s) => s.telemetry.mode);
  const connected = useConsoleStore((s) => s.connected);
  const modeChangedAt = useConsoleStore((s) => s.telemetry.modeChangedAt);
  const faultCode = useConsoleStore((s) => s.telemetry.faultCode);

  const isDisabled = telemetryMode === Mode.Disabled || telemetryMode === Mode.Idle;
  const isTeleop = telemetryMode === Mode.Teleop;
  const isAutonomous = telemetryMode === Mode.Autonomous;
  const isSleep = telemetryMode === Mode.Sleep;
  const isFault = telemetryMode === Mode.Fault;

  // Pending transition: store the label and the mode it was set in.
  // When mode changes, the pending label is automatically cleared.
  const [pending, setPending] = useState<{ label: string; mode: Mode } | null>(null);
  const pendingLabel = pending && pending.mode === telemetryMode ? pending.label : null;

  // Timeout fallback if command doesn't result in mode change
  useEffect(() => {
    if (pending !== null) {
      const timeout = setTimeout(() => setPending(null), 3000);
      return () => clearTimeout(timeout);
    }
  }, [pending]);

  // Elapsed time since last mode change
  const [modeElapsed, setModeElapsed] = useState("");
  useEffect(() => {
    const tick = () => {
      const secs = Math.floor((Date.now() - modeChangedAt) / 1000);
      if (secs >= 3600) {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const s = secs % 60;
        setModeElapsed(`${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`);
      } else {
        const m = Math.floor(secs / 60);
        const s = secs % 60;
        setModeElapsed(`${m}:${String(s).padStart(2, "0")}`);
      }
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [modeChangedAt]);

  return (
    <div className="flex items-center gap-3 bg-card/90 backdrop-blur-sm border border-border rounded-lg px-3 py-1.5 shadow-lg">
      {/* Mode badge + elapsed */}
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="flex items-center gap-2 cursor-help">
            <Badge variant={getModeVariant(telemetryMode)} className="text-xs h-5 text-foreground">
              {ModeLabels[telemetryMode]}
            </Badge>
            <span className="text-xs font-mono text-muted-foreground tabular-nums">{modeElapsed}</span>
          </div>
        </TooltipTrigger>
        <TooltipContent side="bottom">{modeTooltips[telemetryMode]}</TooltipContent>
      </Tooltip>

      {/* Separator */}
      <div className="w-px h-4 bg-border" />

      {/* Action buttons or pending spinner */}
      {pendingLabel ? (
        <Button variant="secondary" size="sm" disabled className="gap-2 h-7">
          <CircleNotch className="h-3.5 w-3.5 animate-spin" />
          <span className="text-xs">{pendingLabel}...</span>
        </Button>
      ) : (
        <>
          {/* Idle/Disabled: Sleep, Autonomy, Enable */}
          {isDisabled && (
            <>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={!connected}
                    onClick={() => { setPending({ label: "Sleeping", mode: telemetryMode }); sendSleep(); }}
                    className="gap-1.5 h-7 border-indigo-500 text-indigo-400 hover:bg-indigo-600 hover:text-white"
                  >
                    <Moon className="h-3.5 w-3.5" weight="fill" />
                    Sleep
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Power down sensors</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="default"
                    size="sm"
                    disabled={!connected}
                    onClick={() => { setPending({ label: "Entering autonomy", mode: telemetryMode }); sendAutonomous(); }}
                    className="gap-1.5 h-7 bg-green-600 hover:bg-green-500 disabled:bg-gray-600"
                  >
                    <Robot className="h-3.5 w-3.5" weight="fill" />
                    Autonomy
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Enter autonomous mode</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="default"
                    size="sm"
                    disabled={!connected}
                    onClick={() => { setPending({ label: "Enabling", mode: telemetryMode }); sendEnable(); }}
                    className="gap-1.5 h-7 bg-green-600 hover:bg-green-500 disabled:bg-gray-600"
                  >
                    <Play className="h-3.5 w-3.5" weight="fill" />
                    Enable
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Enter teleop mode</TooltipContent>
              </Tooltip>
            </>
          )}

          {/* Teleop: Autonomy, Disable */}
          {isTeleop && (
            <>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="default"
                    size="sm"
                    disabled={!connected}
                    onClick={() => { setPending({ label: "Entering autonomy", mode: telemetryMode }); sendAutonomous(); }}
                    className="gap-1.5 h-7 bg-green-600 hover:bg-green-500"
                  >
                    <Robot className="h-3.5 w-3.5" weight="fill" />
                    Autonomy
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Switch to autonomous mode</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="default"
                    size="sm"
                    onClick={() => { setPending({ label: "Disabling", mode: telemetryMode }); sendDisable(); }}
                    className="gap-1.5 h-7 bg-amber-600 hover:bg-amber-500"
                  >
                    <Stop className="h-3.5 w-3.5" weight="fill" />
                    Disable
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Exit teleop and stop motors</TooltipContent>
              </Tooltip>
            </>
          )}

          {/* Autonomous: Stop */}
          {isAutonomous && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="default"
                  size="sm"
                  onClick={() => { setPending({ label: "Stopping", mode: telemetryMode }); sendStopAutonomy(); }}
                  className="gap-1.5 h-7 bg-amber-600 hover:bg-amber-500"
                >
                  <Stop className="h-3.5 w-3.5" weight="fill" />
                  Stop
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Stop autonomous mode</TooltipContent>
            </Tooltip>
          )}

          {/* Sleep: Wake */}
          {isSleep && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="default"
                  size="sm"
                  disabled={!connected}
                  onClick={() => { setPending({ label: "Waking", mode: telemetryMode }); sendWake(); }}
                  className="gap-1.5 h-7 bg-indigo-600 hover:bg-indigo-500"
                >
                  <Sun className="h-3.5 w-3.5" weight="fill" />
                  Wake
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Wake rover and start sensors</TooltipContent>
            </Tooltip>
          )}

          {/* Fault: reason + Clear Fault */}
          {isFault && (
            <>
              {faultCode > 0 && (
                <span className="text-xs text-red-400 max-w-48 truncate">
                  {faultDescriptions[faultCode] ?? `Unknown fault (${faultCode})`}
                </span>
              )}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="default"
                    size="sm"
                    disabled={!connected}
                    onClick={() => { setPending({ label: "Clearing fault", mode: telemetryMode }); sendClearFault(); }}
                    className="gap-1.5 h-7 bg-red-600 hover:bg-red-500"
                  >
                    <ArrowCounterClockwise className="h-3.5 w-3.5" weight="bold" />
                    Clear Fault
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">Clear fault and return to Idle</TooltipContent>
              </Tooltip>
            </>
          )}
        </>
      )}
    </div>
  );
}
