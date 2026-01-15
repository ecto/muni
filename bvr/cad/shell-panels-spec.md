# BVR1 Shell Panel Specifications for SendCutSend

**Document Version:** 1.0
**Date:** 2026-01-15
**Material:** 5052 Aluminum, 2mm thickness
**Finish:** Orange powder coat

## Overview

Shell panels to cover the BVR1 chassis frame (380mm × 500mm × 180mm).

## Panel Dimensions

All dimensions in millimeters. Shell provides 20mm clearance around frame.

### 1. Front Panel

**Dimensions:** 420mm wide × 200mm tall

**Cutouts:**
- Blower nozzle slot: 500mm × 50mm, centered horizontally, bottom edge 100mm from panel bottom
  - Corner radius: 15mm
- Mounting holes: 8× M5 (5.3mm) holes on 10mm inset from edges

```
    420mm
┌───────────────────────────────────────────┐
│  ○                                     ○  │  ← M5 holes, 10mm from edge
│                                           │
│           ┌───────────────────┐           │  ← Nozzle slot: 500mm × 50mm
│           │                   │           │    centered, 100mm from bottom
│           │   500mm × 50mm    │           │
│           │    R15 corners    │           │
│           └───────────────────┘           │
│                                           │
│  ○                                     ○  │
└───────────────────────────────────────────┘
    200mm
```

### 2. Rear Panel

**Dimensions:** 420mm wide × 200mm tall

**Cutouts:**
- Intake vent: 12× 10mm diameter holes in 3×4 grid
  - Grid: 3 columns × 4 rows, 25mm spacing
  - Grid centered at 210mm from left, 150mm from bottom
- Drain holes: 2× 6mm holes at bottom corners (20mm inset)
- Mounting holes: 8× M5 (5.3mm) holes on 10mm inset from edges

```
    420mm
┌───────────────────────────────────────────┐
│  ○                                     ○  │
│                                           │
│              ○   ○   ○   ○                │  ← Intake vents: 12× 10mm
│              ○   ○   ○   ○                │    25mm spacing
│              ○   ○   ○   ○                │
│                                           │
│  ○                                     ○  │
│                                           │
│  ◐                                     ◐  │  ← 6mm drain holes
└───────────────────────────────────────────┘
    200mm
```

### 3. Side Panels (×2, mirror)

**Dimensions:** 540mm long × 200mm tall

**Cutouts:**
- Mounting holes: 10× M5 (5.3mm) holes on 10mm inset from edges
- Motor access cutout (optional): 100mm × 80mm at wheel locations

```
    540mm
┌───────────────────────────────────────────────────────────┐
│  ○        ○                            ○        ○         │
│                                                           │
│                                                           │
│  ○        ○                            ○        ○         │
│                                                           │
│  ○                                                     ○  │
└───────────────────────────────────────────────────────────┘
    200mm
```

### 4. Top Panel

**Dimensions:** 420mm wide × 540mm long

**Cutouts:**
- Sensor mast hole: 30mm diameter, centered at 210mm × 450mm (rear section)
- E-stop access: 30mm diameter, centered at 350mm × 100mm (front right)
- Mounting holes: 12× M5 (5.3mm) holes around perimeter

```
                    420mm
┌───────────────────────────────────────────┐
│  ○           ○           ○           ○    │
│                                           │
│                        ◎ ← E-stop (30mm)  │
│  ○                                     ○  │
│                                           │
│                                           │
│  ○                                     ○  │
│                                           │
│               ◎ ← Sensor mast (30mm)      │
│                                           │
│  ○           ○           ○           ○    │
└───────────────────────────────────────────┘
                    540mm
```

## Bending Requirements

None required for initial version. Panels are flat, assembled with L-brackets.

## Assembly Hardware

- 38× M5 × 10mm button head cap screws
- 38× M5 T-slot nuts (2020 compatible)
- 4× 90° corner brackets (hidden inside)
- Weather seal tape for panel joints (optional)

## SendCutSend Order Details

| Panel | Qty | Material | Thickness | Finish | Est. Cost |
|-------|-----|----------|-----------|--------|-----------|
| Front | 1 | 5052-H32 | 2mm | Orange powder | ~$25 |
| Rear | 1 | 5052-H32 | 2mm | Orange powder | ~$25 |
| Side (L) | 1 | 5052-H32 | 2mm | Orange powder | ~$30 |
| Side (R) | 1 | 5052-H32 | 2mm | Orange powder | ~$30 |
| Top | 1 | 5052-H32 | 2mm | Orange powder | ~$35 |
| **Total** | 5 | | | | **~$145** |

*Note: Prices are estimates. Actual cost depends on SendCutSend pricing.*

## DXF File Checklist

Generate these DXF files:
- [ ] `shell-front.dxf` - Front panel with nozzle cutout
- [ ] `shell-rear.dxf` - Rear panel with intake vents and drains
- [ ] `shell-side-left.dxf` - Left side panel
- [ ] `shell-side-right.dxf` - Right side panel (mirror of left)
- [ ] `shell-top.dxf` - Top panel with sensor and E-stop holes

## Notes

1. **Front panel nozzle cutout**: Per blower-cad-spec.md, the slot is 500mm × 50mm with 15mm corner radii
2. **Intake vents**: 12× 10mm holes provide 100-150% of blower inlet area
3. **Drain holes**: Positioned at lowest points when rover is level
4. **Powder coat**: Orange color to match Muni brand (RAL 2004 or similar)
5. **Edge treatment**: All edges deburred, no sharp corners

## DXF Generation

To generate DXF files, use one of:
1. **FreeCAD**: Import this spec, draw geometry, export DXF
2. **LibreCAD**: Direct 2D CAD drawing
3. **OpenSCAD**: Script geometry, export DXF
4. **Online tools**: dxf-creator.com or similar

Example LibreCAD workflow:
1. Create new drawing (mm units)
2. Draw rectangle for panel outline
3. Draw cutout geometries
4. Add mounting holes (circles)
5. Export as DXF (R12 format for compatibility)
