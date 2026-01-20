/**
 * WebRTC-based rover connection hook.
 *
 * Uses WebRTC DataChannels for low-latency, unreliable command delivery.
 * This eliminates TCP's head-of-line blocking that causes jerky teleop.
 *
 * Architecture:
 * - WebSocket for signaling only (SDP offer/answer, ICE candidates)
 * - DataChannel "commands" for teleop commands (unreliable, unordered)
 * - DataChannel "telemetry" for rover state (unreliable)
 */

import { useEffect, useRef, useCallback } from "react";
import { useConsoleStore } from "@/store";
import {
  encodeTwist,
  encodeEStop,
  encodeHeartbeat,
  encodeTool,
  decodeTelemetry,
  telemetryFromDecoded,
  decodeVideoFrame,
  videoFrameToBlobUrl,
} from "@/lib/protocol";

const RECONNECT_DELAY_MS = 2000;
const COMMAND_INTERVAL_MS = 10; // 100Hz - matches rover control loop
const HEARTBEAT_INTERVAL_MS = 100;
const INPUT_UPDATE_INTERVAL_MS = 8; // 120Hz input polling - faster than command rate

const SPEED_NORMAL = 2.0;
const SPEED_BOOST = 5.0;
const MAX_ANGULAR_VEL = 1.5;
const TOOL_DEADZONE = 0.01;

// Track page visibility for safety
let isPageVisible = typeof document !== "undefined" ? !document.hidden : true;

// Signaling message types (must match Rust server)
interface SignalingMessage {
  type: "offer" | "answer" | "candidate";
  data: string | IceCandidate;
}

interface IceCandidate {
  candidate: string;
  sdpMid?: string;
  sdpMLineIndex?: number;
}

