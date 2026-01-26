import { useEffect } from "react";
import { useParams, Link } from "react-router-dom";
import { Scene } from "@/components/scene/Scene";
import { CameraFeedCard } from "@/components/scene/EquirectangularSky";
import { ModeBar } from "@/components/teleop/ModeBar";
import { StatusPanel } from "@/components/teleop/StatusPanel";
import { DiagnosticsPanel } from "@/components/teleop/DiagnosticsPanel";
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
import { Warning, CircleNotch, Robot } from "@phosphor-icons/react";

export function RoverView() {
  const { roverId } = useParams();
  const rovers = useConsoleStore((s) => s.rovers);
  const telemetryMode = useConsoleStore((s) => s.telemetry.mode);
  const selectRover = useConsoleStore((s) => s.selectRover);
  const rtcAddress = useConsoleStore((s) => s.rtcAddress);
  const connecting = useConsoleStore((s) => s.connecting);
  const connected = useConsoleStore((s) => s.connected);

  // Initialize input and connection hooks
  useKeyboard();
  useGamepad();
  // WebRTC handles both commands and video streaming
  const { disconnect, sendEStopRelease, sendEnable, sendDisable, sendSleep, sendWake, sendAutonomous, sendStopAutonomy, toggleLidar } = useRoverConnectionRtc();

  // Get selected rover info
  const selectedRover = rovers.find((r) => r.id === roverId);
  const isEStop = telemetryMode === Mode.EStop;

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
        {/* Breadcrumb — top left */}
        <div className="absolute top-4 left-4 pointer-events-auto z-10">
          <nav className="flex items-center gap-1.5 text-xs bg-card/80 backdrop-blur-sm border border-border rounded-md px-2.5 py-1.5 shadow-sm">
            <Link to="/fleet" className="text-muted-foreground hover:text-foreground transition-colors">
              Fleet
            </Link>
            <span className="text-muted-foreground/40">/</span>
            <span className="font-medium">{selectedRover?.name ?? roverId}</span>
          </nav>
        </div>

        {/* Top center — Mode bar */}
        <div className="absolute top-4 left-1/2 -translate-x-1/2 pointer-events-auto z-10">
          <ModeBar
            sendSleep={sendSleep}
            sendWake={sendWake}
            sendEnable={sendEnable}
            sendDisable={sendDisable}
            sendAutonomous={sendAutonomous}
            sendStopAutonomy={sendStopAutonomy}
          />
        </div>

        {/* Right side panels */}
        <div className="absolute top-16 right-4 bottom-12 flex flex-col gap-3 pointer-events-auto overflow-y-auto overflow-x-hidden pl-2 scrollbar-thin items-end">
          <StatusPanel />
          <DiagnosticsPanel onToggleLidar={toggleLidar} />
          <InputPanel />
        </div>

        {/* Bottom left — Camera feeds */}
        <div className="absolute bottom-12 left-4 pointer-events-auto">
          <CameraFeedCard />
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 pointer-events-auto">
          <ConnectionBar />
        </div>
      </div>
    </div>
  );
}
