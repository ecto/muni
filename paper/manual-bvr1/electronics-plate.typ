#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Electronics Plate Section (BVR1)
// Layout, Drilling, Mounting

= Electronics Plate

All the brains live on one removable plate. The Jetson compute module, four VESC motor controllers, DC-DC converter, and fusing all mount here. When something goes wrong (and eventually it will), you can unbolt four screws, slide the plate out, and work on it at a bench.

The layout is designed for airflow and serviceability. VESCs go near the edges where they can radiate heat. The Jetson sits in the middle with space around it for convection. Connectors face outward so you can plug and unplug without removing the plate.

= Electronics Plate Layout

#procedure([Reference: plate fabrication], time: "outsource or 1 hr", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Plate outline
    rect((-6, -3.5), (6, 3.5), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)

    // Dimensions
    dim-h(-4, -6, 6, "300", offset: 0.8)
    dim-v(6, -3.5, 3.5, "200", offset: 0.8)

    // Mounting holes in corners
    for (x, y) in ((-5.5, 3), (5.5, 3), (-5.5, -3), (5.5, -3)) {
      circle((x, y), radius: 0.2, fill: white, stroke: 1pt + diagram-black)
      content((x, y), text(size: 5pt)[5.2])
    }

    // Jetson footprint
    rect((-5, 0.5), (-1.5, 3), stroke: 1pt + diagram-gray, fill: none)
    content((-3.25, 1.75), text(size: 7pt)[Jetson Orin NX])
    // Jetson mounting holes
    for (x, y) in ((-4.7, 0.8), (-1.8, 0.8), (-4.7, 2.7), (-1.8, 2.7)) {
      circle((x, y), radius: 0.12, fill: white, stroke: 0.5pt + diagram-gray)
    }
    callout-leader((-3.25, 1.75), (-7, 2), "1")

    // VESCs footprint
    for i in range(4) {
      let x = -4.5 + i * 2.5
      rect((x, -2.8), (x + 2, -0.8), stroke: 1pt + diagram-gray, fill: none)
      content((x + 1, -1.8), text(size: 6pt)[VESC #(i+1)])
      // VESC mounting holes
      circle((x + 0.3, -2.5), radius: 0.1, fill: white, stroke: 0.5pt + diagram-gray)
      circle((x + 1.7, -2.5), radius: 0.1, fill: white, stroke: 0.5pt + diagram-gray)
      circle((x + 0.3, -1.1), radius: 0.1, fill: white, stroke: 0.5pt + diagram-gray)
      circle((x + 1.7, -1.1), radius: 0.1, fill: white, stroke: 0.5pt + diagram-gray)
    }
    callout-leader((-2.5, -1.8), (-7, -2), "2")

    // DC-DC
    rect((2, 0.5), (4, 2), stroke: 1pt + diagram-gray, fill: none)
    content((3, 1.25), text(size: 6pt)[DC-DC])
    callout-leader((3, 1.25), (7, 1), "3")

    // Fuse holder
    rect((4.5, 0.5), (5.5, 2), stroke: 1pt + diagram-gray, fill: none)
    content((5, 1.25), text(size: 5pt)[Fuse])
    callout-leader((5, 1.25), (7, 2.5), "4")

    // USB hub area
    rect((0.5, 1), (1.8, 2.5), stroke: 0.5pt + diagram-gray, fill: none)
    content((1.15, 1.75), text(size: 5pt)[USB])
  }),
  caption: none,
)

