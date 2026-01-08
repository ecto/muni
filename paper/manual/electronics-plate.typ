#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Electronics Section (BVR0)
// Direct chassis mounting, no custom plate

= Electronics Mounting

BVR0 takes the simplest possible approach: mount electronics directly to the chassis using zip ties, electrical tape, and the T-slot channels. No custom plate, no drilling, no fabrication.

This isn't pretty, but it works. The goal of BVR0 is to get a rover running with zero custom parts. You can always upgrade to a proper electronics plate later (see BVR1 manual).

= Direct Mounting Strategy

#procedure([Mount electronics to chassis], time: "45 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of chassis with electronics positions
    rect((-5, -4), (5, 4), stroke: 1.5pt + diagram-black, fill: none, radius: 2pt)

    // Vertical posts at corners
    for (x, y) in ((-4.5, 3.5), (4.5, 3.5), (-4.5, -3.5), (4.5, -3.5)) {
      rect((x - 0.3, y - 0.3), (x + 0.3, y + 0.3), fill: diagram-light, stroke: 1pt + diagram-black)
    }

    // VESCs on vertical posts (at each corner)
    rect((-4.2, 2.2), (-3, 3.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((-3.6, 2.7), text(size: 5pt)[V-FL])

    rect((3, 2.2), (4.2, 3.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((3.6, 2.7), text(size: 5pt)[V-FR])

    rect((-4.2, -3.2), (-3, -2.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((-3.6, -2.7), text(size: 5pt)[V-RL])

    rect((3, -3.2), (4.2, -2.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((3.6, -2.7), text(size: 5pt)[V-RR])
    callout-leader((3.6, -2.7), (6, -2.5), "2")

    // Wheels (outside frame)
    for (x, y) in ((-5.5, 3), (5.5, 3), (-5.5, -3), (5.5, -3)) {
      circle((x, y), radius: 0.6, stroke: 1pt + diagram-gray, fill: white)
    }

    // Jetson + CAN board (center, on top rail)
    rect((-2, 2.5), (1, 4), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
    content((-0.5, 3.5), text(size: 7pt, weight: "bold")[Jetson])
    content((-0.5, 2.9), text(size: 5pt)[+ CAN])
    callout-leader((-0.5, 3.2), (-6, 3.5), "1")

    // DC-DC (on frame somewhere central)
    rect((1.5, 2.5), (2.8, 4), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((2.15, 3.25), text(size: 5pt)[DC-DC])
    callout-leader((2.15, 3.25), (6, 3.5), "3")

    // Battery (center spine)
    rect((-1.5, -1), (1.5, 1), fill: diagram-light, stroke: 1pt + diagram-gray, radius: 2pt)
    content((0, 0), text(size: 6pt)[Battery])

    // Direction
    motion-arrow((0, 1.5), (0, 2.2))
    content((0.5, 1.8), text(size: 5pt)[Front])
  }),
  caption: [VESCs at corners (on vertical posts), Jetson and DC-DC on top rail.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Components:*
    #table(
      columns: (auto, 1fr),
      stroke: none,
      inset: 3pt,
      [#text(fill: muni-orange, weight: "bold")[1]], [Jetson Orin NX + CAN board],
      [#text(fill: muni-orange, weight: "bold")[2]], [VESC 6.7 ×4 (one per corner)],
      [#text(fill: muni-orange, weight: "bold")[3]], [DC-DC 48V→12V],
    )
  ],
  [
    *Mounting Methods:*
    - Electrical tape (quick, repositionable)
    - Zip ties through T-slot channels
    - Velcro strips (for DC-DC)
    - Double-sided foam tape (vibration dampening)
  ]
)

#pagebreak()

// =============================================================================

= Jetson + CAN Board

#procedure([Mount Jetson compute module], time: "15 min", difficulty: 1)

#v(1em)

The Jetson Orin NX sits on a carrier board with an integrated CAN interface. This eliminates the need for a separate USB-CAN adapter.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of Jetson mounted to rail
    // Extrusion
    rect((-3, 0), (3, 0.5), fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 0.25), text(size: 6pt)[2020 Rail])

    // Foam tape layer
    rect((-2, 0.5), (2, 0.7), fill: rgb("#94a3b8"), stroke: 0.5pt + diagram-black)
    content((0, 0.6), text(size: 4pt, fill: white)[Foam tape])

    // Carrier board
    rect((-2.5, 0.7), (2.5, 1), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 0.85), text(size: 5pt)[Carrier board])

    // Jetson module
    rect((-1.5, 1), (1.5, 1.8), fill: diagram-black, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 1.4), text(size: 6pt, fill: white)[Jetson Orin NX])

    // CAN board (stacked or adjacent)
    rect((2.8, 0.7), (4.2, 1.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((3.5, 1.1), text(size: 5pt)[CAN])

    // Electrical tape straps
    line((-2.5, 0.3), (-2.5, 1.8), stroke: 2pt + muni-orange)
    line((2.5, 0.3), (2.5, 1.8), stroke: 2pt + muni-orange)
    content((0, 2.2), text(size: 6pt, fill: muni-orange)[Electrical tape straps])
  }),
  caption: [Jetson mounted with foam tape and electrical tape straps.],
)

#v(1em)

*Mounting Steps:*

+ Clean the extrusion surface with isopropyl alcohol
+ Apply double-sided foam tape to carrier board bottom
+ Press carrier board onto top of frame rail
+ Wrap electrical tape around rail and carrier (2-3 wraps)
+ Connect CAN board to carrier via ribbon cable or headers
+ Route power and data cables away from tape

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Carrier Board with CAN:*
    - Waveshare or Seeed carrier with CAN
    - Or: separate CAN HAT/board
    - CAN-H, CAN-L, GND to VESC bus
    - 120Ω termination at end of bus
  ],
  [
    *Power:*
    - 12V from DC-DC converter
    - Barrel jack or screw terminal
    - ~3A average, 5A peak
  ]
)

#v(1em)

#lesson[
  Electrical tape sounds janky, but it's actually great for prototyping. It's repositionable, leaves no residue, and you can see exactly where everything is. Once the layout is proven, upgrade to proper mounts.
]

#pagebreak()

// =============================================================================

= VESC Mounting

#procedure([Mount motor controllers], time: "15 min", difficulty: 1)

#v(1em)

Each VESC mounts directly to the vertical post next to its wheel. This keeps phase wires as short as possible: the motor is right there.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Isometric-ish view of corner with VESC on vertical post
    content((0, 4.5), text(size: 8pt, weight: "bold")[CORNER DETAIL])

    // Vertical post
    rect((-0.3, -2), (0.3, 3), fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 3.4), text(size: 6pt)[Vertical post])

    // VESC mounted on post
    rect((0.3, 0), (2.3, 1.8), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
    content((1.3, 1.2), text(size: 7pt, weight: "bold")[VESC])
    content((1.3, 0.6), text(size: 6pt)[FL])

    // Tape/velcro indicator
    rect((0.3, 0.5), (0.5, 1.3), fill: muni-orange, stroke: none)
    content((0.4, 1.8), text(size: 5pt, fill: muni-orange)[Tape])

    // Motor (wheel)
    circle((0, -3.5), radius: 1.2, stroke: 2pt + diagram-black, fill: white)
    content((0, -3.5), text(size: 6pt)[Motor])

    // Phase wires (very short!)
    line((1.3, 0), (1.3, -0.5), stroke: 1.5pt + rgb("#3b82f6"))
    line((1.3, -0.5), (0.5, -2.5), stroke: 1.5pt + rgb("#3b82f6"))
    content((2, -1), text(size: 5pt, fill: rgb("#3b82f6"))[Phase])

    // Power wire (longer, to bus)
    line((2.3, 0.9), (3.5, 0.9), stroke: 2pt + diagram-accent)
    content((4.2, 0.9), text(size: 5pt, fill: diagram-accent)[48V])

    // CAN wire
    line((2.3, 1.4), (3.5, 1.4), stroke: 1pt + rgb("#22c55e"))
    content((4.2, 1.4), text(size: 5pt, fill: rgb("#22c55e"))[CAN])
  }),
  caption: [VESC on vertical post, directly adjacent to its motor. Minimal phase wire length.],
)

