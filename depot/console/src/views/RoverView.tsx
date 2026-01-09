import { useParams, Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { Robot, BatteryHigh, MapPin, GameController, ArrowLeft, Thermometer } from "@phosphor-icons/react";
import { ModeLabels, type Mode } from "@/lib/types";

export function RoverView() {
  const { roverId } = useParams<{ roverId: string }>();
  const { rovers } = useConsoleStore();

  const rover = rovers.find((r) => r.id === roverId);

  if (!rover) {
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
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link
              to="/fleet"
              className="h-10 w-10 flex items-center justify-center border border-border hover:border-primary transition-colors"
            >
              <ArrowLeft className="h-4 w-4" />
            </Link>
            <div>
              <h1 className="text-2xl font-bold">{rover.name || rover.id}</h1>
              <p className="text-muted-foreground">
                {rover.online ? "Online" : "Offline"} · {ModeLabels[rover.mode as Mode]}
              </p>
            </div>
          </div>
          {rover.online && (
            <Link
              to={`/fleet/${rover.id}/teleop`}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              <GameController className="h-4 w-4" />
              Enter Teleop
            </Link>
          )}
        </div>

        {/* Status Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-card border border-border p-4">
            <div className="flex items-center gap-2 mb-2">
              <BatteryHigh className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Battery</span>
            </div>
            <p className="text-2xl font-mono">{rover.batteryVoltage.toFixed(1)}V</p>
          </div>

          <div className="bg-card border border-border p-4">
            <div className="flex items-center gap-2 mb-2">
              <MapPin className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Position</span>
            </div>
            <p className="text-lg font-mono">
              ({rover.lastPose.x.toFixed(2)}, {rover.lastPose.y.toFixed(2)})
            </p>
            <p className="text-sm text-muted-foreground font-mono">
              θ = {(rover.lastPose.theta * 180 / Math.PI).toFixed(1)}°
            </p>
          </div>

          <div className="bg-card border border-border p-4">
            <div className="flex items-center gap-2 mb-2">
              <Thermometer className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Motors</span>
            </div>
            <p className="text-lg font-mono">--°C</p>
            <p className="text-sm text-muted-foreground">Telemetry not connected</p>
          </div>
        </div>

        {/* Connection Info */}
        <div className="bg-card border border-border p-6">
          <h2 className="font-medium mb-4">Connection Details</h2>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground mb-1">Rover ID</p>
              <p className="font-mono">{rover.id}</p>
            </div>
            <div>
              <p className="text-muted-foreground mb-1">WebSocket Address</p>
              <p className="font-mono">{rover.address}</p>
            </div>
            <div>
              <p className="text-muted-foreground mb-1">Video Address</p>
              <p className="font-mono">{rover.videoAddress}</p>
            </div>
            <div>
              <p className="text-muted-foreground mb-1">Last Seen</p>
              <p className="font-mono">
                {new Date(rover.lastSeen).toLocaleTimeString()}
              </p>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-4">
          <Link
            to={`/sessions?rover=${rover.id}`}
            className="px-4 py-2 border border-border hover:border-primary transition-colors"
          >
            View Sessions
          </Link>
        </div>
      </div>
    </div>
  );
}
