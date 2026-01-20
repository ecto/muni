import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { CellTower, Robot } from "@phosphor-icons/react";
import { ModeLabels, type Mode } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";
import {
  Map,
  MapTileLayer,
  MapMarker,
  MapZoomControl,
  MapFullscreenControl,
  MapControlContainer,
} from "@/components/ui/map";
import { Card, CardContent } from "@/components/ui/card";

// Default center (San Francisco) when no GPS data
const DEFAULT_CENTER: [number, number] = [37.7749, -122.4194];
const DEFAULT_ZOOM = 12;

export function DashboardView() {
  const { rovers, gpsStatus } = useConsoleStore();

  const onlineRovers = rovers.filter((r) => r.online);

  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

  // Use depot GPS position as map center when available
  const hasDepotPosition = gpsStatus?.latitude != null && gpsStatus?.longitude != null;
  const mapCenter: [number, number] = hasDepotPosition
    ? [gpsStatus.latitude!, gpsStatus.longitude!]
    : DEFAULT_CENTER;
  const mapZoom = hasDepotPosition ? 18 : DEFAULT_ZOOM;

  return (
    <div className="relative min-h-full">
      {/* Full-page map */}
      <Map center={mapCenter} zoom={mapZoom} className="!z-0 absolute inset-0">
        <MapTileLayer />

        {/* Depot/Base Station marker */}
        {hasDepotPosition && (
          <MapMarker
            position={[gpsStatus.latitude!, gpsStatus.longitude!]}
            iconAnchor={[20, 20]}
            icon={
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
            }
          />
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
            <MapMarker
              key={rover.id}
              position={[roverLat, roverLon]}
              iconAnchor={[16, 16]}
              icon={
                <Link to={`/fleet/${rover.id}`} className="relative group">
                  <div className="flex items-center justify-center w-8 h-8 bg-primary rounded-full border-2 border-white shadow-lg ring-2 ring-primary/30">
                    <Robot className="w-4 h-4 text-primary-foreground" weight="bold" />
                  </div>
                  {/* Tooltip */}
                  <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-background border border-border rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="font-medium">{rover.name || rover.id}</span>
                    <br />
                    <span className="text-muted-foreground">
                      {ModeLabels[rover.mode as Mode]} · {getBatteryPercent(rover.batteryVoltage).toFixed(0)}%
                    </span>
                  </div>
                </Link>
              }
            />
          );
        })}

        <MapControlContainer className="top-3 right-3 flex flex-col gap-2">
          <MapZoomControl />
          <MapFullscreenControl />
        </MapControlContainer>
      </Map>

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