#v(1em)

*Mounting:*
- Electrical tape or Velcro to vertical post
- VESC flat against post, heatsink facing out
- Position at comfortable height for wiring
- One VESC per corner (4 total)

*Wiring:*
- Phase wires: direct to motor (< 15cm ideal)
- 48V power: runs from central bus to each corner
- CAN: daisy-chain around frame perimeter
- Termination: 120Ω at first and last VESC

#v(1em)

#lesson[
  Mounting VESCs at the corners means longer power runs but shorter phase wires. Phase wires carry high-frequency switching currents: keeping them short reduces EMI and heat. The 48V DC bus doesn't care about a few extra centimeters.
]

#note[
  Label each VESC with its motor position (FL, FR, RL, RR). You'll thank yourself during debugging.
]

#pagebreak()

// =============================================================================

= DC-DC Converter

#procedure([Mount voltage regulator], time: "10 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // DC-DC mounted to vertical rail
    // Vertical extrusion
    rect((-0.3, -2), (0.3, 3), fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 3.4), text(size: 6pt)[Frame rail])

    // DC-DC
    rect((0.5, -0.5), (3, 1.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
    content((1.75, 0.8), text(size: 8pt, weight: "bold")[DC-DC])
    content((1.75, 0.2), text(size: 6pt)[48V→12V])

    // Velcro attachment
    rect((0.3, 0), (0.5, 1), fill: muni-orange, stroke: none)
    content((0.4, 1.5), text(size: 5pt, fill: muni-orange)[Velcro])

    // Input/output wires
    line((3, 1), (4, 1), stroke: 2pt + diagram-accent)
    content((4.5, 1), text(size: 5pt)[48V in])
    line((3, 0), (4, 0), stroke: 1.5pt + rgb("#3b82f6"))
    content((4.5, 0), text(size: 5pt)[12V out])
  }),
  caption: [DC-DC converter attached to frame rail with Velcro.],
)

