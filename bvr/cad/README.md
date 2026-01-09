# BVR CAD

Parametric CAD system for BVR rover parts, written in Rust using [manifold-rs](https://github.com/WilstonOreo/manifold-rs).

## Structure

```
cad/
├── src/
│   ├── lib.rs          # Core Part abstraction
│   ├── step.rs         # STEP export (optional, requires OpenCASCADE)
│   ├── parts/          # Part definitions
│   │   ├── motor_mount.rs      # Custom: L-bracket motor mounts
│   │   ├── electronics_plate.rs # Custom: Electronics mounting plate
│   │   ├── sensor_mount.rs     # Custom: Sensor mast bracket
│   │   ├── wheel_spacer.rs     # Custom: Wheel spacers
│   │   ├── frame.rs            # Frame: 2020 extrusion, brackets
│   │   ├── hub_motor.rs        # Reference: Hub motors
│   │   ├── sensors.rs          # Reference: LiDAR, cameras, GPS
│   │   ├── electronics.rs      # Reference: VESCs, Jetson, DC-DC
│   │   ├── battery.rs          # Reference: Battery packs
│   │   └── assembly.rs         # Complete BVR1 assembly
│   └── bin/
│       └── generate_all.rs
├── exports/            # Generated STL files (29 parts)
└── viewer.html         # 3D viewer (Three.js)
```

## Quick Start

```bash
# Build the CAD library
cargo build

# Generate all parts (31 STL files)
cargo run --bin generate-all

# Run tests (51 tests)
cargo test

# View parts in browser
open viewer.html
```

## Part Categories

### Custom Fabricated Parts

These are parts designed for manufacturing (3D printing, laser cutting):

| Part | Description | File |
|------|-------------|------|
| Motor Mount | L-bracket for hub motors | `motor_mount_8in.stl` |
| Electronics Plate | Jetson + 4x VESC mounting | `electronics_plate.stl` |
| Sensor Mount | 1" tube clamp for mast | `sensor_mount.stl` |
| Wheel Spacer | Hub motor axle spacer | `wheel_spacer_8in.stl` |
| Battery Tray | Custom battery enclosure | `battery_tray.stl` |

### Reference Parts (Visualization)

Simplified models of off-the-shelf components for assembly visualization:

| Part | Description | File |
|------|-------------|------|
| Hub Motor 8" | 48V hub motor with wheel | `ref_hub_motor_8in.stl` |
| Hoverboard Motor | 6.5" BVR0-style motor | `ref_hub_motor_hoverboard.stl` |
| LiDAR Mid-360 | Livox Mid-360 sensor | `ref_lidar_mid360.stl` |
| Camera | Insta360 X4 | `ref_camera_insta360.stl` |
| GPS Antenna | RTK antenna | `ref_gps_antenna.stl` |
| VESC 6 | Motor controller | `ref_vesc_6.stl` |
| Jetson | reComputer carrier | `ref_jetson_recomputer.stl` |
| DC-DC | 48V to 12V converter | `ref_dcdc.stl` |
| E-Stop | Mushroom button | `ref_estop.stl` |
| Downtube Battery | BVR0 e-bike battery | `ref_battery_downtube.stl` |
| Custom Battery | BVR1 13S4P pack | `ref_battery_custom.stl` |

### Frame Components

2020 aluminum extrusion system:

| Part | Description | File |
|------|-------------|------|
| 2020 Extrusion | Standard profiles | `extrusion_2020_*.stl` |
| Corner Bracket | 90° joining bracket | `corner_bracket.stl` |
| T-Nut | M5 sliding nut | `tnut_m5.stl` |
| BVR1 Frame | Complete chassis | `bvr1_frame.stl` |

### Complete Assemblies

| Assembly | Description | File |
|----------|-------------|------|
| BVR0 Full | Prototype with hoverboard motors, downtube battery | `bvr0_assembly.stl` |
| BVR0 Simple | Simplified BVR0 for fast rendering | `bvr0_assembly_simple.stl` |
| BVR1 Full | Production with 8" motors, custom battery | `bvr1_assembly.stl` |
| BVR1 Simple | Simplified BVR1 for fast rendering | `bvr1_assembly_simple.stl` |

## Usage Examples

### Frame

```rust
use bvr_cad::parts::{Extrusion2020, CornerBracket, BVR1Frame};

// Single extrusion
let rail = Extrusion2020::new(500.0).generate();
rail.write_stl("rail.stl")?;

// Complete BVR1 frame assembly
let frame = BVR1Frame::default_bvr1().generate();
frame.write_stl("bvr1_frame.stl")?;
```

### Motor Mount

```rust
use bvr_cad::parts::MotorMount;

let mount = MotorMount::hub_motor_8in();
mount.generate().write_stl("motor_mount.stl")?;
mount.generate_flat_plate().write_stl("plate.stl")?; // For laser cutting
```

### Complete Assembly

```rust
use bvr_cad::parts::{BVR0Assembly, BVR1Assembly};

// BVR0: Prototype with hoverboard motors, downtube battery
let bvr0 = BVR0Assembly::default_bvr0();
bvr0.generate().write_stl("bvr0_complete.stl")?;

// BVR1: Production with 8" motors, custom battery
let bvr1 = BVR1Assembly::default_bvr1();
bvr1.generate().write_stl("bvr1_complete.stl")?;
```

### Custom Primitives

```rust
use bvr_cad::{Part, centered_cube, centered_cylinder, bolt_pattern};

// Primitives
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

## Manufacturing

### Laser Cutting (SendCutSend)
- Use `generate_flat_plate()` methods for 2D profiles
- Export STL, convert to DXF using FreeCAD/Blender

### 3D Printing
- Use `generate()` for full 3D parts
- STL files ready for slicing

### STEP Export (for Shapr3D, Fusion 360)
Requires OpenCASCADE (optional feature):

```bash
# Install dependencies
brew install cmake opencascade  # macOS
sudo apt install cmake libocct-dev  # Ubuntu

# Build with STEP support
cargo build --features step
```

## Dependencies

- `manifold-rs` - CSG geometry kernel (requires cmake, ninja)
- `nalgebra` - Linear algebra
- `opencascade` (optional) - STEP export (requires OCCT)

## Build Requirements

```bash
# macOS
brew install cmake ninja

# Ubuntu/Debian
sudo apt install cmake ninja-build build-essential
```
