# Teleop System

Remote operation of BVR over LTE.

## Architecture

```
┌─────────────────────┐         ┌─────────────────┐         ┌─────────────────────┐
│   Operator Station  │         │   Cloud Relay   │         │        Rover        │
│                     │         │                 │         │                     │
│  ┌───────────────┐  │         │  ┌───────────┐  │         │  ┌───────────────┐  │
│  │ Xbox          │  │  QUIC   │  │ Relay     │  │  QUIC   │  │ teleop crate  │  │
│  │ Controller    │──┼────────►│  │ Server    │──┼────────►│  │               │  │
│  └───────────────┘  │         │  │           │  │         │  │ • UDP server  │  │
│                     │         │  │ • Auth    │  │         │  │ • Commands    │  │
│  ┌───────────────┐  │         │  │ • Logging │  │         │  │ • Telemetry   │  │
│  │ Video View    │◄─┼─────────│  │ • Routing │◄─┼─────────│  │               │  │
│  └───────────────┘  │         │  └───────────┘  │         │  └───────────────┘  │
│                     │         │                 │         │                     │
│  ┌───────────────┐  │         │                 │         │  ┌───────────────┐  │
│  │ Telemetry     │◄─┼─────────┼─────────────────┼─────────┼──│ GStreamer     │  │
│  │ Display       │  │         │                 │         │  │ (video)       │  │
│  └───────────────┘  │         │                 │         │  └───────────────┘  │
└─────────────────────┘         └─────────────────┘         └─────────────────────┘
```

## Xbox Controller Mapping

```
┌─────────────────────────────────────────────────────────────────┐
│                      Xbox Controller                             │
│                                                                  │
│         [LB] ───────────────────────────────── [RB]             │
│            Tool prev                      Tool next              │
│                                                                  │
│    [LT] ─────────────────────────────────────── [RT]            │
│       Tool axis -                         Tool axis +            │
│       (e.g., auger down)                  (e.g., auger up)       │
│                                                                  │
│         ┌───┐                               ┌───┐               │
│         │ L │  Linear velocity              │ R │  (camera pan) │
│         │   │  (forward/back)               │   │               │
│         └─┬─┘                               └───┘               │
│           │                                                      │
│      ◄────┴────►  Angular velocity                              │
│                   (turn left/right)                              │
│                                                                  │
│     [View]          [Menu]           [A] [B] [X] [Y]            │
│      E-Stop          Enable          Tool actions                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

| Input          | Action                               |
| -------------- | ------------------------------------ |
| Left Stick Y   | Linear velocity (forward/back)       |
| Left Stick X   | Angular velocity (turn)              |
| **L3 (click)** | **Boost mode** (hold for full speed) |
| RT             | Tool axis positive (e.g., lift up)   |
| LT             | Tool axis negative (e.g., lift down) |
| A              | Tool action 1 (e.g., toggle auger)   |
| B              | Tool action 2                        |
| LB             | Previous tool                        |
| RB             | Next tool                            |
| View (☰☰)      | **E-Stop** (immediate stop)          |
| Menu (≡)       | Enable/arm system                    |

### Keyboard Controls (Web Operator)

| Key       | Action                               |
| --------- | ------------------------------------ |
| W / ↑     | Forward                              |
| S / ↓     | Backward                             |
| A / ←     | Turn left                            |
| D / →     | Turn right                           |
| **Shift** | **Boost mode** (hold for full speed) |
| E         | Tool axis up                         |
| Q         | Tool axis down                       |
| Space     | Tool action A                        |
| F         | Tool action B                        |
| Escape    | E-Stop                               |
| Enter     | Enable                               |
| C         | Cycle camera mode                    |

### Boost Mode

Hold **L3** (gamepad) or **Shift** (keyboard) to enable boost mode:

| Mode   | Max Duty | Approx. Speed |
| ------ | -------- | ------------- |
| Normal | 50%      | ~3 m/s        |
| Boost  | 95%      | ~6 m/s        |

### Input Processing

```
Linear velocity = left_stick_y × max_linear_speed
Angular velocity = left_stick_x × max_angular_speed
Tool axis = RT - LT  (range -1.0 to +1.0)
```

Dead zones applied to sticks (typically 0.1).

## Protocol

### Command Messages (Operator → Rover)

| Type           | ID   | Payload                                         |
| -------------- | ---- | ----------------------------------------------- |
| Twist          | 0x01 | linear (f64 LE) + angular (f64 LE) + boost (u8) |
| E-Stop         | 0x02 | (none)                                          |
| Heartbeat      | 0x03 | (none)                                          |
| Set Mode       | 0x04 | mode (u8)                                       |
| Tool           | 0x05 | axis (f32) + motor (f32) + actions (u8)         |
| E-Stop Release | 0x06 | (none)                                          |

### Telemetry Messages (Rover → Operator)

| Type   | ID   | Payload                                             |
| ------ | ---- | --------------------------------------------------- |
| Status | 0x10 | mode, voltage, timestamp, velocity, temps, currents |

### Timing

| Parameter          | Value   |
| ------------------ | ------- |
| Command rate       | 50 Hz   |
| Telemetry rate     | 20 Hz   |
| Heartbeat interval | 100 ms  |
| Command timeout    | 250 ms  |
| Connection timeout | 1000 ms |

## Video Streaming

Video uses a separate path from commands/telemetry for efficiency.

### GStreamer Pipeline (Rover)

```bash
gst-launch-1.0 \
  v4l2src device=/dev/video0 ! \
  video/x-raw,width=1280,height=720,framerate=30/1 ! \
  nvvidconv ! \
  nvv4l2h265enc bitrate=2000000 ! \
  rtph265pay ! \
  udpsink host=<relay> port=5000