#v(1em)

*Mounting Tips:*
- Velcro strips work well for DC-DC (easy removal)
- Route high-current wires away from signal wires
- Leave slack for service access
- Position near Jetson to minimize 12V wire runs

#v(1em)

#warning[
  Don't mount the DC-DC upside down. The heatsink needs to face up or outward for convection cooling.
]

#note[
  BVR0 has no inline fuse. The battery's internal BMS provides overcurrent protection. This is acceptable for a prototype but not recommended for production. BVR1 adds proper fusing.
]

#pagebreak()

// =============================================================================

= Wiring Overview

#procedure([Route and secure cables], time: "30 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Simplified wiring diagram
    content((0, 4), text(size: 9pt, weight: "bold")[Cable Routing])

    // Power (red/orange)
    line((-4, 2), (4, 2), stroke: 3pt + diagram-accent)
    content((-5, 2), text(size: 6pt, fill: diagram-accent)[48V])

    // 12V (blue)
    line((-4, 1), (2, 1), stroke: 2pt + rgb("#3b82f6"))
    content((-5, 1), text(size: 6pt, fill: rgb("#3b82f6"))[12V])

    // CAN (green)
    line((-4, 0), (4, 0), stroke: 1.5pt + rgb("#22c55e"))
    content((-5, 0), text(size: 6pt, fill: rgb("#22c55e"))[CAN])

    // Phase wires (gray, multiple)
    for x in (-3, -1, 1, 3) {
      line((x, 2), (x, 3), stroke: 2pt + diagram-gray)
    }
    content((0, 3.3), text(size: 6pt)[Phase wires to motors])

    // Callouts
    content((5, 2), text(size: 5pt)[8-10 AWG])
    content((5, 1), text(size: 5pt)[14-18 AWG])
    content((5, 0), text(size: 5pt)[22 AWG twisted])
  }),
  caption: [Keep power, 12V, and signal cables separated.],
)

#v(1em)

*Cable Management:*
- Bundle power cables together (red/black)
- Bundle CAN cables separately (twisted pair)
- Use split loom or spiral wrap for protection
- Zip tie to frame at regular intervals
- Leave service loops near connectors

#v(1em)

*Cable Separation:*
- Keep CAN bus away from motor phase wires (EMI)
- Cross power and signal cables at 90° angles
- Don't run cables over hot components

#note[
  Messy wiring works for BVR0 prototyping. But label everything. Future you will appreciate it.
]

#pagebreak()
