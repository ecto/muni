import { useOperatorStore } from "@/store";
import { View, Mode, ModeLabels, type RoverInfo } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  MapPin,
  Plugs,
  Robot,
  GameController,
  MapTrifold,
  VideoCamera,
} from "@phosphor-icons/react";
import { useRef, useEffect, useCallback, useState } from "react";
import { useDiscovery } from "@/hooks/useDiscovery";

// Map constants
const MIN_SCALE = 4; // minimum pixels per meter
const MAX_SCALE = 20; // maximum pixels per meter
const MAP_PADDING = 60; // padding around rovers in pixels

function BatteryIcon({ voltage }: { voltage: number }) {
  if (voltage < 42)
    return (
      <BatteryWarning className="h-4 w-4 text-destructive" weight="fill" />
    );
  if (voltage < 45)
    return <BatteryLow className="h-4 w-4 text-orange-500" weight="fill" />;
  return <BatteryFull className="h-4 w-4 text-green-500" weight="fill" />;
}

function getModeVariant(
  mode: Mode
): "default" | "secondary" | "destructive" | "outline" {
  switch (mode) {
    case Mode.Teleop:
    case Mode.Autonomous:
      return "default";
    case Mode.EStop:
    case Mode.Fault:
      return "destructive";
    case Mode.Idle:
      return "secondary";
    default:
      return "outline";
  }
}

function RoverListItem({
  rover,
  isSelected,
  onSelect,
  onConnect,
  now,
}: {
  rover: RoverInfo;
  isSelected: boolean;
  onSelect: () => void;
  onConnect: () => void;
  now: number;
}) {
  const timeSinceLastSeen = now - rover.lastSeen;
  const lastSeenText =
    timeSinceLastSeen < 60000
      ? "Just now"
      : timeSinceLastSeen < 3600000
      ? `${Math.floor(timeSinceLastSeen / 60000)}m ago`
      : `${Math.floor(timeSinceLastSeen / 3600000)}h ago`;

  return (
    <div
      className={`p-3 cursor-pointer transition-colors border-l-2 ${
        isSelected
          ? "bg-accent border-l-primary"
          : "border-l-transparent hover:bg-accent/50"
      }`}
      onClick={onSelect}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2">
          <Robot
            className={`h-5 w-5 ${
              rover.online ? "text-primary" : "text-muted-foreground"
            }`}
            weight="fill"
          />
          <div>
            <div className="font-medium text-sm">{rover.name}</div>
            <div className="text-xs text-muted-foreground flex items-center gap-1">
              {rover.online ? (
                <>
                  <span className="inline-block w-1.5 h-1.5 rounded-full bg-green-500" />
                  Online
                </>
              ) : (
                <>
                  <span className="inline-block w-1.5 h-1.5 rounded-full bg-muted-foreground" />
                  {lastSeenText}
                </>
              )}
            </div>
          </div>
        </div>
        <Badge variant={getModeVariant(rover.mode)} className="text-xs">
          {ModeLabels[rover.mode]}
        </Badge>
      </div>

      <div className="mt-2 flex items-center justify-between text-xs text-muted-foreground">
        <div className="flex items-center gap-1">
          <BatteryIcon voltage={rover.batteryVoltage} />
          <span className="font-mono">{rover.batteryVoltage.toFixed(1)}V</span>
        </div>
        <div className="flex items-center gap-1">
          <MapPin className="h-3 w-3" weight="fill" />
          <span className="font-mono">
            ({rover.lastPose.x.toFixed(1)}, {rover.lastPose.y.toFixed(1)})
          </span>
        </div>
      </div>

      {isSelected && (
        <Button
          className="w-full mt-3 gap-2"
          size="sm"
          disabled={!rover.online}
          onClick={(e) => {
            e.stopPropagation();
            onConnect();
          }}
        >
          <GameController className="h-4 w-4" weight="fill" />
          {rover.online ? "Connect & Control" : "Offline"}
        </Button>
      )}
    </div>
  );
}

