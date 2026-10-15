# BVR2 Power Washer — Design Document

**Date:** 2026-02-22
**Status:** Draft

## 1. Overview

BVR2 is an autonomous sidewalk power washer for the muni platform. Unlike BVR1 (blower-based snow removal), BVR2 uses a chassis-mounted high-pressure nozzle to clean the ground while driving in boustrophedon (lawn-mower) lanes. It carries an onboard 20L water tank and returns to a depot station for automated refills (Roomba-style).

V1 uses a **static-mounted nozzle** — the rover itself is the positioning system. The chassis is designed with mounting provisions for a CoreXY gantry upgrade (V2) if water efficiency proves to be the bottleneck.

**Key specs:**

- 500×500mm chassis, 2020 aluminum extrusion
- 4WD skid-steer: 4× hub motors (same as BVR1), LiDAR SLAM for positioning
- Static-mounted turbo nozzle (V1), CoreXY gantry upgrade path (V2)
- 1000–1500 PSI pressure washer, 0.5 GPM turbo nozzle
- 20L onboard water tank, depot auto-refill
- ~42 kg full / ~22 kg empty
- ~1.9 m² per fill, ~17–19 m² per battery charge

## 2. Nozzle Mount

### V1: Static Mount

Single turbo nozzle fixed to the underside of the chassis, centered. The rover drives in tight boustrophedon lanes to cover the target area.

**Components:**

- 1/4" QC nozzle fitting on aluminum bracket
- 12V solenoid valve for on/off
- ~100mm standoff to ground → ~50mm effective cleaning width
- Bracket bolts to 2020 crossmember

**Lane pattern:**

- 40mm lane spacing (50mm nozzle width, 20% overlap)
- Rover drives at 0.075 m/s during wash
- Coverage rate: **~10 m²/hr**
- Water efficiency: **~10.5 L/m²**

Total mechanical parts: **3** (nozzle, solenoid, bracket). BOM: **~$20**.

### V2 Upgrade: CoreXY Gantry

The chassis includes **pre-drilled mounting holes for 3× MGN12 linear rails** (400mm) on the underside. The gantry upgrade converts BVR2 from continuous lane driving to a park-wash-advance pattern.

**V2 adds:**

- 3× MGN12 linear rails (400mm travel)
- GT2 6mm belt, idler pulleys at corners
- 2× NEMA 17 steppers (frame-mounted)
- TMC2209 stepper drivers with sensorless homing (StallGuard)
- RP2040 MCU on CAN bus
- Cable chain for high-pressure hose
- Lightweight ASA/aluminum carriage
- ~350×350mm working area

**V2 performance:**

- Coverage rate: **~23 m²/hr** (2.3× V1)
- Water efficiency: **~3.1 L/m²** (3.4× V1)
- ~35–40 m² per charge (2× V1)

**V2 cost:** ~$150–200, +2.5 kg, ~20 mechanical parts.

### Why start with static?

| Metric | V1 Static | V2 Gantry |
|--------|-----------|-----------|
| Coverage rate | 10 m²/hr | 23 m²/hr |
| Water efficiency | 10.5 L/m² | 3.1 L/m² |
| Area per charge | ~17–19 m² | ~35–40 m² |
| Refills per 30 m² | ~16 | ~5 |
| Mechanical parts | 3 | ~20+ |
| BOM cost | $20 | $150–200 |
| Added weight | 0.2 kg | 2.5 kg |
| Failure modes | nozzle clog | belt skip, rail jam, stepper stall, hose snag |
| New firmware | solenoid on/off | CoreXY kinematics, raster planner, homing, MCU |

17–19 m² per charge is ~190 linear meters of 0.1m sidewalk strip — enough to validate the real unknowns: dock alignment, splash containment, pump integration, mission planning, and depot refill. The gantry is a known mechanism; these are not. If refill trips prove to be the bottleneck in field testing, the gantry is a bolt-on upgrade.

## 3. Pump & Water System

The pump is cannibalized from a cordless pressure washer — the motor and pump head are reused, powered from the rover's 48V rail via a buck converter matched to the pump's native voltage (~40V).

