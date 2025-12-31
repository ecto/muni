#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Power System Section
// Battery, Fuse, DC-DC, Distribution

= Battery Tray

Secure mounting for the 48V battery pack.

#v(1em)

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

    // Battery
    rect((-3.5, 0.7), (3.5, 2.2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
    content((0, 1.45), text(size: 8pt, weight: "bold")[48V 20Ah])
    content((0, 1), text(size: 6pt)[960 Wh])

    // Retention strap
    line((-3.5, 2.2), (-3.5, 2.6), stroke: 1.5pt + muni-orange)
    line((-3.5, 2.6), (3.5, 2.6), stroke: 1.5pt + muni-orange)
    line((3.5, 2.6), (3.5, 2.2), stroke: 1.5pt + muni-orange)
    content((0, 2.9), text(size: 6pt, fill: muni-orange)[Retention strap])

    // XT90 connector
    connector-xt((4.5, 1.45), size: "90")
    content((5.5, 1.45), text(size: 6pt)[XT90])

    // Dimension
    dim-h(-5.5, -3.5, 3.5, "180mm", offset: 0.3)
  }),
  caption: [Battery mounted on tray with retention strap. XT90 connector for quick disconnect.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Tray Construction:*
    - Material: 2mm 6061-T6 aluminum
    - Flat size: 220mm × 160mm
    - Bend: 15mm lip on all 4 sides (90°)
    - Final inside: 190mm × 130mm × 15mm deep
    - 10mm EVA foam padding (bottom)
    - 2× 10mm cable routing holes

    #v(0.3em)
    *CAD File:* `bvr/cad/battery-tray.dxf`
  ],
  [
    *Retention Requirements:*
    - Secure in all axes
    - Quick-release for service
    - Must hold during tip-over
    - Vibration dampening (10mm EVA foam)

    *Strap:*
    - 25mm nylon webbing
    - Cam buckle (not ratchet)
    - Route over battery, through frame slots
  ]
)

#v(1em)

#warning[
  Battery must not shift during operation. Loose batteries can short on frame, causing fire.
]

#pagebreak()

// =============================================================================

= Fuse and E-Stop

Install overcurrent protection and emergency disconnect.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Power path diagram
    // Battery
    battery-top((-5, 0), size: (1.5, 0.8))

    // XT90
    connector-xt((-3, 0), size: "90")
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
  }),
  caption: [Power flows: Battery → XT90 → Fuse → E-Stop relay → Power bus],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *#text(fill: muni-orange)[1] Fuse (100A):*
    - ANL or MIDI style fuse
    - Inline fuse holder with ring terminals
    - Mount accessible for replacement
    - Size: protects wiring, not electronics
  ],
  [
    *#text(fill: muni-orange)[2] E-Stop Relay:*
    - Normally-open contactor (closes when safe)
    - 12V coil, controlled by Jetson GPIO
    - 100A+ contact rating
    - Fails safe: power loss = stop
  ]
)

#v(1em)

*Wiring:*
- Use 8 AWG wire for main power path
- Ring terminals with heat shrink
- Keep runs short between fuse and relay

#pagebreak()

// =============================================================================

= DC-DC Converter

Step down 48V main power to 12V for electronics.

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

Main power bus connects battery to all high-current loads.

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
    - XT90 to 4× XT60
    - Simpler for prototypes
    - Higher resistance
  ]
)

#pagebreak()
