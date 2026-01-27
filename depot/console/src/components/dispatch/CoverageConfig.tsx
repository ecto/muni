import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { CoverageConfig as CoverageConfigType } from "@/lib/types";

interface CoverageConfigProps {
  zoneId: string;
  initialConfig?: CoverageConfigType | null;
  coverageWaypointCount?: number;
  onGenerate: (zoneId: string, config: CoverageConfigType) => Promise<void>;
}

export function CoverageConfig({
  zoneId,
  initialConfig,
  coverageWaypointCount,
  onGenerate,
}: CoverageConfigProps) {
  const [toolWidth, setToolWidth] = useState(initialConfig?.toolWidth ?? 0.8);
  const [overlapPct, setOverlapPct] = useState(initialConfig?.overlapPct ?? 10);
  const [useCustomAngle, setUseCustomAngle] = useState(
    initialConfig?.swathAngle != null
  );
  const [swathAngle, setSwathAngle] = useState(
    initialConfig?.swathAngle != null
      ? ((initialConfig.swathAngle * 180) / Math.PI).toFixed(0)
      : "0"
  );
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    setGenerating(true);
    setError(null);
    try {
      const config: CoverageConfigType = {
        toolWidth,
        overlapPct,
        swathAngle: useCustomAngle
          ? (parseFloat(swathAngle) * Math.PI) / 180
          : null,
      };
      await onGenerate(zoneId, config);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Generation failed");
    } finally {
      setGenerating(false);
    }
  };

  return (
    <div className="space-y-3 p-3 bg-zinc-900 rounded-lg border border-zinc-800">
      <div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
        Coverage Settings
      </div>

      {/* Tool width */}
      <div>
        <label className="text-xs text-muted-foreground">
          Tool width (m)
        </label>
        <input
          type="number"
          min={0.3}
          max={2.0}
          step={0.1}
          value={toolWidth}
          onChange={(e) => setToolWidth(parseFloat(e.target.value) || 0.8)}
          className="w-full mt-1 px-2 py-1.5 bg-zinc-800 border border-zinc-700 rounded text-sm"
        />
      </div>

      {/* Overlap */}
      <div>
        <label className="text-xs text-muted-foreground">
          Overlap ({overlapPct}%)
        </label>
        <input
          type="range"
          min={0}
          max={50}
          step={5}
          value={overlapPct}
          onChange={(e) => setOverlapPct(parseInt(e.target.value))}
          className="w-full mt-1"
        />
      </div>

      {/* Swath angle */}
      <div>
        <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
          <input
            type="checkbox"
            checked={useCustomAngle}
            onChange={(e) => setUseCustomAngle(e.target.checked)}
            className="rounded border-zinc-600"
          />
          Custom sweep angle
        </label>
        {useCustomAngle && (
          <div className="mt-1 flex items-center gap-2">
            <input
              type="number"
              min={-180}
              max={180}
              step={5}
              value={swathAngle}
              onChange={(e) => setSwathAngle(e.target.value)}
              className="flex-1 px-2 py-1.5 bg-zinc-800 border border-zinc-700 rounded text-sm"
            />
            <span className="text-xs text-muted-foreground">deg</span>
          </div>
        )}
      </div>

      {/* Generate button */}
      <Button
        onClick={handleGenerate}
        disabled={generating}
        className="w-full"
        size="sm"
      >
        {generating ? "Generating..." : "Generate Coverage Path"}
      </Button>

      {/* Error */}
      {error && (
        <div className="text-xs text-red-400">{error}</div>
      )}

      {/* Stats */}
      {coverageWaypointCount != null && coverageWaypointCount > 0 && (
        <div className="text-xs text-muted-foreground">
          {coverageWaypointCount} waypoints generated
        </div>
      )}
    </div>
  );
}
