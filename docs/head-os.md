# Cortex head OS plan

## Scope and service inventory

The cortex is a persistent self that can move between bodies: personality data (memory, voice, preferences, body schemas and developmental experience) belongs to the head, while each body contributes a separately versioned schema. The head is externally USB-only: one USB-C port plus pogo dock. The J401’s J16 exposes 3.3 V CAN logic; an add-on transceiver is required before connecting the lymb CAN pair.

This repository is now the Muni hardware/software concept tree; the old BVR runtime moved to the archive tag and depot to `ecto/depo` (`ARCHIVE.md`). Relevant service responsibilities from the archived architecture and Ipse README are:

| Service | Responsibility / source |
|---|---|
| `ipse-runtime` (new integration service) | Mind/world-model, interoception, temperament, action arbitration; see Ipse `ipse-world`, `ipse-temperament`, `ipse-split`. Keep expressed reluctance and experience records distinct from trainable weights. |
| `ipse-body` (new adapter) | Body discovery, joint table, schema selection and calibration; Ipse `ipse-deploy` and `ipse-schema`. Version schema per body, never overwrite head personality on a swap. |
| `ipse-sim` / `phyz` adapter | Local differentiable simulation and body-schema fitting; Ipse `ipse-sim`, Muni design stack. Not on the motor safety path. |
| `lymbd` (new) | CAN discovery, actuator state and bounded command interface for the arm. The archived `bvrd` architecture provides patterns for CAN, watchdogs, state and control, but is not directly the cortex runtime. |
| `voice` (new) | Audio I/O, speech recognition/synthesis and selected voice assets/preferences. |
| `frog-node` (new, dock-only) | Advertises an inference worker to frog cloud/zest routing while docked and idle; unloads or yields on undock or body activity. |
| `head-agent` (new) | Mode/state coordination, device and body presence, health/readiness. |
| system services | Network, time sync, encrypted-store mount/unlock, log/metrics forwarding and update agent. |

These are service boundaries to implement, not a claim that all are already shipped. The archived BVR stack additionally documents `recording` (Rerun), `metrics` (UDP to InfluxDB), `gps`, `camera`, teleop and tools; only recording/metrics concepts transfer directly. Head camera/GPS/rover teleop are not assumed because the product I/O is USB-only. Ipse explicitly requires experience to remain separately versioned from weights.

## OS choices

| Option | CUDA / TensorRT | Reproducibility | OTA / rollback | Boot, security and team cost |
|---|---|---|---|---|
| **JetPack 7.2.1, Ubuntu 24.04 stock** | Best-supported NVIDIA driver, CUDA and TensorRT path; least friction for Orin inference | Image manifest and provisioning scripts help, but mutable apt/system state drifts | No robust system rollback by itself; add RAUC or Mender only after validating partition layout and NVIDIA boot chain | Fastest bring-up and easiest vendor debugging. Secure Boot is platform-supported but provisioning/keys are separate. Encrypt personality data independently. |
| **JetPack headless + Mender/RAUC A/B** | Same NVIDIA stack if built from the supported JetPack BSP | Image-based releases are substantially more repeatable | Strong atomic A/B update and rollback when boot-health confirmation is integrated | More storage and bootloader/update integration work; secure boot and encrypted data need explicit key provisioning. Good shipping direction. |
| **NVIDIA Jetson Yocto** | NVIDIA BSP can supply the supported GPU stack, but integration/testing burden is higher | Strongest image and package reproducibility | A/B update frameworks fit naturally into an immutable image | Secure boot and measured image composition can be tightly controlled; more custom recipes, slower iteration and high maintenance cost for a small team. |
| **NixOS (`jetpack-nixos`)** | CUDA/TensorRT availability depends on the project’s support matrix and matching NVIDIA binaries; verify exact Orin/J401 release before committing | Excellent declarative system reproducibility | Generations allow system rollback; does not automatically provide a tested power-fail-safe A/B transaction | Attractive developer ergonomics, but carrier/BSP and vendor support gaps increase debugging risk. Secure boot/data encryption still require separate design. |

Boot time is dominated by firmware, services and model loading, not the package manager. Keep noncritical voice, simulation and inference-node services out of the critical boot target; measure cold boot to CAN-safe readiness and first response on hardware. No option encrypts user/personality data automatically. Use a dedicated LUKS2 data volume with a device-bound secret or user unlock, recovery key escrow outside the head, and a documented reset/transfer policy. Secure Boot protects boot integrity; it is not a substitute for data-at-rest encryption.

## Recommendation

**MVP now:** keep the already booted Ubuntu installation on the J401, pin the exact JetPack/L4T and CUDA/TensorRT versions, avoid major OS changes, and run native systemd services. Use containers only for non-hardware-facing workloads (for example, simulation or the docked inference worker) where NVIDIA Container Toolkit is validated. Keep CAN, personality mount, state coordination and actuator control on the host. This minimizes time-to-first-body and keeps NVIDIA’s supported stack intact.

