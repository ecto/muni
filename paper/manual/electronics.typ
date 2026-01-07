#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Electronics Section
// VESCs, Jetson, CAN adapter

= Electronics

The electronics are the nervous system: motor controllers that translate commands into wheel motion, a compute module that runs the autonomy stack, and a CAN bus that ties everything together.

We use VESC motor controllers because they're open-source, powerful, and have a decade of real-world use in electric skateboards and robotics. The Jetson Orin NX handles perception and planning. It's overkill for teleoperation, but essential for autonomous operation.

= VESC Mounting

#procedure([Mount motor controllers], time: "20 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // VESC with mounting detail
    rect((-3, -1.5), (3, 1.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 0.5), text(size: 10pt, weight: "bold")[VESC 6.7])
    content((0, -0.3), text(size: 7pt)[60V 100A])

    // Heatsink fins on top
    for x in range(-2, 3) {
      line((x * 0.8, 1.5), (x * 0.8, 1.8), stroke: 1pt + diagram-gray)
    }
    content((0, 2.1), text(size: 6pt)[Heatsink (face up)])

    // Mounting holes
    for (x, y) in ((-2.5, 1), (2.5, 1), (-2.5, -1), (2.5, -1)) {
      circle((x, y), radius: 0.15, fill: white, stroke: 1pt + diagram-black)
    }

    // Power input
    content((-4, 1), text(size: 6pt)[48V+])
    line((-3, 1), (-3.5, 1), stroke: 2pt + diagram-accent)
    content((-4, 0), text(size: 6pt)[GND])
    line((-3, 0), (-3.5, 0), stroke: 1.5pt + diagram-black)

    // Phase output
    content((4.2, 1), text(size: 6pt)[Phase A])
    line((3, 0.8), (3.5, 0.8), stroke: 1.5pt + rgb("#3b82f6"))
    content((4.2, 0.3), text(size: 6pt)[Phase B])
    line((3, 0.3), (3.5, 0.3), stroke: 1.5pt + rgb("#22c55e"))
    content((4.2, -0.4), text(size: 6pt)[Phase C])
    line((3, -0.4), (3.5, -0.4), stroke: 1.5pt + rgb("#eab308"))

    // CAN
    content((4, -1.2), text(size: 6pt)[CAN])
    line((3, -1.2), (3.5, -1.2), stroke: 1pt + diagram-black)

    // Standoffs
    for (x, y) in ((-2.5, -1), (2.5, -1)) {
      rect((x - 0.1, y - 0.5), (x + 0.1, y), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    }
    content((0, -2.3), text(size: 6pt)[M3 standoffs (6-10mm)])
  }),
  caption: [VESC mounted on standoffs for airflow. Heatsink faces up.],
)

#v(1em)

*Mounting:*
- M3×6 standoffs at all 4 corners
- M3×8 bolts through plate into standoffs
- Thermal pad between VESC and plate (optional, for heat transfer)

*Power Connections:*
- 10 AWG wire for 48V input
- XT60 connectors recommended
- Keep power wires short

#v(0.5em)

#pitfall[
  VESCs generate serious heat under load. Without standoffs for airflow, thermal throttling kicks in after 3 minutes of hard driving.
]

#pagebreak()

// =============================================================================

= VESC Configuration

#procedure([Set CAN IDs and motor parameters], time: "10 min per VESC", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of rover showing VESC IDs
    rect((-3, -2.5), (3, 2.5), stroke: 1.5pt + diagram-black, radius: 4pt)

    let wheels = (
      ((-2.5, 2), "0", "FL"),
      ((2.5, 2), "1", "FR"),
      ((-2.5, -2), "2", "RL"),
      ((2.5, -2), "3", "RR"),
    )

    for (pos, id, label) in wheels {
      circle(pos, radius: 0.6, fill: diagram-light, stroke: 1.5pt + diagram-black)
      content(pos, text(size: 10pt, weight: "bold")[#id])
      let label-pos = (pos.at(0) * 1.4, pos.at(1) * 1.15)
      content(label-pos, text(size: 8pt)[#label])
    }

    // Direction arrow
    motion-arrow((0, 2.8), (0, 3.5))
    content((0, 3.8), text(size: 7pt, weight: "bold")[FRONT])
  }),
  caption: [CAN ID assignment. ID 0-3 for wheels, ID 10+ for tools.],
)

#v(1em)

*VESC Tool Configuration:*

#spec-table(
  [*Parameter*], [*Value*],
  [Controller ID], [0, 1, 2, 3 (unique per VESC)],
  [CAN Mode], [VESC],
  [CAN Baud Rate], [CAN_500K],
  [Send CAN Status], [Enabled],
  [CAN Status Rate], [50 Hz],
  [Motor Type], [BLDC or FOC (depends on motor)],
  [Current Limit], [30A (per motor)],
)

