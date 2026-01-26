import { useEffect, useRef } from "react";
import { useConsoleStore } from "@/store";
import { AlertSeverity, Mode, type RoverInfo } from "@/lib/types";

let alertCounter = 0;
function makeAlertId(): string {
  return `alert-${Date.now()}-${++alertCounter}`;
}

/**
 * Derives alerts from existing telemetry data in the Zustand store.
 * Monitors rover battery, mode, online status, service health, and GPS.
 * Call once at the app root level.
 */
export function useAlerts() {
  const prevRoversRef = useRef<Map<string, RoverInfo>>(new Map());
  const prevServicesRef = useRef<Map<string, string>>(new Map());
  const prevGpsFixRef = useRef<string | null>(null);

  useEffect(() => {
    // Poll store at 2Hz to derive alerts
    const interval = setInterval(() => {
      const state = useConsoleStore.getState();
      const addAlert = state.addAlert;
      const clearAlert = state.clearAlert;

      // --- Rover alerts ---
      const prevRovers = prevRoversRef.current;
      const nextRovers = new Map<string, RoverInfo>();

      for (const rover of state.rovers) {
        nextRovers.set(rover.id, rover);
        const prev = prevRovers.get(rover.id);

        // Battery thresholds
        if (rover.online && rover.batteryVoltage > 0) {
          if (rover.batteryVoltage < 42) {
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Critical,
              source: "rover",
              sourceId: rover.id,
              message: `Battery critical (${rover.batteryVoltage.toFixed(1)}V)`,
              timestamp: Date.now(),
              acknowledged: false,
            });
          } else if (rover.batteryVoltage < 45) {
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Warning,
              source: "rover",
              sourceId: rover.id,
              message: `Battery low (${rover.batteryVoltage.toFixed(1)}V)`,
              timestamp: Date.now(),
              acknowledged: false,
            });
          }
        }

        // Online → Offline transition
        if (prev?.online && !rover.online) {
          addAlert({
            id: makeAlertId(),
            severity: AlertSeverity.Critical,
            source: "rover",
            sourceId: rover.id,
            message: "Connection lost",
            timestamp: Date.now(),
            acknowledged: false,
          });
        }

        // Offline → Online transition
        if (prev && !prev.online && rover.online) {
          addAlert({
            id: makeAlertId(),
            severity: AlertSeverity.Info,
            source: "rover",
            sourceId: rover.id,
            message: "Rover online",
            timestamp: Date.now(),
            acknowledged: false,
          });
          // Clear any active "Connection lost" alerts for this rover
          for (const a of state.alerts) {
            if (
              !a.clearedAt &&
              a.source === "rover" &&
              a.sourceId === rover.id &&
              a.message === "Connection lost"
            ) {
              clearAlert(a.id);
            }
          }
        }

        // Mode transitions
        if (prev && prev.mode !== rover.mode) {
          if (rover.mode === Mode.EStop) {
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Critical,
              source: "rover",
              sourceId: rover.id,
              message: "Emergency stop activated",
              timestamp: Date.now(),
              acknowledged: false,
            });
          } else if (rover.mode === Mode.Fault) {
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Critical,
              source: "rover",
              sourceId: rover.id,
              message: "Fault detected",
              timestamp: Date.now(),
              acknowledged: false,
            });
          } else if (prev.mode === Mode.EStop) {
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Info,
              source: "rover",
              sourceId: rover.id,
              message: "E-Stop cleared",
              timestamp: Date.now(),
              acknowledged: false,
            });
            // Clear active e-stop alerts
            for (const a of state.alerts) {
              if (
                !a.clearedAt &&
                a.source === "rover" &&
                a.sourceId === rover.id &&
                a.message === "Emergency stop activated"
              ) {
                clearAlert(a.id);
              }
            }
          } else {
            // General mode change info
            const modeNames: Record<number, string> = {
              0: "Disabled",
              1: "Idle",
              2: "Teleop",
              3: "Autonomous",
              6: "Sleep",
            };
            const modeName = modeNames[rover.mode] ?? `Mode ${rover.mode}`;
            addAlert({
              id: makeAlertId(),
              severity: AlertSeverity.Info,
              source: "rover",
              sourceId: rover.id,
              message: `Mode changed to ${modeName}`,
              timestamp: Date.now(),
              acknowledged: false,
            });
          }
        }
      }
      prevRoversRef.current = nextRovers;

      // --- Service health alerts ---
      const prevServices = prevServicesRef.current;
      const nextServices = new Map<string, string>();

      for (const service of state.services) {
        nextServices.set(service.name, service.status);
        const prev = prevServices.get(service.name);

        if (prev === "healthy" && service.status === "unhealthy") {
          addAlert({
            id: makeAlertId(),
            severity: AlertSeverity.Warning,
            source: "service",
            sourceId: service.name,
            message: `${service.name} service unhealthy`,
            timestamp: Date.now(),
            acknowledged: false,
          });
        }
        if (prev === "unhealthy" && service.status === "healthy") {
          // Clear active unhealthy alerts
          for (const a of state.alerts) {
            if (
              !a.clearedAt &&
              a.source === "service" &&
              a.sourceId === service.name &&
              a.message.includes("unhealthy")
            ) {
              clearAlert(a.id);
            }
          }
        }
      }
      prevServicesRef.current = nextServices;

      // --- GPS alerts ---
      const gps = state.gpsStatus;
      if (gps) {
        const prevFix = prevGpsFixRef.current;
        const currentFix = gps.fixQuality;

        if (prevFix && prevFix !== "no_fix" && currentFix === "no_fix") {
          addAlert({
            id: makeAlertId(),
            severity: AlertSeverity.Warning,
            source: "gps",
            sourceId: "base",
            message: "GPS fix lost",
            timestamp: Date.now(),
            acknowledged: false,
          });
        }
        if (prevFix === "no_fix" && currentFix && currentFix !== "no_fix") {
          for (const a of state.alerts) {
            if (
              !a.clearedAt &&
              a.source === "gps" &&
              a.message === "GPS fix lost"
            ) {
              clearAlert(a.id);
            }
          }
        }
        prevGpsFixRef.current = currentFix;
      }
    }, 500);

    return () => clearInterval(interval);
  }, []);
}
