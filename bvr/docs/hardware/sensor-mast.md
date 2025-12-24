# Sensor Mast Design

Mounting strategy for perception sensors and antennas on BVR rovers.

## Components

| Component       | Dimensions         | Weight | FOV / Coverage Required                |
| --------------- | ------------------ | ------ | -------------------------------------- |
| Livox Mid-360   | 65 × 65 × 60 mm    | 265g   | 360° horiz × 59° vert (-7° to +52°)    |
| Insta360 X4     | 46 × 123.6 × 37.6  | 203g   | 360° spherical (dual fisheye)          |
| GNSS Antenna    | ~65mm dia × 25mm   | ~100g  | Clear sky view (>10° elevation)        |
| LTE Antennas    | Variable           | ~50g   | Horizontal omnidirectional (to towers) |

## FOV Analysis

### Livox Mid-360

The Mid-360 has asymmetric vertical coverage: -7° below horizontal, +52° above.

```
                        +52°
                         ╱
                        ╱
                       ╱
    ═══════════════════════════════════  ← Horizon (0°)
                       ╲
                        ╲  -7°
                         ╲
                    BLIND ZONE
```

**Critical constraint**: Objects directly below the sensor (within ~7° of vertical) are invisible.
At height `h`, the blind spot radius is `h × tan(7°) ≈ h × 0.12`.

### Insta360 X4

Dual fisheye lenses provide full spherical coverage, but:
- The "stitch zone" at the equator (between lenses) has slightly lower quality
- Any mounting hardware directly adjacent to the lenses will appear in frame
- The rover chassis will be visible in the downward-facing portion

**Goal**: Mount high enough that chassis occupies minimal frame area (ideally <30°).

### GNSS Antenna

Requires unobstructed sky view for satellite reception:
- **Minimum**: Clear view above 15° elevation angle
- **Ideal**: Clear view above 10° elevation angle
- **Ground plane**: Metal surface beneath antenna improves multipath rejection

### LTE Antennas

Cellular antennas need horizontal radiation pattern toward towers:
- Work best mounted **vertically** (omnidirectional in horizontal plane)
- Avoid placing directly next to metal (detuning)
- MIMO configurations benefit from antenna separation (~λ/4 = ~50mm at LTE bands)

## Mounting Options

### Option A: Bucket Combo Antenna (Recommended)

Use a professional combo antenna that houses GNSS + LTE in a single weatherproof unit.

```
           ┌───────────────────┐
           │   Bucket Antenna  │  ← GNSS + 2×LTE (e.g., Taoglas Storm MA111)
           │  ┌─────┐ ┌─────┐  │     ~100mm diameter puck
           │  │GNSS │ │ LTE │  │
           └──┴──┬──┴─┴─────┴──┘
                 │
           ┌─────┴─────┐
           │ Insta360  │  ← 360° camera
           │    X4     │
           └─────┬─────┘
                 │  100mm spacing
           ┌─────┴─────┐
           │  Mid-360  │  ← LiDAR, dome up
           │   LiDAR   │
           └─────┬─────┘
                 │  Mast (18-24")
    ═════════════════════════════
              Chassis
```

**Advantages**:
- Single cable run (GNSS + LTE coax bundle)
- Weatherproof IP67 enclosure
- Professional appearance
- GNSS ground plane integrated
- Optimal antenna placement for both functions

**Disadvantages**:
- Higher cost (~$150-300 vs ~$80 for separate antennas)
- Larger diameter may appear slightly in Insta360 frame

**Recommended Products**:

| Product               | GNSS      | LTE        | Price  | Notes                      |
| --------------------- | --------- | ---------- | ------ | -------------------------- |
| Taoglas Storm MA111   | L1/L2     | 2×MIMO     | ~$180  | High-performance           |
| Panorama LGMM4-7-38   | GPS       | 4×4 MIMO   | ~$250  | Best LTE, overkill GNSS    |
| Mobile Mark MLTM-GNSS | L1/L2     | 2×MIMO     | ~$150  | Good balance               |
| Tallysman HC881       | L1/L2/L5  | 2×MIMO     | ~$200  | Survey-grade GNSS          |

### Option B: Separate Antennas

