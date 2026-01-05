# WebSocket Binary Protocols Reference

This document provides a complete reference for WebSocket binary protocols used between the depot console and BVR rovers.

## Protocol Overview

The console uses two separate WebSocket connections:
1. **Teleop Connection** (port 4850): Bidirectional command/telemetry
2. **Video Connection** (port 4851): Unidirectional JPEG frame stream

Both use **binary protocols** for performance (100Hz command rate, 30Hz video).

## Message Type IDs

| Range     | Direction       | Purpose                    |
|-----------|-----------------|----------------------------|
| 0x01-0x0F | Console → Rover | Commands                   |
| 0x10-0x1F | Rover → Console | Telemetry/Status          |
| 0x20-0x2F | Rover → Console | Video frames              |

## Teleop Connection (Port 4850)

### Connection Setup

```typescript
const ws = new WebSocket("ws://rover-hostname:4850");
ws.binaryType = "arraybuffer";  // CRITICAL: Must be arraybuffer

ws.onopen = () => {
  console.log("Connected to rover");
  // Start sending heartbeat
};

ws.onmessage = (event: MessageEvent<ArrayBuffer>) => {
  const telemetry = decodeTelemetry(event.data);
  // Update UI
};

ws.onclose = () => {
  console.log("Disconnected");
  // Reconnect after delay
};
```

## Command Messages (Console → Rover)

### MSG_TWIST (0x01)

Send velocity commands (linear, angular, boost).

**Frequency**: 100 Hz (10ms interval)

**Payload** (25 bytes):
```
[0]:    Message type (0x01)
[1-8]:  Linear velocity (f64, little-endian, m/s)
[9-16]: Angular velocity (f64, little-endian, rad/s)
[17]:   Boost flag (u8, 0 or 1)
[18-24]: Reserved (7 bytes, set to 0)
```

**Encoding:**
```typescript
export function encodeTwist(twist: Twist): ArrayBuffer {
  const buffer = new ArrayBuffer(25);
  const view = new DataView(buffer);

  view.setUint8(0, 0x01);                          // MSG_TWIST
  view.setFloat64(1, twist.linear, true);          // Little-endian f64
  view.setFloat64(9, twist.angular, true);
  view.setUint8(17, twist.boost ? 1 : 0);
  // Bytes 18-24 reserved (zero-filled by default)

  return buffer;
}
```

**Value Ranges:**
- `linear`: -5.0 to 5.0 m/s (rover will clamp to max speed)
- `angular`: -2.5 to 2.5 rad/s (rover will clamp to max turn rate)
- `boost`: boolean (multiplies limits if enabled)

**Example:**
```typescript
// Forward at 2 m/s
const twist = { linear: 2.0, angular: 0.0, boost: false };
const buffer = encodeTwist(twist);
ws.send(buffer);
```

### MSG_ESTOP (0x02)

Trigger emergency stop.

**Frequency**: On-demand (user click or keyboard shortcut)

**Payload** (1 byte):
```
[0]: Message type (0x02)
```

**Encoding:**
```typescript
export function encodeEStop(): ArrayBuffer {
  const buffer = new ArrayBuffer(1);
  const view = new DataView(buffer);
  view.setUint8(0, 0x02);  // MSG_ESTOP
  return buffer;
}
```

**Effect:**
- Immediate motor stop
- Rover transitions to EStop mode
- LEDs flash red
- Requires EStopRelease to resume

### MSG_HEARTBEAT (0x03)

Periodic connection keepalive.

**Frequency**: 10 Hz (100ms interval)

**Payload** (9 bytes):
```
[0]:   Message type (0x03)
[1-8]: Timestamp (f64, little-endian, milliseconds since epoch)
```

**Encoding:**
```typescript
export function encodeHeartbeat(): ArrayBuffer {
  const buffer = new ArrayBuffer(9);
  const view = new DataView(buffer);

  view.setUint8(0, 0x03);  // MSG_HEARTBEAT
  view.setFloat64(1, performance.now(), true);

  return buffer;
}
```

**Purpose:**
- Detect connection issues
- Measure round-trip latency (if rover echoes timestamp)
- Keep WebSocket alive (prevent idle timeout)

### MSG_SET_MODE (0x04)

Request mode transition (Enable, Disable, Autonomous).

