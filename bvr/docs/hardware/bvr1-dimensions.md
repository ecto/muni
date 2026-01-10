# BVR1 Dimension Optimization

This document captures the first-principles analysis used to determine optimal dimensions for BVR1.

## Design Goals

1. **Maximize accessibility**: Fit through narrow sidewalk pinch points
2. **Maintain stability**: Safe operation during turns and on slopes
3. **Preserve capability**: Sufficient traction and battery for snow work
4. **Keep it simple**: Fewer parts, easier to build and maintain

## Constraints

### Hard Constraints

| Constraint | Requirement | Source |
|------------|-------------|--------|
| ADA clear path | Leave ≥915mm (36") when yielding | ADA 4.3.3 |
| Component fit | Jetson, 4× VESC, battery must fit | BOM |
| Ground clearance | ≥75mm for snow/curb cuts | Field experience |
| Stability | Tip angle >30° at rest | Safety |

### Task Requirements

| Task | Force (N) | Precision | Frequency |
|------|-----------|-----------|-----------|
| Snow plowing | 50-100 | Low | Core |
| Snow blowing | 20-50 | Low | Core |
| Salt spreading | ~0 | Medium | Core |
| Debris pickup | 10-20 | High | Occasional |

## Analysis

### 1. Component Inventory

| Component | L×W×H (mm) | Notes |
|-----------|------------|-------|
| Jetson Orin NX (reComputer) | 130×120×50 | Fixed |
| VESC 6 (×4) | 75×40×20 each | Can stack 2×2 |
| Battery (48V 15Ah) | 280×140×70 | ~3L volume, 720Wh |
| DC-DC converter | 100×60×30 | |
| Wiring margins | +50mm each axis | |

**Minimum electronics footprint**: ~350mm × 350mm

### 2. Stability Analysis

#### Static Tip-Over

```
Tip angle θ = atan(W_track / (2 × H_cg))

For stability: θ > 30° (conservative)
Therefore: W_track > 2 × H_cg × tan(30°)
```

With H_cg ≈ 150mm:
```
W_track_min = 2 × 150 × 0.577 = 173mm
```

#### Dynamic Stability (Turning)

At maximum speed in a tight turn:
```
W_track > 2 × v²_max × H_cg / (r_min × g)
```

With v_max = 2.5 m/s, r_turn = 0.5m:
```
W_track_min = 2 × (2.5)² × 0.15 / (0.5 × 9.8) = 383mm
```

**Dynamic stability is the binding constraint.**

### 3. Traction Analysis

```
F_traction = μ × m × g

For icy conditions: μ ≈ 0.2-0.3
Required force for light snow work: ~50N
```

Minimum mass:
```
m_min = 50N / (0.3 × 9.8) = 17kg
```

Target mass: 20-22kg (provides margin for heavier conditions)

### 4. Wheel Selection

| Wheel | Diameter | Hub Width | Clearance | Notes |
|-------|----------|-----------|-----------|-------|
| 6.5" hoverboard | 165mm | 80mm | ~50mm | BVR0, marginal clearance |
| 8" scooter | 200mm | 85mm | ~75mm | Good balance |
| 10" pneumatic | 250mm | 90mm | ~100mm | Overkill, adds height |

**Selected: 6.5" (168mm) UUMotor SVB6HS**
- Proven, available, lower CG than 8" wheels
- 75mm effective ground clearance with current mount geometry

### 5. Mast Height Optimization

The sensor mast carries LiDAR, 360° camera, and GPS antenna. Height is constrained by:

#### Perception Requirements

**LiDAR body clearance**: The Livox Mid-360 has -7° to +52° vertical FOV. The -7° down angle must clear the robot body:

```
        LiDAR at height h
              ●
             /│\
            / │ \  ← -7° max down angle
           /  │  \
     ┌────/───┴───\────┐  ← Body top at Z=300mm
     │   BODY          │
     └─────────────────┘
```

With mast 200mm behind body center, front edge 450mm away:
```
tan(7°) = (h - 300) / 450
h_min = 300 + 450 × tan(7°) = 355mm
```

**Camera ground visibility**: The Insta360 X4 needs to see ground near the robot. For ground at body edge (300mm horizontal) with 60° down angle:
```
h_cam = 300 × tan(60°) = 520mm minimum
```

Since camera is 100mm below LiDAR:
```
h_lidar > 520 + 100 = 620mm from ground
```

#### Stability Impact

Mast sensors affect CG:

| Component | Mass (kg) | Height |
|-----------|-----------|--------|
| LiDAR | 0.5 | h_mast |
| Camera | 0.3 | h_mast - 100mm |
| GPS | 0.1 | h_mast - 50mm |
| Mast tube | 0.5 | h_mast / 2 |

CG sensitivity: Every 100mm taller → 5.8mm higher CG → ~0.5° less tip margin

#### Vibration

Mast natural frequency decreases with length (f ∝ 1/L²). At 400mm, f_n ≈ 80 Hz (safe). Longer masts risk resonance with motor/wheel vibrations.

#### LiDAR Forward Blind Zone

The -7° down angle creates a forward blind zone where ground is invisible:
```
d_blind = h_lidar / tan(7°) = 8.1 × h_lidar
```

| LiDAR Height | Blind Zone |
|--------------|------------|
| 600mm | 4.9m |
| 700mm | 5.7m |
| 800mm | 6.5m |

This is mitigated by: (1) 360° camera for near-field, (2) slowing near obstacles.

#### Optimal Mast Height

| Constraint | Requirement | Min Value |
|------------|-------------|-----------|
| LiDAR clears body | h_lidar > 355mm | 355mm |
| Camera sees ground | h_cam > 520mm | 620mm for LiDAR |
| CG stability | Lower is better | - |
| Vibration | Shorter is stiffer | - |

**Optimal: mast_height = 400mm above frame** (LiDAR at 700mm from ground)

This provides 80mm margin above the camera visibility minimum, with acceptable CG and vibration trade-offs.

### 6. Body vs Track Width

The body can be narrower than the track if wheels protrude:

```
    ◯────────────────────◯  ← Wheels at track width
       ┌──────────────┐
       │    BODY      │     ← Body narrower
       └──────────────┘
    ◯────────────────────◯

    ◄───── W_track ──────►
       ◄── W_body ──►
```

This allows:
- Narrow body for component fit
- Wide track for stability
- Total width = W_track + wheel_hub_width

## Optimized Dimensions

### Frame (2020 Extrusion)

| Dimension | Value | Rationale |
|-----------|-------|-----------|
| Width | 380mm | Fits components, allows wheel protrusion |
| Length | 500mm | Compact, adequate wheelbase |
| Height | 180mm | Low CG, fits battery + electronics |

### Overall Robot

Wheels are positioned OUTSIDE the frame to avoid intersection with the narrow body.

| Dimension | Value | Notes |
|-----------|-------|-------|
| Track width | ~548mm | Wheel centers (frame + bracket + axle offset) |
| Wheelbase | 460mm | Frame length minus corner posts |
| Total width | ~600mm | Track + wheel hub width |
| Total length | ~550mm | Frame + wheel protrusion |
| Total height | ~700mm | Body + mast |
| Ground clearance | 75mm | With L-bracket mounts |

Wheel placement calculation:
```
wheel_center_x = frame_edge + bracket_depth + axle_offset
               = 190mm + 20mm + 64mm = 274mm from centerline
track_width    = 2 × 274mm = 548mm
total_width    = track_width + hub_width = 548 + 52 = 600mm
```

### Mass Budget

| Component | Mass (kg) |
|-----------|-----------|
| Frame (aluminum) | 3.5 |
| Motors (4×) | 4.0 |
| VESCs (4×) | 0.8 |
| Battery | 4.0 |
| Jetson + carrier | 0.5 |
| Wiring, fasteners | 1.5 |
| Sensors + mast | 1.5 |
| Electronics plate | 0.5 |
| **Subtotal** | **16.3** |
| Contingency (20%) | 3.3 |
| **Total** | **~20kg** |

## Comparison to BVR0

| Spec | BVR0 | BVR1 | Change |
|------|------|------|--------|
| Frame width | 500mm | 380mm | -24% |
| Frame length | 500mm | 500mm | 0% |
| Total width | ~600mm | ~600mm | 0% |
| Mass | ~30kg | ~20kg | -33% |
| Ground clearance | 50mm | 75mm | +50% |

Note: Total width is similar because BVR1 uses narrower frame but wheels protrude outside to avoid intersection. The narrower frame still provides better component layout and lower mass.

## Accessibility Analysis

With wheels outside frame, total width is ~600mm (similar to BVR0). The narrower frame provides:
- Better internal component layout
- Lower mass (20kg vs 30kg)
- Improved stability (lower CG)

| Sidewalk Width | BVR1 (600mm) | Clearance |
|----------------|--------------|-----------|
| 4.0 ft (1220mm) | ⚠️ Tight | 620mm total |
| 4.5 ft (1370mm) | ✅ OK | 770mm total |
| 5.0 ft (1524mm) | ✅ Good | 924mm total |

| Pinch Point | Fits? | Notes |
|-------------|-------|-------|
| 700mm gap | ✅ Yes | 100mm margin |
| 650mm gap | ✅ Barely | 50mm margin |
| 600mm gap | ❌ No | Need exact fit |

## Stability Analysis

### Center of Gravity

| Component | Mass (kg) | Height (mm) | Moment |
|-----------|-----------|-------------|--------|
| 4× hub motors | 4.0 | 84 | 336 |
| Battery | 4.0 | 150 | 600 |
| Frame | 3.5 | 210 | 735 |
| Electronics | 2.0 | 160 | 320 |
| Trays/panels | 1.0 | 170 | 170 |
| Mast + sensors | 1.5 | 550 | 825 |
| Wiring/misc | 4.0 | 200 | 800 |
| **Total** | **20 kg** | | **3786** |

**H_cg = 3786 / 20 = 189mm**

### Tip-Over Angles

```
Roll:  θ = atan((W_track/2) / H_cg) = atan(200/189) = 46.6°
Pitch: θ = atan((L_wheelbase/2) / H_cg) = atan(230/189) = 50.6°
```

Both angles exceed 45°, indicating good static stability.

### Real-World Scenarios

| Scenario | Angle | Risk |
|----------|-------|------|
| Sidewalk cross-slope | 1-2° | ✅ Safe |
| Steep curb cut | 8° | ✅ Safe |
| ADA max ramp | 4.8° | ✅ Safe |
| Aggressive turn (2m/s, r=1m) | ~12° equivalent | ✅ Safe |
| Wheel impact with obstacle | Impulse | ⚠️ Main risk |

### Mitigation

1. **Software rate limiting**: Caps acceleration to prevent jerky movements
2. **Speed limiting near obstacles**: LiDAR-triggered slowdown reduces impact energy
3. **Low CG design**: Heavy components (battery, motors) positioned low
4. **Snow accumulation awareness**: Operational procedure to clear before use

## Trade-offs Accepted

1. **Less traction force**: -33% mass means -33% max pushing force on ice. Mitigated by targeting lighter snow conditions and accepting slower operation in heavy snow.

2. **Smaller battery**: ~720Wh vs 960Wh. Runtime reduced from ~4h to ~3h. Acceptable for typical shift length.

3. **Tighter component fit**: Less margin for wiring and future additions. Requires careful layout.

4. **Forward LiDAR blind zone**: 5.7m ahead not visible to LiDAR at ground level. Mitigated by camera and speed control.

## Future Considerations

1. **Ballast mode**: Optional weight plates for heavy snow conditions
2. **Tool width**: Foldable/extendable tools for wider clearing path
3. **Suspension**: If ground clearance proves insufficient, consider simple suspension

## References

- ADA Accessibility Guidelines: Section 4.3 (Accessible Route)
- UUMotor SVB6HS datasheet
- Livox Mid-360 mounting guidelines