#v(0.5em)

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
      [#text(fill: muni-orange, weight: "bold")[2]], [VESC 6.7 ×4 (60×40mm each)],
      [#text(fill: muni-orange, weight: "bold")[3]], [DC-DC 48V→12V],
      [#text(fill: muni-orange, weight: "bold")[4]], [100A fuse holder],
    )
  ],
  [
    *Plate Material:*
    - 6mm (1/4") 6061-T6 aluminum (recommended)
    - Or: 5mm acrylic (lighter, less heat dissipation)
    - Or: 3mm FR4/G10 (good insulator)

    #v(0.3em)
    *CAD File:* `bvr/cad/exports/electronics_plate.stl`
  ]
)

#pagebreak()

// =============================================================================

= Drilling Guide

#procedure([Drill mounting holes], time: "30 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Grid with coordinates
    // Origin at center of plate

    // Plate outline
    rect((-6, -4), (6, 4), stroke: 1pt + diagram-black, fill: diagram-light)

    // Grid lines
    for x in range(-5, 6) {
      line((x, -4), (x, 4), stroke: 0.25pt + diagram-gray)
    }
    for y in range(-3, 4) {
      line((-6, y), (6, y), stroke: 0.25pt + diagram-gray)
    }

    // Origin marker
    circle((0, 0), radius: 0.1, fill: muni-orange)
    content((0.5, 0.3), text(size: 5pt, fill: muni-orange)[0,0])

    // Corner mounting holes (5.2mm for M5 clearance)
    let corners = ((-5.5, 3.5), (5.5, 3.5), (-5.5, -3.5), (5.5, -3.5))
    for (x, y) in corners {
      circle((x, y), radius: 0.25, fill: white, stroke: 1.5pt + diagram-black)
    }

    // Jetson holes (M3, 3.2mm)
    let jetson_holes = ((-4.5, 2.5), (-2, 2.5), (-4.5, 1), (-2, 1))
    for (x, y) in jetson_holes {
      circle((x, y), radius: 0.15, fill: white, stroke: 1pt + muni-orange)
    }

    // VESC holes (M3, 3.2mm) - just show pattern for one
    let vesc_pattern = ((0.3, 0.3), (1.7, 0.3), (0.3, 1.2), (1.7, 1.2))
    for (dx, dy) in vesc_pattern {
      // VESC 1
      circle((-4.7 + dx, -2.8 + dy), radius: 0.12, fill: white, stroke: 1pt + diagram-gray)
    }

    // Legend
    circle((-5, -5.5), radius: 0.25, fill: white, stroke: 1.5pt + diagram-black)
    content((-3.5, -5.5), text(size: 6pt)[5.2mm (M5 clearance)])

    circle((-0.5, -5.5), radius: 0.15, fill: white, stroke: 1pt + muni-orange)
    content((1.2, -5.5), text(size: 6pt)[3.2mm (M3 clearance)])

    // Scale
    dim-h(-6.5, -1, 0, "10mm grid", offset: 0)
  }),
  caption: [Hole positions. Grid squares = 10mm. Origin at plate center.],
)

#v(1em)

*Drill Sizes:*
#spec-table(
  [*Hole Type*], [*Drill Size*], [*Purpose*],
  [M5 clearance], [5.2mm], [Plate mounting to frame],
  [M3 clearance], [3.2mm], [Electronics mounting],
  [M3 tap], [2.5mm], [If threading aluminum],
  [M4 clearance], [4.2mm], [Larger components],
)

#pagebreak()

// =============================================================================

= Plate Mounting

#procedure([Mount plate to frame], time: "15 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Cross-section view of plate mounting

    // Extrusion (cross section)
    extrusion-end((0, 0), size: 1.2)
    content((0, -1.5), text(size: 7pt)[2020 Extrusion])

    // T-nut in slot
    rect((-0.15, 0.4), (0.15, 0.6), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    content((0.8, 0.5), text(size: 5pt)[T-Nut])

    // Bolt coming down
    line((0, 0.6), (0, 1.5), stroke: 1.5pt + diagram-black)
    rect((-0.2, 1.5), (0.2, 1.7), fill: diagram-black)
    content((0.8, 1.6), text(size: 5pt)[M5×16])

    // Standoff
    rect((-0.15, 1.7), (0.15, 2.5), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0.8, 2.1), text(size: 5pt)[Standoff])

    // Plate
    rect((-1.5, 2.5), (1.5, 2.7), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 3), text(size: 7pt)[Electronics Plate])

    // Dimension
    dim-v(-1, 1.7, 2.5, "15-25", offset: 0.3)
  }),
  caption: [Cross-section: standoff mounting provides airflow under plate.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Mounting Hardware (per corner):*
    - 1× M5×16 or M5×20 bolt
    - 1× M5 T-nut (drop-in or slide-in)
    - 1× M5 standoff (15-25mm height)
    - 1× M5 nut or second standoff
  ],
  [
    *Standoff Height:*
    - 15mm: Minimal, tight fit
    - 20mm: Recommended (good airflow)
    - 25mm: Maximum cable clearance

    Use same height at all 4 corners.
  ]
)

#v(1em)

*Installation:*
+ Insert T-nuts into top extrusion slots
+ Thread M5 bolts through standoffs
+ Position plate on standoffs
+ Align with T-nuts
+ Tighten to 4 Nm

#note[
  Leave plate loose until all electronics are mounted. Easier access.
]

#pagebreak()