**Shipping:** move to a headless, NVIDIA-supported JetPack rootfs with signed, versioned A/B updates through Mender or RAUC after proving boot success/rollback on the actual J401 module and storage. Keep `/data/personality` on a separate encrypted partition excluded from all system images and update manifests. Yocto is a later option if product scale justifies owning the BSP; do not make NixOS the first customer image until its exact Jetson GPU stack and recovery path are validated.

## Runtime and data layout

```text
systemd (host)
  head-agent ── ipse-runtime ── ipse-body ── lymbd ── SocketCAN can0
       ├────── voice
       ├────── recorder / metrics
       └────── frog-node [docked + idle only; container optional]

/boot + rootfs: replaceable OS slot A/B
/var: bounded logs, caches, crash records (non-personality)
/data/personality: LUKS2, persistent head identity and experience
  identity/  memory/  preferences/  voice/  experience/  weights/
/data/bodies/<body-id>/<schema-version>: replaceable body schema/calibration
```

Systemd owns ordering, restart policy, resource limits and health dependencies. Make `head-agent` and CAN control native host services. A container must not own CAN timing or hold the personality volume open across updates. The frog worker is a separate constrained service/container, network-disabled except for its authenticated inference channel, and enabled only when a physical dock signal is present, the head is idle, thermals/power are healthy, and no body-control session has priority. Undock or activity revokes the worker lease and drains tasks.

Mount the encrypted personality volume before Ipse/voice services start; fail closed if it cannot be unlocked. Updates replace only the inactive system slot. Personality schema migrations are explicit, journaled and backward-readable for at least one system rollback window; retain a pre-migration snapshot. A body swap adds/selects a body-scoped schema and calibration record without changing head identity, memory or preferences. Export/restore requires authenticated user action and encryption; never silently bind personality to an arm serial number.

### J16 CAN bring-up

J401 J16 is 3.3 V controller logic, **not** differential CAN. Fit a compatible 3.3 V transceiver and connect its TX/RX/standby signals per the carrier pinout, then CAN-H/CAN-L to the lymb bus with correct end termination. On Jetson Linux, verify the installed BSP exposes the controller through `mttcan`; load/confirm the module and pinmux before relying on any interface name. First use classic CAN at 1 Mbit/s only if every node and bus timing supports it. CAN-FD requires confirmed controller, transceiver and actuator support, plus separately configured arbitration/data rates; do not assume `can0` means FD is enabled.

Bring-up sequence (bench, arm unpowered or mechanically restrained):

```sh
lsmod | grep mttcan
ip -details link show can0
sudo ip link set can0 down
sudo ip link set can0 type can bitrate 1000000 dbitrate 4000000 fd on   # FD only after validated
sudo ip link set can0 up
ip -details -statistics link show can0
 candump can0
```

For a classic-only first test, omit `dbitrate` and `fd on`. Confirm pinmux, bitrate, termination and error counters with a passive capture before sending frames. Keep motor enable/torque commands inhibited until node identity, limits, watchdog and e-stop behavior are verified. The archived BVR docs describe SocketCAN and CAN diagnostics, but lymb protocol/IDs remain a cortex integration task.

### Logging and telemetry

Use journald with persistent, size-bounded storage and structured tracing fields (`boot_id`, `service`, `body_id`, schema version, mode). Record safety/mode transitions, CAN errors and health, update slot/rollback events, unlock/migration events, inference lease state, and resource/thermal metrics. Do not write raw personality or private audio into ordinary logs. Keep an opt-in, separately encrypted experience recorder (Ipse contract) and a bounded diagnostic ring buffer. Export aggregate health and performance when networked; queue safely while offline. Avoid importing the old depot’s InfluxDB/SFTP coupling into the standalone head.

## Ordered J401 bring-up

1. Record module SKU/RAM/storage, J401 revision, current Ubuntu/JetPack/L4T versions, boot source and recovery method; save a known-good image before changing boot or partitioning.
2. Update only through the supported NVIDIA/Seeed path if needed. Confirm reboot, USB-C console/access, networking, thermal and power behavior; pin packages and capture the image manifest.
3. Inventory J16 pinout, pinmux and 3.3 V levels against the exact carrier revision. Wire transceiver and terminated CAN test fixture; scope TX/RX and differential bus before connecting the lymb.
4. Verify `mttcan`, pinmux and `can0`; configure classic 1 Mbit/s, capture passively, validate error counters and physical termination. Test FD separately only with confirmed FD-capable components.
5. Add a native `can-setup.service` and `lymbd.service` with explicit dependencies, restart limits and safe default (listen-only/no torque). Confirm watchdog and stop behavior before enabling bounded commands.
6. Implement `head-agent` and body discovery; load a versioned lymb schema and confirm disconnect/swap does not alter head personality state.
7. Create and test the encrypted personality volume, unlock/recovery flow, offline behavior, and a simulated update/rollback preserving `/data`.
8. Start Ipse runtime, voice and recorder; verify persistent experience is versioned separately from model weights, then test restore and migration recovery.
9. Add dock sensing and `frog-node`; test idle-only lease, resource/thermal limits, undock cancellation, and priority handoff to body control.
10. Exercise repeated cold boots, power interruption during update and data writes, CAN disconnects, full logs, low storage and service crash recovery. Freeze the shipping image only after rollback and encrypted-data recovery work on the physical J401.
