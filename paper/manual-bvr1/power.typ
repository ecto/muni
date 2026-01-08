#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Power System Section (BVR1)
// Custom battery pack, Fuse, DC-DC, Distribution, Lighting

= Power System

The rover runs on 48V nominal (13S lithium). This voltage is high enough to be efficient (less current means thinner wires and less heat) but low enough to avoid the regulatory complexity of "high voltage" systems.

BVR1 uses a custom battery pack instead of an off-the-shelf downtube battery. This allows for a form factor optimized for the rover's layout and higher capacity cells.

The power path is simple: battery → fuse → e-stop relay → distribution bus → loads. Every component can be isolated, and the e-stop cuts power to everything downstream instantly.

Respect the battery. A 48V 20Ah pack stores nearly 1 kWh of energy. That's enough to weld metal if shorted, or start a fire if punctured. The safety section covers handling in detail.

= Battery Pack

#procedure([Install custom battery pack], time: "20 min", difficulty: 2)

#v(1em)

BVR1 uses a custom 13S4P battery pack built with 21700 cells. The pack mounts in a dedicated tray with integrated BMS and Anderson PowerPole connectors.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of battery mounting
    // Frame rail
    rect((-5, 0), (5, 0.4), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 0.2), text(size: 6pt)[Frame rail])

    // Battery tray
    rect((-4, 0.4), (4, 0.7), fill: diagram-gray, stroke: 1pt + diagram-black)
    content((0, 0.55), text(size: 5pt)[Tray])

    // Battery pack
    rect((-3.5, 0.7), (3.5, 2.2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
    content((0, 1.6), text(size: 8pt, weight: "bold")[48V 28Ah])
    content((0, 1.1), text(size: 6pt)[13S4P 21700])

    // Retention strap
    line((-3.5, 2.2), (-3.5, 2.6), stroke: 1.5pt + muni-orange)
    line((-3.5, 2.6), (3.5, 2.6), stroke: 1.5pt + muni-orange)
    line((3.5, 2.6), (3.5, 2.2), stroke: 1.5pt + muni-orange)
    content((0, 2.9), text(size: 6pt, fill: muni-orange)[Retention strap])

    // Power connector
    rect((4, 1.2), (4.6, 1.7), fill: diagram-accent, stroke: 1pt + diagram-black)
    content((5.3, 1.45), text(size: 5pt)[Anderson])

    // Dimension
    dim-h(-5.5, -3.5, 3.5, "200mm", offset: 0.3)
  }),
  caption: [Custom battery pack mounted in tray with retention strap.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Pack Specifications:*
    - Configuration: 13S4P (48V nominal)
    - Cells: Samsung 50E or Molicel P42A
    - Capacity: 28Ah (1,344 Wh)
    - Max discharge: 80A continuous
    - BMS: 13S 100A with balancing
    - Connector: Anderson SB50 or similar
  ],
  [
    *Tray Construction:*
    - Material: 2mm 6061-T6 aluminum
    - Flat size: 240mm × 180mm
    - Bend: 15mm lip on all 4 sides (90°)
    - 10mm EVA foam padding (bottom)
    - Retention: 25mm nylon strap + cam buckle

    #v(0.3em)
    *CAD File:* `bvr/cad/battery-tray.dxf`
  ]
)

#v(1em)

#warning[
  Battery must not shift during operation. Loose batteries can short on frame, causing fire.
]

#pagebreak()

// =============================================================================

= Fuse and E-Stop

#procedure([Wire safety disconnect], time: "30 min", difficulty: 2)

#v(1em)

