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
import { pushSnapshotDirect } from "@/lib/interpolation";
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
    setConnecting,
    setConnected,
    setLatency,
    updateTelemetry,
    setVideoConnected,
    setVideoFps,
    setVideoFrame,
    setCameraFrame,
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

  // Video frame tracking (per-camera)
  const videoFrameCountRef = useRef(0);
  const lastVideoFpsUpdateRef = useRef(0);
  const cameraBlobUrlsRef = useRef<Map<number, string>>(new Map());
  const firstFrameLoggedRef = useRef(false);

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

    // Mark as connecting
    setConnecting(true);

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

      // Create command channel (unordered for true UDP semantics)
      // Critical commands (SetMode, EStop) use MUST_ACK flag with retry logic
      const commandChannel = pc.createDataChannel("commands", {
        ordered: false,
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

          // Handle enable rising edge (Enter key) - only if not already operator
          const enableRising = input.enable && !prevEnableRef.current;
          prevEnableRef.current = input.enable;
          if (enableRising && !isOperatorRef.current && commandChannelRef.current?.readyState === "open") {
            // Claim operator and enable teleop
            isOperatorRef.current = true;
            commandChannelRef.current.send(encodeSetMode(Mode.Idle));
            commandChannelRef.current.send(encodeSetMode(Mode.Teleop));
          }

          // Handle disable rising edge (Backspace key)
          const disableRising = input.disable && !prevDisableRef.current;
          prevDisableRef.current = input.disable;
          if (disableRising && isOperatorRef.current && commandChannelRef.current?.readyState === "open") {
            // Release operator and disable
            isOperatorRef.current = false;
            commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
          }
        }, INPUT_UPDATE_INTERVAL_MS);

        // Handle enable key held during connection (fixes race condition where
        // user presses Enter before WebRTC is ready)
        const { input } = useConsoleStore.getState();
        if (input.enable && !isOperatorRef.current) {
          isOperatorRef.current = true;
          commandChannelRef.current!.send(encodeSetMode(Mode.Idle));
          commandChannelRef.current!.send(encodeSetMode(Mode.Teleop));
          prevEnableRef.current = true;
        }
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

                // Push snapshot for interpolation (direct to buffer, no React updates)
                // Use cmd_velocity until firmware populates meas_velocity
                pushSnapshotDirect({
                  serverTimestamp: decoded.timestamp_us,
                  receivedAt: now,
                  pose: decoded.pose,
                  velocity: {
                    linear: decoded.cmd_velocity.linear,
                    angular: decoded.cmd_velocity.angular,
                  },
                });
              } else {
                console.warn("[WebRTC] Failed to decode telemetry, size:", msgEvent.data.byteLength);
              }
            }
          };
        } else if (channel.label === "video") {
          channel.binaryType = "arraybuffer";

          const setupVideoChannel = () => {
            setVideoConnected(true);
            videoFrameCountRef.current = 0;
            lastVideoFpsUpdateRef.current = performance.now();
            firstFrameLoggedRef.current = false;
          };

          // Channel might already be open or need to wait for open event
          if (channel.readyState === "open") {
            setupVideoChannel();
          } else {
            channel.onopen = () => setupVideoChannel();
          }

          channel.onmessage = (msgEvent) => {
            if (!(msgEvent.data instanceof ArrayBuffer)) {
              console.warn("[WebRTC] Video message not ArrayBuffer:", typeof msgEvent.data);
              return;
            }

            const frame = decodeVideoFrame(msgEvent.data);
            if (!frame) {
              console.warn("[WebRTC] Failed to decode video frame, size:", msgEvent.data.byteLength);
              return;
            }

            // Create new blob URL
            const blobUrl = videoFrameToBlobUrl(frame);

            // Revoke previous blob URL for this camera after a delay
            // (Image loading is asynchronous)
            const oldUrl = cameraBlobUrlsRef.current.get(frame.cameraId);
            if (oldUrl) {
              setTimeout(() => URL.revokeObjectURL(oldUrl), 200);
            }

            cameraBlobUrlsRef.current.set(frame.cameraId, blobUrl);
            setCameraFrame(frame.cameraId, blobUrl, frame.timestamp_ms);

            // Log first frame received for debugging (only once per connection)
            if (!firstFrameLoggedRef.current) {
              console.log(`[WebRTC] First video frame received: camera=${frame.cameraId}, ${frame.width}x${frame.height}, ${frame.jpegData.length} bytes`);
              firstFrameLoggedRef.current = true;
            }

            // Update FPS counter (total across all cameras)
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
        try {
          // Create and send offer
          const offer = await pc.createOffer();
          await pc.setLocalDescription(offer);

          const msg: SignalingMessage = {
            type: "offer",
            data: offer.sdp!,
          };
          ws.send(JSON.stringify(msg));
        } catch (err) {
          console.error("[WebRTC] Failed to create offer:", err);
        }
      };

      ws.onmessage = async (event) => {
        try {
          const msg = JSON.parse(event.data) as SignalingMessage;

          switch (msg.type) {
            case "answer":
              await pc.setRemoteDescription({
                type: "answer",
                sdp: msg.data as string,
              });
              break;

            case "candidate": {
              const candidate = msg.data as IceCandidate;
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
    setConnecting,
    setConnected,
    setLatency,
    updateTelemetry,
    setVideoConnected,
    setVideoFps,
    setVideoFrame,
    setCameraFrame,
    clearIntervals,
    sendCommands,
  ]);

  // Keep connectRef in sync (always points to latest connect function)
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

    commandChannelRef.current = null;
    setConnected(false);
    setVideoConnected(false);
    setVideoFrame(null, 0);
    setVideoFps(0);
  }, [clearIntervals, setConnected, setVideoConnected, setVideoFrame, setVideoFps]);

  // Stable disconnect ref for cleanup
  const disconnectRef = useRef(disconnect);
  useEffect(() => {
    disconnectRef.current = disconnect;
  }, [disconnect]);

  // Connect when RTC address changes
  // NOTE: Only depend on rtcAddress to avoid reconnection loops.
  // connect/disconnect are accessed via refs to get latest versions.
  useEffect(() => {
    // Don't connect to default localhost - wait for real rover address from discovery
    if (rtcAddress === "ws://localhost:4852") {
      return;
    }

    // Use ref to get current connect function (avoids stale closure)
    connectRef.current();

    // Track page visibility
    const handleVisibilityChange = () => {
      isPageVisible = !document.hidden;
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      disconnectRef.current();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rtcAddress]);

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
        commandChannelRef.current.send(encodeEStopRelease());
      }
    }, []),
    sendEnable: useCallback(() => {
      const channel = commandChannelRef.current;
      if (channel?.readyState !== "open") {
        return;
      }
      // Claim operator status - we're taking control
      isOperatorRef.current = true;

      // State machine requires: Disabled -> Idle -> Teleop
      // Send Enable (Idle) first, then TeleopCommand (Teleop)
      channel.send(encodeSetMode(Mode.Idle));

      // Small delay to ensure state machine processes first command
      setTimeout(() => {
        if (commandChannelRef.current?.readyState === "open") {
          commandChannelRef.current.send(encodeSetMode(Mode.Teleop));
        }
      }, 50);
    }, []),
    sendDisable: useCallback(() => {
      // Release operator status
      isOperatorRef.current = false;

      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
      }
    }, []),
  };
}
