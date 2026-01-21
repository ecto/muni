import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Scene } from "@/components/scene/Scene";
import { CameraFeedCard } from "@/components/scene/EquirectangularSky";
import { TelemetryPanel } from "@/components/teleop/TelemetryPanel";
import { InputPanel } from "@/components/teleop/InputPanel";
import { ConnectionBar } from "@/components/teleop/ConnectionBar";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { useKeyboard } from "@/hooks/useKeyboard";
import { useGamepad } from "@/hooks/useGamepad";
import { useRoverConnectionRtc } from "@/hooks/useRoverConnectionRtc";
import { useConsoleStore } from "@/store";
import { Mode } from "@/lib/types";
import { ArrowLeft, Warning, Play, Stop, CircleNotch, Robot } from "@phosphor-icons/react";
import { Link } from "react-router-dom";

export function RoverView() {
  const navigate = useNavigate();
  const { roverId } = useParams();
  const { rovers, telemetry, selectRover, rtcAddress, connecting, connected } = useConsoleStore();

  // Initialize input and connection hooks
  useKeyboard();
  useGamepad();
  // WebRTC handles both commands and video streaming
  const { disconnect, sendEStopRelease, sendEnable, sendDisable } = useRoverConnectionRtc();

  // Get selected rover info
  const selectedRover = rovers.find((r) => r.id === roverId);
  const isEStop = telemetry.mode === Mode.EStop;
  const isDisabled = telemetry.mode === Mode.Disabled || telemetry.mode === Mode.Idle;
  const isTeleop = telemetry.mode === Mode.Teleop;

  // Loading states for enable/disable actions
  const [enabling, setEnabling] = useState(false);
  const [disabling, setDisabling] = useState(false);

  // Clear loading states when mode changes or after timeout
  useEffect(() => {
    if (isTeleop) setEnabling(false);
    if (isDisabled) setDisabling(false);
  }, [isTeleop, isDisabled]);

  // Timeout to clear loading states if command fails
  useEffect(() => {
    if (enabling) {
      const timeout = setTimeout(() => setEnabling(false), 3000);
      return () => clearTimeout(timeout);
    }
  }, [enabling]);

  useEffect(() => {
    if (disabling) {
      const timeout = setTimeout(() => setDisabling(false), 3000);
      return () => clearTimeout(timeout);
    }
  }, [disabling]);

  // Select rover on mount and when rovers list updates
  // (rover might not be available immediately if discovery data hasn't arrived)
  // Only re-select if the rtc address hasn't been set yet
  useEffect(() => {
    if (roverId && rtcAddress === "ws://localhost:4852") {
      selectRover(roverId);
    }
  }, [roverId, selectRover, rovers, rtcAddress]);

  // Disconnect when leaving
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  const handleExit = () => {
    disconnect();
    navigate("/fleet");
  };

  // Show not found state
  if (!selectedRover && rovers.length > 0) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <Robot className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
          <h3 className="font-medium mb-2">Rover Not Found</h3>
          <p className="text-sm text-muted-foreground mb-4">
            Rover "{roverId}" is not registered
          </p>
          <Link to="/fleet" className="text-primary hover:underline">
            ← Back to fleet
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full overflow-hidden bg-background relative">
      {/* 3D Scene (full area) */}
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
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="lg"
                onClick={sendEStopRelease}
                className="border-red-500 text-red-500 hover:bg-red-500 hover:text-white text-lg px-8 py-6"
              >
                Release E-Stop
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">Clear emergency stop and return to idle</TooltipContent>
          </Tooltip>
        </div>
      )}

      {/* Connecting Overlay */}
      {connecting && !connected && (
        <div className="absolute inset-0 flex items-center justify-center z-40 pointer-events-none">
          <div className="flex flex-col items-center gap-3">
            <CircleNotch className="h-12 w-12 animate-spin text-primary" />
            <span className="text-lg text-muted-foreground">
              Connecting to {selectedRover?.name ?? "rover"}...
            </span>
          </div>
        </div>
      )}

      {/* UI Overlay */}
      <div className="absolute inset-0 pointer-events-none">
        {/* Top left panels - scrollable for shorter windows */}
        <div className="absolute top-4 left-4 bottom-16 flex flex-col gap-3 pointer-events-auto overflow-y-auto overflow-x-hidden pr-2 scrollbar-thin">
          {/* Exit button */}
          <Button
            variant="secondary"
            size="sm"
            onClick={handleExit}
            className="w-fit gap-2 shrink-0"
          >
            <ArrowLeft className="h-4 w-4" weight="bold" />
            <span>{selectedRover?.name ?? "Fleet"}</span>
          </Button>
          <TelemetryPanel />
          <InputPanel />
          <CameraFeedCard />
        </div>

        {/* Top right controls */}
        <div className="absolute top-4 right-4 pointer-events-auto flex items-center gap-2">
          {/* Enable/Disable button */}
          {isDisabled && !enabling && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="default"
                  size="sm"
                  disabled={!connected}
                  onClick={() => {
                    setEnabling(true);
                    sendEnable();
                  }}
                  className="gap-2 bg-green-600 hover:bg-green-500 disabled:bg-gray-600"
                >
                  <Play className="h-4 w-4" weight="fill" />
                  Enable
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Enter teleop mode and start accepting drive commands</TooltipContent>
            </Tooltip>
          )}
          {enabling && (
            <Button variant="secondary" size="sm" disabled className="gap-2">
              <CircleNotch className="h-4 w-4 animate-spin" />
              Enabling...
            </Button>
          )}
          {isTeleop && !disabling && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="default"
                  size="sm"
                  onClick={() => {
                    setDisabling(true);
                    sendDisable();
                  }}
                  className="gap-2 bg-amber-600 hover:bg-amber-500"
                >
                  <Stop className="h-4 w-4" weight="fill" />
                  Disable
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Exit teleop mode and stop motors</TooltipContent>
            </Tooltip>
          )}
          {disabling && (
            <Button variant="secondary" size="sm" disabled className="gap-2">
              <CircleNotch className="h-4 w-4 animate-spin" />
              Disabling...
            </Button>
          )}
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 pointer-events-auto">
          <ConnectionBar />
        </div>
      </div>
    </div>
  );
}
