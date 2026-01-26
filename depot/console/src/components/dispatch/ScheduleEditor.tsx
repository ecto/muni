import { useState, useMemo } from "react";
import cronstrue from "cronstrue";
import { Button } from "@/components/ui/button";
import type { Schedule } from "@/lib/types";

interface ScheduleEditorProps {
  schedule: Schedule;
  onChange: (schedule: Schedule) => void;
}

const CRON_PRESETS = [
  { label: "Every hour", cron: "0 * * * *" },
  { label: "Every 2 hours", cron: "0 */2 * * *" },
  { label: "Every 4 hours", cron: "0 */4 * * *" },
  { label: "Every 6 hours", cron: "0 */6 * * *" },
  { label: "Every 12 hours", cron: "0 */12 * * *" },
  { label: "Daily at 6 AM", cron: "0 6 * * *" },
  { label: "Daily at midnight", cron: "0 0 * * *" },
];

function CronDescription({ cron }: { cron: string }) {
  const description = useMemo(() => {
    try {
      return cronstrue.toString(cron);
    } catch {
      return "Invalid cron expression";
    }
  }, [cron]);

  return <span className="text-xs text-muted-foreground">{description}</span>;
}

export function ScheduleEditor({ schedule, onChange }: ScheduleEditorProps) {
  const [cronInput, setCronInput] = useState(schedule.cron || "0 * * * *");

  const handleTriggerChange = (trigger: Schedule["trigger"]) => {
    onChange({
      ...schedule,
      trigger,
      cron: trigger === "cron" ? cronInput : undefined,
    });
  };

  const handleCronChange = (cron: string) => {
    setCronInput(cron);
    onChange({ ...schedule, cron });
  };

  return (
    <div className="space-y-3">
      {/* Trigger type selector */}
      <div>
        <label className="text-xs font-medium text-muted-foreground block mb-1">
          Schedule Type
        </label>
        <div className="flex gap-1">
          {(["manual", "cron"] as const).map((type) => (
            <Button
              key={type}
              variant={schedule.trigger === type ? "default" : "outline"}
              size="sm"
              className="h-7 px-3 text-xs capitalize"
              onClick={() => handleTriggerChange(type)}
            >
              {type}
            </Button>
          ))}
        </div>
      </div>

      {/* Cron configuration */}
      {schedule.trigger === "cron" && (
        <div className="space-y-2 pl-2 border-l-2 border-blue-500/30">
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Cron Expression
            </label>
            <input
              type="text"
              value={cronInput}
              onChange={(e) => handleCronChange(e.target.value)}
              className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-700 rounded-md text-sm font-mono"
              placeholder="0 * * * *"
            />
            <div className="mt-1">
              <CronDescription cron={cronInput} />
            </div>
          </div>

          {/* Presets */}
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Presets
            </label>
            <div className="flex flex-wrap gap-1">
              {CRON_PRESETS.map((preset) => (
                <Button
                  key={preset.cron}
                  variant={cronInput === preset.cron ? "secondary" : "outline"}
                  size="sm"
                  className="h-6 px-2 text-[10px]"
                  onClick={() => handleCronChange(preset.cron)}
                >
                  {preset.label}
                </Button>
              ))}
            </div>
          </div>

          {/* Active window */}
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1">
              Active Window (optional)
            </label>
            <div className="flex items-center gap-2">
              <input
                type="time"
                value={schedule.windowStart || ""}
                onChange={(e) =>
                  onChange({ ...schedule, windowStart: e.target.value || undefined })
                }
                className="px-2 py-1 bg-zinc-900 border border-zinc-700 rounded-md text-xs"
              />
              <span className="text-xs text-muted-foreground">to</span>
              <input
                type="time"
                value={schedule.windowEnd || ""}
                onChange={(e) =>
                  onChange({ ...schedule, windowEnd: e.target.value || undefined })
                }
                className="px-2 py-1 bg-zinc-900 border border-zinc-700 rounded-md text-xs"
              />
            </div>
            {schedule.windowStart && schedule.windowEnd && (
              <div className="text-[10px] text-muted-foreground mt-1">
                Only runs between {schedule.windowStart} and {schedule.windowEnd}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Loop toggle */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="schedule-loop"
          checked={schedule.loop}
          onChange={(e) => onChange({ ...schedule, loop: e.target.checked })}
          className="w-3.5 h-3.5"
        />
        <label htmlFor="schedule-loop" className="text-xs font-medium">
          Loop continuously
        </label>
      </div>
    </div>
  );
}
