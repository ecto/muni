import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { Zone, Task } from "@/lib/types";

interface ZoneListProps {
  zones: Zone[];
  selectedZoneId: string | null;
  onSelectZone: (id: string | null) => void;
  onEditZone: (zone: Zone) => void;
  onDeleteZone: (id: string) => void;
  onStartPatrol: (zoneId: string) => void;
  onStopPatrol: (zoneId: string) => void;
  getActiveTask: (zoneId: string) => Task | null;
  canStartPatrol: boolean;
}

export function ZoneList({
  zones,
  selectedZoneId,
  onSelectZone,
  onEditZone,
  onDeleteZone,
  onStartPatrol,
  onStopPatrol,
  getActiveTask,
  canStartPatrol,
}: ZoneListProps) {
  if (zones.length === 0) {
    return (
      <div className="p-4 text-center text-xs text-muted-foreground">
        No zones defined. Use the map to draw a zone.
      </div>
    );
  }

  return (
    <div className="space-y-1 p-2">
      {zones.map((zone) => {
        const activeTask = getActiveTask(zone.id);
        const isActive = !!activeTask;
        const isSelected = selectedZoneId === zone.id;

        return (
          <div
            key={zone.id}
            className={`p-2 rounded-lg border transition-colors cursor-pointer ${
              isSelected
                ? "border-blue-500 bg-blue-500/10"
                : isActive
                  ? "border-green-500/50 bg-green-500/10"
                  : "border-zinc-800 bg-zinc-900 hover:border-zinc-700"
            }`}
            onClick={() => onSelectZone(zone.id)}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {isActive && (
                  <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                )}
                <span className="font-medium text-sm">{zone.name}</span>
                <Badge variant="outline" className="text-[10px] h-4 px-1">
                  {zone.zoneType}
                </Badge>
              </div>
              {isActive ? (
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 px-2 text-xs text-red-400 hover:text-red-300"
                  onClick={(e) => {
                    e.stopPropagation();
                    onStopPatrol(zone.id);
                  }}
                >
                  Stop
                </Button>
              ) : (
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 px-2 text-xs text-green-400 hover:text-green-300"
                  onClick={(e) => {
                    e.stopPropagation();
                    onStartPatrol(zone.id);
                  }}
                  disabled={!canStartPatrol}
                >
                  Patrol
                </Button>
              )}
            </div>
            <div className="text-xs text-muted-foreground mt-1">
              {zone.waypoints.length} waypoints
              {zone.polygonLatlng && zone.polygonLatlng.length > 0 && (
                <span> · {zone.polygonLatlng.length} vertices</span>
              )}
              {zone.zoneType === "coverage" && zone.coverageWaypoints && (
                <span> · {zone.coverageWaypoints.length} sweep pts</span>
              )}
              {isActive && activeTask && (
                <span className="ml-2">
                  · WP {activeTask.waypoint + 1} · Lap {activeTask.lap + 1}
                </span>
              )}
            </div>

            {/* Selected zone actions */}
            {isSelected && (
              <div className="flex gap-2 mt-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="h-6 px-2 text-xs flex-1"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEditZone(zone);
                  }}
                >
                  Edit
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-6 px-2 text-xs text-red-400 border-red-400/30"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteZone(zone.id);
                    onSelectZone(null);
                  }}
                  disabled={isActive}
                >
                  Delete
                </Button>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