BVR1 adds proper safety systems that BVR0 lacks. The design philosophy: one main fuse protects the wiring, one relay provides software-controlled shutdown.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Power path diagram
    // Battery
    battery-top((-5, 0), size: (1.5, 0.8))

    // Anderson connector
    rect((-3.4, -0.3), (-2.6, 0.3), fill: diagram-accent, stroke: 1pt + diagram-black)
    line((-4.25, 0), (-3.4, 0), stroke: 2pt + diagram-accent)

    // Fuse
    rect((-2, -0.4), (-0.5, 0.4), fill: rgb("#fbbf24"), stroke: 1pt + diagram-black, radius: 2pt)
    content((-1.25, 0), text(size: 8pt, weight: "bold")[100A])
    line((-2.6, 0), (-2, 0), stroke: 2pt + diagram-accent)
    callout-leader((-1.25, 0), (-1.25, 1.5), "1")

    // E-Stop relay
    rect((0.5, -0.4), (2, 0.4), fill: muni-danger, stroke: 1pt + diagram-black, radius: 2pt)
    content((1.25, 0), text(size: 7pt, fill: white, weight: "bold")[E-STOP])
    line((-0.5, 0), (0.5, 0), stroke: 2pt + diagram-accent)
    callout-leader((1.25, 0), (1.25, 1.5), "2")

    // Output
    line((2, 0), (3, 0), stroke: 2pt + diagram-accent)
    content((4, 0), text(size: 7pt)[To power bus])

    // E-Stop button
    circle((1.25, -2), radius: 0.5, fill: muni-danger, stroke: 2pt + diagram-black)
    content((1.25, -2), text(size: 6pt, fill: white)[STOP])
    line((1.25, -1.5), (1.25, -0.4), stroke: 1pt + diagram-black)
    content((2.5, -2), text(size: 6pt)[Mushroom button])

    // Watchdog signal
    line((1.25, 0.4), (1.25, 0.8), stroke: 1pt + rgb("#22c55e"))
    content((2.5, 0.8), text(size: 5pt, fill: rgb("#22c55e"))[Watchdog])
  }),
  caption: [Power flows: Battery → Connector → Fuse → E-Stop relay → Power bus],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *#text(fill: muni-orange)[1] Main Fuse (100A):*
    - ANL or MIDI style fuse
    - Inline fuse holder with ring terminals
    - Protects main wiring from shorts
    - Mount accessible for replacement
    - Sized for wire gauge, not load
  ],
  [
    *#text(fill: muni-orange)[2] E-Stop Relay:*
    - Normally-open contactor
    - 12V coil with watchdog timer
    - 100A+ contact rating
    - Fails safe: power loss = motors stop
    - Physical button + software control
  ]
)

#v(1em)

*Why no per-VESC fuses?*
VESCs have built-in overcurrent protection. Adding fuses per VESC would only protect the short wire run from bus to VESC. The main fuse protects the longer battery-to-bus wiring where a short is most likely.

#pagebreak()

// =============================================================================

= E-Stop Relay Wiring

#procedure([Wire GPIO to relay], time: "15 min", difficulty: 2)

#v(1em)

The e-stop relay is controlled by the Jetson's GPIO. A normally-open relay means the default state (GPIO LOW at boot) keeps motors disabled.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson box
    rect((-5, 1), (-1, 3), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((-3, 2.5), text(size: 8pt, weight: "bold")[Jetson])
    content((-3, 1.8), text(size: 6pt)[GPIO Header])

    // Pin 32 (GPIO12)
    circle((-1.5, 2.2), radius: 0.12, fill: muni-danger, stroke: 1pt + diagram-black)
    content((-2.3, 2.2), text(size: 5pt)[Pin 32])

    // Pin 6 (GND)
    circle((-1.5, 1.5), radius: 0.12, fill: diagram-black, stroke: none)
    content((-2.3, 1.5), text(size: 5pt)[Pin 6])

    // Relay coil box
    rect((2, 0.5), (5, 3.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((3.5, 3), text(size: 8pt, weight: "bold")[Relay])

    // Coil symbol
    rect((2.8, 1.3), (4.2, 2.3), fill: white, stroke: 1pt + diagram-black, radius: 2pt)
    content((3.5, 1.8), text(size: 6pt)[Coil])

    // Coil terminals
    content((2.5, 2.5), text(size: 5pt)[(+)])
    content((2.5, 1.1), text(size: 5pt)[(−)])

    // Wire from GPIO12 to coil (+)
    line((-1.38, 2.2), (0.5, 2.2), stroke: 1.5pt + muni-danger)
    line((0.5, 2.2), (0.5, 2.5), stroke: 1.5pt + muni-danger)
    line((0.5, 2.5), (2.8, 2.5), stroke: 1.5pt + muni-danger)
    circle((2.8, 2.5), radius: 0.1, fill: muni-danger, stroke: none)

    // Wire from GND to coil (-)
    line((-1.38, 1.5), (0, 1.5), stroke: 1.5pt + diagram-black)
    line((0, 1.5), (0, 1.1), stroke: 1.5pt + diagram-black)
    line((0, 1.1), (2.8, 1.1), stroke: 1.5pt + diagram-black)
    circle((2.8, 1.1), radius: 0.1, fill: diagram-black, stroke: none)

    // Labels
    content((0.5, 2.8), text(size: 5pt, fill: muni-danger)[GPIO12])
    content((0, 0.8), text(size: 5pt)[GND])

    // Relay contacts (NO)
    line((4.2, 1.8), (5.5, 1.8), stroke: 1.5pt + diagram-accent)
    line((5.5, 1.8), (5.5, 2.5), stroke: 1.5pt + diagram-accent)
    line((5.5, 2.5), (6.5, 2.5), stroke: 1.5pt + diagram-accent)
    content((6, 3), text(size: 5pt)[To 48V bus])

    // NO indicator
    line((5.2, 1.5), (5.8, 1.2), stroke: 1pt + diagram-gray)
    content((6.3, 1.3), text(size: 5pt, fill: diagram-gray)[NO])
  }),
  caption: [E-stop relay wiring. GPIO12 HIGH closes relay, enabling motor power.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Wiring:*
    - Jetson Pin 32 (GPIO12) → Relay coil (+)
    - Jetson Pin 6 (GND) → Relay coil (−)
    - Relay contacts in series with 48V bus
  ],
  [
    *Relay Selection:*
    - Coil: 3.3V or 5V (with transistor driver)
    - Contacts: 100A+ DC rated
    - Type: Normally-Open (NO)
    - Example: TE Connectivity EV200
  ]
)

