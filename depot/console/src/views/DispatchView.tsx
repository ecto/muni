import { useState } from "react";
import { useDispatch } from "@/hooks/useDispatch";
import { useConsoleStore } from "@/store";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import type { Zone, Mission, Waypoint } from "@/lib/types";
import { TaskStatus } from "@/lib/types";

// Simple zone editor modal
function ZoneEditorModal({
  zone,
  onSave,
  onClose,
}: {
  zone?: Zone;
  onSave: (name: string, waypoints: Waypoint[]) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState(zone?.name || "");
  const [waypointsText, setWaypointsText] = useState(
    zone?.waypoints
      ? zone.waypoints.map((w) => `${w.x},${w.y}`).join("\n")
      : ""
  );

  const handleSave = () => {
    const waypoints: Waypoint[] = waypointsText
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => {
        const [x, y] = line.split(",").map((n) => parseFloat(n.trim()));
        return { x: x || 0, y: y || 0 };
      });
    onSave(name, waypoints);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-[500px]">
        <CardHeader>
          <CardTitle>{zone ? "Edit Zone" : "Create Zone"}</CardTitle>
          <CardDescription>
            Define a zone with waypoints for the rover to follow.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
              placeholder="Zone name"
            />
          </div>
          <div>
            <label className="text-sm font-medium">
              Waypoints (x,y per line)
            </label>
            <textarea
              value={waypointsText}
              onChange={(e) => setWaypointsText(e.target.value)}
              className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono h-32"
              placeholder="0,0&#10;1,0&#10;1,1&#10;0,1"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={!name.trim()}>
              Save
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Mission editor modal
function MissionEditorModal({
  mission,
  zones,
  rovers,
  onSave,
  onClose,
}: {
  mission?: Mission;
  zones: Zone[];
  rovers: { id: string; name: string }[];
  onSave: (
    name: string,
    zoneId: string,
    roverId: string | undefined,
    loop: boolean
  ) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState(mission?.name || "");
  const [zoneId, setZoneId] = useState(mission?.zoneId || zones[0]?.id || "");
  const [roverId, setRoverId] = useState(mission?.roverId || "");
  const [loop, setLoop] = useState(mission?.schedule?.loop ?? false);

  const handleSave = () => {
    onSave(name, zoneId, roverId || undefined, loop);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-[500px]">
        <CardHeader>
          <CardTitle>{mission ? "Edit Mission" : "Create Mission"}</CardTitle>
          <CardDescription>
            Configure a mission to dispatch to a rover.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
              placeholder="Mission name"
            />
          </div>
          <div>
            <label className="text-sm font-medium">Zone</label>
            <select
              value={zoneId}
              onChange={(e) => setZoneId(e.target.value)}
              className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
            >
              {zones.map((z) => (
                <option key={z.id} value={z.id}>
                  {z.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-sm font-medium">Rover (optional)</label>
            <select
              value={roverId}
              onChange={(e) => setRoverId(e.target.value)}
              className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
            >
              <option value="">Any available rover</option>
              {rovers.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name}
                </option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="loop"
              checked={loop}
              onChange={(e) => setLoop(e.target.checked)}
              className="w-4 h-4"
            />
            <label htmlFor="loop" className="text-sm font-medium">
              Loop continuously
            </label>
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button
              onClick={handleSave}
              disabled={!name.trim() || !zoneId}
            >
              Save
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Task status badge
function TaskStatusBadge({ status }: { status: string }) {
  const variants: Record<string, "default" | "secondary" | "destructive" | "outline"> = {
    pending: "secondary",
    assigned: "outline",
    active: "default",
    done: "secondary",
    failed: "destructive",
    cancelled: "secondary",
  };

  return (
    <Badge variant={variants[status] || "secondary"}>
      {status}
    </Badge>
  );
}

export function DispatchView() {
  const {
    zones,
    missions,
    tasks,
    connectedRovers,
    error,
    createZone,
    updateZone,
    deleteZone,
    createMission,
    deleteMission,
    startMission,
    stopMission,
    cancelTask,
  } = useDispatch();

  const { rovers } = useConsoleStore();

  const [showZoneEditor, setShowZoneEditor] = useState(false);
  const [editingZone, setEditingZone] = useState<Zone | undefined>();
  const [showMissionEditor, setShowMissionEditor] = useState(false);
  const [editingMission, setEditingMission] = useState<Mission | undefined>();

  // Get active tasks
  const activeTasks = tasks.filter(
    (t) => t.status === TaskStatus.Active || t.status === TaskStatus.Assigned
  );

  // Handle zone save
  const handleZoneSave = async (name: string, waypoints: Waypoint[]) => {
    try {
      if (editingZone) {
        await updateZone(editingZone.id, { name, waypoints });
      } else {
        await createZone({ name, waypoints });
      }
      setShowZoneEditor(false);
      setEditingZone(undefined);
    } catch (e) {
      console.error("Failed to save zone:", e);
    }
  };

  // Handle mission save
  const handleMissionSave = async (
    name: string,
    zoneId: string,
    roverId: string | undefined,
    loop: boolean
  ) => {
    try {
      await createMission({
        name,
        zoneId,
        roverId,
        schedule: { trigger: "manual", loop },
      });
      setShowMissionEditor(false);
      setEditingMission(undefined);
    } catch (e) {
      console.error("Failed to save mission:", e);
    }
  };

  // Handle start mission
  const handleStartMission = async (id: string) => {
    try {
      await startMission(id);
    } catch (e) {
      console.error("Failed to start mission:", e);
    }
  };

  // Handle stop mission
  const handleStopMission = async (id: string) => {
    try {
      await stopMission(id);
    } catch (e) {
      console.error("Failed to stop mission:", e);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Dispatch</h1>
        <p className="text-muted-foreground">
          Manage zones, missions, and dispatch tasks to rovers.
        </p>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 text-red-400">
          {error}
        </div>
      )}

      {/* Connected Rovers */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Connected Rovers</CardTitle>
          <CardDescription>
            Rovers connected to the dispatch service
          </CardDescription>
        </CardHeader>
        <CardContent>
          {connectedRovers.length === 0 ? (
            <p className="text-muted-foreground text-sm">
              No rovers connected to dispatch
            </p>
          ) : (
            <div className="flex flex-wrap gap-2">
              {connectedRovers.map((r) => (
                <Badge
                  key={r.roverId}
                  variant={r.taskId ? "default" : "secondary"}
                >
                  {r.roverId}
                  {r.taskId && " (active)"}
                </Badge>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Active Tasks */}
      {activeTasks.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Active Tasks</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {activeTasks.map((task) => {
              const mission = missions.find((m) => m.id === task.missionId);
              const zone = zones.find((z) => z.id === mission?.zoneId);
              return (
                <div
                  key={task.id}
                  className="flex items-center justify-between p-4 bg-zinc-900 rounded-lg"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">
                        {mission?.name || "Unknown Mission"}
                      </span>
                      <TaskStatusBadge status={task.status} />
                    </div>
                    <div className="text-sm text-muted-foreground">
                      Rover: {task.roverId} | Zone: {zone?.name || "Unknown"}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      Waypoint {task.waypoint + 1} | Lap {task.lap + 1}
                    </div>
                  </div>
                  <div className="flex items-center gap-4">
                    <div className="w-32">
                      <Progress value={task.progress} />
                      <span className="text-xs text-muted-foreground">
                        {task.progress}%
                      </span>
                    </div>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => cancelTask(task.id)}
                    >
                      Cancel
                    </Button>
                  </div>
                </div>
              );
            })}
          </CardContent>
        </Card>
      )}

      <div className="grid grid-cols-2 gap-6">
        {/* Zones */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle className="text-lg">Zones</CardTitle>
              <CardDescription>Areas for rovers to patrol</CardDescription>
            </div>
            <Button
              size="sm"
              onClick={() => {
                setEditingZone(undefined);
                setShowZoneEditor(true);
              }}
            >
              + Add Zone
            </Button>
          </CardHeader>
          <CardContent>
            {zones.length === 0 ? (
              <p className="text-muted-foreground text-sm">No zones defined</p>
            ) : (
              <div className="space-y-2">
                {zones.map((zone) => (
                  <div
                    key={zone.id}
                    className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg"
                  >
                    <div>
                      <div className="font-medium">{zone.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {zone.waypoints.length} waypoints
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setEditingZone(zone);
                          setShowZoneEditor(true);
                        }}
                      >
                        Edit
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => deleteZone(zone.id)}
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Missions */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle className="text-lg">Missions</CardTitle>
              <CardDescription>Scheduled work for rovers</CardDescription>
            </div>
            <Button
              size="sm"
              onClick={() => {
                setEditingMission(undefined);
                setShowMissionEditor(true);
              }}
              disabled={zones.length === 0}
            >
              + Add Mission
            </Button>
          </CardHeader>
          <CardContent>
            {missions.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                No missions defined
              </p>
            ) : (
              <div className="space-y-2">
                {missions.map((mission) => {
                  const zone = zones.find((z) => z.id === mission.zoneId);
                  const activeTask = activeTasks.find(
                    (t) => t.missionId === mission.id
                  );
                  return (
                    <div
                      key={mission.id}
                      className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg"
                    >
                      <div>
                        <div className="font-medium">{mission.name}</div>
                        <div className="text-sm text-muted-foreground">
                          Zone: {zone?.name || "Unknown"}
                          {mission.roverId && ` | Rover: ${mission.roverId}`}
                          {mission.schedule.loop && " | Loop"}
                        </div>
                      </div>
                      <div className="flex gap-2">
                        {activeTask ? (
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={() => handleStopMission(mission.id)}
                          >
                            Stop
                          </Button>
                        ) : (
                          <Button
                            variant="default"
                            size="sm"
                            onClick={() => handleStartMission(mission.id)}
                            disabled={connectedRovers.length === 0}
                          >
                            Start
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => deleteMission(mission.id)}
                          disabled={!!activeTask}
                        >
                          Delete
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Separator />

      {/* Recent Tasks */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Recent Tasks</CardTitle>
          <CardDescription>History of dispatched tasks</CardDescription>
        </CardHeader>
        <CardContent>
          {tasks.length === 0 ? (
            <p className="text-muted-foreground text-sm">No tasks yet</p>
          ) : (
            <div className="space-y-2">
              {tasks.slice(0, 10).map((task) => {
                const mission = missions.find((m) => m.id === task.missionId);
                return (
                  <div
                    key={task.id}
                    className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg"
                  >
                    <div className="flex items-center gap-3">
                      <TaskStatusBadge status={task.status} />
                      <div>
                        <div className="font-medium">
                          {mission?.name || "Unknown Mission"}
                        </div>
                        <div className="text-sm text-muted-foreground">
                          Rover: {task.roverId} |{" "}
                          {new Date(task.createdAt).toLocaleString()}
                        </div>
                      </div>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {task.lap} laps
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Modals */}
      {showZoneEditor && (
        <ZoneEditorModal
          zone={editingZone}
          onSave={handleZoneSave}
          onClose={() => {
            setShowZoneEditor(false);
            setEditingZone(undefined);
          }}
        />
      )}

      {showMissionEditor && (
        <MissionEditorModal
          mission={editingMission}
          zones={zones}
          rovers={rovers.map((r) => ({ id: r.id, name: r.name }))}
          onSave={handleMissionSave}
          onClose={() => {
            setShowMissionEditor(false);
            setEditingMission(undefined);
          }}
        />
      )}
    </div>
  );
}
