import { useEffect, useRef, useCallback } from "react";
import { useOperatorStore } from "@/store";
import {
  encodeTwist,
  encodeEStop,
  encodeEStopRelease,
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

  // Use ref for connect function to avoid circular dependency
  const connectRef = useRef<() => void>(() => {});

  const clearIntervals = useCallback(() => {
    if (commandIntervalRef.current) {
      clearInterval(commandIntervalRef.current);
      commandIntervalRef.current = null;
    }
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
  }, []);

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
            // Normal mode: conservative speed. Boost mode: full power!
            const speedMult = input.boost ? 5.0 : 2.0;
            const linear = input.linear * speedMult;
            const angular = input.angular * 1.5; // Max 1.5 rad/s
            ws.send(encodeTwist(linear, angular, input.boost));

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

      ws.onclose = () => {
        setConnected(false);
        clearIntervals();

        // Reconnect after delay
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      ws.onerror = () => {
        setConnected(false);
      };

      wsRef.current = ws;
    } catch {
      setConnected(false);

      // Retry connection
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [roverAddress, setConnected, setLatency, updateTelemetry, clearIntervals]);

  // Keep connectRef in sync
  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  const disconnect = useCallback(() => {
    clearIntervals();

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setConnected(false);
  }, [clearIntervals, setConnected]);

  // Connect when component mounts (TeleopScreen), disconnect on unmount
  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
    // Note: we intentionally only run this on mount/unmount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Expose methods for manual control
  return {
    connect,
    disconnect,
    sendEStop: useCallback(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(encodeEStop());
      }
    }, []),
    sendEStopRelease: useCallback(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(encodeEStopRelease());
      }
    }, []),
  };
}
