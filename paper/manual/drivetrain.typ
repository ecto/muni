#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Drivetrain Section
// Motor brackets, Hub motors, Wheel alignment

= Motor Bracket Design

Each hub motor requires a mounting bracket to attach to the chassis frame.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Bracket top view
    content((-4, 3), text(size: 8pt, weight: "bold")[TOP VIEW])

    rect((-5.5, 0), (-2.5, 2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)

    // Frame mounting holes (slots for adjustment)
    rect((-5.2, 1.6), (-4.8, 1.8), fill: white, stroke: 1pt + diagram-black)
    rect((-3.2, 1.6), (-2.8, 1.8), fill: white, stroke: 1pt + diagram-black)
    content((-4, 2.3), text(size: 5pt)[M5 slots])

    // Motor mounting holes
    let motor_pattern = 30  // mm between holes
    for (dx, dy) in ((-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)) {
      circle((-4 + dx, 0.8 + dy), radius: 0.12, fill: white, stroke: 1pt + muni-orange)
    }
    content((-4, 0.2), text(size: 5pt)[M4 holes])

    // Dimensions
    dim-h(2.5, -5.5, -2.5, "80", offset: 0.3)
    dim-v(-2.3, 0, 2, "50", offset: 0.3)

    // Bracket side view
    content((4, 3), text(size: 8pt, weight: "bold")[SIDE VIEW])

    rect((2.5, 0), (5.5, 0.3), fill: diagram-light, stroke: 1.5pt + diagram-black)
    rect((3.5, 0.3), (4.5, 2), fill: diagram-light, stroke: 1.5pt + diagram-black)

    // Angle indicator
    content((5.8, 1), text(size: 6pt)[L-bracket])
    dim-v(5.7, 0, 2, "50", offset: 0.2)
  }),
  caption: [Motor bracket with slotted holes for alignment adjustment.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Bracket Specifications:*
    - Material: 3mm aluminum or steel
    - Frame holes: M5, slotted 10mm for adjustment
    - Motor holes: M4, match motor bolt pattern
    - L-bracket design for rigidity
  ],
  [
    *Sourcing Options:*
    - Custom CNC cut (recommended)
    - 3D printed (PLA not recommended, use PETG/ABS)
    - Off-the-shelf motor mounts (verify hole pattern)
  ]
)

#pagebreak()

// =============================================================================

= Motor Bracket Mounting

Attach motor brackets to the chassis at each corner.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of chassis corner
    // Extrusion corner
    rect((-4, -0.3), (0, 0.3), fill: diagram-light, stroke: 1pt + diagram-black)
    rect((-0.3, -4), (0.3, 0), fill: diagram-light, stroke: 1pt + diagram-black)

    // Corner bracket
    corner-bracket((0, 0), size: 0.8)

    // Motor bracket position
    rect((-3, -2.5), (-1, -0.5), fill: diagram-light, stroke: 1.5pt + muni-orange, radius: 2pt)
    content((-2, -1.5), text(size: 6pt, fill: muni-orange)[Bracket])

    // T-nut positions
    circle((-2.5, 0), radius: 0.15, fill: muni-orange, stroke: none)
    circle((-1.5, 0), radius: 0.15, fill: muni-orange, stroke: none)

    // Dimension from corner
    dim-h(-3.5, -3, 0, "offset", offset: 0.5)

    content((2, 0), text(size: 7pt)[Frame corner])
    callout-leader((-2, -1.5), (-5, -2), "A")
  }),
  caption: [Motor bracket position at frame corner. All 4 corners mirror this layout.],
)

#v(1em)

*Mounting Procedure:*

+ Slide T-nuts into bottom extrusion channel
+ Position bracket with motor axle aligned to wheel position
+ Insert M5×10 bolts through bracket slots into T-nuts
+ Leave bolts finger-tight for adjustment
+ Verify bracket is perpendicular to extrusion
+ Tighten to 4 Nm

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Bracket Positions:*
    #spec-table(
      [*Corner*], [*Offset from corner*],
      [Front Left], [50mm],
      [Front Right], [50mm],
      [Rear Left], [50mm],
      [Rear Right], [50mm],
    )
  ],
  [
    *Alignment Check:*
    - Motor axles should be parallel
    - Equal distance from frame edges
    - Perpendicular to travel direction
  ]
)

#pagebreak()

// =============================================================================

= Hub Motor Installation

