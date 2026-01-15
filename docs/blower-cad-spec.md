# Blower System CAD Specification

**Document Version:** 1.0
**Date:** 2026-01-14
**Status:** Draft
**Author:** Muni Robotics

## 1. Overview

### 1.1 System Architecture

The blower system integrates a commercial 90mm Electric Ducted Fan (EDF) motor/impeller unit into the BVR chassis via a custom 3D printed air handling system. The assembly consists of three primary components:

1. **Volute housing** - Converts radial airflow from the centrifugal impeller into directed axial flow
2. **Expansion nozzle** - Transitions from 90mm circular to 500mm × 30mm rectangular slot
3. **Shell integration** - Mounting hardware and cutouts for the powder-coated aluminum chassis shell

### 1.2 Design Goals

- **Exit velocity:** 120-140 MPH (54-63 m/s)
- **Volumetric flow:** 400-500 CFM (11.3-14.2 m³/min)
- **Outdoor durability:** ASA filament with UV and thermal resistance
- **Maintainability:** Tool-free impeller access, field-replaceable parts
- **Safety:** No exposed rotating components, guarded intake

### 1.3 Performance Requirements

| Parameter | Target | Notes |
|-----------|--------|-------|
| Exit velocity | 120-140 MPH | Measured at nozzle exit |
| Exit slot dimensions | 500mm × 30mm | Rectangular cross-section |
| Expansion angle | ≤15° half-angle | Prevents flow separation |
| Discharge angle | 20° downward | For snow clearing effectiveness |
| Motor power | 1500-2000W | Commercial EDF unit |
| Impeller diameter | 90mm | Pre-balanced assembly |

---

## 2. Volute Housing Specifications

### 2.1 Geometry

The volute housing captures radial airflow from the centrifugal impeller and converts it to directed axial flow. Design follows classic centrifugal scroll principles:

```
Top view (impeller rotation CW when viewed from front):

        ╭─────────╮
       ╱           ╲
      │    EDF      │  ← Motor mounting face
      │  Impeller   │
       ╲           ╱
        ╰─────┬───╯
              │
         ╔════▼════╗  ← Volute scroll (expanding radius)
         ║         ║
         ║    O    ║  ← Impeller center
         ║         ║
         ╚════╤════╝
              │
              └────→ To expansion nozzle (90mm outlet)

Side view:

    ┌─────────┐
    │  Motor  │
    │  Mount  │
    └────┬────┘
         │
    ╔════▼════╗
    ║ Volute  ║ ← 3-4mm wall thickness
    ╚════╤════╝
         │
         └────→ 90mm circular outlet
```

**Scroll profile:**
- Start radius: 46mm (just outside impeller tip clearance of 1mm)
- End radius: 65mm (at cutoff)
- Spiral angle: 270° wrap
- Cutoff angle: 90° from outlet centerline
- Tongue clearance: 2mm from impeller tip

**Dimensions:**
- Inlet diameter: 90mm (impeller OD + 2mm clearance)
- Outlet diameter: 90mm (circular, concentric transition to nozzle)
- Housing depth: 55mm (accommodates 50mm impeller + 5mm clearance)
- Overall diameter: 140mm (maximum at spiral end)

### 2.2 Material Specifications

**Primary material:** ASA (Acrylonitrile Styrene Acrylate)
- UV resistant (outdoor use)
- Glass transition temp: 100°C (adequate for motor heat)
- Impact resistant
- Minimal moisture absorption

**Print settings:**
- Layer height: 0.2mm
- Wall thickness: 3-4mm (12-16 perimeters at 0.4mm nozzle)
- Infill: 30% gyroid or honeycomb
- Top/bottom layers: 5 layers minimum

### 2.3 Motor Mounting Interface

The volute front face provides mounting for the commercial EDF motor unit.

**Mounting bolt pattern:**
- 4× M4 threaded inserts (heatset brass, 5mm OD × 6mm length)
- Bolt circle diameter: 100mm (verify against actual EDF motor)
- Counterbore depth: 2mm (flush M4 socket head cap screws)

**Impeller inlet:**
- Central hole diameter: 92mm (allows impeller hub passage)
- Stepped recess: 1mm deep × 95mm diameter (motor housing flange seat)
- O-ring groove: 3mm wide × 2mm deep at 97mm diameter (optional air seal)

