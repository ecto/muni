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

  const getDiscoveryUrl = useCallback(() => {
    if (window.location.hostname === "localhost") {
      return "ws://localhost:4860/ws";
    }
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.hostname}:4860/ws`;
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

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
        console.debug("[discovery] Disconnected, reconnecting...");
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      ws.onerror = (e) => {
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
    connect();

    return () => {
      disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    connect,
    disconnect,
  };
}
