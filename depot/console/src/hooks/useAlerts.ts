import { useEffect, useCallback } from "react";
import { useConsoleStore } from "@/store";
import type { Alert } from "@/lib/types";

/**
 * Fetches alerts from the dispatch server and keeps them synced.
 * WebSocket updates are handled by useDispatch (alert_created/alert_acknowledged/alert_cleared).
 * Call once at the app root level.
 */
export function useAlerts() {
  const setAlerts = useConsoleStore((s) => s.setAlerts);

  const getDispatchUrl = useCallback((path: string) => {
    if (window.location.hostname === "localhost") {
      return `http://depot:4890${path}`;
    }
    const protocol = window.location.protocol;
    return `${protocol}//${window.location.hostname}:4890${path}`;
  }, []);

  const fetchAlerts = useCallback(async () => {
    try {
      const url = getDispatchUrl("/api/alerts?limit=200");
      const response = await fetch(url);
      if (response.ok) {
        const alerts: Alert[] = await response.json();
        setAlerts(alerts);
      }
    } catch {
      // Silently fail — alerts will populate from WebSocket
    }
  }, [getDispatchUrl, setAlerts]);

  const acknowledgeAlert = useCallback(
    async (id: string) => {
      try {
        const url = getDispatchUrl(`/api/alerts/${id}/ack`);
        const response = await fetch(url, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ by: "operator" }),
        });
        if (response.ok) {
          const updated: Alert = await response.json();
          useConsoleStore.getState().updateAlert(id, {
            acknowledgedAt: updated.acknowledgedAt,
            acknowledgedBy: updated.acknowledgedBy,
          });
        }
      } catch (e) {
        console.error("[alerts] Failed to acknowledge:", e);
      }
    },
    [getDispatchUrl]
  );

  const clearAlert = useCallback(
    async (id: string) => {
      try {
        const url = getDispatchUrl(`/api/alerts/${id}/clear`);
        const response = await fetch(url, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
        });
        if (response.ok) {
          const updated: Alert = await response.json();
          useConsoleStore.getState().updateAlert(id, {
            clearedAt: updated.clearedAt,
          });
        }
      } catch (e) {
        console.error("[alerts] Failed to clear:", e);
      }
    },
    [getDispatchUrl]
  );

  // Fetch initial alerts
  useEffect(() => {
    fetchAlerts();
    // Refresh every 30 seconds as a fallback
    const interval = setInterval(fetchAlerts, 30_000);
    return () => clearInterval(interval);
  }, [fetchAlerts]);

  return { fetchAlerts, acknowledgeAlert, clearAlert };
}