**Alignment features:**
- 2× dowel pin holes: 4mm diameter × 6mm deep, 180° opposed
- Locating boss: 85mm diameter × 2mm height (concentric pilot)

### 2.4 Outlet Interface

**Outlet flange:**
- OD: 110mm
- ID: 90mm
- Thickness: 5mm
- Bolt holes: 4× M5, 100mm bolt circle, 90° spacing

**Sealing surface:**
- Flat face (±0.2mm flatness)
- Optional gasket groove: 2mm wide × 1mm deep

### 2.5 Print Orientation and Supports

**Recommended orientation:**
- Outlet flange face DOWN on build plate
- Motor mounting face UP
- Minimizes support material inside airflow path

**Support requirements:**
- Tree supports for scroll undercuts
- Support interface layer: 0.2mm gap
- Avoid supports on motor mounting face (use support blockers)

**Post-processing:**
- Remove supports carefully with flush cutters
- Light sanding of internal scroll surface (220 grit)
- Insert heatset inserts at 250°C with soldering iron tip

**Estimated print time:** 14-18 hours (0.4mm nozzle, 0.2mm layers)

---

## 3. Expansion Nozzle Specifications

### 3.1 Transition Geometry

The expansion nozzle smoothly transitions circular flow to rectangular slot output while maintaining attached flow (no separation).

```
Side view (vertical transition):

Inlet:  ⌒⌒⌒  90mm diameter
        ║ ║
        ║ ║  ← Gradual taper (15° half-angle max)
        ║ ║
       ╔═══╗
      ╔════╗
     ╔═════╗
    ╔══════╗ 30mm height
    ╚══════╝
Outlet: 500mm × 30mm slot

Top view (horizontal transition):

Inlet:   ╭─╮  90mm diameter
         │ │
        ╭───╮
       ╭─────╮
      ╭───────╮
     ╭─────────╮
    ╭───────────╮
   ╭─────────────╮
  ╔═══════════════╗ 500mm width
  ╚═══════════════╝
```

**Transition profile:**
- Inlet: 90mm diameter (circular)
- Outlet: 500mm × 30mm (rectangular with rounded corners)
- Length: 300mm (measured along centerline)
- Expansion half-angle: 14.5° vertical, 38° horizontal
- Wall thickness: 3mm minimum

**Notes on geometry:**
- Vertical expansion is gentle (14.5° < 15° limit, prevents separation)
- Horizontal expansion is aggressive (38° > 15°) BUT acceptable because:
  - Short aspect ratio transition
  - Exit velocity remains high (prevents backflow)
  - Slight turbulence acceptable for snow clearing (not precision airflow)
- Alternatively, split into two stages if testing shows separation issues

**Corner radii:**
- Inlet to nozzle: 10mm blend radius (smooth transition)
- Outlet corners: 15mm radius (reduces stress concentration)

### 3.2 Mounting to Volute

**Inlet flange:**
- OD: 110mm (matches volute outlet flange)
- ID: 90mm
- Thickness: 5mm
- Bolt holes: 4× M5 clearance (5.5mm diameter), 100mm bolt circle

**Assembly:**
- Gasket: 1mm silicone sheet or EPDM foam (optional, reduces vibration)
- Fasteners: 4× M5 × 20mm socket head cap screws
- Torque: 4-5 Nm (thread locker recommended)

### 3.3 Discharge Angle

The nozzle is angled 20° downward relative to horizontal chassis plane for optimal snow clearing.

```
Side view showing angle:

Chassis reference ─────────────
                ╲
              20° ╲  ← Nozzle centerline
                   ╲
                    ╲___  Ground interaction zone
                         ╲
```

**Implementation:**
- The 20° angle is built into the nozzle body geometry
- Nozzle mounts square to volute (volute provides the angle)
- Volute mounting brackets set the 20° pitch relative to chassis

### 3.4 Outlet Features

**Reinforcement:**
- 5mm thick flange around entire outlet perimeter
- 3mm internal ribs every 100mm along length (prevents flexing)

**Edge treatment:**
- 2mm radius on all sharp edges (safety, reduces wear on rubber snow skirt)

**Outlet sealing surface:**
- Flat face for rubber snow containment skirt attachment
- 3mm × 3mm step recess inboard (skirt capture groove)

