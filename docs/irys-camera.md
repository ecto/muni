# irys sim camera: the K1's head, rendered in-process

`docs/sim-service-plan.md` planned for a Python `sim-world` container to render depth from a 3DGS `.ply` with gsplat and ship it to `depot/sim-bridge` over ZMQ. This note replaces that render path with [irys](https://github.com/ecto/irys), running in-process in Rust. It produces batched depth and RGB cameras for the sim service and for RL, with no Python, CUDA or ZMQ.

The simulated robot is ipse's **Booster K1** humanoid, not the BVR. The sensor is the K1's head stereo camera.

![chase view, head RGB, head depth](irys-camera-demo.png)

*Chase view, then head RGB at 64 spp, then head depth (near is bright, black is no return within 20 m). All three are 544×448 renders of the K1 walking the snowy sidewalk.*

## What muni needs from a sim camera

| | K1 head camera (real robot) | Rendered |
|---|---|---|
| Sensor | Stereo pair in the head (ZED-style module, `Head_2_ZED` mesh) | Left eye, centre or right eye |
| Resolution | 544×448 per eye | 544×448, or any size via `HeadCamera::scaled` |
| Intrinsics | fx = fy = 204.639, cx = 277.811, cy = 221.246, no distortion | Same focal length; principal point centred (see API changes) |
| FOV | 106° H × 95° V | Same |
| Baseline | 0.0525 m | 0.0525 m (`Eye::Left/Right`) |
| Rates | Raw eyes ~25 Hz, vendor StereoNet depth ~17.7 Hz | Well above both (see numbers) |
| Mount | ipse `head_extrinsics`: `Head_2` link + (0.06, 0, 0.03) m, looking along link +x | Same |
| Depth | mono16 mm, aligned to the left/colour frame | Metric `f32` z-depth, plus a 16-bit mm PNG in the demo |

The camera numbers come from ipse's `docs/camera.md` (measured on the robot) and `ipse-sim::see::head_extrinsics`. The URDF has no camera frame or intrinsics.

What consumes the output:

- **sim-bridge's LiDAR path.** Livox packets go to bvrd's costmap/SLAM at 10 Hz, about 2.9k points per scan.
- **RL.** The batch sizes are 1 for debugging, 8 for a laptop, and 64 for training. The gym plan's "1000+ env steps/s" target holds only if rendering is not every step (see the numbers).

## How irys fits

- **In-process.** It is a Rust library on wgpu 30, so there is no container, network hop or second language. It uses Metal on macOS and Vulkan elsewhere.
- **Deterministic.** irys renders bit-identically across runs for a given seed. The depth path has no randomness that matters, because the pixel jitter is ≤ ½ px.
- **Depth + RGB, plus semantic id later.** Depth uses irys's cheapest path: one primary ray per pixel on hardware ray queries, with no shading or light sampling. RGB is the full OpenPBR spectral path tracer. A per-pixel material id already exists as irys debug view 1 (a hash), and a real instance-id AOV is a small irys change (listed below).
- **Differentiable later.** irys-diff's wavefront (`GpuPrb`) carries gradients. This camera uses the forward-only `look` renderer today.

## What replaces gsplat

The world is **meshes and boxes now**, not reconstructed splats. irys has splats (Phase 2b: `irys-prims::Splats`, a splat BVH in `GpuPrb`), but they are parked: there is no `.ply` loader, the OpenPBR `look` path does not render them, and they refuse NEE.

**What is lost for now:**

- **Photoreal reconstructions of real mapped sidewalks.** The `splat-worker` → `splat.ply` → world loop does not run.
- **Appearance and geometry from real data.** The street here is authored: road, curb, sidewalk with uncleared snow edges, ploughed windrow, buried lawn, lamp posts, hydrant, bench, brick walls.
- **Snow.** It is a white OpenPBR with subsurface (weight 0.7, radius 5 cm) and fuzz (0.35) set. irys currently shades subsurface as a Lambertian of `subsurface_color`, so it reads as matte white with a sheen. There is no sparkle and no volume.

**The plan back to real worlds:**

1. **Meshed reconstructions.** Export a mesh from the splat-worker run (TSDF or 2DGS surface) and load it as a `look::scene::Mesh`. This works with today's irys.
2. **Splats in `look`.** Add an irys `.ply` loader and splat any-hit in `scene.wgsl`, then render splat worlds directly.
3. **Collision.** sim-bridge's parry world stays the physics truth. In irys mode the scenario's obstacles are also rendered as boxes, so what the camera sees and what the rover hits agree.

## The API

The crate is `depot/muni-irys`, a standalone crate like the other depot services.

```rust
use muni_irys::{K1Model, Street, HeadCamera, SimCamera, Eye, street::SIDEWALK_Z};
let k1 = K1Model::load_default()?;                   // MUNI_K1_DIR or ~/Developer/booster_assets/robots/K1
let pose = k1.standing(x, y, yaw, roll, pitch, q, SIDEWALK_Z);  // q: joint name -> rad; rests on its lowest vertex
let mut sim = SimCamera::new(k1, &Street::snowy(80.0), HeadCamera::K1, vec![pose; 64])?;
let depth = sim.depth_batch(Eye::Left);              // Vec<DepthImage>: metric z, 0 = no return
let rgb   = sim.rgb_batch(Eye::Left, 16, 1.0);       // Vec<RgbImage>: sRGB8
sim.set_poses(next_poses)?;                          // every K1 moves (forward kinematics)
let cloud = depth[0].points(&sim.cam);               // optical-frame points
```

**Articulation.** Each URDF `<visual>` STL is one irys mesh (one BLAS). Each link is placed by forward kinematics (`urdf::Robot::fk`) as one TLAS instance per environment.

- **Joint angles:** from the caller, or from an ipse bus trace (`trace::read`, the `ipse-bus-trace` v1–v3 format, `K1_SERIAL_JOINTS` order).
- **Base pose:** IMU roll/pitch from the trace. Odometer xy/θ is opt-in in the demo, because a seated trace's odometer drifts metres.

**Batching.** Environment *i* is a copy of the street plus one K1, offset `i × 40 m` along y, all in one scene. Bottom-level meshes are shared, and each copy costs only instances.

**Assets.** The K1 URDF and meshes are Booster's (BSD-3-Clause) and are **not copied into muni**. They load by path, `MUNI_K1_DIR` overrides it, and the ZED variant `K1_22dof-ZED.urdf` is the default.

**Scene.** muni's snowy sidewalk, not an ipse map. The point is the snow-clearing setting, and an ipse-map scene can be loaded as meshes the same way later.

## sim-bridge

Build with `cargo build --features irys` and set:

```toml
[sensors]
depth = "irys"   # default "geometric"
```

One thread owns the GPU and a `SimCamera` with one environment per rover.

- At each 10 Hz scan, a rover's pose (x, y, θ) stands its K1 on the sidewalk. Its left-eye depth is unprojected and subsampled to ≤ 2880 points in `LidarSim`'s frame, then sent as Livox packets exactly as before.
- The geometric raycast stays the default and the fallback. A build without the feature refuses `depth = "irys"` by name.
- The ZMQ `sim-world` hop the plan described was never built, so there is no ZMQ code to keep. The "old path" kept is the geometric LiDAR.

## Numbers

Apple M4 Max, Metal, release build, 544×448, K1 = 298k triangles plus a 129-box street per environment. From `cargo run --release --example bench`:

| batch | depth fps | depth ms / batch | RGB 1 spp fps | RGB 16 spp fps | scene re-upload ms | full step ms (move all + depth all) |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 130 | 7.7 | 54 | 5.4 | 81 | 91 |
| 8 | 126 | 64 | 59 | 5.1 | 79 | 147 |
| 64 | 125 | 512 | 59 | 5.4 | 94 | 621 |

- **Depth is overhead-bound.** The ray dispatch is 1.3–2.7 ms of each 8 ms camera. The rest is irys's per-call buffer clear, two blocking polls, and reading back a 4-float accumulator plus a spectrum buffer, then the CPU conversion. Throughput is flat in batch because `GpuLook::render` is one camera per call.
- **Moving the robots costs a full scene upload** (~80–95 ms) because `GpuLook` cannot update instances. That dominates a step at small batch.
- **Against the gsplat plan.** It quoted "~2–5 ms" for four 256×128 cubemap faces (131k px) and targeted 1000+ env steps/s. irys depth is 7.7 ms for one 544×448 view (244k px), so it is comparable per pixel today. It is not near 1000 steps/s at batch 64, because of the two irys gaps below.
- **What a fix would buy.** The dispatch time alone puts 64 cameras at ~150 ms, and a refit instead of a re-upload makes the step about the same.

The sim-bridge path at 10 Hz per rover is comfortably real-time for a few rovers. Check: one rover streamed 290 Livox packets/s, about 30 packets × 10 Hz.

Depth is verified metric by `tests/depth.rs`: sidewalk pixels back-project onto the sidewalk plane within 1 cm.

## irys API changes this wants (not made; irys is shared)

1. **Instance update.** Something like `GpuLook::set_instances(&[Instance])` that rebuilds or refits the TLAS only, keeping the BLAS and material buffers. This removes the ~85 ms per step.
2. **A depth/AOV mode.** Output first-hit `t` (or z), instance id and material id to their own buffers, with a real miss sentinel. Today depth decodes debug view 3, where a miss is shaded sky and a hit is told apart by `x == 0 && y ∈ {0,1}`; that works but is fragile.
3. **Batched cameras.** `render_batch(&[Camera], &Opts)` with one submission, one poll and one readback, and the spectrum readback made optional. This turns the 8 ms per camera into roughly the 1.3–2.7 ms dispatch.
4. **Principal-point offset** (`cx`, `cy` ≠ centre) on `Camera`. The K1's is 5.8 px / 2.8 px off centre.
5. **Splats in `look`** (a `.ply` loader plus splat any-hit), to bring back reconstructed worlds.
6. **Subsurface and volumes** for snow, beyond Lambertian-of-subsurface-colour.

## Dependency

```toml
irys-diff = { git = "https://github.com/ecto/irys", rev = "942c17e", features = ["gpu", "assets"] }
[patch.crates-io]   # irys pins tang by git; every tang in the graph must match
tang      = { git = "https://github.com/ecto/tang", rev = "bfc8003" }
tang-expr = { git = "https://github.com/ecto/tang", rev = "bfc8003" }
```

irys is private, and cargo fetches it over https with the machine's git credentials. To build against a local irys checkout, put an **untracked** `.cargo/config.toml` in the crate directory:

```toml
[patch."https://github.com/ecto/irys"]
irys-diff = { path = "/path/to/irys/crates/irys-diff" }
```

Never commit it, and never commit a `Cargo.lock` with local paths.

The crate requires a GPU with ray queries (wgpu `EXPERIMENTAL_RAY_QUERY`: Apple silicon, recent NVIDIA/AMD on Vulkan). Without one, `SimCamera::new` returns an error, and sim-bridge falls back only by configuration (`depth = "geometric"`).

## Running

```
cd depot/muni-irys
cargo test --release                                         # skips GPU/asset tests if missing
cargo run --release --example bench -- 1,8,64 10
cargo run --release --example demo -- out/demo 24 64         # synthetic gait
cargo run --release --example demo -- out/trace 50 4 ~/Developer/ipse/traces/m8-head.trace
```

## Risks

- **No ray-query GPU on the depot box** means no irys mode. wgpu's ray query is still `EXPERIMENTAL`.
- **The rendered street is not the physics world.** Obstacles are shared, but the curb and snow are visual only in sim-bridge.
- **Sim-to-real.** Rendered depth is perfect geometry. Real StereoNet depth is noisy, holey and range-limited, so a noise model belongs in front of RL.
- **The irys rev is pinned to a private repo.** CI needs credentials, and the gaps above are on irys's side.
