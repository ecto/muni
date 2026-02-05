import { useMemo, useRef } from "react";
import { useConsoleStore } from "@/store";
import { CellTower, Broadcast, Heartbeat, Planet, Crosshair, CellSignalFull, CellSignalMedium, CellSignalLow, CellSignalSlash } from "@phosphor-icons/react";
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

// Direct colors for chart legibility in both light and dark mode
const constellationColors: Record<Constellation, string> = {
  gps: "#f59e0b",      // Amber
  glonass: "#06b6d4",  // Cyan
  galileo: "#84cc16",  // Lime
  beidou: "#a855f7",   // Purple
  qzss: "#f43f5e",     // Rose
  unknown: "#71717a",  // Zinc
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
    color: "#f59e0b",  // Amber - matches GPS constellation
  },
  satellites: {
    label: "Satellites",
    color: "#06b6d4",  // Cyan
  },
};

function SignalIcon({ snr }: { snr?: number }) {
  if (!snr || snr === 0) return <CellSignalSlash className="h-4 w-4 text-muted-foreground" />;
  if (snr >= 40) return <CellSignalFull className="h-4 w-4 text-green-500" />;
  if (snr >= 25) return <CellSignalMedium className="h-4 w-4 text-yellow-500" />;
  return <CellSignalLow className="h-4 w-4 text-orange-500" />;
}

// Type for SNR chart data
type SnrDataPoint = {
  prn: string;
  snr: number;
  constellation: Constellation;
  used: boolean;
};

export function BaseStationView() {
  const gpsStatus = useConsoleStore((s) => s.gpsStatus);
  const status: GpsStatus = gpsStatus ?? defaultStatus;

  // Refs to preserve last good data (prevents flashing when data temporarily empty)
  const cachedSatellitesRef = useRef<SatelliteInfo[]>([]);
  const cachedConstellationsRef = useRef<Partial<Record<Constellation, SatelliteInfo[]>>>({});
  const cachedSnrDataRef = useRef<SnrDataPoint[]>([]);

  const fixConfig = fixQualityConfig[status.fixQuality ?? "no_fix"] ?? fixQualityConfig.no_fix;

  // Format history data for chart - memoized
  const historyData = useMemo(() =>
    (status.history ?? []).map((point) => ({
      time: new Date(point.timestamp).toLocaleTimeString([], { minute: "2-digit", second: "2-digit" }),
      hdop: point.hdop ?? 0,
      satellites: point.satellites,
    })),
    [status.history]
  );

  // Deduplicate satellites by constellation+prn - pure computation
  const computedSatellites = useMemo(() => {
    return (status.satelliteInfo ?? []).reduce((acc, sat) => {
      const key = `${sat.constellation}-${sat.prn}`;
      if (!acc.seen.has(key)) {
        acc.seen.add(key);
        acc.list.push(sat);
      }
      return acc;
    }, { seen: new Set<string>(), list: [] as SatelliteInfo[] }).list;
  }, [status.satelliteInfo]);

  // Cache last good data in refs (avoids setState-in-useEffect cascade)
  if (computedSatellites.length > 0) {
    cachedSatellitesRef.current = computedSatellites;
  }
  const uniqueSatellites = computedSatellites.length > 0 ? computedSatellites : cachedSatellitesRef.current;

  // Group satellites by constellation - pure computation
  const computedConstellations = useMemo(() => {
    return uniqueSatellites.reduce((acc, sat) => {
      const key = sat.constellation;
      if (!acc[key]) acc[key] = [];
      acc[key]!.push(sat);
      return acc;
    }, {} as Partial<Record<Constellation, SatelliteInfo[]>>);
  }, [uniqueSatellites]);

  if (Object.keys(computedConstellations).length > 0) {
    cachedConstellationsRef.current = computedConstellations;
  }
  const satellitesByConstellation = Object.keys(computedConstellations).length > 0
    ? computedConstellations
    : cachedConstellationsRef.current;

  // Prepare satellite SNR data for bar chart - pure computation
  const computedSnrData = useMemo(() => {
    return uniqueSatellites
      .filter((sat) => sat.snr != null && sat.snr > 0)
      .slice(0, 16)
      .map((sat) => ({
        prn: `${constellationLabels[sat.constellation].charAt(0)}${sat.prn}`,
        snr: sat.snr ?? 0,
        constellation: sat.constellation,
        used: sat.used,
      }));
  }, [uniqueSatellites]);

  if (computedSnrData.length > 0) {
    cachedSnrDataRef.current = computedSnrData;
  }
  const satelliteSnrData = computedSnrData.length > 0 ? computedSnrData : cachedSnrDataRef.current;

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

        {/* Connection Status Card with Map */}
        <Card className="overflow-hidden">
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
            <CardContent className="pt-0 space-y-4">
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

              {/* Map with position */}
              {status.latitude != null && status.longitude != null && (
                <div className="relative h-64 rounded-lg overflow-hidden border border-border">
                  <Map
                    center={[status.latitude, status.longitude]}
                    zoom={18}
                    className="h-full w-full"
                  >
                    <MapTileLayer />
                    <MapMarker
                      position={[status.latitude, status.longitude]}
                      iconAnchor={[16, 16]}
                      icon={
                        <div className="flex items-center justify-center w-8 h-8 bg-green-500 rounded-full border-2 border-white shadow-lg ring-2 ring-green-500/30">
                          <CellTower className="w-4 h-4 text-white" weight="bold" />
                        </div>
                      }
                    />
                    <MapControlContainer className="top-2 right-2 flex flex-col gap-1">
                      <MapZoomControl />
                      <MapFullscreenControl />
                    </MapControlContainer>
                  </Map>
                  {/* Coordinates overlay */}
                  <div className="absolute bottom-2 left-2 bg-background/90 backdrop-blur-sm rounded px-2 py-1 text-xs font-mono border border-border">
                    <span className="text-muted-foreground">Lat:</span> {status.latitude.toFixed(7)}°{" "}
                    <span className="text-muted-foreground ml-2">Lng:</span> {status.longitude.toFixed(7)}°
                    {status.altitude != null && (
                      <span className="ml-2"><span className="text-muted-foreground">Alt:</span> {status.altitude.toFixed(1)}m</span>
                    )}
                  </div>
                </div>
              )}
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
                      <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.6} />
                      <stop offset="95%" stopColor="#f59e0b" stopOpacity={0.1} />
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
                    stroke="#f59e0b"
                    fill="url(#hdopGradient)"
                    strokeWidth={2}
                    isAnimationActive={false}
                  />
                </AreaChart>
              </ChartContainer>
            </CardContent>
          </Card>
        )}

        {/* Satellite Signal Strength Card */}
        {status.connected && (
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
                  <Bar dataKey="snr" radius={[4, 4, 0, 0]} isAnimationActive={false}>
                    {satelliteSnrData.map((entry, index) => (
                      <Cell
                        key={`cell-${index}`}
                        fill={entry.used ? constellationColors[entry.constellation] : "#52525b"}
                        opacity={entry.used ? 1 : 0.5}
                      />
                    ))}
                  </Bar>
                </BarChart>
              </ChartContainer>

              {/* Constellation Legend */}
              {Object.keys(satellitesByConstellation).length > 0 && (
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
              )}
            </CardContent>
          </Card>
        )}

        {/* Satellite Details Card */}
        {status.connected && uniqueSatellites.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Satellites in View</CardTitle>
              <CardDescription>
                {uniqueSatellites.filter((s) => s.used).length} used in fix · {uniqueSatellites.length} total visible
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
                {uniqueSatellites.slice(0, 16).map((sat) => (
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