### 3.5 Print Orientation and Supports

**Recommended orientation:**
- Inlet flange DOWN on build plate
- Outlet facing UP
- Nozzle axis parallel to build plate Y-axis (maximizes strength in longest dimension)

**Support requirements:**
- Supports needed under horizontal expansion surfaces
- Use organic tree supports (follows curves better than linear)
- 0.2mm support gap

**Segmentation option:**
If nozzle exceeds build volume (>350mm printers), split into two pieces:
- Joint at 150mm from inlet (mid-length)
- 10mm lap joint with 6× M4 threaded inserts
- Seal with silicone caulk during assembly

**Estimated print time:** 20-28 hours (monolithic), 12-16 hours per half (segmented)

---

## 4. Shell Integration

### 4.1 Shell Front Panel Cutout

The powder-coated aluminum shell front panel requires a precise cutout for the nozzle outlet.

**Cutout dimensions:**
- Width: 500mm (matches nozzle outlet)
- Height: 50mm (30mm slot + 10mm clearance top/bottom)
- Corner radius: 15mm (matches nozzle, reduces stress)
- Location: Centered horizontally, 120mm above chassis bottom plane

**Edge treatment:**
- Debur all edges (file or tumble after laser cutting)
- Optional: 2mm foam weather seal around perimeter (adhesive backed EPDM)

**Clearance:**
- 10mm gap around nozzle outlet (allows thermal expansion, vibration)

### 4.2 Mounting Points to Chassis

The blower assembly mounts to the 2020 aluminum extrusion chassis frame using custom brackets.

**Volute mounting brackets (2×):**
```
Side view of bracket:

    ╔════════╗  ← Volute mounting face (vertical)
    ║        ║
    ║  M5    ║  ← Threaded holes in volute body
    ║        ║
    ╚═══╤════╝
        │
        └─┐ 90° bend
          │
      ┌───┴────┐  ← 2020 extrusion mounting face (horizontal)
      │  ╱  ╲  │  ← T-nut slot clearance
      │ O    O │  ← M5 clearance holes for T-nuts
      └────────┘
```

**Bracket specifications:**
- Material: 3mm aluminum plate or 5mm ASA printed part
- Volute interface: 2× M5 threaded inserts per bracket
- Chassis interface: 2× M5 T-slot nuts per bracket (1010 series)
- Angle: Bracket provides 20° pitch (nozzle points downward)

**T-slot nut specifications:**
- Style: Drop-in T-nut (2020 compatible)
- Fastener: M5 × 12mm button head cap screw
- Torque: 5-6 Nm

**Nozzle support bracket (1×):**
- Attaches to nozzle outlet flange underside
- Mounts to chassis cross-member with 2× M5 T-nuts
- Prevents cantilever stress on volute outlet

### 4.3 Intake Vents

Air intake vents in the shell rear panel supply the blower system.

**Vent specifications:**
- Location: Top rear panel, above motor
- Total area: 100-150mm² (1.1-1.7× blower inlet area)
- Pattern: 12× 10mm diameter holes in 3×4 grid
- Spacing: 25mm centers
- Baffle: Internal deflector plate (prevents direct water ingress)

```
Rear panel (exterior view):

    Shell surface
    ┌─────────────────┐
    │  O   O   O   O  │  ← 10mm holes
    │                 │
    │  O   O   O   O  │
    │                 │
    │  O   O   O   O  │
    └─────────────────┘

Interior baffle (angled deflector):

          Air path
            ↓  ↓  ↓
    Shell ─┬───────┬─
           │  ╱ ╲  │ ← 45° baffle
           │╱     ╲│
           └───────┘ → To motor inlet
```

**Baffle mount:**
- 3D printed ASA part
- Clip-fit to shell interior or M3 fasteners
- 45° angle deflects water down and out

### 4.4 Drain Holes

Prevent water accumulation in blower housing cavity.

**Drain hole specifications:**
- Location: Bottom corners of shell cavity (2×)
- Diameter: 6mm
- Grommet: Rubber edge trim grommet (prevents chafing)
- Position: Lowest point when chassis is level

**Drainage path:**
- Holes positioned to drain water that enters via intake vents
- Not directly under motor (electronics protection)

---

## 5. CAD Checklist