#v(1em)

*Operation:*
- Boot: GPIO12 is LOW → relay open → motors disabled
- Software sets GPIO12 HIGH → relay closes → motors enabled
- Watchdog timeout → GPIO12 goes LOW → motors disabled
- Physical e-stop button breaks coil circuit → motors disabled

#v(0.5em)

#lesson[
  The first prototype used a normally-closed relay. A boot glitch meant motors got power before software loaded. Now we always use normally-open for fail-safe.
]

#pagebreak()

// =============================================================================

= Lighting System

#procedure([Install headlights and tail lights], time: "30 min", difficulty: 2)

#v(1em)

BVR1 includes integrated lighting for visibility and safety. Headlights illuminate the path ahead for camera perception in low light. Tail lights provide visibility to pedestrians and vehicles.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of rover with light positions
    rect((-4, -3), (4, 3), stroke: 1.5pt + diagram-black, fill: none, radius: 2pt)

    // Headlights (front)
    rect((-3.5, 2.8), (-2.5, 3.2), fill: rgb("#fef08a"), stroke: 1pt + diagram-black, radius: 2pt)
    rect((2.5, 2.8), (3.5, 3.2), fill: rgb("#fef08a"), stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 3.7), text(size: 7pt, weight: "bold")[Headlights (white)])

    // Tail lights (rear)
    rect((-3.5, -3.2), (-2.5, -2.8), fill: muni-danger, stroke: 1pt + diagram-black, radius: 2pt)
    rect((2.5, -3.2), (3.5, -2.8), fill: muni-danger, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, -3.7), text(size: 7pt, weight: "bold")[Tail lights (red)])

    // Direction arrow
    motion-arrow((0, 1.5), (0, 2.2))
    content((0.5, 1.8), text(size: 6pt)[Front])

    // MCU location
    rect((-1, -1), (1, 0.5), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 2pt)
    content((0, -0.25), text(size: 6pt)[LED MCU])
  }),
  caption: [Light positions: white headlights front, red tail lights rear.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Headlights:*
    - 2× high-power white LED modules
    - 12V, 5W each
    - Wide beam for path illumination
    - Mount to front frame rail
    - Controlled via CAN bus from MCU
  ],
  [
    *Tail Lights:*
    - 2× red LED strips or modules
    - 12V, 2W each
    - Always on when rover is powered
    - Mount to rear frame rail
    - Can flash during reversing
  ]
)

#v(1em)

*Wiring:*
- Lights connect to LED MCU (RP2350)
- MCU receives commands over CAN bus
- 12V power from DC-DC converter
- Use 18 AWG wire for light circuits

