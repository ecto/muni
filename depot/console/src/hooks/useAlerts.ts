import { useEffect, useCallback, useRef } from "react";
import { useConsoleStore } from "@/store";
import type { Alert } from "@/lib/types";

/**
 * Fetches alerts from the dispatch server and keeps them synced.
 * WebSocket updates are handled by useDispatch (alert_created/alert_acknowledged/alert_cleared).
 * Call once at the app root level.
 */
/** Dispatch API base — proxied by Vite (dev) and nginx (prod) */
const DISPATCH_API = "/api/dispatch";

export function useAlerts() {
  const setAlerts = useConsoleStore((s) => s.setAlerts);
  const failedRef = useRef(false);

  const fetchAlerts = useCallback(async () => {
    try {
      const response = await fetch(`${DISPATCH_API}/api/alerts?limit=200`);
      if (response.ok) {
        failedRef.current = false;
        const alerts: Alert[] = await response.json();
        setAlerts(alerts);
      } else {
        failedRef.current = true;
      }
    } catch {
      failedRef.current = true;
    }
  }, [setAlerts]);

  const acknowledgeAlert = useCallback(
    async (id: string) => {
      try {
        const response = await fetch(`${DISPATCH_API}/api/alerts/${id}/ack`, {
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
    []
  );

  const clearAlert = useCallback(
    async (id: string) => {
      try {
        const response = await fetch(`${DISPATCH_API}/api/alerts/${id}/clear`, {
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
    []
  );

  // Fetch initial alerts, poll every 30s only while dispatch is reachable
  useEffect(() => {
    fetchAlerts();
    const interval = setInterval(() => {
      if (!failedRef.current) fetchAlerts();
    }, 30_000);
    return () => clearInterval(interval);
  }, [fetchAlerts]);

  return { fetchAlerts, acknowledgeAlert, clearAlert };
}
