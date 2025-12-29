#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Wiring Section
// CAN bus, Power, Signals, Cable management

= CAN Bus Wiring

Daisy-chain all CAN devices together.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // CAN bus topology
    // Jetson/CAN adapter
    rect((-6, -0.5), (-4.5, 0.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 4pt)
    content((-5.25, 0), text(size: 6pt, weight: "bold")[Jetson])

    // VESCs
    for (i, x) in ((-3, -1, 1, 3)).enumerate() {
      rect((x - 0.6, -0.5), (x + 0.6, 0.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 4pt)
      content((x, 0), text(size: 6pt)[V#(i+1)])
    }

    // Tool
    rect((5, -0.5), (6.5, 0.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 4pt)
    content((5.75, 0), text(size: 6pt, weight: "bold")[Tool])

    // CAN bus lines
    line((-4.5, 0.2), (5, 0.2), stroke: 1.5pt + diagram-black)
    line((-4.5, -0.2), (5, -0.2), stroke: 1.5pt + diagram-gray)
    content((0, 0.6), text(size: 6pt)[CAN_H (black)])
    content((0, -0.6), text(size: 6pt)[CAN_L (gray)])

    // Termination resistors
    line((-5.25, -0.5), (-5.25, -1.2), stroke: 1pt + diagram-black)
    rect((-5.5, -1.5), (-5, -1.2), fill: white, stroke: 1pt + diagram-black)
    content((-5.25, -1.35), text(size: 5pt)[120Ω])

    line((5.75, -0.5), (5.75, -1.2), stroke: 1pt + diagram-black)
    rect((5.5, -1.5), (6, -1.2), fill: white, stroke: 1pt + diagram-black)
    content((5.75, -1.35), text(size: 5pt)[120Ω])

    // Callouts
    callout-leader((-5.25, -1.35), (-6, -2), "A")
    callout-leader((5.75, -1.35), (6.5, -2), "B")
  }),
  caption: [CAN bus with 120Ω termination at each end (A and B).],
)

#v(1em)

*Wiring Rules:*

- Use twisted pair wire (22 AWG)
- CAN_H to CAN_H, CAN_L to CAN_L at each device
- Maximum total bus length: 40m at 500K baud
- Exactly 2 termination resistors (one at each end)
- Keep CAN wires away from motor phase wires

*JST Connector Pinout:*
#spec-table(
  [*Pin*], [*Signal*], [*Color (typical)*],
  [1], [GND], [Black],
  [2], [CAN_L], [Gray or White],
  [3], [CAN_H], [Orange or Yellow],
  [4], [+5V (optional)], [Red],
)

#pagebreak()

// =============================================================================

= Motor Phase Wiring

Connect VESC outputs to hub motor phase wires.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // VESC
    rect((-3, -1), (0, 1), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((-1.5, 0), text(size: 9pt, weight: "bold")[VESC])

    // Phase outputs
    line((0, 0.6), (1.5, 0.6), stroke: 2pt + rgb("#3b82f6"))
    line((0, 0), (1.5, 0), stroke: 2pt + rgb("#22c55e"))
    line((0, -0.6), (1.5, -0.6), stroke: 2pt + rgb("#eab308"))

    content((0.8, 1), text(size: 6pt)[A (Blue)])
    content((0.8, 0.4), text(size: 6pt)[B (Green)])
    content((0.8, -0.2), text(size: 6pt)[C (Yellow)])

    // Bullet connectors
    for y in (0.6, 0, -0.6) {
      circle((1.5, y), radius: 0.15, fill: rgb("#fbbf24"), stroke: 1pt + diagram-black)
      circle((2.2, y), radius: 0.15, fill: rgb("#fbbf24"), stroke: 1pt + diagram-black)
    }
    content((1.85, -1.2), text(size: 6pt)[4mm bullets])

    // Motor wires
    line((2.2, 0.6), (4, 0.6), stroke: 2pt + rgb("#3b82f6"))
    line((2.2, 0), (4, 0), stroke: 2pt + rgb("#22c55e"))
    line((2.2, -0.6), (4, -0.6), stroke: 2pt + rgb("#eab308"))

    // Motor
    circle((5, 0), radius: 1, stroke: 1.5pt + diagram-black, fill: diagram-light)
    content((5, 0), text(size: 7pt)[Motor])
  }),
  caption: [Phase wires connect VESC to motor via bullet connectors.],
)

#v(1em)

*Connection Notes:*

