import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import {
  CellTower,
  Robot,
  VideoCamera,
  MapTrifold,
  Warning,
  CheckCircle,
} from "@phosphor-icons/react";

export function DashboardView() {
  const { rovers, gpsStatus, sessions } = useConsoleStore();

  const onlineRovers = rovers.filter((r) => r.online);
  const offlineRovers = rovers.filter((r) => !r.online);

  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground">
            Fleet operations overview
          </p>
        </div>

        {/* Alerts */}
        {offlineRovers.length > 0 && (
          <div className="bg-destructive/10 border border-destructive/20 p-4 flex items-start gap-3">
            <Warning className="h-5 w-5 text-destructive mt-0.5" />
            <div>
              <p className="font-medium text-destructive">
                {offlineRovers.length} rover{offlineRovers.length > 1 ? "s" : ""} offline
              </p>
              <p className="text-sm text-muted-foreground">
                {offlineRovers.map((r) => r.name || r.id).join(", ")}
              </p>
            </div>
          </div>
        )}

        {/* Status Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Base Station */}
          <Link
            to="/base-station"
            className="bg-card border border-border p-4 hover:border-primary transition-colors"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <CellTower className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium text-foreground">Base Station</span>
              </div>
              {gpsStatus ? (
                gpsOk ? (
                  <CheckCircle className="h-5 w-5 text-green-500" />
                ) : (
                  <Warning className="h-5 w-5 text-yellow-500" />
                )
              ) : (
                <span className="text-xs text-muted-foreground">Unknown</span>
              )}
            </div>
            {gpsStatus ? (
              <div className="text-sm text-muted-foreground">
                <p>
                  {gpsStatus.mode === "base" ? "Base Mode" : "Rover Mode"} ·{" "}
                  {gpsStatus.satellites} satellites
                </p>
                <p className="font-mono text-xs">
                  {(gpsStatus.fixQuality ?? "no_fix").replace("_", " ").toUpperCase()}
                </p>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                GPS status not available
              </p>
            )}
          </Link>

          {/* Fleet */}
          <Link
            to="/fleet"
            className="bg-card border border-border p-4 hover:border-primary transition-colors"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <Robot className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium text-foreground">Fleet</span>
              </div>
              <span className="text-sm">
                <span className="text-green-500">{onlineRovers.length}</span>
                <span className="text-muted-foreground">/{rovers.length}</span>
              </span>
            </div>
            {rovers.length > 0 ? (
              <div className="text-sm text-muted-foreground">
                {onlineRovers.length > 0 && (
                  <p>
                    Online: {onlineRovers.map((r) => r.name || r.id).join(", ")}
                  </p>
                )}
                {onlineRovers.length === 0 && <p>No rovers online</p>}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No rovers registered
              </p>
            )}
          </Link>

          {/* Sessions */}
          <Link
            to="/sessions"
            className="bg-card border border-border p-4 hover:border-primary transition-colors"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <VideoCamera className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium text-foreground">Sessions</span>
              </div>
              <span className="text-sm text-muted-foreground">
                {sessions.length} total
              </span>
            </div>
            <p className="text-sm text-muted-foreground">
              View recorded sessions and telemetry
            </p>
          </Link>

          {/* Maps */}
          <Link
            to="/maps"
            className="bg-card border border-border p-4 hover:border-primary transition-colors"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <MapTrifold className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium text-foreground">Maps</span>
              </div>
            </div>
            <p className="text-sm text-muted-foreground">
              Browse 3D Gaussian splat maps
            </p>
          </Link>
        </div>

        {/* Quick Stats */}
        <div className="border-t border-border pt-6">
          <h2 className="text-sm font-medium text-muted-foreground mb-4">
            SYSTEM STATUS
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground">Discovery</p>
              <p className="font-mono">● Connected</p>
            </div>
            <div>
              <p className="text-muted-foreground">InfluxDB</p>
              <p className="font-mono">● Healthy</p>
            </div>
            <div>
              <p className="text-muted-foreground">SFTP</p>
              <p className="font-mono">● Running</p>
            </div>
            <div>
              <p className="text-muted-foreground">NTRIP</p>
              <p className="font-mono">
                {gpsStatus?.clients !== undefined
                  ? `● ${gpsStatus.clients} clients`
                  : "○ Unknown"}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
