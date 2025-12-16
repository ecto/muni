# Assembly Guide

## Bill of Materials

### Chassis

| Part                 | Qty | Source        |
| -------------------- | --- | ------------- |
| 2020 extrusion 600mm | 8   | 80/20, Misumi |
| 2020 corner bracket  | 16  | Amazon        |
| M5×10 button head    | 64  | McMaster      |
| M5 T-nut             | 64  | Amazon        |

### Drivetrain

| Part                 | Qty | Source           |
| -------------------- | --- | ---------------- |
| Hoverboard hub motor | 4   | eBay, AliExpress |
| VESC 6               | 4   | Trampa, Flipsky  |
| Motor mount bracket  | 4   | Custom (CAD)     |
| Wheel adapter        | 4   | Custom (CAD)     |

### Electronics

| Part              | Qty | Source             |
| ----------------- | --- | ------------------ |
| Jetson Orin NX    | 1   | NVIDIA             |
| 13S 20Ah battery  | 1   | Custom or supplier |
| 48V→12V DCDC 20A  | 1   | Amazon             |
| CAN transceiver   | 1   | Amazon (MCP2551)   |
| E-Stop relay 100A | 1   | Amazon             |
| E-Stop button     | 1   | Amazon             |
| LTE modem         | 1   | Sierra Wireless    |

### Wiring

| Part                   | Qty | Source |
| ---------------------- | --- | ------ |
| 8 AWG silicone wire    | 3m  | Amazon |
| 14 AWG silicone wire   | 5m  | Amazon |
| 22 AWG twisted pair    | 3m  | Amazon |
| XT90 connectors        | 4   | Amazon |
| XT30 connectors        | 10  | Amazon |
| Heat shrink assortment | 1   | Amazon |

## Assembly Steps

### 1. Chassis Frame

1. Cut extrusions to length (if not pre-cut)
2. Deburr all cuts
3. Assemble base rectangle (600×600mm)
4. Add corner gussets for rigidity
5. Mount vertical supports for electronics

### 2. Motor Mounting

1. Attach motor mount brackets to extrusion
2. Install hub motors into brackets
3. Route phase wires to VESC locations
4. Ensure wheels spin freely

### 3. Electronics Mounting

1. Mount electronics plate to chassis
2. Install Jetson with standoffs
3. Mount VESCs with thermal pad to plate
4. Install DCDC converter
5. Route all power wiring

### 4. Wiring

**Power:**

1. Connect battery to main fuse
2. Wire fuse → E-Stop relay → VESCs
3. Wire battery → DCDC → Jetson/accessories
4. Install XT90 for battery disconnect

**Signal:**

1. Connect CAN bus daisy chain
2. Add 120Ω termination resistors
3. Wire E-Stop button to relay
4. Connect LTE modem to Jetson USB

### 5. Testing

1. Check all connections with multimeter
2. Power on with battery (no motors first)
3. Verify Jetson boots
4. Verify VESCs initialize
5. Test E-Stop cuts power
6. Run motor spin test (wheels off ground)

## Tool Mounting

Tools attach via:

- Quick-release mechanical mount (front)
- Deutsch DT connector (power + CAN)

See individual tool assembly guides in respective repos.

## Quality Checklist

- [ ] All bolts torqued
- [ ] No exposed wiring
- [ ] CAN termination verified
- [ ] E-Stop tested
- [ ] No rubbing/interference when wheels spin
- [ ] Battery secure and protected
- [ ] All connectors fully seated
- [ ] Thermal management adequate
