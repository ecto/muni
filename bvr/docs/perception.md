# Perception & Mapping

BVR's perception system for safety, navigation, and situational awareness.

## Hardware Evolution

| Generation | Sensors | Cost | Notes |
|------------|---------|------|-------|
| bvr0 (original) | Livox Mid-360 lidar + Insta360 X4 | ~$1700 | Full 3D, IMU, expensive |
| bvr0 (current) | 3× Arducam IMX291 USB cameras | ~$156 | Vision-only, monocular depth |

The current platform uses three 120° FOV cameras spaced 120° apart for
near-360° coverage, with monocular depth estimation replacing lidar for
obstacle detection and visual odometry replacing wheel encoders.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Perception Pipeline (camera)                        │
│                                                                             │
│  3× Arducam IMX291 (120° FOV, 1080p, USB 2.0, $52/ea)                     │
│  ├── H.264 stream (1920×1080 @ 30fps) → WebRTC teleop                     │
│  └── Raw RGB (640×480 @ 15fps) → Depth estimation pipeline                │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │  Depth Anything V2/V3 Small (ONNX, TensorRT on Jetson)               │ │
│  │  RGB → relative depth → metric scale → 3D back-projection            │ │
│  └──────────┬──────────────────────────────┬────────────────────────────┘ │
│             │                              │                               │
│             ▼                              ▼                               │
│  ┌─────────────────────┐     ┌──────────────────────────┐                │
│  │  Costmap (obstacles) │     │  Visual Odometry (ICP)    │                │
│  │  ground/obstacle     │     │  frame-to-frame ego-motion│                │
│  │  classification      │     │  → EKF prediction          │                │
│  └──────────┬──────────┘     └──────────────────────────┘                │
│             │                                                              │
│             ▼                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │  Navigation: A* planner + Pure Pursuit / MPPI controller            │ │
│  │  Collision Guard: arc projection + velocity scaling                  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Camera Configuration

Three cameras mounted at 120° intervals provide near-complete horizontal
coverage. Each camera has independent extrinsics defined in `bvr.toml`:

```toml
[[camera.mounts]]
id = "left"
device_hint = "/dev/video0"
mount_position = [0.25, 0.12, 0.25]      # [forward, left, up] meters
mount_rotation = [0.0, -0.175, -2.094]    # [roll, pitch=-10°, yaw=-120°]
fov_h = 1.745   # ~100° horizontal
fov_v = 1.18    # ~68° vertical

[[camera.mounts]]
id = "center"
device_hint = "/dev/video2"
mount_position = [0.25, 0.0, 0.25]
mount_rotation = [0.0, -0.175, 0.0]

[[camera.mounts]]
id = "right"
device_hint = "/dev/video4"
mount_position = [0.25, -0.12, 0.25]
mount_rotation = [0.0, -0.175, 2.094]     # yaw +120°
```

## Depth Estimation Pipeline

### Stage 1: Raw Capture
Each camera spawns a dedicated GStreamer pipeline for raw RGB frames at
reduced resolution (640×480 @ 15fps) to minimize USB bandwidth. This runs
in parallel with the H.264 streaming pipeline (1080p @ 30fps for teleop).

### Stage 2: Monocular Depth
The `depth` crate runs Depth Anything V2/V3 Small via ONNX Runtime. On
Jetson Orin NX, this uses the TensorRT execution provider for ~15ms
inference per frame. The model outputs relative/inverse depth; we convert
to metric using known camera mount height.

### Stage 3: Back-Projection
`backproject_and_classify()` converts depth pixels to 3D points in the
rover body frame using camera intrinsics (from FOV) and extrinsics (mount
position + rotation). Points are classified by height:
- **Ground**: z < `ground_threshold` (0.10m)
- **Obstacle**: z >= `min_obstacle_height` (0.05m)

Coordinate frame conversion:
- Camera: Z-forward, X-right, Y-down (OpenCV)
- Rover: X-forward, Y-left, Z-up (ROS)

