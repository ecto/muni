#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Chassis Assembly Section
// Cutting extrusions, Base frame, Verticals, Top frame, Squareness

= Chassis Assembly

The chassis is the skeleton of the rover. It's built from 2020 aluminum extrusion, the same stuff used in 3D printer frames and CNC machines. The T-slot design means you can mount anything anywhere, and if you mess up a hole, just slide the T-nut to a new position.

The frame goes together like adult LEGO. No welding, no precision machining. If you can use a saw and a hex key, you can build this chassis.

= Cutting Extrusions

#procedure([Cut aluminum stock to length], time: "20 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Miter saw setup
    content((0, 4), text(size: 9pt, weight: "bold")[Cutting Setup])

    // Extrusion on saw
    rect((-4, 1), (4, 1.4), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 1.2), text(size: 6pt)[2020 Extrusion])

    // Saw blade
    circle((0, 2.5), radius: 1.5, stroke: 2pt + diagram-black, fill: none)
    line((0, 1), (0, 1.4), stroke: 2pt + muni-danger)

    // Stop block
    rect((3, 0.5), (3.5, 2), fill: diagram-gray, stroke: 1pt + diagram-black)
    content((3.25, 2.3), text(size: 6pt)[Stop])

    // Dimension
    dim-h(0.5, -4, 3, "600", offset: 0.5)

    // Fence
    line((-4.5, 0.5), (4.5, 0.5), stroke: 1.5pt + diagram-black)
    content((5, 0.5), text(size: 6pt)[Fence])
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Procedure:*
    + Clamp stop block at 600mm from blade
    + Place extrusion against fence and stop
    + Cut slowly to prevent burrs
    + Rotate 90° and re-cut if needed for square ends
    + Deburr all cut edges with file or deburring tool
  ],
  [
    *Cut List (BVR0 standard):*
    #spec-table(
      [*Qty*], [*Length*], [*Purpose*],
      [4], [600mm], [Base frame],
      [4], [600mm], [Top frame],
      [4], [250mm], [Vertical posts],
    )

    #v(0.3em)
    #text(size: 7pt, fill: gray)[Total: 5.8m of 2020 extrusion needed.]
  ]
)

#v(1em)

#warning[
  Aluminum chips are sharp. Wear safety glasses. Clean chips from T-slots before assembly.
]

#v(0.5em)

#pitfall[
  Cutting too fast causes burrs that jam T-nuts. Slow cuts with a fine-tooth blade save deburring time.
]

#pagebreak()

// =============================================================================

= Base Frame Assembly

#procedure([Assemble the base frame square], time: "15 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of base frame
    rect((-4, -3), (4, 3), stroke: 2pt + diagram-black, radius: 0pt)

    // Corner brackets
    for (x, y) in ((-4, -3), (4, -3), (-4, 3), (4, 3)) {
      corner-bracket((x, y), size: 0.6)
    }

    // T-nut positions
    for x in (-3, -1, 1, 3) {
      // Bottom edge
      circle((x, -3), radius: 0.15, fill: muni-orange, stroke: none)
      // Top edge
      circle((x, 3), radius: 0.15, fill: muni-orange, stroke: none)
    }
    for y in (-2, 0, 2) {
      // Left edge
      circle((-4, y), radius: 0.15, fill: muni-orange, stroke: none)
      // Right edge
      circle((4, y), radius: 0.15, fill: muni-orange, stroke: none)
    }

    // Dimensions
    dim-h(-4, -4, 4, "600", offset: 1)
    dim-v(4, -3, 3, "600", offset: 1)

    // Diagonal check
    line((-4, -3), (4, 3), stroke: 0.5pt + diagram-gray)
    line((-4, 3), (4, -3), stroke: 0.5pt + diagram-gray)
    content((0, 0), text(size: 7pt, fill: diagram-gray)[Diagonals equal?])

    // Legend
    circle((-2, -5), radius: 0.15, fill: muni-orange, stroke: none)
    content((0, -5), text(size: 7pt)[= T-nut location])
  }),
  caption: none,
)

#v(1em)

*Assembly Steps:*

