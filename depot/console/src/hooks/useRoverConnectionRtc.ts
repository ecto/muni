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
  encodeEStopRelease,
  encodeHeartbeat,
  encodeTool,
  encodeSetMode,
  decodeTelemetry,
  telemetryFromDecoded,
  decodeVideoFrame,
  videoFrameToBlobUrl,
} from "@/lib/protocol";
import { Mode } from "@/lib/types";

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
    rtcAddress,
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
    enable: false,
    disable: false,
    toolAxis: 0,
    actionA: false,
    actionB: false,
  });

  // Track previous enable/disable state for edge detection
  const prevEnableRef = useRef(false);
  const prevDisableRef = useRef(false);

  // Track if this console is the active operator (has control)
  // Only operators can send commands - observers can watch but not control
  const isOperatorRef = useRef(false);

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

    // Only send commands if we are the active operator
    // This prevents observers from accidentally sending commands
    if (!isOperatorRef.current) {
      return;
    }

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

    // rtcAddress comes directly from discovery - no conversion needed
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
          // Update input state
          currentInputRef.current = {
            linear: input.linear,
            angular: input.angular,
            boost: input.boost,
            estop: input.estop,
            enable: input.enable,
            disable: input.disable,
            toolAxis: input.toolAxis,
            actionA: input.actionA,
            actionB: input.actionB,
          };

          // Handle enable rising edge (Enter key)
          const enableRising = input.enable && !prevEnableRef.current;
          prevEnableRef.current = input.enable;
          if (enableRising && commandChannelRef.current?.readyState === "open") {
            // Claim operator and enable teleop
            isOperatorRef.current = true;
            console.log("[WebRTC] Enable key pressed - claiming operator control");
            commandChannelRef.current.send(encodeSetMode(Mode.Idle));
            setTimeout(() => {
              if (commandChannelRef.current?.readyState === "open") {
                commandChannelRef.current.send(encodeSetMode(Mode.Teleop));
              }
            }, 50);
          }

          // Handle disable rising edge (Backspace key)
          const disableRising = input.disable && !prevDisableRef.current;
          prevDisableRef.current = input.disable;
          if (disableRising && isOperatorRef.current && commandChannelRef.current?.readyState === "open") {
            // Release operator and disable
            isOperatorRef.current = false;
            console.log("[WebRTC] Disable key pressed - releasing operator control");
            commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
          }
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
          console.log("[WebRTC] Telemetry channel established");
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
              } else {
                console.warn("[WebRTC] Failed to decode telemetry, size:", msgEvent.data.byteLength);
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

      ws.onclose = (event) => {
        console.log("[WebRTC] Signaling WebSocket closed, code:", event.code, "reason:", event.reason);
        // Don't reconnect here - let the PC connection state handler do it
      };

      ws.onerror = () => {
        console.error("[WebRTC] Signaling WebSocket error - connection refused or unreachable");
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
    rtcAddress,
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

    // Release operator status on disconnect
    isOperatorRef.current = false;

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

  // Connect when RTC address changes
  useEffect(() => {
    console.log("[WebRTC] rtcAddress changed to:", rtcAddress);
    // Don't connect to default localhost - wait for real rover address from discovery
    if (rtcAddress === "ws://localhost:4852") {
      console.log("[WebRTC] Skipping connection - still default localhost");
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
  }, [rtcAddress, connect, disconnect]);

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
      if (commandChannelRef.current?.readyState === "open") {
        console.log("[WebRTC] Sending E-Stop Release");
        commandChannelRef.current.send(encodeEStopRelease());
      }
    }, []),
    sendEnable: useCallback(() => {
      const channel = commandChannelRef.current;
      if (channel?.readyState !== "open") {
        console.warn("[WebRTC] Cannot send - channel not open");
        return;
      }
      // Claim operator status - we're taking control
      isOperatorRef.current = true;
      console.log("[WebRTC] Claiming operator control");

      // State machine requires: Disabled -> Idle -> Teleop
      // Send Enable (Idle) first, then TeleopCommand (Teleop)
      channel.send(encodeSetMode(Mode.Idle));
      // Small delay to ensure state machine processes first command
      setTimeout(() => {
        if (commandChannelRef.current?.readyState === "open") {
          commandChannelRef.current.send(encodeSetMode(Mode.Teleop));
          console.log("[WebRTC] Enable sequence sent");
        }
      }, 50);
    }, []),
    sendDisable: useCallback(() => {
      // Release operator status
      isOperatorRef.current = false;
      console.log("[WebRTC] Releasing operator control");

      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
      }
    }, []),
  };
}
