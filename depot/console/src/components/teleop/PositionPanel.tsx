import { useState, useEffect } from "react";
import { Badge } from "@/components/ui/badge";
import { Compass, Camera } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { CameraMode } from "@/lib/types";

const cameraModeLabels: Record<CameraMode, string> = {
  [CameraMode.ThirdPerson]: "3rd",
  [CameraMode.FirstPerson]: "1st",
  [CameraMode.FreeLook]: "Free",
};

export function PositionPanel() {
  const cameraMode = useConsoleStore((s) => s.cameraMode);
  const setCameraMode = useConsoleStore((s) => s.setCameraMode);

  // Poll renderPose at 4Hz instead of subscribing (avoids 10Hz+ re-renders)
  const [renderPose, setRenderPose] = useState({ x: 0, y: 0, theta: 0, roll: 0, pitch: 0 });
  useEffect(() => {
    const id = setInterval(() => {
      setRenderPose(useConsoleStore.getState().renderPose);
    }, 250);
    return () => clearInterval(id);
  }, []);

  const cycleCameraMode = () => {
    const modes = [
      CameraMode.ThirdPerson,
      CameraMode.FirstPerson,
      CameraMode.FreeLook,
    ];
    const currentIndex = modes.indexOf(cameraMode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setCameraMode(modes[nextIndex]);
  };

  return (
    <div className="w-36 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border">
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          <Compass className="h-3.5 w-3.5" weight="fill" />
          Position
        </span>
      </div>

      {/* Content */}
      <div className="p-2 space-y-1 text-xs">
        <div className="flex justify-between">
          <span className="text-muted-foreground">X</span>
          <span className="font-mono">
            {renderPose.x >= 0 ? "+" : ""}
            {renderPose.x.toFixed(2)}m
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Y</span>
          <span className="font-mono">
            {renderPose.y >= 0 ? "+" : ""}
            {renderPose.y.toFixed(2)}m
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">θ</span>
          <span className="font-mono">
            {((renderPose.theta * 180) / Math.PI).toFixed(1)}°
          </span>
        </div>

        {/* Camera mode */}
        <div className="flex items-center justify-between pt-1 border-t border-border">
          <span className="flex items-center gap-1 text-muted-foreground">
            <Camera className="h-3.5 w-3.5" weight="fill" />
            Cam
          </span>
          <Badge
            variant="secondary"
            className="text-[10px] h-4 px-1.5 cursor-pointer hover:bg-secondary/80"
            onClick={cycleCameraMode}
          >
            {cameraModeLabels[cameraMode]}
          </Badge>
        </div>
      </div>
    </div>
  );
}