function FleetMap({
  rovers,
  selectedRoverId,
  onSelectRover,
}: {
  rovers: RoverInfo[];
  selectedRoverId: string | null;
  onSelectRover: (id: string) => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const transformRef = useRef({ centerX: 0, centerY: 0, scale: 10 });

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;

    // Calculate bounds of all rovers
    let minX = 0,
      maxX = 0,
      minY = 0,
      maxY = 0;
    if (rovers.length > 0) {
      minX = Math.min(...rovers.map((r) => r.lastPose.x));
      maxX = Math.max(...rovers.map((r) => r.lastPose.x));
      minY = Math.min(...rovers.map((r) => r.lastPose.y));
      maxY = Math.max(...rovers.map((r) => r.lastPose.y));
    }

    // Add some padding to bounds
    const boundsWidth = Math.max(maxX - minX, 20); // minimum 20m
    const boundsHeight = Math.max(maxY - minY, 20);

    // Calculate scale to fit all rovers
    const scaleX = (width - MAP_PADDING * 2) / boundsWidth;
    const scaleY = (height - MAP_PADDING * 2) / boundsHeight;
    const scale = Math.min(
      Math.max(Math.min(scaleX, scaleY), MIN_SCALE),
      MAX_SCALE
    );

    // Calculate center in world coordinates
    const worldCenterX = (minX + maxX) / 2;
    const worldCenterY = (minY + maxY) / 2;

    // Map center in screen coordinates
    const mapCenterX = width / 2;
    const mapCenterY = height / 2;

    // Store transform for click handling
    transformRef.current = {
      centerX: worldCenterX,
      centerY: worldCenterY,
      scale,
    };

    // Helper to convert world to screen coordinates
    const toScreen = (wx: number, wy: number) => ({
      x: mapCenterX + (wx - worldCenterX) * scale,
      y: mapCenterY - (wy - worldCenterY) * scale, // Flip Y
    });

    // Clear canvas
    ctx.fillStyle = "#1c1917"; // stone-900
    ctx.fillRect(0, 0, width, height);

    // Draw grid
    ctx.strokeStyle = "rgba(255, 255, 255, 0.05)";
    ctx.lineWidth = 1;

    const gridSpacing = scale * 10; // 10m grid
    const origin = toScreen(0, 0);

    // Vertical grid lines
    for (let x = origin.x % gridSpacing; x < width; x += gridSpacing) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    // Horizontal grid lines
    for (let y = origin.y % gridSpacing; y < height; y += gridSpacing) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw origin marker
    ctx.strokeStyle = "rgba(255, 255, 255, 0.3)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(origin.x - 15, origin.y);
    ctx.lineTo(origin.x + 15, origin.y);
    ctx.moveTo(origin.x, origin.y - 15);
    ctx.lineTo(origin.x, origin.y + 15);
    ctx.stroke();

    // Origin label
    ctx.fillStyle = "rgba(255, 255, 255, 0.3)";
    ctx.font = "10px Inter, system-ui, sans-serif";
    ctx.textAlign = "left";
    ctx.fillText("(0, 0)", origin.x + 8, origin.y - 8);

    // Draw rovers
    rovers.forEach((rover) => {
      const pos = toScreen(rover.lastPose.x, rover.lastPose.y);
      const isSelected = rover.id === selectedRoverId;

      // Selection ring
      if (isSelected) {
        ctx.strokeStyle = "#f97316"; // orange-500
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 22, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Rover body
      ctx.save();
      ctx.translate(pos.x, pos.y);
      ctx.rotate(-rover.lastPose.theta); // Negative because screen Y is flipped

      // Body rectangle (slightly larger for visibility)
      ctx.fillStyle = rover.online
        ? isSelected
          ? "#f97316" // orange-500
          : "#78716c" // stone-500
        : "#44403c"; // stone-700
      ctx.fillRect(-10, -14, 20, 28);

      // Direction indicator (front)
      ctx.fillStyle = rover.online ? "#fbbf24" : "#57534e"; // amber-400 / stone-600
      ctx.fillRect(-6, -14, 12, 5);

      ctx.restore();

      // Label background for readability
      ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
      const label = rover.name.replace("Beaver-", "BVR-");
      const labelWidth = ctx.measureText(label).width + 8;
      ctx.fillRect(pos.x - labelWidth / 2, pos.y + 20, labelWidth, 16);

      // Label
      ctx.fillStyle = rover.online ? "#fafaf9" : "#a8a29e"; // stone-50 / stone-400
      ctx.font = "11px Inter, system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(label, pos.x, pos.y + 32);

      // Online indicator dot
      ctx.beginPath();
      ctx.arc(pos.x + labelWidth / 2 - 4, pos.y + 28, 3, 0, Math.PI * 2);
      ctx.fillStyle = rover.online ? "#22c55e" : "#ef4444"; // green-500 / red-500
      ctx.fill();
    });

    // Draw scale bar
    const scaleBarMeters = 10;
    const scaleBarPixels = scaleBarMeters * scale;
    ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(20, height - 30);
    ctx.lineTo(20 + scaleBarPixels, height - 30);
    ctx.stroke();

    // Scale ticks
    ctx.beginPath();
    ctx.moveTo(20, height - 35);
    ctx.lineTo(20, height - 25);
    ctx.moveTo(20 + scaleBarPixels, height - 35);
    ctx.lineTo(20 + scaleBarPixels, height - 25);
    ctx.stroke();

    // Scale label
    ctx.fillStyle = "rgba(255, 255, 255, 0.5)";
    ctx.font = "11px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(`${scaleBarMeters}m`, 20 + scaleBarPixels / 2, height - 15);
  }, [rovers, selectedRoverId]);

  useEffect(() => {
    draw();
  }, [draw]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resizeObserver = new ResizeObserver(() => {
      canvas.width = canvas.offsetWidth;
      canvas.height = canvas.offsetHeight;
      draw();
    });

    resizeObserver.observe(canvas);
    return () => resizeObserver.disconnect();
  }, [draw]);

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    const { centerX, centerY, scale } = transformRef.current;
    const mapCenterX = canvas.width / 2;
    const mapCenterY = canvas.height / 2;

    // Find clicked rover
    for (const rover of rovers) {
      const screenX = mapCenterX + (rover.lastPose.x - centerX) * scale;
      const screenY = mapCenterY - (rover.lastPose.y - centerY) * scale;
      const distance = Math.sqrt(
        (clickX - screenX) ** 2 + (clickY - screenY) ** 2
      );

      if (distance < 25) {
        onSelectRover(rover.id);
        return;
      }
    }
  };

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-full cursor-crosshair"
      onClick={handleClick}
    />
  );
}