+ *Pre-insert T-nuts* into all extrusion channels (8 per extrusion, 32 total for base)
+ *Dry-fit* all four extrusions in a square, corners aligned
+ *Attach corner brackets* loosely (finger-tight M5×10 bolts)
+ *Check squareness*: measure both diagonals. They must be equal (±1mm).
+ *If not square*: tap the long diagonal corner with a mallet to adjust
+ *Tighten all bolts* to 4 Nm in a star pattern

#v(1em)

#note[
  Leave extra T-nuts in channels for later mounting. Easier now than adding drop-in nuts later.
]

#v(0.5em)

#pagebreak()

// =============================================================================

= Vertical Posts

#procedure([Install corner vertical posts], time: "10 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Isometric view of frame with verticals
    let ox = 0
    let oy = 0
    let scale = 1.2

    // Base frame (parallelogram for isometric)
    line((ox - 3, oy - 1), (ox + 1, oy - 1), stroke: 1.5pt + diagram-black)
    line((ox + 1, oy - 1), (ox + 3, oy + 0.5), stroke: 1.5pt + diagram-black)
    line((ox + 3, oy + 0.5), (ox - 1, oy + 0.5), stroke: 1.5pt + diagram-black)
    line((ox - 1, oy + 0.5), (ox - 3, oy - 1), stroke: 1.5pt + diagram-black)

    // Vertical posts
    let posts = (
      (ox - 3, oy - 1),
      (ox + 1, oy - 1),
      (ox + 3, oy + 0.5),
      (ox - 1, oy + 0.5),
    )

    for (px, py) in posts {
      line((px, py), (px, py + 3), stroke: 2pt + diagram-black)
      // Top cap
      circle((px, py + 3), radius: 0.1, fill: diagram-black)
    }

    // Corner bracket at base
    corner-bracket((ox - 3, oy - 1), size: 0.4)

    // Dimension for height
    dim-v(ox + 4, oy - 1, oy + 2, "200-400", offset: 0.5)

    // Callout
    callout-leader((ox - 3, oy + 1), (ox - 5, oy + 2), "A")
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Mounting Method A: Corner Bracket*

    Use 90° corner brackets at each post base.

    - 2× M5×10 bolts per bracket
    - Insert T-nuts in both base and post
    - Tighten to 4 Nm
  ],
  [
    *Mounting Method B: Blind Joint*

    Use blind joint connectors for cleaner look.

    - Drill 5mm access hole in base extrusion
    - Thread M5×25 bolt through into post
    - Hidden hardware, harder to adjust
  ]
)

#v(1em)

*Height Calculation:*

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Ground line
    line((-3, 0), (3, 0), stroke: 1pt + diagram-gray)
    content((3.5, 0), text(size: 5pt)[Ground])

    // Wheel
    circle((0, 0.8), radius: 0.8, stroke: 1pt + diagram-black, fill: diagram-light)
    content((0, 0.8), text(size: 5pt)[Wheel])

    // Ground clearance
    dim-v(-1.5, 0, 0.5, "50", offset: 0.3)
    content((-2.5, 0.25), text(size: 5pt)[Clearance])

    // Base frame
    rect((-0.5, 1.4), (0.5, 1.6), fill: diagram-light, stroke: 1pt + diagram-black)
    content((1.2, 1.5), text(size: 5pt)[Base])

    // Vertical post
    rect((-0.1, 1.6), (0.1, 4.1), fill: diagram-light, stroke: 1pt + diagram-black)
    dim-v(0.8, 1.6, 4.1, "250", offset: 0.3)

    // Top frame
    rect((-0.5, 4.1), (0.5, 4.3), fill: diagram-light, stroke: 1pt + diagram-black)

    // Mast
    rect((-0.05, 4.3), (0.05, 6.3), fill: diagram-light, stroke: 1pt + diagram-black)
    dim-v(0.8, 4.3, 6.3, "350", offset: 0.3)
    content((1.5, 5.3), text(size: 5pt)[Mast])

    // Camera
    circle((0, 6.5), radius: 0.2, fill: diagram-black)

    // Total height
    dim-v(-2.5, 0, 6.5, "700", offset: 0.3)
  }),
  caption: none,
)

