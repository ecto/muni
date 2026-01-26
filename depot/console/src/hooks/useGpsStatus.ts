import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import type { GpsStatus } from "@/lib/types";

/**
 * Hook to connect to the GPS status service and receive live updates
 */
export function useGpsStatus() {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const connectRef = useRef<() => void>(() => {});
  const intentionalCloseRef = useRef(false);
  const setGpsStatus = useConsoleStore((s) => s.setGpsStatus);

  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    intentionalCloseRef.current = false;

    // Determine WebSocket URL - always use the proxy path
    const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${wsProtocol}//${window.location.host}/api/gps/ws`;

    console.debug("[GPS] Connecting to", wsUrl);

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      console.debug("[GPS] Connected");
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === "gps_status" && msg.data) {
          // Get current state to preserve satellite data if new data is empty
          const currentStatus = useConsoleStore.getState().gpsStatus;
          const newSatelliteInfo = msg.data.satelliteInfo ?? [];

          const status: GpsStatus = {
            connected: msg.data.connected ?? false,
            mode: msg.data.mode ?? "unknown",
            fixQuality: msg.data.fixQuality ?? "no_fix",
            satellites: msg.data.satellites ?? 0,
            latitude: msg.data.latitude,
            longitude: msg.data.longitude,
            altitude: msg.data.altitude,
            hdop: msg.data.hdop,
            vdop: msg.data.vdop,
            pdop: msg.data.pdop,
            // Preserve previous satellite info if new data is empty
            satelliteInfo: newSatelliteInfo.length > 0
              ? newSatelliteInfo
              : (currentStatus?.satelliteInfo ?? []),
            history: msg.data.history ?? [],
            surveyIn: msg.data.surveyIn,
            rtcmMessages: msg.data.rtcmMessages ?? [],
            clients: msg.data.clients,
            lastUpdate: msg.data.lastUpdate ?? Date.now(),
          };
          setGpsStatus(status);
        }
      } catch (e) {
        console.error("[GPS] Failed to parse message:", e);
      }
    };

    ws.onerror = (error) => {
      // Ignore errors from stale connections
      if (wsRef.current !== ws) return;
      console.error("[GPS] WebSocket error:", error);
    };

    ws.onclose = () => {
      // If a new connection has already been started, ignore this close event
      if (wsRef.current !== ws) {
        return;
      }
      wsRef.current = null;
      if (intentionalCloseRef.current) {
        console.debug("[GPS] Disconnected (intentional)");
        return;
      }
      console.debug("[GPS] Disconnected, reconnecting in 3s...");
      // Mark as disconnected
      setGpsStatus({ connected: false } as GpsStatus);
      // Schedule reconnect using ref to avoid stale closure
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, 3000);
    };
  }, [setGpsStatus]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  useEffect(() => {
    // Defer connection to next tick - allows StrictMode's immediate
    // cleanup to cancel before we actually open the WebSocket
    const timeoutId = setTimeout(connect, 0);

    return () => {
      clearTimeout(timeoutId);
      intentionalCloseRef.current = true;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connect]);
}