#v(1em)

*Motor Detection:*

+ Connect VESC to computer via USB
+ Open VESC Tool
+ Run Motor Detection wizard
+ Save configuration to VESC
+ Disconnect USB, connect CAN

#pagebreak()

// =============================================================================

= Jetson Mounting

#procedure([Install compute module], time: "15 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson module
    rect((-3, -1.5), (3, 1.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 0), text(size: 10pt, weight: "bold")[Jetson Orin NX])

    // Carrier board outline
    rect((-3.5, -2), (3.5, 2), stroke: 1pt + diagram-gray, fill: none, radius: 2pt)
    content((0, -1.7), text(size: 6pt, fill: diagram-gray)[Carrier Board])

    // Mounting holes
    for (x, y) in ((-3, 1.5), (3, 1.5), (-3, -1.5), (3, -1.5)) {
      circle((x, y), radius: 0.15, fill: white, stroke: 1pt + diagram-black)
    }

    // Standoffs below
    for (x, y) in ((-3, -1.5), (3, -1.5)) {
      rect((x - 0.12, y - 0.8), (x + 0.12, y), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    }

    // Connectors
    content((4.5, 1), text(size: 6pt)[USB 3.0])
    line((3, 1), (4, 1), stroke: 1pt + diagram-black)

    content((4.5, 0.3), text(size: 6pt)[USB-C])
    line((3, 0.3), (4, 0.3), stroke: 1pt + diagram-black)

    content((4.5, -0.4), text(size: 6pt)[Ethernet])
    line((3, -0.4), (4, -0.4), stroke: 1pt + diagram-black)

    content((4.5, -1.1), text(size: 6pt)[12V DC])
    line((3, -1.1), (4, -1.1), stroke: 1pt + diagram-black)

    // Airflow arrows
    motion-arrow((-4, 0), (-3.5, 0))
    motion-arrow((3.5, 0), (4, 0))
    content((0, 2.5), text(size: 6pt, fill: diagram-gray)[Airflow])
  }),
  caption: [Jetson mounted on standoffs. Ensure airflow around heatsink.],
)

#v(1em)

*Connections:*

