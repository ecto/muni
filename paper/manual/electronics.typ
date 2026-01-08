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
  [CAN pins], [CAN transceiver module],
  [USB 3.0], [USB hub (camera, LTE)],
  [12V DC], [From DC-DC converter],
  [GPIO], [Not used (e-stop is hardwired)],
)

#v(1em)

*Software:*
- JetPack 6.0 or later
- bvrd daemon (auto-start on boot)
- Insta360 SDK for camera

#v(0.5em)

*Jetson Setup:*

```bash
# Flash JetPack 6.0 using SDK Manager on host PC
# After boot, install dependencies:
sudo apt update && sudo apt install -y can-utils
pip install pyserial

# Clone firmware repo
git clone https://github.com/muni-works/muni
cd muni/bvr/firmware

# Build and install bvrd
cargo build --release
sudo cp target/release/bvrd /usr/local/bin/
sudo cp config/bvrd.service /etc/systemd/system/
sudo systemctl enable bvrd
```

Full setup instructions at #link("https://github.com/muni-works/muni/blob/main/bvr/firmware/README.md")[github.com/muni-works/muni].

#pagebreak()

// =============================================================================

= GPIO Pinout

#procedure([Reference: GPIO connections], time: "5 min", difficulty: 1)

#v(1em)

BVR0 uses minimal GPIO: just the CAN interface on the carrier board. The E-stop is wired directly in the power path (not relay-controlled). BVR1 adds GPIO-controlled e-stop relay and watchdog.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // 40-pin header representation (pins 1-20)
    content((0, 4), text(size: 8pt, weight: "bold")[40-Pin Header (pins 1-20)])

    // Pin rows (10 rows = pins 1-20)
    for i in range(10) {
      let y = 3 - i * 0.5
      // Left column (odd pins: 1, 3, 5, ...)
      circle((-0.5, y), radius: 0.15, fill: diagram-light, stroke: 0.5pt + diagram-black)
      content((-1.2, y), text(size: 5pt)[#(i * 2 + 1)])
      // Right column (even pins: 2, 4, 6, ...)
      circle((0.5, y), radius: 0.15, fill: diagram-light, stroke: 0.5pt + diagram-black)
      content((1.2, y), text(size: 5pt)[#(i * 2 + 2)])
    }

    // Pin 1 (3.3V) - row 0
    circle((-0.5, 3), radius: 0.15, fill: muni-orange, stroke: none)
    line((-0.65, 3), (-2.5, 3), stroke: 1pt + muni-orange)
    content((-3.2, 3), text(size: 6pt, fill: muni-orange)[3.3V])

    // Pin 6 (GND) - row 2 (pins 5-6), y = 3 - 2*0.5 = 2.0
    circle((0.5, 2), radius: 0.15, fill: diagram-black, stroke: none)
    line((0.65, 2), (2.5, 2), stroke: 1pt + diagram-black)
    content((3.2, 2), text(size: 6pt)[GND])

    // Pin 2 (5V) - row 0
    circle((0.5, 3), radius: 0.15, fill: muni-danger, stroke: none)
    line((0.65, 3), (2.5, 3), stroke: 1pt + muni-danger)
    content((3.2, 3), text(size: 6pt, fill: muni-danger)[5V])

    // Note about unused pins
    content((0, -2.5), text(size: 6pt, fill: diagram-gray)[BVR0: No GPIO used. CAN via carrier board.])
  }),
  caption: [GPIO header reference. BVR0 uses carrier board CAN, not GPIO.],
)

#v(1em)

*BVR0 GPIO Usage:*

BVR0 doesn't use any GPIO pins directly. The CAN bus is provided by the carrier board's dedicated CAN controller (not bit-banged GPIO).

#spec-table(
  [*Pin*], [*Function*], [*BVR0*], [*BVR1*],
  [1], [3.3V], [Unused], [Status LED],
  [2], [5V], [Unused], [Unused],
  [6], [GND], [Unused], [E-Stop ground],
  [32], [GPIO12], [Unused], [E-Stop relay],
)