Mount hub motors to the brackets and connect phase wires.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Exploded view
    assembly-step((-4, 3), "1")
    assembly-step((-4, 0), "2")
    assembly-step((-4, -3), "3")

    // Bracket
    rect((-1, 2), (1, 3.5), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 2.75), text(size: 7pt)[Bracket])

    // Motor mounting holes on bracket
    for (dx, dy) in ((-0.4, 0.3), (0.4, 0.3), (-0.4, 1), (0.4, 1)) {
      circle((dx, 2 + dy), radius: 0.1, fill: white, stroke: 0.5pt + diagram-gray)
    }

    // Bolts
    for dx in (-0.4, 0.4) {
      bolt-iso((dx, 1), length: 0.5, head-size: 0.2)
    }
    content((1.5, 1), text(size: 6pt)[M4×8 ×4])
    explode-arrow((0, 1.5), (0, 2))

    // Motor
    circle((0, -1), radius: 1.2, stroke: 2pt + diagram-black, fill: diagram-light)
    circle((0, -1), radius: 0.4, fill: diagram-gray)
    content((0, -1), text(size: 6pt)[Axle])
    content((1.8, -1), text(size: 7pt)[Hub Motor])

    // Phase wires
    line((0.8, -0.3), (1.5, 0.5), stroke: 1.5pt + rgb("#3b82f6"))
    line((0.9, -0.4), (1.6, 0.4), stroke: 1.5pt + rgb("#22c55e"))
    line((1, -0.5), (1.7, 0.3), stroke: 1.5pt + rgb("#eab308"))
    content((2.5, 0.4), text(size: 6pt)[Phase wires])

    explode-arrow((0, 0.2), (0, 0.8))

    // Wheel
    circle((0, -4.5), radius: 1.5, stroke: 2pt + diagram-black, fill: white)
    content((0, -4.5), text(size: 7pt)[Wheel])
    content((2, -4.5), text(size: 6pt)[Pre-mounted])
  }),
  caption: [Motor installation sequence. Wheels typically come pre-mounted on hub motors.],
)

#v(1em)

*Installation Steps:*

+ Align motor mounting holes with bracket holes
+ Insert M4×8 bolts through bracket into motor
+ Tighten in cross pattern to 2 Nm
+ Route phase wires toward electronics bay
+ Secure wires with cable ties (leave slack for wheel movement)

#warning[
  Do not pinch phase wires between motor and bracket. This can cause shorts.
]

#pagebreak()

// =============================================================================

= Wheel Alignment

Verify all wheels are parallel and the rover tracks straight.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Top view of rover with alignment lines
    rect((-4, -3), (4, 3), stroke: 1pt + diagram-black, fill: none)

    // Wheels
    for (x, y) in ((-4.5, 2), (4.5, 2), (-4.5, -2), (4.5, -2)) {
      rect((x - 0.3, y - 0.8), (x + 0.3, y + 0.8), fill: diagram-black, radius: 2pt)
    }

    // Alignment string lines
    line((-4.5, 3.5), (-4.5, -3.5), stroke: 1pt + muni-orange)
    line((4.5, 3.5), (4.5, -3.5), stroke: 1pt + muni-orange)

    // Cross-string
    line((-4.5, 0), (4.5, 0), stroke: 0.5pt + muni-orange)

    // Measurement points
    content((-4.5, 3.8), text(size: 6pt, fill: muni-orange)[String A])
    content((4.5, 3.8), text(size: 6pt, fill: muni-orange)[String B])

    // Dimension checks
    dim-h(-4.2, -4.5, -3.5, "d1", offset: 0.3)
    dim-h(-4.2, 3.5, 4.5, "d2", offset: 0.3)

    content((0, -5), text(size: 7pt)[d1 = d2 ± 2mm at front and rear])
  }),
  caption: [String alignment method. Stretch strings parallel to frame sides.],
)

#v(1em)

*Alignment Procedure:*

+ Stretch two parallel strings along frame sides
+ Measure gap from string to front wheel edge
+ Measure gap from string to rear wheel edge
+ Gaps should be equal (±2mm) on each side
+ If not equal: loosen bracket, adjust, re-tighten

#v(1em)

*Common Issues:*

#spec-table(
  [*Symptom*], [*Cause*], [*Fix*],
  [Rover pulls left], [Right wheels toe-in], [Adjust right brackets outward],
  [Rover pulls right], [Left wheels toe-in], [Adjust left brackets outward],
  [Excessive tire wear], [Wheels not parallel], [Realign all brackets],
  [Vibration at speed], [Wheel out of round], [Replace tire or motor],
)

#pagebreak()
