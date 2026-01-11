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
├── firmware/               # (Legacy) Older firmware workspace
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

### Python (depot/splat-worker)

- GPU-accelerated Gaussian splatting for 3D reconstruction

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

# Cross-compile for Jetson (aarch64)
cargo build --release --target aarch64-unknown-linux-gnu

# Deploy to rover
./deploy.sh <rover-hostname>           # bvrd only
./deploy.sh <rover-hostname> --cli     # bvrd + CLI
./deploy.sh <rover-hostname> --restart # Deploy and restart service

# Run locally with mock CAN
cargo run --bin bvrd
```

Cross-compilation uses `cross` (install: `cargo install cross --git https://github.com/cross-rs/cross`).

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
cargo build --release -p rover-leds
picotool load target/thumbv8m.main-none-eabihf/release/rover-leds -t elf -f
```

**ESP32-S3 (Heltec):**
```bash
cd mcu/bins/esp32s3
cargo install espup && espup install
source ~/export-esp.sh
cargo build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/heltec-attachment
```

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
