import { useConsoleStore } from "@/store";
import { CellTower, Broadcast, MapPin as MapPinIcon, Heartbeat, Planet, Crosshair, CellSignalFull, CellSignalMedium, CellSignalLow, CellSignalSlash } from "@phosphor-icons/react";
import type { GpsStatus, SatelliteInfo, Constellation } from "@/lib/types";
import {
  Map,
  MapTileLayer,
  MapMarker,
  MapZoomControl,
  MapFullscreenControl,
  MapControlContainer,
} from "@/components/ui/map";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
import { Area, AreaChart, XAxis, YAxis, Bar, BarChart, Cell } from "recharts";

const defaultStatus: GpsStatus = {
  connected: false,
  mode: "unknown",
  fixQuality: "no_fix",
  satellites: 0,
  lastUpdate: 0,
};

const fixQualityConfig: Record<string, { label: string; variant: "default" | "secondary" | "destructive" | "outline" }> = {
  no_fix: { label: "No Fix", variant: "destructive" },
  gps: { label: "GPS", variant: "default" },
  dgps: { label: "DGPS", variant: "default" },
  pps: { label: "PPS", variant: "default" },
  rtk_float: { label: "RTK Float", variant: "secondary" },
  rtk_fixed: { label: "RTK Fixed", variant: "default" },
  estimated: { label: "Estimated", variant: "outline" },
  manual: { label: "Manual", variant: "outline" },
  simulation: { label: "Simulation", variant: "outline" },
};

const constellationColors: Record<Constellation, string> = {
  gps: "hsl(var(--chart-1))",
  glonass: "hsl(var(--chart-2))",
  galileo: "hsl(var(--chart-3))",
  beidou: "hsl(var(--chart-4))",
  qzss: "hsl(var(--chart-5))",
  unknown: "hsl(var(--muted-foreground))",
};

const constellationLabels: Record<Constellation, string> = {
  gps: "GPS",
  glonass: "GLONASS",
  galileo: "Galileo",
  beidou: "BeiDou",
  qzss: "QZSS",
  unknown: "Unknown",
};

const hdopChartConfig: ChartConfig = {
  hdop: {
    label: "HDOP",
    color: "hsl(var(--chart-1))",
  },
  satellites: {
    label: "Satellites",
    color: "hsl(var(--chart-2))",
  },
};

function SignalIcon({ snr }: { snr?: number }) {
  if (!snr || snr === 0) return <CellSignalSlash className="h-4 w-4 text-muted-foreground" />;
  if (snr >= 40) return <CellSignalFull className="h-4 w-4 text-green-500" />;
  if (snr >= 25) return <CellSignalMedium className="h-4 w-4 text-yellow-500" />;
  return <CellSignalLow className="h-4 w-4 text-orange-500" />;
}

