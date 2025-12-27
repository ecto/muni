# Custom 21700 Battery Pack

Spec for a 48 V-class pack to replace the Towntube, targeting roughly 2× capacity with two packs per rover.

## Goals

- Nominal 48 V system compatibility
- Per-pack energy: ~1.5–1.9 kWh
- Continuous current headroom for traction and tools
- Safe parallel use of two packs on one rover
- Serviceable, swappable enclosure with telemetry

## Electrical architecture

- Topology: 13s Li-ion (46.8 V nominal, 54.6 V full, ~39 V empty)
- Candidates:
  - 13s7p with 5 Ah cells: ~32.5 Ah, ~1.55 kWh
  - 13s8p with 5 Ah cells: ~37–40 Ah, ~1.8–1.9 kWh
- Voltage limits: charge 54.6 V, discharge floor 39 V (BMS cutoff ~36–37 V)
- Current targets (per pack):
  - Continuous: size for 60–80 A
  - Peak: 120–150 A for several seconds
  - Main fuse: near 125 A (refine after load profile)
- Interconnect: copper or nickel-copper busbars sized for <10°C rise at continuous current
- Precharge path to protect downstream caps

## Cell selection

- Energy-focused 21700s for runtime; consider:
  - Samsung 50E/50G, Molicel M50A (energy)
  - Molicel P45B if higher peak current is needed with smaller p-count
- Match cells by capacity and IR; bin and pair by group
- Cell-level or group-level fusing (fusible nickel or PTC)

## BMS and protection

- Smart BMS, 13s, with UART or CAN for telemetry
- Features: balancing, OV/UV, OC, OT/UT, short-circuit protection
- Temp sensing: 3–4 NTCs spread across pack (center and edges)
- Main contactor or high-side solid-state switch, driven by BMS
- Ideal diode or back-to-back FET stage for reverse/current sharing control
- Main fuse placed at pack positive near the terminal

## Parallel operation (two packs on rover)

- Goal: allow both packs to feed the DC bus without backfeeding
- Approaches:
  - Ideal diode controllers on each pack output
  - Or BMS-managed contactors with sequenced close and precharge
- Both packs must be within a small voltage window before paralleling; enforce via BMS rules
- Telemetry should expose pack state so the vehicle controller can decide active/standby behavior

## Connectors and interfaces

- High-current: keyed, touch-safe connectors (SurLok Plus, Anderson SBS/PP series)
- Low-voltage: sealed comms port for BMS CAN/UART and optional wake/enable
- Charge port: recessed, fused, with interlock to block drive during charge

## Mechanical and thermal

- Cell holders with compression to manage vibration; add shock mounting inside enclosure
- Thermal interface pads or air gaps between parallel groups; avoid solid potting that traps heat
- Venting or burst disk plus gasketing for IP54+ sealing; include pressure equalization vent
- Insulation sheets between series groups; flame-retardant barriers if volume allows

## Enclosure and serviceability

- Rugged metal or reinforced polymer housing with a sealed, removable lid
- External indicators: SOC bar or small display, fault LED
- Mounting: sled or rail system for quick swap; keyed so packs cannot be misoriented

## Charging strategy

- CC/CV to 54.6 V; charge current C/5 to C/3 for longevity
- Support onboard and offboard chargers; allow balance-friendly current
- Optional: disable drive when charge port engaged via interlock

## Telemetry

- Expose via CAN or UART:
  - Pack voltage, current, power
  - Per-temp sensors, pack max/min temp
  - SOC, SOH, cycle count
  - Faults and contactor state
- Rover controller should log these for lifetime and diagnostics

## Test and validation

- Electrical: load step tests to verify voltage sag and thermal rise at continuous and peak currents
- Protection: induce controlled OV/UV/OC/OT events to verify BMS actions and contactor behavior
- Thermal: worst-case duty in sealed enclosure, monitor hottest cells and busbars
- Parallel: connect two packs at different SOCs within allowed window, verify current sharing and no backfeed
- Environmental: vibration and shock per rover profile; water ingress check for connectors and seams

## Open items to finalize

- Pick cell model (energy vs mid-power) and p-count (7p vs 8p)
- Choose BMS with CAN and contactor drive that supports parallel packs
- Select connector family and fuse ratings
- Define precharge resistor and timing
- Confirm enclosure form factor and mounting with rover frame