#spec-table(
  [*Port*], [*Connection*],
  [USB 3.0 #1], [USB-CAN adapter],
  [USB 3.0 #2], [USB hub (camera, LTE)],
  [12V DC], [From DC-DC converter],
  [GPIO], [E-Stop relay control],
)

#v(1em)

*Software:*
- JetPack 6.0 or later
- bvrd daemon (auto-start on boot)
- Insta360 SDK for camera

#v(0.5em)

#video-link("https://muni.works/docs/jetson", [Jetson Setup Guide])

#pagebreak()

// =============================================================================

= GPIO Pinout

#procedure([Wire E-Stop relay], time: "10 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // 40-pin header representation
    content((0, 4), text(size: 8pt, weight: "bold")[40-Pin Header (partial)])

    // Pin rows
    for i in range(10) {
      let y = 3 - i * 0.5
      // Left column (odd pins)
      circle((-0.5, y), radius: 0.15, fill: diagram-light, stroke: 0.5pt + diagram-black)
      content((-1.2, y), text(size: 5pt)[#(i * 2 + 1)])
      // Right column (even pins)
      circle((0.5, y), radius: 0.15, fill: diagram-light, stroke: 0.5pt + diagram-black)
      content((1.2, y), text(size: 5pt)[#(i * 2 + 2)])
    }

    // Highlight used pins
    // Pin 32 (GPIO12) - E-Stop
    circle((0.5, 3 - 7.5 * 0.5), radius: 0.15, fill: muni-danger, stroke: none)
    line((0.65, 3 - 7.5 * 0.5), (2.5, 3 - 7.5 * 0.5), stroke: 1pt + muni-danger)
    content((4, 3 - 7.5 * 0.5), text(size: 6pt, fill: muni-danger)[E-Stop Relay])

    // Pin 6 (GND)
    circle((0.5, 3 - 2.5 * 0.5), radius: 0.15, fill: diagram-black, stroke: none)
    line((0.65, 3 - 2.5 * 0.5), (2.5, 3 - 2.5 * 0.5), stroke: 1pt + diagram-black)
    content((3.5, 3 - 2.5 * 0.5), text(size: 6pt)[GND])

    // Pin 1 (3.3V)
    circle((-0.5, 3), radius: 0.15, fill: muni-orange, stroke: none)
    line((-0.65, 3), (-2.5, 3), stroke: 1pt + muni-orange)
    content((-3.5, 3), text(size: 6pt)[3.3V])
  }),
  caption: [GPIO header. Only pins used by BVR0 are highlighted.],
)

#v(1em)

*GPIO Assignments:*

#spec-table(
  [*Pin*], [*GPIO*], [*Function*], [*Direction*], [*Notes*],
  [32], [GPIO12], [E-Stop Relay], [Output], [High = relay closed = power on],
  [6], [GND], [Relay ground], [--], [Common ground],
  [1], [3.3V], [Status LED], [Power], [Optional status indicator],
)

#v(1em)

*E-Stop Relay Wiring:*

```
Jetson Pin 32 (GPIO12) ──┬── Relay coil (+)
                         │
Relay coil (-) ──────────┴── Jetson Pin 6 (GND)
```

The relay is a normally-open (NO) type. When GPIO12 is LOW (default at boot), the relay is open and power is cut to motors. Software must explicitly set GPIO12 HIGH to enable motor power.

#v(0.5em)

#lesson[
  The first prototype used a normally-closed relay. Boot glitch meant motors got power before software loaded. Now we always use normally-open for fail-safe.
]

#pagebreak()

// =============================================================================

= USB-CAN Adapter

#procedure([Connect CAN bus interface], time: "5 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // USB-CAN adapter
    rect((-2, -0.8), (2, 0.8), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 0), text(size: 9pt, weight: "bold")[USB-CAN])

    // USB side
    line((-2, 0), (-3.5, 0), stroke: 1.5pt + diagram-black)
    rect((-4, -0.3), (-3.5, 0.3), fill: diagram-gray, stroke: 1pt + diagram-black, radius: 1pt)
    content((-4.8, 0), text(size: 6pt)[USB])

    // CAN side
    line((2, 0.3), (3.5, 0.3), stroke: 1pt + diagram-black)
    line((2, -0.3), (3.5, -0.3), stroke: 1pt + diagram-black)
    content((4.2, 0.3), text(size: 6pt)[CAN_H])
    content((4.2, -0.3), text(size: 6pt)[CAN_L])

    // Termination switch
    rect((0.5, -1.5), (1.5, -1), fill: diagram-light, stroke: 0.5pt + diagram-black, radius: 2pt)
    content((1, -1.25), text(size: 5pt)[120Ω])
    content((1, -1.8), text(size: 5pt, fill: diagram-gray)[Term. switch])
  }),
  caption: [USB-CAN adapter provides CAN bus access from Jetson.],
)

#v(1em)

*Recommended Adapters:*
- Canable Pro (open source)
- PEAK PCAN-USB
- Innomaker USB-CAN

*Configuration:*
```
# Set up CAN interface
sudo ip link set can0 type can bitrate 500000
sudo ip link set can0 up

# Test with candump
candump can0
```

#v(1em)

*Termination:*
- If adapter is at end of CAN bus: enable 120Ω termination
- If adapter is in middle of chain: disable termination
- Total bus should have exactly 2 termination resistors

#v(0.5em)

#pitfall[
  Three termination resistors = 40Ω total = signal reflections = random VESC dropouts. Use a multimeter to verify 60Ω across CAN_H/CAN_L (two 120Ω in parallel).
]

#pagebreak()
