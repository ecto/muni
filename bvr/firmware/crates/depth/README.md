# depth — Camera-Based Perception

Monocular depth estimation, 3D back-projection, and visual odometry for
BVR rovers using commodity USB cameras instead of lidar.

## Architecture

```
3× Arducam IMX291 (120° FOV, USB 2.0)
        │
        ▼ raw RGB (640×480 @ 15fps)
┌─────────────────────────────────────────────────┐
│  depth crate                                    │
│                                                 │
│  1. DepthEstimator (ONNX)                       │
│     RGB → relative depth map (518×518)          │
│     Depth Anything V2/V3 Small, TensorRT on     │
│     Jetson Orin NX                              │
│                                                 │
│  2. estimate_metric_scale()                     │
│     Relative depth → metric depth using known   │
│     camera mount height as reference            │
│                                                 │
│  3. backproject_and_classify()                  │
│     Depth pixels → 3D points in rover frame     │
│     Classified as ground or obstacle by height  │
│                                                 │
│  4. VisualOdometry (2D ICP)                     │
│     Frame-to-frame ego-motion from ground-plane │
│     point alignment. Returns (dx, dy, dθ)       │
│     compatible with EKF prediction.             │
└──────────┬──────────────────────┬───────────────┘
           │                      │
           ▼                      ▼
     Costmap update         EKF prediction
     (obstacles →           (replaces wheel
      navigation)            odometry)
```

## Coordinate Frames

**Camera frame** (OpenCV): Z-forward, X-right, Y-down
**Rover frame** (ROS): X-forward, Y-left, Z-up

Base rotation (camera → rover, no mount):
```
rover_X =  cam_Z    (forward = depth)
rover_Y = -cam_X    (left = -right)
rover_Z = -cam_Y    (up = -down)
```

**Mount rotation convention**: `[roll, pitch, yaw]` Euler angles (intrinsic
ZYX via `nalgebra::from_euler_angles`) describing the rotation FROM rover
frame TO camera frame. Negative pitch = looking down.

Transform to rover frame: `R_mount.transpose() * R_cam_to_rover_base * p_cam + translation`

## Camera Mount Configuration

```toml
# config/bvr.toml — 3× cameras at 120° apart for ~360° coverage

[[camera.mounts]]
id = "left"
device_hint = "/dev/video0"
mount_position = [0.25, 0.12, 0.25]      # [forward, left, up] meters
mount_rotation = [0.0, -0.175, -2.094]    # [roll, pitch, yaw] radians
fov_h = 1.745   # ~100° horizontal
fov_v = 1.18    # ~68° vertical

[[camera.mounts]]
id = "center"
device_hint = "/dev/video2"
mount_position = [0.25, 0.0, 0.25]
mount_rotation = [0.0, -0.175, 0.0]       # forward-facing

[[camera.mounts]]
id = "right"
device_hint = "/dev/video4"
mount_position = [0.25, -0.12, 0.25]
mount_rotation = [0.0, -0.175, 2.094]     # yaw +120°
```

## Depth Perception Configuration

```toml
[depth]
enabled = true
model_path = "/var/lib/bvr/models/depth-anything-v2-small.onnx"
model_input_width = 518
model_input_height = 518
capture_width = 640
capture_height = 480
capture_fps = 15
max_depth = 8.0
min_depth = 0.3
ground_threshold = 0.10
stride = 4
visual_odometry = true
icp_max_dist = 0.5
icp_max_iterations = 20
```

## Visual Odometry

Ego-motion estimation from depth-derived 2D scans, using Iterative Closest
Point (ICP) with KD-tree nearest-neighbor search. Replaces wheel odometry
for platforms without encoders (chain drive).

1. Project all classified 3D points to 2D ground plane `(x, y)`
2. Build KD-tree from previous frame's points
3. Iteratively find correspondences and solve SVD-based rigid alignment
4. Return `(dx, dy, dθ)` — same interface as `WheelOdometry::update()`
5. Feed into `EkfPoseEstimator::predict()` alongside IMU gyro-Z

Sanity checks: rejects estimates exceeding `max_displacement` (0.5m) or
`max_rotation` (0.3 rad) per frame. Requires minimum 50 points.

## Feature Flags

- **default**: Core types, back-projection, visual odometry. No ML dependencies.
- **onnx**: Adds `DepthEstimator` (ONNX Runtime inference). Requires `ort` crate.
  On Jetson, automatically uses TensorRT execution provider.

```toml
# For development (no model inference)
depth = { path = "crates/depth" }

# For Jetson deployment (with TensorRT)
depth = { path = "crates/depth", features = ["onnx"] }
```

## Integration with bvrd

The depth pipeline is wired into `bins/bvrd/src/init.rs` and
`bins/bvrd/src/run_loop.rs`:

1. **init.rs**: Creates `CameraGeometry` per mount, spawns raw capture threads,
   initializes `VisualOdometry` and `DepthPerceptionConfig`
2. **run_loop.rs**: Each iteration drains latest raw frame per camera, runs
   depth estimation + back-projection, updates costmap, runs VO → EKF

The pipeline is gated on `[depth] enabled = true` in `bvr.toml`. When depth
is enabled alongside lidar, both feed into the same costmap.

## Model Setup

Download Depth Anything V2 Small ONNX:

```bash
# On the Jetson or dev machine
mkdir -p /var/lib/bvr/models
# Download from HuggingFace (Depth-Anything-V2-Small-hf)
# Export to ONNX with input shape [1, 3, 518, 518]
```

On first run with TensorRT EP, ONNX Runtime builds a TRT engine (takes
~2-5 minutes). Subsequent runs load the cached engine instantly.
