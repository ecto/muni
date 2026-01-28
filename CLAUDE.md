# CLAUDE.md

This document provides an overview of the Muni codebase for AI assistants.

## Project Overview

Muni is an open-source municipal robotics project building autonomous utility vehicles for public works, starting with sidewalk snow removal. The system consists of:

- **Rovers**: Autonomous vehicles (currently the BVR "Base Vectoring Rover" morphology)
- **Depot**: Base station infrastructure for fleet operations, metrics, and teleop
- **MCU Firmware**: Embedded controllers for LEDs and tool attachments

## Repository Structure

```
muni/
├── bvr/                    # Base Vectoring Rover (first morphology)
│   ├── firmware/           # Onboard Rust software (Jetson Orin NX)
│   │   ├── bins/           # Executables (bvrd daemon, muni CLI, train)
│   │   ├── crates/         # Library crates (control, can, teleop, etc.)
│   │   └── config/         # Runtime configuration and systemd services
│   ├── training/           # Behavioral cloning pipeline (Python)
│   │   ├── extract_demos.py    # .rrd → demos.npz
│   │   ├── train_bc.py         # demos.npz → policy.json (MLP)
│   │   ├── evaluate.py         # policy + data → metrics report
│   │   └── requirements.txt    # rerun-sdk, numpy, torch
│   ├── cad/                # Mechanical design files
│   ├── electrical/         # Schematics and PCBs
│   └── docs/               # BVR-specific documentation
├── depot/                  # Base station services
│   ├── console/            # React web app (fleet ops, teleop UI, dispatch)
│   ├── discovery/          # Rover registration service (Rust)
│   ├── dispatch/           # Mission planning & task dispatch (Rust)
│   ├── gps-status/         # GPS/RTK status service (Rust)
│   ├── map-api/            # Map serving API (Rust)
│   ├── mapper/             # Map processing orchestrator (Rust)
│   ├── splat-worker/       # GPU 3D reconstruction (Python)
│   ├── grafana/            # Dashboard provisioning
│   └── scripts/            # Maintenance scripts
├── mcu/                    # Embedded firmware (RP2350, ESP32-S3)
│   ├── bins/               # Target-specific binaries
│   └── crates/             # Shared embedded crates
├── paper/                  # Technical documents (Typst)
└── web/                    # Static website (GitHub Pages)
```

## Technology Stack

### Rust (bvr/firmware, depot services, mcu)

- **Async runtime**: Tokio for firmware; Embassy for MCU
- **Serialization**: serde + toml/JSON
- **Logging**: tracing with tracing-subscriber
- **Recording**: Rerun (.rrd files) for telemetry
- **Error handling**: thiserror for library errors, anyhow for applications
- **CLI**: clap with derive feature
- **Math**: nalgebra for linear algebra

### TypeScript/React (depot/console)

- **Framework**: React 19 with Vite
- **State**: Zustand for global state
- **Styling**: Tailwind CSS v4
- **3D**: React Three Fiber + drei
- **UI Components**: Radix UI primitives
- **Routing**: React Router v7
- **Linting**: ESLint with typescript-eslint

### Python (depot/splat-worker, bvr/training)

- GPU-accelerated Gaussian splatting for 3D reconstruction (depot/splat-worker)
- **Behavioral cloning pipeline** (bvr/training): PyTorch, NumPy, Rerun SDK
  - Runs on depot or dev machine, NOT on rover (training only)
  - Output: JSON policy files loaded by the Rust `policy` crate on the rover

### Typst (paper/)

- Technical documents, datasheets, and manuals

## Key Conventions

### Rust Code Style

- Use `//!` module-level docs at the top of lib.rs files
- Use `///` for function/struct documentation
- Prefer `thiserror` for custom error types in libraries
- Use workspace dependencies in Cargo.toml (version.workspace = true)
- Tests go in a `#[cfg(test)] mod tests` block at the bottom of files
- Edition 2021 for bvr/firmware, Edition 2024 for mcu

### TypeScript Code Style

- Use `const` enums with `as const` for type-safe constants
- Prefer interfaces over types for object shapes
- Export types alongside runtime values
- Use React hooks for stateful logic (custom hooks in `src/hooks/`)
- Components in `src/components/`, views/pages in `src/views/`

### File Naming

- Rust: snake_case for files and modules
- TypeScript: PascalCase for components, camelCase for hooks/utilities
- Use `.tsx` for React components, `.ts` for pure TypeScript

### Git Conventions

- LFS is configured for large binary files (CAD, images, PDFs)
- Web assets in `web/` are excluded from LFS for GitHub Pages compatibility
- Verify LFS status with: `git check-attr filter <file>`

## Development Workflows

### BVR Firmware (bvr/firmware)

```bash
# Build for development (native, macOS/Linux)
cargo build

# Run tests
cargo test

# Run locally with mock CAN
cargo run --bin bvrd
```

Cross-compilation uses `cross` (install: `cargo install cross --git https://github.com/cross-rs/cross`).

### Deployment (muni CLI)

The `muni` CLI is the primary tool for deployment. Install globally with:
```bash
cargo install --path bvr/firmware/bins/cli
```

