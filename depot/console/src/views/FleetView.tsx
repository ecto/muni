import { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { useDispatch } from "@/hooks/useDispatch";
import {
  Robot,
  GameController,
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  ChartLineUp,
  VideoCamera,
} from "@phosphor-icons/react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Mode, ModeLabels, type RoverInfo, type Mode as ModeType } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";

function DirectConnectCard() {
  const [hostname, setHostname] = useState("");
  const navigate = useNavigate();

  const submit = () => {
    const trimmed = hostname.trim();
    if (trimmed) navigate(`/rover/${trimmed}`);
  };

  return (
    <div className="bg-muted/50 border border-border p-8 text-center rounded-lg">
      <Robot className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
      <h3 className="font-medium mb-1">No Rovers Discovered</h3>
      <p className="text-sm text-muted-foreground mb-4">
        Discovery service unavailable. Connect directly by hostname or IP.
      </p>
      <div className="flex gap-2 max-w-xs mx-auto">
        <Input
          value={hostname}
          onChange={(e) => setHostname(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="frog-0 or 192.168.1.100"
          autoFocus
        />
        <Button onClick={submit} disabled={!hostname.trim()}>
          Connect
        </Button>
      </div>
    </div>
  );
}

function timeAgo(timestamp: number): string {
  const diff = Date.now() - timestamp;
  if (diff < 5000) return "now";
  if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return `${Math.floor(diff / 86400000)}d ago`;
}

function getModeVariant(
  mode: ModeType
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

function BatteryDisplay({ voltage }: { voltage: number }) {
  const percent = getBatteryPercent(voltage);
  const Icon =
    voltage < 42
      ? BatteryWarning
      : voltage < 45
        ? BatteryLow
        : BatteryFull;
  const color =
    voltage < 42
      ? "text-red-500"
      : voltage < 45
        ? "text-orange-500"
        : "text-green-500";
  const barColor =
    voltage < 42
      ? "bg-red-500"
      : voltage < 45
        ? "bg-orange-500"
        : "bg-green-500";

  return (
    <div className="flex items-center gap-2">
      <Icon className={`h-4 w-4 ${color}`} weight="fill" />
      <div className="flex-1">
        <div className="flex items-center justify-between text-xs mb-0.5">
          <span className="font-medium">{percent.toFixed(0)}%</span>
          <span className="text-muted-foreground">{voltage.toFixed(1)}V</span>
        </div>
        <div className="h-1.5 bg-muted rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full ${barColor}`}
            style={{ width: `${percent}%` }}
          />
        </div>
      </div>
    </div>
  );
}

function RoverCard({
  rover,
  taskInfo,
}: {
  rover: RoverInfo;
  taskInfo?: { missionName?: string; progress: number; waypoint: number; lap: number };
}) {
  const isOnline = rover.online;
  const displayMode =
    rover.mode === Mode.Disabled ? Mode.Idle : rover.mode;

  // We can't access SubsystemHealth from RoverInfo directly,
  // but we show what we have from discovery data
  const grafanaUrl = `/grafana/d/rover-${rover.id}`;

  return (
    <div
      className={`border border-border rounded-lg overflow-hidden ${
        !isOnline ? "opacity-50" : ""
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-muted/30 border-b border-border">
        <div className="flex items-center gap-2.5">
          <span
            className={`h-2.5 w-2.5 rounded-full flex-shrink-0 ${
              isOnline ? "bg-green-500" : "bg-red-500"
            }`}
          />
          <Link
            to={`/fleet/${rover.id}`}
            className="font-medium hover:text-primary transition-colors"
          >
            {rover.name || rover.id}
          </Link>
          {isOnline && (
            <Badge
              variant={getModeVariant(displayMode)}
              className="text-[10px] h-5 px-1.5"
            >
              {ModeLabels[displayMode]}
            </Badge>
          )}
        </div>
        {!isOnline && rover.lastSeen > 0 && (
          <span className="text-xs text-muted-foreground">
            seen {timeAgo(rover.lastSeen)}
          </span>
        )}
      </div>

      {isOnline && (
        <div className="p-4 space-y-3">
          {/* Battery + Power */}
          <BatteryDisplay voltage={rover.batteryVoltage} />

          {/* Task progress (if any) */}
          {taskInfo && (
            <div className="p-2 bg-primary/5 border border-primary/10 rounded text-xs">
              <div className="flex items-center justify-between mb-1">
                <span className="font-medium">
                  {taskInfo.missionName ?? "Active Task"}
                </span>
                <span className="text-muted-foreground">
                  {(taskInfo.progress * 100).toFixed(0)}%
                </span>
              </div>
              <div className="h-1 bg-muted rounded-full overflow-hidden mb-1">
                <div
                  className="h-full bg-primary rounded-full"
                  style={{ width: `${taskInfo.progress * 100}%` }}
                />
              </div>
              <span className="text-muted-foreground">
                WP {taskInfo.waypoint} · Lap {taskInfo.lap}
              </span>
            </div>
          )}

          {/* Quick Actions */}
          <div className="flex gap-2">
            <Button
              asChild
              size="sm"
              variant="default"
              className="h-7 px-2.5 gap-1 flex-1"
            >
              <Link to={`/fleet/${rover.id}`}>
                <GameController className="h-3.5 w-3.5" />
                Teleop
              </Link>
            </Button>
            <Button
              asChild
              size="sm"
              variant="outline"
              className="h-7 px-2.5 gap-1"
            >
              <a href={grafanaUrl} target="_blank" rel="noopener noreferrer">
                <ChartLineUp className="h-3.5 w-3.5" />
                Grafana
              </a>
            </Button>
            <Button
              asChild
              size="sm"
              variant="outline"
              className="h-7 px-2.5 gap-1"
            >
              <Link to={`/sessions?rover=${rover.id}`}>
                <VideoCamera className="h-3.5 w-3.5" />
                Sessions
              </Link>
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

export function FleetView() {
  const rovers = useConsoleStore((s) => s.rovers);
  const { tasks, missions } = useDispatch();

  const { onlineRovers, sortedRovers, activeCount, faultCount } =
    useMemo(() => {
      const online = rovers.filter((r) => r.online);
      const active = rovers.filter(
        (r) =>
          r.online &&
          (r.mode === Mode.Teleop || r.mode === Mode.Autonomous)
      );
      const fault = rovers.filter(
        (r) =>
          r.online &&
          (r.mode === Mode.EStop || r.mode === Mode.Fault)
      );
      const sorted = [...rovers].sort((a, b) => {
        if (a.online !== b.online) return a.online ? -1 : 1;
        return (a.name || a.id).localeCompare(b.name || b.id);
      });
      return {
        onlineRovers: online,
        sortedRovers: sorted,
        activeCount: active.length,
        faultCount: fault.length,
      };
    }, [rovers]);

  // Build task info map
  const taskInfoMap = useMemo(() => {
    const map = new Map<
      string,
      { missionName?: string; progress: number; waypoint: number; lap: number }
    >();
    for (const task of tasks) {
      if (task.status !== "active" && task.status !== "assigned") continue;
      const mission = missions.find((m) => m.id === task.missionId);
      map.set(task.roverId, {
        missionName: mission?.name,
        progress: task.progress,
        waypoint: task.waypoint,
        lap: task.lap,
      });
    }
    return map;
  }, [tasks, missions]);

  return (
    <TooltipProvider delayDuration={0}>
      <div className="min-h-full p-6">
        <div className="max-w-5xl mx-auto space-y-6">
          {/* Header */}
          <div>
            <h1 className="text-2xl font-bold">Fleet</h1>
            <p className="text-sm text-muted-foreground">
              {rovers.length} total · {onlineRovers.length} online ·{" "}
              {activeCount} active
              {faultCount > 0 && (
                <span className="text-red-500">
                  {" · "}
                  {faultCount} fault
                </span>
              )}
            </p>
          </div>

          {/* Summary Bar */}
          <div className="flex gap-3 text-sm">
            <SummaryPill
              label="Total"
              value={rovers.length}
              variant="default"
            />
            <SummaryPill
              label="Online"
              value={onlineRovers.length}
              variant="success"
            />
            <SummaryPill
              label="Active"
              value={activeCount}
              variant="primary"
            />
            {faultCount > 0 && (
              <SummaryPill
                label="Fault"
                value={faultCount}
                variant="danger"
              />
            )}
          </div>

          {/* Rover Cards */}
          {rovers.length === 0 ? (
            <DirectConnectCard />
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {sortedRovers.map((rover) => (
                <RoverCard
                  key={rover.id}
                  rover={rover}
                  taskInfo={taskInfoMap.get(rover.id)}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </TooltipProvider>
  );
}

function SummaryPill({
  label,
  value,
  variant,
}: {
  label: string;
  value: number;
  variant: "default" | "success" | "primary" | "danger";
}) {
  const colors = {
    default: "bg-muted text-foreground",
    success: "bg-green-500/10 text-green-600 dark:text-green-400",
    primary: "bg-primary/10 text-primary",
    danger: "bg-red-500/10 text-red-600 dark:text-red-400",
  };

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium ${colors[variant]}`}
    >
      {value} {label}
    </span>
  );
}
