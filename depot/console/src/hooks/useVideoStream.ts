import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import {
  decodeVideoFrame,
  videoFrameToBlobUrl,
  MSG_VIDEO_FRAME,
} from "@/lib/protocol";

const RECONNECT_DELAY_MS = 2000;

/**
 * Hook to connect to the rover's video stream via WebSocket.
 *
 * The video stream uses a separate WebSocket connection (or channel) to avoid
 * blocking telemetry with large video frames. Video frames are JPEG-encoded
 * equirectangular images from the Insta360 X4.
 */
export function useVideoStream() {
  const { videoAddress, setVideoConnected, setVideoFps, setVideoFrame } =
    useConsoleStore();

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const frameCountRef = useRef(0);
  const lastFpsUpdateRef = useRef(0);
  const lastBlobUrlRef = useRef<string | null>(null);

  // Use ref for connect function to avoid circular dependency
  const connectRef = useRef<() => void>(() => {});

  // Use video address from store
  const videoUrl = videoAddress;

  const connect = useCallback(() => {
    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    try {
      const ws = new WebSocket(videoUrl);
      ws.binaryType = "arraybuffer";

      ws.onopen = () => {
        setVideoConnected(true);
        frameCountRef.current = 0;
        lastFpsUpdateRef.current = performance.now();
      };

      ws.onmessage = (event) => {
        if (!(event.data instanceof ArrayBuffer)) {
          return;
        }

        // Check message type
        const view = new DataView(event.data);
        if (view.getUint8(0) !== MSG_VIDEO_FRAME) {
          return;
        }

        const frame = decodeVideoFrame(event.data);
        if (!frame) {
          return;
        }

        // Revoke previous blob URL to avoid memory leaks
        if (lastBlobUrlRef.current) {
          URL.revokeObjectURL(lastBlobUrlRef.current);
        }

        // Create new blob URL
        const blobUrl = videoFrameToBlobUrl(frame);
        lastBlobUrlRef.current = blobUrl;

        setVideoFrame(blobUrl, frame.timestamp_ms);

        // Update FPS counter
        frameCountRef.current++;
        const now = performance.now();
        const elapsed = now - lastFpsUpdateRef.current;

        if (elapsed >= 1000) {
          const fps = (frameCountRef.current / elapsed) * 1000;
          setVideoFps(Math.round(fps));
          frameCountRef.current = 0;
          lastFpsUpdateRef.current = now;
        }
      };

      ws.onclose = () => {
        setVideoConnected(false);
        setVideoFps(0);

        // Reconnect after delay
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      ws.onerror = () => {
        setVideoConnected(false);
      };

      wsRef.current = ws;
    } catch {
      setVideoConnected(false);

      // Retry connection
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [videoUrl, setVideoConnected, setVideoFps, setVideoFrame]);

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

    // Clean up last blob URL
    if (lastBlobUrlRef.current) {
      URL.revokeObjectURL(lastBlobUrlRef.current);
      lastBlobUrlRef.current = null;
    }

    setVideoConnected(false);
    setVideoFrame(null, 0);
  }, [setVideoConnected, setVideoFrame]);

  // Connect when component mounts (TeleopScreen), disconnect on unmount
  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
    // Note: we intentionally only run this on mount/unmount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    connect,
    disconnect,
  };
}
