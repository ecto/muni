# BVR Firmware

Onboard software for the Base Vectoring Rover, targeting Jetson Orin NX.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  bvrd (main daemon)                                             │
│                                                                 │
│   ┌───────────┐  ┌────────────┐  ┌──────────┐  ┌────────────┐ │
│   │ teleop    │  │ control    │  │ state    │  │ tools      │ │
│   │           │  │            │  │          │  │            │ │
│   │ • UDP     │  │ • Mixer    │  │ • Modes  │  │ • Registry │ │
│   │ • Cmds    │  │ • Limiter  │  │ • E-Stop │  │ • Auger    │ │
│   └─────┬─────┘  └──────┬─────┘  └────┬─────┘  └──────┬─────┘ │
│         │               │              │               │       │
│         └───────────────┴──────────────┴───────────────┘       │
│                              │                                  │
│                         ┌────┴────┐                            │
│                         │   can   │                            │
│                         │  (VESC) │                            │
│                         └────┬────┘                            │
└──────────────────────────────┼─────────────────────────────────┘
                               │
                          CAN bus
```

## Crates

| Crate     | Purpose                            |
| --------- | ---------------------------------- |
| `types`   | Shared types, message definitions  |
| `can`     | CAN bus abstraction, VESC protocol |
| `control` | Motor mixing, velocity control     |
| `state`   | State machine, mode management     |
| `hal`     | GPIO, ADC, power monitoring        |
| `teleop`  | LTE comms, command/telemetry       |
| `tools`   | Tool discovery + implementations   |

## Binaries

| Binary | Purpose           |
| ------ | ----------------- |
| `bvrd` | Main daemon       |
| `bvr`  | Debug/control CLI |

## Building

```bash
# Native (for development on macOS/Linux)
cargo build

# Cross-compile for Jetson (aarch64)
cargo build --release --target aarch64-unknown-linux-gnu
```

## Deployment

Deploy to the rover over Tailscale:

```bash
# Deploy bvrd only
./deploy.sh frog-0

# Deploy bvrd + bvr CLI
./deploy.sh frog-0 --cli

# Deploy and restart service
./deploy.sh frog-0 --cli --restart

# Deploy with config file
./deploy.sh frog-0 --config --restart
```

### First-time Rover Setup

**1. SSH Key Setup (on your Mac):**

```bash
# Copy SSH key to rover
ssh-copy-id cam@frog-0

# Add key to agent (stores passphrase in Keychain)
eval "$(ssh-agent -s)"
ssh-add --apple-use-keychain ~/.ssh/id_ed25519
```

**2. Passwordless sudo (on the rover):**

```bash
sudo visudo
# Add at end: cam ALL=(ALL) NOPASSWD: ALL
```

**3. Install systemd service (on the rover):**

```bash
scp config/bvrd.service frog-0:/tmp/
ssh frog-0 'sudo mv /tmp/bvrd.service /etc/systemd/system/ && \
            sudo systemctl daemon-reload && \
            sudo systemctl enable bvrd'
```

**4. Set Tailscale hostname (optional):**

```bash
ssh frog-0 'sudo tailscale set --hostname=frog-0'
```

### Cross-Compilation

The deploy script uses `cross` for ARM64 cross-compilation with GStreamer support:

```bash
# Install cross (one-time)
cargo install cross --git https://github.com/cross-rs/cross

# Ensure Docker is running
open -a Docker

# Build manually (deploy.sh does this automatically)
cross build --release --target aarch64-unknown-linux-gnu --bin bvrd
```

The `Cross.toml` configures the build container with GStreamer ARM64 libraries.

## Configuration

Runtime config lives in `config/bvr.toml`. See the file for all options.

## Running

```bash
# On the Jetson
./target/release/bvrd --config /etc/bvr/bvr.toml --can-interface can0

# With custom VESC IDs (default is 0 1 2 3 for FL FR RL RR)
./target/release/bvrd --vesc-ids 0 1 2 3
```

## CLI Usage

```bash
# Scan CAN bus for VESCs
bvr scan

# Scan with custom interface/duration
bvr scan --interface can0 --duration 3

# Send velocity command
bvr drive --linear 0.5 --angular 0.0

# Emergency stop
bvr estop

# Monitor telemetry (TODO)
bvr monitor
```

## Development

### On macOS

CAN is mocked — you can develop and test the control logic without hardware:

```bash
cargo run --bin bvrd
# Logs mock CAN traffic
```

### On Linux with vcan

```bash
# Set up virtual CAN
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0

# Run with vcan
cargo run --bin bvrd -- --can-interface vcan0
```

### Testing

```bash
cargo test
```

## Documentation

- [Architecture](../docs/architecture.md)
- [CAN Protocol](../docs/can-protocol.md)
- [Teleop System](../docs/teleop.md)
- [Tool System](../docs/tools.md)
- [Power System](../docs/power.md)