#spec-table(
  [*Component*], [*Height*], [*Cumulative*],
  [Wheel radius], [80mm], [80mm],
  [Ground clearance], [50mm], [--],
  [Base to top frame], [250mm posts + 40mm], [370mm],
  [Sensor mast], [330mm], [700mm],
)

#pagebreak()

// =============================================================================

= Top Frame

#procedure([Complete the box frame], time: "15 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Isometric box
    let ox = 0
    let oy = -1

    // Base
    line((ox - 3, oy), (ox + 1, oy), stroke: 1pt + diagram-gray)
    line((ox + 1, oy), (ox + 3, oy + 1), stroke: 1pt + diagram-gray)
    line((ox + 3, oy + 1), (ox - 1, oy + 1), stroke: 1pt + diagram-gray)
    line((ox - 1, oy + 1), (ox - 3, oy), stroke: 1pt + diagram-gray)

    // Verticals
    for (px, py) in ((ox - 3, oy), (ox + 1, oy), (ox + 3, oy + 1), (ox - 1, oy + 1)) {
      line((px, py), (px, py + 3), stroke: 1.5pt + diagram-black)
    }

    // Top frame (highlighted)
    line((ox - 3, oy + 3), (ox + 1, oy + 3), stroke: 2pt + muni-orange)
    line((ox + 1, oy + 3), (ox + 3, oy + 4), stroke: 2pt + muni-orange)
    line((ox + 3, oy + 4), (ox - 1, oy + 4), stroke: 2pt + muni-orange)
    line((ox - 1, oy + 4), (ox - 3, oy + 3), stroke: 2pt + muni-orange)

    // Corner brackets on top
    for (px, py) in ((ox - 3, oy + 3), (ox + 1, oy + 3), (ox + 3, oy + 4), (ox - 1, oy + 4)) {
      corner-bracket((px, py), size: 0.4)
    }

    content((5, oy + 3.5), text(size: 8pt, fill: muni-orange, weight: "bold")[Top Frame])
  }),
  caption: none,
)

#v(1em)

*Assembly:*

+ Attach corner brackets to top of each vertical post (loosely)
+ Place top frame extrusions onto brackets
+ Align extrusions flush with vertical posts
+ Check that top frame is level (use spirit level)
+ Tighten all connections to 4 Nm

#v(1em)

#note[
  The top frame provides mounting points for the electronics plate, sensor mast, and protective covers.
]

#pagebreak()

// =============================================================================

= Squareness Check

#procedure([Verify frame geometry], time: "5 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of frame with measurement points
    rect((-4, -3), (4, 3), stroke: 2pt + diagram-black)

    // Diagonal measurements
    line((-4, -3), (4, 3), stroke: 1.5pt + muni-orange)
    line((-4, 3), (4, -3), stroke: 1.5pt + muni-orange)

    // Measurement labels
    content((2, 1.5), text(size: 8pt, fill: muni-orange)[D1])
    content((-2, 1.5), text(size: 8pt, fill: muni-orange)[D2])

    // Corners labeled
    content((-4.5, -3), text(size: 7pt)[A])
    content((4.5, -3), text(size: 7pt)[B])
    content((4.5, 3), text(size: 7pt)[C])
    content((-4.5, 3), text(size: 7pt)[D])

    // Check mark
    check-mark((0, -5), size: 0.5)
    content((1.5, -5), text(size: 8pt)[D1 = D2 ± 1mm])
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Squareness Test:*

    + Measure diagonal A→C (D1)
    + Measure diagonal B→D (D2)
    + Compare: D1 should equal D2 within 1mm
    + If not equal: loosen corners, tap long diagonal, re-tighten
  ],
  [
    *Rigidity Test:*

    + Grip opposite corners
    + Try to twist the frame
    + Frame should not flex or rack
    + If loose: check all bolt torque, add corner braces if needed
  ]
)

#v(1em)

*Final Checklist:*

#checklist(
  [All corners have brackets installed],
  [All bolts torqued to 4 Nm],
  [Diagonals equal within 1mm],
  [Frame does not rack or twist],
  [All T-slots clear of debris],
  [Extra T-nuts in channels for later use],
)

#v(0.5em)

#tip[
  Take a photo of the diagonal measurements. Useful reference if the frame gets knocked out of square later.
]

#pagebreak()
