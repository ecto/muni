# bvr

**Base Vectoring Rover** — Muni's foundational mobile platform.

24×24" footprint, 4 hoverboard hub motors, 2020 aluminum chassis, 48V/12V power rails.

## Repository Structure

```
bvr/
├── firmware/       # Rust — onboard software (Jetson Orin NX)
├── depot/          # Base station — fleet monitoring infrastructure
├── cad/            # Mechanical design (STEP, native files)
├── electrical/     # Schematics, PCB designs, BOM
├── manufacturing/  # Assembly procedures, test fixtures
└── docs/           # Product documentation
```

## Getting Started

### Firmware

```bash
cd firmware
cargo build --release
```

See [firmware/README.md](firmware/README.md) for details.

### Base Station (Depot)

```bash
cd depot
./scripts/setup.sh
```

See [depot/README.md](depot/README.md) for fleet monitoring setup.

## Hardware

- **Compute:** Jetson Orin NX
- **Motors:** 4× hoverboard hub motors
- **Control:** 4× ESCs over CAN bus
- **Power:** 48V main rail, 12V accessory rail
- **Chassis:** 2020 aluminum extrusion, 24×24"

## License

Proprietary — Muni Municipal Robotics