```

### Latency Budget

| Component        | Typical Latency |
| ---------------- | --------------- |
| Capture + encode | 20-40 ms        |
| LTE uplink       | 30-80 ms        |
| Relay            | 1-5 ms          |
| LTE downlink     | 30-80 ms        |
| Decode + display | 20-40 ms        |
| **Total**        | **100-250 ms**  |

## Safety Considerations

### Collision Guard

The collision guard automatically scales down teleop linear velocity when
obstacles are detected along the projected trajectory. It acts as a safety
layer between the operator's commands and the motors — the operator can always
steer, but the system prevents driving into walls.

#### How It Works

```
Operator command ──► Collision Guard ──► Motor mixer
                         │
                    Costmap lookup
                    (from LiDAR)
```

1. **Project trajectory** — the current commanded velocity (linear + angular) is
   forward-simulated as a differential-drive arc over the lookahead window
2. **Sample costmap** — 10 points along the arc are checked against the 2D
   costmap built from LiDAR
3. **Find nearest obstacle** — the closest sample with cost ≥ 253 (INSCRIBED)
   determines the obstacle distance
4. **Scale linear velocity** — linearly interpolated between full speed and zero
   based on obstacle distance within the scaling zone

#### Parameters

| Parameter              | Default | Description                               |
| ---------------------- | ------- | ----------------------------------------- |
| `enabled`              | `true`  | Enable/disable the guard                  |
| `lookahead_time`       | 1.0 s   | How far ahead to project the arc          |
| `num_samples`          | 10      | Sample points along the arc               |
| `full_speed_distance`  | 1.0 m   | No scaling beyond this distance           |
| `stop_distance`        | 0.15 m  | Full stop below this distance             |
| `cost_threshold`       | 253     | Costmap cost that counts as blocked       |

Configured in `bvr.toml` under `[collision_guard]`.

#### Scaling Curve

```
Linear velocity scale
  1.0 ┤ ████████████████████
      │                     ╲
      │                      ╲  scaling zone
      │                       ╲
  0.0 ┤────────────────────────╳─────────
      0m    stop         full_speed    ∞
           0.15m          1.0m
           ◄─── obstacle distance ───►
```

- **> 1.0 m**: Full speed (scale = 1.0)
- **0.15 – 1.0 m**: Linear ramp from 1.0 → 0.0
- **< 0.15 m**: Full stop (scale = 0.0)

Angular velocity is **never** scaled — the operator can always steer away from
obstacles. Only forward/backward motion is limited.

#### Console Visualization

The console replicates the collision guard algorithm client-side using the same
costmap data and commanded velocity, providing two visual indicators:

**3D Arc Overlay** — a solid line on the ground plane showing the projected
trajectory, colored per-segment:
- **Green** — clear, no scaling applied
- **Yellow** — within scaling zone, velocity being reduced
- **Red** — obstacle hit, arc truncated at this point

The arc is only visible in Teleop mode when the costmap is active and the rover
is moving.

**Proximity Bar** — a thin horizontal bar in the StatusPanel showing clearance
along the current trajectory:
- Full width = `full_speed_distance` (1.0 m)
- Fill color matches the arc (green → yellow → red)
- Label shows distance in meters or "clear"

#### Implementation

| Component | Location |
| --------- | -------- |
| Firmware guard | `bvr/firmware/crates/control/src/lib.rs` — `CollisionGuard::scale()` |
| Configuration | `bvr/firmware/config/bvr.toml` — `[collision_guard]` |
| 3D arc overlay | `depot/console/src/components/scene/CollisionGuardArc.tsx` |
| Guard state store | `depot/console/src/lib/collisionGuardStore.ts` |
| Proximity bar | `depot/console/src/components/teleop/StatusPanel.tsx` |

### Tab Visibility (Web Operator)

When the browser tab loses focus (Alt+Tab, switch tabs, minimize):

1. **Immediate zero velocity** sent to rover
2. Commands continue at zero while hidden
3. Rover stays in Teleop mode (no watchdog timeout)
4. Control resumes when tab regains focus

This prevents accidental movement when the operator looks away.

### Latency Handling

- Display latency indicator to operator
- Reduce max speed when latency is high
- Audio warning if latency > 500ms

### Connection Loss

1. No heartbeat for 100ms → Warning
2. No heartbeat for 250ms → Safe stop (coast)
3. No heartbeat for 1000ms → Full stop + hold

### E-Stop Behavior

1. E-Stop command received
2. State machine → EStop mode
3. All motor commands → 0
4. Tools disabled
5. Remains in EStop until explicit release

## Cloud Relay

For NAT traversal, a cloud relay forwards packets between operator and rover.

### Requirements

- Low latency (< 10ms added)
- Handles packet loss gracefully
- Authentication per rover
- Logging for debugging

### Deployment Options

| Platform | Notes                         |
| -------- | ----------------------------- |
| Fly.io   | Edge locations, easy deploy   |
| Railway  | Simple, good DX               |
| AWS/GCP  | More control, more complexity |

### Relay Protocol

1. Rover connects with API key
2. Relay assigns session ID
3. Operator connects with session ID
4. Relay forwards packets bidirectionally
5. Disconnection → notify other party
