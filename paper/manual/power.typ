#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Power System Section
// Battery, Fuse, DC-DC, Distribution

= Power System

The rover runs on 48V nominal (13S lithium). This voltage is high enough to be efficient (less current means thinner wires and less heat) but low enough to avoid the regulatory complexity of "high voltage" systems.

BVR0 keeps the power system as simple as possible: battery → distribution bus → loads. The battery's integrated BMS handles overcurrent and undervoltage protection. No separate inline fuses.

Respect the battery. A 48V 20Ah pack stores nearly 1 kWh of energy. That's enough to weld metal if shorted, or start a fire if punctured. The safety section covers handling in detail.

#note[
  BVR0 has an E-stop on the sensor mast but no inline fuse. The E-stop cuts power to all motors. BVR1 adds fuses and a relay-based e-stop with watchdog.
]

= Battery Mounting

#procedure([Mount downtube battery to frame], time: "15 min", difficulty: 1)

#v(1em)

BVR0 uses an off-the-shelf 48V downtube-style e-bike battery. These batteries have an integrated mounting system: a bracket bolts to the frame, and the battery slides in and locks. No custom fabrication required.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of downtube battery on central spine
    content((0, 4), text(size: 8pt, weight: "bold")[SIDE VIEW])

    // Central spine (2020 extrusion running front-to-back)
    rect((-5, 0), (5, 0.5), fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 0.25), text(size: 6pt)[Central spine (2020)])

    // Battery bracket (bolted to extrusion)
    rect((-3, 0.5), (3, 0.8), fill: diagram-gray, stroke: 1pt + diagram-black)
    content((0, 0.65), text(size: 5pt)[Mounting rail])

    // Downtube battery (elongated shape)
    rect((-3.5, 0.8), (3.5, 2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 1.6), text(size: 8pt, weight: "bold")[48V 20Ah])
    content((0, 1.2), text(size: 6pt)[Downtube battery])

    // Lock mechanism
    circle((3, 1.4), radius: 0.2, fill: muni-orange, stroke: 1pt + diagram-black)
    content((4, 1.4), text(size: 5pt, fill: muni-orange)[Key lock])

    // Power cable
    line((3.5, 1.4), (5, 1.4), stroke: 2pt + diagram-accent)
    connector-xt((5.5, 1.4), size: "60")
    content((6.5, 1.4), text(size: 5pt)[XT60])

    // Dimensions
    dim-h(-4, -3.5, 3.5, "~400mm", offset: 0.3)
  }),
  caption: [Downtube battery slides onto rail mounted to central 2020 spine.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Battery Requirements:*
    - 48V nominal (13S lithium)
    - 15-20Ah capacity
    - Downtube/bottle mount style
    - Integrated BMS
    - XT60 or similar output connector

    #v(0.3em)
    *Common sources:*
    - Unit Pack Power (AliExpress)
    - Luna Cycle
    - EM3ev
  ],
  [
    *Mounting:*
    - Battery rail bolts directly to 2020 T-slot
    - Use M5 bolts + T-nuts (2-4 per rail)
    - Rail position: center of frame, lengthwise
    - Battery slides in from one end, locks with key

    #v(0.3em)
    *Why downtube?*
    Off-the-shelf, replaceable, keyed lock, integrated BMS, weather-resistant housing.
  ]
)

#v(1em)

#note[
  No custom battery tray needed. The downtube battery's integrated mounting system is purpose-built for this. One less custom part.
]

#pagebreak()

// =============================================================================

= Safety Considerations

BVR0 has a physical E-stop button on the sensor mast and relies on the battery's BMS for overcurrent protection. This is sufficient for prototype testing and supervised operation.

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *What BVR0 has:*
    - E-stop button (physical kill on mast)
    - BMS overcurrent protection (~60-100A)
    - BMS undervoltage cutoff
    - BMS short circuit protection
    - Cell balancing
  ],
  [
    *What BVR1 adds:*
    - Inline fuse (wire protection)
    - E-stop relay (software-controlled kill)
    - Watchdog timer (auto-stop on software crash)
    - Headlights and tail lights
  ]
)

#v(1em)

*Emergency Shutdown (BVR0):*
+ Press E-stop button on sensor mast (cuts power to motors)
+ Or: remove battery key and slide battery off rail
+ Or: disconnect XT60 at battery output

#v(1em)

#note[
  The E-stop button is wired directly in the motor power path. Pressing it immediately cuts power to all VESCs. No software required.
]

