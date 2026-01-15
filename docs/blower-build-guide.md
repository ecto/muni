# Blower System Assembly Guide

**Purpose:** Debris-clearing tool for autonomous rover demonstration
**Timeline:** Week 4 (2 days assembly + testing)
**Skill Level Required:** Basic mechanical assembly, soldering, electrical connections
**Estimated Total Time:** 8-10 hours (excluding 3D printing time)

---

## Table of Contents

1. [Pre-Assembly Checklist](#pre-assembly-checklist)
2. [Mechanical Assembly (Day 1)](#mechanical-assembly-day-1)
3. [Shell Integration (Day 2 Morning)](#shell-integration-day-2-morning)
4. [Electrical Integration (Day 2 Afternoon)](#electrical-integration-day-2-afternoon)
5. [Bench Testing](#bench-testing)
6. [Debris Testing](#debris-testing)
7. [Troubleshooting](#troubleshooting)

---

## Pre-Assembly Checklist

### Parts List

**Electronics:**
- [ ] 90mm EDF unit (motor + impeller, pre-balanced)
- [ ] VESC 75/300 motor controller with CAN capability
- [ ] Conformal coating (MG Chemicals 422B or equivalent)
- [ ] Heat shrink tubing assortment
- [ ] Silicone wire: 14 AWG (1m red, 1m black)
- [ ] XT60 or XT90 connector pair (power connection)

**Mechanical:**
- [ ] 3D printed volute housing (ASA filament)
- [ ] 3D printed expansion nozzle (ASA filament)
- [ ] Aluminum shell from SendCutSend (powder-coated orange)
- [ ] Intake vent baffles (3D printed or machined)
- [ ] Drain grommets (6mm rubber, McMaster)

**Mounting Hardware:**
- [ ] M5×10mm button head screws (20x)
- [ ] M5×16mm button head screws (10x)
- [ ] M5 T-nuts for 2020 extrusion (20x)
- [ ] M4×8mm screws for motor mount (4x)
- [ ] M3×8mm screws for VESC mount (4x)
- [ ] Threadlocker (Loctite 243 blue)

**Consumables:**
- [ ] Isopropyl alcohol (cleaning)
- [ ] Shop towels
- [ ] Electrical tape
- [ ] Zip ties (assorted sizes)
- [ ] Cable lacing cord (optional, for clean routing)

### Tools Required

**Hand Tools:**
- [ ] Hex key set (1.5mm - 5mm)
- [ ] Screwdriver set (Phillips, flathead)
- [ ] Wire cutters/strippers
- [ ] Needle nose pliers
- [ ] Adjustable wrench
- [ ] Deburring tool or utility knife

**Power Tools:**
- [ ] Drill with bits (2.5mm, 3mm, 4mm)
- [ ] Soldering iron (60W recommended)
- [ ] Heat gun or lighter (for heat shrink)

**Test Equipment:**
- [ ] Multimeter
- [ ] Power supply or battery (48V, 10A minimum)
- [ ] Tachometer or laser tachometer (optional)
- [ ] Clamp meter (for current measurement)

**Safety Equipment:**
- [ ] Safety glasses
- [ ] Work gloves
- [ ] Hearing protection (for testing)
- [ ] Ventilation (for soldering/coating)

### Pre-Fabrication Requirements

Before starting assembly, ensure these items are ready:

1. **3D Printing Complete:**
   - Volute housing (print time: 20-30 hours)
   - Expansion nozzle sections (print time: 10-15 hours)
   - Intake vent baffles (print time: 2-4 hours)
   - All parts inspected, no warping or layer separation
   - Support material removed and cleaned up

2. **Shell Fabrication:**
   - Aluminum shell panels cut and bent by SendCutSend
   - Powder coating applied (orange, Muni standard)
   - Blower cutout integrated into shell design
   - Mounting holes for chassis attachment pre-drilled

3. **Workspace Setup:**
   - Clean, well-lit work area
   - ESD-safe mat (for electronics)
   - Parts organized in bins/trays
   - Access to compressed air (for cleaning)

---

## Mechanical Assembly (Day 1)

**Time Required:** 3-4 hours
**Goal:** Complete blower assembly ready for shell integration

### Step 1: Inspect Components (15 minutes)

1. **Unpack EDF unit carefully:**
   - Check motor shaft for damage
   - Verify impeller blades are intact and balanced
   - Inspect motor wires for fraying
   - Confirm mounting holes align with volute housing

2. **Inspect 3D printed parts:**
   - Check for warping (especially on large flat surfaces)
   - Remove any support material residue
   - Test-fit parts together before assembly
   - Sand any rough edges that could cause air leaks

3. **Verify hardware:**
   - Count all screws and T-nuts
   - Check for correct lengths
   - Ensure Loctite is not expired (shelf life ~2 years)

> **Photo Placeholder:** Laid-out components on workbench

### Step 2: Mount Motor to Volute Housing (30 minutes)

The volute housing is the heart of the blower system. Proper mounting ensures good airflow and minimal vibration.

1. **Prepare mounting surface:**
   - Clean volute motor mount face with isopropyl alcohol
   - Remove any print artifacts around screw holes
   - Check that motor mounting face is flat

2. **Apply Loctite to motor screws:**
   - Use **blue Loctite 243** only (removable)
   - Apply one small drop to each M4 screw thread
   - Do NOT use red Loctite (permanent)

3. **Align motor to volute:**
   - Motor output shaft should be centered in volute inlet
   - Motor wires should exit toward the rear (away from nozzle)
   - Use alignment marks on 3D print if present

4. **Torque screws in star pattern:**
   - Tighten opposite screws alternately
   - Final torque: snug + 1/4 turn (approximately 2-3 Nm)
   - Do NOT overtighten into plastic

5. **Verify alignment:**
   - Motor shaft should spin freely without rubbing
   - No visible wobble when spun by hand
   - Air gap around shaft should be even (±0.5mm)

> **Photo Placeholder:** Motor mounted to volute, showing alignment

### Step 3: Install Impeller (45 minutes)

**WARNING:** Impeller installation is critical for safety. An improperly secured impeller can cause catastrophic failure at high RPM.

1. **Inspect impeller and shaft:**
   - Clean both with isopropyl alcohol
   - Check for burrs or damage
   - Verify set screw hole is clean and threaded

2. **Slide impeller onto motor shaft:**
   - Align keyway if present
   - Push impeller fully onto shaft until it seats
   - Impeller should NOT contact volute housing walls

3. **Position impeller for optimal clearance:**
   - Measure gap between impeller and volute inlet (~2-3mm ideal)
   - Adjust position on shaft if needed
   - Mark shaft position with marker

4. **Apply Loctite to set screw:**
   - Use **blue Loctite 243**
   - Apply to set screw threads
   - Insert set screw into impeller hub

5. **Tighten set screw:**
   - Tighten until it contacts shaft flat (if present)
   - Final torque: snug + 1/8 turn
   - Do NOT overtighten (can strip aluminum hub)

6. **Balance check (critical):**
   - Spin impeller by hand (should coast smoothly)
   - No wobble or vibration when spinning
   - If vibration detected, loosen and re-align

7. **Let Loctite cure:**
   - Wait 10 minutes minimum before proceeding
   - Full cure: 24 hours (do not run at full power before then)

> **Photo Placeholder:** Impeller installed with proper clearance visible

### Step 4: Attach Expansion Nozzle (30 minutes)

The expansion nozzle transitions from the circular volute outlet to a wide slot for maximum coverage.

1. **Dry-fit nozzle to volute:**
   - Align mounting holes
   - Check that transition is smooth (no sharp steps)
   - Mark any areas that need trimming/sanding

2. **Sand mating surfaces if needed:**
   - Goal: airtight seal at junction
   - Use 220-grit sandpaper on high spots
   - Clean with compressed air

3. **Apply sealant (optional but recommended):**
   - Use silicone RTV sealant at nozzle-volute junction
   - Thin bead around perimeter
   - Prevents air leaks that reduce efficiency

4. **Bolt nozzle to volute:**
   - Use M5×10mm screws
   - Start all screws before tightening any
   - Tighten in star pattern
   - Final torque: snug (do not overtighten into plastic)

5. **Wipe excess sealant:**
   - Remove any sealant squeeze-out inside nozzle
   - Smooth with finger or tool
   - Let cure per manufacturer instructions (usually 24 hours)

6. **Measure nozzle outlet dimensions:**
   - Should be approximately 500mm wide × 50mm tall
   - Verify no obstructions in airflow path

> **Photo Placeholder:** Nozzle attached, showing smooth transition

### Step 5: Test-Fit in Shell (45 minutes)

Before final integration, ensure the blower assembly fits properly in the shell.

1. **Position shell on workbench:**
   - Place shell upside-down or on side for access
   - Ensure it's stable and won't roll

2. **Insert blower assembly into shell cutout:**
   - Nozzle should extend through front opening
   - Motor should be accessible from rear
   - Volute should sit flush against shell interior

3. **Check clearances:**
   - Minimum 10mm clearance to shell walls (vibration isolation)
   - No interference with chassis mounting brackets
   - Intake area has clear path to top vents

4. **Mark mounting points:**
   - Use blower mounting holes as template
   - Transfer marks to shell interior
   - Double-check measurements before drilling

5. **Drill mounting holes in shell:**
   - Use 5.5mm drill bit for M5 bolts
   - Deburr all holes
   - Clean metal shavings with compressed air

6. **Remove blower from shell:**
   - Set aside for painting/finishing
   - Store in safe location (protect nozzle from damage)

> **Photo Placeholder:** Blower test-fit in shell with clearances visible

### Step 6: Paint/Finish (Optional, 1-2 hours + dry time)

If 3D printed parts are not pre-finished:

1. **Surface preparation:**
   - Sand parts with 220-grit sandpaper
   - Fill layer lines with filler primer (optional)
   - Clean with isopropyl alcohol

2. **Primer (if painting):**
   - Use plastic-safe primer
   - 2-3 light coats, 10 minutes between coats
   - Let dry 1 hour

3. **Paint:**
   - Use high-temperature paint (optional, for motor area)
   - Match Muni orange if desired
   - 2-3 coats, let dry per manufacturer instructions

4. **Clear coat (optional):**
   - UV-resistant clear coat for outdoor durability
   - 2 coats
   - Let cure 24 hours before handling

**Alternative:** Leave ASA parts unpainted (ASA is UV-resistant and durable as-is)

---

## Shell Integration (Day 2 Morning)

**Time Required:** 2-3 hours
**Goal:** Blower mechanically integrated into shell and mounted to chassis

### Step 7: Mount Blower Assembly to Shell (45 minutes)

1. **Position blower in shell:**
   - Align mounting holes
   - Ensure nozzle is centered in shell opening
   - Check that motor is accessible for wiring

2. **Install vibration dampers (recommended):**
   - Use rubber washers or grommets under bolt heads
   - Helps reduce noise and vibration transmission
   - McMaster P/N: 90295A120 (M5 size)

3. **Bolt blower to shell:**
   - Use M5×16mm screws through shell into volute mounting bosses
   - Add washers under bolt heads
   - Apply blue Loctite to threads
   - Torque to snug + 1/4 turn

4. **Verify secure mounting:**
   - Try to wiggle blower assembly (should be solid)
   - Check all screws are tight
   - No rattling or loose parts

> **Photo Placeholder:** Blower mounted inside shell

### Step 8: Secure Shell to Chassis (1 hour)

The shell mounts to the rover's 2020 aluminum extrusion chassis.

1. **Position shell on rover chassis:**
   - Align with front of rover (nozzle pointing forward)
   - Center left-to-right
   - Check ground clearance (shell should not drag)

2. **Install T-nuts in chassis extrusion:**
   - Insert T-nuts into 2020 extrusion slots
   - Loosely position at mounting points
   - Align with shell mounting holes

3. **Install mounting brackets:**
   - Use custom brackets or L-brackets
   - Attach brackets to shell with M5 screws
   - Attach brackets to chassis with M5 screws into T-nuts

4. **Tighten all mounting hardware:**
   - Chassis-to-bracket: 5 Nm torque
   - Bracket-to-shell: snug + 1/4 turn
   - Apply blue Loctite to critical fasteners

5. **Check rigidity:**
   - Push down on shell (should not flex)
   - No movement relative to chassis
   - All mounting points load-bearing

> **Photo Placeholder:** Shell mounted to chassis with visible brackets

### Step 9: Install Intake Vent Baffles (30 minutes)

Intake vents allow air to enter the blower while preventing debris from entering.

1. **Locate intake vent positions:**
   - Top rear of shell
   - Should have clear path to volute intake
   - Multiple smaller vents preferred over one large vent

2. **Install vent baffles:**
   - Baffles prevent rain from entering
   - Use louver-style or mesh-covered openings
   - Secure with M3 screws or adhesive

3. **Verify airflow path:**
   - Air should flow: vents → volute → impeller → nozzle
   - No obstructions or sharp bends
   - Use smoke pencil or tissue test to verify flow direction

> **Photo Placeholder:** Intake vents installed on shell top

### Step 10: Install Drain Grommets (15 minutes)

Drain grommets allow water to escape if rain enters the shell.

1. **Locate low points in shell:**
   - Water will pool at lowest interior points
   - Typically bottom rear corners

2. **Drill 6mm holes for grommets:**
   - Deburr holes
   - Clean metal shavings

3. **Install rubber grommets:**
   - Push into holes from interior
   - Should seat flush
   - Allows water to drain but prevents debris from entering

---

## Electrical Integration (Day 2 Afternoon)

**Time Required:** 2-3 hours
**Goal:** Blower powered and controllable via CAN bus

### Step 11: Route 48V Power from Battery (45 minutes)

**SAFETY:** Work with rover powered OFF. Verify with multimeter before touching any wires.

1. **Identify battery power tap point:**
   - Locate main 48V battery distribution point
   - Ensure fuse or circuit breaker is available
   - Check wire gauge (14 AWG minimum for 10A load)

2. **Cut and strip power wires:**
   - Cut silicone wire to length (measure carefully)
   - Strip 5mm of insulation from each end
   - Twist strands tightly

3. **Install inline fuse (recommended):**
   - Use 15A blade fuse or equivalent
   - Protects blower circuit from overcurrent
   - Mount fuse holder in accessible location

4. **Route wires to blower location:**
   - Follow existing wire harnesses
   - Avoid sharp edges (use grommets where passing through metal)
   - Leave 300mm extra length at blower end (for service)
   - Secure with zip ties every 150mm

5. **Label wires:**
   - Use heat shrink labels or tape
   - Mark: "48V BLOWER +" and "48V BLOWER -"
   - Critical for safety during service

> **Photo Placeholder:** Power wires routed along chassis

### Step 12: Connect VESC Motor Controller (1 hour)

The VESC controls blower speed via CAN commands from the rover's main computer.

1. **Mount VESC near blower:**
   - Use M3 screws to mount to chassis or shell interior
   - Ensure adequate ventilation around VESC
   - Avoid locations where water could drip on it

2. **Connect battery power to VESC:**
   - Solder XT60/XT90 connector to 48V wires
   - Connect to VESC power input
   - Observe polarity (red=+, black=-)
   - **DOUBLE CHECK POLARITY BEFORE POWERING ON**

3. **Connect VESC to blower motor:**
   - Motor has 3 phase wires (typically yellow, blue, black)
   - Phase order determines direction (doesn't matter for blower)
   - Solder motor wires to VESC motor output
   - Use 14 AWG wire or heavier
   - Heat shrink all connections

4. **Connect CAN bus:**
   - VESC has CAN-H and CAN-L terminals
   - Daisy-chain from existing VESC CAN bus
   - Use twisted pair wire (22 AWG)
   - Observe correct polarity (CAN-H to CAN-H, CAN-L to CAN-L)

5. **Configure VESC CAN address:**
   - Use VESC Tool software via USB
   - Set unique CAN ID (suggest: 10 for blower)
   - Configure motor parameters:
     - Motor type: BLDC
     - Battery voltage: 48V
     - Battery cutoff: 42V
     - Max motor current: 40A
     - Max battery current: 15A
   - Save configuration to VESC

6. **Verify VESC configuration:**
   - Check motor detection was successful
   - Test motor spins in VESC Tool (low duty cycle)
   - No error codes displayed

> **Photo Placeholder:** VESC wired to blower motor with CAN bus

### Step 13: Apply Conformal Coating (30 minutes)

Conformal coating protects electrical connections from moisture and corrosion.

**IMPORTANT:** Work in well-ventilated area. Coating is flammable and has strong fumes.

1. **Prepare surfaces:**
   - Clean all connections with isopropyl alcohol
   - Let dry completely (5 minutes)
   - Mask off areas that shouldn't be coated (connectors, vents)

2. **Apply conformal coating:**
   - Use MG Chemicals 422B or equivalent
   - Brush or spray onto:
     - Solder joints
     - Wire terminations
     - VESC PCB (avoid connectors)
   - Apply thin, even coat (2-3 coats better than one thick coat)

3. **Cure coating:**
   - Let dry per manufacturer instructions (typically 1-4 hours)
   - Full cure: 24 hours
   - Do not operate blower until coating is dry

> **Photo Placeholder:** Coated connections (before final cure)

### Step 14: Cable Management (30 minutes)

Proper cable management prevents chafing, interference, and maintenance issues.

1. **Bundle power wires:**
   - Use zip ties every 150mm along wire runs
   - Avoid over-tightening (can damage insulation)
   - Leave service loops at connections

2. **Secure CAN bus wires:**
   - Keep CAN wires away from high-current power wires (EMI)
   - Use cable lacing or spiral wrap
   - Protect from abrasion at any sharp edges

3. **Final inspection:**
   - Tug test all connections (should not pull free)
   - Check for any exposed wire
   - Verify no wires are pinched or under tension
   - Ensure all wires are secured (nothing hanging loose)

4. **Documentation:**
   - Take photos of wiring for reference
   - Note VESC CAN ID in system documentation
   - Update rover wiring diagram if applicable

> **Photo Placeholder:** Clean cable routing with labeled wires

### Step 15: Wiring Diagram

```
                                    ┌───────────────────────────────┐
                                    │  Rover Main Battery (48V)     │
                                    │  13S Li-ion, 20Ah             │
                                    └────────┬──────────────────────┘
                                             │
                                       [15A Fuse]
                                             │
                                    ┌────────┴────────┐
                                    │  14 AWG Power   │
                       Red (+48V) ──┤                 │
                       Black (GND) ─┤                 │
                                    └────────┬────────┘
                                             │
                                             │ XT60 Connector
                                             │
                        ┌────────────────────┴─────────────────────┐
                        │         VESC 75/300                      │
                        │         CAN ID: 10                       │
                        │                                          │
                        │  Power In:  +48V, GND                   │
                        │  Motor Out: Phase A, B, C (3 wires)     │
                        │  CAN Bus:   CAN-H, CAN-L (twisted pair) │
                        └───────┬────────────────┬─────────────────┘
                                │                │
                        Motor Phase Wires        │ CAN Bus (22 AWG twisted pair)
                                │                │
                                │                │ CAN-H (Yellow)
                    ┌───────────┴──────┐         │ CAN-L (Green)
                    │  90mm EDF Motor  │         │
                    │  Brushless, 48V  │         └─────> To other VESCs (daisy chain)
                    │  Max 40A         │                 Termination: 120Ω at each end
                    └──────────────────┘

Notes:
- All power connections use 14 AWG silicone wire minimum
- CAN bus uses 22 AWG twisted pair
- Inline 15A fuse protects blower circuit
- VESC CAN ID must be unique (suggest ID 10)
- Phase wire order doesn't matter (determines spin direction only)
```

---

## Bench Testing

**Time Required:** 1-2 hours
**Goal:** Verify blower operates safely at all power levels before field deployment

**SAFETY FIRST:**
- Wear safety glasses at all times
- Wear hearing protection above 50% power
- Keep hands, tools, and loose clothing away from impeller
- Secure rover to prevent movement during testing
- Have emergency stop readily accessible

### Test 1: Power-On Checklist (15 minutes)

Before first power-on, verify all critical items:

- [ ] **Electrical:**
  - [ ] Battery voltage measures 48V ± 2V
  - [ ] All polarities correct (no reversed connections)
  - [ ] No short circuits (multimeter test)
  - [ ] Fuse installed in blower circuit
  - [ ] VESC power LED illuminates when powered
  - [ ] CAN bus terminators in place (120Ω at each end)

- [ ] **Mechanical:**
  - [ ] Impeller spins freely by hand
  - [ ] No obstructions in airflow path
  - [ ] All mounting bolts tight
  - [ ] No loose wires near impeller
  - [ ] Intake vents clear

- [ ] **Software:**
  - [ ] Rover `bvrd` daemon running
  - [ ] VESC detected on CAN bus (check logs)
  - [ ] Blower control available in depot console
  - [ ] E-stop functional

### Test 2: Spin-Up Test (30 minutes)

Gradually increase blower speed while monitoring for issues.

1. **Initial power-on (0% → 10%):**
   - Set blower to 10% power via depot console
   - Impeller should start spinning smoothly
   - Listen for unusual noises (grinding, squealing, rattling)
   - Verify rotation direction (doesn't matter, but should be consistent)

2. **Low power test (10% → 30%):**
   - Increase to 30% in 5% increments
   - Hold each level for 30 seconds
   - Monitor:
     - Vibration (should be minimal)
     - Sound (should be smooth hum, no rattling)
     - Airflow (should feel at nozzle outlet)

3. **Medium power test (30% → 60%):**
   - Increase to 60% in 10% increments
   - Hold each level for 30 seconds
   - Monitor same parameters as above
   - Airflow should be strong and consistent

4. **High power test (60% → 100%):**
   - Increase to 100% in 10% increments
   - Hold at 100% for 60 seconds
   - **Expected behavior:**
     - Strong, steady airflow
     - Smooth sound (turbine-like)
     - Minimal vibration
   - **Unacceptable behavior (STOP IMMEDIATELY):**
     - Loud rattling or grinding
     - Excessive vibration
     - Burning smell
     - VESC error lights

5. **Shutdown test:**
   - Reduce power to 0%
   - Impeller should coast down smoothly (15-30 seconds)
   - No sudden stops or grinding

> **Photo Placeholder:** Blower running during bench test

### Test 3: Current Draw Measurement (15 minutes)

Measure electrical current to verify system is within specifications.

1. **Setup clamp meter:**
   - Clamp around positive power wire to VESC
   - Set meter to DC current mode
   - Zero meter if applicable

2. **Measure current at various power levels:**

   | Power Level | Expected Current | Actual Current | Notes |
   |-------------|------------------|----------------|-------|
   | 10%         | ~1-2A           |                |       |
   | 30%         | ~3-5A           |                |       |
   | 50%         | ~6-10A          |                |       |
   | 80%         | ~12-18A         |                |       |
   | 100%        | ~15-25A         |                |       |

3. **Verify current is within limits:**
   - Should not exceed VESC battery current limit (15A typical)
   - Should not trip inline fuse (15A)
   - If current is too high, reduce max power in VESC settings

4. **Calculate power consumption:**
   - Power (W) = Voltage (V) × Current (A)
   - At 100%: ~48V × 20A = ~960W maximum
   - Battery runtime: 20Ah ÷ 20A = ~1 hour at 100% (theoretical)

### Test 4: Vibration Check (15 minutes)

Excessive vibration can indicate imbalance, misalignment, or mounting issues.

1. **Visual inspection at operating speed:**
   - Run blower at 50% power
   - Observe impeller through intake (if visible)
   - Should spin true with no wobble

2. **Hand vibration test:**
   - Lightly touch shell exterior near blower
   - Should feel smooth vibration, no harsh buzzing
   - Compare to motor VESCs (blower should be similar or less)

3. **Listening test:**
   - Stand 1m from blower
   - Listen for:
     - Smooth turbine sound: GOOD
     - Rhythmic thumping: BAD (unbalanced impeller)
     - High-pitched squeal: BAD (bearing issue)
     - Rattling: BAD (loose parts)

4. **If excessive vibration detected:**
   - Power down immediately
   - Check impeller set screw tightness
   - Verify impeller is not damaged
   - Check motor mounting bolts
   - Re-test after corrections

### Test 5: Thermal Test (30 minutes)

Verify system does not overheat during extended operation.

1. **Baseline temperature measurement:**
   - Measure ambient air temperature
   - Measure VESC heatsink temperature (should be ~ambient)
   - Measure motor housing temperature (should be ~ambient)

2. **Run at 80% power for 15 minutes:**
   - Simulates sustained demo operation
   - Monitor temperatures every 5 minutes:

   | Time | VESC Temp | Motor Temp | Notes |
   |------|-----------|------------|-------|
   | 0min | ___°C     | ___°C      |       |
   | 5min | ___°C     | ___°C      |       |
   | 10min| ___°C     | ___°C      |       |
   | 15min| ___°C     | ___°C      |       |

3. **Acceptable temperature limits:**
   - VESC heatsink: <70°C (140°F)
   - Motor housing: <80°C (176°F)
   - If temps exceed limits, increase cooling or reduce max power

4. **Cool-down test:**
   - Reduce to 0% power
   - Monitor temperature decrease
   - Should return to near-ambient within 10 minutes

5. **Long-term thermal considerations:**
   - If running >30 min continuously, consider adding cooling fan
   - Ensure intake vents are not blocked during operation
   - Monitor temps during debris testing (additional load)

---

## Debris Testing

**Time Required:** 1-2 hours
**Goal:** Determine optimal power settings and effective range for debris clearing

**Test Objects:**
- [ ] Paper (single sheet, 8.5×11")
- [ ] Paper cup (empty, 16oz)
- [ ] Plastic bottle (empty, 500ml)
- [ ] Leaves (if available)
- [ ] Tennis ball (worst-case heavy object)

### Test 6: Effective Range at Various Power Levels (45 minutes)

Determine at what distance the blower can move various objects.

**Setup:**
1. Mark distances on floor: 2ft, 5ft, 8ft, 10ft, 15ft
2. Place test object at each distance
3. Run blower at various power levels
4. Record if object moves

**Test Matrix:**

| Object | Power Level | 2ft | 5ft | 8ft | 10ft | 15ft | Notes |
|--------|-------------|-----|-----|-----|------|------|-------|
| Paper  | 30%         |     |     |     |      |      |       |
| Paper  | 50%         |     |     |     |      |      |       |
| Paper  | 80%         |     |     |     |      |      |       |
| Cup    | 30%         |     |     |     |      |      |       |
| Cup    | 50%         |     |     |     |      |      |       |
| Cup    | 80%         |     |     |     |      |      |       |
| Bottle | 50%         |     |     |     |      |      |       |
| Bottle | 80%         |     |     |     |      |      |       |
| Bottle | 100%        |     |     |     |      |      |       |

**Expected Results:**
- Paper should move at 8-10ft with 50% power
- Cup should move at 5-8ft with 50% power
- Bottle should move at 3-5ft with 80% power

### Test 7: Demo Power Level Optimization (30 minutes)

Find the "sweet spot" for live demonstrations.

**Goals:**
- Impressive but not dangerous
- Reliable object movement
- Reasonable battery consumption
- Not excessively loud

**Recommended Settings:**
- **Cruise power:** 40-50% (background operation during patrol)
- **Demo power:** 80-90% (when approaching object)
- **Max power:** 100% (reserved for heavy objects or wow factor)

**Demo Choreography Practice:**
1. Rover approaches object at 3 mph, blower at 50%
2. At 8 feet: Operator sees object, maintains 50%
3. At 5 feet: Object starts to move from airflow
4. At 3 feet: Operator boosts to 80% via console
5. Object tumbles aside dramatically
6. Rover passes, operator returns to 50%

**Practice this sequence 10 times with various objects to build operator confidence.**

### Test 8: Safety Checks (15 minutes)

Verify blower does not pose safety risks during demo.

1. **Max airflow test:**
   - Person stands in front of blower at 5ft
   - Run blower at 100%
   - Airflow should be strong but not painful
   - No flying debris created

2. **Noise level test:**
   - Measure sound level at 3ft with smartphone app
   - Target: <85 dB at 80% power (safe for 8hr exposure)
   - If too loud, add sound dampening or reduce max power

3. **Emergency stop test:**
   - Run blower at 100%
   - Trigger emergency stop
   - Blower should shut down within 1 second
   - Verify impeller coasts safely (no sudden stop)

4. **Fail-safe test:**
   - Disconnect CAN bus while blower running
   - Blower should shut down or go to safe state
   - Verify behavior matches firmware expectations

---

## Troubleshooting

### Common Issues and Fixes

#### Issue: Blower won't start / no response to commands

**Symptoms:**
- VESC powered on (LED lit) but motor doesn't spin
- No response when changing power level in console

**Diagnosis:**
1. Check VESC CAN ID matches firmware expectations
   - Use VESC Tool to verify CAN ID (should be 10)
   - Check `bvrd` logs for CAN communication errors
2. Verify CAN bus wiring polarity
   - CAN-H to CAN-H, CAN-L to CAN-L
   - Swap wires if necessary and re-test
3. Check VESC motor detection
   - Re-run motor detection in VESC Tool
   - Verify motor parameters are saved

**Fix:**
- Correct CAN ID in VESC Tool, save configuration
- Fix CAN bus wiring if reversed
- Re-run motor detection and save parameters

---

#### Issue: Excessive vibration or noise

**Symptoms:**
- Blower rattles or vibrates harshly
- Rhythmic thumping sound
- Vibration felt throughout rover chassis

**Diagnosis:**
1. Check impeller set screw
   - Remove volute cover if needed
   - Verify set screw is tight
   - Check for shaft flat engagement
2. Inspect impeller for damage
   - Look for broken blades
   - Check for cracks or deformation
3. Check motor mounting bolts
   - Verify all bolts are tight
   - Check for cracks in volute housing

**Fix:**
- Tighten impeller set screw with Loctite
- Replace impeller if damaged (use spare)
- Tighten motor mounting bolts
- If vibration persists, motor may be damaged (replace)

---

#### Issue: Low airflow / weak performance

**Symptoms:**
- Blower runs but airflow is weak
- Cannot move objects that should be within range
- Motor spins but little air output

**Diagnosis:**
1. Check for obstructions
   - Inspect intake vents (blocked?)
   - Check volute interior (debris?)
   - Verify nozzle is clear
2. Check impeller direction
   - Should blow OUT of nozzle, not suck IN
   - If reversed, swap any two motor phase wires
3. Check for air leaks
   - Inspect volute-to-nozzle seal
   - Check for cracks in housing

**Fix:**
- Clear any obstructions
- Reverse motor direction if needed (swap two phase wires)
- Re-seal volute-to-nozzle junction with RTV sealant
- Replace housing if cracked

---

#### Issue: VESC overheating

**Symptoms:**
- VESC very hot to touch (>70°C)
- VESC thermal protection shuts down motor
- VESC error LED flashing

**Diagnosis:**
1. Check current draw
   - Measure with clamp meter
   - Should be <15A at 100% power
2. Check VESC mounting
   - Should have thermal contact to metal heatsink
   - Ensure adequate airflow around VESC
3. Check motor parameters
   - Max current settings may be too high

**Fix:**
- Reduce max battery current in VESC settings (try 12A)
- Add heatsink or cooling fan to VESC
- Improve airflow around VESC mounting location
- Limit max power to 80% if problem persists

---

#### Issue: Blower works intermittently

**Symptoms:**
- Blower starts and stops randomly
- Works sometimes, fails other times
- CAN communication errors in logs

**Diagnosis:**
1. Check power connections
   - Verify XT60 connector is fully seated
   - Check for loose solder joints
   - Measure voltage at VESC (should be 48V ± 2V)
2. Check CAN bus connections
   - Verify twisted pair is intact
   - Check for damaged wire insulation
   - Ensure termination resistors in place
3. Check for shorts or ground loops
   - Use multimeter to check resistance to ground

**Fix:**
- Re-solder any cold joints
- Replace damaged CAN bus wires
- Ensure proper termination (120Ω at each end of bus)
- Add ferrite beads to CAN wires if EMI suspected

---

#### Issue: Current draw too high / fuse blows

**Symptoms:**
- Inline fuse blows when blower at high power
- Current exceeds 15A (typical limit)
- Battery protection trips

**Diagnosis:**
1. Check motor resistance
   - Measure phase-to-phase resistance (should be ~0.1-0.5Ω)
   - Low resistance = high current
2. Check for mechanical binding
   - Spin impeller by hand (should be smooth)
   - Check for rubbing against volute
3. Check VESC current limits
   - Verify battery current limit is set correctly

**Fix:**
- Reduce max duty cycle in VESC settings
- Check for mechanical issues (binding, rubbing)
- Use higher-current fuse (20A) if motor and wiring can handle it
- Limit max power to 80% in firmware

---

### Spare Parts to Have On Hand

Keep these spares available during demo days:

**Critical (have at demo):**
- [ ] Spare impeller (pre-balanced)
- [ ] Spare VESC (pre-configured with correct CAN ID)
- [ ] Inline fuse (15A, same type as installed)
- [ ] XT60/XT90 connectors (2x pairs)
- [ ] Assorted M5 and M4 bolts
- [ ] Blue Loctite
- [ ] Zip ties (assorted sizes)
- [ ] Electrical tape
- [ ] Multimeter

**Nice to Have (keep in shop):**
- [ ] Complete 3D printed volute housing
- [ ] Complete 3D printed nozzle
- [ ] 90mm EDF motor (complete unit)
- [ ] 1m each 14 AWG red/black wire
- [ ] Heat shrink tubing
- [ ] Conformal coating (small bottle)
- [ ] Rubber vibration dampers

---

## Final Checklist

Before declaring blower integration complete:

**Mechanical:**
- [ ] All bolts torqued and Loctite applied
- [ ] No rattles or vibrations at any power level
- [ ] Impeller spins freely, no rubbing
- [ ] Shell securely mounted to chassis
- [ ] Intake vents clear and baffled
- [ ] Drain grommets installed

**Electrical:**
- [ ] All connections soldered and heat-shrunk
- [ ] Conformal coating applied and cured
- [ ] VESC configured with correct CAN ID
- [ ] Inline fuse installed
- [ ] Wires secured and labeled
- [ ] No exposed wire or short circuits

**Testing:**
- [ ] Spin-up test passed (0-100% smooth)
- [ ] Current draw within limits (<15A at 100%)
- [ ] Vibration check passed
- [ ] Thermal test passed (15min at 80%, <70°C)
- [ ] Debris test completed, optimal power determined
- [ ] Emergency stop functional
- [ ] Demo choreography practiced 10+ times

**Documentation:**
- [ ] Photos taken of wiring and assembly
- [ ] VESC CAN ID recorded in system docs
- [ ] Test results logged
- [ ] Known issues documented
- [ ] Spare parts inventory confirmed

**Demo Readiness:**
- [ ] Operator trained on blower control
- [ ] Backup battery charged
- [ ] Spare impeller on-site
- [ ] Tools for quick fixes available
- [ ] Failure mode responses practiced

---

## Notes

**Revision History:**
- v1.0 (2026-01-14): Initial guide created for F.Inc Artifact Week 4

**Contributors:**
- Assembly guide authored for Muni BVR blower integration

**Feedback:**
If you encounter issues not covered in this guide, or have suggestions for improvement, document them for future revisions.

---

**Good luck with your assembly. Take your time, double-check everything, and don't skip the testing steps. A well-built blower system will make for an impressive demo.**
