import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import type { RoverInfo } from "@/lib/types";

const RECONNECT_DELAY_MS = 3000;

/**
 * Hook to connect to the discovery service and receive live rover updates.
 */
export function useDiscovery() {
  const { setRovers } = useConsoleStore();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const connectRef = useRef<() => void>(() => {});
  const intentionalCloseRef = useRef(false);

  const getDiscoveryUrl = useCallback(() => {
    if (window.location.hostname === "localhost") {
      // Dev mode: use VITE_DEPOT_HOST env var, or fall back to depot hostname
      // Note: browsers can't resolve Tailscale MagicDNS, so use IP if needed
      // Set VITE_DEPOT_HOST=100.71.209.42 in .env.local for Tailscale
      const depotHost = import.meta.env.VITE_DEPOT_HOST || "depot";
      return `ws://${depotHost}:4860/ws`;
    }
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.hostname}:4860/ws`;
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    intentionalCloseRef.current = false;
    const url = getDiscoveryUrl();
    console.debug("[discovery] Connecting to", url);

    try {
      const ws = new WebSocket(url);

      ws.onopen = () => {
        console.debug("[discovery] Connected");
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);

          if (message.type === "rovers") {
            const rovers: RoverInfo[] = message.data.map(
              (r: {
                id: string;
                name: string;
                address: string;
                videoAddress: string;
                online: boolean;
                batteryVoltage: number;
                lastPose: { x: number; y: number; theta: number };
                mode: number;
                lastSeen: number;
              }) => ({
                id: r.id,
                name: r.name,
                address: r.address,
                videoAddress: r.videoAddress,
                online: r.online,
                batteryVoltage: r.batteryVoltage,
                lastPose: r.lastPose,
                mode: r.mode,
                lastSeen: r.lastSeen,
              })
            );

            setRovers(rovers);
          }
        } catch (e) {
          console.error("[discovery] Failed to parse message:", e);
        }
      };

      ws.onclose = () => {
        // If a new connection has already been started, ignore this close event
        if (wsRef.current !== ws) {
          return;
        }
        if (intentionalCloseRef.current) {
          console.debug("[discovery] Disconnected (intentional)");
          return;
        }
        console.debug("[discovery] Disconnected, reconnecting...");
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      ws.onerror = (e) => {
        // Ignore errors from stale connections
        if (wsRef.current !== ws) return;
        console.error("[discovery] WebSocket error:", e);
      };

      wsRef.current = ws;
    } catch (e) {
      console.error("[discovery] Connection error:", e);
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [getDiscoveryUrl, setRovers]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  const disconnect = useCallback(() => {
    intentionalCloseRef.current = true;

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  useEffect(() => {
    // Defer connection to next tick - allows StrictMode's immediate
    // cleanup to cancel before we actually open the WebSocket
    const timeoutId = setTimeout(connect, 0);

    return () => {
      clearTimeout(timeoutId);
      disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    connect,
    disconnect,
  };
}
