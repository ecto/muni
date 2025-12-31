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
  const { setGpsStatus } = useConsoleStore();

  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    // Determine WebSocket URL
    // In development, connect directly; in production, use the proxy
    const isDev = import.meta.env.DEV;
    const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = isDev
      ? `${wsProtocol}//${window.location.hostname}:4880/ws`
      : `${wsProtocol}//${window.location.host}/api/gps/ws`;

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
          const status: GpsStatus = {
            connected: msg.data.connected ?? false,
            mode: msg.data.mode ?? "unknown",
            fixQuality: msg.data.fixQuality ?? "no_fix",
            satellites: msg.data.satellites ?? 0,
            latitude: msg.data.latitude,
            longitude: msg.data.longitude,
            altitude: msg.data.altitude,
            hdop: msg.data.hdop,
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
      console.error("[GPS] WebSocket error:", error);
    };

    ws.onclose = () => {
      console.debug("[GPS] Disconnected, reconnecting in 3s...");
      wsRef.current = null;
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
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);
}
