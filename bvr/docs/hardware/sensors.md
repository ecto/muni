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

**Orientation**: Dome pointing UP (not rotated).

**Position**: Top-center of rover chassis, stacked with Insta360 above.

#### Height Tradeoffs

The -7° vertical FOV limit creates a ground blind spot. At height `h`:

- **Blind spot radius** = h × tan(7°) ≈ h × 0.12
- **Closest visible ground** = h / tan(7°) ≈ h × 8.1

| Height      | Blind Radius | Closest Ground | Chassis in Insta360 Frame |
| ----------- | ------------ | -------------- | ------------------------- |
| 12" (305mm) | 1.5" (38mm)  | 2.5m           | 45°                       |
| 18" (457mm) | 2.2" (56mm)  | 3.7m           | 34°                       |
| 24" (610mm) | 3.0" (76mm)  | 4.9m           | 27°                       |

**Recommendation**: 18-24" total sensor height balances ground coverage and
camera view. Final decision deferred to prototyping.

#### Stacking Order

```
                    ┌─────────┐
                    │Insta360 │  ← Top (lighter, needs 360° view)
                    │   X4    │
                    └────┬────┘
                         │      ~100mm spacing
                    ┌────┴────┐
                    │ Mid-360 │  ← Below camera
                    │         │
                    └────┬────┘
                         │
                    ┌────┴────┐
                    │  Pole   │  Variable height (12-24")
                    │ (1" AL) │
                    └────┬────┘
                         │
    ═══════════════════════════════════════
                    Rover Chassis
```

#### Connection

- **bvr0**: Direct Ethernet cable from Mid-360 to Jetson
- **bvr1**: Via network switch (adds debug port for laptop)
- Power from 12V rail (Mid-360 accepts 9-27V)

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

### IMU Access

The Mid-360 includes a **built-in 200Hz IMU** (accelerometer + gyroscope) that
streams alongside point cloud data over Ethernet.

#### Data Ports

| Port      | Protocol       | Data                       |
| --------- | -------------- | -------------------------- |
| UDP 56000 | Livox Protocol | Point cloud (200k pts/sec) |
| UDP 56001 | Livox Protocol | IMU samples (200 Hz)       |
| TCP 56100 | Livox Protocol | Control, configuration     |

#### IMU Data Structure

```rust
pub struct ImuSample {
    pub timestamp_ns: u64,
    pub accel: [f32; 3],  // m/s², body frame
    pub gyro: [f32; 3],   // rad/s, body frame
}
```

#### Integration Options

| Method                      | Complexity | Notes                         |
| --------------------------- | ---------- | ----------------------------- |
| **Direct UDP parsing**      | Medium     | No dependencies, full control |
| **Livox SDK 2.0 (C++ FFI)** | Medium     | Official SDK, callbacks       |
| **ROS 2 driver bridge**     | Low        | Easy but adds ROS dependency  |

For bvr, we recommend direct UDP parsing to avoid external dependencies.

### PPS Sync

The Mid-360 provides a **PPS (pulse per second) output** for hardware synchronization
with cameras.

#### Sync Connector Pinout (8-pin)

| Pin | Function             |
| --- | -------------------- |
| 1   | GND                  |
| 2   | PPS Out (3.3V pulse) |
| 3   | GPRMC TX             |
| 4   | GPRMC RX             |
| 5   | GPS PPS In           |
| 6-8 | Reserved             |

#### bvr0 (Software Sync)

- Timestamp both sensors with system clock
- Accept 10-50ms uncertainty
- Interpolate LiDAR poses to camera timestamps during offline processing

#### bvr1 (Hardware Sync)

- Connect PPS Out to camera external trigger input
- Requires camera with hardware trigger (FLIR Blackfly, industrial cameras)
- The Insta360 X4 does **not** support hardware triggering

### Snow Handling

The Mid-360 uses 905nm wavelength which is relatively robust to precipitation.

| Condition      | Effect                    | Mitigation                    |
| -------------- | ------------------------- | ----------------------------- |
| Light snow     | Occasional noise points   | Filter by return intensity    |
| Moderate snow  | More noise, reduced range | Statistical outlier removal   |
| Heavy/blizzard | Significant degradation   | Don't map in these conditions |
| Snow on ground | No issue                  | LiDAR measures snow surface   |
| Snow on lens   | Blocked scanning          | Lens hood, compressed air     |

```rust
// Filter weak returns (likely snow/rain)
fn filter_precipitation(points: &[Point3]) -> Vec<Point3> {
    points.iter()
        .filter(|p| p.intensity > 25)      // Snow has weak returns
        .filter(|p| p.return_count == 1)   // Single returns only
        .cloned()
        .collect()
}
```

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

## Calibration

### LiDAR-Camera Extrinsic

The transform from LiDAR frame to camera frame must be known for sensor fusion.

#### Initial Estimate (Manual Measurement)

```json
{
  "lidar_to_camera": {
    "translation": [0.0, 0.0, 0.1],
    "rotation": [0, 0, 0, 1]
  }
}
```

With sensors stacked vertically and aligned, translation is approximately
[0, 0, spacing] where spacing is the distance between sensor origins (~100mm).

#### Automatic Refinement

Use one of these tools on recorded data:

| Tool                                                                                         | Method         | Accuracy |
| -------------------------------------------------------------------------------------------- | -------------- | -------- |
| [direct_visual_lidar_calibration](https://github.com/koide3/direct_visual_lidar_calibration) | Edge alignment | ±1cm     |
| [lidar_camera_calibration](https://github.com/ankitdhall/lidar_camera_calibration)           | Checkerboard   | ±2mm     |

For the Insta360's dual fisheye lenses, calibrate each lens separately.

### Validation

Project LiDAR points onto camera image. Well-calibrated sensors show:

- Point cloud edges align with image edges
- No systematic offset in any direction
- Consistent alignment across the frame