#v(1em)

#note[
  BVR1 adds e-stop relay on GPIO12. See BVR1 manual for wiring details. The relay is normally-open: GPIO LOW = motors disabled (fail-safe).
]

#pagebreak()

// =============================================================================

= CAN Transceiver

#procedure([Connect CAN bus interface], time: "10 min", difficulty: 1)

#v(1em)

The Jetson carrier board (Seeed reComputer J401 or similar) has CAN controller pins exposed. A CAN transceiver module converts these logic-level signals to the differential CAN bus.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson carrier board
    rect((-5, -1), (-1.5, 1.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((-3.25, 0.8), text(size: 7pt, weight: "bold")[Jetson Carrier])
    content((-3.25, 0.2), text(size: 6pt)[reComputer J401])

    // CAN controller pins
    circle((-1.8, -0.2), radius: 0.1, fill: diagram-black)
    circle((-1.8, -0.5), radius: 0.1, fill: diagram-black)
    content((-2.5, -0.2), text(size: 5pt)[CAN_TX])
    content((-2.5, -0.5), text(size: 5pt)[CAN_RX])

    // Transceiver module
    rect((0.5, -1), (3, 1), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((1.75, 0.3), text(size: 7pt, weight: "bold")[CAN])
    content((1.75, -0.3), text(size: 6pt)[Transceiver])

    // Logic side wires
    line((-1.7, -0.2), (0.5, -0.2), stroke: 1pt + diagram-black)
    line((-1.7, -0.5), (0.5, -0.5), stroke: 1pt + diagram-black)

    // CAN bus side
    line((3, 0.3), (4.5, 0.3), stroke: 1.5pt + rgb("#22c55e"))
    line((3, -0.3), (4.5, -0.3), stroke: 1.5pt + rgb("#22c55e"))
    content((5.2, 0.3), text(size: 6pt, fill: rgb("#22c55e"))[CAN_H])
    content((5.2, -0.3), text(size: 6pt, fill: rgb("#22c55e"))[CAN_L])

    // Power
    line((-1.8, 0.8), (0.5, 0.8), stroke: 1pt + muni-danger)
    content((-2.3, 0.8), text(size: 5pt, fill: muni-danger)[3.3V])

    // GND
    line((-1.8, -0.8), (0.5, -0.8), stroke: 1pt + diagram-black)
    content((-2.3, -0.8), text(size: 5pt)[GND])

    // Termination resistor on module
    rect((1.2, -1.8), (2.3, -1.3), fill: white, stroke: 0.5pt + diagram-black, radius: 2pt)
    content((1.75, -1.55), text(size: 5pt)[120Ω])
    content((1.75, -2.1), text(size: 5pt, fill: diagram-gray)[Onboard term.])
  }),
  caption: [CAN transceiver connects carrier board CAN pins to differential bus.],
)

#v(1em)

*Transceiver Modules:*
- Waveshare SN65HVD230 module
- Any 3.3V CAN transceiver breakout
- Often included on carrier board (check your model)

*Wiring:*
#spec-table(
  [*Carrier Pin*], [*Transceiver*], [*Notes*],
  [CAN_TX], [TXD], [Logic level out],
  [CAN_RX], [RXD], [Logic level in],
  [3.3V], [VCC], [Power],
  [GND], [GND], [Common ground],
)

*Configuration:*
```bash
# Enable CAN interface (may vary by carrier board)
sudo modprobe mttcan
sudo ip link set can0 type can bitrate 500000
sudo ip link set can0 up

# Test with candump
candump can0
```

#v(1em)

*Termination:*
- Most transceiver modules have onboard 120Ω termination
- CAN bus needs exactly 2 termination resistors (one at each end)
- If transceiver is at bus end: enable termination
- Verify: measure 60Ω across CAN_H/CAN_L (two 120Ω in parallel)

#pitfall[
  The reComputer carrier uses the Jetson's native CAN controller (mttcan driver), not a USB adapter. Check your carrier's pinout for CAN_TX/CAN_RX locations.
]

#pagebreak()