**Deploy to rovers:**
```bash
muni deploy rover <hostname>       # Cross-compile bvrd, upload, restart
muni deploy rover frog-0 --cli     # Also deploy muni CLI to rover
muni deploy rover frog-0 --all     # Full deploy: bvrd + CLI + config
muni deploy rover frog-0 --no-restart  # Deploy without restarting service
```

**Deploy to depot:**
```bash
muni deploy depot                  # Rsync + docker compose build/up
muni deploy depot --sync-only      # Only sync files, no rebuild
muni deploy depot --service console  # Rebuild specific service
```

**Other muni commands:**
```bash
muni rover scan                    # Scan CAN bus for VESCs
muni rover drive --linear 0.5      # Send velocity command
muni rover estop                   # Send e-stop
muni gps monitor                   # GPS status TUI
muni gps configure-base            # Configure GPS as RTK base station
muni gps configure-rover           # Configure GPS as RTK rover
```

### Depot Console (depot/console)

```bash
cd depot/console

# Install dependencies
npm install

# Development with hot-reload
npm run dev        # Runs on http://localhost:5173

# Type check and build
npm run build

# Lint
npm run lint
```

### Depot Services (Docker)

```bash
cd depot

# Start all services
docker compose up -d

# Development mode (console hot-reload)
docker compose up -d discovery influxdb grafana
cd console && npm run dev

# With GPU splatting support
docker compose --profile gpu up -d

# With RTK base station
docker compose --profile rtk up -d
```

### MCU Firmware (mcu/)

**RP2350 (Pico 2 W):**
```bash
cd mcu
rustup target add thumbv8m.main-none-eabihf
cargo build --release -p mcu-rp2350
picotool load target/thumbv8m.main-none-eabihf/release/mcu-rp2350 -t elf -f
```

**ESP32-S3 (Heltec):**
```bash
cd mcu/bins/esp32s3
cargo install espup && espup install
source ~/export-esp.sh
cargo build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
```

### Behavioral Cloning Training (bvr/training)

The training pipeline converts teleop recordings into learned navigation
policies. It runs offline on a dev machine or depot — not on the rover.

```bash
cd bvr/training

# 1. Set up Python environment
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# 2. Collect .rrd session files from rover
#    Sessions are recorded by bvrd at /var/log/bvr/sessions/ on the rover.
scp frog-0:/var/log/bvr/sessions/2026-01-26T*/session.rrd ./sessions/

# 3. Extract (observation, action) pairs from teleop recordings
python extract_demos.py --sessions-dir ./sessions --output demos.npz

# 4. Train MLP policy (7→64→64→2, ~5k params)
python train_bc.py --data demos.npz --output bc-teleop-v0.1.0.json

# 5. Evaluate and print metrics
python evaluate.py --policy bc-teleop-v0.1.0.json --data demos.npz

# 6. Deploy to rover
scp bc-teleop-v0.1.0.json frog-0:/var/lib/bvr/policies/default.json
```

**Pipeline architecture:**

```
teleop .rrd ──→ extract_demos.py ──→ demos.npz ──→ train_bc.py ──→ policy.json
                                                                        │
                 rover (bvrd) ◂── scp ──────────────────────────────────┘
                 Policy::load() auto-detects architecture (linear or mlp)
```

**Key details:**

