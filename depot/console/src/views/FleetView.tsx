import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { Robot, BatteryHigh, MapPin, GameController } from "@phosphor-icons/react";
import { ModeLabels, type Mode } from "@/lib/types";

export function FleetView() {
  const { rovers } = useConsoleStore();

  const onlineRovers = rovers.filter((r) => r.online);
  const offlineRovers = rovers.filter((r) => !r.online);

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Fleet</h1>
            <p className="text-muted-foreground">
              {onlineRovers.length} online, {offlineRovers.length} offline
            </p>
          </div>
        </div>

        {/* Rover List */}
        {rovers.length === 0 ? (
          <div className="bg-muted/50 border border-border p-8 text-center">
            <Robot className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
            <h3 className="font-medium mb-2">No Rovers Registered</h3>
            <p className="text-sm text-muted-foreground max-w-md mx-auto">
              Rovers will appear here once they connect to the discovery service.
              Make sure rovers are configured to register with this depot.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {rovers.map((rover) => (
              <Link
                key={rover.id}
                to={`/fleet/${rover.id}`}
                className="bg-card border border-border p-4 flex items-center gap-4 hover:border-primary transition-colors block"
              >
                <div
                  className={`h-12 w-12 flex items-center justify-center ${
                    rover.online ? "bg-primary/20" : "bg-muted"
                  }`}
                >
                  <Robot
                    className={`h-6 w-6 ${
                      rover.online ? "text-primary" : "text-muted-foreground"
                    }`}
                  />
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{rover.name || rover.id}</h3>
                    <span
                      className={`text-xs px-2 py-0.5 ${
                        rover.online
                          ? "bg-green-500/20 text-green-500"
                          : "bg-muted text-muted-foreground"
                      }`}
                    >
                      {rover.online ? "ONLINE" : "OFFLINE"}
                    </span>
                    <span className="text-xs px-2 py-0.5 bg-muted text-muted-foreground">
                      {ModeLabels[rover.mode as Mode]}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 mt-1 text-sm text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <BatteryHigh className="h-3 w-3" />
                      {rover.batteryVoltage.toFixed(1)}V
                    </span>
                    <span className="flex items-center gap-1 font-mono text-xs">
                      <MapPin className="h-3 w-3" />
                      ({rover.lastPose.x.toFixed(1)}, {rover.lastPose.y.toFixed(1)})
                    </span>
                  </div>
                </div>

                {rover.online && (
                  <Link
                    to={`/fleet/${rover.id}/teleop`}
                    className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <GameController className="h-4 w-4" />
                    Teleop
                  </Link>
                )}
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
