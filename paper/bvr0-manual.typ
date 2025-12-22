// BVR0 Technical Manual
// Base Vectoring Rover - Revision 0

#import "lib/template.typ": *
#import "lib/diagrams.typ": *
#import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge

#show: manual.with(
  title: "BVR0",
  subtitle: "Base Vectoring Rover",
  revision: "0.1",
  date: "December 2025",
  doc-type: "Technical Manual",
  cover-image: "../images/bvr0-disassembled.jpg",
)

// =============================================================================
= Overview
// =============================================================================

// Large annotated diagram - this IS the overview
#figure(
  cetz.canvas({
    import cetz.draw: *

    // === TOP VIEW (left side) ===
    let tx = -5  // top view center x

    // Chassis
    rect((tx - 2.5, -2.5), (tx + 2.5, 2.5), stroke: 1.5pt + diagram-black, radius: 4pt)

    // Wheels
    for (x, y) in ((tx - 2.7, 2), (tx + 2.7, 2), (tx - 2.7, -2), (tx + 2.7, -2)) {
      rect((x - 0.4, y - 0.6), (x + 0.4, y + 0.6), fill: diagram-black, radius: 2pt)
    }

    // Electronics bay
    rect((tx - 1.8, -1.8), (tx + 1.8, 0.8), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 2pt)

    // Tool mount
    rect((tx - 1.2, 2.1), (tx + 1.2, 2.5), fill: diagram-light, stroke: 0.5pt + diagram-gray)

    // Sensor mast
    circle((tx, 1.2), radius: 0.25, fill: diagram-black)

    // Dimensions
    dim-h(-2.5, tx - 2.5, tx + 2.5, "600", offset: 1.2)
    dim-v(tx + 2.5, -2.5, 2.5, "600", offset: 1.2)

    // Front indicator
    motion-arrow((tx, 3), (tx, 3.8))
    content((tx, 4.1), text(size: 6pt)[FRONT])

    // Label
    content((tx, -4), text(size: 8pt, weight: "bold")[TOP VIEW])

    // === SIDE VIEW (right side) ===
    let sx = 4  // side view center x

    // Ground line
    line((sx - 3.5, -2), (sx + 3.5, -2), stroke: 0.5pt + diagram-gray)

    // Chassis body
    rect((sx - 2.5, -1.5), (sx + 2.5, -0.3), stroke: 1.5pt + diagram-black, radius: 2pt)

    // Wheels
    circle((sx - 2, -2), radius: 0.6, stroke: 1.5pt + diagram-black, fill: diagram-light)
    circle((sx + 2, -2), radius: 0.6, stroke: 1.5pt + diagram-black, fill: diagram-light)

    // Sensor mast
    line((sx, -0.3), (sx, 2.5), stroke: 1.5pt + diagram-black)

    // LiDAR
    rect((sx - 0.35, 1.8), (sx + 0.35, 2.2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)

    // Camera
    circle((sx, 2.7), radius: 0.2, fill: diagram-black)

    // Tool mount
    rect((sx + 2.2, -1.2), (sx + 3, -0.6), fill: diagram-light, stroke: 0.5pt + diagram-gray)

    // Height dimension
    dim-v(sx + 3.2, -2, 2.7, "700", offset: 0.3)

    // Ground clearance
    line((sx - 1, -2), (sx - 1, -1.5), stroke: 0.5pt + diagram-gray)
    content((sx - 1.5, -1.75), text(size: 5pt)[50])

    // Label
    content((sx, -4), text(size: 8pt, weight: "bold")[SIDE VIEW])

    // === CALLOUTS ===
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
    // Component key
    #text(weight: "bold", size: 9pt)[Components]

    #table(
      columns: (auto, 1fr),
      stroke: none,
      inset: 4pt,
      [#text(fill: muni-orange, weight: "bold")[1]], [Hub motor wheels (×4) — 350W each],
      [#text(fill: muni-orange, weight: "bold")[2]], [Electronics bay — Jetson, VESCs, power],
      [#text(fill: muni-orange, weight: "bold")[3]], [Tool mount — quick-attach interface],
      [#text(fill: muni-orange, weight: "bold")[4]], [360° camera — Insta360 X4],
      [#text(fill: muni-orange, weight: "bold")[5]], [LiDAR — Livox Mid-360],
      [#text(fill: muni-orange, weight: "bold")[6]], [Tool attachment point],
    )
  ],
  [
    // Key specs
    #text(weight: "bold", size: 9pt)[Specifications]

    #table(
      columns: (1fr, auto),
      stroke: none,
      inset: 4pt,
      [Dimensions], [600 × 600 × 700 mm],
      [Weight], [~30 kg with battery],
      [Battery], [48V 20Ah (960 Wh)],
      [Motors], [4× 350W hub motors],
      [Speed], [1.0–2.5 m/s],
      [Runtime], [~4 hours],
      [Temp range], [-20°C to +40°C],
    )
  ]
)

#pagebreak()

// =============================================================================
= Bill of Materials
// =============================================================================

// Full parts layout diagram
#figure(
  cetz.canvas({
    import cetz.draw: *

    // === EXPLODED PARTS LAYOUT ===
    // All major components shown as if laid out on a table before assembly

    // --- CHASSIS (top left) ---
    let cx = -6
    let cy = 4

    // Extrusion pieces
    for i in range(4) {
      rect((cx - 2 + i * 0.4, cy - 0.1), (cx - 1.7 + i * 0.4, cy + 2),
           fill: diagram-light, stroke: 0.75pt + diagram-black)
    }
    content((cx - 0.8, cy + 1), text(size: 5pt)[×8])
    callout((cx + 0.5, cy + 2.3), "A")

    // Corner brackets
    for i in range(4) {
      corner-bracket((cx + 2 + i * 0.6, cy + 1), size: 0.4)
    }
    content((cx + 4.5, cy + 1), text(size: 5pt)[×16])

    // Electronics plate
    rect((cx - 1.5, cy - 1.5), (cx + 1.5, cy - 0.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((cx, cy - 1), text(size: 5pt)[Plate])

    // --- DRIVETRAIN (top right) ---
    let dx = 3
    let dy = 4

    // Hub motors
    for i in range(2) {
      for j in range(2) {
        circle((dx + i * 1.8, dy + j * 1.5), radius: 0.6, stroke: 1pt + diagram-black, fill: diagram-light)
        circle((dx + i * 1.8, dy + j * 1.5), radius: 0.2, fill: diagram-black)
      }
    }
    content((dx + 0.9, dy - 0.5), text(size: 5pt)[Hub Motors ×4])
    callout((dx + 2.5, dy + 2.3), "B")

    // VESCs
    for i in range(4) {
      vesc-top((dx + 4 + i * 0.8, dy + 0.5 + calc.rem(i, 2) * 1), size: (0.6, 0.4), id: none)
    }
    content((dx + 5.6, dy - 0.3), text(size: 5pt)[VESC ×4])

    // --- ELECTRONICS (middle left) ---
    let ex = -5
    let ey = 0

    // Jetson
    jetson-top((ex, ey), size: (1.5, 1))
    callout((ex - 1.2, ey + 0.8), "C")

    // USB CAN
    rect((ex + 2, ey - 0.3), (ex + 2.8, ey + 0.3), fill: diagram-light, stroke: 0.75pt + diagram-black, radius: 2pt)
    content((ex + 2.4, ey), text(size: 4pt)[CAN])

    // LTE modem
    rect((ex + 3.2, ey - 0.3), (ex + 4.2, ey + 0.3), fill: diagram-light, stroke: 0.75pt + diagram-black, radius: 2pt)
    content((ex + 3.7, ey), text(size: 4pt)[LTE])

    // --- PERCEPTION (middle right) ---
    let px = 3
    let py = 0

    // LiDAR
    lidar-top((px, py), size: 0.6)
    content((px, py - 1), text(size: 5pt)[Mid-360])
    callout((px - 0.8, py + 0.8), "D")

    // Camera
    camera-top((px + 2.5, py), radius: 0.4)
    content((px + 2.5, py - 0.8), text(size: 5pt)[X4])

    // Sensor pole
    rect((px + 4.5, py - 1), (px + 4.7, py + 1), fill: diagram-light, stroke: 0.75pt + diagram-black)
    content((px + 5.3, py), text(size: 5pt)[Pole])

    // --- POWER (bottom left) ---
    let bx = -5
    let by = -3.5

    // Battery
    battery-top((bx, by), size: (2, 1))
    callout((bx - 1.5, by + 0.8), "E")

    // DC-DC
    rect((bx + 2, by - 0.4), (bx + 3, by + 0.4), fill: diagram-light, stroke: 0.75pt + diagram-black, radius: 2pt)
    content((bx + 2.5, by), text(size: 4pt)[DC-DC])

    // Fuse
    rect((bx + 3.5, by - 0.2), (bx + 4.2, by + 0.2), fill: rgb("#fbbf24"), stroke: 0.75pt + diagram-black, radius: 2pt)
    content((bx + 3.85, by), text(size: 4pt)[100A])

    // E-Stop
    estop-symbol((bx + 5, by), size: 0.4)

    // --- CONNECTORS (bottom right) ---
    let wx = 3
    let wy = -3.5

    // XT90
    connector-xt((wx, wy), size: "90")
    content((wx, wy - 0.6), text(size: 5pt)[XT90])

    // XT30
    connector-xt((wx + 1.5, wy), size: "30")
    content((wx + 1.5, wy - 0.6), text(size: 5pt)[XT30])

    // DT connector
    connector-dt((wx + 3, wy), pins: 4)
    content((wx + 3, wy - 0.6), text(size: 5pt)[DT])

    // Hardware
    screw-actual-size((wx + 5, wy + 0.3), thread: "M5", length: 10)
    tnut-side((wx + 6, wy + 0.3), size: 0.3)
    callout((wx + 5.5, wy + 1), "F")
  }),
  caption: none,
)

#v(-0.5em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 1.5em,
  [
    #text(weight: "bold", size: 9pt)[Parts Key]
    #table(
      columns: (auto, 1fr, auto),
      stroke: none,
      inset: 3pt,
      [#text(fill: muni-orange, weight: "bold")[A]], [Chassis: extrusions, brackets, plate], [\$150],
      [#text(fill: muni-orange, weight: "bold")[B]], [Drivetrain: motors, VESCs, mounts], [\$800],
      [#text(fill: muni-orange, weight: "bold")[C]], [Electronics: Jetson, CAN, LTE], [\$900],
      [#text(fill: muni-orange, weight: "bold")[D]], [Perception: LiDAR, camera, pole], [\$1,800],
      [#text(fill: muni-orange, weight: "bold")[E]], [Power: battery, DC-DC, fuse, E-stop], [\$400],
      [#text(fill: muni-orange, weight: "bold")[F]], [Hardware: bolts, T-nuts, wire, connectors], [\$100],
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
      [Hardware/Wiring], [\$100],
      [*Total*], [*\$4,150*],
    )

    #v(0.5em)
    #text(size: 7pt, fill: gray)[All parts commercially available. Custom fab limited to plate cutting.]
  ]
)

#pagebreak()

// =============================================================================
= Assembly
// =============================================================================

// Tools required - visual only
#figure(
  cetz.canvas({
    import cetz.draw: *

    // Section label
    content((0, 2.5), text(size: 10pt, weight: "bold")[Required Tools])

    // Hex keys
    tool-hex-key((-5, 0), size: 1.4)
    content((-5, -1.4), text(size: 8pt)[Hex Keys])
    content((-5, -1.9), text(size: 6pt, fill: diagram-gray)[2.5, 3, 4, 5mm])

    // Screwdriver
    tool-screwdriver((-2.5, 0), size: 1.4, tip: "phillips")
    content((-2.5, -1.4), text(size: 8pt)[Screwdriver])
    content((-2.5, -1.9), text(size: 6pt, fill: diagram-gray)[Phillips #2])

    // Wrench
    tool-wrench((0, 0), size: 1.4)
    content((0, -1.4), text(size: 8pt)[Wrenches])
    content((0, -1.9), text(size: 6pt, fill: diagram-gray)[8, 10, 13mm])

    // Multimeter
    tool-multimeter((2.5, 0), size: 1.2)
    content((2.5, -1.4), text(size: 8pt)[Multimeter])
    content((2.5, -1.9), text(size: 6pt, fill: diagram-gray)[V / Ω / Cont.])

    // Torque indicator
    torque-indicator((5.5, 0), value: "4 Nm", size: 1.4)
    content((5.5, -1.4), text(size: 8pt)[Torque])
    content((5.5, -1.9), text(size: 6pt, fill: diagram-gray)[All M5 bolts])
  }),
  caption: none,
)

#v(1em)

== Phase 1: Chassis Frame

#figure(
  cetz.canvas({
    import cetz.draw: *

    // === 4-STEP CHASSIS ASSEMBLY ===
    let pw = 3.2
    let ph = 2.8
    let gap = 0.4

    // Step 1: Cut extrusions
    step-panel((0, 0), size: (pw, ph), step-num: 1, title: "Cut")
    // Extrusions
    for i in range(4) {
      rect((0.3 + i * 0.5, 0.5), (0.5 + i * 0.5, 2.2), fill: diagram-light, stroke: 0.75pt + diagram-black)
    }
    content((1.5, 0.25), text(size: 5pt)[600mm × 8])

    panel-arrow-h((0, 0), from-size: (pw, ph), gap: gap)

    // Step 2: Base frame
    step-panel((pw + gap, 0), size: (pw, ph), step-num: 2, title: "Base")
    // Square frame top view
    rect((pw + gap + 0.5, 0.5), (pw + gap + 2.7, 2.2), stroke: 1.5pt + diagram-black)
    // Corner brackets
    for (x, y) in ((0.5, 0.5), (2.7, 0.5), (0.5, 2.2), (2.7, 2.2)) {
      corner-bracket((pw + gap + x, y), size: 0.3)
    }
    content((pw + gap + 1.6, 0.25), text(size: 5pt)[Check diagonals])

    panel-arrow-h((pw + gap, 0), from-size: (pw, ph), gap: gap)

    // Step 3: Verticals
    step-panel((2 * (pw + gap), 0), size: (pw, ph), step-num: 3, title: "Verticals")
    // Isometric-ish view
    let bx = 2 * (pw + gap) + 1.6
    let by = 1.2
    rect((bx - 1, by - 0.6), (bx + 1, by + 0.6), stroke: 1pt + diagram-black)
    // Vertical posts
    for (dx, dy) in ((-0.8, -0.4), (0.8, -0.4), (-0.8, 0.4), (0.8, 0.4)) {
      line((bx + dx, by + dy), (bx + dx + 0.15, by + dy + 0.8), stroke: 1.5pt + diagram-black)
    }
    content((bx, 0.25), text(size: 5pt)[4 corners])

    panel-arrow-h((2 * (pw + gap), 0), from-size: (pw, ph), gap: gap)

    // Step 4: Done - check mark
    step-panel((3 * (pw + gap), 0), size: (pw, ph), step-num: 4, title: "Verify")
    check-mark((3 * (pw + gap) + 1.6, 1.2), size: 0.6)
    content((3 * (pw + gap) + 1.6, 0.4), text(size: 5pt)[Square & rigid])
  }),
  caption: none,
)

== Phase 2: Motor Mounting

#figure(
  cetz.canvas({
    import cetz.draw: *

    // === EXPLODED VIEW: Motor Mount Assembly ===

    // Step numbers
    assembly-step((-3.5, 4), "1")
    assembly-step((-3.5, 1.5), "2")
    assembly-step((-3.5, -1.5), "3")

    // --- Part 1: Extrusion (top, assembled position reference) ---
    extrusion-end((0, 4.5), size: 0.6)
    content((1.2, 4.5), text(size: 7pt)[2020 Extrusion])

    // --- Part 2: T-Nut + Bolt (exploded above bracket) ---
    // T-nut
    tnut-side((-0.5, 2.8), size: 0.35)
    explode-arrow((-0.5, 2.8), (-0.5, 4.2))
    content((-1.5, 2.8), text(size: 6pt)[T-Nut])

    // Bolt
    bolt-iso((0.5, 2.8), length: 0.6, head-size: 0.25)
    explode-arrow((0.5, 2.8), (0.5, 4.2))
    content((1.5, 2.8), text(size: 6pt)[M5×10])

    // --- Part 3: Motor Mount Bracket ---
    rect((-1, 0.5), (1, 2), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 1.25), text(size: 7pt)[Mount Bracket])
    // Mounting holes
    circle((-0.5, 1.6), radius: 0.08, fill: white, stroke: 0.5pt + diagram-gray)
    circle((0.5, 1.6), radius: 0.08, fill: white, stroke: 0.5pt + diagram-gray)
    // Motor attachment holes
    circle((-0.4, 0.7), radius: 0.06, fill: white, stroke: 0.5pt + diagram-gray)
    circle((0.4, 0.7), radius: 0.06, fill: white, stroke: 0.5pt + diagram-gray)

    // Explode arrow from bracket to extrusion
    explode-arrow((0, 2), (0, 4.2))

    // --- Part 4: Hub Motor (exploded below bracket) ---
    // Motor body
    circle((0, -1.5), radius: 1, stroke: 1.5pt + diagram-black, fill: diagram-light)
    circle((0, -1.5), radius: 0.6, stroke: 1pt + diagram-black)
    circle((0, -1.5), radius: 0.25, fill: diagram-black)
    content((0, -1.5), text(size: 5pt, fill: white)[Axle])
    content((1.8, -1.5), text(size: 7pt)[Hub Motor])

    // Motor mounting bolts
    bolt-iso((-0.4, -0.3), length: 0.4, head-size: 0.2)
    bolt-iso((0.4, -0.3), length: 0.4, head-size: 0.2)
    content((1.5, -0.3), text(size: 6pt)[M4×8 (×4)])

    // Explode arrow from motor to bracket
    explode-arrow((0, -0.5), (0, 0.5))

    // --- Part 5: Wheel/Tire (exploded below motor) ---
    circle((0, -4), radius: 1.2, stroke: 2pt + diagram-black, fill: white)
    circle((0, -4), radius: 0.8, stroke: 1pt + diagram-gray)
    content((0, -4), text(size: 6pt)[160mm])
    content((1.8, -4), text(size: 7pt)[Tire])

    // Explode arrow from tire to motor
    explode-arrow((0, -2.8), (0, -2.5))

    // Assembly direction indicator
    line((3, -4), (3, 4.5), stroke: 1pt + diagram-accent, mark: (end: ">"))
    content((3.8, 0), text(size: 7pt, fill: diagram-accent)[Assembly])
  }),
  caption: [Exploded view: (1) Insert T-nuts into extrusion, (2) Bolt bracket to frame, (3) Attach motor and tire],
)

== Phase 3: Electronics Mounting

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Electronics plate
    rect((-4, -2.5), (4, 2.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)

    // Mounting holes (corners)
    for (x, y) in ((-3.5, 2), (3.5, 2), (-3.5, -2), (3.5, -2)) {
      circle((x, y), radius: 0.15, fill: white, stroke: 0.5pt + diagram-gray)
    }

    // Jetson compute module
    jetson-top((-2, 1), size: (2.2, 1.5))

    // VESCs (4 units in a row)
    for i in range(4) {
      vesc-top((-2.5 + i * 1.8, -1), size: (1.4, 0.8), id: str(i + 1))
    }

    // DCDC converter
    rect((1.5, 0.5), (2.8, 1.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((2.15, 1), text(size: 5pt)[DC-DC])

    // Main fuse
    rect((3, 0.5), (3.7, 1.5), fill: rgb("#fbbf24"), stroke: 1pt + diagram-black, radius: 2pt)
    content((3.35, 1), text(size: 5pt)[100A])

    // Callouts
    callout-leader((-2, 1), (-4.5, 2.5), "1")
    callout-leader((-0.7, -1), (-2, -3), "2")
    callout-leader((2.15, 1), (3.5, 2.5), "3")
    callout-leader((3.35, 1), (4.5, 0), "4")
  }),
  caption: [Electronics plate: (1) Jetson Orin NX, (2) VESC motor controllers, (3) DC-DC converter, (4) Main fuse],
)

== Phase 4: Wiring

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Battery
    battery-top((-3, 3), size: (2, 1))

    // Connection from battery
    line((-3, 2.5), (-3, 2), stroke: 3pt + diagram-accent)

    // Main fuse
    rect((-3.5, 1.5), (-2.5, 2), fill: rgb("#fbbf24"), stroke: 1pt + diagram-black, radius: 2pt)
    content((-3, 1.75), text(size: 6pt, weight: "bold")[100A])
    callout-leader((-3, 1.75), (-4.5, 1.5), "1")

    // E-Stop relay
    line((-3, 1.5), (-3, 1), stroke: 3pt + diagram-accent)
    rect((-3.5, 0.5), (-2.5, 1), fill: diagram-danger, stroke: 1pt + diagram-black, radius: 2pt)
    content((-3, 0.75), text(size: 5pt, fill: white, weight: "bold")[E-STOP])
    callout-leader((-3, 0.75), (-4.5, 0.5), "2")

    // Main power bus
    line((-3, 0.5), (-3, 0), stroke: 3pt + diagram-accent)
    line((-3, 0), (3, 0), stroke: 3pt + diagram-accent)

    // Branch to DC-DC
    line((2.5, 0), (2.5, -0.5), stroke: 2pt + diagram-accent)
    rect((2, -1.2), (3, -0.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((2.5, -0.85), text(size: 5pt)[DC-DC])
    line((2.5, -1.2), (2.5, -1.7), stroke: 1.5pt + rgb("#3b82f6"))
    content((2.5, -2), text(size: 5pt)[12V])
    callout-leader((2.5, -0.85), (4, -0.5), "3")

    // Branches to VESCs
    let vesc-x = (-2, -0.5, 1, 2)
    for i in range(4) {
      let x = vesc-x.at(i)
      line((x, 0), (x, -0.5), stroke: 2pt + diagram-accent)
      vesc-top((x, -1.1), size: (0.8, 0.6), id: str(i + 1))
    }
    callout-leader((-0.5, -1.1), (-1.5, -2.5), "4")

    // XT90 connector symbol
    connector-xt((-3, 2.25), size: "90")
    callout-leader((-3, 2.25), (-4.5, 2.5), "5")
  }),
  caption: [Power distribution: (1) Main fuse, (2) E-Stop relay, (3) DC-DC converter, (4) VESCs, (5) XT90 disconnect],
)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Multi-step CAN connection sequence
    let pw = 2.8  // panel width
    let ph = 2.2  // panel height
    let gap = 0.6

    // Panel 1: Strip wire
    step-panel((0, 0), size: (pw, ph), step-num: 1, title: "Strip")
    // Wire with stripped end
    line((0.3, 0.8), (2.5, 0.8), stroke: 2pt + diagram-accent)
    line((2.0, 0.7), (2.5, 0.7), stroke: 1.5pt + rgb("#cd7f32"))  // exposed copper
    line((2.0, 0.9), (2.5, 0.9), stroke: 1.5pt + rgb("#cd7f32"))
    content((1.4, 0.4), text(size: 5pt)[Strip 5mm])

    panel-arrow-h((0, 0), from-size: (pw, ph), gap: gap)

    // Panel 2: Tin wire
    step-panel((pw + gap, 0), size: (pw, ph), step-num: 2, title: "Tin")
    // Soldering iron approaching wire
    line((pw + gap + 0.3, 0.8), (pw + gap + 2.0, 0.8), stroke: 2pt + diagram-accent)
    line((pw + gap + 1.5, 0.7), (pw + gap + 2.0, 0.7), stroke: 1.5pt + rgb("#c0c0c0"))  // tinned
    line((pw + gap + 1.5, 0.9), (pw + gap + 2.0, 0.9), stroke: 1.5pt + rgb("#c0c0c0"))
    // Soldering iron
    rect((pw + gap + 2.2, 0.5), (pw + gap + 2.6, 1.1), fill: diagram-accent, stroke: 0.75pt + diagram-black, radius: 1pt)
    content((pw + gap + 1.4, 0.4), text(size: 5pt)[Apply solder])

    panel-arrow-h((pw + gap, 0), from-size: (pw, ph), gap: gap)

    // Panel 3: Insert into connector
    step-panel((2 * (pw + gap), 0), size: (pw, ph), step-num: 3, title: "Insert")
    // JST connector
    rect((2 * (pw + gap) + 1.0, 0.5), (2 * (pw + gap) + 2.0, 1.1), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    // Wire going in
    line((2 * (pw + gap) + 0.3, 0.8), (2 * (pw + gap) + 1.0, 0.8), stroke: 2pt + diagram-accent)
    insert-arrow((2 * (pw + gap) + 0.8, 0.8), (2 * (pw + gap) + 1.0, 0.8))
    content((2 * (pw + gap) + 1.5, 0.3), text(size: 5pt)[Push until click])

    panel-arrow-h((2 * (pw + gap), 0), from-size: (pw, ph), gap: gap)

    // Panel 4: Verify
    step-panel((3 * (pw + gap), 0), size: (pw, ph), step-num: 4, title: "Verify")
    // Connected wire in connector
    rect((3 * (pw + gap) + 1.0, 0.5), (3 * (pw + gap) + 2.0, 1.1), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    line((3 * (pw + gap) + 0.3, 0.8), (3 * (pw + gap) + 1.0, 0.8), stroke: 2pt + diagram-accent)
    // Tug arrow
    motion-arrow((3 * (pw + gap) + 0.5, 0.6), (3 * (pw + gap) + 0.3, 0.4), label: "Tug")
    // Check mark
    check-mark((3 * (pw + gap) + 2.4, 0.8), size: 0.3)
  }),
  caption: [CAN wiring sequence: (1) Strip 5mm insulation, (2) Tin exposed wire, (3) Insert into JST connector, (4) Verify with gentle tug test],
)

== Phase 5: Testing

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Horizontal testing flow
    let steps = (
      (0, 0, "1", "Power Off\nMultimeter", "No shorts"),
      (3, 0, "2", "Power On\nNo Motors", "Jetson boots"),
      (6, 0, "3", "VESC\nStatus", "4× green LED"),
      (9, 0, "4", "E-Stop\nTest", "Power cuts"),
      (12, 0, "5", "Motor\nSpin", "Wheels up"),
    )

    for (x, y, num, label, check) in steps {
      process-box((x, y), label, width: 2.4, height: 1.2)
      callout((x - 0.9, y + 0.4), num)
      content((x, y - 1), text(size: 5pt, fill: diagram-gray)[#check])
    }

    for i in range(4) {
      flow-arrow((0.2 + i * 3 + 1.2, 0), (0.2 + (i + 1) * 3 - 1.2, 0))
    }

    // Final check
    check-mark((14, 0), size: 0.5)
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 1em,
  [
    #text(weight: "bold", size: 9pt)[Quality Checklist]
    #checklist(
      [All bolts torqued to 4 Nm],
      [No exposed wiring],
      [CAN bus termination verified],
      [E-Stop cuts power in 100ms],
    )
  ],
  [
    #v(1.2em)
    #checklist(
      [All wheels spin freely],
      [Battery secure],
      [All connectors clicked],
      [Thermal management OK],
    )
  ]
)

#pagebreak()

// =============================================================================
= Electrical System
// =============================================================================

== Power Distribution

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Battery
    rect((-3, 4), (3, 5), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((0, 4.5), text(weight: "bold", size: 9pt)[48V Battery Pack])
    content((0, 3.6), text(size: 8pt)[13S LiPo, 39-54.6V])

    // Main line down from battery
    line((0, 4), (0, 3), stroke: 2pt + black)

    // Fuse
    rect((-0.8, 2.5), (0.8, 3), fill: muni-light-gray, stroke: 1pt + black, radius: 2pt)
    content((0, 2.75), text(size: 7pt)[100A Fuse])

    // Line down from fuse
    line((0, 2.5), (0, 2), stroke: 2pt + black)

    // Split to three branches
    line((-3, 2), (3, 2), stroke: 2pt + black)
    line((-3, 2), (-3, 1), stroke: 2pt + black)
    line((0, 2), (0, 1), stroke: 2pt + black)
    line((3, 2), (3, 1), stroke: 2pt + black)

    // VESCs box
    rect((-4, 0), (-2, 1), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((-3, 0.5), text(size: 8pt, weight: "bold")[VESCs (×4)])

    // E-Stop box
    rect((-1, 0), (1, 1), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((0, 0.5), text(size: 8pt, weight: "bold")[E-Stop])

    // DCDC box
    rect((2, 0), (4, 1), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((3, 0.5), text(size: 8pt, weight: "bold")[DCDC])
    content((3, 0.2), text(size: 7pt)[48→12V])

    // Line down from DCDC
    line((3, 0), (3, -0.5), stroke: 1.5pt + black)
    line((2, -0.5), (4, -0.5), stroke: 1.5pt + black)
    line((2, -0.5), (2, -1), stroke: 1.5pt + black)
    line((4, -0.5), (4, -1), stroke: 1.5pt + black)

    // Jetson box
    rect((1.2, -2), (2.8, -1), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((2, -1.5), text(size: 8pt, weight: "bold")[Jetson])
    content((2, -1.8), text(size: 7pt)[12V])

    // Tools box
    rect((3.2, -2), (4.8, -1), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((4, -1.5), text(size: 8pt, weight: "bold")[Tools])
    content((4, -1.8), text(size: 7pt)[12V])
  }),
  caption: [Power distribution from 48V battery to all subsystems],
)

== CAN Bus

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson
    rect((-4, 0), (-2.5, 0.8), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((-3.25, 0.4), text(weight: "bold", size: 8pt)[Jetson])

    // VESCs
    for i in range(4) {
      let x = -1.5 + i * 1.5
      rect((x - 0.5, 0), (x + 0.5, 0.8), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content((x, 0.4), text(size: 8pt, weight: "bold")[VESC#(i + 1)])
    }

    // Tool MCU
    rect((5, 0), (6.5, 0.8), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
    content((5.75, 0.4), text(size: 8pt, weight: "bold")[Tool])

    // CAN bus line
    line((-2.5, 0.4), (5, 0.4), stroke: 1.5pt + black)

    // Termination resistors
    line((-3.25, 0), (-3.25, -0.5), stroke: 1pt + black)
    rect((-3.5, -0.8), (-3, -0.5), fill: white, stroke: 1pt + black)
    content((-3.25, -0.65), text(size: 6pt)[120Ω])

    line((5.75, 0), (5.75, -0.5), stroke: 1pt + black)
    rect((5.5, -0.8), (6, -0.5), fill: white, stroke: 1pt + black)
    content((5.75, -0.65), text(size: 6pt)[120Ω])
  }),
  caption: [CAN bus daisy chain with 120Ω termination at each end],
)

== Connectors

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Connector types visualization
    let connectors = (
      ("XT90", (-3.5, 0), "90A", "Battery"),
      ("Bullet", (-1.5, 0), "60A", "Motors"),
      ("XT30", (0.5, 0), "30A", "12V"),
      ("JST", (2.5, 0), "3A", "Signals"),
      ("DT", (4.5, 0), "25A", "Tools"),
    )

    for (name, pos, rating, use) in connectors {
      rect((pos.at(0) - 0.6, -0.5), (pos.at(0) + 0.6, 0.5),
           fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content(pos, text(size: 7pt, weight: "bold")[#name])
      content((pos.at(0), -0.9), text(size: 6pt)[#rating])
      content((pos.at(0), 0.9), text(size: 6pt)[#use])
    }
  }),
  caption: [Connector types used throughout the rover],
)

== VESC Configuration

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of rover showing VESC positions
    rect((-2.5, -2.5), (2.5, 2.5), stroke: 1pt + black, radius: 4pt)

    // Wheel positions with IDs
    let wheels = (
      ((-2, 2), "ID 0", "FL"),
      ((2, 2), "ID 1", "FR"),
      ((-2, -2), "ID 2", "RL"),
      ((2, -2), "ID 3", "RR"),
    )

    for (pos, id, label) in wheels {
      circle(pos, radius: 0.5, fill: muni-light-gray, stroke: 1pt + black)
      content(pos, text(size: 6pt)[#id])
      let label-pos = (pos.at(0) * 1.5, pos.at(1) * 1.3)
      content(label-pos, text(size: 7pt)[#label])
    }

    // Direction arrow
    line((0, 2.8), (0, 3.5), stroke: 1pt + black, mark: (end: ">"))
    content((0, 3.8), text(size: 7pt)[Front])
  }),
  caption: [CAN ID assignment by wheel position],
)

#spec-table(
  [*Setting*], [*Value*],
  [Controller ID], [0-3 (unique per VESC)],
  [CAN Mode], [VESC],
  [CAN Baud Rate], [CAN_500K],
  [Send CAN Status], [Enabled],
  [CAN Status Rate], [50 Hz],
)

#pagebreak()

// =============================================================================
= Operation
// =============================================================================

== Startup

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Startup flowchart
    let steps = (
      (0, 5, "Pre-flight\nInspection"),
      (0, 3.5, "Connect\nBattery"),
      (0, 2, "Wait for\nBoot (30s)"),
      (0, 0.5, "Connect\nOperator"),
      (0, -1, "Verify\nTelemetry"),
    )

    for (x, y, label) in steps {
      rect((x - 1, y - 0.5), (x + 1, y + 0.5), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content((x, y), text(size: 7pt)[#label])
    }

    for i in range(4) {
      let y1 = 5 - i * 1.5 - 0.5
      let y2 = 5 - (i + 1) * 1.5 + 0.5
      line((0, y1), (0, y2), stroke: 1pt + black, mark: (end: ">"))
    }
  }),
  caption: [Startup sequence from inspection to operation],
)

== Controls

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Controller layout
    rect((-4, -2), (4, 2), stroke: 1pt + black, radius: 8pt)

    // Left stick (movement)
    circle((-2.5, 0), radius: 0.8, stroke: 1pt + black)
    circle((-2.5, 0.3), radius: 0.25, fill: black)
    content((-2.5, -1.5), text(size: 7pt)[Movement])

    // Right stick (camera)
    circle((2.5, 0), radius: 0.8, stroke: 1pt + black)
    circle((2.5, 0), radius: 0.25, fill: black)
    content((2.5, -1.5), text(size: 7pt)[Camera])

    // Buttons
    circle((0, 0.8), radius: 0.4, fill: rgb("#C41E3A"), stroke: none)
    content((0, 0.8), text(fill: white, size: 6pt)[STOP])
    content((0, -0.5), text(size: 7pt)[E-Stop])

    // Bumpers
    rect((-3.5, 1.5), (-1.5, 1.8), fill: muni-light-gray, stroke: 0.5pt + black, radius: 2pt)
    rect((1.5, 1.5), (3.5, 1.8), fill: muni-light-gray, stroke: 0.5pt + black, radius: 2pt)
    content((-2.5, 2.2), text(size: 6pt)[Speed -])
    content((2.5, 2.2), text(size: 6pt)[Speed +])
  }),
  caption: [Gamepad control layout for teleoperation],
)

== Shutdown

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Shutdown steps
    let steps = (
      (0, 4, "Release\nControls"),
      (0, 2.5, "Disconnect\nInterface"),
      (0, 1, "Press\nE-Stop"),
      (0, -0.5, "Disconnect\nBattery"),
    )

    for (x, y, label) in steps {
      rect((x - 1, y - 0.5), (x + 1, y + 0.5), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content((x, y), text(size: 7pt)[#label])
    }

    for i in range(3) {
      let y1 = 4 - i * 1.5 - 0.5
      let y2 = 4 - (i + 1) * 1.5 + 0.5
      line((0, y1), (0, y2), stroke: 1pt + black, mark: (end: ">"))
    }
  }),
  caption: [Shutdown sequence],
)

== Tool Attachment

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Tool attachment sequence
    rect((-4, 0), (-2, 1.5), stroke: 1pt + black, radius: 2pt)
    content((-3, 0.75), text(size: 7pt)[Rover])

    // Tool
    rect((0, 0), (2, 1.5), stroke: 1pt + black, radius: 2pt)
    content((1, 0.75), text(size: 7pt)[Tool])

    // Mount interface
    rect((-2, 0.5), (-1.5, 1), fill: muni-light-gray, stroke: 0.5pt + black)
    rect((-0.5, 0.5), (0, 1), fill: muni-light-gray, stroke: 0.5pt + black)

    // Arrow showing attachment
    line((-1.2, 0.75), (-0.8, 0.75), stroke: 1pt + black, mark: (end: ">"))

    // Connector below
    circle((-1.75, -0.3), radius: 0.2, fill: muni-light-gray, stroke: 0.5pt + black)
    circle((-0.25, -0.3), radius: 0.2, fill: muni-light-gray, stroke: 0.5pt + black)
    content((0, -0.7), text(size: 6pt)[DT Connector])
  }),
  caption: [Tool attachment via quick-release mount and DT connector],
)

#pagebreak()

// =============================================================================
= Safety
// =============================================================================

#danger[
  Heavy powered machine. Can cause serious injury. Maintain situational awareness.
]

== Hazard Zones

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Rover outline (top view)
    rect((-3, -2), (3, 2), stroke: 1.5pt + diagram-black, radius: 4pt)
    content((0, 0), text(size: 8pt)[BVR0])

    // Wheels
    for (x, y) in ((-3, 1.5), (3, 1.5), (-3, -1.5), (3, -1.5)) {
      rect((x - 0.4, y - 0.6), (x + 0.4, y + 0.6), fill: diagram-black, radius: 2pt)
    }

    // Warning triangles at wheel areas (pinch points)
    for pos in ((-3.8, 1.5), (3.8, 1.5), (-3.8, -1.5), (3.8, -1.5)) {
      warning-symbol(pos, size: 0.6)
    }

    // Tool mount hazard (front)
    warning-symbol((0, 2.8), size: 0.6)

    // Direction indicator
    line((0, 2.3), (0, 2.1), stroke: 0.5pt + diagram-gray, mark: (end: ">"))
    content((0, 2.5), text(size: 5pt)[FRONT])

    // Legend
    warning-symbol((-2.5, -3.5), size: 0.4)
    content((-0.5, -3.5), text(size: 7pt)[Pinch/Crush Hazard Zone])
  }),
  caption: [Hazard zones: wheel areas and tool mount require clearance during operation],
)

== Battery Safety

#warning[
  Li-ion batteries can catch fire if damaged or short-circuited.
]

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Battery care icons
    let items = (
      ("15-25°C", (-3, 0), "Storage Temp"),
      ("No Water", (0, 0), "Keep Dry"),
      ("Inspect", (3, 0), "Check Damage"),
    )

    for (icon, pos, label) in items {
      circle(pos, radius: 0.8, fill: muni-light-gray, stroke: 1pt + black)
      content(pos, text(size: 7pt)[#icon])
      content((pos.at(0), pos.at(1) - 1.2), text(size: 6pt)[#label])
    }
  }),
  caption: [Battery handling requirements],
)

== Emergency Stop

#figure(
  cetz.canvas({
    import cetz.draw: *

    // E-Stop sources
    rect((-4, 0), (-2, 1.5), fill: rgb("#FEF2F2"), stroke: 1pt + muni-danger, radius: 4pt)
    content((-3, 0.75), text(size: 7pt)[Physical\nButton])

    rect((-1, 0), (1, 1.5), fill: rgb("#FEF2F2"), stroke: 1pt + muni-danger, radius: 4pt)
    content((0, 0.75), text(size: 7pt)[Software\nSpacebar])

    rect((2, 0), (4, 1.5), fill: rgb("#FEF2F2"), stroke: 1pt + muni-danger, radius: 4pt)
    content((3, 0.75), text(size: 7pt)[Connection\nLoss])

    // All lead to stop
    line((-3, 0), (-3, -0.5), stroke: 1pt + muni-danger)
    line((0, 0), (0, -0.5), stroke: 1pt + muni-danger)
    line((3, 0), (3, -0.5), stroke: 1pt + muni-danger)
    line((-3, -0.5), (3, -0.5), stroke: 1pt + muni-danger)
    line((0, -0.5), (0, -1), stroke: 1pt + muni-danger, mark: (end: ">"))

    rect((-1.5, -2), (1.5, -1), fill: muni-danger, stroke: none, radius: 4pt)
    content((0, -1.5), text(fill: white, weight: "bold", size: 8pt)[MOTORS STOP])
  }),
  caption: [Three independent paths to emergency stop],
)

#note[
  To reset: resolve cause, release button, reconnect, verify dashboard.
]

#pagebreak()

// =============================================================================
= Maintenance
// =============================================================================

== Pre-Operation Inspection

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Inspection points on rover
    rect((-3, -2), (3, 2), stroke: 1pt + black, radius: 4pt)

    // Inspection points
    let points = (
      ((-2.5, 1.5), "1", "Wheels"),
      ((2.5, 1.5), "2", "Connectors"),
      ((0, 0), "3", "E-Stop"),
      ((-2.5, -1.5), "4", "Battery"),
      ((2.5, -1.5), "5", "Sensors"),
    )

    for (pos, num, label) in points {
      circle(pos, radius: 0.4, fill: muni-light-gray, stroke: 1pt + black)
      content(pos, text(size: 8pt, weight: "bold")[#num])
    }

    // Legend
    for (i, (_, num, label)) in points.enumerate() {
      content((5, 1.5 - i * 0.6), text(size: 7pt)[#num. #label])
    }
  }),
  caption: [Pre-operation inspection points],
)

#checklist(
  [Battery voltage > 40V],
  [No visible damage to chassis or wheels],
  [All connectors secure],
  [Wheels spin freely],
  [E-Stop button functions],
  [Sensors clean and unobstructed],
)

== Maintenance Schedule

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Maintenance schedule timeline
    line((-4, 0), (4, 0), stroke: 1pt + black)

    // Weekly
    circle((-3, 0), radius: 0.3, fill: muni-light-gray, stroke: 1pt + black)
    content((-3, 0.7), text(size: 7pt, weight: "bold")[Weekly])
    content((-3, -0.7), text(size: 6pt)[Clean, Check])

    // Monthly
    circle((0, 0), radius: 0.3, fill: muni-light-gray, stroke: 1pt + black)
    content((0, 0.7), text(size: 7pt, weight: "bold")[Monthly])
    content((0, -0.7), text(size: 6pt)[Inspect, Torque])

    // Seasonal
    circle((3, 0), radius: 0.3, fill: muni-light-gray, stroke: 1pt + black)
    content((3, 0.7), text(size: 7pt, weight: "bold")[Seasonal])
    content((3, -0.7), text(size: 6pt)[Full Service])
  }),
  caption: [Maintenance schedule intervals],
)

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 1em,
  [
    *Weekly*
    - Clean wheels/chassis
    - Wipe lenses
    - Check connections
  ],
  [
    *Monthly*
    - Inspect wiring
    - Verify bolt torque
    - Clean contacts
  ],
  [
    *Seasonal*
    - Full electrical check
    - Check bearings
    - Replace worn parts
  ]
)

