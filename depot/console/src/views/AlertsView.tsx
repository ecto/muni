import { useMemo, useState } from "react";
import { useConsoleStore } from "@/store";
import { AlertSeverity, type Alert, type AlertSeverity as AlertSeverityType } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Warning,
  WarningCircle,
  CheckCircle,
  Info,
  Eye,
  Funnel,
} from "@phosphor-icons/react";

function timeAgo(ms: number): string {
  const secs = Math.floor((Date.now() - ms) / 1000);
  if (secs < 10) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

function SeverityIcon({ severity }: { severity: AlertSeverityType }) {
  switch (severity) {
    case AlertSeverity.Critical:
      return <WarningCircle className="h-4 w-4 text-red-500" weight="fill" />;
    case AlertSeverity.Warning:
      return <Warning className="h-4 w-4 text-yellow-500" weight="fill" />;
    case AlertSeverity.Info:
      return <Info className="h-4 w-4 text-blue-500" weight="fill" />;
  }
}

function SeverityBadge({ severity }: { severity: AlertSeverityType }) {
  const variant =
    severity === AlertSeverity.Critical
      ? "destructive"
      : severity === AlertSeverity.Warning
        ? "outline"
        : "secondary";
  return (
    <Badge variant={variant} className="text-[10px] h-5 px-1.5 uppercase">
      {severity}
    </Badge>
  );
}

function AlertRow({
  alert,
  onAcknowledge,
}: {
  alert: Alert;
  onAcknowledge: (id: string) => void;
}) {
  const isActive = !alert.clearedAt;
  const isAcknowledged = alert.acknowledged;

  return (
    <div
      className={`flex items-start gap-3 px-4 py-3 border-b border-border ${
        isActive && !isAcknowledged ? "bg-muted/30" : ""
      } ${alert.clearedAt ? "opacity-50" : ""}`}
    >
      <SeverityIcon severity={alert.severity} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <SeverityBadge severity={alert.severity} />
          <span className="text-sm font-medium">{alert.sourceId}</span>
          <span className="text-xs text-muted-foreground">
            {timeAgo(alert.timestamp)}
          </span>
          {alert.clearedAt && (
            <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
              cleared
            </Badge>
          )}
          {isAcknowledged && !alert.clearedAt && (
            <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
              ack
            </Badge>
          )}
        </div>
        <p className="text-sm text-foreground">{alert.message}</p>
      </div>
      {isActive && !isAcknowledged && (
        <Button
          size="sm"
          variant="outline"
          className="h-7 px-2 text-xs shrink-0"
          onClick={() => onAcknowledge(alert.id)}
        >
          <Eye className="h-3 w-3 mr-1" />
          ACK
        </Button>
      )}
    </div>
  );
}

type FilterSeverity = "all" | AlertSeverityType;

export function AlertsView() {
  const alerts = useConsoleStore((s) => s.alerts);
  const acknowledgeAlert = useConsoleStore((s) => s.acknowledgeAlert);
  const [filter, setFilter] = useState<FilterSeverity>("all");
  const [showCleared, setShowCleared] = useState(false);

  const activeAlerts = useMemo(
    () =>
      alerts.filter((a) => {
        if (!showCleared && a.clearedAt) return false;
        if (filter !== "all" && a.severity !== filter) return false;
        return true;
      }),
    [alerts, filter, showCleared]
  );

  const activeUncleared = alerts.filter((a) => !a.clearedAt);
  const criticalCount = activeUncleared.filter(
    (a) => a.severity === AlertSeverity.Critical
  ).length;
  const warningCount = activeUncleared.filter(
    (a) => a.severity === AlertSeverity.Warning
  ).length;
  const infoCount = activeUncleared.filter(
    (a) => a.severity === AlertSeverity.Info
  ).length;

  return (
    <div className="min-h-full p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold">Alerts</h1>
          <p className="text-sm text-muted-foreground">
            {criticalCount} critical · {warningCount} warning · {infoCount} info
          </p>
        </div>

        {/* Active Alarms Summary */}
        {(criticalCount > 0 || warningCount > 0) && (
          <div
            className={`border rounded-lg p-4 ${
              criticalCount > 0
                ? "border-red-500/30 bg-red-500/5"
                : "border-yellow-500/30 bg-yellow-500/5"
            }`}
          >
            <div className="flex items-center gap-2 mb-2">
              {criticalCount > 0 ? (
                <WarningCircle
                  className="h-5 w-5 text-red-500"
                  weight="fill"
                />
              ) : (
                <Warning
                  className="h-5 w-5 text-yellow-500"
                  weight="fill"
                />
              )}
              <span className="font-medium text-sm">
                {criticalCount + warningCount} Active Alarm
                {criticalCount + warningCount !== 1 ? "s" : ""}
              </span>
            </div>
            <div className="space-y-1">
              {activeUncleared
                .filter(
                  (a) =>
                    a.severity === AlertSeverity.Critical ||
                    a.severity === AlertSeverity.Warning
                )
                .slice(0, 5)
                .map((alert) => (
                  <div
                    key={alert.id}
                    className="flex items-center gap-2 text-sm"
                  >
                    <SeverityIcon severity={alert.severity} />
                    <span className="font-medium">{alert.sourceId}</span>
                    <span className="text-muted-foreground">
                      {alert.message}
                    </span>
                    <span className="text-xs text-muted-foreground/50 ml-auto">
                      {timeAgo(alert.timestamp)}
                    </span>
                    {!alert.acknowledged && (
                      <Button
                        size="sm"
                        variant="outline"
                        className="h-6 px-2 text-[10px]"
                        onClick={() => acknowledgeAlert(alert.id)}
                      >
                        ACK
                      </Button>
                    )}
                  </div>
                ))}
            </div>
          </div>
        )}

        {/* Filters */}
        <div className="flex items-center gap-2">
          <Funnel className="h-4 w-4 text-muted-foreground" />
          {(
            [
              ["all", "All"],
              [AlertSeverity.Critical, "Critical"],
              [AlertSeverity.Warning, "Warning"],
              [AlertSeverity.Info, "Info"],
            ] as const
          ).map(([value, label]) => (
            <Button
              key={value}
              size="sm"
              variant={filter === value ? "default" : "outline"}
              className="h-7 px-2.5 text-xs"
              onClick={() => setFilter(value as FilterSeverity)}
            >
              {label}
            </Button>
          ))}
          <div className="flex-1" />
          <Button
            size="sm"
            variant={showCleared ? "default" : "outline"}
            className="h-7 px-2.5 text-xs"
            onClick={() => setShowCleared(!showCleared)}
          >
            {showCleared ? "Hide Cleared" : "Show Cleared"}
          </Button>
        </div>

        {/* Alert History */}
        <div className="border border-border rounded-lg overflow-hidden">
          <div className="px-4 py-2 bg-muted/30 border-b border-border">
            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              {showCleared ? "All History" : "Active & Uncleared"} ({activeAlerts.length})
            </span>
          </div>
          {activeAlerts.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground">
              <CheckCircle className="h-12 w-12 mx-auto mb-3 opacity-30" />
              <p className="text-sm">No alerts to display</p>
            </div>
          ) : (
            <div>
              {activeAlerts.map((alert) => (
                <AlertRow
                  key={alert.id}
                  alert={alert}
                  onAcknowledge={acknowledgeAlert}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