**Frequency**: On-demand (user action)

**Payload** (2 bytes):
```
[0]: Message type (0x04)
[1]: Target mode (u8)
```

**Mode Values:**
- `0`: Disabled
- `1`: Idle
- `2`: Teleop
- `3`: Autonomous
- `4`: EStop (use MSG_ESTOP instead)

**Encoding:**
```typescript
export function encodeSetMode(mode: Mode): ArrayBuffer {
  const buffer = new ArrayBuffer(2);
  const view = new DataView(buffer);

  view.setUint8(0, 0x04);  // MSG_SET_MODE
  view.setUint8(1, mode);

  return buffer;
}
```

**Example:**
```typescript
// Request autonomous mode
const buffer = encodeSetMode(Mode.Autonomous);
ws.send(buffer);
```

### MSG_TOOL (0x05)

Control tool attachments (brush, snowblower, etc.).

**Frequency**: On-demand (user control)

**Payload** (variable, max 64 bytes):
```
[0]:    Message type (0x05)
[1]:    Tool ID (u8, 0-15)
[2]:    Command (u8, tool-specific)
[3-63]: Parameters (tool-specific)
```

**Encoding:**
```typescript
export function encodeTool(toolId: number, command: number, params: Uint8Array): ArrayBuffer {
  const buffer = new ArrayBuffer(3 + params.length);
  const view = new DataView(buffer);

  view.setUint8(0, 0x05);    // MSG_TOOL
  view.setUint8(1, toolId);
  view.setUint8(2, command);
  new Uint8Array(buffer, 3).set(params);

  return buffer;
}
```

## Telemetry Messages (Rover → Console)

### MSG_TELEMETRY (0x10)

Comprehensive rover state update.

**Frequency**: 20 Hz (50ms interval, configurable on rover)

**Payload** (92 bytes minimum):
```
[0]:     Message type (0x10)
[1]:     Mode (u8): 0=Disabled, 1=Idle, 2=Teleop, 3=Autonomous, 4=EStop, 5=Fault
[2-9]:   Pose X (f64, little-endian, meters)
[10-17]: Pose Y (f64, little-endian, meters)
[18-21]: Pose Theta (f32, little-endian, radians)
[22-25]: Velocity Linear (f32, little-endian, m/s)
[26-29]: Velocity Angular (f32, little-endian, rad/s)
[30]:    Velocity Boost (u8, 0 or 1)
[31-34]: Battery Voltage (f32, little-endian, volts)
[35-38]: Battery Current (f32, little-endian, amps)
[39-42]: Temp FL Motor (f32, little-endian, °C)
[43-46]: Temp FR Motor (f32, little-endian, °C)
[47-50]: Temp RL Motor (f32, little-endian, °C)
[51-54]: Temp RR Motor (f32, little-endian, °C)
[55-58]: Temp FL Controller (f32, little-endian, °C)
[59-62]: Temp FR Controller (f32, little-endian, °C)
[63-66]: Temp RL Controller (f32, little-endian, °C)
[67-70]: Temp RR Controller (f32, little-endian, °C)
[71-74]: ERPM FL (i32, little-endian, electrical RPM)
[75-78]: ERPM FR (i32, little-endian)
[79-82]: ERPM RL (i32, little-endian)
[83-86]: ERPM RR (i32, little-endian)
[87-90]: GPS Fix (u8: 0=none, 1=2D, 2=3D, 3=RTK)
[91-...]: Additional fields (optional)
```

