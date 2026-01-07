#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Overview Section
// What is BVR0, Specifications

= Overview

The BVR0 is the first rover in the Muni fleet. It's intentionally simple: a rigid aluminum frame, four hub motors, and enough compute to handle autonomy. No suspension, no steering linkages, no complex mechanisms. Everything that can break has been removed.

The design philosophy is "municipal-grade": it needs to survive Cleveland winters, sidewalk salt, and the occasional collision with a park bench. The 2020 aluminum extrusion frame can be rebuilt with hardware store parts. The hub motors are the same units used in hoverboards and e-scooters (proven, cheap, replaceable). The electronics are mounted on a single plate that slides out for service.

#figure(
  cetz.canvas({
    import cetz.draw: *

    let tx = -5

    rect((tx - 2.5, -2.5), (tx + 2.5, 2.5), stroke: 1.5pt + diagram-black, radius: 4pt)

    for (x, y) in ((tx - 2.7, 2), (tx + 2.7, 2), (tx - 2.7, -2), (tx + 2.7, -2)) {
      rect((x - 0.4, y - 0.6), (x + 0.4, y + 0.6), fill: diagram-black, radius: 2pt)
    }

    rect((tx - 1.8, -1.8), (tx + 1.8, 0.8), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 2pt)
    rect((tx - 1.2, 2.1), (tx + 1.2, 2.5), fill: diagram-light, stroke: 0.5pt + diagram-gray)
    circle((tx, 1.2), radius: 0.25, fill: diagram-black)

    dim-h(-2.5, tx - 2.5, tx + 2.5, "600", offset: 1.2)
    dim-v(tx + 2.5, -2.5, 2.5, "600", offset: 1.2)

    motion-arrow((tx, 3), (tx, 3.8))
    content((tx, 4.1), text(size: 6pt)[FRONT])
    content((tx, -4), text(size: 8pt, weight: "bold")[TOP VIEW])

    let sx = 4

    line((sx - 3.5, -2), (sx + 3.5, -2), stroke: 0.5pt + diagram-gray)
    rect((sx - 2.5, -1.5), (sx + 2.5, -0.3), stroke: 1.5pt + diagram-black, radius: 2pt)

    circle((sx - 2, -2), radius: 0.6, stroke: 1.5pt + diagram-black, fill: diagram-light)
    circle((sx + 2, -2), radius: 0.6, stroke: 1.5pt + diagram-black, fill: diagram-light)

    line((sx, -0.3), (sx, 2.5), stroke: 1.5pt + diagram-black)
    rect((sx - 0.35, 1.8), (sx + 0.35, 2.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    circle((sx, 2.7), radius: 0.2, fill: diagram-black)
    rect((sx + 2.2, -1.2), (sx + 3, -0.6), fill: diagram-light, stroke: 0.5pt + diagram-gray)

    dim-v(sx + 3.2, -2, 2.7, "700", offset: 0.3)
    content((sx, -4), text(size: 8pt, weight: "bold")[SIDE VIEW])

    callout-leader((tx - 2.7, 2), (-9, 3), "1")
    callout-leader((tx, -0.5), (-9, -1), "2")
    callout-leader((tx, 2.3), (-9, 4.5), "3")
    callout-leader((sx, 2.7), (8, 3.5), "4")
    callout-leader((sx, 2), (8, 2), "5")
    callout-leader((sx + 2.6, -0.9), (8, 0), "6")
  }),
  caption: none,
)

#v(-0.5em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    #text(weight: "bold", size: 9pt)[Components]
    #table(
      columns: (auto, 1fr),
      stroke: none,
      inset: 4pt,
      [#text(fill: muni-orange, weight: "bold")[1]], [Hub motor wheels (×4)],
      [#text(fill: muni-orange, weight: "bold")[2]], [Electronics bay],
      [#text(fill: muni-orange, weight: "bold")[3]], [Tool mount],
      [#text(fill: muni-orange, weight: "bold")[4]], [360° camera],
      [#text(fill: muni-orange, weight: "bold")[5]], [LiDAR sensor],
      [#text(fill: muni-orange, weight: "bold")[6]], [Tool attachment],
    )
  ],
  [
    #text(weight: "bold", size: 9pt)[Key Specifications]
    #table(
      columns: (1fr, auto),
      stroke: none,
      inset: 4pt,
      [Dimensions], [600 × 600 × 700 mm],
      [Weight], [~30 kg with battery],
      [Speed], [1.0–2.5 m/s],
      [Runtime], [~4 hours],
      [Temp range], [-20°C to +40°C],
    )
  ]
)

#pagebreak()

// =============================================================================

= Specifications

These are the target specifications for a standard BVR0 build. Your rover may vary slightly depending on component sourcing and local modifications.

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Mechanical*
    #spec-table(
      [Footprint], [600 × 600 mm],
      [Height], [700 mm (with mast)],
      [Weight], [~30 kg],
      [Ground clearance], [50 mm],
      [Wheel diameter], [160 mm],
      [Frame], [2020 aluminum extrusion],
    )

    #v(1em)

    *Electrical*
    #spec-table(
      [Main battery], [48V 20Ah (960 Wh)],
      [Chemistry], [13S LiPo],
      [Voltage range], [39–54.6V],
      [Accessory rail], [12V 10A],
      [Main fuse], [100A],
    )
  ],
  [
    *Drivetrain*
    #spec-table(
      [Motors], [4× 350W hub motors],
      [Controllers], [4× VESC 6.7],
      [Drive type], [Skid-steer],
      [Max speed], [2.5 m/s],
      [Cruise speed], [1.0 m/s],
    )

    #v(1em)

    *Perception*
    #spec-table(
      [LiDAR], [Livox Mid-360],
      [Camera], [Insta360 X4 (360°)],
      [GPS], [RTK-capable (optional)],
    )

    #v(1em)

    *Compute*
    #spec-table(
      [Main computer], [Jetson Orin NX 16GB],
      [Connectivity], [LTE + WiFi],
      [CAN bus], [500K baud],
    )
  ]
)

#pagebreak()
