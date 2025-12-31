import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Compass, Camera } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { CameraMode } from "@/lib/types";

const cameraModeLabels: Record<CameraMode, string> = {
  [CameraMode.ThirdPerson]: "3rd Person",
  [CameraMode.FirstPerson]: "1st Person",
  [CameraMode.FreeLook]: "Free Look",
};

export function PositionPanel() {
  const { renderPose, cameraMode, setCameraMode } = useConsoleStore();

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
    <Card className="w-48 bg-card/90 backdrop-blur">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Compass className="h-4 w-4" weight="fill" />
          Position
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-muted-foreground">X</span>
          <span className="font-mono">
            {renderPose.x >= 0 ? "+" : ""}
            {renderPose.x.toFixed(2)} m
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">Y</span>
          <span className="font-mono">
            {renderPose.y >= 0 ? "+" : ""}
            {renderPose.y.toFixed(2)} m
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">θ</span>
          <span className="font-mono">
            {((renderPose.theta * 180) / Math.PI).toFixed(1)}°
          </span>
        </div>

        <Separator />

        <div className="flex items-center justify-between">
          <span className="flex items-center gap-1 text-muted-foreground">
            <Camera className="h-4 w-4" weight="fill" />
            Camera
          </span>
          <Badge
            variant="secondary"
            className="cursor-pointer hover:bg-secondary/80"
            onClick={cycleCameraMode}
          >
            {cameraModeLabels[cameraMode]}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground">C: toggle · V: free</p>
      </CardContent>
    </Card>
  );
}