- **Observation**: 7-dim vector `[x, y, θ, v_lin, v_ang, goal_dx, goal_dy]`,
  normalized to [-1, 1]. Goal is in robot frame. During training, goals are
  pseudo-goals (robot's position 2s in the future). At deploy, classical
  planner provides real goals.
- **Action**: 2-dim `[linear_vel, angular_vel]`, tanh-bounded to [-1, 1].
- **Normalization constants** are shared between Python (extract_demos.py)
  and Rust (policy crate `NormalizationConfig`). If one changes, update both.
- **Rerun SDK version** must match the firmware's rerun version (currently 0.22).
  Check `bvr/firmware/Cargo.toml` workspace deps.
- **Policy JSON** is directly loadable by the Rust `policy` crate — no
  conversion step needed. Architecture is auto-detected from the JSON.

### Static Website (web/)

```bash
# Local development
python3 -m http.server 8000
# Visit http://localhost:8000

# Deployment: Push to main, GitHub Pages auto-deploys from web/
```

### Technical Documents (paper/)

```bash
cd paper
make                    # Build all documents
make bvr0-manual.pdf    # Build specific document
```

Requires Typst installed.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Depot (Base Station)                                                        │
│   Console (:80)     Grafana (:3000)     InfluxDB        SFTP (:2222)       │
│   Fleet ops         Dashboards          Metrics DB      Session storage    │
│   Teleop UI         Alerts              Time series     Recording sync     │
│   Dispatch (:4890)  PostgreSQL                                              │
│   Mission planning  Zone/task storage                                       │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │
                        UDP metrics │ WebSocket teleop
                        SFTP sync   │ RTK corrections
                        WS dispatch │ (task assignments)
                                    │
┌───────────────────────────────────┴─────────────────────────────────────────┐
│ BVR Rover                                                                   │
│   Jetson Orin NX running bvrd daemon                                        │
│   ├── teleop     WebSocket comms, video streaming                          │
│   ├── dispatch   Mission tasks from depot, progress reporting              │
│   ├── control    Differential drive mixer, rate limiting                   │
│   ├── state      Mode management (Idle → Teleop → Autonomous → EStop)      │
│   ├── gps        RTK positioning                                           │
│   └── recording  Session capture to .rrd files                             │
│                                                                             │
│   CAN bus → 4x VESC motor controllers + MCU for LEDs                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Files Reference

| Purpose | Path |
|---------|------|
| BVR firmware workspace | `bvr/firmware/Cargo.toml` |
| Main daemon entry | `bvr/firmware/bins/bvrd/src/main.rs` |
| Muni CLI (deploy, rover, gps) | `bvr/firmware/bins/cli/src/main.rs` |
| Motor control logic | `bvr/firmware/crates/control/src/lib.rs` |
| State machine | `bvr/firmware/crates/state/src/lib.rs` |
| Dispatch client | `bvr/firmware/crates/dispatch/src/lib.rs` |
| Shared types | `bvr/firmware/crates/types/src/lib.rs` |
| Runtime config | `bvr/firmware/config/bvr.toml` |
| Console app entry | `depot/console/src/main.tsx` |
| Console state | `depot/console/src/store.ts` |
| Console types | `depot/console/src/lib/types.ts` |
| Dispatch service | `depot/dispatch/src/main.rs` |
| Dispatch UI | `depot/console/src/views/DispatchView.tsx` |
| Docker services | `depot/docker-compose.yml` |
| Policy inference (Rust) | `bvr/firmware/crates/policy/src/lib.rs` |
| Demo extraction (Python) | `bvr/training/extract_demos.py` |
| BC training (Python) | `bvr/training/train_bc.py` |
| Policy evaluation (Python) | `bvr/training/evaluate.py` |
| Deployed policy (rover) | `/var/lib/bvr/policies/default.json` |
| Session recordings (rover) | `/var/log/bvr/sessions/` |
| MCU LED controller | `mcu/bins/rp2350/src/main.rs` |
| GitHub Pages CI | `.github/workflows/pages.yml` |

## Testing

### Rust
```bash
# All tests in a workspace
cargo test

# Specific crate
cargo test -p control
```

### TypeScript
```bash
cd depot/console
npm run lint    # ESLint
npm run build   # Type checking via tsc
```

### Policy / Training
```bash
# Rust policy crate (MLP inference, JSON round-trip, backward compat)
cargo test -p policy

# Python pipeline (requires .rrd sessions and venv with requirements.txt)
python extract_demos.py --sessions-dir ./sessions --output demos.npz
python train_bc.py --data demos.npz --output test.json --epochs 20
python evaluate.py --policy test.json --data demos.npz
```

## Common Tasks

### Adding a new firmware crate
1. Create directory in `bvr/firmware/crates/<name>/`
2. Add `Cargo.toml` with `version.workspace = true`, `edition.workspace = true`
3. Add to `[workspace.dependencies]` in `bvr/firmware/Cargo.toml` if reused
4. Add to bin dependencies as needed

### Adding a new depot service
1. Create Rust project in `depot/<name>/`
2. Add Dockerfile
3. Add service to `depot/docker-compose.yml`
4. Update `depot/README.md`

### Adding web assets
1. Copy files directly to `web/` (don't symlink)
2. Verify not tracked by LFS: `git check-attr filter web/<file>`
3. If tracked by LFS, update `.gitattributes` with exclusion rule

## Environment Variables

### Depot (.env)
- `CONSOLE_PASSWORD` - Console authentication
- `INFLUXDB_ADMIN_TOKEN` - InfluxDB access token
- `GRAFANA_ADMIN_PASSWORD` - Grafana admin password
- `SESSIONS_PATH` - Session storage location
- `RETENTION_DAYS` - Auto-cleanup threshold (default: 30)

### Rover (bvr.toml)
Configuration is file-based, not environment variables. See `bvr/firmware/config/bvr.toml`.

## Important Notes

- **Safety**: The rover has multiple safety systems (watchdog, e-stop, rate limiting). Never bypass these.
- **CAN IDs**: VESCs use IDs 1-4 (FL, FR, RL, RR). MCU peripherals use 0x0B00+ range.
- **Cross-compilation**: Requires Docker for `cross` to work with GStreamer ARM64 libs.
- **LFS**: Large files are tracked with Git LFS. Run `git lfs pull` after cloning.
- **Tailscale**: Rovers connect to depot via Tailscale for secure networking.
- **Policy normalization**: The Python training scripts and Rust `policy` crate share normalization constants (`max_position=50`, `max_linear_vel=2`, `max_angular_vel=2`). These must stay in sync — see `NormalizationConfig` in `policy/src/lib.rs` and the constants at the top of `extract_demos.py`.
- **Rerun version**: The training pipeline's `rerun-sdk` version must match the firmware workspace's `rerun` crate version (currently 0.22). Mismatched versions will fail to decode `.rrd` files.