### 5.1 Critical Dimensions to Verify

**Before finalizing CAD, measure and verify:**

- [ ] EDF motor mounting bolt pattern (may vary by manufacturer)
- [ ] EDF motor housing OD and flange thickness
- [ ] Impeller outer diameter (should be 90mm ±1mm)
- [ ] Impeller hub projection (volute inlet clearance)
- [ ] 2020 extrusion T-slot dimensions (verify drop-in T-nut fit)
- [ ] Shell front panel thickness (affects cutout edge clearance)
- [ ] Chassis mounting points (exact X/Y/Z coordinates for brackets)
- [ ] Ground clearance at nozzle outlet (min 50mm when pitched 20°)

### 5.2 Clearances to Check

**Use CAD interference detection:**

- [ ] Impeller to volute scroll clearance: 1-2mm (360° around)
- [ ] Motor housing to volute front face: No interference on bolt heads
- [ ] Nozzle outlet to shell cutout: 10mm gap all around
- [ ] Blower assembly to chassis frame: 15mm min clearance (vibration)
- [ ] Intake vents to motor inlet: Unobstructed air path
- [ ] Drain holes to ground: Holes not blocked when rover on flat surface
- [ ] Cable routing: 25mm clearance for motor power cable (12AWG)

### 5.3 Assembly Order Verification

**Simulate assembly in CAD to confirm:**

1. [ ] Install heatset inserts into volute (before mounting motor)
2. [ ] Attach motor to volute (4× M4 screws accessible with hex key)
3. [ ] Mount volute brackets to 2020 extrusion (T-nuts slide in from end)
4. [ ] Attach volute assembly to brackets (4× M5 screws)
5. [ ] Bolt nozzle to volute outlet flange (4× M5 screws from inside)
6. [ ] Install nozzle support bracket to chassis
7. [ ] Slide shell over chassis (nozzle outlet passes through cutout)
8. [ ] No interference or trapped fasteners

### 5.4 Stress Analysis Considerations

**Structural checks (if using FEA simulation):**

- [ ] Nozzle cantilever load: 2kg mass at outlet (simulates impact)
- [ ] Motor vibration: 50Hz excitation, 5G peak (unbalanced impeller scenario)
- [ ] Mounting bracket bending: 10kg load at volute center of mass
- [ ] Thermal expansion: 60°C operating temp (ASA Tg check)

**Target safety factor:** 2.5× minimum for printed parts

### 5.5 STL Export Settings

**For manufacturing-ready STL files:**

- [ ] Resolution: 0.01mm tolerance (high quality)
- [ ] Units: Millimeters
- [ ] Repair mesh: Close holes, fix normals
- [ ] Orientation: Part in print orientation (correct Z-axis)
- [ ] File naming: `blower-volute-v1.0.stl`, `blower-nozzle-v1.0.stl`
- [ ] Manifold check: All edges shared by exactly 2 faces
- [ ] Scale verification: Import into slicer and measure known dimension

---

## 6. Bill of Materials - 3D Printed Parts

| Part Name | Quantity | Material | Est. Weight | Est. Print Time | Notes |
|-----------|----------|----------|-------------|-----------------|-------|
| Volute housing | 1 | ASA | 450g | 16h | Includes motor mount |
| Expansion nozzle | 1 | ASA | 800g | 24h | Monolithic version |
| Expansion nozzle (half A) | 1 | ASA | 420g | 14h | Segmented version |
| Expansion nozzle (half B) | 1 | ASA | 420g | 14h | Segmented version |
| Intake baffle | 1 | ASA | 40g | 2h | Water deflector |
| Volute mounting bracket | 2 | ASA | 60g ea. | 3h ea. | Alt: 3mm aluminum |
| Nozzle support bracket | 1 | ASA | 50g | 2.5h | Outlet stabilizer |

**Total print time (monolithic nozzle):** ~42 hours
**Total print time (segmented nozzle):** ~51 hours
**Total filament:** ~1.5 kg ASA

