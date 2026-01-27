import { useState, useMemo, useCallback } from "react";
import { useDispatch } from "@/hooks/useDispatch";
import { useConsoleStore } from "@/store";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ZoneEditor } from "@/components/dispatch/ZoneEditor";
import { ZoneList } from "@/components/dispatch/ZoneList";
import { MissionEditor } from "@/components/dispatch/MissionEditor";
import { TaskTimeline } from "@/components/dispatch/TaskTimeline";
import { CoverageConfig } from "@/components/dispatch/CoverageConfig";
import type { Zone, Waypoint, Schedule, CoverageConfig as CoverageConfigType } from "@/lib/types";
import { TaskStatus } from "@/lib/types";

type DispatchTab = "zones" | "missions" | "timeline";

// Simple zone editor modal for name + waypoints (text-based fallback)
function ZoneCreateModal({
  zone,
  onSave,
  onClose,
}: {
  zone?: Zone;
  onSave: (name: string, waypoints: Waypoint[], polygonLatlng?: [number, number][]) => void;
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
    onSave(name, waypoints, zone?.polygonLatlng);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="w-[500px] bg-zinc-950 border border-zinc-800 rounded-lg p-6 space-y-4">
        <div>
          <h2 className="text-lg font-semibold">
            {zone ? "Edit Zone" : "Create Zone"}
          </h2>
          <p className="text-xs text-muted-foreground">
            Define waypoints for the rover to follow. Draw the polygon on the map.
          </p>
        </div>
        <div>
          <label className="text-sm font-medium">Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full mt-1 px-3 py-2 bg-zinc-900 border border-zinc-700 rounded-md text-sm"
            placeholder="Zone name"
            autoFocus
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
            placeholder={"0,0\n1,0\n1,1\n0,1"}
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
      </div>
    </div>
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
    generateCoverage,
    createMission,
    updateMission,
    deleteMission,
    startMission,
    stopMission,
    cancelTask,
  } = useDispatch();

  const rovers = useConsoleStore((s) => s.rovers);
  const gpsStatus = useConsoleStore((s) => s.gpsStatus);

  const [activeTab, setActiveTab] = useState<DispatchTab>("zones");
  const [selectedZoneId, setSelectedZoneId] = useState<string | null>(null);
  const [showZoneEditor, setShowZoneEditor] = useState(false);
  const [editingZone, setEditingZone] = useState<Zone | undefined>();
  const [drawingMode, setDrawingMode] = useState(false);
  const [timelineCollapsed, setTimelineCollapsed] = useState(false);

  // Active tasks
  const activeTasks = useMemo(
    () =>
      tasks.filter(
        (t) => t.status === TaskStatus.Active || t.status === TaskStatus.Assigned
      ),
    [tasks]
  );

  // Map center from GPS
  const hasDepotPosition = gpsStatus?.latitude != null && gpsStatus?.longitude != null;
  const mapCenter: [number, number] = hasDepotPosition
    ? [gpsStatus.latitude!, gpsStatus.longitude!]
    : [37.7749, -122.4194];
  const mapZoom = hasDepotPosition ? 18 : 15;

  // Zone handlers
  const handleZoneSave = useCallback(
    async (name: string, waypoints: Waypoint[], polygonLatlng?: [number, number][]) => {
      try {
        if (editingZone) {
          await updateZone(editingZone.id, { name, waypoints, polygonLatlng });
        } else {
          await createZone({ name, waypoints, polygonLatlng });
        }
        setShowZoneEditor(false);
        setEditingZone(undefined);
      } catch (e) {
        console.error("Failed to save zone:", e);
      }
    },
    [editingZone, updateZone, createZone]
  );

  const handleDrawComplete = useCallback(
    async (polygon: [number, number][]) => {
      setDrawingMode(false);
      // Open zone editor modal with the drawn polygon
      setEditingZone({
        id: "",
        name: "",
        zoneType: "route",
        waypoints: polygon.map(([lat, lng]) => ({ x: lat, y: lng })),
        polygonLatlng: polygon,
        createdAt: "",
        updatedAt: "",
      } as Zone);
      setShowZoneEditor(true);
    },
    []
  );

  // Coverage generation
  const handleGenerateCoverage = useCallback(
    async (zoneId: string, config: CoverageConfigType) => {
      await generateCoverage(zoneId, config);
    },
    [generateCoverage]
  );

  // Selected zone (for coverage config panel)
  const selectedZone = useMemo(
    () => zones.find((z) => z.id === selectedZoneId) ?? null,
    [zones, selectedZoneId]
  );

  // Quick patrol
  const handleStartPatrol = useCallback(
    async (zoneId: string) => {
      try {
        const zone = zones.find((z) => z.id === zoneId);
        if (!zone) return;
        let mission = missions.find((m) => m.zoneId === zoneId && m.schedule.loop);
        if (!mission) {
          mission = await createMission({
            name: `${zone.name} Patrol`,
            zoneId,
            schedule: { trigger: "manual", loop: true },
          });
        }
        await startMission(mission.id);
      } catch (e) {
        console.error("Failed to start patrol:", e);
      }
    },
    [zones, missions, createMission, startMission]
  );

  const handleStopPatrol = useCallback(
    async (zoneId: string) => {
      try {
        const mission = missions.find((m) => m.zoneId === zoneId);
        if (mission) await stopMission(mission.id);
      } catch (e) {
        console.error("Failed to stop patrol:", e);
      }
    },
    [missions, stopMission]
  );

  const getActiveTask = useCallback(
    (zoneId: string) => {
      const mission = missions.find((m) => m.zoneId === zoneId);
      if (!mission) return null;
      return activeTasks.find((t) => t.missionId === mission.id) || null;
    },
    [missions, activeTasks]
  );

  // Mission handlers
  const handleCreateMission = useCallback(
    async (data: { name: string; zoneId: string; roverId?: string; schedule: Schedule }) => {
      try {
        await createMission(data);
      } catch (e) {
        console.error("Failed to create mission:", e);
      }
    },
    [createMission]
  );

  const handleUpdateMission = useCallback(
    async (id: string, updates: Record<string, unknown>) => {
      try {
        await updateMission(id, updates);
      } catch (e) {
        console.error("Failed to update mission:", e);
      }
    },
    [updateMission]
  );

  return (
    <div className="flex flex-col h-full">
      {/* Main content area */}
      <div className="flex flex-1 min-h-0">
        {/* Map / Zone Editor — Left side (60%) */}
        <div className="flex-1 relative">
          <ZoneEditor
            zones={zones}
            selectedZoneId={selectedZoneId}
            onSelectZone={setSelectedZoneId}
            center={mapCenter}
            zoom={mapZoom}
            drawingMode={drawingMode}
            onDrawComplete={handleDrawComplete}
            showCoveragePath={selectedZone?.zoneType === "coverage"}
          />

          {/* Error overlay */}
          {error && (
            <div className="absolute top-4 left-4 right-4 bg-red-500/90 backdrop-blur rounded-lg p-3 text-white text-sm z-[1000]">
              {error}
            </div>
          )}

          {/* Active task overlay */}
          {activeTasks.length > 0 && (
            <div className="absolute bottom-4 left-4 max-w-sm z-[1000]">
              {activeTasks.slice(0, 2).map((task) => {
                const mission = missions.find((m) => m.id === task.missionId);
                const zone = zones.find((z) => z.id === mission?.zoneId);
                return (
                  <div
                    key={task.id}
                    className="bg-zinc-900/90 backdrop-blur rounded-lg p-3 mb-2 border border-zinc-800"
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
                      <div className="flex-1 h-1 bg-zinc-800 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-green-500 transition-all"
                          style={{ width: `${task.progress}%` }}
                        />
                      </div>
                      <span className="text-xs text-muted-foreground w-20 text-right">
                        WP {task.waypoint + 1} · L{task.lap + 1}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Side Panel — Right side (40%) */}
        <div className="w-80 border-l border-zinc-800 bg-zinc-950 flex flex-col">
          {/* Tab bar */}
          <div className="flex border-b border-zinc-800">
            {(
              [
                { id: "zones", label: "Zones", count: zones.length },
                { id: "missions", label: "Missions", count: missions.length },
                { id: "timeline", label: "History", count: tasks.length },
              ] as const
            ).map(({ id, label, count }) => (
              <button
                key={id}
                className={`flex-1 px-3 py-2.5 text-xs font-medium transition-colors ${
                  activeTab === id
                    ? "text-foreground border-b-2 border-primary"
                    : "text-muted-foreground hover:text-foreground"
                }`}
                onClick={() => setActiveTab(id)}
              >
                {label}
                <Badge variant="outline" className="ml-1.5 text-[9px] h-4 px-1">
                  {count}
                </Badge>
              </button>
            ))}
          </div>

          {/* Panel header */}
          <div className="px-3 py-2 border-b border-zinc-800 flex items-center justify-between">
            <div>
              <p className="text-xs text-muted-foreground">
                {connectedRovers.length} rover{connectedRovers.length !== 1 ? "s" : ""} connected
              </p>
            </div>
            {activeTab === "zones" && (
              <div className="flex gap-1">
                <Button
                  variant={drawingMode ? "default" : "outline"}
                  size="sm"
                  className="h-6 px-2 text-xs"
                  onClick={() => setDrawingMode(!drawingMode)}
                >
                  {drawingMode ? "Cancel Draw" : "Draw Zone"}
                </Button>
                <Button
                  variant="outline"
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
            )}
          </div>

          {/* Tab content */}
          <div className="flex-1 overflow-y-auto">
            {activeTab === "zones" && (
              <>
                <ZoneList
                  zones={zones}
                  selectedZoneId={selectedZoneId}
                  onSelectZone={setSelectedZoneId}
                  onEditZone={(zone) => {
                    setEditingZone(zone);
                    setShowZoneEditor(true);
                  }}
                  onDeleteZone={(id) => {
                    deleteZone(id);
                    setSelectedZoneId(null);
                  }}
                  onStartPatrol={handleStartPatrol}
                  onStopPatrol={handleStopPatrol}
                  getActiveTask={getActiveTask}
                  canStartPatrol={connectedRovers.length > 0}
                />
                {selectedZone && (
                  <div className="px-2 pb-2">
                    <CoverageConfig
                      zoneId={selectedZone.id}
                      initialConfig={selectedZone.coverageConfig}
                      coverageWaypointCount={selectedZone.coverageWaypoints?.length}
                      onGenerate={handleGenerateCoverage}
                    />
                  </div>
                )}
              </>
            )}

            {activeTab === "missions" && (
              <MissionEditor
                missions={missions}
                zones={zones}
                connectedRovers={connectedRovers}
                roverNames={rovers.map((r) => ({ id: r.id, name: r.name }))}
                activeTasks={activeTasks}
                onCreateMission={handleCreateMission}
                onUpdateMission={handleUpdateMission}
                onDeleteMission={(id) => deleteMission(id)}
                onStartMission={(id) => startMission(id)}
                onStopMission={(id) => stopMission(id)}
              />
            )}

            {activeTab === "timeline" && (
              <div className="p-2 space-y-2">
                {/* Recent tasks list */}
                {tasks.length === 0 ? (
                  <div className="p-4 text-center text-xs text-muted-foreground">
                    No tasks yet
                  </div>
                ) : (
                  tasks.slice(0, 20).map((task) => {
                    const mission = missions.find((m) => m.id === task.missionId);
                    const statusColor = {
                      pending: "secondary",
                      assigned: "outline",
                      active: "default",
                      done: "secondary",
                      failed: "destructive",
                      cancelled: "secondary",
                    }[task.status] as "default" | "secondary" | "destructive" | "outline";

                    return (
                      <div
                        key={task.id}
                        className="p-2 bg-zinc-900 rounded-lg border border-zinc-800"
                      >
                        <div className="flex items-center gap-2">
                          <Badge variant={statusColor}>
                            {task.status}
                          </Badge>
                          <span className="text-sm truncate">
                            {mission?.name || "Unknown"}
                          </span>
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">
                          {task.roverId} · {task.lap} laps ·{" "}
                          {new Date(task.createdAt).toLocaleTimeString()}
                          {task.error && (
                            <span className="text-red-400 ml-1">· {task.error}</span>
                          )}
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Bottom Timeline Bar (always visible, collapsible) */}
      <div className="border-t border-zinc-800 bg-zinc-950">
        <button
          className="w-full px-3 py-1.5 flex items-center justify-between hover:bg-zinc-900/50 transition-colors"
          onClick={() => setTimelineCollapsed(!timelineCollapsed)}
        >
          <span className="text-xs font-medium text-muted-foreground">
            Task Timeline
          </span>
          <span className="text-xs text-muted-foreground">
            {timelineCollapsed ? "▲" : "▼"}
          </span>
        </button>
        {!timelineCollapsed && (
          <div className="px-3 pb-2">
            <TaskTimeline
              tasks={tasks}
              missions={missions}
              onCancelTask={cancelTask}
            />
          </div>
        )}
      </div>

      {/* Zone Editor Modal */}
      {showZoneEditor && (
        <ZoneCreateModal
          zone={editingZone}
          onSave={handleZoneSave}
          onClose={() => {
            setShowZoneEditor(false);
            setEditingZone(undefined);
          }}
        />
      )}
    </div>
  );
}