**Decoding:**
```typescript
export function decodeTelemetry(data: ArrayBuffer): Telemetry {
  if (data.byteLength < 92) {
    throw new Error(`Telemetry frame too short: ${data.byteLength} bytes`);
  }

  const view = new DataView(data);
  const type = view.getUint8(0);

  if (type !== 0x10) {
    throw new Error(`Invalid message type: ${type}, expected 0x10`);
  }

  return {
    mode: view.getUint8(1) as Mode,
    pose: {
      x: view.getFloat64(2, true),
      y: view.getFloat64(10, true),
      theta: view.getFloat32(18, true),
    },
    velocity: {
      linear: view.getFloat32(22, true),
      angular: view.getFloat32(26, true),
      boost: view.getUint8(30) !== 0,
    },
    power: {
      voltage: view.getFloat32(31, true),
      current: view.getFloat32(35, true),
    },
    temperatures: {
      motors: [
        view.getFloat32(39, true),  // FL
        view.getFloat32(43, true),  // FR
        view.getFloat32(47, true),  // RL
        view.getFloat32(51, true),  // RR
      ],
      controllers: [
        view.getFloat32(55, true),  // FL
        view.getFloat32(59, true),  // FR
        view.getFloat32(63, true),  // RL
        view.getFloat32(67, true),  // RR
      ],
    },
    erpm: [
      view.getInt32(71, true),   // FL
      view.getInt32(75, true),   // FR
      view.getInt32(79, true),   // RL
      view.getInt32(83, true),   // RR
    ],
    gps_fix: view.getUint8(87),
  };
}
```

**Usage:**
```typescript
ws.onmessage = (event: MessageEvent<ArrayBuffer>) => {
  try {
    const telemetry = decodeTelemetry(event.data);
    useConsoleStore.getState().updateTelemetry(telemetry);
  } catch (error) {
    console.error("Failed to decode telemetry:", error);
  }
};
```

## Video Connection (Port 4851)

### Connection Setup

```typescript
const videoWs = new WebSocket("ws://rover-hostname:4851");
videoWs.binaryType = "arraybuffer";

videoWs.onmessage = (event: MessageEvent<ArrayBuffer>) => {
  const frame = decodeVideoFrame(event.data);
  displayFrame(frame);
};
```

### MSG_VIDEO_FRAME (0x20)

JPEG-encoded 360° equirectangular video frame.

**Frequency**: 30 Hz (33ms interval, varies with camera)

**Payload** (variable):
```
[0]:      Message type (0x20)
[1-8]:    Timestamp (f64, little-endian, ms since epoch)
[9-12]:   Width (u32, little-endian, pixels, typically 5760)
[13-16]:  Height (u32, little-endian, pixels, typically 2880)
[17-20]:  JPEG size (u32, little-endian, bytes)
[21-...]: JPEG data (variable length)
```

**Decoding:**
```typescript
export function decodeVideoFrame(data: ArrayBuffer): VideoFrame {
  if (data.byteLength < 21) {
    throw new Error("Video frame header too short");
  }

  const view = new DataView(data);
  const type = view.getUint8(0);

  if (type !== 0x20) {
    throw new Error(`Invalid message type: ${type}, expected 0x20`);
  }

  const timestamp = view.getFloat64(1, true);
  const width = view.getUint32(9, true);
  const height = view.getUint32(13, true);
  const jpegSize = view.getUint32(17, true);

  // Extract JPEG data
  const jpegData = new Uint8Array(data, 21, jpegSize);

  // Create blob URL for display
  const blob = new Blob([jpegData], { type: "image/jpeg" });
  const url = URL.createObjectURL(blob);

  return { timestamp, width, height, url };
}
```

**Display:**
```typescript
function VideoDisplay() {
  const { videoFrame, videoConnected } = useConsoleStore((state) => ({
    videoFrame: state.videoFrame,
    videoConnected: state.videoConnected,
  }));

  useEffect(() => {
    // Cleanup blob URL when frame changes
    return () => {
      if (videoFrame) {
        URL.revokeObjectURL(videoFrame);
      }
    };
  }, [videoFrame]);

  return (
    <div className="relative w-full h-full">
      {videoConnected && videoFrame ? (
        <img src={videoFrame} alt="360 Video" className="w-full h-full object-cover" />
      ) : (
        <div className="flex items-center justify-center h-full">
          <p>No video signal</p>
        </div>
      )}
    </div>
  );
}
```

## Latency Measurement

### Round-Trip Time (RTT)

Measure latency by echoing heartbeat timestamps.

**Console sends:**
```typescript
const sendTime = performance.now();
const buffer = encodeHeartbeat(sendTime);
ws.send(buffer);
```

**Rover echoes back** (in telemetry or dedicated response).

**Console receives:**
```typescript
ws.onmessage = (event) => {
  const receiveTime = performance.now();
  const telemetry = decodeTelemetry(event.data);

  // Assume rover includes sendTime in telemetry
  const rtt = receiveTime - telemetry.echoTimestamp;
  useConsoleStore.getState().setLatency(rtt);
};
```

