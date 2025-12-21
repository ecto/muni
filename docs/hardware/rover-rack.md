# Rover Rack

The rover electronics are mounted in a custom 10" rack built from 2020 aluminum
extrusion. This provides a modular, rugged enclosure that integrates with the
rover chassis.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Rover Rack (10" / 2020 AL)                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          2U Touchscreen                             │   │
│  │                    Status / Teleop View / Debug                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  Jetson Orin NX + Carrier Board                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  Network Switch + LTE Modem                                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  RTK GPS (ZED-F9P) + USB CAN Adapter                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  Power Distribution (DC-DC + Fuses)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│                   Mounts to rover chassis via 2020 brackets                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Design

### Why 2020 Aluminum?

- **Modular**: Same extrusion as rover chassis, shared fasteners
- **Rugged**: Handles vibration and impacts
- **Serviceable**: Slide-in components, no special tools
- **Customizable**: Easy to add/move mounting points
- **Cost-effective**: ~$30 in extrusion vs $80+ for commercial rack

### Dimensions

| Dimension  | Measurement | Notes                      |
| ---------- | ----------- | -------------------------- |
| Width      | 254mm (10") | Standard 10" rack width    |
| Depth      | 200mm       | Fits Jetson carrier boards |
| Height     | 6U (267mm)  | 2U display + 4U components |
| Rail width | 20mm        | 2020 extrusion             |

## Bill of Materials

### Frame

| Part                 | Qty | Unit  | Total | Notes            |
| -------------------- | --- | ----- | ----- | ---------------- |
| 2020 extrusion 254mm | 4   | $3    | $12   | Horizontal rails |
| 2020 extrusion 267mm | 4   | $3    | $12   | Vertical posts   |
| 2020 corner bracket  | 8   | $1.50 | $12   | Frame corners    |
| M5×10 BHCS           | 32  | $0.10 | $3    | Frame assembly   |
| M5 T-nut             | 32  | $0.15 | $5    | Frame assembly   |
| **Subtotal**         |     |       | $44   |                  |

### Rack Mounting

| Part                 | Qty | Unit  | Total | Notes              |
| -------------------- | --- | ----- | ----- | ------------------ |
| 10" rack shelf       | 3   | $10   | $30   | Laser cut aluminum |
| 10" rack ears (pair) | 2   | $8    | $16   | For Jetson mount   |
| M4 mounting screws   | 20  | $0.10 | $2    | Component mounting |
| **Subtotal**         |     |       | $48   |                    |

### Display

| Part               | Qty | Unit | Total | Notes              |
| ------------------ | --- | ---- | ----- | ------------------ |
| 7" IPS touchscreen | 1   | $50  | $50   | 1024×600, HDMI     |
| 2U display bracket | 1   | $15  | $15   | Custom or 3D print |
| **Subtotal**       |     |      | $65   |                    |

### Electronics (see [bom.md](bom.md) for full details)

| Part                  | Qty | Unit | Total  | Notes              |
| --------------------- | --- | ---- | ------ | ------------------ |
| Jetson Orin NX 16GB   | 1   | $600 | $600   | Main compute       |
| Carrier board (Seeed) | 1   | $100 | $100   | A603 or similar    |
| LTE modem             | 1   | $80  | $80    | Sierra MC7455      |
| USB CAN adapter       | 1   | $30  | $30    | VESC communication |
| Gigabit switch        | 1   | $20  | $20    | 5-port             |
| ZED-F9P RTK GPS       | 1   | $275 | $275   | RTK-capable        |
| GNSS antenna          | 1   | $80  | $80    | Multi-band         |
| **Subtotal**          |     |      | $1,185 |                    |

### Power Distribution

| Part               | Qty | Unit | Total | Notes              |
| ------------------ | --- | ---- | ----- | ------------------ |
| 48V→12V DC-DC 20A  | 1   | $40  | $40   | Main 12V rail      |
| 48V→5V DC-DC 5A    | 1   | $15  | $15   | Display, USB       |
| Fuse block (6-way) | 1   | $20  | $20   | Branch protection  |
| Terminal blocks    | 4   | $3   | $12   | Power distribution |
| **Subtotal**       |     |      | $87   |                    |

### Total Rack Cost

| Category           | Cost    |
| ------------------ | ------- |
| Frame              | $44     |
| Rack mounting      | $48     |
| Display            | $65     |
| Electronics        | $1,185  |
| Power distribution | $87     |
| **Total**          | ~$1,430 |

Note: This is for the rack assembly only. See [bom.md](bom.md) for the
complete rover BOM including chassis, drivetrain, and perception.

## Assembly

### 1. Frame Construction

```
        254mm (10")
    ┌─────────────────┐
    │ ┌─────────────┐ │
    │ │             │ │  267mm (6U)
    │ │             │ │
    │ │             │ │
    │ └─────────────┘ │
    └─────────────────┘
      2020 extrusion
```

1. Cut 2020 extrusion to length (or order pre-cut)
2. Assemble rectangular frame with corner brackets
3. Add cross-braces if needed for rigidity
4. Mount to rover chassis via T-slot brackets

### 2. Display Installation (Top 2U)

1. Mount display bracket to top of frame
2. Secure 7" display to bracket
3. Route HDMI and power cables down rear channel

### 3. Compute Installation (1U)

1. Mount Jetson carrier board to rack shelf
2. Attach rack ears and slide into frame
3. Connect: HDMI, USB (CAN, GPS), Ethernet, power

### 4. Network/Comms (1U)

1. Mount switch and LTE modem to shelf
2. Connect: Jetson, LiDAR, camera (if ethernet)
3. Install LTE antennas on rover exterior

### 5. RTK GPS (1U)

1. Mount ZED-F9P to shelf with USB hub
2. Connect USB CAN adapter
3. Route GNSS antenna cable to sensor pole

### 6. Power Distribution (Bottom 1U)

1. Mount DC-DC converters to shelf
2. Install fuse block
3. Wire: 48V input, 12V/5V distribution
4. Connect all components

## Wiring Diagram

```
48V Battery
    │
    ├── 8AWG ──► E-Stop Relay ──► Distribution Block
    │                                    │
    │                            ┌───────┴───────┐
    │                            │               │
    │                      48V→12V DC-DC    48V→5V DC-DC
    │                            │               │
    │                      ┌─────┴─────┐    ┌────┴────┐
    │                      │           │    │         │
    │                   Jetson    Switch  Display   USB
    │                   (12V)     (12V)   (5V)      Hub
    │
    └── Direct 48V ──► VESC Controllers (via CAN bus)
```

## Display UI

The 2U touchscreen shows the local dashboard served by the `ui` crate:

| View   | Purpose                          |
| ------ | -------------------------------- |
| Status | Mode, battery, connectivity      |
| Teleop | Video feed (if local operator)   |
| Debug  | CAN status, motor temps, GPS fix |
| Config | WiFi, rover settings             |

Access via touch or connect remotely at `http://rover:8080/dashboard`.

## Mounting to Chassis

The rack mounts to the rover chassis using 2020 L-brackets:

```
       Rover Chassis (2020 frame)
    ┌─────────────────────────────────┐
    │                                 │
    │   ┌─────────────────────────┐   │
    │   │                         │   │
    │   │      Rover Rack         │   │
    │   │     (electronics)       │   │
    │   │                         │   │
    │   └─────────────────────────┘   │
    │         │           │           │
    │    L-bracket   L-bracket        │
    │                                 │
    └─────────────────────────────────┘
```

Use vibration-dampening mounts if needed for sensitive electronics.

## Weatherproofing

For outdoor operation:

| Protection  | Method                         |
| ----------- | ------------------------------ |
| Rain        | Polycarbonate cover + gaskets  |
| Dust        | Filtered vents, sealed cables  |
| Temperature | Active cooling fan with filter |
| Vibration   | Rubber standoffs on components |

The display should be visible through the cover; use anti-glare coating
for outdoor visibility.

## Comparison: Rover vs Depot Rack

| Feature      | Rover Rack           | Depot Rack             |
| ------------ | -------------------- | ---------------------- |
| Frame        | 2020 aluminum        | Commercial 10" rack    |
| Display      | 2U (7")              | 3U (10.1")             |
| Power        | 48V battery          | External power station |
| Compute      | Jetson Orin NX       | Beelink EQ12           |
| Connectivity | LTE + local WiFi     | Ethernet + WiFi        |
| Purpose      | Mobile robot control | Fleet operations       |