#pagebreak()

// =============================================================================

= DC-DC Converter

#procedure([Install voltage regulator], time: "10 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // DC-DC converter diagram
    rect((-2, -1.5), (2, 1.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 0.5), text(size: 10pt, weight: "bold")[DC-DC])
    content((0, -0.3), text(size: 8pt)[48V → 12V])
    content((0, -0.9), text(size: 7pt)[10A / 120W])

    // Input
    line((-4, 0.5), (-2, 0.5), stroke: 2pt + diagram-accent)
    line((-4, -0.5), (-2, -0.5), stroke: 1.5pt + diagram-black)
    content((-4.5, 0.5), text(size: 6pt)[48V+])
    content((-4.5, -0.5), text(size: 6pt)[GND])

    // Output
    line((2, 0.5), (4, 0.5), stroke: 1.5pt + rgb("#3b82f6"))
    line((2, -0.5), (4, -0.5), stroke: 1.5pt + diagram-black)
    content((4.5, 0.5), text(size: 6pt)[12V+])
    content((4.5, -0.5), text(size: 6pt)[GND])

    // Loads
    rect((5, -0.3), (7, 0.3), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((6, 0), text(size: 6pt)[Jetson])
    line((4, 0.5), (5, 0), stroke: 1pt + rgb("#3b82f6"))

    rect((5, -1.3), (7, -0.7), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((6, -1), text(size: 6pt)[LTE/USB])
    line((4, 0.5), (5, -1), stroke: 1pt + rgb("#3b82f6"))
  }),
  caption: [DC-DC powers all 12V devices from the 48V main bus.],
)

#v(1em)

*Specifications:*
#spec-table(
  [*Parameter*], [*Value*],
  [Input voltage], [36-60V (fits 13S LiPo range)],
  [Output voltage], [12V regulated],
  [Output current], [10A continuous],
  [Efficiency], [>90%],
  [Mounting], [M3 holes, heatsink on bottom],
)

#v(1em)

*12V Load Budget:*
#spec-table(
  [*Device*], [*Current*],
  [Jetson Orin NX], [~5A peak, ~3A average],
  [LTE modem], [~1A],
  [USB hub], [~0.5A],
  [Accessories], [~1A reserve],
  [*Total*], [*~6A typical, 10A max*],
)

#pagebreak()

// =============================================================================

= Power Distribution

#procedure([Wire power bus], time: "30 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Battery at top
    battery-top((0, 3), size: (2, 1))
    content((0, 3), text(size: 7pt, weight: "bold")[48V])

    // Main line direct to bus
    line((0, 2.5), (0, 0.3), stroke: 3pt + diagram-accent)

    // Power bus bar
    rect((-4, -0.3), (4, 0.3), fill: diagram-accent, stroke: none, radius: 2pt)
    content((0, 0), text(size: 7pt, fill: white, weight: "bold")[POWER BUS])

    // Branches to VESCs (at corners)
    for (i, x) in ((-3.5, -1.5, 1.5, 3.5)).enumerate() {
      line((x, -0.3), (x, -1), stroke: 2pt + diagram-accent)
      vesc-top((x, -1.8), size: (1.2, 0.8), id: str(i + 1))
    }

    // Branch to DC-DC
    line((0, -0.3), (0, -1), stroke: 2pt + diagram-accent)
    rect((-0.6, -1.8), (0.6, -1), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, -1.4), text(size: 6pt)[DC-DC])
    line((0, -1.8), (0, -2.3), stroke: 1.5pt + rgb("#3b82f6"))
    content((0, -2.6), text(size: 6pt)[12V])

    // Note: E-stop on mast, no inline fuse
    content((5, 1.5), text(size: 6pt, fill: diagram-gray)[No inline fuse])
    content((5, 1), text(size: 6pt, fill: diagram-gray)[E-stop on mast])
    content((5, 0.5), text(size: 6pt, fill: diagram-gray)[(BVR0)])
  }),
  caption: [BVR0 power topology: battery direct to bus. Simple but minimal protection.],
)

#v(1em)

*Bus Options:*

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Splitter Cable (BVR0):*
    - XT60 to 5× XT60 (or solder joints)
    - Simple, no fabrication
    - Higher resistance, less tidy
  ],
  [
    *Bus Bar (BVR1):*
    - Solid copper bar with tapped holes
    - Clean, low resistance
    - Integrates fuse and e-stop relay
  ]
)

#pagebreak()