## Error Handling

### Connection Errors

```typescript
ws.onerror = (error) => {
  console.error("WebSocket error:", error);
  useConsoleStore.getState().addToast({
    title: "Connection Error",
    description: "Failed to connect to rover",
    variant: "destructive",
  });
};
```

### Frame Validation

```typescript
function validateFrame(data: ArrayBuffer, expectedType: number, minSize: number): boolean {
  if (data.byteLength < minSize) {
    console.warn(`Frame too short: ${data.byteLength} < ${minSize}`);
    return false;
  }

  const view = new DataView(data);
  const type = view.getUint8(0);

  if (type !== expectedType) {
    console.warn(`Unexpected message type: ${type} (expected ${expectedType})`);
    return false;
  }

  return true;
}
```

### Reconnection Logic

```typescript
function useRoverConnection() {
  const [reconnectDelay, setReconnectDelay] = useState(1000); // Start with 1s
  const address = useConsoleStore((state) => state.roverAddress);

  const connect = useCallback(() => {
    const ws = new WebSocket(address);
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
      console.log("Connected");
      setReconnectDelay(1000); // Reset delay on success
      useConsoleStore.getState().setConnected(true);
    };

    ws.onclose = () => {
      console.log("Disconnected");
      useConsoleStore.getState().setConnected(false);

      // Exponential backoff (max 30s)
      const delay = Math.min(reconnectDelay * 2, 30000);
      setReconnectDelay(delay);

      setTimeout(connect, delay);
    };

    return ws;
  }, [address, reconnectDelay]);

  useEffect(() => {
    const ws = connect();
    return () => ws.close();
  }, [connect]);
}
```

## Performance Considerations

### Send Rate Limiting

```typescript
// Good: Rate-limited command sending
useEffect(() => {
  if (!ws || !connected) return;

  const interval = setInterval(() => {
    const input = useConsoleStore.getState().input;
    const twist = { linear: input.linear, angular: input.angular, boost: input.boost };
    ws.send(encodeTwist(twist));
  }, 10); // 100Hz

  return () => clearInterval(interval);
}, [ws, connected]);

// Bad: Sending on every state change (unpredictable rate)
useEffect(() => {
  if (!ws || !connected) return;
  ws.send(encodeTwist(input));  // ❌ Can send 1000+ Hz
}, [input]);
```

### Batching (Future Optimization)

For future optimization, batch multiple small messages:

```typescript
// Batch MSG_HEARTBEAT + MSG_TWIST into single frame
function encodeBatch(messages: ArrayBuffer[]): ArrayBuffer {
  const totalSize = messages.reduce((sum, msg) => sum + msg.byteLength, 0);
  const batch = new ArrayBuffer(totalSize);
  const view = new Uint8Array(batch);

  let offset = 0;
  for (const msg of messages) {
    view.set(new Uint8Array(msg), offset);
    offset += msg.byteLength;
  }

  return batch;
}
```

## Debugging Tools

### Message Logger

```typescript
function logMessage(data: ArrayBuffer, direction: "sent" | "received") {
  const view = new DataView(data);
  const type = view.getUint8(0);
  const typeName = getMessageTypeName(type);

  console.log(`[${direction}] ${typeName} (0x${type.toString(16).padStart(2, "0")}) - ${data.byteLength} bytes`);
}

function getMessageTypeName(type: number): string {
  const names: Record<number, string> = {
    0x01: "MSG_TWIST",
    0x02: "MSG_ESTOP",
    0x03: "MSG_HEARTBEAT",
    0x04: "MSG_SET_MODE",
    0x05: "MSG_TOOL",
    0x10: "MSG_TELEMETRY",
    0x20: "MSG_VIDEO_FRAME",
  };
  return names[type] || "UNKNOWN";
}
```

### Hex Dump

```typescript
function hexDump(data: ArrayBuffer, maxBytes: number = 64): string {
  const bytes = new Uint8Array(data).slice(0, maxBytes);
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join(" ");
  return hex;
}

// Usage
console.log("Telemetry hex:", hexDump(telemetryBuffer));
```

## References

- WebSocket API: https://developer.mozilla.org/en-US/docs/Web/API/WebSocket
- DataView: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView
- ArrayBuffer: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer
