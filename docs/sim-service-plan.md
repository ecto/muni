# Simulation Service + Neural Autonomy Foundation

## Vision

Split simulation out of bvrd into a standalone service that:
1. Provides synthetic sensors over real hardware protocols (bvrd unchanged)
2. Loads 3DGS reconstructions from real mapping sessions as "worlds"
3. Exposes a gym-style training API for learning a world model (DreamerV3-style)
4. Becomes the foundation for replacing the classical autonomy stack with a unified neural model

The loop: **map real world -> reconstruct 3DGS -> simulate in reconstruction -> train world model -> plan in latent space -> deploy on rover**.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  sim-world  (Python/GPU Docker container)                   │
│                                                             │
│  - Loads 3DGS .ply worlds (from splat-worker output)        │
│  - Renders depth from arbitrary poses (gsplat rasterizer)   │
│  - Serves depth buffers via ZMQ to sim-bridge               │
│  - REST control API (:4900) - world select, spawn, reset    │
│  - Gym training API (:4901) - step/reset/observe/reward     │
│  - Batch env support for parallel training rollouts         │
└────────────────────────┬────────────────────────────────────┘
                         │ ZMQ (depth buffers, <1ms latency)
┌────────────────────────┴────────────────────────────────────┐
│  sim-bridge  (Rust Docker container)                        │
│                                                             │
│  - 2D diff-drive physics at 100Hz (from sim crate)          │
│  - VESC CAN simulation on vcan0 (from sim crate)            │
│  - Depth buffer -> 3D point cloud unprojection              │
│  - Livox SDK2 UDP packet construction (ports 56301/56401)   │
│  - IMU synthesis from physics state                         │
│  - Geometric raycast fallback (parry3d, no GPU needed)      │
└─────────┬──────────────────────────────────┬────────────────┘
     vcan0 (CAN frames)              UDP (Livox packets)
          │                                  │
┌─────────┴──────────────────────────────────┴────────────────┐
│  bvrd  (UNMODIFIED)                                         │
│  --can-interface vcan0                                      │
│  --lidar-ip <sim-bridge-ip>                                 │
└─────────────────────────────────────────────────────────────┘
```

### Why Two Containers

- **sim-world** needs GPU + Python (PyTorch, gsplat) for 3DGS rendering. Mirrors existing `splat-worker` container.
- **sim-bridge** needs deterministic 100Hz timing for physics, CAN protocol emulation, and Livox UDP packet construction. Rust is ideal and can reuse code directly from `bvr/firmware/crates/sim/`.
- sim-bridge works standalone (geometric mode, no GPU) or with sim-world (3DGS mode).

### Zero Firmware Changes

- **LiDAR**: sim-bridge sends Livox SDK2 UDP packets to bvrd's listen ports. Same binary format as `parse_point_packet()` in `bvr/firmware/crates/lidar/src/driver.rs:656`.
- **CAN/Motors**: Linux vcan interface. bvrd opens `vcan0` instead of `can0`. Config-only change in `bvr.toml`.
- **IMU**: Sent on UDP port 56401 in same format bvrd parses.

---

## World Model

### Primary: 3DGS from Real Data

Worlds are gaussian splat .ply files produced by the existing splat-worker pipeline. Stored in `/data/maps/<name>/splat.ply`. Each world gets a `world.toml`:

```toml
[world]
name = "Sidewalk Mission Alpha"
origin_gps = [37.7749, -122.4194]
ground_z = 0.0

[spawn_points]
default = { x = 0.0, y = 0.0, theta = 0.0 }

[bounds]
min = [-50.0, -50.0]
max = [50.0, 50.0]
```

### Fallback: Geometric (parry3d)

Existing `sim/src/world.rs` provides box/wall obstacles with raycasting. Works without GPU. Good for unit tests and CI.

### LiDAR Rendering Pipeline

```
1. sim-bridge sends current pose to sim-world via ZMQ
2. sim-world renders 4 cubemap faces (90deg each, 256x128px) via gsplat
   - Covers 360deg horizontal, -30 to +30 vertical (Mid-360 FOV)
   - ~2-5ms on modern GPU
3. Returns 4 depth buffers to sim-bridge via ZMQ
4. sim-bridge unprojects pixels to 3D points in rover frame
5. Applies noise (2cm gaussian, 1% dropout) from existing LidarConfig
6. Packs into Livox SDK2 Cartesian-32 packets:
   - Header: 36 bytes (version, length, dot_num, frame_cnt, timestamp_ns)
   - Points: 14 bytes each (x_mm:i32, y_mm:i32, z_mm:i32, reflectivity:u8, tag:u8)
   - 96 points per packet
7. Sends to bvrd UDP port 56301 at 10Hz
8. Synthesizes IMU from physics (gyro_z = angular_vel, accel = gravity) -> port 56401
```

---

## Training API (Gym Interface)

For DreamerV3-style world model training. Exposed by sim-world on port 4901.

```python
# Python client usage
env = MuniSimEnv(host="sim-world:4901", world="sidewalk-alpha")

obs = env.reset(spawn="default")
# obs = { "lidar": np.array(...), "odom": np.array([x, y, theta, v, w]), "imu": np.array(...) }

for step in range(max_steps):
    action = agent.act(obs)  # action = [linear_vel, angular_vel]
    obs, reward, done, info = env.step(action)
    # reward = progress_to_goal - collision_penalty - jerk_penalty
    # info = { "pose": [x,y,theta], "collision": bool, "goal_dist": float }
```

### Key Design Decisions

- **Observation space**: Raw LiDAR point cloud (N x 3), odometry (5,), IMU (6,). NOT pre-processed costmaps — the neural model learns its own representation.
- **Action space**: Continuous [linear_vel, angular_vel]. Matches the twist command bvrd accepts.
- **Reward**: Goal-conditioned. `r = progress_toward_goal - collision * 10 - jerk * 0.1`
- **Batch mode**: Multiple environments on one GPU. sim-world renders depth for N rovers in parallel via batched gsplat calls. Target: 1000+ env steps/sec for training.
- **Episode**: Terminates on goal reach, collision, timeout, or out-of-bounds.

### Training Loop Architecture

```
┌─────────────────────────────────────────────┐
│  Training Script (Python)                    │
│  - DreamerV3 / similar world model algo      │
│  - Collects trajectories from batch envs     │
│  - Learns: encoder, dynamics, reward, decoder │
│  - Plans in latent space at inference time    │
└──────────────────┬──────────────────────────┘
                   │ step(action) / reset()
┌──────────────────┴──────────────────────────┐
│  sim-world batch API                         │
│  - N parallel environments on 1 GPU          │
│  - Batched gsplat rendering                  │
│  - Vectorized physics (or delegate to bridge)│
│  - Returns obs tensors directly (no UDP)     │
└─────────────────────────────────────────────┘
```

For training, the gym API bypasses sim-bridge entirely — no UDP packets, no CAN frames. Direct tensor-to-tensor for speed. sim-bridge is only needed when testing against real bvrd.

---

## Files to Create

### `depot/sim-world/` (Python)

```
depot/sim-world/
  Dockerfile              # GPU container (base from splat-worker)
  requirements.txt        # torch, gsplat, numpy, pyzmq, fastapi, gymnasium
  src/
    __init__.py
    server.py             # Main: FastAPI + ZMQ depth publisher
    renderer.py           # gsplat cubemap depth rendering
    world_loader.py       # Load .ply into GPU tensors
    api.py                # REST control endpoints (world select, spawn, status)
    gym_env.py            # Gymnasium-compatible env (step/reset/observe)
    gym_server.py         # Network gym API for remote training
    batch_env.py          # Vectorized env for parallel rollouts
    rewards.py            # Reward function definitions
```

### `depot/sim-bridge/` (Rust)

```
depot/sim-bridge/
  Cargo.toml
  Dockerfile              # Alpine musl static build
  src/
    main.rs               # Entry: physics loop, ZMQ client, UDP sender
    physics.rs            # Copied from bvr/firmware/crates/sim/src/physics.rs
    vesc.rs               # Copied from bvr/firmware/crates/sim/src/vesc.rs
    world.rs              # Copied from bvr/firmware/crates/sim/src/world.rs
    lidar_render.rs       # Depth buffer -> point cloud unprojection
    lidar_noise.rs        # Noise model from sim/src/lidar.rs LidarConfig
    livox_protocol.rs     # Construct Livox SDK2 UDP packets (inverse of driver.rs parse)
    can_bridge.rs         # vcan0 interface + VESC sim loop
    imu_sim.rs            # IMU from physics angular_vel + gravity
    zmq_client.rs         # ZMQ connection to sim-world
    config.rs             # CLI args, env vars
```

### Docker Compose Addition

```yaml
# depot/docker-compose.yml additions
sim-world:
  build: ./sim-world
  profiles: [sim]
  ports: ["4900:4900", "4901:4901"]
  volumes:
    - maps-data:/data/maps:ro
  deploy:
    resources:
      reservations:
        devices:
          - driver: nvidia
            count: 1
            capabilities: [gpu]

sim-bridge:
  build: ./sim-bridge
  profiles: [sim]
  network_mode: host
  cap_add: [NET_ADMIN]
  depends_on: [sim-world]