**Pump:**

- Source: Cordless pressure washer pump module (~40V BLDC)
- Chassis-mounted, ~3.5 kg
- Buck converter from 48V rail to pump voltage
- 1000–1500 PSI at 0.5 GPM
- Controlled via VESC or dedicated ESC on CAN bus

**Tank:**

- 20L rectangular HDPE tank, centered on top deck
- Ultrasonic level sensor for fill monitoring
- Gravity feed to pump inlet (tank above pump)

**Nozzle:**

- 0.5 GPM turbo/rotary nozzle — lower flow rate extends runtime vs standard 1.0+ GPM nozzles
- 1/4" quick-connect fitting
- 12V solenoid valve for instant on/off

**Runtime per fill (V1 static):**

- 20L ÷ 1.89 L/min (0.5 GPM) ≈ 10.6 min of active spray
- At 10.5 L/m², one fill covers **~1.9 m²**

## 4. Depot Refill Dock

The rover autonomously docks at a depot station for water refill when the tank runs low.

**Dock interface:**

- Magnetic quick-connect fitting mounted at rear of rover
- Aligns during standard dock approach (same as BVR1 charge dock)
- Garden hose feed at depot, with booster pump if pressure is insufficient

**Fill control:**

- Float valve or level sensor triggers fill-stop
- Target fill time: ~60 sec (20L at ~0.33 L/sec)
- Depot reports fill status over network

**Optional:** Trickle charge battery during refill dwell time via dock charge contacts.

## 5. Splash Containment

Outdoor use means no water recovery in V1 — runoff is acceptable for sidewalk/concrete surfaces. The skirt contains splash to prevent spraying bystanders or equipment.

**Skirt design:**

- EPDM rubber flap skirt, 3–5mm thick
- Hangs from frame perimeter to ~5–10mm off ground
- Overlapping strips at corners for flexibility over uneven surfaces
- Trailing silicone squeegee lip to reduce water escaping rearward during advance

**Considerations:**

- Skirt must flex enough for ~10mm ground clearance variation
- No vacuum/recovery system in V1 — keeps weight and complexity down
- May add recovery in future revision if needed for indoor or sensitive surfaces

## 6. Drivetrain

**4WD skid-steer** — same drivetrain as BVR1. This was chosen over 2WD differential drive for traction in all-season conditions.

**Why not 2WD differential?**

2WD was considered for its superior positioning precision on flat concrete (no tire scrub). However, BVR2 must handle the same winter conditions as BVR1 — snow, ice, slush on sidewalks. On ice (μ = 0.05–0.15), a 2WD rover at ~42 kg has only ~14–43 N of available traction force from two driven wheels, which is marginal for reliable mobility. 4WD doubles available traction to ~28–86 N against a ~284 N normal force.

**Components:**

