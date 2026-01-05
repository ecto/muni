# BVR CAD

Parametric CAD system for BVR rover parts, written in Rust using [manifold-rs](https://github.com/WilstonOreo/manifold-rs).

## Structure

```
cad/
├── src/
│   ├── lib.rs          # Core Part abstraction
│   ├── parts/          # Part definitions
│   │   └── motor_mount.rs
│   └── bin/            # CLI tools
│       └── motor_mount.rs
├── exports/            # Generated STL/DXF files
└── Cargo.toml
```

## Quick Start

```bash
# Build the CAD library
cargo build

# Generate motor mount STL files
cargo run --bin motor-mount

# Run tests
cargo test
```

## Parts Library

### Motor Mount (`parts/motor_mount.rs`)

L-bracket mount for hub motors, designed for:
- 8" hub motors (48V, ~200mm OD) - default
- 6.5" hoverboard motors (BVR0 style)

Features:
- Parametric dimensions (axle size, bolt pattern, plate thickness)
- Mounting tabs for 2020 aluminum extrusion
- M5/M6 clearance holes

```rust
use bvr_cad::parts::MotorMount;

let mount = MotorMount::hub_motor_8in();
let part = mount.generate();
part.write_stl("motor_mount.stl")?;
```

## Manufacturing

### Laser Cutting (SendCutSend)
Use `generate_flat_plate()` and `generate_tabs()` for 2D profiles.
Export as DXF for laser cutting services.

### 3D Printing
Use `generate()` for full L-bracket mount.
Export as STL for FDM or SLA printing.

## Part API

```rust
use bvr_cad::{Part, centered_cube, centered_cylinder, bolt_pattern};

// Create primitives
let plate = centered_cube("plate", 100.0, 100.0, 6.0);
let hole = centered_cylinder("hole", 5.0, 10.0, 32);

// Boolean operations
let result = plate.difference(&hole);

// Transformations
let moved = result.translate(10.0, 0.0, 0.0);
let rotated = moved.rotate(0.0, 0.0, 45.0);

// Export
result.write_stl("output.stl")?;
```

## Dependencies

- `manifold-rs` - CSG geometry kernel (Rust bindings for Manifold)
- `nalgebra` - Linear algebra

Note: manifold-rs requires a C++ compiler for building.
