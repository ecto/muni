import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import {
  Robot,
  GameController,
  BatteryFull,
  BatteryLow,
  BatteryWarning,
} from "@phosphor-icons/react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  TooltipProvider,
} from "@/components/ui/tooltip";
import { Mode, ModeLabels, type RoverInfo } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";

function formatLastSeen(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  if (diff < 5000) return "now";
  if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return `${Math.floor(diff / 86400000)}d ago`;
}

function getModeVariant(mode: Mode): "default" | "secondary" | "destructive" | "outline" {
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

function BatteryCell({ voltage }: { voltage: number }) {
  const percent = getBatteryPercent(voltage);
  const Icon = voltage < 42 ? BatteryWarning : voltage < 45 ? BatteryLow : BatteryFull;
  const color = voltage < 42 ? "text-red-500" : voltage < 45 ? "text-orange-500" : "text-green-500";

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className="flex items-center gap-1 cursor-help">
          <Icon className={`h-3.5 w-3.5 ${color}`} weight="fill" />
          <span className="tabular-nums">{percent.toFixed(0)}%</span>
        </div>
      </TooltipTrigger>
      <TooltipContent side="top">{voltage.toFixed(1)}V</TooltipContent>
    </Tooltip>
  );
}

function StatusDot({ online }: { online: boolean }) {
  return (
    <span
      className={`h-2 w-2 rounded-full ${online ? "bg-green-500" : "bg-red-500"}`}
      aria-label={online ? "online" : "offline"}
    />
  );
}

function RoverRow({ rover }: { rover: RoverInfo }) {
  const theta = ((rover.lastPose.theta * 180) / Math.PI).toFixed(0);

  return (
    <tr className={`border-b border-border hover:bg-muted/50 transition-colors ${!rover.online ? "opacity-50" : ""}`}>
      {/* Status */}
      <td className="py-2 px-3">
        <StatusDot online={rover.online} />
      </td>

      {/* Name */}
      <td className="py-2 px-3">
        <Link
          to={`/fleet/${rover.id}`}
          className="font-medium hover:text-primary transition-colors"
        >
          {rover.name || rover.id}
        </Link>
      </td>

      {/* Mode */}
      <td className="py-2 px-3">
        <Badge variant={getModeVariant(rover.mode)} className="text-[10px] h-5 px-1.5">
          {ModeLabels[rover.mode]}
        </Badge>
      </td>

      {/* Battery */}
      <td className="py-2 px-3 text-sm">
        {rover.online ? <BatteryCell voltage={rover.batteryVoltage} /> : "—"}
      </td>

      {/* Position */}
      <td className="py-2 px-3 font-mono text-xs text-muted-foreground">
        {rover.online ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="cursor-help">
                {rover.lastPose.x.toFixed(1)}, {rover.lastPose.y.toFixed(1)}
              </span>
            </TooltipTrigger>
            <TooltipContent side="top">θ = {theta}°</TooltipContent>
          </Tooltip>
        ) : (
          "—"
        )}
      </td>

      {/* Last Seen */}
      <td className="py-2 px-3 text-xs text-muted-foreground">
        {formatLastSeen(rover.lastSeen)}
      </td>

      {/* Actions */}
      <td className="py-2 px-3 text-right">
        {rover.online && (
          <Button asChild size="sm" variant="default" className="h-7 px-2 gap-1">
            <Link to={`/fleet/${rover.id}/teleop`}>
              <GameController className="h-3.5 w-3.5" />
              Teleop
            </Link>
          </Button>
        )}
      </td>
    </tr>
  );
}

export function FleetView() {
  const { rovers } = useConsoleStore();

  const onlineRovers = rovers.filter((r) => r.online);
  const offlineRovers = rovers.filter((r) => !r.online);

  // Sort: online first, then by name
  const sortedRovers = [...rovers].sort((a, b) => {
    if (a.online !== b.online) return a.online ? -1 : 1;
    return (a.name || a.id).localeCompare(b.name || b.id);
  });

  return (
    <TooltipProvider delayDuration={0}>
      <div className="min-h-full p-6">
        <div className="max-w-5xl mx-auto space-y-6">
          {/* Header */}
          <div>
            <h1 className="text-2xl font-bold">Fleet</h1>
            <p className="text-sm text-muted-foreground">
              {onlineRovers.length} online, {offlineRovers.length} offline
            </p>
          </div>

          {/* Rover Table */}
          {rovers.length === 0 ? (
            <div className="bg-muted/50 border border-border p-8 text-center">
              <Robot className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
              <h3 className="font-medium mb-2">No Rovers Registered</h3>
              <p className="text-sm text-muted-foreground max-w-md mx-auto">
                Rovers will appear here once they connect to the discovery service.
                Make sure rovers are configured to register with this depot.
              </p>
            </div>
          ) : (
            <div className="border border-border rounded-lg overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-muted/50 border-b border-border">
                  <tr className="text-left text-xs text-muted-foreground uppercase tracking-wider">
                    <th className="py-2 px-3 w-8"></th>
                    <th className="py-2 px-3">Name</th>
                    <th className="py-2 px-3">Mode</th>
                    <th className="py-2 px-3">Battery</th>
                    <th className="py-2 px-3">Position</th>
                    <th className="py-2 px-3">Last Seen</th>
                    <th className="py-2 px-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {sortedRovers.map((rover) => (
                    <RoverRow key={rover.id} rover={rover} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </TooltipProvider>
  );
}
