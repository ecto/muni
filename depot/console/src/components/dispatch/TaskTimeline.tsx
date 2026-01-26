import { useState, useMemo } from "react";
import { Button } from "@/components/ui/button";
import type { Task, Mission } from "@/lib/types";

interface TaskTimelineProps {
  tasks: Task[];
  missions: Mission[];
  onCancelTask: (id: string) => void;
}

type TimeRange = "1h" | "6h" | "24h" | "7d";

const TIME_RANGE_MS: Record<TimeRange, number> = {
  "1h": 60 * 60 * 1000,
  "6h": 6 * 60 * 60 * 1000,
  "24h": 24 * 60 * 60 * 1000,
  "7d": 7 * 24 * 60 * 60 * 1000,
};

function TaskBlock({
  task,
  mission,
  rangeStart,
  rangeEnd,
  onCancel,
}: {
  task: Task;
  mission: Mission | undefined;
  rangeStart: number;
  rangeEnd: number;
  onCancel: () => void;
}) {
  const start = new Date(task.createdAt).getTime();
  const end = task.endedAt ? new Date(task.endedAt).getTime() : Date.now();
  const rangeDuration = rangeEnd - rangeStart;

  // Clamp to visible range
  const visStart = Math.max(start, rangeStart);
  const visEnd = Math.min(end, rangeEnd);
  const left = ((visStart - rangeStart) / rangeDuration) * 100;
  const width = Math.max(((visEnd - visStart) / rangeDuration) * 100, 0.5);

  const statusColor = {
    pending: "bg-zinc-600",
    assigned: "bg-blue-600",
    active: "bg-green-500",
    done: "bg-zinc-500",
    failed: "bg-red-500",
    cancelled: "bg-zinc-600",
  }[task.status] || "bg-zinc-600";

  const isActive = task.status === "active" || task.status === "assigned";

  return (
    <div
      className={`absolute top-0.5 bottom-0.5 rounded-sm group cursor-pointer ${statusColor} ${
        isActive ? "animate-pulse" : ""
      }`}
      style={{ left: `${left}%`, width: `${width}%`, minWidth: "4px" }}
      title={`${mission?.name || "Unknown"} — ${task.status}`}
    >
      {/* Tooltip on hover */}
      <div className="absolute bottom-full left-0 mb-1 px-2 py-1 bg-zinc-900 border border-zinc-700 rounded text-[10px] whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none">
        <div className="font-medium">{mission?.name || "Unknown"}</div>
        <div className="text-muted-foreground">
          {task.roverId} · {task.status}
          {task.progress > 0 && ` · ${task.progress}%`}
        </div>
        <div className="text-muted-foreground">
          {new Date(task.createdAt).toLocaleTimeString()}
          {task.endedAt && ` — ${new Date(task.endedAt).toLocaleTimeString()}`}
        </div>
        {isActive && (
          <button
            className="mt-1 text-red-400 hover:text-red-300 pointer-events-auto"
            onClick={(e) => {
              e.stopPropagation();
              onCancel();
            }}
          >
            Cancel
          </button>
        )}
      </div>
    </div>
  );
}

export function TaskTimeline({
  tasks,
  missions,
  onCancelTask,
}: TaskTimelineProps) {
  const [timeRange, setTimeRange] = useState<TimeRange>("24h");

  const now = Date.now();
  const rangeStart = now - TIME_RANGE_MS[timeRange];
  const rangeEnd = now;

  // Group tasks by rover
  const tasksByRover = useMemo(() => {
    const map = new Map<string, Task[]>();
    for (const task of tasks) {
      const created = new Date(task.createdAt).getTime();
      const ended = task.endedAt ? new Date(task.endedAt).getTime() : now;
      // Only show tasks that overlap with the visible range
      if (ended < rangeStart || created > rangeEnd) continue;

      const existing = map.get(task.roverId) || [];
      existing.push(task);
      map.set(task.roverId, existing);
    }
    return map;
  }, [tasks, rangeStart, rangeEnd]);

  // Time axis labels
  const timeLabels = useMemo(() => {
    const labels: { label: string; position: number }[] = [];
    const steps =
      timeRange === "1h" ? 6 : timeRange === "6h" ? 6 : timeRange === "24h" ? 8 : 7;
    for (let i = 0; i <= steps; i++) {
      const t = rangeStart + (TIME_RANGE_MS[timeRange] * i) / steps;
      const d = new Date(t);
      const label =
        timeRange === "7d"
          ? d.toLocaleDateString(undefined, { weekday: "short" })
          : d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
      labels.push({ label, position: (i / steps) * 100 });
    }
    return labels;
  }, [timeRange, rangeStart]);

  const rovers = Array.from(tasksByRover.keys()).sort();

  return (
    <div className="space-y-2">
      {/* Header with time range selector */}
      <div className="flex items-center justify-between px-2">
        <span className="text-xs font-medium text-muted-foreground">Timeline</span>
        <div className="flex gap-1">
          {(["1h", "6h", "24h", "7d"] as TimeRange[]).map((range) => (
            <Button
              key={range}
              variant={timeRange === range ? "secondary" : "ghost"}
              size="sm"
              className="h-5 px-2 text-[10px]"
              onClick={() => setTimeRange(range)}
            >
              {range}
            </Button>
          ))}
        </div>
      </div>

      {/* Timeline content */}
      {rovers.length === 0 ? (
        <div className="px-2 py-3 text-center text-xs text-muted-foreground">
          No tasks in this time range
        </div>
      ) : (
        <div className="space-y-0.5">
          {/* Time axis */}
          <div className="relative h-4 ml-20">
            {timeLabels.map((t, i) => (
              <span
                key={i}
                className="absolute text-[9px] text-muted-foreground/50 -translate-x-1/2"
                style={{ left: `${t.position}%` }}
              >
                {t.label}
              </span>
            ))}
          </div>

          {/* Rover rows */}
          {rovers.map((roverId) => {
            const roverTasks = tasksByRover.get(roverId) || [];
            return (
              <div key={roverId} className="flex items-center h-6">
                <div className="w-20 pr-2 text-right">
                  <span className="text-[10px] text-muted-foreground truncate block">
                    {roverId}
                  </span>
                </div>
                <div className="flex-1 relative h-full bg-zinc-900/50 rounded-sm border border-zinc-800/50">
                  {roverTasks.map((task) => (
                    <TaskBlock
                      key={task.id}
                      task={task}
                      mission={missions.find((m) => m.id === task.missionId)}
                      rangeStart={rangeStart}
                      rangeEnd={rangeEnd}
                      onCancel={() => onCancelTask(task.id)}
                    />
                  ))}
                </div>
              </div>
            );
          })}

          {/* Legend */}
          <div className="flex items-center gap-3 pl-20 pt-1">
            {[
              { label: "Active", color: "bg-green-500" },
              { label: "Assigned", color: "bg-blue-600" },
              { label: "Done", color: "bg-zinc-500" },
              { label: "Failed", color: "bg-red-500" },
            ].map(({ label, color }) => (
              <div key={label} className="flex items-center gap-1">
                <div className={`w-2 h-2 rounded-sm ${color}`} />
                <span className="text-[9px] text-muted-foreground">{label}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
