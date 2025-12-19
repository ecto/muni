import { useEffect, useRef, useCallback } from "react";
import { useOperatorStore } from "@/store";
import {
  encodeTwist,
  encodeEStop,
  encodeHeartbeat,
  encodeTool,
  decodeTelemetry,
  telemetryFromDecoded,
} from "@/lib/protocol";

const COMMAND_INTERVAL_MS = 20; // 50Hz
const HEARTBEAT_INTERVAL_MS = 100;
const RECONNECT_DELAY_MS = 2000;

export function useRoverConnection() {
  const { roverAddress, setConnected, setLatency, updateTelemetry } =
    useOperatorStore();

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const commandIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null
  );
  const heartbeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null
  );
  const lastSendTimeRef = useRef<number>(0);

  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    try {
      const ws = new WebSocket(roverAddress);
      ws.binaryType = "arraybuffer";

      ws.onopen = () => {
        setConnected(true);

        // Start sending commands at 50Hz
        commandIntervalRef.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            const state = useOperatorStore.getState();
            const { input } = state;

            // Send E-Stop immediately if pressed
            if (input.estop) {
              ws.send(encodeEStop());
              return;
            }

            // Send twist command
            const linear = input.linear * 2.0; // Max 2 m/s
            const angular = input.angular * 1.5; // Max 1.5 rad/s
            ws.send(encodeTwist(linear, angular));

            // Send tool command if any tool input
            if (
              Math.abs(input.toolAxis) > 0.01 ||
              input.actionA ||
              input.actionB
            ) {
              ws.send(
                encodeTool(input.toolAxis, 0, input.actionA, input.actionB)
              );
            }

            lastSendTimeRef.current = performance.now();
          }
        }, COMMAND_INTERVAL_MS);

        // Heartbeat at 10Hz
        heartbeatIntervalRef.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(encodeHeartbeat());
          }
        }, HEARTBEAT_INTERVAL_MS);
      };

      ws.onmessage = (event) => {
        if (!(event.data instanceof ArrayBuffer)) {
          return;
        }

        const decoded = decodeTelemetry(event.data);
        if (decoded) {
          const telemetry = telemetryFromDecoded(decoded);

          // Compute round-trip latency
          const now = performance.now();
          const latency = Math.round(now - lastSendTimeRef.current);
          setLatency(latency);

          updateTelemetry({
            ...telemetry,
            connected: true,
            latency_ms: latency,
          });
        }
      };

      ws.onclose = (_event) => {
        setConnected(false);
        cleanup();

        // Reconnect after delay
        reconnectTimeoutRef.current = setTimeout(connect, RECONNECT_DELAY_MS);
      };

      ws.onerror = (_error) => {
        setConnected(false);
      };

      wsRef.current = ws;
    } catch (_error) {
      setConnected(false);

      // Retry connection
      reconnectTimeoutRef.current = setTimeout(connect, RECONNECT_DELAY_MS);
    }
  }, [roverAddress, setConnected, setLatency, updateTelemetry]);

  const cleanup = useCallback(() => {
    if (commandIntervalRef.current) {
      clearInterval(commandIntervalRef.current);
      commandIntervalRef.current = null;
    }
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
  }, []);

  const disconnect = useCallback(() => {
    cleanup();

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setConnected(false);
  }, [cleanup, setConnected]);

  // Connect on mount, disconnect on unmount
  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  // Expose methods for manual control
  return {
    connect,
    disconnect,
    sendEStop: useCallback(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(encodeEStop());
      }
    }, []),
  };
}
