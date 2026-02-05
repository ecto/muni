import { useState, useEffect, useRef } from "react";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { useConsoleStore } from "@/store";
import type { SlamStatus } from "@/lib/types";

export function SlamPanel() {
  const [slam, setSlam] = useState<SlamStatus | undefined>(
    useConsoleStore.getState().telemetry.slamStatus
  );
  const prevLcRef = useRef(0);
  const [lcFlash, setLcFlash] = useState(false);

  useEffect(() => {
    const id = setInterval(() => {
      setSlam(useConsoleStore.getState().telemetry.slamStatus);
    }, 250);
    return () => clearInterval(id);
  }, []);

  // Flash on loop closure count increase
  useEffect(() => {
    if (!slam) return;
    if (slam.loopClosureCount > prevLcRef.current && prevLcRef.current > 0) {
      setLcFlash(true);
      const timeout = setTimeout(() => setLcFlash(false), 1500);
      return () => clearTimeout(timeout);
    }
    prevLcRef.current = slam.loopClosureCount;
  }, [slam]);

  if (!slam) return null;

  const confPct = Math.round(slam.confidence * 100);
  const confColor =
    slam.confidence >= 0.7
      ? "text-green-500"
      : slam.confidence >= 0.4
        ? "text-yellow-500"
        : "text-red-500";

  return (
    <div className="p-2">
      <div className="grid grid-cols-3 gap-1 text-xs text-center">
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="cursor-help">
              <div className="text-muted-foreground">Conf</div>
              <div className={`font-mono ${confColor}`}>{confPct}%</div>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">
            SLAM scan match confidence. Green &ge;70%, Yellow &ge;40%, Red &lt;40%
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="cursor-help">
              <div className="text-muted-foreground">KF</div>
              <div className="font-mono text-stone-900 dark:text-stone-100">
                {slam.keyframeCount}
              </div>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">
            Keyframes in SLAM pose graph
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className={`cursor-help rounded transition-colors ${
                lcFlash ? "bg-green-500/20 animate-pulse" : ""
              }`}
            >
              <div className="text-muted-foreground">LC</div>
              <div className="font-mono text-stone-900 dark:text-stone-100">
                {slam.loopClosureCount}
              </div>
            </div>
          </TooltipTrigger>
          <TooltipContent side="right">
            Loop closures detected. Flashes green when new closure found.
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
