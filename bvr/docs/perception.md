# Perception & Mapping

BVR's perception system for safety, situational awareness, and 3D reconstruction.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Perception Pipeline                             │
│                                                                              │
│  ┌─────────────────┐         ┌─────────────────┐         ┌───────────────┐ │
│  │  Livox Mid-360  │         │   Insta360 X4   │         │   Jetson      │ │
│  │  (LiDAR)        │         │   (360° Camera) │         │   Orin NX     │ │
│  │                 │         │                 │         │               │ │
│  │  • 200k pts/s   │         │  • 5.7K @ 30fps │         │  • 100 TOPS   │ │
│  │  • 360° × 59°   │         │  • 360° × 360°  │         │  • Ampere GPU │ │
│  │  • 10 Hz        │         │  • USB-C UVC    │         │               │ │
│  └────────┬────────┘         └────────┬────────┘         └───────────────┘ │
│           │                           │                                     │
│           ▼                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Processing Layers                             │   │
│  │                                                                       │   │
│  │  Layer 1: SAFETY (< 10ms latency)                                    │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐│   │
│  │  │ Point cloud → radius filter → E-Stop trigger                    ││   │
│  │  │ No ML, no uncertainty — pure geometry                           ││   │
│  │  └─────────────────────────────────────────────────────────────────┘│   │
│  │                                                                       │   │
│  │  Layer 2: AWARENESS (< 100ms latency)                                │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐│   │
│  │  │ 360° video → encode → stream to operator                        ││   │
│  │  │ Point cloud → optional classification (person/obstacle/etc)     ││   │
│  │  └─────────────────────────────────────────────────────────────────┘│   │
│  │                                                                       │   │
│  │  Layer 3: MAPPING (offline / background)                             │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐│   │
│  │  │ LiDAR + Camera → recording → post-processing → Gaussian splat   ││   │
│  │  │ Or: real-time splatting for live 3D (bvr1 goal)                 ││   │
│  │  └─────────────────────────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Layer 1: Safety

**Goal**: Stop before hitting anything, regardless of conditions.

### Implementation

```rust
// crates/lidar/src/safety.rs

pub struct SafetyZone {
    /// Inner radius — immediate stop
    pub stop_radius: f32,      // 1.0m
    /// Outer radius — slow down
    pub slow_radius: f32,      // 2.5m
    /// Height band to check (ignore ground, ignore high objects)
    pub height_min: f32,       // 0.1m
    pub height_max: f32,       // 1.8m
}

impl SafetyZone {
    pub fn check(&self, points: &[Point3]) -> SafetyStatus {
        for point in points {
            // Filter by height
            if point.z < self.height_min || point.z > self.height_max {
                continue;
            }

            let distance = (point.x.powi(2) + point.y.powi(2)).sqrt();

            if distance < self.stop_radius {
                return SafetyStatus::Stop;
            }
            if distance < self.slow_radius {
                return SafetyStatus::Slow;
            }
        }
        SafetyStatus::Clear
    }
}
```

### Latency Requirements

| Stage             | Budget | Actual        |
| ----------------- | ------ | ------------- |
| LiDAR acquisition | 100ms  | 10ms (100 Hz) |
| Point filtering   | 10ms   | <1ms          |
| Decision          | 1ms    | <0.1ms        |
| CAN command       | 5ms    | 1ms           |
| Motor response    | 50ms   | 10ms          |
| Mechanical stop   | 500ms  | ~200ms        |

**Total: ~225ms** — well under requirements for 1 m/s operation.

## LiDAR Data Pipeline

The Mid-360 streams point cloud and IMU data to the Jetson for processing.

### Data Flow

```
Mid-360
   │
   ├── UDP 56000 ───► Point Cloud Parser ───► Safety Check ───► E-Stop
   │                         │
   │                         └──────────────► Recording ───► .pcd files
   │
   └── UDP 56001 ───► IMU Parser ───► Pose Estimator ───► Telemetry
                            │
                            └──────────────► Recording ───► .rrd
```

### lidar Crate (To Be Implemented)

```rust
// crates/lidar/src/lib.rs

pub struct Mid360 {
    socket: UdpSocket,
    config: Config,
}

impl Mid360 {
    /// Connect to Mid-360 at given IP
    pub fn connect(ip: &str) -> Result<Self, LidarError>;

    /// Get next point cloud frame (blocking)
    pub fn recv_points(&mut self) -> Result<PointCloud, LidarError>;

    /// Get next IMU sample (blocking)
    pub fn recv_imu(&mut self) -> Result<ImuSample, LidarError>;
}

pub struct PointCloud {
    pub points: Vec<Point3>,
    pub timestamp_ns: u64,
}

pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: u8,
    pub return_count: u8,
}
```

