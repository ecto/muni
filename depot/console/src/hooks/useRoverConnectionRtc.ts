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
  encodeLidarToggle,
  decodeTelemetry,
  telemetryFromDecoded,
  decodeVideoFrame,
  videoFrameToBlobUrl,
  decodePointCloud,
  decodeCostmap,
  decodeObstacles,
} from "@/lib/protocol";
import { pushSnapshotDirect } from "@/lib/interpolation";
import { Mode } from "@/lib/types";
import { setCameraFrame as setMutableCameraFrame, getCameraCount } from "@/lib/videoFrameStore";
import { setPointCloudData, clearPointCloudData } from "@/lib/pointCloudStore";
import { setCostmapData, clearCostmapData } from "@/lib/costmapStore";
import { setObstacleData, clearObstacleData } from "@/lib/obstacleStore";

const RECONNECT_DELAY_MS = 2000;
const COMMAND_INTERVAL_MS = 10; // 100Hz - matches rover control loop
const HEARTBEAT_INTERVAL_MS = 100;
const INPUT_UPDATE_INTERVAL_MS = 8; // 120Hz input polling - faster than command rate
const TELEMETRY_TIMEOUT_MS = 5000; // Force reconnect if no telemetry for 5s

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
  const rtcAddress = useConsoleStore((s) => s.rtcAddress);
  const setConnecting = useConsoleStore((s) => s.setConnecting);
  const setConnected = useConsoleStore((s) => s.setConnected);
  const setLatency = useConsoleStore((s) => s.setLatency);
  const updateTelemetry = useConsoleStore((s) => s.updateTelemetry);
  const setVideoConnected = useConsoleStore((s) => s.setVideoConnected);
  const setVideoFps = useConsoleStore((s) => s.setVideoFps);
  const setVideoFrame = useConsoleStore((s) => s.setVideoFrame);
  const updateChannelMetrics = useConsoleStore((s) => s.updateChannelMetrics);
  const resetChannelMetrics = useConsoleStore((s) => s.resetChannelMetrics);

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

  // Telemetry throttling - drop messages that arrive faster than expected rate
  // At 50Hz telemetry, we expect ~20ms between messages. Accept if >= 15ms has passed.
  const lastTelemetryTimeRef = useRef<number>(0);
  const MIN_TELEMETRY_INTERVAL_MS = 15;

  // Throttle React state updates to reduce re-renders (10Hz is enough for UI)
  // Interpolation buffer still receives all messages for smooth 3D rendering
  const lastReactUpdateRef = useRef<number>(0);
  const REACT_UPDATE_INTERVAL_MS = 100; // 10Hz for UI updates

  // Throttle rover roster updates separately (2Hz — battery/mode change slowly)
  const lastRoverUpdateRef = useRef<number>(0);
  const ROVER_UPDATE_INTERVAL_MS = 500; // 2Hz for fleet roster

  // Channel metrics tracking
  const channelStatsRef = useRef<Map<string, { count: number; lastTime: number }>>(
    new Map([
      ["telemetry", { count: 0, lastTime: 0 }],
      ["video", { count: 0, lastTime: 0 }],
      ["pointcloud", { count: 0, lastTime: 0 }],
      ["costmap", { count: 0, lastTime: 0 }],
      ["obstacles", { count: 0, lastTime: 0 }],
    ])
  );
  const metricsIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastMetricsUpdateRef = useRef<number>(performance.now());

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

  // Track previous mode for modeChangedAt detection
  const prevModeRef = useRef<number>(Mode.Idle);

  // Track if this console is the active operator (has control)
  // Only operators can send commands - observers can watch but not control
  const isOperatorRef = useRef(false);

  // Telemetry watchdog: timestamp of last received telemetry message.
  // If no telemetry arrives for TELEMETRY_TIMEOUT_MS, the DataChannel is stale
  // and we force a reconnection (browser readyState can lie about dead SCTP).
  const lastTelemetryReceivedRef = useRef<number>(0);

  // Pending mode: tracks a SetMode we're trying to deliver over the unreliable
  // DataChannel. The heartbeat loop resends until telemetry confirms the rover
  // actually changed mode (or we disconnect).
  const pendingModeRef = useRef<Mode | null>(null);

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
    if (metricsIntervalRef.current) {
      clearInterval(metricsIntervalRef.current);
      metricsIntervalRef.current = null;
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
      channel.send(encodeTool(input.toolAxis, input.toolAxis, input.actionA, input.actionB));
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

      // Create command channel with negotiated ID for Safari/webrtc-rs compatibility
      const commandChannel = pc.createDataChannel("commands", {
        ordered: true,
        negotiated: true,
        id: 0,  // Fixed channel ID agreed upon by both sides
      });
      commandChannelRef.current = commandChannel;
      commandChannel.binaryType = "arraybuffer";

      commandChannel.onopen = () => {
        console.log("[WebRTC] Command channel opened");
        setConnected(true);
        lastSendTimeRef.current = performance.now();
        lastTelemetryTimeRef.current = 0; // Reset throttle on reconnect
        lastTelemetryReceivedRef.current = 0; // Reset watchdog — prevent stale timestamp from previous connection
        pendingModeRef.current = null; // Clear any pending mode from previous connection

        // Mark rover online in fleet roster — WebRTC proves reachability
        const { selectedRoverId, updateRover } = useConsoleStore.getState();
        if (selectedRoverId) {
          updateRover(selectedRoverId, { online: true, lastSeen: Date.now() });
        }

        // Start command loop
        commandIntervalRef.current = setInterval(
          sendCommands,
          COMMAND_INTERVAL_MS
        );

        // Start heartbeat loop (also checks for stale DataChannel and retries SetMode)
        heartbeatIntervalRef.current = setInterval(() => {
          if (commandChannelRef.current?.readyState === "open") {
            commandChannelRef.current.send(encodeHeartbeat());

            // Retry pending SetMode: if we sent a SetMode but telemetry hasn't
            // confirmed the mode change yet, resend. The unreliable DataChannel
            // (maxRetransmits: 0) can silently drop SCTP packets.
            if (pendingModeRef.current !== null) {
              commandChannelRef.current.send(encodeSetMode(pendingModeRef.current));
            }
          }

          // Telemetry watchdog: if we've been connected but haven't received
          // telemetry for TELEMETRY_TIMEOUT_MS, the DataChannel is stale.
          // Browser readyState can remain "open" even after SCTP transport dies.
          const lastRecv = lastTelemetryReceivedRef.current;
          if (lastRecv > 0 && performance.now() - lastRecv > TELEMETRY_TIMEOUT_MS) {
            console.warn("[WebRTC] Telemetry watchdog: no data for 5s, forcing reconnect");
            lastTelemetryReceivedRef.current = 0; // Prevent repeated triggers
            peerConnectionRef.current?.close();
            signalingWsRef.current?.close();
            setConnected(false);
            clearIntervals();
            reconnectTimeoutRef.current = setTimeout(() => {
              connectRef.current();
            }, RECONNECT_DELAY_MS);
          }
        }, HEARTBEAT_INTERVAL_MS);

        // Start metrics calculation interval (1Hz)
        lastMetricsUpdateRef.current = performance.now();
        metricsIntervalRef.current = setInterval(() => {
          const now = performance.now();
          const elapsed = (now - lastMetricsUpdateRef.current) / 1000; // seconds
          lastMetricsUpdateRef.current = now;

          channelStatsRef.current.forEach((stats, channelName) => {
            const messagesPerSec = elapsed > 0 ? stats.count / elapsed : 0;
            stats.count = 0; // Reset count for next interval

            // Get existing history and append new sample (keep last 30)
            const existing = useConsoleStore.getState().channelMetrics.get(channelName);
            const history = existing?.history ?? [];
            const newHistory = [...history, messagesPerSec].slice(-30);

            updateChannelMetrics(channelName, {
              messagesPerSec,
              lastMessageTime: stats.lastTime,
              history: newHistory,
            });
          });
        }, 1000);

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
            pendingModeRef.current = Mode.Teleop;
            commandChannelRef.current.send(encodeSetMode(Mode.Teleop));
          }

          // Handle disable rising edge (Backspace key)
          const disableRising = input.disable && !prevDisableRef.current;
          prevDisableRef.current = input.disable;
          if (disableRising && isOperatorRef.current && commandChannelRef.current?.readyState === "open") {
            // Release operator and disable
            isOperatorRef.current = false;
            pendingModeRef.current = Mode.Disabled;
            commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
          }
        }, INPUT_UPDATE_INTERVAL_MS);

        // Handle enable key held during connection (fixes race condition where
        // user presses Enter before WebRTC is ready)
        const { input, pointCloudEnabled } = useConsoleStore.getState();
        if (input.enable && !isOperatorRef.current) {
          isOperatorRef.current = true;
          pendingModeRef.current = Mode.Teleop;
          commandChannelRef.current!.send(encodeSetMode(Mode.Teleop));
          prevEnableRef.current = true;
        }

        // Sync LiDAR state on connection - rover starts with streaming disabled,
        // but console may have it enabled from a previous session
        if (pointCloudEnabled) {
          console.log("[WebRTC] Syncing LiDAR state: enabled");
          commandChannelRef.current!.send(encodeLidarToggle(true));
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
          console.log("[WebRTC] Telemetry channel received");
          channel.binaryType = "arraybuffer";
          channel.onmessage = (msgEvent) => {
            if (msgEvent.data instanceof ArrayBuffer) {
              const now = performance.now();

              // Throttle: only process if enough time has passed since last message
              // This drops stale queued messages during backpressure (tab backgrounded, GC pause)
              if (now - lastTelemetryTimeRef.current < MIN_TELEMETRY_INTERVAL_MS) {
                return; // Drop - too soon since last processed message
              }
              lastTelemetryTimeRef.current = now;
              lastTelemetryReceivedRef.current = now;

              // Track channel metrics
              const stats = channelStatsRef.current.get("telemetry");
              if (stats) {
                stats.count++;
                stats.lastTime = now;
              }

              const decoded = decodeTelemetry(msgEvent.data);
              if (decoded) {
                // Clear pending mode once telemetry confirms the rover changed
                if (pendingModeRef.current !== null && decoded.mode === pendingModeRef.current) {
                  pendingModeRef.current = null;
                }

                // Always push to interpolation buffer at full rate for smooth 3D rendering
                pushSnapshotDirect({
                  serverTimestamp: decoded.timestamp_us,
                  receivedAt: now,
                  pose: decoded.pose,
                  velocity: {
                    linear: decoded.cmd_velocity.linear,
                    angular: decoded.cmd_velocity.angular,
                  },
                });

                // Throttle React state updates to 10Hz to reduce re-renders
                // The 3D scene reads from interpolation buffer directly at 60fps
                if (now - lastReactUpdateRef.current >= REACT_UPDATE_INTERVAL_MS) {
                  lastReactUpdateRef.current = now;
                  const telemetry = telemetryFromDecoded(decoded);
                  const latency = Math.round(now - lastSendTimeRef.current);
                  lastSendTimeRef.current = now;
                  setLatency(latency);

                  const modeChanged = decoded.mode !== prevModeRef.current;
                  if (modeChanged) prevModeRef.current = decoded.mode;

                  updateTelemetry({
                    ...telemetry,
                    connected: true,
                    latency_ms: latency,
                    ...(modeChanged && { modeChangedAt: Date.now() }),
                  });

                  // Sync to fleet roster at 2Hz — battery/mode change slowly
                  if (now - lastRoverUpdateRef.current >= ROVER_UPDATE_INTERVAL_MS) {
                    lastRoverUpdateRef.current = now;
                    const { selectedRoverId, updateRover } = useConsoleStore.getState();
                    if (selectedRoverId) {
                      updateRover(selectedRoverId, {
                        online: true,
                        batteryVoltage: telemetry.power.battery_voltage,
                        mode: telemetry.mode,
                        lastSeen: Date.now(),
                      });
                    }
                  }
                }
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

            // Track channel metrics
            const stats = channelStatsRef.current.get("video");
            if (stats) {
              stats.count++;
              stats.lastTime = performance.now();
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

            // Store in mutable store (bypasses React for performance)
            setMutableCameraFrame(frame.cameraId, blobUrl, frame.timestamp_ms);

            // Log first frame received for debugging (only once per connection)
            if (!firstFrameLoggedRef.current) {
              console.log(`[WebRTC] First video frame received: camera=${frame.cameraId}, ${frame.width}x${frame.height}, ${frame.jpegData.length} bytes`);
              firstFrameLoggedRef.current = true;
            }

            // Update camera count in Zustand store (for UI badge) - infrequent, only on change
            const currentCount = getCameraCount();
            if (useConsoleStore.getState().cameraCount !== currentCount) {
              useConsoleStore.getState().setCameraCount(currentCount);
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
        } else if (channel.label === "pointcloud") {
          console.log("[WebRTC] Pointcloud channel received");
          channel.binaryType = "arraybuffer";

          channel.onopen = () => {
            console.log("[WebRTC] Pointcloud channel opened");
          };

          channel.onmessage = (msgEvent) => {
            if (!(msgEvent.data instanceof ArrayBuffer)) {
              console.warn("[WebRTC] Pointcloud message not ArrayBuffer");
              return;
            }

            // Track channel metrics
            const stats = channelStatsRef.current.get("pointcloud");
            if (stats) {
              stats.count++;
              stats.lastTime = performance.now();
            }

            const cloud = decodePointCloud(msgEvent.data);
            if (cloud) {
              setPointCloudData(cloud.points, cloud.reflectivity, cloud.tag);
            }
          };

          channel.onclose = () => {
            console.log("[WebRTC] Pointcloud channel closed");
          };
        } else if (channel.label === "costmap") {
          console.log("[WebRTC] Costmap channel received");
          channel.binaryType = "arraybuffer";

          channel.onopen = () => {
            console.log("[WebRTC] Costmap channel opened");
          };

          channel.onmessage = (msgEvent) => {
            if (!(msgEvent.data instanceof ArrayBuffer)) {
              console.warn("[WebRTC] Costmap message not ArrayBuffer");
              return;
            }

            // Track channel metrics
            const stats = channelStatsRef.current.get("costmap");
            if (stats) {
              stats.count++;
              stats.lastTime = performance.now();
            }

            const costmap = decodeCostmap(msgEvent.data);
            if (costmap) {
              setCostmapData(
                costmap.cells,
                costmap.width,
                costmap.height,
                costmap.resolution,
                costmap.originX,
                costmap.originY,
              );
            }
          };

          channel.onclose = () => {
            console.log("[WebRTC] Costmap channel closed");
            clearCostmapData();
          };
        } else if (channel.label === "obstacles") {
          console.log("[WebRTC] Obstacles channel received");
          channel.binaryType = "arraybuffer";

          channel.onopen = () => {
            console.log("[WebRTC] Obstacles channel opened");
          };

          channel.onmessage = (msgEvent) => {
            if (!(msgEvent.data instanceof ArrayBuffer)) {
              console.warn("[WebRTC] Obstacles message not ArrayBuffer");
              return;
            }

            // Track channel metrics
            const stats = channelStatsRef.current.get("obstacles");
            if (stats) {
              stats.count++;
              stats.lastTime = performance.now();
            }

            const obstacles = decodeObstacles(msgEvent.data);
            if (obstacles) {
              setObstacleData(obstacles);
            }
          };

          channel.onclose = () => {
            console.log("[WebRTC] Obstacles channel closed");
            clearObstacleData();
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
    updateChannelMetrics,
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
    pendingModeRef.current = null;

    // Reset telemetry throttle
    lastTelemetryTimeRef.current = 0;

    // Reset channel metrics
    resetChannelMetrics();

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
    clearPointCloudData();
    clearCostmapData();
    clearObstacleData();
  }, [clearIntervals, setConnected, setVideoConnected, setVideoFrame, setVideoFps, resetChannelMetrics]);

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
      pendingModeRef.current = Mode.Teleop;

      // Rover boots into Idle — go straight to Teleop
      channel.send(encodeSetMode(Mode.Teleop));
    }, []),
    sendDisable: useCallback(() => {
      // Release operator status
      isOperatorRef.current = false;
      pendingModeRef.current = Mode.Disabled;

      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeSetMode(Mode.Disabled));
      }
    }, []),
    toggleLidar: useCallback((enabled: boolean) => {
      console.log(`[WebRTC] Toggling LiDAR: ${enabled}`);
      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeLidarToggle(enabled));
        console.log("[WebRTC] LiDAR toggle command sent");
      } else {
        console.warn("[WebRTC] Cannot toggle LiDAR - command channel not open");
      }
    }, []),
    sendSleep: useCallback(() => {
      console.log("[WebRTC] sendSleep called");
      // Release operator status (going to sleep)
      isOperatorRef.current = false;
      pendingModeRef.current = Mode.Sleep;

      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeSetMode(Mode.Sleep));
        console.log("[WebRTC] Sleep command sent");
      } else {
        console.warn("[WebRTC] Cannot send Sleep - command channel not open");
      }
    }, []),
    sendWake: useCallback(() => {
      console.log("[WebRTC] sendWake called");
      const channel = commandChannelRef.current;
      if (channel?.readyState !== "open") {
        console.warn("[WebRTC] Cannot send Wake - command channel not open");
        return;
      }
      // Claim operator status - we're waking up
      isOperatorRef.current = true;
      pendingModeRef.current = Mode.Idle;

      // Send Idle to wake from Sleep (state machine: Sleep -> Wake -> Idle)
      channel.send(encodeSetMode(Mode.Idle));
      console.log("[WebRTC] Wake command sent (SetMode Idle)");
    }, []),
    sendAutonomous: useCallback(() => {
      const channel = commandChannelRef.current;
      if (channel?.readyState !== "open") return;
      // Don't claim operator — autonomous mode drives itself, no Twist commands from console
      isOperatorRef.current = false;
      pendingModeRef.current = Mode.Autonomous;
      channel.send(encodeSetMode(Mode.Autonomous));
    }, []),
    sendStopAutonomy: useCallback(() => {
      pendingModeRef.current = Mode.Idle;
      if (commandChannelRef.current?.readyState === "open") {
        commandChannelRef.current.send(encodeSetMode(Mode.Idle));
      }
    }, []),
  };
}
