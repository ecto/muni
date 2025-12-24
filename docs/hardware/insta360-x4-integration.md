# Insta360 X4 Video Feed Integration

This document traces the complete path from the Insta360 X4 camera on the rover to the spherical skybox in the operator console.

## Quick Start Checklist

### Day 1: Hardware Setup

- [ ] Connect Insta360 X4 to Jetson via USB-C (use USB 3.0 port)
- [ ] Power on camera, enable **Webcam Mode** (Settings → USB → Webcam)
- [ ] Verify device appears: `ls /dev/video*`
- [ ] Test GStreamer capture:
  ```bash
  gst-launch-1.0 v4l2src device=/dev/video0 num-buffers=30 ! \
    video/x-raw,width=1920,height=960 ! videoconvert ! autovideosink
  ```

### Day 1: Firmware Test

```bash
cd bvr/firmware
cargo build --release

# Run with camera at 1920x960 (2:1 equirectangular)
./target/release/bvrd --camera-resolution 1920x960 --camera-fps 30
```

Look for log output:
```
INFO Detected USB camera (video0)
INFO Camera capture started
INFO WebSocket video server listening addr="0.0.0.0:4851"
```

### Day 1: Operator Test

```bash
cd depot/operator
npm install
npm run dev
```

1. Open http://localhost:5173
2. Connect to rover (or use localhost if running locally)
3. Enter teleop mode
4. Verify "360° XX FPS" badge in top-right corner
5. Look around: the skybox should show the 360° feed

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ROVER (Jetson)                                 │
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────────┐  │
│  │ Insta360 X4  │───►│ camera crate │───►│ watch::channel<VideoFrame>   │  │
│  │ (USB UVC)    │    │ (GStreamer)  │    │                              │  │
│  └──────────────┘    └──────────────┘    └──────────────┬───────────────┘  │
│                                                         │                   │
│                      ┌──────────────────────────────────┼───────────────┐  │
│                      │                                  │               │  │
│                      ▼                                  ▼               │  │
│              ┌──────────────┐                   ┌──────────────┐        │  │
│              │ UDP Video    │                   │ WS Video     │        │  │
│              │ Server :4842 │                   │ Server :4851 │        │  │
│              └──────────────┘                   └──────────────┘        │  │
│                                                         │               │  │
└─────────────────────────────────────────────────────────┼───────────────────┘
                                                          │
                                    ┌─────────────────────┘
                                    │ WebSocket (binary)
                                    │ [0x20][timestamp][w][h][jpeg...]
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           OPERATOR (Browser)                                │
│                                                                             │
│  ┌──────────────────┐    ┌──────────────────┐    ┌────────────────────┐   │
│  │ useVideoStream() │───►│ decodeVideoFrame │───►│ Blob URL           │   │
│  │ WebSocket client │    │ (protocol.ts)    │    │                    │   │
│  └──────────────────┘    └──────────────────┘    └─────────┬──────────┘   │
│                                                            │              │
│                                                            ▼              │
│                                               ┌────────────────────────┐  │
│                                               │ EquirectangularSky.tsx │  │
│                                               │ - Inverted sphere      │  │
│                                               │ - TextureLoader        │  │
│                                               │ - Counter-rotation     │  │
│                                               └────────────────────────┘  │
│                                                            │              │
│                                                            ▼              │
│                                               ┌────────────────────────┐  │
│                                               │ Three.js Canvas        │  │
│                                               │ + WebXR (VR headset)   │  │
│                                               └────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Camera Capture (Rover)

**File**: `bvr/firmware/crates/camera/src/lib.rs`

The camera crate uses GStreamer for capture:

```rust
// USB camera pipeline (V4L2)
v4l2src device=/dev/videoX do-timestamp=true !
video/x-raw,width=WIDTH,height=HEIGHT,framerate=FPS/1 !
videoconvert !
video/x-raw,format=RGB !
appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false
```

Key configuration (`Config` struct):
- `width`: Capture width (default 640, recommend 2048 for 360°)
- `height`: Capture height (default 480, recommend 1024 for 360°)
- `fps`: Target framerate (default 30)
- `jpeg_quality`: 1-100 (default 60, lower = faster + less bandwidth)

Output: `Frame` struct with JPEG data, dimensions, timestamp, sequence number.

### 2. Video Server (Rover)

**File**: `bvr/firmware/crates/teleop/src/video_ws.rs`

WebSocket server that streams frames to browser operators:

```rust
// Frame encoding format:
[0x20]                    // 1 byte: message type (video frame)
[timestamp: u64 LE]       // 8 bytes: milliseconds since epoch
[width: u16 LE]           // 2 bytes: frame width
[height: u16 LE]          // 2 bytes: frame height
[jpeg_data: ...]          // variable: JPEG image data
```

Default port: 4851

### 3. Main Daemon (Rover)

**File**: `bvr/firmware/bins/bvrd/src/main.rs`

CLI arguments for camera:
- `--no-camera`: Disable camera auto-detection
- `--camera-resolution WxH`: Set resolution (e.g., "2048x1024")
- `--camera-fps N`: Set framerate
- `--ws-video-port N`: WebSocket video port (default 4851)

### 4. Discovery Service (Depot)

**File**: `depot/discovery/src/main.rs`

Rovers register with:
```json
{
  "id": "bvr0",
  "name": "BVR-0",
  "address": "ws://192.168.1.100:4850",
  "video_address": "ws://192.168.1.100:4851"
}
```