Split LTE antennas to chassis perimeter, GNSS stays on mast.

```
           ┌─────────────┐
           │ GNSS Puck   │  ← Dedicated multi-band GNSS antenna
           └──────┬──────┘
           ┌──────┴──────┐
           │  Insta360   │
           │     X4      │
           └──────┬──────┘
           ┌──────┴──────┐
           │  Mid-360    │
           │   LiDAR     │
           └──────┬──────┘
                  │  Mast
    ══════════════════════════════
              Chassis
                 │
         ┌───────┴───────┐
        LTE             LTE    ← Whip antennas at chassis corners
      (Main)          (Aux)       Vertical orientation
```

**Advantages**:
- Simpler GNSS antenna (just GPS puck)
- LTE antenna separation improves MIMO diversity
- Keeps mast profile minimal
- Lower cost

**Disadvantages**:
- More cable runs
- LTE antennas exposed to damage
- Less professional appearance
- Chassis corner antennas may obstruct workspace

### Option C: Offset LTE on Mast

Mount LTE antennas on short arms offset from the main mast.

```
                 ┌─────────────┐
                 │ GNSS Puck   │
                 └──────┬──────┘
      ┌────────┐ ┌──────┴──────┐ ┌────────┐
      │LTE Ant │─│  Insta360   │─│LTE Ant │  ← Arms at ~45° to avoid camera FOV
      │ Main   │ │     X4      │ │  Aux   │
      └────────┘ └──────┬──────┘ └────────┘
                 ┌──────┴──────┐
                 │  Mid-360    │
                 └──────┬──────┘
                        │
    ════════════════════════════════════════
                    Chassis
```

**Advantages**:
- Antenna separation for MIMO
- All antennas on single mast assembly
- LTE antennas elevated for better reception

**Disadvantages**:
- LTE antennas appear in Insta360 frame
- More complex mast fabrication
- Wind loading concerns

## Recommended Design: Option A with Bucket Antenna

For BVR, Option A provides the best balance of performance, simplicity, and durability.

### Stacking Order (Bottom to Top)