**Hardware (not printed):**
- 4× M4 × 12mm socket head cap screws (motor to volute)
- 4× M4 heatset inserts, 5mm OD × 6mm length
- 8× M5 × 20mm socket head cap screws (nozzle to volute, brackets to volute)
- 8× M5 × 12mm button head cap screws (brackets to chassis T-nuts)
- 8× M5 drop-in T-nuts, 2020 series
- 6× M4 × 16mm socket head cap screws (nozzle segment joint, if segmented)
- 12× M4 heatset inserts (nozzle segment joint, if segmented)
- 1× Silicone caulk tube (assembly sealing)
- 2× Rubber edge trim grommets, 6mm ID (drain holes)

**Print settings reminder:**
- Nozzle: 0.4mm
- Layer height: 0.2mm
- Wall thickness: 3-4mm (12-16 perimeters)
- Infill: 30% gyroid
- Material: ASA (outdoor rated)
- Bed temp: 100°C, nozzle temp: 250°C
- Enclosure: Required (ASA warps without)

---

## 7. Design Notes and Rationale

### 7.1 Why ASA Over Other Materials

**ASA selected over alternatives:**
- **vs. PLA:** PLA softens at 60°C (inadequate for summer outdoor use)
- **vs. PETG:** PETG creeps under sustained load, absorbs moisture
- **vs. ABS:** ASA has superior UV resistance (ABS yellows and degrades)
- **vs. Nylon:** Nylon requires dry box storage, expensive, hygroscopic
- **vs. Polycarbonate:** PC is overkill (more difficult to print, unnecessary strength)

### 7.2 Expansion Angle Trade-offs

The nozzle horizontal expansion (38° half-angle) exceeds the classical 15° diffuser limit. This is acceptable because:

1. **Application context:** Snow clearing tolerates turbulence (not a precision air bearing)
2. **High exit velocity:** 120+ MPH prevents backflow even with separation
3. **Short length:** 300mm transition limits separation region
4. **Alternative:** Two-stage expansion (adds complexity, print time, leak potential)

**Testing plan:** If field testing shows poor performance (low velocity, high motor current), redesign as two-stage:
- Stage 1: 90mm circular → 200mm × 45mm (15° vertical, 15° horizontal)
- Stage 2: 200mm × 45mm → 500mm × 30mm (gradual rectangular morph)

### 7.3 Print Segmentation Decision

**Monolithic nozzle:**
- Pros: No joints, no leaks, stronger
- Cons: Requires ≥350mm build plate, long print time

**Segmented nozzle:**
- Pros: Fits smaller printers (250mm class), easier re-printing if damaged
- Cons: Joint requires sealing, slight performance loss

**Recommendation:** Print monolithic if printer allows. Use segmented as fallback.

### 7.4 Mounting Strategy

The blower mounts to 2020 extrusion (not directly to shell) because:
- Shell is cosmetic, not structural
- Vibration isolation (shell floats on rubber mounts)
- Serviceability (remove shell without disturbing blower)
- Load path (blower forces go directly to chassis frame)

---

## 8. Future Enhancements

**Potential improvements for future revisions:**

1. **Active cooling:** Add 40mm axial fan to motor housing (extends duty cycle)
2. **Vibration damping:** Rubber isolators at mounting brackets (reduces noise)
3. **Modular nozzles:** Quick-change outlet inserts (different slot widths for tuning)
4. **Sensor integration:** Anemometer at outlet (measure actual velocity, closed-loop control)
5. **Sound attenuation:** Internal acoustic foam lining (reduces 6-8kHz whine)

---

## Appendices

### A. Reference Standards

- **Centrifugal fan design:** AMCA 210 (Air Movement and Control Association)
- **Diffuser expansion limits:** Fluid Mechanics, White (McGraw-Hill), Chapter 6
- **ASA material properties:** ASTM D638 (tensile), ASTM D648 (heat deflection)

### B. CAD Software Recommendations

- **Fusion 360:** Excellent for sheet metal shell, T-slot frame modeling
- **SolidWorks:** Industry standard, parametric design
- **Onshape:** Cloud-based, free for public projects
- **FreeCAD:** Open-source option

### C. Print Service Specifications

If outsourcing 3D printing:

**Material:** ASA (UV stabilized)
**Layer height:** 0.2mm
**Infill:** 30% minimum
**Wall thickness:** 4mm
**Finish:** As-printed (no post-processing required)
**Tolerances:** ±0.2mm on mounting features, ±0.5mm on airflow surfaces
**Delivery format:** Upload STL files, specify print orientation in notes

---

**END OF SPECIFICATION**