#v(1em)

*Light States:*
#spec-table(
  [*State*], [*Headlights*], [*Tail Lights*],
  [Idle], [Off], [Dim (10%)],
  [Teleop], [On], [On],
  [Autonomous], [On], [On],
  [Reversing], [On], [Flashing],
  [E-Stop], [Off], [Fast flash],
)

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
    content((0, -0.9), text(size: 7pt)[15A / 180W])

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
    rect((5, 0.7), (7, 1.3), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((6, 1), text(size: 6pt)[Jetson])
    line((4, 0.5), (5, 1), stroke: 1pt + rgb("#3b82f6"))

    rect((5, -0.3), (7, 0.3), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((6, 0), text(size: 6pt)[Lights])
    line((4, 0.5), (5, 0), stroke: 1pt + rgb("#3b82f6"))

    rect((5, -1.3), (7, -0.7), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((6, -1), text(size: 6pt)[LTE/USB])
    line((4, 0.5), (5, -1), stroke: 1pt + rgb("#3b82f6"))
  }),
  caption: [DC-DC powers all 12V devices including lighting.],
)

#v(1em)

*Specifications:*
#spec-table(
  [*Parameter*], [*Value*],
  [Input voltage], [36-60V (fits 13S LiPo range)],
  [Output voltage], [12V regulated],
  [Output current], [15A continuous],
  [Efficiency], [>90%],
  [Mounting], [M3 holes, heatsink on bottom],
)

#v(1em)

*12V Load Budget:*
#spec-table(
  [*Device*], [*Current*],
  [Jetson Orin NX], [~5A peak, ~3A average],
  [Headlights], [~1A],
  [Tail lights], [~0.5A],
  [LTE modem], [~1A],
  [USB hub], [~0.5A],
  [Accessories], [~1A reserve],
  [*Total*], [*~8A typical, 12A max*],
)

#note[
  BVR1 uses a 15A DC-DC (vs 10A in BVR0) to handle the additional lighting load.
]

#pagebreak()

// =============================================================================

= Power Distribution

#procedure([Wire power bus], time: "45 min", difficulty: 3)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Battery at top
    battery-top((0, 4), size: (2, 1))
    content((0, 4), text(size: 7pt, weight: "bold")[48V])

    // Main line
    line((0, 3.5), (0, 2.5), stroke: 3pt + diagram-accent)

    // Fuse
    rect((-0.5, 2), (0.5, 2.5), fill: rgb("#fbbf24"), stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 2.25), text(size: 6pt)[100A])

    // E-Stop
    line((0, 2), (0, 1.2), stroke: 3pt + diagram-accent)
    rect((-0.5, 0.7), (0.5, 1.2), fill: muni-danger, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 0.95), text(size: 5pt, fill: white)[ESTOP])

    // Power bus bar
    line((0, 0.7), (0, 0), stroke: 3pt + diagram-accent)
    rect((-4, -0.3), (4, 0.3), fill: diagram-accent, stroke: none, radius: 2pt)
    content((0, 0), text(size: 7pt, fill: white, weight: "bold")[POWER BUS])

    // Branches to VESCs
    for (i, x) in ((-3, -1.5, 0, 1.5)).enumerate() {
      line((x, -0.3), (x, -1), stroke: 2pt + diagram-accent)
      vesc-top((x, -1.8), size: (1.2, 0.8), id: str(i + 1))
    }

    // Branch to DC-DC
    line((3, -0.3), (3, -1), stroke: 2pt + diagram-accent)
    rect((2.4, -1.8), (3.6, -1), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((3, -1.4), text(size: 6pt)[DC-DC])
    line((3, -1.8), (3, -2.3), stroke: 1.5pt + rgb("#3b82f6"))
    content((3, -2.6), text(size: 6pt)[12V])
  }),
  caption: [Power distribution topology. All 48V loads connect to central bus.],
)

#v(1em)

*Bus Options:*

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Bus Bar (recommended):*
    - Solid copper bar with tapped holes
    - Clean, low resistance
    - Easy inspection
  ],
  [
    *Splitter Cable:*
    - Anderson to 4× XT60
    - Simpler for prototypes
    - Higher resistance
  ]
)

#pagebreak()
