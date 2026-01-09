import { useConsoleStore } from "@/store";
import { CellTower, Broadcast, MapPin, Heartbeat } from "@phosphor-icons/react";
import type { GpsStatus } from "@/lib/types";

const defaultStatus: GpsStatus = {
  connected: false,
  mode: "unknown",
  fixQuality: "no_fix",
  satellites: 0,
  lastUpdate: 0,
};

export function BaseStationView() {
  const { gpsStatus } = useConsoleStore();

  const status: GpsStatus = gpsStatus ?? defaultStatus;

  const fixQualityColors: Record<string, string> = {
    no_fix: "text-red-500",
    gps: "text-yellow-500",
    dgps: "text-yellow-500",
    pps: "text-yellow-500",
    rtk_float: "text-blue-500",
    rtk_fixed: "text-green-500",
    estimated: "text-yellow-500",
    manual: "text-blue-500",
    simulation: "text-purple-500",
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold">Base Station</h1>
          <p className="text-muted-foreground">
            RTK GPS base station status and configuration
          </p>
        </div>

        {/* Connection Status */}
        <div className="bg-card border border-border p-6">
          <div className="flex items-center gap-3 mb-4">
            <div
              className={`h-10 w-10 flex items-center justify-center ${
                status.connected ? "bg-green-500/20" : "bg-muted"
              }`}
            >
              <CellTower
                className={`h-5 w-5 ${
                  status.connected ? "text-green-500" : "text-muted-foreground"
                }`}
              />
            </div>
            <div>
              <h2 className="font-medium text-foreground">GPS Module</h2>
              <p className="text-sm text-muted-foreground">
                {status.connected ? "Connected" : "Not connected"}
                {status.connected && ` · ${status.mode} mode`}
              </p>
            </div>
          </div>

          {status.connected && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 pt-4 border-t border-border">
              <div>
                <p className="text-xs text-muted-foreground mb-1">Fix Quality</p>
                <p
                  className={`font-mono text-sm ${
                    fixQualityColors[status.fixQuality ?? "no_fix"] ?? "text-muted-foreground"
                  }`}
                >
                  {(status.fixQuality ?? "no_fix").replace("_", " ").toUpperCase()}
                </p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">Satellites</p>
                <p className="font-mono text-sm">{status.satellites}</p>
              </div>
              {status.hdop != null && (
                <div>
                  <p className="text-xs text-muted-foreground mb-1">HDOP</p>
                  <p className="font-mono text-sm">{status.hdop.toFixed(2)}</p>
                </div>
              )}
              {status.clients != null && (
                <div>
                  <p className="text-xs text-muted-foreground mb-1">NTRIP Clients</p>
                  <p className="font-mono text-sm">{status.clients}</p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Position */}
        {status.connected && status.latitude != null && (
          <div className="bg-card border border-border p-6">
            <div className="flex items-center gap-2 mb-4">
              <MapPin className="h-4 w-4 text-muted-foreground" />
              <h2 className="font-medium">Position</h2>
            </div>
            <div className="grid grid-cols-3 gap-4 font-mono text-sm">
              <div>
                <p className="text-xs text-muted-foreground mb-1">Latitude</p>
                <p>{status.latitude?.toFixed(8)}°</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">Longitude</p>
                <p>{status.longitude?.toFixed(8)}°</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">Altitude</p>
                <p>{status.altitude?.toFixed(2)} m</p>
              </div>
            </div>
          </div>
        )}

        {/* Survey-In Status */}
        {status.surveyIn && (
          <div className="bg-card border border-border p-6">
            <div className="flex items-center gap-2 mb-4">
              <Heartbeat className="h-4 w-4 text-muted-foreground" />
              <h2 className="font-medium">Survey-In Progress</h2>
            </div>
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div>
                <p className="text-xs text-muted-foreground mb-1">Status</p>
                <p className={status.surveyIn.valid ? "text-green-500" : "text-yellow-500"}>
                  {status.surveyIn.valid ? "Valid" : "In Progress"}
                </p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">Duration</p>
                <p className="font-mono">{Math.floor(status.surveyIn.duration / 60)}m</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-1">Accuracy</p>
                <p className="font-mono">{status.surveyIn.accuracy?.toFixed(3) ?? "—"} m</p>
              </div>
            </div>
          </div>
        )}

        {/* RTCM Messages */}
        {status.rtcmMessages && status.rtcmMessages.length > 0 && (
          <div className="bg-card border border-border p-6">
            <div className="flex items-center gap-2 mb-4">
              <Broadcast className="h-4 w-4 text-muted-foreground" />
              <h2 className="font-medium">RTCM Output</h2>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
              {status.rtcmMessages.map((msg) => (
                <div key={msg.type} className="font-mono">
                  <span className="text-muted-foreground">MSG {msg.type}:</span>{" "}
                  <span>{msg.count}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Not Connected State */}
        {!status.connected && (
          <div className="bg-muted/50 border border-border p-8 text-center">
            <CellTower className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
            <h3 className="font-medium text-foreground mb-2">GPS Module Not Connected</h3>
            <p className="text-sm text-muted-foreground max-w-md mx-auto">
              Connect a ZED-F9P GPS module via USB to the depot server.
              The device should appear as /dev/ttyUSB0 or /dev/ttyACM0.
            </p>
            <p className="text-xs text-muted-foreground mt-4 font-mono">
              docker compose --profile rtk up -d
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
