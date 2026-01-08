#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Drivetrain Section
// Hub motors, Direct mounting, Wheel alignment

= Drivetrain

The BVR0 uses skid-steer drive: four independent hub motors, one in each wheel. To turn, the left and right sides spin at different speeds (or opposite directions). It's the same principle as a tank or a Roomba.

Hub motors eliminate chains, belts, gearboxes, and axles. The motor *is* the wheel. This means fewer parts, less maintenance, and no drivetrain to align. The tradeoff is that hub motors are heavier than outrunner motors with belt drive, but for a utility rover that's not a problem.

The motors we use are 350W hoverboard motors. They're mass-produced, cheap (around \$85 each), and rated for exactly the kind of abuse a sidewalk rover will see.

= Direct Frame Mounting

#procedure([Mount hub motors directly to frame], time: "30 min", difficulty: 1)

#v(1em)

The BVR0 keeps things simple: the hub motors bolt directly to the 2020 aluminum extrusion frame. No brackets, no adapters. The motor's flat mounting face sits against the extrusion, and M5 bolts pass through into T-nuts in the channel.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of motor mounting
    content((0, 4.5), text(size: 8pt, weight: "bold")[SIDE VIEW])

    // Extrusion (cross-section)
    rect((-1, 0), (1, 2), fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 1), text(size: 6pt)[2020])

    // T-slot detail
    rect((-0.3, -0.1), (0.3, 0.1), fill: white, stroke: 0.5pt + diagram-black)

    // Motor mounting face
    rect((-2, -0.5), (2, -0.1), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, -0.8), text(size: 6pt)[Motor face])

    // Bolts through T-slot
    for x in (-0.6, 0.6) {
      line((x, -0.1), (x, 0.8), stroke: 1.5pt + muni-orange)
      circle((x, -0.2), radius: 0.15, fill: muni-orange, stroke: none)
    }
    content((2.3, 0.3), text(size: 5pt, fill: muni-orange)[M5×12])

    // Motor body (hub)
    circle((0, -2.5), radius: 1.8, stroke: 2pt + diagram-black, fill: white)
    circle((0, -2.5), radius: 0.5, fill: diagram-gray)
    content((0, -2.5), text(size: 5pt)[Axle])

    // Dimension
    dim-h(-4, -1.5, 1.5, "42", offset: 0.3)
    content((-4, -1.2), text(size: 5pt)[Bolt pattern])
  }),
  caption: [Hub motor mounts directly to extrusion. No bracket required.],
)

#v(1em)

*Mounting Procedure:*

+ Slide 2× drop-in T-nuts into the extrusion channel at each wheel position
+ Position hub motor with mounting face flat against extrusion
+ Align motor's 42mm bolt pattern with T-nuts
+ Insert M5×12 bolts through motor mounting holes into T-nuts
+ Hand-tighten, then torque to 4 Nm

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Motor Positions:*

    Mount motors at frame corners, centered on each side rail. The exact position isn't critical: the T-slot system allows adjustment.

    #spec-table(
      [*Location*], [*Rail*],
      [Front Left], [Left side rail],
      [Front Right], [Right side rail],
      [Rear Left], [Left side rail],
      [Rear Right], [Right side rail],
    )
  ],
  [
    *Hardware Per Motor:*
    - 2× M5×12 button head bolt
    - 2× M5 drop-in T-nut

    #v(0.5em)
    #text(size: 7pt, fill: gray)[Standard 6.5" hoverboard motors have a 42mm square bolt pattern with M5 threads.]
  ]
)

#v(1em)

#note[
  The 42mm hole pattern fits standard 6.5" hoverboard hub motors. Some motors may have different patterns: verify before ordering.
]

#v(0.5em)

#lesson[
  We originally planned custom brackets. Then we realized the motor face sits flat against the extrusion perfectly. Sometimes the simplest solution is no solution at all.
]

#pagebreak()

// =============================================================================

= Hub Motor Installation

#procedure([Install and wire hub motors], time: "20 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Exploded view
    assembly-step((-4, 2.5), "1")
    assembly-step((-4, 0), "2")
    assembly-step((-4, -2.5), "3")

    // T-nuts
    for dx in (-0.4, 0.4) {
      rect((dx - 0.2, 2.2), (dx + 0.2, 2.5), fill: diagram-gray, stroke: 0.5pt + diagram-black)
    }
    content((1.5, 2.4), text(size: 6pt)[T-nuts in channel])

    // Extrusion section
    rect((-1, 1.3), (1, 1.8), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 1.55), text(size: 6pt)[Extrusion])

    explode-arrow((0, 1.9), (0, 2.2))

    // Bolts
    for dx in (-0.4, 0.4) {
      bolt-iso((dx, 0.5), length: 0.5, head-size: 0.2)
    }
    content((1.5, 0.5), text(size: 6pt)[M5×12 ×2])
    explode-arrow((0, 0.8), (0, 1.3))

    // Motor
    circle((0, -1.5), radius: 1.2, stroke: 2pt + diagram-black, fill: diagram-light)
    circle((0, -1.5), radius: 0.4, fill: diagram-gray)
    content((0, -1.5), text(size: 6pt)[Axle])
    content((1.8, -1.5), text(size: 7pt)[Hub Motor])

    // Phase wires
    line((0.8, -0.7), (1.5, 0), stroke: 1.5pt + rgb("#3b82f6"))
    line((0.9, -0.8), (1.6, -0.1), stroke: 1.5pt + rgb("#22c55e"))
    line((1, -0.9), (1.7, -0.2), stroke: 1.5pt + rgb("#eab308"))
    content((2.5, -0.1), text(size: 6pt)[Phase wires])

    explode-arrow((0, -0.1), (0, 0.3))
  }),
  caption: [Motor installation sequence. Wheels come pre-mounted on hub motors.],
)

#v(1em)

*Installation Steps:*

+ Position T-nuts in extrusion channel at wheel location
+ Align motor mounting holes with T-nut positions
+ Insert M5×12 bolts through motor into T-nuts
+ Hand-thread first, then tighten to 4 Nm
+ Route phase wires toward electronics bay
+ Secure wires with cable ties (leave slack for service)

#warning[
  Do not pinch phase wires between motor and frame. Route wires away from mounting surface before tightening.
]

#v(0.5em)

#lesson[
  Hand-thread every bolt first before using a driver. Cross-threading an M5 into aluminum is easy to do and hard to fix.
]

#pagebreak()

// =============================================================================

= Wheel Alignment

#procedure([Check and adjust wheel alignment], time: "15 min", difficulty: 2)

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
+ If not equal: loosen motor bolts, slide motor in T-slot, re-tighten

#v(1em)

*Common Issues:*

#spec-table(
  [*Symptom*], [*Cause*], [*Fix*],
  [Rover pulls left], [Right wheels toe-in], [Slide right motors outward],
  [Rover pulls right], [Left wheels toe-in], [Slide left motors outward],
  [Excessive tire wear], [Wheels not parallel], [Realign all motors],
  [Vibration at speed], [Wheel out of round], [Replace tire or motor],
)

#note[
  The T-slot mounting makes alignment adjustable. This is one advantage of the direct-mount approach: loosen two bolts, slide, retighten.
]

#pagebreak()