export function BaseStationView() {
  const { gpsStatus } = useConsoleStore();
  const status: GpsStatus = gpsStatus ?? defaultStatus;

  const fixConfig = fixQualityConfig[status.fixQuality ?? "no_fix"] ?? fixQualityConfig.no_fix;

  // Format history data for chart
  const historyData = (status.history ?? []).map((point) => ({
    time: new Date(point.timestamp).toLocaleTimeString([], { minute: "2-digit", second: "2-digit" }),
    hdop: point.hdop ?? 0,
    satellites: point.satellites,
  }));

  // Group satellites by constellation
  const satellitesByConstellation = (status.satelliteInfo ?? []).reduce((acc, sat) => {
    const key = sat.constellation;
    if (!acc[key]) acc[key] = [];
    acc[key].push(sat);
    return acc;
  }, {} as Record<Constellation, SatelliteInfo[]>);

  // Prepare satellite SNR data for bar chart
  const satelliteSnrData = (status.satelliteInfo ?? [])
    .filter((sat) => sat.snr != null && sat.snr > 0)
    .slice(0, 16) // Show top 16 by SNR
    .map((sat) => ({
      prn: `${constellationLabels[sat.constellation].charAt(0)}${sat.prn}`,
      snr: sat.snr ?? 0,
      constellation: sat.constellation,
      used: sat.used,
    }));

  const snrChartConfig: ChartConfig = {
    snr: {
      label: "SNR (dB-Hz)",
      color: "hsl(var(--chart-1))",
    },
  };

  return (
    <div className="min-h-full p-6">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold text-foreground">Base Station</h1>
          <p className="text-muted-foreground">
            RTK GPS base station status and configuration
          </p>
        </div>

        {/* Connection Status Card */}
        <Card>
          <CardHeader className="pb-4">
            <div className="flex items-center gap-3">
              <div
                className={`h-12 w-12 rounded-lg flex items-center justify-center ${
                  status.connected
                    ? "bg-green-500/20 dark:bg-green-500/10"
                    : "bg-muted"
                }`}
              >
                <CellTower
                  className={`h-6 w-6 ${
                    status.connected ? "text-green-500" : "text-muted-foreground"
                  }`}
                  weight="duotone"
                />
              </div>
              <div className="flex-1">
                <CardTitle className="flex items-center gap-2">
                  GPS Module
                  {status.connected && (
                    <Badge variant={fixConfig.variant} className="font-mono text-xs">
                      {fixConfig.label}
                    </Badge>
                  )}
                </CardTitle>
                <CardDescription>
                  {status.connected ? `Connected · ${status.mode} mode` : "Not connected"}
                </CardDescription>
              </div>
            </div>
          </CardHeader>

          {status.connected && (
            <CardContent className="pt-0">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6 pt-4 border-t border-border">
                <div className="space-y-1">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                    Satellites
                  </p>
                  <div className="flex items-center gap-2">
                    <Planet className="h-4 w-4 text-muted-foreground" />
                    <p className="font-mono text-lg font-semibold text-foreground">
                      {status.satellites}
                    </p>
                  </div>
                </div>
                {status.hdop != null && (
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      HDOP
                    </p>
                    <div className="flex items-center gap-2">
                      <Crosshair className="h-4 w-4 text-muted-foreground" />
                      <p className="font-mono text-lg font-semibold text-foreground">
                        {status.hdop.toFixed(2)}
                      </p>
                    </div>
                  </div>
                )}
                {status.pdop != null && (
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      PDOP
                    </p>
                    <p className="font-mono text-lg font-semibold text-foreground">
                      {status.pdop.toFixed(2)}
                    </p>
                  </div>
                )}
                {status.vdop != null && (
                  <div className="space-y-1">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      VDOP
                    </p>
                    <p className="font-mono text-lg font-semibold text-foreground">
                      {status.vdop.toFixed(2)}
                    </p>
                  </div>
                )}
              </div>
            </CardContent>
          )}
        </Card>

        {/* Accuracy Chart Card */}
        {status.connected && historyData.length > 5 && (
          <Card>
            <CardHeader>
              <CardTitle>Accuracy Over Time</CardTitle>
              <CardDescription>HDOP and satellite count (last 60 seconds)</CardDescription>
            </CardHeader>
            <CardContent>
              <ChartContainer config={hdopChartConfig} className="h-48 w-full">
                <AreaChart data={historyData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                  <defs>
                    <linearGradient id="hdopGradient" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="hsl(var(--chart-1))" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="hsl(var(--chart-1))" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <XAxis
                    dataKey="time"
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    tick={{ fontSize: 10 }}
                  />
                  <YAxis
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    tick={{ fontSize: 10 }}
                    domain={[0, "auto"]}
                  />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Area
                    type="monotone"
                    dataKey="hdop"
                    stroke="hsl(var(--chart-1))"
                    fill="url(#hdopGradient)"
                    strokeWidth={2}
                  />
                </AreaChart>
              </ChartContainer>
            </CardContent>
          </Card>
        )}

        {/* Satellite Signal Strength Card */}
        {status.connected && satelliteSnrData.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Satellite Signal Strength</CardTitle>
              <CardDescription>
                SNR in dB-Hz · Highlighted bars indicate satellites used in fix
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ChartContainer config={snrChartConfig} className="h-48 w-full">
                <BarChart data={satelliteSnrData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                  <XAxis
                    dataKey="prn"
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    tick={{ fontSize: 10 }}
                  />
                  <YAxis
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    tick={{ fontSize: 10 }}
                    domain={[0, 60]}
                  />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Bar dataKey="snr" radius={[4, 4, 0, 0]}>
                    {satelliteSnrData.map((entry, index) => (
                      <Cell
                        key={`cell-${index}`}
                        fill={entry.used ? constellationColors[entry.constellation] : "hsl(var(--muted))"}
                        opacity={entry.used ? 1 : 0.5}
                      />
                    ))}
                  </Bar>
                </BarChart>
              </ChartContainer>

              {/* Constellation Legend */}
              <div className="flex flex-wrap gap-4 mt-4 pt-4 border-t border-border">
                {Object.entries(satellitesByConstellation).map(([constellation, sats]) => (
                  <div key={constellation} className="flex items-center gap-2">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: constellationColors[constellation as Constellation] }}
                    />
                    <span className="text-xs text-muted-foreground">
                      {constellationLabels[constellation as Constellation]} ({sats.length})
                    </span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Satellite Details Card */}
        {status.connected && status.satelliteInfo && status.satelliteInfo.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Satellites in View</CardTitle>
              <CardDescription>
                {status.satelliteInfo.filter((s) => s.used).length} used in fix · {status.satelliteInfo.length} total visible
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
                {status.satelliteInfo.slice(0, 16).map((sat) => (
                  <div
                    key={`${sat.constellation}-${sat.prn}`}
                    className={`p-3 rounded-lg border ${
                      sat.used
                        ? "bg-primary/5 border-primary/20"
                        : "bg-muted/30 border-border"
                    }`}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-mono text-sm font-medium">
                        {constellationLabels[sat.constellation].charAt(0)}{sat.prn}
                      </span>
                      <SignalIcon snr={sat.snr} />
                    </div>
                    <div className="grid grid-cols-2 gap-1 text-xs text-muted-foreground">
                      <div>
                        <span className="block">SNR</span>
                        <span className="font-mono text-foreground">{sat.snr ?? "—"}</span>
                      </div>
                      <div>
                        <span className="block">Elev</span>
                        <span className="font-mono text-foreground">{sat.elevation ?? "—"}°</span>
                      </div>
                    </div>
                    {sat.used && (
                      <Badge variant="secondary" className="mt-2 text-[10px] h-5">
                        In Use
                      </Badge>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Position & Map Card */}
        {status.connected && status.latitude != null && status.longitude != null && (
          <Card className="overflow-hidden">
            <CardHeader>
              <div className="flex items-center gap-2">
                <MapPinIcon className="h-5 w-5 text-foreground" weight="duotone" />
                <CardTitle>Position</CardTitle>
              </div>
            </CardHeader>

            <CardContent className="p-0">
              {/* Map */}
              <div className="h-80 relative">
                <Map
                  center={[status.latitude, status.longitude]}
                  zoom={18}
                  className="h-full w-full"
                >
                  <MapTileLayer />
                  <MapMarker position={[status.latitude, status.longitude]}>
                    <div className="flex items-center justify-center w-8 h-8 bg-green-500 rounded-full border-3 border-white shadow-lg ring-2 ring-green-500/30">
                      <CellTower className="w-4 h-4 text-white" weight="bold" />
                    </div>
                  </MapMarker>
                  <MapControlContainer className="top-3 right-3 flex flex-col gap-2">
                    <MapZoomControl />
                    <MapFullscreenControl />
                  </MapControlContainer>
                </Map>
              </div>

              {/* Coordinates */}
              <div className="grid grid-cols-3 divide-x divide-border border-t border-border">
                <div className="p-4 text-center">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1">
                    Latitude
                  </p>
                  <p className="font-mono text-sm text-foreground">
                    {status.latitude?.toFixed(8)}°
                  </p>
                </div>
                <div className="p-4 text-center">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1">
                    Longitude
                  </p>
                  <p className="font-mono text-sm text-foreground">
                    {status.longitude?.toFixed(8)}°
                  </p>
                </div>
                <div className="p-4 text-center">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1">
                    Altitude
                  </p>
                  <p className="font-mono text-sm text-foreground">
                    {status.altitude?.toFixed(2)} m
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Survey-In Status Card */}
        {status.surveyIn && (
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <Heartbeat className="h-5 w-5 text-foreground" weight="duotone" />
                <CardTitle>Survey-In Progress</CardTitle>
                <Badge variant={status.surveyIn.valid ? "default" : "secondary"}>
                  {status.surveyIn.valid ? "Valid" : "In Progress"}
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-1">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                    Duration
                  </p>
                  <p className="font-mono text-lg font-semibold text-foreground">
                    {Math.floor(status.surveyIn.duration / 60)}m {status.surveyIn.duration % 60}s
                  </p>
                </div>
                <div className="space-y-1">
                  <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                    Accuracy
                  </p>
                  <p className="font-mono text-lg font-semibold text-foreground">
                    {status.surveyIn.accuracy?.toFixed(3) ?? "—"} m
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* RTCM Messages Card */}
        {status.rtcmMessages && status.rtcmMessages.length > 0 && (
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <Broadcast className="h-5 w-5 text-foreground" weight="duotone" />
                <CardTitle>RTCM Output</CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {status.rtcmMessages.map((msg) => (
                  <div key={msg.type} className="bg-muted/50 rounded-lg p-3">
                    <p className="text-xs text-muted-foreground mb-1">MSG {msg.type}</p>
                    <p className="font-mono text-lg font-semibold text-foreground">{msg.count}</p>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Not Connected State */}
        {!status.connected && (
          <Card className="border-dashed">
            <CardContent className="py-12 text-center">
              <CellTower className="h-16 w-16 mx-auto mb-4 text-muted-foreground/50" weight="duotone" />
              <h3 className="font-semibold text-foreground mb-2">GPS Module Not Connected</h3>
              <p className="text-sm text-muted-foreground max-w-md mx-auto mb-4">
                Connect a ZED-F9P GPS module via USB to the depot server.
                The device should appear as /dev/ttyUSB0 or /dev/ttyACM0.
              </p>
              <code className="text-xs bg-muted px-3 py-2 rounded-md font-mono text-muted-foreground">
                docker compose --profile rtk up -d
              </code>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
