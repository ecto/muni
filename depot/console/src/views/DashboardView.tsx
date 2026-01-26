import { useMemo } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { useServiceHealth } from "@/hooks/useServiceHealth";
import { useDispatch } from "@/hooks/useDispatch";
import { useWeather } from "@/hooks/useWeather";
import {
  CellTower,
  Robot,
  CheckCircle,
  Warning,
  WarningCircle,
  NavigationArrow,
  Heartbeat,
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  Clock,
  CloudSnow,
} from "@phosphor-icons/react";
import { Mode, ModeLabels, AlertSeverity, type RoverInfo, type Mode as ModeType } from "@/lib/types";
import { WeatherCard } from "@/components/dashboard/WeatherCard";
import { getBatteryPercent } from "@/lib/utils";
import {
  Map,
  MapTileLayer,
  MapMarker,
  MapZoomControl,
  MapFullscreenControl,
  MapControlContainer,
} from "@/components/ui/map";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";

const DEFAULT_CENTER: [number, number] = [37.7749, -122.4194];
const DEFAULT_ZOOM = 12;

// --- KPI Cards ---

function KpiCard({
  label,
  value,
  icon,
  variant = "default",
}: {
  label: string;
  value: string;
  icon: React.ReactNode;
  variant?: "default" | "success" | "warning" | "danger";
}) {
  const borderColor = {
    default: "border-border",
    success: "border-green-500/30",
    warning: "border-yellow-500/30",
    danger: "border-red-500/30",
  }[variant];

  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 bg-card border ${borderColor} rounded-lg`}
    >
      <div className="text-muted-foreground">{icon}</div>
      <div>
        <div className="text-lg font-semibold leading-none">{value}</div>
        <div className="text-xs text-muted-foreground mt-0.5">{label}</div>
      </div>
    </div>
  );
}

// --- Fleet Status Card ---

function FleetStatusCard({ rover }: { rover: RoverInfo }) {
  const isOnline = rover.online;
  const displayMode =
    rover.mode === Mode.Disabled ? Mode.Idle : rover.mode;
  const batteryPercent = getBatteryPercent(rover.batteryVoltage);

  const BatteryIcon =
    rover.batteryVoltage < 42
      ? BatteryWarning
      : rover.batteryVoltage < 45
        ? BatteryLow
        : BatteryFull;
  const batteryColor =
    rover.batteryVoltage < 42
      ? "text-red-500"
      : rover.batteryVoltage < 45
        ? "text-orange-500"
        : "text-green-500";

  const getModeVariant = (
    mode: ModeType
  ): "default" | "secondary" | "destructive" | "outline" => {
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
  };

  return (
    <Link
      to={`/fleet/${rover.id}`}
      className={`block p-3 border border-border rounded-lg hover:bg-muted/50 transition-colors ${
        !isOnline ? "opacity-50" : ""
      }`}
    >
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-2">
          <span
            className={`h-2 w-2 rounded-full flex-shrink-0 ${
              isOnline ? "bg-green-500" : "bg-red-500"
            }`}
          />
          <span className="text-sm font-medium">
            {rover.name || rover.id}
          </span>
        </div>
        {isOnline && (
          <Badge
            variant={getModeVariant(displayMode)}
            className="text-[10px] h-4 px-1.5"
          >
            {ModeLabels[displayMode]}
          </Badge>
        )}
      </div>
      {isOnline ? (
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <BatteryIcon className={`h-3 w-3 ${batteryColor}`} weight="fill" />
            {batteryPercent.toFixed(0)}%
          </span>
          <span>{rover.batteryVoltage.toFixed(1)}V</span>
        </div>
      ) : (
        <div className="text-xs text-muted-foreground/50">
          {rover.lastSeen > 0
            ? `Last seen ${timeAgo(rover.lastSeen)}`
            : "Never seen"}
        </div>
      )}
    </Link>
  );
}

// --- Activity Feed ---

function ActivityFeed() {
  const alerts = useConsoleStore((s) => s.alerts);
  const recent = alerts.slice(0, 8);

  if (recent.length === 0) {
    return (
      <div className="text-xs text-muted-foreground/50 py-4 text-center">
        No recent activity
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {recent.map((alert) => {
        const Icon =
          alert.severity === AlertSeverity.Critical
            ? WarningCircle
            : alert.severity === AlertSeverity.Warning
              ? Warning
              : CheckCircle;
        const color =
          alert.severity === AlertSeverity.Critical
            ? "text-red-500"
            : alert.severity === AlertSeverity.Warning
              ? "text-yellow-500"
              : "text-muted-foreground";

        return (
          <div
            key={alert.id}
            className="flex items-start gap-2 text-xs py-1"
          >
            <Icon
              className={`h-3.5 w-3.5 mt-0.5 flex-shrink-0 ${color}`}
              weight="fill"
            />
            <div className="flex-1 min-w-0">
              <span className="text-foreground">{alert.sourceId}</span>
              <span className="text-muted-foreground"> {alert.message}</span>
            </div>
            <span className="text-muted-foreground/50 flex-shrink-0">
              {timeAgo(new Date(alert.createdAt).getTime())}
            </span>
          </div>
        );
      })}
    </div>
  );
}

// --- Helpers ---

function timeAgo(ms: number): string {
  const secs = Math.floor((Date.now() - ms) / 1000);
  if (secs < 10) return "now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

// --- Main Dashboard ---

export function DashboardView() {
  const navigate = useNavigate();
  const rovers = useConsoleStore((s) => s.rovers);
  const gpsStatus = useConsoleStore((s) => s.gpsStatus);
  const alerts = useConsoleStore((s) => s.alerts);
  const { healthyCount, totalCount } = useServiceHealth();
  const { tasks } = useDispatch();
  const { weather } = useWeather();

  const onlineRovers = useMemo(
    () => rovers.filter((r) => r.online),
    [rovers]
  );

  const activeAlerts = useMemo(
    () => alerts.filter((a) => !a.clearedAt && !a.acknowledgedAt),
    [alerts]
  );
  const criticalAlerts = activeAlerts.filter(
    (a) => a.severity === AlertSeverity.Critical
  );
  const warningAlerts = activeAlerts.filter(
    (a) => a.severity === AlertSeverity.Warning
  );

  const activeTasks = useMemo(
    () => tasks.filter((t) => t.status === "active" || t.status === "assigned"),
    [tasks]
  );
  const completedTodayTasks = useMemo(() => {
    const todayStart = new Date();
    todayStart.setHours(0, 0, 0, 0);
    return tasks.filter(
      (t) =>
        t.status === "done" &&
        t.endedAt &&
        new Date(t.endedAt).getTime() > todayStart.getTime()
    );
  }, [tasks]);

  const gpsOk =
    gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";
  const hasDepotPosition =
    gpsStatus?.latitude != null && gpsStatus?.longitude != null;
  const mapCenter: [number, number] = hasDepotPosition
    ? [gpsStatus.latitude!, gpsStatus.longitude!]
    : DEFAULT_CENTER;
  const mapZoom = hasDepotPosition ? 18 : DEFAULT_ZOOM;

  const alertVariant =
    criticalAlerts.length > 0
      ? "danger"
      : warningAlerts.length > 0
        ? "warning"
        : "success";

  const servicesVariant =
    healthyCount === totalCount
      ? "success"
      : healthyCount > 0
        ? "warning"
        : "danger";

  return (
    <div className="min-h-full p-4 space-y-4">
      {/* KPI Row */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
        <KpiCard
          label="Fleet"
          value={`${onlineRovers.length}/${rovers.length}`}
          icon={<Robot className="h-5 w-5" />}
          variant={onlineRovers.length === rovers.length ? "success" : "warning"}
        />
        <KpiCard
          label="Active Tasks"
          value={String(activeTasks.length)}
          icon={<NavigationArrow className="h-5 w-5" />}
        />
        <KpiCard
          label="Completed Today"
          value={String(completedTodayTasks.length)}
          icon={<CheckCircle className="h-5 w-5" />}
          variant="success"
        />
        <KpiCard
          label="Alerts"
          value={
            criticalAlerts.length + warningAlerts.length > 0
              ? `${criticalAlerts.length + warningAlerts.length}`
              : "0"
          }
          icon={<Warning className="h-5 w-5" />}
          variant={alertVariant as "default" | "success" | "warning" | "danger"}
        />
        <KpiCard
          label="Services"
          value={`${healthyCount}/${totalCount}`}
          icon={<Heartbeat className="h-5 w-5" />}
          variant={servicesVariant}
        />
      </div>

      {/* Main Content: Map + Fleet Status */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Operations Map */}
        <div className="lg:col-span-2 border border-border rounded-lg overflow-hidden h-[400px] relative">
          <Map
            center={mapCenter}
            zoom={mapZoom}
            className="!z-0 absolute inset-0"
          >
            <MapTileLayer />

            {/* Depot marker */}
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
                      <CellTower
                        className="w-5 h-5 text-white"
                        weight="bold"
                      />
                    </div>
                    <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-background border border-border rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                      <span className="font-medium">Depot Base Station</span>
                      <br />
                      <span className="text-muted-foreground">
                        {gpsStatus.satellites} sats ·{" "}
                        {(gpsStatus.fixQuality ?? "no_fix").replace("_", " ")}
                      </span>
                    </div>
                  </div>
                }
              />
            )}

            {/* Online rover markers */}
            {onlineRovers.map((rover, idx) => {
              if (!hasDepotPosition) return null;
              const offset = 0.0001 * (idx + 1);
              const roverLat = gpsStatus.latitude! + offset;
              const roverLon = gpsStatus.longitude! + offset;

              return (
                <MapMarker
                  key={rover.id}
                  position={[roverLat, roverLon]}
                  iconAnchor={[16, 16]}
                  eventHandlers={{
                    click: () => navigate(`/fleet/${rover.id}`),
                  }}
                  icon={
                    <div className="relative group cursor-pointer">
                      <div className="flex items-center justify-center w-8 h-8 bg-primary rounded-full border-2 border-white shadow-lg ring-2 ring-primary/30">
                        <Robot
                          className="w-4 h-4 text-primary-foreground"
                          weight="bold"
                        />
                      </div>
                      <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-background border border-border rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                        <span className="font-medium">
                          {rover.name || rover.id}
                        </span>
                        <br />
                        <span className="text-muted-foreground">
                          {ModeLabels[rover.mode as ModeType]} ·{" "}
                          {getBatteryPercent(rover.batteryVoltage).toFixed(0)}%
                        </span>
                      </div>
                    </div>
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

        {/* Fleet Status Panel */}
        <div className="border border-border rounded-lg overflow-hidden">
          <div className="px-4 py-2 border-b border-border bg-muted/30">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                Fleet Status
              </span>
              <Link
                to="/fleet"
                className="text-xs text-primary hover:underline"
              >
                View all
              </Link>
            </div>
          </div>
          <div className="p-2 space-y-2 max-h-[356px] overflow-y-auto">
            {rovers.length === 0 ? (
              <div className="text-xs text-muted-foreground text-center py-8">
                No rovers registered
              </div>
            ) : (
              rovers.map((rover) => (
                <FleetStatusCard key={rover.id} rover={rover} />
              ))
            )}
          </div>
        </div>
      </div>

      {/* Bottom Row: Activity Feed, Base Station, Weather */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Recent Activity */}
        <div className="border border-border rounded-lg overflow-hidden">
          <div className="px-4 py-2 border-b border-border bg-muted/30">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
                <Clock className="h-3.5 w-3.5" />
                Recent Activity
              </span>
              <Link
                to="/alerts"
                className="text-xs text-primary hover:underline"
              >
                View all
              </Link>
            </div>
          </div>
          <div className="p-3 max-h-[200px] overflow-y-auto">
            <ActivityFeed />
          </div>
        </div>

        {/* Base Station */}
        <div className="border border-border rounded-lg overflow-hidden">
          <div className="px-4 py-2 border-b border-border bg-muted/30">
            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
              <CellTower className="h-3.5 w-3.5" />
              Base Station
            </span>
          </div>
          <div className="p-3">
            {gpsStatus?.connected ? (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">
                    {(gpsStatus.fixQuality ?? "no_fix").replace("_", " ")}
                  </span>
                  <span
                    className={`h-2 w-2 rounded-full ${
                      gpsOk ? "bg-green-500" : "bg-yellow-500"
                    }`}
                  />
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
                  <div>
                    <span className="block text-foreground font-medium">
                      {gpsStatus.satellites}
                    </span>
                    Satellites
                  </div>
                  <div>
                    <span className="block text-foreground font-medium">
                      {gpsStatus.hdop?.toFixed(1) ?? "—"}
                    </span>
                    HDOP
                  </div>
                  {gpsStatus.clients != null && (
                    <div>
                      <span className="block text-foreground font-medium">
                        {gpsStatus.clients}
                      </span>
                      RTK Clients
                    </div>
                  )}
                  {gpsStatus.altitude != null && (
                    <div>
                      <span className="block text-foreground font-medium">
                        {gpsStatus.altitude.toFixed(1)}m
                      </span>
                      Altitude
                    </div>
                  )}
                </div>
                <Link
                  to="/base-station"
                  className="text-xs text-primary hover:underline block"
                >
                  Details →
                </Link>
              </div>
            ) : (
              <div className="text-xs text-muted-foreground text-center py-4">
                <CellTower className="h-8 w-8 mx-auto mb-2 opacity-30" />
                GPS not connected
              </div>
            )}
          </div>
        </div>

        {/* Weather */}
        <div className="border border-border rounded-lg overflow-hidden">
          <div className="px-4 py-2 border-b border-border bg-muted/30">
            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
              <CloudSnow className="h-3.5 w-3.5" />
              Weather
            </span>
          </div>
          <WeatherCard weather={weather} />
        </div>
      </div>
    </div>
  );
}
