import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import {
  encodeTwist,
  encodeEStop,
  encodeEStopRelease,
  encodeHeartbeat,
  encodeTool,
  decodeTelemetry,
  telemetryFromDecoded,
} from "@/lib/protocol";

const COMMAND_INTERVAL_MS = 10; // 100Hz: higher rate for lower latency and packet loss redundancy
const HEARTBEAT_INTERVAL_MS = 100;
const RECONNECT_DELAY_MS = 2000;

// Track page visibility for safety: stop commands when tab is hidden
let isPageVisible = typeof document !== "undefined" ? !document.hidden : true;

export function useRoverConnection() {
  const { roverAddress, setConnected, setLatency, updateTelemetry } =
    useConsoleStore();

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

        // Send immediate zero command to establish session quickly
        ws.send(encodeTwist(0, 0, false));
        ws.send(encodeHeartbeat());

        // Start sending commands at 50Hz
        commandIntervalRef.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            // Safety: if page is not visible, send zero velocity and skip
            // This prevents stale commands when browser throttles the tab
            if (!isPageVisible) {
              ws.send(encodeTwist(0, 0, false));
              return;
            }

            const state = useConsoleStore.getState();
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

    // Safety: track page visibility to stop commands when tab is hidden
    const handleVisibilityChange = () => {
      isPageVisible = !document.hidden;
      if (!isPageVisible) {
        // Immediately send stop command when losing focus
        if (wsRef.current?.readyState === WebSocket.OPEN) {
          wsRef.current.send(encodeTwist(0, 0, false));
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
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
