import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Scene, XRButton } from "@/components/scene/Scene";
import { VideoStatusBadge } from "@/components/scene/EquirectangularSky";
import { TelemetryPanel } from "@/components/teleop/TelemetryPanel";
import { InputPanel } from "@/components/teleop/InputPanel";
import { PositionPanel } from "@/components/teleop/PositionPanel";
import { ConnectionBar } from "@/components/teleop/ConnectionBar";
import { Button } from "@/components/ui/button";
import { useKeyboard } from "@/hooks/useKeyboard";
import { useGamepad } from "@/hooks/useGamepad";
import { useRoverConnection } from "@/hooks/useRoverConnection";
import { useVideoStream } from "@/hooks/useVideoStream";
import { useConsoleStore } from "@/store";
import { Mode } from "@/lib/types";
import { ArrowLeft, Warning } from "@phosphor-icons/react";

export function TeleopView() {
  const navigate = useNavigate();
  const { roverId } = useParams();
  const { rovers, telemetry, selectRover } = useConsoleStore();

  // Initialize input and connection hooks
  useKeyboard();
  useGamepad();
  const { disconnect, sendEStopRelease } = useRoverConnection();
  useVideoStream();

  // Get selected rover info
  const selectedRover = rovers.find((r) => r.id === roverId);
  const isEStop = telemetry.mode === Mode.EStop;

  // Select rover on mount
  useEffect(() => {
    if (roverId) {
      selectRover(roverId);
    }
  }, [roverId, selectRover]);

  // Check for WebXR support
  const [xrSupported, setXrSupported] = useState(false);
  useEffect(() => {
    if (navigator.xr) {
      navigator.xr.isSessionSupported("immersive-vr").then(setXrSupported);
    }
  }, []);

  const handleExit = () => {
    disconnect();
    navigate("/fleet");
  };

  return (
    <div className="h-screen w-screen overflow-hidden bg-background">
      {/* 3D Scene (full screen) */}
      <Scene />

      {/* E-Stop Overlay */}
      {isEStop && (
        <div className="absolute inset-0 bg-red-950/90 flex flex-col items-center justify-center z-50">
          <Warning className="h-24 w-24 text-red-500 mb-6" weight="fill" />
          <h1 className="text-4xl font-bold text-white mb-2">
            Emergency Stop Active
          </h1>
          <p className="text-lg text-red-200 mb-8">
            All motors are disabled. Clear the E-Stop to resume operation.
          </p>
          <Button
            variant="outline"
            size="lg"
            onClick={sendEStopRelease}
            className="border-red-500 text-red-500 hover:bg-red-500 hover:text-white text-lg px-8 py-6"
          >
            Release E-Stop
          </Button>
        </div>
      )}

      {/* UI Overlay */}
      <div className="absolute inset-0 pointer-events-none">
        {/* Top left panels */}
        <div className="absolute top-4 left-4 flex flex-col gap-4 pointer-events-auto">
          {/* Exit button */}
          <Button
            variant="secondary"
            size="sm"
            onClick={handleExit}
            className="w-fit gap-2"
          >
            <ArrowLeft className="h-4 w-4" weight="bold" />
            <span>{selectedRover?.name ?? "Fleet"}</span>
          </Button>
          <TelemetryPanel />
          <InputPanel />
          <PositionPanel />
        </div>

        {/* Top right status and VR button */}
        <div className="absolute top-4 right-4 flex flex-col gap-2 items-end pointer-events-auto">
          <VideoStatusBadge />
          {xrSupported && <XRButton />}
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 pointer-events-auto">
          <ConnectionBar />
        </div>
      </div>
    </div>
  );
}
