# BVR Architecture

**Base Vectoring Rover** — Muni's foundational mobile platform.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Operator Station                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Desktop App                                                         │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌────────────────────────────┐  │   │
│  │  │ Video View  │  │ Xbox Ctrl   │  │ Telemetry                  │  │   │
│  │  │ (H.265)     │  │ (gamepad)   │  │ (voltage, temps, mode)     │  │   │
│  │  └─────────────┘  └─────────────┘  └────────────────────────────┘  │   │
│  └───────────────────────────────┬─────────────────────────────────────┘   │
│                                  │ QUIC / UDP                               │
└──────────────────────────────────┼─────────────────────────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │       Cloud Relay           │
                    │  (optional, for NAT)        │
                    └──────────────┬──────────────┘
                                   │ LTE
┌──────────────────────────────────┼─────────────────────────────────────────┐
│                                  │                                   Rover │
│  ┌───────────────────────────────┴─────────────────────────────────────┐   │
│  │  bvrd (Jetson Orin NX)                                              │   │
│  │                                                                      │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────┐  │   │
│  │  │ teleop     │  │ control    │  │ state      │  │ tools        │  │   │
│  │  │            │  │            │  │            │  │              │  │   │
│  │  │ • Commands │  │ • Mixer    │  │ • Modes    │  │ • Discovery  │  │   │
│  │  │ • Telemetry│  │ • Limiter  │  │ • E-Stop   │  │ • Snow auger │  │   │
│  │  │ • Video    │  │ • Watchdog │  │            │  │ • (future)   │  │   │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘  └──────┬───────┘  │   │
│  │         │               │               │               │          │   │
│  │         └───────────────┴───────────────┴───────────────┘          │   │
│  │                                │                                    │   │
│  │                           ┌────┴────┐                               │   │
│  │                           │   can   │                               │   │
│  │                           │  (VESC) │                               │   │
│  │                           └────┬────┘                               │   │
│  └────────────────────────────────┼────────────────────────────────────┘   │
│                                   │ CAN bus                                │
│       ┌───────────┬───────────────┼───────────────┬───────────┐           │
│       │           │               │               │           │           │
│  ┌────┴────┐ ┌────┴────┐    ┌────┴────┐    ┌────┴────┐ ┌────┴─────┐     │
│  │ VESC FL │ │ VESC FR │    │ VESC RL │    │ VESC RR │ │ Tool MCU │     │
│  │ ID=1    │ │ ID=2    │    │ ID=3    │    │ ID=4    │ │ (RP2040) │     │
│  └────┬────┘ └────┬────┘    └────┬────┘    └────┬────┘ └────┬─────┘     │
│       │           │               │               │           │           │
│  ┌────┴────┐ ┌────┴────┐    ┌────┴────┐    ┌────┴────┐ ┌────┴─────┐     │
│  │ Hub     │ │ Hub     │    │ Hub     │    │ Hub     │ │ Snow     │     │
│  │ Motor   │ │ Motor   │    │ Motor   │    │ Motor   │ │ Auger    │     │
│  └─────────┘ └─────────┘    └─────────┘    └─────────┘ └──────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Hardware

| Component    | Specification                  |
| ------------ | ------------------------------ |
| Compute      | Jetson Orin NX                 |
| Chassis      | 24×24" 2020 aluminum extrusion |
| Motors       | 4× hoverboard hub motors       |
| ESCs         | 4× VESC (CAN bus)              |
| Power        | 48V main, 12V accessory rail   |
| Connectivity | LTE modem                      |

## Software Crates

| Crate     | Purpose                                                     |
| --------- | ----------------------------------------------------------- |
| `types`   | Shared types (Twist, Mode, etc.)                            |
| `can`     | CAN bus + VESC protocol                                     |
| `control` | Differential drive mixer, rate limiter, watchdog            |
| `state`   | State machine (Disabled → Idle → Teleop/Autonomous → EStop) |
| `hal`     | GPIO, ADC, power monitoring                                 |
| `teleop`  | LTE communications, command/telemetry                       |
| `tools`   | Tool discovery and implementations                          |

## Binaries

| Binary | Purpose                          |
| ------ | -------------------------------- |
| `bvrd` | Main daemon — runs on the Jetson |
| `cli`  | Debug/control CLI tool           |

## Threading Model

```
bvrd process
│
├── Main thread (control loop, 100Hz)
│   ├── Read CAN frames
│   ├── Process commands from channel
│   ├── Update state machine
│   ├── Compute motor outputs
│   ├── Send CAN commands
│   └── Update telemetry
│
└── Tokio runtime (async)
    └── Teleop server task
        ├── Receive UDP commands
        └── Send UDP telemetry
```

## State Machine

```
                    ┌──────────┐
                    │ Disabled │◄─────────────────────┐
                    └────┬─────┘                      │
                         │ Enable                     │ Disable
                         ▼                            │
                    ┌──────────┐                      │
         ┌──────────│   Idle   │──────────┐          │
         │          └────┬─────┘          │          │
         │ Teleop Cmd    │ Autonomous     │          │
         ▼               ▼                │          │
    ┌──────────┐   ┌────────────┐        │          │
    │  Teleop  │◄─►│ Autonomous │        │          │
    └────┬─────┘   └─────┬──────┘        │          │
         │               │                │          │
         │ E-Stop        │ E-Stop         │          │
         └───────┬───────┘                │          │
                 ▼                        │          │
            ┌──────────┐                  │          │
            │  E-Stop  │──────────────────┘          │
            └────┬─────┘                             │
                 │ Release                           │
                 └───► Idle                          │
                                                     │
            ┌──────────┐                             │
            │  Fault   │─────────────────────────────┘
            └──────────┘
              Clear
```

## Data Flow

1. **Operator → Rover**

   - Xbox controller input → Operator app
   - Twist command serialized → UDP packet
   - LTE → Cloud relay → Rover
   - `teleop` receives → sends to `bvrd` via channel

2. **Rover → Operator**

   - `bvrd` reads VESC status via CAN
   - Builds telemetry struct
   - `teleop` serializes → UDP packet
   - Rover → Cloud relay → LTE → Operator app

3. **Tool Control**
   - Xbox RT/LT + A button → Tool command
   - `tools` registry routes to active tool
   - Tool outputs CAN command
   - Tool MCU executes

## Configuration

Runtime configuration is in `config/bvr.toml`:

```toml
[chassis]
wheel_diameter_m = 0.165
track_width_m = 0.55
wheelbase_m = 0.55

[can]
interface = "can0"
vesc_ids = [1, 2, 3, 4]
pole_pairs = 15

[control]
loop_rate_hz = 100
command_timeout_ms = 250

[teleop]
port = 4840
```

## Safety

1. **Watchdog**: No command for 250ms → safe stop
2. **E-Stop**: Immediate stop, requires explicit release
3. **Rate limiting**: Acceleration capped to prevent tip-over
4. **Voltage monitoring**: Low battery → reduced power → shutdown

