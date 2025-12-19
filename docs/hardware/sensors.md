# Sensor Stack

BVR's perception system for safety, teleoperation, and mapping.

## Overview

| Sensor        | Purpose                              | Price   | Procurement        |
| ------------- | ------------------------------------ | ------- | ------------------ |
| Livox Mid-360 | Safety, obstacle detection, geometry | ~$1,500 | Livox direct / DJI |
| Insta360 X4   | Teleop, classification, appearance   | ~$300   | Amazon Prime       |

**Total: ~$1,800**

## Livox Mid-360

**Primary safety sensor** — provides deterministic obstacle detection regardless of lighting.

### Specifications

| Parameter  | Value                                        |
| ---------- | -------------------------------------------- |
| FOV        | 360° horizontal × 59° vertical (-7° to +52°) |
| Range      | 0.1m – 70m (10% reflectivity)                |
| Point rate | 200,000 pts/sec                              |
| Scan rate  | 10 Hz                                        |
| IMU        | Built-in, 200 Hz                             |
| Interface  | 100BASE-T Ethernet                           |
| Power      | 9-27V, 9W typical                            |
| Weight     | 265g                                         |
| Dimensions | 65 × 65 × 60 mm                              |

### Mounting

- Top-center of rover chassis
- Clear 360° horizontal sightline
- Elevated ~300mm above chassis for ground clearance
- Ethernet to Jetson, power from 12V rail

### Safety Integration

The Mid-360 enables a simple, deterministic safety layer:

```rust
// No ML in the critical path — just geometry
if lidar.any_point_within_radius(1.5) {
    state.trigger_estop();
}
```

Latency budget:

| Stage            | Time          |
| ---------------- | ------------- |
| LiDAR scan       | 10ms (100 Hz) |
| Point processing | 1-2ms         |
| CAN command      | 1ms           |
| VESC reaction    | 10ms          |
| Motor decel      | ~200ms        |
| **Total**        | **~225ms**    |

At 1 m/s, this means 22cm of travel before stop — safe with 1.5m detection radius.

### PPS Sync

The Mid-360 provides a PPS (pulse per second) output for hardware synchronization
with cameras. For bvr1, this enables precise LiDAR-camera calibration for
high-quality Gaussian splatting.

## Insta360 X4

**Situational awareness and appearance capture** — provides 360° video for
teleop and texture data for mapping.

### Specifications

| Parameter  | Value                      |
| ---------- | -------------------------- |
| Resolution | 8K (5.7K @ 30fps typical)  |
| FOV        | 360° × 360° (dual fisheye) |
| Sensor     | 1/2" CMOS × 2              |
| Shutter    | Rolling                    |
| Interface  | USB-C (UVC mode), WiFi     |
| Battery    | 135 min                    |
| Weight     | 203g                       |
| Dimensions | 46 × 123.6 × 37.6 mm       |

### Mounting

- Top-center, above Mid-360 on extension pole
- 360° clear view (minimize rover body in frame)
- USB-C to Jetson for live streaming
- Optional: external power via USB-PD

### UVC Streaming Mode

The Insta360 X4 can stream as a USB webcam (UVC mode):

1. Connect USB-C to Jetson
2. Enable webcam mode on camera
3. Access via `/dev/video*` with v4l2

```bash
# Check camera availability
v4l2-ctl --list-devices

# Stream with GStreamer
gst-launch-1.0 v4l2src device=/dev/video0 ! videoconvert ! autovideosink
```

### Limitations (bvr0 prototype)

| Issue            | Impact                      | Mitigation                    |
| ---------------- | --------------------------- | ----------------------------- |
| Rolling shutter  | Motion blur during movement | Reduce speed during mapping   |
| No hardware sync | Timing uncertainty vs LiDAR | Software timestamp alignment  |
| Auto exposure    | Appearance inconsistency    | Post-processing normalization |
| Processed output | No raw sensor access        | Accept for prototype          |

These limitations are acceptable for bvr0. For bvr1, consider upgrading to
hardware-synced global shutter cameras (FLIR Blackfly S, Arducam IMX296, etc.).

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Sensors                                                                  │
│                                                                          │
│  ┌─────────────┐              ┌─────────────┐                          │
│  │ Livox       │              │ Insta360    │                          │
│  │ Mid-360     │              │ X4          │                          │
│  └──────┬──────┘              └──────┬──────┘                          │
│         │ Ethernet                   │ USB-C                           │
└─────────┼────────────────────────────┼──────────────────────────────────┘
          │                            │
          ▼                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ Jetson Orin NX                                                           │
│                                                                          │
│  ┌─────────────┐              ┌─────────────┐                          │
│  │ lidar crate │              │ camera crate│                          │
│  │             │              │             │                          │
│  │ • Point     │              │ • Equirect  │                          │
│  │   cloud     │              │   decode    │                          │
│  │ • Safety    │              │ • JPEG enc  │                          │
│  │   zones     │              │ • Streaming │                          │
│  └──────┬──────┘              └──────┬──────┘                          │
│         │                            │                                  │
│         ▼                            ▼                                  │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ bvrd                                                             │   │
│  │                                                                   │   │
│  │  Safety Layer    │    Recording     │    Teleop Streaming       │   │
│  │  (E-Stop on      │    (.rrd files)  │    (WebSocket video)      │   │
│  │   obstacle)      │                  │                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
          │                            │
          │ LTE                        │ LTE
          ▼                            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ Operator Station                                                         │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Operator App (React Three Fiber)                                 │   │
│  │                                                                   │   │
│  │  ┌───────────────┐    ┌───────────────┐    ┌────────────────┐  │   │
│  │  │ 360° Skybox   │    │ Rover Model   │    │ Point Cloud    │  │   │
│  │  │ (live feed)   │    │ (telemetry)   │    │ (optional)     │  │   │
│  │  └───────────────┘    └───────────────┘    └────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Future: Gaussian Splatting (bvr1)

For high-fidelity 3D reconstruction, bvr1 may upgrade to:

| Component  | Upgrade                     | Benefit                       |
| ---------- | --------------------------- | ----------------------------- |
| Camera     | FLIR Blackfly S × 2-3       | Global shutter, hardware sync |
| Sync       | PPS from Mid-360 to cameras | Precise temporal alignment    |
| Processing | Real-time splatting on Orin | Live 3D reconstruction        |

Pipeline options:

- **[Splat-SLAM](https://github.com/eriksandstroem/Splat-SLAM)** — RGB-only Gaussian SLAM
- **[GS-LIVM](https://github.com/xieyuser/GS-LIVM)** — LiDAR-Inertial-Visual Gaussian mapping
- **[LiV-GS](https://arxiv.org/abs/2411.12185)** — Large-scale outdoor LiDAR-visual splatting

For bvr0, we'll experiment with the Insta360 + Mid-360 combination to learn
what actually matters before specifying the bvr1 sensor stack.
