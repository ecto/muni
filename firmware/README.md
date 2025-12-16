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

| Binary | Purpose            |
| ------ | ------------------ |
| `bvrd` | Main daemon        |
| `cli`  | Debug/control tool |

## Building

```bash
# Native (for development on macOS/Linux)
cargo build

# Cross-compile for Jetson (aarch64)
cargo build --release --target aarch64-unknown-linux-gnu
```

## Configuration

Runtime config lives in `config/bvr.toml`. See the file for all options.

## Running

```bash
# On the Jetson
./target/release/bvrd --config /etc/bvr/bvr.toml --can-interface can0

# With custom VESC IDs
./target/release/bvrd --vesc-ids 1 2 3 4 --pole-pairs 15
```

## CLI Usage

```bash
# Send velocity command
cli drive --linear 0.5 --angular 0.0

# Emergency stop
cli estop

# Monitor telemetry (TODO)
cli monitor
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