- Motor wire colors may not match VESC colors
- If motor spins wrong direction: swap any two phase wires
- Use 4mm gold bullet connectors (60A rated)
- Solder connections, use heat shrink
- Keep phase wires away from signal wires (EMI)

#v(1em)

*Wire Lengths:*
#spec-table(
  [*Motor Position*], [*Approx. Length*],
  [Front Left], [400mm],
  [Front Right], [500mm],
  [Rear Left], [300mm],
  [Rear Right], [400mm],
)

#note[
  Add 50mm extra for service loops. Too tight = strain on connectors.
]

#pagebreak()

// =============================================================================

= Signal Wiring

Connect low-voltage signals: USB, GPIO, sensors.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson
    rect((-5, -2), (-2, 2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((-3.5, 1.5), text(size: 8pt, weight: "bold")[Jetson])

    // USB ports
    for (i, label) in ((0.8, "USB-CAN"), (0.2, "Camera"), (-0.4, "LTE"), (-1, "USB Hub")) {
      rect((-2, i - 0.2), (-1.5, i + 0.2), fill: diagram-gray, stroke: 0.5pt + diagram-black)
      line((-1.5, i), (0, i), stroke: 1pt + diagram-black)
      content((1.5, i), text(size: 6pt)[#label])
    }

    // GPIO
    rect((-2, -1.6), (-1.5, -1.2), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    line((-1.5, -1.4), (0, -1.4), stroke: 1pt + muni-danger)
    content((1.5, -1.4), text(size: 6pt)[E-Stop GPIO])

    // Ethernet
    rect((-5, -1.6), (-4.5, -1.2), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    line((-4.5, -1.4), (-6, -1.4), stroke: 1pt + diagram-black)
    content((-6.8, -1.4), text(size: 6pt)[LiDAR])
  }),
  caption: [Jetson connections. USB for peripherals, GPIO for E-Stop, Ethernet for LiDAR.],
)

#v(1em)

*USB Allocation:*
#spec-table(
  [*Port*], [*Device*], [*Cable*],
  [USB 3.0 #1], [USB-CAN adapter], [USB-A to adapter],
  [USB 3.0 #2], [USB Hub], [USB-A to hub],
  [Hub Port 1], [Insta360 X4], [USB-C],
  [Hub Port 2], [LTE modem], [USB-A],
)

*GPIO:*
- Pin for E-Stop relay control
- Active-high: GPIO high = relay closed = power on
- On Jetson startup: default low = safe state

#pagebreak()

// =============================================================================

= Cable Management

Organize and secure all wiring for reliability and serviceability.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Frame cross-section
    rect((-5, 0), (5, 3), stroke: 1pt + diagram-black, fill: none)
    content((0, 3.3), text(size: 7pt)[Frame interior (top view)])

    // Cable runs
    // Power (thick, orange)
    line((-4, 0.5), (4, 0.5), stroke: 3pt + diagram-accent)
    content((-4.5, 0.5), text(size: 5pt)[48V])

    // 12V (blue)
    line((-4, 1), (4, 1), stroke: 2pt + rgb("#3b82f6"))
    content((-4.5, 1), text(size: 5pt)[12V])

    // CAN (thin, black)
    line((-4, 1.5), (4, 1.5), stroke: 1.5pt + diagram-black)
    content((-4.5, 1.5), text(size: 5pt)[CAN])

    // USB (gray)
    line((-4, 2), (2, 2), stroke: 1pt + diagram-gray)
    content((-4.5, 2), text(size: 5pt)[USB])

    // Separation note
    dim-v(5.5, 0.5, 1.5, "sep", offset: 0.3)

    // Cable ties
    for x in (-3, -1, 1, 3) {
      line((x, 0.3), (x, 2.2), stroke: 0.5pt + diagram-gray)
      content((x, 2.5), text(size: 4pt)[tie])
    }
  }),
  caption: [Route power and signal cables separately. Secure every 150mm.],
)

#v(1em)

*Routing Rules:*

- Separate power (48V) from signals by at least 25mm
- CAN bus twisted pair reduces interference
- Use cable ties every 100-150mm
- Leave service loops at connectors
- Label both ends of each cable

*Cable Tie Points:*
- Frame corners
- Near each connector
- Before/after bends
- At entry to electronics bay

#v(1em)

#checklist(
  [No cables in wheel path],
  [No cables near hot components (VESCs)],
  [All connectors accessible],
  [Service loops at key points],
  [Labels on power cables],
)

#pagebreak()