### Integration with bvrd

```rust
// In bvrd main loop

// Spawn LiDAR processing thread
let lidar = Mid360::connect("192.168.1.10")?;
let (points_tx, points_rx) = channel();

std::thread::spawn(move || {
    loop {
        if let Ok(cloud) = lidar.recv_points() {
            let _ = points_tx.send(cloud);
        }
    }
});

// In control loop
while let Ok(cloud) = points_rx.try_recv() {
    let status = safety_zone.check(&cloud.points);
    if status == SafetyStatus::Stop {
        state.trigger_estop();
    }
}
```

## Layer 2: Situational Awareness

**Goal**: Operator sees everything, can identify what's around the rover.

### 360° Video Streaming

The Insta360 X4 provides equirectangular video that can be:

1. **Streamed to operator** — rendered as a skybox in the 3D scene
2. **Used for classification** — ML models identify objects in view
3. **Recorded for mapping** — stored alongside LiDAR for later processing

#### Streaming Architecture

```
Insta360 X4 (USB UVC)
       │
       ▼ v4l2src
┌─────────────────┐
│ camera crate    │
│                 │
│ • Equirect cap  │
│ • JPEG encode   │
│ • Frame buffer  │
└────────┬────────┘
         │
         ▼ WebSocket
┌─────────────────┐         ┌─────────────────┐
│ teleop/ws.rs    │ ──────► │ Operator App    │
│                 │         │                 │
│ • Video frames  │         │ • Decode JPEG   │
│ • Telemetry     │         │ • Update skybox │
│ • Commands      │         │ • 3D scene      │
└─────────────────┘         └─────────────────┘
```

#### Operator App Rendering

The 360° feed is rendered as an environment map (skybox) in Three.js:

```tsx
// Equirectangular texture as environment
<Environment
  background
  files={videoTextureUrl}
  encoding={sRGBEncoding}
/>

// Or custom sphere with video texture
<mesh>
  <sphereGeometry args={[500, 64, 32]} />
  <meshBasicMaterial
    map={videoTexture}
    side={BackSide}
  />
</mesh>
```

The rover model and UI are rendered on top, creating an immersive telepresence
experience.

### Operator UI Enhancements

The 360° video provides situational awareness but lacks depth perception. These
enhancements help operators judge distances:

#### Top-Down Minimap

Display LiDAR points as a bird's eye view, colored by distance:

```
┌─────────────────────────────┐
│         Top-Down View       │
│                             │
│    ·  ·  ·  ·  ·  ·  ·     │  Green: > 3m
│  ·                    ·     │  Yellow: 1.5-3m
│ ·   ┌─────────┐      ·     │  Red: < 1.5m (danger)
│ ·   │  ROVER  │      ·     │
│ ·   │    ▲    │      ·     │
│ ·   └─────────┘      ·     │
│  ·                    ·     │
│    ·  ·  ·  ·  ·  ·  ·     │
│                             │
│     ○ 1.5m safety zone      │
└─────────────────────────────┘
```

#### Distance Overlays

Overlay nearest obstacle distance on 360° view sectors:

```
┌─────────────────────────────────────────────────────────────┐
│                       360° Video Feed                        │
│                                                              │
│   [2.3m]              [CLEAR]              [0.8m] ⚠️        │
│     ↓                   ↓                    ↓              │
│  ┌──────┐           ┌──────┐            ┌──────┐           │
│  │ wall │           │ open │            │person│           │
│  └──────┘           └──────┘            └──────┘           │
└─────────────────────────────────────────────────────────────┘
```

#### Implementation (React Three Fiber)

```tsx
// components/scene/Minimap.tsx
function Minimap({ points }: { points: Point3[] }) {
  return (
    <div className="absolute bottom-20 right-4 w-48 h-48 bg-black/50 rounded-lg">
      <Canvas orthographic camera={{ zoom: 20 }}>
        {/* Safety zone ring */}
        <mesh rotation={[-Math.PI / 2, 0, 0]}>
          <ringGeometry args={[1.4, 1.5, 32]} />
          <meshBasicMaterial color="red" transparent opacity={0.3} />
        </mesh>

        {/* LiDAR points */}
        <Points positions={points} colors={distanceColors} />

        {/* Rover indicator */}
        <mesh>
          <coneGeometry args={[0.2, 0.4, 3]} />
          <meshBasicMaterial color="white" />
        </mesh>
      </Canvas>
    </div>
  );
}
```

#### VR/XR Mode

The operator app supports WebXR for immersive telepresence:

- **Vision Pro / Quest**: Enter VR mode for full 360° immersion
- **Head tracking**: Look around naturally in the 360° feed
- **Depth cues**: LiDAR overlay provides spatial understanding

