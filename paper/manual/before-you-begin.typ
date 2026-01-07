#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Before You Begin
// Build overview, prerequisites, timeline

= Before You Begin

This section explains what you're building, what you'll need, and how long it takes.

== What You're Building

The BVR0 is a four-wheeled skid-steer rover with hub motors, a 48V power system, and onboard compute. It's designed for outdoor municipal work (snow clearing, mapping, patrol) but the base platform is general-purpose.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Simple block diagram
    rect((-4, 0), (4, 2), stroke: 1pt + diagram-black, radius: 4pt)
    content((0, 1), text(size: 9pt, weight: "bold")[BVR0])

    // Subsystems
    let subs = (
      (-3, -1.5, "Frame"),
      (-1, -1.5, "Power"),
      (1, -1.5, "Drive"),
      (3, -1.5, "Compute"),
    )
    for (x, y, label) in subs {
      rect((x - 0.8, y - 0.4), (x + 0.8, y + 0.4), stroke: 0.5pt + diagram-gray, radius: 2pt)
      content((x, y), text(size: 7pt)[#label])
      line((x, 0), (x, y + 0.4), stroke: 0.5pt + diagram-gray)
    }
  }),
  caption: [Four subsystems: mechanical frame, power distribution, drivetrain, and compute/sensors.],
)

== Prerequisites

*Skills needed:*
- Basic hand tools (hex keys, screwdrivers, wrenches)
- Wire stripping and crimping
- Soldering (through-hole level)
- Comfort with Linux command line
- Ability to read wiring diagrams

*You do NOT need:*
- CNC or machining (parts are outsourced or hand-cut)
- PCB design (all boards are off-the-shelf)
- Deep embedded programming (firmware is pre-built)

== Build Phases

The build has four phases. Complete each before starting the next.

#figure(
  cetz.canvas({
    import cetz.draw: *

    let phases = (
      (0, "1. Mechanical", "4-6 hours", "Frame, brackets,\nmounting"),
      (4, "2. Power", "2-3 hours", "Battery, fuse,\ndistribution"),
      (8, "3. Electronics", "3-4 hours", "VESCs, Jetson,\nwiring"),
      (12, "4. Software", "2-3 hours", "Flash, config,\ntest"),
    )

    for (x, title, time, desc) in phases {
      rect((x - 1.5, -1), (x + 1.5, 1), stroke: 1pt + diagram-black, radius: 4pt, fill: diagram-light)
      content((x, 0.5), text(size: 8pt, weight: "bold")[#title])
      content((x, -0.3), text(size: 6pt, fill: muni-gray)[#time])
    }

    // Arrows between phases
    for i in range(3) {
      let x = i * 4 + 1.5
      motion-arrow((x + 0.2, 0), (x + 0.8, 0))
    }

    // Total
    content((6, -2), text(size: 9pt)[*Total: 12-16 hours* (split across 2-3 days recommended)])
  }),
  caption: none,
)

#pagebreak()

== Recommended Order

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Day*], [*Tasks*], [*Time*],
  [1], [Cut extrusions, assemble frame, mount motor brackets], [4-5 hr],
  [2], [Install motors, wire power system, mount electronics plate], [4-5 hr],
  [3], [Wire VESCs and CAN bus, flash Jetson, configure and test], [4-5 hr],
)

#v(1em)

#tip[
  Don't rush. A clean build with good cable management saves hours of debugging later.
]

== Tools Required

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Essential:*
    - Hex key set (2, 2.5, 3, 4, 5 mm)
    - Phillips screwdriver
    - Wire strippers (10-22 AWG)
    - Crimping tool (for ferrules)
    - Soldering iron + solder
    - Multimeter
    - Heat gun or lighter (heat shrink)
  ],
  [
    *Helpful:*
    - Torque wrench (4 Nm range)
    - Miter saw or hacksaw (for extrusions)
    - Deburring tool
    - Cable tie gun
    - Label maker
    - Helping hands (for soldering)
  ]
)

== Materials Checklist

Before starting, verify you have:

#checklist(
  [All BOM items received and inspected],
  [Extrusions cut to length (or stock to cut)],
  [Motor brackets fabricated (or ordered)],
  [Electronics plate fabricated (or ordered)],
  [Battery charged to 50%],
  [Jetson flashed with JetPack],
)

#v(1em)

See *Appendix A: Bill of Materials* for the complete parts list with vendors.

#pagebreak()