== Storage

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Storage checklist icons
    let items = (
      ("50-60%", (-3, 0), "Battery Level"),
      ("Disconnect", (0, 0), "Unplug Battery"),
      ("15-25°C", (3, 0), "Temperature"),
    )

    for (icon, pos, label) in items {
      rect((pos.at(0) - 0.9, pos.at(1) - 0.5), (pos.at(0) + 0.9, pos.at(1) + 0.5),
           fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content(pos, text(size: 7pt)[#icon])
      content((pos.at(0), pos.at(1) - 1), text(size: 6pt)[#label])
    }
  }),
  caption: [Storage preparation requirements],
)

== Troubleshooting

#spec-table(
  [*Symptom*], [*Solution*],
  [Rover won't power on], [Check battery connection, verify fuse],
  [No video feed], [Check LTE connection, verify camera USB],
  [Motor not responding], [Check CAN wiring, verify VESC ID],
  [E-Stop won't release], [Check relay wiring, verify button not stuck],
  [Poor LTE signal], [Relocate antenna, check SIM data plan],
  [Erratic movement], [Verify VESC IDs match wheel positions],
)

#v(2em)
#align(center)[
  #text(size: 10pt)[
    *Municipal Robotics* \
    Cleveland, Ohio \
    #link("https://muni.works")[muni.works]
  ]
]
