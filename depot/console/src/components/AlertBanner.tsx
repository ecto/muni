import { Link } from "react-router-dom";
import { useConsoleStore } from "@/store";
import { AlertSeverity } from "@/lib/types";
import { Warning, WarningCircle } from "@phosphor-icons/react";

export function AlertBanner() {
  const alerts = useConsoleStore((s) => s.alerts);

  const activeAlerts = alerts.filter((a) => !a.clearedAt && !a.acknowledgedAt);
  const criticalCount = activeAlerts.filter(
    (a) => a.severity === AlertSeverity.Critical
  ).length;
  const warningCount = activeAlerts.filter(
    (a) => a.severity === AlertSeverity.Warning
  ).length;

  if (criticalCount === 0 && warningCount === 0) return null;

  const isCritical = criticalCount > 0;

  return (
    <Link
      to="/alerts"
      className={`flex items-center gap-2 px-4 py-1.5 text-xs font-medium ${
        isCritical
          ? "bg-red-500/15 text-red-600 dark:text-red-400 border-b border-red-500/20"
          : "bg-yellow-500/15 text-yellow-700 dark:text-yellow-400 border-b border-yellow-500/20"
      } hover:opacity-80 transition-opacity`}
    >
      {isCritical ? (
        <WarningCircle className="h-3.5 w-3.5" weight="fill" />
      ) : (
        <Warning className="h-3.5 w-3.5" weight="fill" />
      )}
      <span>
        {criticalCount > 0 && `${criticalCount} critical`}
        {criticalCount > 0 && warningCount > 0 && " · "}
        {warningCount > 0 && `${warningCount} warning`}
        {" — "}
        {activeAlerts[0]?.message}
        {activeAlerts.length > 1 && ` (+${activeAlerts.length - 1} more)`}
      </span>
    </Link>
  );
}
