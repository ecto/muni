import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { Scene } from "@/components/scene/Scene";
import { ModeBar } from "@/components/teleop/ModeBar";
import { TeleopSidebar } from "@/components/teleop/TeleopSidebar";
import { ConnectionBar } from "@/components/teleop/ConnectionBar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
  const connectDirect = useConsoleStore((s) => s.connectDirect);
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

  // Auto-connect directly when discovery is unavailable but we have a rover ID from the URL
  useEffect(() => {
    if (!selectedRover && rovers.length === 0 && rtcAddress === "ws://localhost:4852" && roverId) {
      connectDirect(roverId);
    }
  }, [selectedRover, rovers.length, rtcAddress, roverId, connectDirect]);

  // Show direct connect prompt only when we don't have a rover ID to try
  if (!selectedRover && rovers.length === 0 && rtcAddress === "ws://localhost:4852" && !roverId) {
    return <DirectConnectPrompt roverId={roverId} onConnect={connectDirect} />;
  }

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

        {/* Right side panel */}
        <div className="absolute top-16 right-4 bottom-12 pointer-events-auto">
          <TeleopSidebar onToggleLidar={toggleLidar} />
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 pointer-events-auto">
          <ConnectionBar />
        </div>
      </div>
    </div>
  );
}

function DirectConnectPrompt({
  roverId,
  onConnect,
}: {
  roverId: string | undefined;
  onConnect: (hostname: string) => void;
}) {
  const [hostname, setHostname] = useState(roverId ?? "");

  const submit = () => {
    const trimmed = hostname.trim();
    if (trimmed) onConnect(trimmed);
  };

  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center">
        <Robot className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
        <h3 className="font-medium mb-1">No rovers discovered</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Connect directly by hostname or IP
        </p>
        <div className="flex gap-2 max-w-xs mx-auto">
          <Input
            value={hostname}
            onChange={(e) => setHostname(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()}
            placeholder="frog-0 or 192.168.1.100"
            autoFocus
          />
          <Button onClick={submit} disabled={!hostname.trim()}>
            Connect
          </Button>
        </div>
      </div>
    </div>
  );
}