export function useRoverConnectionRtc() {
  const {
    roverAddress,
    setConnected,
    setLatency,
    updateTelemetry,
    setVideoConnected,
    setVideoFps,
    setVideoFrame,
  } = useConsoleStore();

  const peerConnectionRef = useRef<RTCPeerConnection | null>(null);
  const commandChannelRef = useRef<RTCDataChannel | null>(null);
  const signalingWsRef = useRef<WebSocket | null>(null);
  const commandIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null
  );
  const heartbeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(
    null
  );
  const inputIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const lastSendTimeRef = useRef<number>(0);
  const connectRef = useRef<() => void>(() => {});

  // Video frame tracking
  const videoFrameCountRef = useRef(0);
  const lastVideoFpsUpdateRef = useRef(0);
  const lastVideoBlobUrlRef = useRef<string | null>(null);

  // Current input state (updated from main thread)
  const currentInputRef = useRef({
    linear: 0,
    angular: 0,
    boost: false,
    estop: false,
    toolAxis: 0,
    actionA: false,
    actionB: false,
  });

  const clearIntervals = useCallback(() => {
    if (commandIntervalRef.current) {
      clearInterval(commandIntervalRef.current);
      commandIntervalRef.current = null;
    }
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
    if (inputIntervalRef.current) {
      clearInterval(inputIntervalRef.current);
      inputIntervalRef.current = null;
    }
  }, []);

  const sendCommands = useCallback(() => {
    const channel = commandChannelRef.current;
    if (!channel || channel.readyState !== "open") return;

    const input = currentInputRef.current;

    // E-Stop takes priority
    if (input.estop) {
      channel.send(encodeEStop());
      return;
    }

    // Safety: send zero velocity when page hidden
    if (!isPageVisible) {
      channel.send(encodeTwist(0, 0, false));
      return;
    }

    // Send velocity command
    const speedMultiplier = input.boost ? SPEED_BOOST : SPEED_NORMAL;
    const linear = input.linear * speedMultiplier;
    const angular = input.angular * MAX_ANGULAR_VEL;
    channel.send(encodeTwist(linear, angular, input.boost));

    // Send tool command if active
    if (
      Math.abs(input.toolAxis) > TOOL_DEADZONE ||
      input.actionA ||
      input.actionB
    ) {
      channel.send(encodeTool(input.toolAxis, 0, input.actionA, input.actionB));
    }
  }, []);

  const connect = useCallback(() => {
    // Clean up existing connection
    if (peerConnectionRef.current) {
      peerConnectionRef.current.close();
      peerConnectionRef.current = null;
    }
    if (signalingWsRef.current) {
      signalingWsRef.current.close();
      signalingWsRef.current = null;
    }
    clearIntervals();

    // Convert rover address to RTC signaling address
    // ws://host:4850 -> ws://host:4852
    // Also resolve Tailscale MagicDNS hostnames to IPs (browsers can't resolve them)
    const tailscaleHosts: Record<string, string> = {
      "frog-0": "100.127.211.5",
      // Add more rovers here as needed
    };

    let rtcAddress = roverAddress.replace(/:4850\b/, ":4852");

    // Replace Tailscale hostnames with IPs
    for (const [hostname, ip] of Object.entries(tailscaleHosts)) {
      rtcAddress = rtcAddress.replace(`://${hostname}:`, `://${ip}:`);
    }

    console.log(`[WebRTC] Connecting to signaling server: ${rtcAddress}`);

    try {
      // Create signaling WebSocket
      const ws = new WebSocket(rtcAddress);
      signalingWsRef.current = ws;

      // Create peer connection with STUN server
      const pc = new RTCPeerConnection({
        iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
      });
      peerConnectionRef.current = pc;

      // Create command channel (unreliable, unordered - like UDP)
      const commandChannel = pc.createDataChannel("commands", {
        ordered: false,
        maxRetransmits: 0, // Truly unreliable
      });
      commandChannelRef.current = commandChannel;
      commandChannel.binaryType = "arraybuffer";

      commandChannel.onopen = () => {
        console.log("[WebRTC] Command channel opened");
        setConnected(true);
        lastSendTimeRef.current = performance.now();

        // Start command loop
        commandIntervalRef.current = setInterval(
          sendCommands,
          COMMAND_INTERVAL_MS
        );

        // Start heartbeat loop
        heartbeatIntervalRef.current = setInterval(() => {
          if (commandChannelRef.current?.readyState === "open") {
            commandChannelRef.current.send(encodeHeartbeat());
          }
        }, HEARTBEAT_INTERVAL_MS);

        // Start polling input state
        inputIntervalRef.current = setInterval(() => {
          const { input } = useConsoleStore.getState();
          currentInputRef.current = {
            linear: input.linear,
            angular: input.angular,
            boost: input.boost,
            estop: input.estop,
            toolAxis: input.toolAxis,
            actionA: input.actionA,
            actionB: input.actionB,
          };
        }, INPUT_UPDATE_INTERVAL_MS);
      };

      commandChannel.onclose = () => {
        console.log("[WebRTC] Command channel closed");
        setConnected(false);
        clearIntervals();
        // Reconnect after delay
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      // Handle incoming data channels (created by server)
      pc.ondatachannel = (event) => {
        const channel = event.channel;
        console.log(`[WebRTC] Received data channel: ${channel.label}`);

        if (channel.label === "telemetry") {
          channel.binaryType = "arraybuffer";
          channel.onmessage = (msgEvent) => {
            if (msgEvent.data instanceof ArrayBuffer) {
              const decoded = decodeTelemetry(msgEvent.data);
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
          };
        } else if (channel.label === "video") {
          channel.binaryType = "arraybuffer";
          setVideoConnected(true);
          videoFrameCountRef.current = 0;
          lastVideoFpsUpdateRef.current = performance.now();

          channel.onmessage = (msgEvent) => {
            if (!(msgEvent.data instanceof ArrayBuffer)) return;

            const frame = decodeVideoFrame(msgEvent.data);
            if (!frame) return;

            // Revoke previous blob URL to avoid memory leaks
            if (lastVideoBlobUrlRef.current) {
              URL.revokeObjectURL(lastVideoBlobUrlRef.current);
            }

            // Create new blob URL
            const blobUrl = videoFrameToBlobUrl(frame);
            lastVideoBlobUrlRef.current = blobUrl;
            setVideoFrame(blobUrl, frame.timestamp_ms);

            // Update FPS counter
            videoFrameCountRef.current++;
            const now = performance.now();
            const elapsed = now - lastVideoFpsUpdateRef.current;

            if (elapsed >= 1000) {
              const fps = (videoFrameCountRef.current / elapsed) * 1000;
              setVideoFps(Math.round(fps));
              videoFrameCountRef.current = 0;
              lastVideoFpsUpdateRef.current = now;
            }
          };

          channel.onclose = () => {
            console.log("[WebRTC] Video channel closed");
            setVideoConnected(false);
            setVideoFps(0);
          };
        }
      };

      // Handle ICE candidates
      pc.onicecandidate = (event) => {
        if (event.candidate && ws.readyState === WebSocket.OPEN) {
          const msg: SignalingMessage = {
            type: "candidate",
            data: {
              candidate: event.candidate.candidate,
              sdpMid: event.candidate.sdpMid ?? undefined,
              sdpMLineIndex: event.candidate.sdpMLineIndex ?? undefined,
            },
          };
          ws.send(JSON.stringify(msg));
        }
      };

      // Handle connection state changes
      pc.onconnectionstatechange = () => {
        console.log(`[WebRTC] Connection state: ${pc.connectionState}`);
        if (
          pc.connectionState === "failed" ||
          pc.connectionState === "disconnected"
        ) {
          setConnected(false);
          clearIntervals();
          reconnectTimeoutRef.current = setTimeout(() => {
            connectRef.current();
          }, RECONNECT_DELAY_MS);
        }
      };

      // WebSocket signaling handlers
      ws.onopen = async () => {
        console.log("[WebRTC] Signaling WebSocket connected");

        try {
          // Create and send offer
          const offer = await pc.createOffer();
          await pc.setLocalDescription(offer);

          const msg: SignalingMessage = {
            type: "offer",
            data: offer.sdp!,
          };
          ws.send(JSON.stringify(msg));
          console.log("[WebRTC] Sent SDP offer");
        } catch (err) {
          console.error("[WebRTC] Failed to create offer:", err);
        }
      };

      ws.onmessage = async (event) => {
        try {
          const msg = JSON.parse(event.data) as SignalingMessage;

          switch (msg.type) {
            case "answer":
              console.log("[WebRTC] Received SDP answer");
              await pc.setRemoteDescription({
                type: "answer",
                sdp: msg.data as string,
              });
              break;

            case "candidate": {
              const candidate = msg.data as IceCandidate;
              console.log("[WebRTC] Received ICE candidate");
              await pc.addIceCandidate({
                candidate: candidate.candidate,
                sdpMid: candidate.sdpMid,
                sdpMLineIndex: candidate.sdpMLineIndex,
              });
              break;
            }
          }
        } catch (err) {
          console.error("[WebRTC] Signaling message error:", err);
        }
      };

      ws.onclose = () => {
        console.log("[WebRTC] Signaling WebSocket closed");
        // Don't reconnect here - let the PC connection state handler do it
      };

      ws.onerror = (err) => {
        console.error("[WebRTC] Signaling WebSocket error:", err);
        setConnected(false);
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };
    } catch (err) {
      console.error("[WebRTC] Connection error:", err);
      setConnected(false);
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [
    roverAddress,
    setConnected,
    setLatency,
    updateTelemetry,
    setVideoConnected,
    setVideoFps,
    setVideoFrame,
    clearIntervals,
    sendCommands,
  ]);

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

    if (peerConnectionRef.current) {
      peerConnectionRef.current.close();
      peerConnectionRef.current = null;
    }

    if (signalingWsRef.current) {
      signalingWsRef.current.close();
      signalingWsRef.current = null;
    }

    // Clean up video blob URL
    if (lastVideoBlobUrlRef.current) {
      URL.revokeObjectURL(lastVideoBlobUrlRef.current);
      lastVideoBlobUrlRef.current = null;
    }

    commandChannelRef.current = null;
    setConnected(false);
    setVideoConnected(false);
    setVideoFrame(null, 0);
    setVideoFps(0);
  }, [clearIntervals, setConnected, setVideoConnected, setVideoFrame, setVideoFps]);

  // Connect when rover address changes
  useEffect(() => {
    // Don't connect to default localhost - wait for real rover address
    if (roverAddress === "ws://localhost:4850") {
      return;
    }

    connect();

    // Track page visibility
    const handleVisibilityChange = () => {
      isPageVisible = !document.hidden;
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      disconnect();
    };
  }, [roverAddress, connect, disconnect]);

  return {
    connect,
    disconnect,
    sendEStop: useCallback(() => {
      currentInputRef.current.estop = true;
      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeEStop());
      }
    }, []),
    sendEStopRelease: useCallback(() => {
      currentInputRef.current.estop = false;
      // Note: E-Stop release requires a special message
      console.warn(
        "E-Stop release: clearing estop state, but dedicated message not implemented"
      );
    }, []),
  };
}
