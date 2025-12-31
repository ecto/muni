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

| Crate       | Purpose                            |
| ----------- | ---------------------------------- |
| `types`     | Shared types, message definitions  |
| `can`       | CAN bus abstraction, VESC protocol |
| `control`   | Motor mixing, velocity control     |
| `state`     | State machine, mode management     |
| `hal`       | GPIO, ADC, power monitoring        |
| `teleop`    | LTE comms, command/telemetry       |
| `tools`     | Tool discovery + implementations   |
| `recording` | Telemetry recording (Rerun .rrd)   |
| `metrics`   | Real-time metrics push to Depot    |
| `gps`       | GPS receiver integration           |
| `camera`    | Camera capture and streaming       |
| `rl`        | RL environment for training        |
| `sim`       | Physics simulation for training    |
| `policy`    | Policy loading and inference       |

## Binaries

| Binary  | Purpose                        |
| ------- | ------------------------------ |
| `bvrd`  | Main daemon                    |
| `muni`  | Unified CLI (rover + GPS)      |
| `train` | RL training for nav policies   |

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

Install the CLI:

```bash
cargo install --path bins/cli
```

### Rover Commands

```bash
# Scan CAN bus for VESCs
muni rover scan

# Scan with custom interface/duration
muni rover scan --interface can0 --duration 3

# Send velocity command
muni rover drive --linear 0.5 --angular 0.0

# Emergency stop
muni rover estop
```

### GPS Commands

```bash
# Monitor GPS status (auto-detects rover vs base mode)
muni gps monitor --port /dev/ttyACM0

# Configure ZED-F9P as base station (fixed position)
muni gps configure-base --port /dev/ttyACM0 \
    --fixed-position 41.481956,-81.8053,213.5

# Configure ZED-F9P as base station (survey-in)
muni gps configure-base --port /dev/ttyACM0 \
    --survey-duration 3600 --survey-accuracy 2.0

# Configure ZED-F9P as rover
muni gps configure-rover --port /dev/ttyACM0
```

See [RTK GPS documentation](../docs/hardware/rtk.md) for full details.

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

## Autonomous Mode & Policies

The rover supports autonomous navigation using RL-trained policies.

### Policy Format

Policies are versioned JSON files with the following structure:

```json
{
  "version": "1.0.0",
  "name": "nav",
  "description": "Navigation policy trained with REINFORCE",
  "observation_size": 7,
  "action_size": 2,
  "architecture": "linear",
  "weights": [[...], [...]],
  "biases": [0.0, 0.0],
  "metrics": {
    "success_rate": 0.85,
    "avg_reward": 95.2,
    "training_iterations": 1000,
    "training_episodes": 10000
  }
}
```

See `config/policies/nav-v0.1.0.json.example` for a complete example.

### Training Policies

```bash
# Run training with default settings
cargo run --bin train -- train --output ./policies

# Train with custom parameters
cargo run --bin train -- train \
  --iterations 2000 \
  --episodes-per-iter 20 \
  --lr 0.005 \
  --output ./policies \
  --name nav \
  --version 1.0.0

# Benchmark environment performance
cargo run --bin train -- bench --steps 100000

# Test heuristic policy
cargo run --bin train -- heuristic --episodes 100 --verbose
```

### Deploying Policies

1. Copy policy files to `/var/lib/bvr/policies/` on the rover
2. Set the default policy in `bvr.toml`:
   ```toml
   [autonomous]
   enabled = true
   policy_dir = "/var/lib/bvr/policies"
   policy_file = "/var/lib/bvr/policies/nav-v1.0.0.json"
   ```
3. Or specify via CLI: `--policy /path/to/policy.json`

### Running Autonomous Mode

```bash
# Start with autonomous mode enabled
./target/release/bvrd --policy /var/lib/bvr/policies/nav-v1.0.0.json --goal "5.0,0.0"

# In simulation mode
cargo run --bin bvrd -- --sim --policy ./policies/nav-v0.1.0.json --goal "5.0,0.0"
```

Switch to autonomous mode via teleop command: `SetMode(Autonomous)`

## Documentation

- [Architecture](../docs/architecture.md)
- [CAN Protocol](../docs/can-protocol.md)
- [Teleop System](../docs/teleop.md)
- [Tool System](../docs/tools.md)
- [Power System](../docs/power.md)
- [Logging & Telemetry](../docs/logging.md)
- [Base Station (Depot)](../depot/README.md)
