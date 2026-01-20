import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import {
  CellTower,
  Robot,
  Warning,
  GameController,
  BatteryHigh,
  ArrowRight,
} from "@phosphor-icons/react";
import { ModeLabels, type Mode } from "@/lib/types";
import {
  Map,
  MapTileLayer,
  MapMarker,
  MapZoomControl,
  MapFullscreenControl,
  MapControlContainer,
} from "@/components/ui/map";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

// Default center (San Francisco) when no GPS data
const DEFAULT_CENTER: [number, number] = [37.7749, -122.4194];
const DEFAULT_ZOOM = 12;

export function DashboardView() {
  const { rovers, gpsStatus } = useConsoleStore();

  const onlineRovers = rovers.filter((r) => r.online);
  const offlineRovers = rovers.filter((r) => !r.online);

  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

  // Use depot GPS position as map center when available
  const hasDepotPosition = gpsStatus?.latitude != null && gpsStatus?.longitude != null;
  const mapCenter: [number, number] = hasDepotPosition
    ? [gpsStatus.latitude!, gpsStatus.longitude!]
    : DEFAULT_CENTER;
  const mapZoom = hasDepotPosition ? 18 : DEFAULT_ZOOM;

  return (
    <div className="h-full relative">
      {/* Full-page map */}
      <Map center={mapCenter} zoom={mapZoom} className="h-full w-full">
        <MapTileLayer />

        {/* Depot/Base Station marker */}
        {hasDepotPosition && (
          <MapMarker position={[gpsStatus.latitude!, gpsStatus.longitude!]}>
            <div className="relative group">
              <div
                className={`flex items-center justify-center w-10 h-10 rounded-full border-3 border-white shadow-lg ${
                  gpsOk
                    ? "bg-green-500 ring-2 ring-green-500/30"
                    : "bg-yellow-500 ring-2 ring-yellow-500/30"
                }`}
              >
                <CellTower className="w-5 h-5 text-white" weight="bold" />
              </div>
              {/* Tooltip */}
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-background border border-border rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                <span className="font-medium">Depot Base Station</span>
                <br />
                <span className="text-muted-foreground">
                  {gpsStatus.satellites} sats · {(gpsStatus.fixQuality ?? "no_fix").replace("_", " ")}
                </span>
              </div>
            </div>
          </MapMarker>
        )}

        {/* Online rover markers */}
        {onlineRovers.map((rover) => {
          // For now, show rovers at depot location with small offset
          // In the future, rovers will report their own GPS coordinates
          if (!hasDepotPosition) return null;

          // Offset each rover slightly from depot (roughly 10 meters per rover)
          const idx = onlineRovers.indexOf(rover);
          const offset = 0.0001 * (idx + 1); // ~11 meters per unit at this latitude
          const roverLat = gpsStatus.latitude! + offset;
          const roverLon = gpsStatus.longitude! + offset;

          return (
            <MapMarker key={rover.id} position={[roverLat, roverLon]}>
              <Link to={`/fleet/${rover.id}`} className="relative group">
                <div className="flex items-center justify-center w-8 h-8 bg-primary rounded-full border-2 border-white shadow-lg ring-2 ring-primary/30">
                  <Robot className="w-4 h-4 text-primary-foreground" weight="bold" />
                </div>
                {/* Tooltip */}
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-background border border-border rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity">
                  <span className="font-medium">{rover.name || rover.id}</span>
                  <br />
                  <span className="text-muted-foreground">
                    {ModeLabels[rover.mode as Mode]} · {rover.batteryVoltage.toFixed(1)}V
                  </span>
                </div>
              </Link>
            </MapMarker>
          );
        })}

        <MapControlContainer className="top-3 right-3 flex flex-col gap-2">
          <MapZoomControl />
          <MapFullscreenControl />
        </MapControlContainer>
      </Map>

      {/* Overlay cards */}
      <div className="absolute top-4 left-4 z-[1000] flex flex-col gap-3 max-w-xs">
        {/* Alert card */}
        {offlineRovers.length > 0 && (
          <Card className="bg-destructive/90 backdrop-blur-sm border-destructive">
            <CardContent className="p-3 flex items-center gap-2">
              <Warning className="h-5 w-5 text-destructive-foreground" />
              <div>
                <p className="text-sm font-medium text-destructive-foreground">
                  {offlineRovers.length} rover{offlineRovers.length > 1 ? "s" : ""} offline
                </p>
                <p className="text-xs text-destructive-foreground/80">
                  {offlineRovers.map((r) => r.name || r.id).join(", ")}
                </p>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Base Station status */}
        <Link to="/base-station">
          <Card className="bg-background/90 backdrop-blur-sm hover:bg-background transition-colors">
            <CardHeader className="p-3 pb-2">
              <CardTitle className="text-sm flex items-center justify-between">
                <span className="flex items-center gap-2">
                  <CellTower className="h-4 w-4" />
                  Base Station
                </span>
                <Badge variant={gpsOk ? "default" : "secondary"} className="text-[10px]">
                  {gpsOk ? "OK" : "No Fix"}
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className="p-3 pt-0">
              {gpsStatus ? (
                <div className="text-xs text-muted-foreground space-y-0.5">
                  <p>{gpsStatus.satellites} satellites</p>
                  <p className="font-mono">
                    {(gpsStatus.fixQuality ?? "no_fix").replace("_", " ").toUpperCase()}
                  </p>
                </div>
              ) : (
                <p className="text-xs text-muted-foreground">Not connected</p>
              )}
            </CardContent>
          </Card>
        </Link>

        {/* Fleet status */}
        <Link to="/fleet">
          <Card className="bg-background/90 backdrop-blur-sm hover:bg-background transition-colors">
            <CardHeader className="p-3 pb-2">
              <CardTitle className="text-sm flex items-center justify-between">
                <span className="flex items-center gap-2">
                  <Robot className="h-4 w-4" />
                  Fleet
                </span>
                <span className="text-xs">
                  <span className="text-green-500">{onlineRovers.length}</span>
                  <span className="text-muted-foreground">/{rovers.length}</span>
                </span>
              </CardTitle>
            </CardHeader>
            {onlineRovers.length > 0 && (
              <CardContent className="p-3 pt-0 space-y-2">
                {onlineRovers.slice(0, 3).map((rover) => (
                  <div
                    key={rover.id}
                    className="flex items-center justify-between text-xs"
                  >
                    <span className="text-foreground">{rover.name || rover.id}</span>
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <BatteryHigh className="h-3 w-3" />
                        {rover.batteryVoltage.toFixed(1)}V
                      </span>
                      {rover.mode === 2 && (
                        <GameController className="h-3 w-3 text-primary" />
                      )}
                    </div>
                  </div>
                ))}
                {onlineRovers.length > 3 && (
                  <p className="text-xs text-muted-foreground">
                    +{onlineRovers.length - 3} more
                  </p>
                )}
              </CardContent>
            )}
          </Card>
        </Link>
      </div>

      {/* Quick actions */}
      <div className="absolute bottom-4 left-4 z-[1000]">
        <Card className="bg-background/90 backdrop-blur-sm">
          <CardContent className="p-2 flex gap-2">
            <Link
              to="/dispatch"
              className="px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors flex items-center gap-1"
            >
              Dispatch
              <ArrowRight className="h-3 w-3" />
            </Link>
            <Link
              to="/fleet"
              className="px-3 py-1.5 text-xs font-medium bg-secondary text-secondary-foreground rounded hover:bg-secondary/80 transition-colors"
            >
              Fleet
            </Link>
          </CardContent>
        </Card>
      </div>

      {/* No position fallback */}
      {!hasDepotPosition && (
        <div className="absolute inset-0 flex items-center justify-center z-[1000] pointer-events-none">
          <Card className="bg-background/95 backdrop-blur-sm pointer-events-auto">
            <CardContent className="p-6 text-center">
              <CellTower className="h-12 w-12 mx-auto mb-3 text-muted-foreground" />
              <h3 className="font-medium mb-1">No GPS Position</h3>
              <p className="text-sm text-muted-foreground mb-3">
                Connect a GPS module to see the depot location
              </p>
              <Link
                to="/base-station"
                className="text-sm text-primary hover:underline"
              >
                Configure Base Station →
              </Link>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