### 5. Video Stream Hook (Operator)

**File**: `depot/operator/src/hooks/useVideoStream.ts`

- Connects to `videoAddress` from store
- Decodes binary WebSocket messages
- Creates Blob URLs from JPEG data
- Updates store: `videoFrame`, `videoFps`, `videoConnected`

### 6. Protocol Decoder (Operator)

**File**: `depot/operator/src/lib/protocol.ts`

```typescript
// Message type constant
export const MSG_VIDEO_FRAME = 0x20;

// Decoded frame structure
interface DecodedVideoFrame {
  timestamp_ms: number;
  width: number;
  height: number;
  jpegData: Uint8Array;
}
```

### 7. Equirectangular Skybox (Operator)

**File**: `depot/operator/src/components/scene/EquirectangularSky.tsx`

Three.js sphere with:
- Radius: 500 units
- Segments: 64 horizontal, 32 vertical
- Material: `MeshBasicMaterial` with `BackSide` rendering
- UVs flipped horizontally for correct orientation
- Rotation: `-renderPose.theta` (counter-rotates with rover heading)

When video frame updates:
1. Load texture from Blob URL via `TextureLoader`
2. Set `colorSpace` to `SRGBColorSpace`
3. Apply as material map
4. Dispose previous texture

---

## Insta360 X4 Setup

### Hardware Connection

1. Connect Insta360 X4 to Jetson via USB-C
2. Power on camera and set to **Webcam Mode**:
   - Settings → USB Mode → Webcam
   - Or use Insta360 app to configure

### Verify Detection

```bash
# Check if camera appears as V4L2 device
ls -la /dev/video*

# Check camera capabilities
v4l2-ctl --device=/dev/video0 --list-formats-ext

# Test capture with GStreamer
gst-launch-1.0 v4l2src device=/dev/video0 num-buffers=10 ! \
  video/x-raw,width=1920,height=960,framerate=30/1 ! \
  jpegenc ! filesink location=test.jpg
```

### Recommended Settings

For the Insta360 X4 in webcam mode:

| Setting | Value | Notes |
|---------|-------|-------|
| Resolution | 1920x960 or 2048x1024 | 2:1 equirectangular aspect |
| FPS | 30 | Balance latency/bandwidth |
| JPEG Quality | 60-70 | ~3-5 Mbps at 30fps |

### Start bvrd with Camera

```bash
# Auto-detect camera with custom resolution
bvrd --camera-resolution 1920x960 --camera-fps 30

# Or with explicit device (if multiple cameras)
# (requires code modification to accept device path)
```

---

## Bandwidth Estimates

| Resolution | FPS | Quality | Estimated Bandwidth |
|------------|-----|---------|---------------------|
| 1280x640   | 30  | 60      | ~2 Mbps             |
| 1920x960   | 30  | 60      | ~4 Mbps             |
| 2048x1024  | 30  | 60      | ~5 Mbps             |
| 2048x1024  | 15  | 60      | ~2.5 Mbps           |

For LTE deployment, 1920x960 @ 30fps @ quality 60 is recommended.

---

## Latency Budget

| Stage | Typical Latency |
|-------|-----------------|
| Camera capture | 16-33ms (1-2 frames) |
| JPEG encoding | 5-15ms (SW) / 2-5ms (HW) |
| WebSocket send | 1-5ms |
| Network (LTE) | 50-150ms |
| Browser decode | 2-5ms |
| Texture upload | 1-3ms |
| Render | 16ms (60fps) |
| **Total** | **~100-230ms** |

---

## Troubleshooting

### Camera Not Detected

1. Check USB connection and power
2. Verify webcam mode is enabled on Insta360
3. Check `dmesg` for USB device enumeration
4. Try different USB port (prefer USB 3.0)

### Low FPS / Stuttering

1. Reduce resolution
2. Lower JPEG quality
3. Check network bandwidth with `iftop`
4. Monitor CPU usage on Jetson

### Distorted/Rotated Image

1. The Insta360 X4 outputs equirectangular by default in webcam mode
2. If image is rotated, adjust GStreamer pipeline with `videoflip`
3. UV flip in EquirectangularSky handles horizontal orientation

### Black/Gray Skybox

1. Check `videoConnected` state in browser console
2. Verify WebSocket connection in Network tab
3. Check rover logs for video server errors
4. Ensure firewall allows port 4851

---

## Code References

### Rover Side
- Camera capture: `bvr/firmware/crates/camera/src/lib.rs`
- Video frame type: `bvr/firmware/crates/teleop/src/video.rs`
- WebSocket server: `bvr/firmware/crates/teleop/src/video_ws.rs`
- Main daemon: `bvr/firmware/bins/bvrd/src/main.rs`

### Operator Side
- Video hook: `depot/operator/src/hooks/useVideoStream.ts`
- Protocol: `depot/operator/src/lib/protocol.ts`
- Skybox: `depot/operator/src/components/scene/EquirectangularSky.tsx`
- Scene: `depot/operator/src/components/scene/Scene.tsx`
- Store: `depot/operator/src/store.ts`

---

## Future Enhancements

1. **Hardware JPEG encoding**: Use `nvjpegenc` on Jetson for lower latency
2. **Adaptive quality**: Reduce quality/resolution when bandwidth is limited
3. **H.265 streaming**: For better compression (requires WebCodecs)
4. **Stereo 360°**: Insta360 X4 can output stereo for VR headsets
5. **WebRTC**: Lower latency than WebSocket for real-time streaming