### Stage 4: Costmap Integration
Ground and obstacle points from all cameras are merged and fed into the
costmap via `NavigationController::update_costmap_from_points()`. The
costmap is sensor-agnostic — same code path works for lidar or depth cameras.

### Stage 5: Visual Odometry
2D ICP (Iterative Closest Point) aligns consecutive depth frames projected
to the ground plane. Returns `(dx, dy, dθ)` matching the `WheelOdometry`
interface, fed directly into the EKF pose estimator.

## Costmap Decoupling

The costmap was originally tightly coupled to `lidar::PointCloud`. It now
accepts generic `&[Vector3<f64>]` point arrays:

```rust
// Generic entry point (works with any point source)
nav.update_costmap_from_points(sensor_pos, &ground_pts, &obstacle_pts);

// Lidar-specific method (delegates to generic, behind feature flag)
#[cfg(feature = "lidar")]
nav.update_costmap(&scan, &robot_tf);
```

Feature flags in `costmap` and `control-loop` crates make lidar optional:
```toml
[features]
default = ["lidar"]
lidar = ["dep:lidar"]
```

## Collision Guard

The collision guard scales teleop velocity based on obstacle proximity.
It works identically with lidar or depth-derived costmaps — it reads the
costmap grid, not raw sensor data.

```toml
[collision_guard]
enabled = true
lookahead_time = 1.0        # seconds to project arc forward
full_speed_distance = 1.0   # no scaling beyond this
stop_distance = 0.15        # full stop below this
```

## Localization

```
                    ┌──────────────┐
                    │ EKF Pose     │
                    │ Estimator    │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
     ┌────────────┐ ┌───────────┐ ┌──────────┐
     │ Visual     │ │ IMU       │ │ GPS      │
     │ Odometry   │ │ (BMI088)  │ │ (if      │
     │ (depth ICP)│ │ gyro-Z    │ │ present) │
     └────────────┘ └───────────┘ └──────────┘
```

The EKF fuses:
- **Prediction**: Visual odometry `(dx, dy, dθ)` + IMU gyro-Z
- **Measurement**: GPS (when available), SLAM pose corrections

Visual odometry replaces wheel odometry on platforms without encoders
(2WD chain drive with casters).

## Key Crates

| Crate | Purpose |
|-------|---------|
| `depth` | Monocular depth, back-projection, visual odometry |
| `camera` | GStreamer capture (H.264 + raw RGB) |
| `costmap` | Occupancy grid, inflation, obstacle extraction |
| `control-loop` | Navigation controller (A* + pursuit/MPPI) |
| `localization` | EKF pose estimator, wheel/visual odometry |
| `collision-monitor` | Teleop velocity scaling near obstacles |

## Configuration Reference

```toml
[depth]
enabled = false              # Enable when model is deployed
model_path = "/var/lib/bvr/models/depth-anything-v2-small.onnx"
model_input_width = 518
model_input_height = 518
capture_width = 640          # Raw RGB resolution for inference
capture_height = 480
capture_fps = 15
max_depth = 8.0              # Meters
min_depth = 0.3
ground_threshold = 0.10      # Height classification (meters)
stride = 4                   # Pixel stride (4 = every 4th pixel)
visual_odometry = true
icp_max_dist = 0.5           # ICP correspondence distance
icp_max_iterations = 20
```

## Migration Status

- [x] Costmap decoupled from lidar (generic point interface)
- [x] Per-camera extrinsics config (`[[camera.mounts]]`)
- [x] Raw RGB capture pipeline
- [x] Depth crate: back-projection + classification
- [x] Visual odometry (2D ICP)
- [x] Pipeline wired into bvrd run loop
- [ ] ONNX model download + TensorRT calibration
- [ ] Field testing + parameter tuning
- [ ] Multi-camera depth fusion
- [ ] Standalone IMU driver (BMI088)
- [ ] SLAM adaptation for depth-only input