| Position | Component      | Height from Chassis | Notes                        |
| -------- | -------------- | ------------------- | ---------------------------- |
| 1        | Mast base      | 0mm                 | 1" aluminum tube, chassis    |
| 2        | Mid-360 LiDAR  | 450mm (18")         | Dome up, centered            |
| 3        | Insta360 X4    | 560mm (22")         | 100mm above LiDAR            |
| 4        | Bucket Antenna | 660mm (26")         | 100mm above camera           |

### Total Height

- **Mast above chassis**: ~660mm (26")
- **Typical chassis height**: ~200mm (8")
- **Total from ground**: ~860mm (34")

This keeps the rover profile reasonable while providing:
- Closest visible ground at ~3.7m (acceptable for outdoor operation)
- Minimal chassis visibility in Insta360 (<35° of frame)
- Unobstructed sky view for GNSS
- Elevated LTE for good cellular coverage

### Mast Construction

```
    ┌──────────────────────────────┐
    │     Bucket Antenna Mount     │  ← 5/8"-11 thread (standard antenna mount)
    │     (flat plate + thread)    │
    └─────────────┬────────────────┘
                  │
    ┌─────────────┴────────────────┐
    │   Insta360 Mount Adapter     │  ← 1/4"-20 thread (camera standard)
    │   (45mm extension)           │
    └─────────────┬────────────────┘
                  │
    ┌─────────────┴────────────────┐
    │     Mid-360 Mount Plate      │  ← M3 mounting holes (4×)
    │     (65×65mm, 3mm aluminum)  │
    └─────────────┬────────────────┘
                  │
    ┌─────────────┴────────────────┐
    │        Main Mast Tube        │  ← 1" OD aluminum tube (0.125" wall)
    │        (450mm length)        │
    └─────────────┬────────────────┘
                  │
    ┌─────────────┴────────────────┐
    │      Chassis Mount Base      │  ← 2020 T-slot compatible
    │      (vibration isolated)    │
    └──────────────────────────────┘
```

### Cable Routing

Route cables internally through the mast tube where possible:

| Cable             | From          | To                | Notes                    |
| ----------------- | ------------- | ----------------- | ------------------------ |
| Ethernet (Cat6)   | Mid-360       | Jetson/Switch     | Shielded, internal route |
| USB-C             | Insta360 X4   | Jetson            | External run with clips  |
| GNSS coax (SMA)   | Bucket Ant    | ZED-F9P           | Internal through tube    |
| LTE coax (2×SMA)  | Bucket Ant    | LTE Modem         | Internal through tube    |
| Power (12V)       | Power Dist    | Mid-360           | Internal, 18 AWG pair    |

**Cable entry**: Exit tube through slot at base, use cable gland for weatherproofing.

## FOV Verification

### Mid-360 Coverage Check

At 450mm (18") mast height, the -7° FOV limit means:
- **Blind spot radius**: 450mm × 0.12 = 54mm (2.1")
- **Closest visible ground**: 450mm × 8.1 = 3.7m (12 ft)

For snow clearing on sidewalks (>1.5m wide), this is acceptable. The front tool will always be within the blind spot, but we detect obstacles at approach distance.

### Insta360 Frame Analysis

With camera at 560mm height:
- **Chassis angle**: atan(200mm / 560mm) ≈ 20° below horizontal
- **Chassis visibility**: ~35-40° of downward hemisphere sees rover
- **Bucket antenna angle**: atan(100mm / 0mm) = directly above, small portion

This is acceptable for situational awareness and teleop. For mapping/reconstruction, mask the rover chassis in post-processing.

### Antenna Clearance

The bucket antenna at 660mm provides:
- **Clear sky view**: 360° above ~10° elevation (bucket antennas are low-profile)
- **LTE coverage**: Omnidirectional horizontal pattern, elevated above obstacles
- **GNSS performance**: Similar to standalone survey antenna with integrated ground plane

## Bill of Materials (Mast Assembly)

| Part                     | Qty | Unit  | Total | Source     |
| ------------------------ | --- | ----- | ----- | ---------- |
| Taoglas Storm MA111      | 1   | $180  | $180  | Taoglas    |
| 1" AL tube (0.125" wall) | 1   | $15   | $15   | McMaster   |
| Mid-360 mount plate      | 1   | $10   | $10   | Custom/PCB |
| Insta360 1/4"-20 adapter | 1   | $8    | $8    | Amazon     |
| 5/8"-11 antenna mount    | 1   | $12   | $12   | Amazon     |
| Coax cables (SMA, 1m)    | 3   | $10   | $30   | Amazon     |
| Cable glands (M20)       | 2   | $3    | $6    | Amazon     |
| 2020 mount bracket       | 1   | $8    | $8    | 80/20      |
| M3/M5 hardware kit       | 1   | $10   | $10   | McMaster   |
| **Total**                |     |       | ~$280 |            |

Note: This is for the mast assembly only. Sensors (Mid-360, Insta360, ZED-F9P) listed in [sensors.md](sensors.md).

## Assembly Notes

### Alignment

1. Mount Mid-360 with cable exit pointing toward chassis rear
2. Align Insta360 lens seam with rover forward axis (better stitching on sides)
3. Bucket antenna orientation per manufacturer spec (typically N/S alignment)

### Vibration Isolation

Use rubber standoffs or dampening mounts at the chassis attachment point. The Insta360 has internal stabilization, but external vibration affects:
- LiDAR point cloud noise
- GNSS fix stability (less critical)

### Quick Release (Optional)

Consider a quick-release mechanism at the mast base for:
- Transport (mast exceeds vehicle doorways)
- Service access
- Tool changes that require mast removal

Options:
- RAM ball mount (heavy duty)
- Custom dovetail + thumb screw
- QD pin through collar

## Future Considerations (bvr1)

For the production rover, consider:

| Improvement                 | Benefit                          | Cost Impact |
| --------------------------- | -------------------------------- | ----------- |
| Integrated mast casting     | Cleaner design, less assembly    | +$200       |
| Internal cable pass-through | Full weather sealing             | +$50        |
| Active gimbal for camera    | Stabilization, look-ahead        | +$400       |
| Additional forward camera   | Better depth for teleop          | +$300       |

