import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import { decodeTelemetry, telemetryFromDecoded } from "@/lib/protocol";
// Vite Web Worker import syntax
import CommandWorker from "@/workers/commandWorker?worker";

const RECONNECT_DELAY_MS = 2000;
const INPUT_UPDATE_INTERVAL_MS = 16; // 60Hz input polling

// Track page visibility for safety: stop commands when tab is hidden
let isPageVisible = typeof document !== "undefined" ? !document.hidden : true;

export function useRoverConnection() {
  const { roverAddress, setConnected, setLatency, updateTelemetry } =
    useConsoleStore();

  const workerRef = useRef<Worker | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastSendTimeRef = useRef<number>(0);
  const connectRef = useRef<() => void>(() => {});

  const clearIntervals = useCallback(() => {
    if (inputIntervalRef.current) {
      clearInterval(inputIntervalRef.current);
      inputIntervalRef.current = null;
    }
  }, []);

  const connect = useCallback(() => {
    // Clean up existing worker
    if (workerRef.current) {
      workerRef.current.postMessage({ type: "stop" });
      workerRef.current.terminate();
      workerRef.current = null;
    }

    try {
      // Create Web Worker for consistent command timing
      const worker = new CommandWorker();
      workerRef.current = worker;

      worker.onmessage = (event) => {
        const { type, data } = event.data;

        switch (type) {
          case "connected":
            setConnected(true);
            lastSendTimeRef.current = performance.now();

            // Start polling input state and sending to worker
            inputIntervalRef.current = setInterval(() => {
              const { input } = useConsoleStore.getState();

              // Safety: send zero velocity when page hidden
              if (!isPageVisible) {
                worker.postMessage({
                  type: "updateInput",
                  input: {
                    linear: 0,
                    angular: 0,
                    boost: false,
                    estop: false,
                    toolAxis: 0,
                    actionA: false,
                    actionB: false,
                  },
                });
                return;
              }

              worker.postMessage({
                type: "updateInput",
                input: {
                  linear: input.linear,
                  angular: input.angular,
                  boost: input.boost,
                  estop: input.estop,
                  toolAxis: input.toolAxis,
                  actionA: input.actionA,
                  actionB: input.actionB,
                },
              });
            }, INPUT_UPDATE_INTERVAL_MS);
            break;

          case "telemetry":
            if (data instanceof ArrayBuffer) {
              const decoded = decodeTelemetry(data);
              if (decoded) {
                const telemetry = telemetryFromDecoded(decoded);
                const now = performance.now();
                const latency = Math.round(now - lastSendTimeRef.current);
                lastSendTimeRef.current = now;
                setLatency(latency);

                updateTelemetry({
                  ...telemetry,
                  connected: true,
                  latency_ms: latency,
                });
              }
            }
            break;

          case "disconnected":
            setConnected(false);
            clearIntervals();
            // Reconnect after delay
            reconnectTimeoutRef.current = setTimeout(() => {
              connectRef.current();
            }, RECONNECT_DELAY_MS);
            break;

          case "error":
            setConnected(false);
            break;
        }
      };

      // Start the worker connection
      worker.postMessage({ type: "start", wsUrl: roverAddress });
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

    if (workerRef.current) {
      workerRef.current.postMessage({ type: "stop" });
      workerRef.current.terminate();
      workerRef.current = null;
    }

    setConnected(false);
  }, [clearIntervals, setConnected]);

  // Connect when rover address changes (after selectRover is called)
  useEffect(() => {
    // Don't connect to default localhost - wait for real rover address
    if (roverAddress === "ws://localhost:4850") {
      return;
    }

    connect();

    // Safety: track page visibility to stop commands when tab is hidden
    const handleVisibilityChange = () => {
      isPageVisible = !document.hidden;
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      disconnect();
    };
  }, [roverAddress, connect, disconnect]);

  // Expose methods for manual control
  return {
    connect,
    disconnect,
    sendEStop: useCallback(() => {
      // E-Stop needs to bypass the worker for immediate response
      // The worker also handles estop via input state, but this is a backup
      workerRef.current?.postMessage({
        type: "updateInput",
        input: {
          linear: 0,
          angular: 0,
          boost: false,
          estop: true,
          toolAxis: 0,
          actionA: false,
          actionB: false,
        },
      });
    }, []),
    sendEStopRelease: useCallback(() => {
      // This needs direct WebSocket access, which we don't have with worker
      // For now, just clear the estop state - the proper fix would be
      // to add an EStopRelease message type to the worker
      console.warn("E-Stop release not fully implemented with Web Worker");
    }, []),
  };
}
