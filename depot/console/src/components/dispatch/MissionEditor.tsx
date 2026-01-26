import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScheduleEditor } from "./ScheduleEditor";
import type { Zone, Mission, Schedule, Task, ConnectedRover } from "@/lib/types";

interface MissionEditorProps {
  missions: Mission[];
  zones: Zone[];
  connectedRovers: ConnectedRover[];
  roverNames: { id: string; name: string }[];
  activeTasks: Task[];
  onCreateMission: (data: {
    name: string;
    zoneId: string;
    roverId?: string;
    schedule: Schedule;
  }) => void;
  onUpdateMission: (
    id: string,
    updates: { name?: string; zoneId?: string; roverId?: string; schedule?: Schedule; enabled?: boolean }
  ) => void;
  onDeleteMission: (id: string) => void;
  onStartMission: (id: string) => void;
  onStopMission: (id: string) => void;
}

function MissionCard({
  mission,
  zone,
  activeTask,
  roverNames,
  connectedRovers,
  onStart,
  onStop,
  onDelete,
  onUpdate,
}: {
  mission: Mission;
  zone: Zone | undefined;
  activeTask: Task | undefined;
  roverNames: { id: string; name: string }[];
  connectedRovers: ConnectedRover[];
  onStart: () => void;
  onStop: () => void;
  onDelete: () => void;
  onUpdate: (updates: { schedule?: Schedule; enabled?: boolean }) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const isActive = !!activeTask;

  return (
    <div className={`p-3 rounded-lg border ${
      isActive
        ? "border-green-500/50 bg-green-500/5"
        : mission.enabled
          ? "border-zinc-800 bg-zinc-900"
          : "border-zinc-800/50 bg-zinc-900/50 opacity-60"
    }`}>
      <div className="flex items-center justify-between">
        <div
          className="flex items-center gap-2 cursor-pointer flex-1"
          onClick={() => setExpanded(!expanded)}
        >
          {isActive && (
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          )}
          <span className="font-medium text-sm">{mission.name}</span>
          <Badge variant="outline" className="text-[10px] h-4 px-1">
            {mission.schedule.trigger}
          </Badge>
          {mission.schedule.loop && (
            <Badge variant="secondary" className="text-[10px] h-4 px-1">
              loop
            </Badge>
          )}
        </div>
        <div className="flex gap-1">
          {isActive ? (
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs text-red-400"
              onClick={onStop}
            >
              Stop
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs"
              onClick={onStart}
              disabled={connectedRovers.length === 0}
            >
              Start
            </Button>
          )}
        </div>
      </div>

      <div className="text-xs text-muted-foreground mt-1">
        {zone?.name || "Unknown zone"}
        {mission.roverId && (
          <span>
            {" "}· {roverNames.find((r) => r.id === mission.roverId)?.name || mission.roverId}
          </span>
        )}
        {isActive && activeTask && (
          <span className="text-green-400">
            {" "}· WP {activeTask.waypoint + 1} · Lap {activeTask.lap + 1} · {activeTask.progress}%
          </span>
        )}
      </div>

      {/* Expanded schedule editor */}
      {expanded && (
        <div className="mt-3 pt-3 border-t border-zinc-800 space-y-3">
          <ScheduleEditor
            schedule={mission.schedule}
            onChange={(schedule) => onUpdate({ schedule })}
          />

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id={`enabled-${mission.id}`}
                checked={mission.enabled}
                onChange={(e) => onUpdate({ enabled: e.target.checked })}
                className="w-3.5 h-3.5"
              />
              <label htmlFor={`enabled-${mission.id}`} className="text-xs">
                Enabled
              </label>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="h-6 px-2 text-xs text-red-400 border-red-400/30"
              onClick={onDelete}
              disabled={isActive}
            >
              Delete
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

export function MissionEditor({
  missions,
  zones,
  connectedRovers,
  roverNames,
  activeTasks,
  onCreateMission,
  onUpdateMission,
  onDeleteMission,
  onStartMission,
  onStopMission,
}: MissionEditorProps) {
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newZoneId, setNewZoneId] = useState(zones[0]?.id || "");
  const [newRoverId, setNewRoverId] = useState("");
  const [newSchedule, setNewSchedule] = useState<Schedule>({
    trigger: "manual",
    loop: false,
  });

  const handleCreate = () => {
    if (!newName.trim() || !newZoneId) return;
    onCreateMission({
      name: newName,
      zoneId: newZoneId,
      roverId: newRoverId || undefined,
      schedule: newSchedule,
    });
    setNewName("");
    setShowCreate(false);
    setNewSchedule({ trigger: "manual", loop: false });
  };

  return (
    <div className="space-y-2 p-2">
      {/* Create new mission */}
      {showCreate ? (
        <div className="p-3 rounded-lg border border-blue-500/50 bg-blue-500/5 space-y-3">
          <div>
            <label className="text-xs font-medium block mb-1">Name</label>
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
              placeholder="Mission name"
              autoFocus
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-xs font-medium block mb-1">Zone</label>
              <select
                value={newZoneId}
                onChange={(e) => setNewZoneId(e.target.value)}
                className="w-full px-2 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-xs"
              >
                {zones.map((z) => (
                  <option key={z.id} value={z.id}>
                    {z.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-xs font-medium block mb-1">Rover</label>
              <select
                value={newRoverId}
                onChange={(e) => setNewRoverId(e.target.value)}
                className="w-full px-2 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-xs"
              >
                <option value="">Any available</option>
                {roverNames.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <ScheduleEditor schedule={newSchedule} onChange={setNewSchedule} />

          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              size="sm"
              className="h-7 px-3 text-xs"
              onClick={() => setShowCreate(false)}
            >
              Cancel
            </Button>
            <Button
              size="sm"
              className="h-7 px-3 text-xs"
              onClick={handleCreate}
              disabled={!newName.trim() || !newZoneId}
            >
              Create
            </Button>
          </div>
        </div>
      ) : (
        <Button
          variant="outline"
          size="sm"
          className="w-full h-8 text-xs"
          onClick={() => setShowCreate(true)}
          disabled={zones.length === 0}
        >
          + Create Mission
        </Button>
      )}

      {/* Mission list */}
      {missions.length === 0 ? (
        <div className="p-4 text-center text-xs text-muted-foreground">
          No missions defined
        </div>
      ) : (
        <div className="space-y-1">
          {missions.map((mission) => {
            const zone = zones.find((z) => z.id === mission.zoneId);
            const activeTask = activeTasks.find(
              (t) => t.missionId === mission.id
            );
            return (
              <MissionCard
                key={mission.id}
                mission={mission}
                zone={zone}
                activeTask={activeTask}
                roverNames={roverNames}
                connectedRovers={connectedRovers}
                onStart={() => onStartMission(mission.id)}
                onStop={() => onStopMission(mission.id)}
                onDelete={() => onDeleteMission(mission.id)}
                onUpdate={(updates) => onUpdateMission(mission.id, updates)}
              />
            );
          })}
        </div>
      )}
    </div>
  );
}
