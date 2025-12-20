import { useEffect, useRef, useCallback } from "react";
import { useOperatorStore } from "@/store";
import type { RoverInfo } from "@/lib/types";

const RECONNECT_DELAY_MS = 3000;

/**
 * Hook to connect to the discovery service and receive live rover updates.
 *
 * The discovery service provides a WebSocket endpoint that streams rover
 * registration and status updates in real-time.
 */
export function useDiscovery() {
  const { setRovers } = useOperatorStore();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const connectRef = useRef<() => void>(() => {});

  // Discovery service URL - same origin in production, configurable for dev
  const getDiscoveryUrl = useCallback(() => {
    // In development, use localhost with the discovery port
    if (window.location.hostname === "localhost") {
      return "ws://localhost:4860/ws";
    }
    // In production (Docker), discovery is on the same host
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.hostname}:4860/ws`;
  }, []);

  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    const url = getDiscoveryUrl();
    console.log("[discovery] Connecting to", url);

    try {
      const ws = new WebSocket(url);

      ws.onopen = () => {
        console.log("[discovery] Connected");
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);

          if (message.type === "rovers") {
            // Transform from server format to client format
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

            console.log("[discovery] Received", rovers.length, "rovers");
            setRovers(rovers);
          }
        } catch (e) {
          console.error("[discovery] Failed to parse message:", e);
        }
      };

      ws.onclose = () => {
        console.log("[discovery] Disconnected, reconnecting...");

        // Reconnect after delay
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

      // Retry connection
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [getDiscoveryUrl, setRovers]);

  // Keep connectRef in sync
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

  // Connect on mount, disconnect on unmount
  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
    // Only run on mount/unmount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    connect,
    disconnect,
  };
}
