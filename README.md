# Muni

Municipal robotics platform.

## Repository Structure

```
muni/
├── depot/          # Base station — fleet monitoring, teleop, metrics
├── bvr/            # Base Vectoring Rover (first morphology)
│   ├── firmware/   # Rust — onboard software (Jetson Orin NX)
│   ├── cad/        # Mechanical design (STEP, native files)
│   ├── electrical/ # Schematics, PCB designs, BOM
│   ├── manufacturing/  # Assembly procedures, test fixtures
│   └── docs/       # BVR-specific documentation
└── docs/           # Platform-level documentation
```

## Morphologies

| Name | Description | Status |
|------|-------------|--------|
| [BVR](bvr/) | 4-wheel skid-steer rover, 24×24" footprint | Active |

## Getting Started

### Base Station (Depot)

The depot provides fleet monitoring and teleop for all morphologies:

```bash
cd depot
docker compose up -d
```

See [depot/README.md](depot/README.md) for details.

### BVR Firmware

```bash
cd bvr/firmware
cargo build --release
```

See [bvr/firmware/README.md](bvr/firmware/README.md) for details.

## License

Proprietary — Muni Municipal Robotics