```tsx
// Already implemented in Scene.tsx
<XR store={xrStore}>
  <EquirectangularSky /> {/* 360° video as environment */}
  <RoverModel />
</XR>
```

## Layer 3: Mapping & Gaussian Splatting

**Goal**: Build photorealistic 3D maps for documentation and future autonomy.

### bvr0 Approach (Prototype)

Record data, process offline:

1. **During operation**: Record LiDAR + camera to `.rrd` files
2. **After operation**: Sync to Depot via SFTP
3. **Offline**: Run Gaussian splatting pipeline on workstation
4. **Output**: `.ply` splat files for visualization

```bash
# Example offline processing
python train_splat.py \
  --lidar sessions/2024-01-15/lidar.rrd \
  --video sessions/2024-01-15/video.mp4 \
  --output splats/sidewalk_01.ply
```

### bvr1 Approach (Production)

Real-time splatting on Orin:

1. **Live**: Stream LiDAR + camera to splatting pipeline
2. **Incremental**: Update Gaussian splat as rover moves
3. **Stream**: Send splat updates to operator app
4. **Render**: Real-time novel view synthesis in browser

Candidate frameworks:

- **[gsplat](https://github.com/nerfstudio-project/gsplat)** — CUDA splatting library
- **[GS-LIVM](https://github.com/xieyuser/GS-LIVM)** — LiDAR-visual Gaussian mapping
- **[Splat-SLAM](https://github.com/eriksandstroem/Splat-SLAM)** — RGB-only Gaussian SLAM

### Splat Rendering in Browser

For displaying Gaussian splats in the operator app:

- **[gsplat.js](https://github.com/huggingface/gsplat.js)** — WebGL splat renderer
- **[antimatter15/splat](https://github.com/antimatter15/splat)** — Lightweight WebGL viewer
- **Custom Three.js** — Point-based rendering with Gaussian blur

```tsx
// Example: Load and render splat in Three.js
import { PLYLoader } from "three/examples/jsm/loaders/PLYLoader";

function SplatViewer({ url }: { url: string }) {
  const [geometry, setGeometry] = useState<BufferGeometry | null>(null);

  useEffect(() => {
    new PLYLoader().load(url, (geo) => {
      setGeometry(geo);
    });
  }, [url]);

  if (!geometry) return null;

  return (
    <points geometry={geometry}>
      <pointsMaterial size={0.01} vertexColors sizeAttenuation />
    </points>
  );
}
```

## Data Recording

All sensor data is recorded for later analysis and training:

### Recording Format

```
sessions/
└── 2024-01-15T14-30-00/
    ├── metadata.json       # Session info, rover ID, etc.
    ├── telemetry.rrd       # Rerun recording (pose, velocity, etc.)
    ├── lidar/
    │   ├── 000000.pcd      # Point cloud frames
    │   ├── 000001.pcd
    │   └── timestamps.csv
    ├── camera/
    │   ├── 000000.jpg      # Equirectangular frames
    │   ├── 000001.jpg
    │   └── timestamps.csv
    └── calibration.json    # LiDAR-camera extrinsics
```

### Synchronization

For bvr0 (software sync):

- Timestamp each frame with system time
- Interpolate LiDAR poses to camera timestamps during processing
- Accept ~10-50ms timing uncertainty

For bvr1 (hardware sync):

- Mid-360 PPS output triggers camera capture
- Microsecond-level synchronization
- Required for high-quality splatting

## Calibration

### LiDAR-Camera Extrinsic

The transform from LiDAR frame to camera frame:

```json
{
  "lidar_to_camera": {
    "rotation": [0, 0, 0, 1], // Quaternion (x, y, z, w)
    "translation": [0, 0, 0.15] // Meters (camera above LiDAR)
  }
}
```

For bvr0, this is measured manually. For bvr1, use automatic calibration:

- [lidar_camera_calib](https://github.com/ankitdhall/lidar_camera_calibration)
- [direct_visual_lidar_calibration](https://github.com/koide3/direct_visual_lidar_calibration)

### Camera Intrinsics

The Insta360 X4 provides pre-calibrated intrinsics in its metadata. For custom
cameras, use:

```bash
# OpenCV checkerboard calibration
python calibrate_camera.py --input images/ --output camera_intrinsics.json
```

## Next Steps

1. **bvr0**: Get basic 360° streaming working in operator app
2. **bvr0**: Record LiDAR + camera data during teleop sessions
3. **bvr0**: Process recorded data with offline splatting pipeline
4. **bvr1**: Investigate real-time splatting on Orin NX
5. **bvr1**: Upgrade to hardware-synced global shutter cameras