```

Launch: `docker compose --profile sim up -d`

---

## Code Reuse

| Source | Destination | Strategy |
|--------|-------------|----------|
| `sim/src/physics.rs` (236 lines) | `sim-bridge/src/physics.rs` | Copy, remove World import |
| `sim/src/vesc.rs` (162 lines) | `sim-bridge/src/vesc.rs` | Copy, bring CAN Frame type |
| `sim/src/world.rs` (~200 lines) | `sim-bridge/src/world.rs` | Copy for geometric fallback |
| `sim/src/lidar.rs` LidarConfig/noise | `sim-bridge/src/lidar_noise.rs` | Extract noise params |
| `lidar/src/driver.rs:656-749` | `sim-bridge/src/livox_protocol.rs` | Reverse: construct packets |
| `splat-worker/src/worker.py` PLY format | `sim-world/src/world_loader.py` | Read same 14-float format |
| `splat-worker/Dockerfile` | `sim-world/Dockerfile` | Same CUDA/PyTorch base |

---

## Implementation Phases

### Phase 1: Geometric Sim Service (sim-bridge only)

- Port physics, vesc, world, lidar from sim crate into sim-bridge
- Implement `livox_protocol.rs` — construct valid Livox SDK2 packets
- Implement `can_bridge.rs` — vcan0 + VESC sim loop
- Implement `imu_sim.rs` — IMU from physics
- Main loop: 100Hz physics, 10Hz geometric raycast, Livox UDP output
- Dockerfile + docker-compose
- **Test**: bvrd connects via vcan0, receives LiDAR on UDP, runs SLAM

### Phase 2: 3DGS Depth Rendering (add sim-world)

- Implement `world_loader.py` — load splat.ply into gsplat tensors
- Implement `renderer.py` — cubemap depth rendering at arbitrary poses
- ZMQ depth publisher + sim-bridge ZMQ client
- Mode switching in sim-bridge (geometric vs 3DGS)
- **Test**: bvrd sees LiDAR from a real reconstructed environment

### Phase 3: Training API

- Implement `gym_env.py` — Gymnasium env wrapping sim-world physics + rendering
- Implement `batch_env.py` — vectorized parallel envs on GPU
- Implement `rewards.py` — goal-conditioned reward functions
- Implement `gym_server.py` — network API for remote training
- **Test**: Train a simple policy (PPO) on point-to-point navigation

### Phase 4: World Model Training

- Integrate DreamerV3 (or DIAMOND) with the gym API
- Train encoder (LiDAR -> latent), dynamics model (latent + action -> latent), reward predictor
- Evaluate: does the world model predict reasonable future observations?
- **Test**: World model imagination matches sim rollouts

### Phase 5: Neural Autonomy Deployment

- Replace classical planner with latent-space planning (CEM/MPPI in world model)
- Sim-to-real transfer: fine-tune on real data, domain randomization
- Deploy on rover alongside classical stack (shadow mode first)
- **Goal**: Unified sensor->action model replaces SLAM + costmap + planner + pursuit

---

## Key Risks

| Risk | Mitigation |
|------|-----------|
| gsplat depth rendering too slow for 10Hz | Reduce resolution, render every other frame, geometric fallback always works |
| vcan in Docker needs NET_ADMIN + host network | Document setup; fallback to existing --sim flag for local dev |
| Livox packet format mismatch | Integration test: construct packets, parse with existing driver.rs parser |
| ZMQ latency between containers | <1ms on localhost; if issues, single-container option via PyO3 |
| Sim-to-real gap for neural model | Domain randomization (noise, lighting, obstacles), fine-tune on real data |
| 3DGS PLY format drift | Pin to splat-worker's export format (14 x float32 per vertex) |

---

## Verification

### Phase 1 Smoke Test
```bash
# Terminal 1: Start sim-bridge with geometric world
docker compose --profile sim up sim-bridge

# Terminal 2: Start bvrd pointing at sim
cargo run --bin bvrd -- --can-interface vcan0 --log-dir ./logs

# Terminal 3: Send velocity command
muni rover drive --linear 0.5 --angular 0.0

# Verify: bvrd logs show LiDAR scans arriving, SLAM processing, costmap updating
```

### Phase 2 Smoke Test
```bash
# Start full sim stack with a real-world map
docker compose --profile sim up -d
curl -X POST http://localhost:4900/worlds/sidewalk-alpha/load
curl -X POST http://localhost:4900/rover/spawn -d '{"x":0,"y":0,"theta":0}'

# bvrd should see rich LiDAR from the gaussian splat world
```

### Phase 3 Smoke Test
```python
import gymnasium
env = gymnasium.make("muni-sim-v0", world="sidewalk-alpha")
obs, info = env.reset()
for _ in range(100):
    obs, reward, done, trunc, info = env.step(env.action_space.sample())
    if done: break
# Should complete without errors, obs shapes correct
```
