#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Tools & Materials Section
// Required tools, Parts list, Hardware reference

= Required Tools

#figure(
  cetz.canvas({
    import cetz.draw: *

    tool-hex-key((-5, 0), size: 1.8)
    content((-5, -2), text(size: 9pt, weight: "bold")[Hex Keys])
    content((-5, -2.6), text(size: 7pt, fill: diagram-gray)[2.5, 3, 4, 5 mm])

    tool-screwdriver((-2, 0), size: 1.8, tip: "phillips")
    content((-2, -2), text(size: 9pt, weight: "bold")[Screwdriver])
    content((-2, -2.6), text(size: 7pt, fill: diagram-gray)[Phillips #2])

    tool-wrench((1, 0), size: 1.8)
    content((1, -2), text(size: 9pt, weight: "bold")[Wrenches])
    content((1, -2.6), text(size: 7pt, fill: diagram-gray)[8, 10, 13 mm])

    tool-multimeter((4, 0), size: 1.6)
    content((4, -2), text(size: 9pt, weight: "bold")[Multimeter])
    content((4, -2.6), text(size: 7pt, fill: diagram-gray)[V / Ω / Continuity])
  }),
  caption: none,
)

#v(2em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Required*
    - Hex key set (metric: 2.5, 3, 4, 5 mm)
    - Phillips screwdriver (#2)
    - Adjustable wrench or socket set (8, 10, 13 mm)
    - Multimeter (voltage, resistance, continuity)
    - Wire strippers (20-12 AWG)
    - Soldering iron (40W+) and solder
    - Heat shrink assortment
    - Miter saw or hacksaw (for extrusions)
  ],
  [
    *Recommended*
    - Torque wrench (4 Nm for M5)
    - Drill and drill bits (3.2, 4.2, 5 mm)
    - Tap set (M4×0.7, M5×0.8)
    - Deburring tool
    - Cable ties (assorted sizes)
    - Label maker
    - Work mat
    - Helping hands / PCB holder
  ]
)

#v(1em)

#note[
  All M5 bolts should be torqued to 4 Nm. Over-tightening can strip aluminum threads.
]

#pagebreak()

// =============================================================================

= Parts List

#figure(
  cetz.canvas({
    import cetz.draw: *

    for i in range(4) {
      rect((-6 + i * 0.4, 3.5), (-5.7 + i * 0.4, 5.5), fill: diagram-light, stroke: 0.75pt + diagram-black)
    }
    callout((-4.5, 5.5), "A")

    for i in range(2) {
      for j in range(2) {
        circle((-2 + i * 1.5, 4 + j * 1.2), radius: 0.5, stroke: 1pt + diagram-black, fill: diagram-light)
      }
    }
    callout((0, 5.5), "B")

    jetson-top((3, 4.5), size: (1.2, 0.8))
    callout((4.5, 5.5), "C")

    lidar-top((-5, 1), size: 0.5)
    camera-top((-3, 1), radius: 0.3)
    callout((-4, 2.2), "D")

    battery-top((0, 1), size: (1.5, 0.8))
    callout((1.5, 2.2), "E")

    screw-actual-size((4, 1.2), thread: "M5", length: 8)
    tnut-side((5, 1.2), size: 0.25)
    callout((4.5, 2.2), "F")
  }),
  caption: none,
)

#v(-0.5em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 1.5em,
  [
    #text(weight: "bold", size: 9pt)[Parts]
    #table(
      columns: (auto, 1fr, auto),
      stroke: none,
      inset: 3pt,
      [#text(fill: muni-orange, weight: "bold")[A]], [Chassis: extrusions, brackets, plate], [\$150],
      [#text(fill: muni-orange, weight: "bold")[B]], [Drivetrain: motors, VESCs, mounts], [\$800],
      [#text(fill: muni-orange, weight: "bold")[C]], [Electronics: Jetson, CAN, LTE], [\$900],
      [#text(fill: muni-orange, weight: "bold")[D]], [Perception: LiDAR, camera, pole], [\$1,800],
      [#text(fill: muni-orange, weight: "bold")[E]], [Power: battery, DC-DC, fuse, E-stop], [\$400],
      [#text(fill: muni-orange, weight: "bold")[F]], [Hardware: bolts, T-nuts, wire], [\$100],
    )
  ],
  [
    #text(weight: "bold", size: 9pt)[Cost Summary]
    #table(
      columns: (1fr, auto),
      stroke: none,
      inset: 3pt,
      [Chassis], [\$150],
      [Drivetrain], [\$800],
      [Electronics], [\$900],
      [Perception], [\$1,800],
      [Power], [\$400],
      [Hardware], [\$100],
      table.hline(stroke: 0.5pt),
      [*Total*], [*\$4,150*],
    )

    #v(0.5em)
    #text(size: 7pt, fill: gray)[All parts commercially available.]
  ]
)

#pagebreak()

// =============================================================================

= Hardware Reference

Standard fasteners and hardware used throughout the build.

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Bolts*
    #spec-table(
      [*Size*], [*Use*],
      [M3×8], [Electronics mounting],
      [M4×8], [Motor to bracket],
      [M5×8], [T-nut, light duty],
      [M5×10], [T-nut, standard],
      [M5×16], [T-nut, through plate],
      [M6×12], [Motor bracket to frame],
    )

    #v(1em)

    *T-Nuts*
    #spec-table(
      [*Type*], [*Use*],
      [M5 drop-in], [Post-assembly insertion],
      [M5 slide-in], [Pre-assembly (easier)],
      [M6 drop-in], [Heavy-duty mounts],
    )
  ],
  [
    *Connectors*
    #spec-table(
      [*Type*], [*Rating*], [*Use*],
      [XT90], [90A], [Battery main],
      [XT60], [60A], [Motor phase],
      [XT30], [30A], [12V power],
      [JST-PH], [3A], [CAN bus, signals],
      [DT 4-pin], [25A], [Tool connector],
    )

    #v(1em)

    *Wire Gauge*
    #spec-table(
      [*AWG*], [*Use*],
      [8 AWG], [Battery to bus],
      [10 AWG], [Bus to VESCs],
      [14 AWG], [12V power],
      [22 AWG], [CAN bus, signals],
    )
  ]
)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Visual reference for common hardware
    content((-5, 2), text(size: 8pt, weight: "bold")[Common Hardware (actual size)])

    screw-actual-size((-5, 0), thread: "M5", length: 8)
    content((-5, -1), text(size: 6pt)[M5×8])

    screw-actual-size((-2.5, 0), thread: "M5", length: 10)
    content((-2.5, -1), text(size: 6pt)[M5×10])

    screw-actual-size((0, 0), thread: "M5", length: 16)
    content((0, -1), text(size: 6pt)[M5×16])

    tnut-side((3, 0), size: 0.4)
    content((3, -1), text(size: 6pt)[M5 T-Nut])

    corner-bracket((5.5, 0), size: 0.5)
    content((5.5, -1), text(size: 6pt)[Corner])
  }),
  caption: none,
)

#pagebreak()