export function HomeScreen() {
  const { rovers, selectedRoverId, selectRover, setView } = useOperatorStore();

  // Connect to discovery service to get live rover updates
  useDiscovery();

  // Current time for "last seen" display, updated every minute
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const interval = setInterval(() => setNow(Date.now()), 60000);
    return () => clearInterval(interval);
  }, []);

  const handleConnect = (roverId: string) => {
    selectRover(roverId);
    setView(View.Teleop);
  };

  const onlineCount = rovers.filter((r) => r.online).length;

  return (
    <div className="h-screen w-screen overflow-hidden bg-background flex dark">
      {/* Rover list panel (left side) */}
      <div className="w-80 h-full flex flex-col border-r border-border bg-card">
        <div className="p-4 border-b border-border">
          <div className="flex items-center gap-2 mb-1">
            <Robot className="h-5 w-5 text-primary" weight="fill" />
            <h1 className="font-semibold text-lg">Fleet Overview</h1>
          </div>
          <div className="flex items-center gap-3 text-sm text-muted-foreground">
            <span className="flex items-center gap-1">
              <Plugs className="h-4 w-4" weight="fill" />
              {onlineCount}/{rovers.length} online
            </span>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto">
          {rovers.length === 0 ? (
            <div className="p-6 text-center text-muted-foreground">
              <Robot
                className="h-12 w-12 mx-auto mb-3 opacity-50"
                weight="thin"
              />
              <p className="text-sm">No rovers detected</p>
              <p className="text-xs mt-1">Waiting for rovers to register...</p>
            </div>
          ) : (
            rovers.map((rover) => (
              <div key={rover.id}>
                <RoverListItem
                  rover={rover}
                  isSelected={selectedRoverId === rover.id}
                  onSelect={() => selectRover(rover.id)}
                  onConnect={() => handleConnect(rover.id)}
                  now={now}
                />
                <Separator />
              </div>
            ))
          )}
        </div>

        <div className="p-4 border-t border-border space-y-2">
          <Button
            variant="outline"
            className="w-full gap-2"
            onClick={() => setView(View.Sessions)}
          >
            <VideoCamera className="h-4 w-4" weight="fill" />
            Session History
          </Button>
          <Button
            variant="outline"
            className="w-full gap-2"
            onClick={() => setView(View.Maps)}
          >
            <MapTrifold className="h-4 w-4" weight="fill" />
            Browse Maps
          </Button>
          <div className="text-xs text-muted-foreground text-center mt-2">
            Muni Robotics Operator Console
          </div>
        </div>
      </div>

      {/* Map area (main content) */}
      <div className="flex-1 relative">
        <FleetMap
          rovers={rovers}
          selectedRoverId={selectedRoverId}
          onSelectRover={selectRover}
        />

        {/* Map legend */}
        <Card className="absolute bottom-4 right-4 bg-card/90 backdrop-blur">
          <CardHeader className="pb-2 pt-4">
            <CardTitle className="text-xs font-medium text-muted-foreground">
              Map Legend
            </CardTitle>
          </CardHeader>
          <CardContent className="pb-4 space-y-2 text-xs">
            <div className="flex items-center gap-2">
              <div className="w-4 h-6 bg-stone-500 rounded-sm" />
              <span>Online rover</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-6 bg-stone-700 rounded-sm" />
              <span>Offline rover</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-6 bg-primary rounded-sm" />
              <span>Selected</span>
            </div>
          </CardContent>
        </Card>

        {/* Scale indicator */}
        <div className="absolute bottom-4 left-4 flex items-center gap-2 text-xs text-muted-foreground">
          <div className="w-20 h-0.5 bg-muted-foreground" />
          <span>10m</span>
        </div>
      </div>
    </div>
  );
}
