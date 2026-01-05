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

// Velocity limits
const SPEED_NORMAL = 2.0; // m/s in normal mode
const SPEED_BOOST = 5.0; // m/s in boost mode
const MAX_ANGULAR_VEL = 1.5; // rad/s
const TOOL_DEADZONE = 0.01; // Minimum tool axis value to trigger command

// Track page visibility for safety: stop commands when tab is hidden
let isPageVisible = typeof document !== "undefined" ? !document.hidden : true;

interface InputState {
  linear: number;
  angular: number;
  boost: boolean;
  estop: boolean;
  toolAxis: number;
  actionA: boolean;
  actionB: boolean;
}

/** Calculate velocity commands from input state */
function calculateVelocities(input: InputState): { linear: number; angular: number } {
  const speedMultiplier = input.boost ? SPEED_BOOST : SPEED_NORMAL;
  return {
    linear: input.linear * speedMultiplier,
    angular: input.angular * MAX_ANGULAR_VEL,
  };
}

/** Check if tool input is active (above deadzone) */
function hasToolInput(input: InputState): boolean {
  return Math.abs(input.toolAxis) > TOOL_DEADZONE || input.actionA || input.actionB;
}

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

        // Start sending commands at 100Hz
        commandIntervalRef.current = setInterval(() => {
          if (ws.readyState !== WebSocket.OPEN) return;

          // Safety: send zero velocity when page hidden to prevent stale commands
          if (!isPageVisible) {
            ws.send(encodeTwist(0, 0, false));
            return;
          }

          const { input } = useConsoleStore.getState();

          // E-Stop takes priority over all other commands
          if (input.estop) {
            ws.send(encodeEStop());
            return;
          }

          // Send velocity command
          const { linear, angular } = calculateVelocities(input);
          ws.send(encodeTwist(linear, angular, input.boost));

          // Send tool command if active
          if (hasToolInput(input)) {
            ws.send(encodeTool(input.toolAxis, 0, input.actionA, input.actionB));
          }

          lastSendTimeRef.current = performance.now();
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
