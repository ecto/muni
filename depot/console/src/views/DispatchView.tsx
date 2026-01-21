import { useState, useMemo } from "react";
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
import type { Zone, Mission, Waypoint } from "@/lib/types";
import { TaskStatus } from "@/lib/types";
import { Scene } from "@/components/scene/Scene";

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
  const [selectedZoneId, setSelectedZoneId] = useState<string | null>(null);

  // Get active tasks
  const activeTasks = useMemo(
    () => tasks.filter(
      (t) => t.status === TaskStatus.Active || t.status === TaskStatus.Assigned
    ),
    [tasks]
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

  // Quick patrol: find or create a looping mission for a zone and start it
  const handleQuickPatrol = async (zoneId: string) => {
    try {
      const zone = zones.find((z) => z.id === zoneId);
      if (!zone) return;

      // Check if there's already a looping mission for this zone
      let mission = missions.find((m) => m.zoneId === zoneId && m.schedule.loop);

      if (!mission) {
        // Create a new looping mission
        mission = await createMission({
          name: `${zone.name} Patrol`,
          zoneId,
          schedule: { trigger: "manual", loop: true },
        });
      }

      // Start the mission
      await startMission(mission.id);
    } catch (e) {
      console.error("Failed to start patrol:", e);
    }
  };

  // Stop patrol for a zone
  const handleStopPatrol = async (zoneId: string) => {
    try {
      // Find the active mission for this zone
      const mission = missions.find((m) => m.zoneId === zoneId);
      if (mission) {
        await stopMission(mission.id);
      }
    } catch (e) {
      console.error("Failed to stop patrol:", e);
    }
  };

  // Check if a zone has an active patrol
  const getZonePatrolStatus = (zoneId: string) => {
    const mission = missions.find((m) => m.zoneId === zoneId);
    if (!mission) return null;
    const activeTask = activeTasks.find((t) => t.missionId === mission.id);
    return activeTask || null;
  };

  return (
    <div className="flex h-full">
      {/* 3D Scene - Left side (70%) */}
      <div className="flex-1 relative">
        <Scene
          mode="dispatch"
          zones={zones}
          activeTasks={activeTasks}
          onZoneClick={setSelectedZoneId}
        />

        {/* Error overlay */}
        {error && (
          <div className="absolute top-4 left-4 right-4 bg-red-500/90 backdrop-blur rounded-lg p-3 text-white text-sm">
            {error}
          </div>
        )}

        {/* Active task overlay */}
        {activeTasks.length > 0 && (
          <div className="absolute bottom-4 left-4 right-4 max-w-md">
            {activeTasks.slice(0, 2).map((task) => {
              const mission = missions.find((m) => m.id === task.missionId);
              const zone = zones.find((z) => z.id === mission?.zoneId);
              return (
                <div
                  key={task.id}
                  className="bg-zinc-900/90 backdrop-blur rounded-lg p-3 mb-2"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                      <span className="font-medium text-sm">
                        {mission?.name || "Patrol"}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {zone?.name}
                      </span>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 px-2 text-xs"
                      onClick={() => cancelTask(task.id)}
                    >
                      Stop
                    </Button>
                  </div>
                  <div className="mt-2 flex items-center gap-2">
                    <Progress value={task.progress} className="h-1 flex-1" />
                    <span className="text-xs text-muted-foreground w-16">
                      WP {task.waypoint + 1} • L{task.lap + 1}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Control Panel - Right side (30%) */}
      <div className="w-80 border-l border-zinc-800 bg-zinc-950 flex flex-col">
        <div className="p-4 border-b border-zinc-800">
          <h1 className="text-lg font-semibold">Dispatch</h1>
          <p className="text-xs text-muted-foreground">
            {connectedRovers.length} rover{connectedRovers.length !== 1 ? "s" : ""} connected
          </p>
        </div>

        <div className="flex-1 overflow-y-auto">
          <div className="p-4 space-y-4">
            {/* Connected Rovers */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-medium">Fleet</h2>
                {connectedRovers.length > 0 && (
                  <Badge variant="secondary" className="text-xs">
                    {connectedRovers.length} online
                  </Badge>
                )}
              </div>
              {connectedRovers.length === 0 ? (
                <p className="text-xs text-muted-foreground">
                  No rovers connected to dispatch
                </p>
              ) : (
                <div className="flex flex-wrap gap-1">
                  {connectedRovers.map((r) => (
                    <Badge
                      key={r.roverId}
                      variant={r.taskId ? "default" : "outline"}
                      className="text-xs"
                    >
                      {r.roverId}
                      {r.taskId && " •"}
                    </Badge>
                  ))}
                </div>
              )}
            </div>

            {/* Zones with Quick Patrol */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-medium">Zones</h2>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 px-2 text-xs"
                  onClick={() => {
                    setEditingZone(undefined);
                    setShowZoneEditor(true);
                  }}
                >
                  + Add
                </Button>
              </div>
              {zones.length === 0 ? (
                <p className="text-xs text-muted-foreground">No zones defined</p>
              ) : (
                <div className="space-y-1">
                  {zones.map((zone) => {
                    const patrolTask = getZonePatrolStatus(zone.id);
                    const isActive = !!patrolTask;
                    return (
                      <div
                        key={zone.id}
                        className={`p-2 rounded-lg border transition-colors cursor-pointer ${
                          selectedZoneId === zone.id
                            ? "border-blue-500 bg-blue-500/10"
                            : isActive
                            ? "border-green-500/50 bg-green-500/10"
                            : "border-zinc-800 bg-zinc-900 hover:border-zinc-700"
                        }`}
                        onClick={() => setSelectedZoneId(zone.id)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            {isActive && (
                              <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                            )}
                            <span className="font-medium text-sm">{zone.name}</span>
                          </div>
                          {isActive ? (
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 px-2 text-xs text-red-400 hover:text-red-300"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleStopPatrol(zone.id);
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
                                handleQuickPatrol(zone.id);
                              }}
                              disabled={connectedRovers.length === 0}
                            >
                              Patrol
                            </Button>
                          )}
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">
                          {zone.waypoints.length} waypoints
                          {isActive && patrolTask && (
                            <span className="ml-2">
                              • WP {patrolTask.waypoint + 1} • Lap {patrolTask.lap + 1}
                            </span>
                          )}
                        </div>
                        {isActive && patrolTask && (
                          <Progress value={patrolTask.progress} className="h-1 mt-2" />
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Missions (collapsed by default) */}
            <details className="group">
              <summary className="flex items-center justify-between cursor-pointer list-none">
                <h2 className="text-sm font-medium">Missions</h2>
                <div className="flex items-center gap-2">
                  <Badge variant="outline" className="text-xs">
                    {missions.length}
                  </Badge>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 px-2 text-xs"
                    onClick={(e) => {
                      e.preventDefault();
                      setEditingMission(undefined);
                      setShowMissionEditor(true);
                    }}
                    disabled={zones.length === 0}
                  >
                    + Add
                  </Button>
                </div>
              </summary>
              <div className="mt-2 space-y-1">
                {missions.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No missions defined
                  </p>
                ) : (
                  missions.map((mission) => {
                    const zone = zones.find((z) => z.id === mission.zoneId);
                    const activeTask = activeTasks.find(
                      (t) => t.missionId === mission.id
                    );
                    return (
                      <div
                        key={mission.id}
                        className="p-2 bg-zinc-900 rounded-lg border border-zinc-800"
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-medium text-sm">{mission.name}</span>
                          <div className="flex gap-1">
                            {activeTask ? (
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-6 px-2 text-xs text-red-400"
                                onClick={() => handleStopMission(mission.id)}
                              >
                                Stop
                              </Button>
                            ) : (
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-6 px-2 text-xs"
                                onClick={() => handleStartMission(mission.id)}
                                disabled={connectedRovers.length === 0}
                              >
                                Start
                              </Button>
                            )}
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 px-2 text-xs text-muted-foreground"
                              onClick={() => deleteMission(mission.id)}
                              disabled={!!activeTask}
                            >
                              ×
                            </Button>
                          </div>
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {zone?.name || "Unknown zone"}
                          {mission.schedule.loop && " • Loop"}
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            </details>

            {/* Recent Tasks (collapsed by default) */}
            <details className="group">
              <summary className="flex items-center justify-between cursor-pointer list-none">
                <h2 className="text-sm font-medium">History</h2>
                <Badge variant="outline" className="text-xs">
                  {tasks.length}
                </Badge>
              </summary>
              <div className="mt-2 space-y-1">
                {tasks.length === 0 ? (
                  <p className="text-xs text-muted-foreground">No tasks yet</p>
                ) : (
                  tasks.slice(0, 10).map((task) => {
                    const mission = missions.find((m) => m.id === task.missionId);
                    return (
                      <div
                        key={task.id}
                        className="p-2 bg-zinc-900 rounded-lg border border-zinc-800"
                      >
                        <div className="flex items-center gap-2">
                          <TaskStatusBadge status={task.status} />
                          <span className="text-sm truncate">
                            {mission?.name || "Unknown"}
                          </span>
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">
                          {task.roverId} • {task.lap} laps •{" "}
                          {new Date(task.createdAt).toLocaleTimeString()}
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            </details>
          </div>
        </div>

        {/* Selected Zone Actions */}
        {selectedZoneId && (
          <div className="p-4 border-t border-zinc-800 space-y-2">
            {(() => {
              const zone = zones.find((z) => z.id === selectedZoneId);
              if (!zone) return null;
              return (
                <>
                  <div className="flex items-center justify-between">
                    <span className="font-medium">{zone.name}</span>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 px-2 text-xs"
                      onClick={() => setSelectedZoneId(null)}
                    >
                      ×
                    </Button>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="flex-1"
                      onClick={() => {
                        setEditingZone(zone);
                        setShowZoneEditor(true);
                      }}
                    >
                      Edit
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="flex-1 text-red-400 border-red-400/50 hover:bg-red-400/10"
                      onClick={() => {
                        deleteZone(zone.id);
                        setSelectedZoneId(null);
                      }}
                    >
                      Delete
                    </Button>
                  </div>
                </>
              );
            })()}
          </div>
        )}
      </div>

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