- 4× 8" hub motors (350W each, 48V), one per wheel
- 4× VESC 6.6 motor controllers (CAN IDs 0–3)
- Wheelbase/track: ~0.45m × 0.45m (slightly smaller than BVR1's 0.55m to fit 500mm frame)

**Positioning:**

- Tire scrub on concrete accepted as tradeoff for all-season traction
- LiDAR SLAM provides positioning precision for lane tracking (not wheel odometry)
- At low wash speeds (~0.075 m/s), scrub forces are manageable
- Lane spacing accuracy: ±20mm with SLAM, sufficient for 40mm lanes with 50mm nozzle

## 7. Chassis & Weight Budget

500×500mm frame built from 2020 aluminum extrusion. Battery, electronics, and pump on top deck. Tank centered on top for balanced CoG. Underside has pre-drilled MGN12 rail mounting holes for V2 gantry upgrade.

| Component              | V1 Weight | V2 Weight |
|------------------------|-----------|-----------|
| Water (full)           | 20.0 kg   | 20.0 kg   |
| Tank (empty, HDPE)     | 1.0 kg    | 1.0 kg    |
| Frame (2020, 500mm)    | 3.0 kg    | 3.0 kg    |
| Battery (48V 13S4P)    | 4.0 kg    | 4.0 kg    |
| 4× hub motors          | 4.0 kg    | 4.0 kg    |
| Pump module            | 3.5 kg    | 3.5 kg    |
| Nozzle mount / gantry  | 0.2 kg    | 2.5 kg    |
| Electronics            | 2.0 kg    | 2.0 kg    |
| Shell + skirt          | 3.0 kg    | 3.0 kg    |
| Hoses/wiring/misc      | 1.5 kg    | 1.5 kg    |
| **Total (full)**       | **42.2 kg** | **44.5 kg** |
| **Total (empty)**      | **22.2 kg** | **24.5 kg** |

## 8. Electrical / Power

**Battery:** 48V 13S4P (960 Wh) — same pack as BVR1.

**Power consumers (V1 — pump on while driving):**

| Consumer              | Power  |
|-----------------------|--------|
| Pump (wash)           | 400 W  |
| Drive motors (wash)   | 70 W   |
| Compute (Jetson)      | 15 W   |
| Misc (sensors, CAN)   | 10 W   |
| **Total (washing)**   | **495 W** |
| **Total (transit, pump off)** | **~85 W** |

**Runtime estimates (V1):**

- Wash: 960 Wh ÷ 495 W ≈ 1.94 hr active wash time
- Water limits first: 20L per fill at 10.5 L/m² → 1.9 m²/fill
- ~9–10 fill cycles per charge
- Refill overhead: ~10 min total (transit + docking + fill)
- **~17–19 m² per charge**

**V2 comparison:** Pump off during advance drops average power to ~315W, extending battery to ~3.05 hr. Combined with 3.4× better water efficiency → ~35–40 m² per charge.

**Motor control:**

- 4× VESC 6.6 for drive motors (CAN IDs 0–3)
- Pump motor via VESC or dedicated ESC on CAN bus
- V2 adds: gantry steppers driven by RP2040 MCU on CAN bus (consistent with BVR1 tool attachment architecture)

## 9. Operational Flow

### V1: Boustrophedon Lane Wash

```
                    ┌──────────┐
                    │  Start   │
                    └────┬─────┘
                         │
                    ┌────▼─────┐
                    │ Navigate │
                    │ to zone  │
                    └────┬─────┘
                         │
                    ┌────▼─────┐
                    │Open valve │
                    │Drive lane │◄──┐
                    │@ 0.075m/s│   │
                    └────┬─────┘   │
                         │         │
                    ┌────▼─────┐   │
                    │Close valve│   │
                    │Turn + step│   │
                    │ 40mm over │───┘
                    └────┬─────┘
                         │
                  ┌──────▼──────┐
                  │ Tank low?   │──Yes──► Dock → Refill (~60s) → Resume
                  └──────┬──────┘
                         │ No
                  ┌──────▼──────┐
                  │Battery low? │──Yes──► Dock → Charge → Resume/End
                  └──────┬──────┘
                         │ No
                         └──────► Next lane
```

**Step detail:**

1. **Navigate to zone** — SLAM-guided navigation to wash area
2. **Open solenoid, drive lane** — Rover drives a straight lane at 0.075 m/s with pump on
3. **Close solenoid, turn + step** — 180° turn, step over 40mm to next lane
4. **Tank low** — Navigate to depot, dock, refill (~60 sec), resume mission
5. **Battery low** — Dock for charge, resume or end mission

### V2: Park-Wash-Advance (with gantry)

Same outer loop, but instead of continuous lane driving:

1. Drive to patch → stop
2. Home gantry (sensorless stall detection, ~2 sec)
3. Open solenoid → raster nozzle across 350×350mm patch (~12 sec spray)
4. Close solenoid → advance 350mm → repeat

## 10. Software Integration

### V1 — New crates (`bvr/firmware/crates/`)

**`wash`** — Wash cycle state machine (sub-behavior within Autonomous mode)

- Mission type: boustrophedon coverage over a dispatched zone
- Lane tracking: SLAM-guided straight-line driving at low speed
- Solenoid valve control (on during lanes, off during turns)
- Coverage map: track washed lanes for progress reporting
- Abort/pause handling

**`water`** — Tank level, refill dock, flow control

- Ultrasonic level sensor driver
- Refill dock magnetic connector detection
- Fill-stop logic (level threshold or timeout)
- Tank level telemetry publishing

### V1 — Extend existing crates

**`tools`** — Wash pump as CAN-discoverable tool type

- Register pump as tool attachment (like blower in BVR1)
- `ToolCommand.motor` → pump power (0–1.0)
- `ToolCommand.action_a` → solenoid toggle
- Tool discovery and health check over CAN

**`control`** — Low-speed lane driving

- Tight SLAM-guided lane tracking at 0.075 m/s
- 180° turn + 40mm lateral step maneuver
- Boustrophedon path generator from zone polygon

**`config`** — Wash params in `bvr.toml`

```toml
[wash]
enabled = true
pump_can_id = 13
lane_spacing_mm = 40
lane_speed_mps = 0.075
solenoid_gpio = 17
tank_capacity_l = 20.0
low_water_threshold_l = 1.0
```

### V2 — Additional crates (gantry upgrade)

**`gantry`** — CoreXY kinematics, raster path planning, homing

- CoreXY forward/inverse kinematics (motor steps ↔ XY position)
- Serpentine raster path generator with configurable line spacing
- Sensorless homing routine via TMC2209 StallGuard
- Step timing / acceleration profiles
- CAN command interface (move, home, status)

**`wash`** extensions — Park-wash-advance mode

- Switch from boustrophedon lanes to patch-based raster
- Gantry command sequencing (home → raster → done)
- Solenoid sync with gantry motion instead of rover motion

**`config`** additions:

```toml
[gantry]
enabled = false  # V1: false, V2: true
step_per_mm = 80
acceleration = 500
working_area_mm = [350, 350]
line_spacing_mm = 50
raster_speed_mms = 200
```

### Autonomy stack integration

The wash cycle runs as a **mission type within Autonomous mode** — no new top-level states needed. The `wash` crate implements a `WashMission` that sits alongside `DispatchedTask` and produces twist commands (drive during wash, stop during turns) plus tool commands (pump on/off, solenoid on/off).

```
Dispatch → TaskAssignment { zone, task_type: "wash" }
    → WashMission { zone polygon, lane generator }
        → Twist commands (low-speed lane driving)
        → ToolCommand (pump + solenoid)
        → WaterMonitor triggers refill when tank low
```

The control loop doesn't change — it already handles tool commands orthogonal to motion. The wash mission just coordinates them.

### CAD (`bvr/cad/src/parts/`)

New parametric parts:

- `frame_500` — 500mm 2020 extrusion frame (with MGN12 mounting holes)
- `water_tank` — 20L HDPE tank mount
- `pump_mount` — Pump bracket on chassis
- `nozzle_mount` — V1 static nozzle bracket
- `skirt` — EPDM skirt attachment points
- `assembly_bvr2` — Full BVR2 assembly
- V2: `gantry` — CoreXY rail/belt/carriage assembly

### Depot integration

- Water level telemetry → depot dashboard
- Wash coverage map (lanes completed, area washed)
- Refill dock status and fill history

## 11. Open Questions

- **Pump selection:** Exact cordless pump model to cannibalize — Ryobi 40V RY40PW01 is a candidate; need to verify pump head is separable and BLDC motor is controllable via ESC
- **Skirt material:** EPDM strip stock vs custom molded — strip is faster to prototype, molded gives better corner sealing
- **Dock connector:** Magnetic quick-connect vs mechanical latch — magnetic is simpler but must handle water pressure without leaking
- **Water level sensor:** Ultrasonic vs capacitive vs float — ultrasonic is non-contact but may have issues with splashing; capacitive is simpler
- **Surface detection:** Whether to add a camera/sensor to assess surface cleanliness and skip already-clean areas — would improve efficiency but adds complexity
- **Lane turn strategy:** U-turn vs 3-point turn vs spin-in-place for 40mm lane step — depends on turning radius achievable at low speed with skid-steer
- **V2 trigger:** What field metric triggers the gantry upgrade? Candidates: refill trips per mission, water cost, coverage rate vs demand
