#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Safety Section
// Hazard zones, Battery safety

= Safety

The BVR0 is a machine, and machines can hurt you. The hazards are real: spinning wheels that don't care about fingers, a battery that can catch fire, a 30kg robot that can pin you against a wall.

None of this is meant to scare you. These hazards are manageable with basic awareness and respect for the machine. The safety protocols in this section come from experience (some of it painful). Follow them.

= Personal Protective Equipment

#procedure([PPE requirements by task], time: "reference", difficulty: 1)

#v(1em)

#spec-table(
  [*Task*], [*Required PPE*],
  [Cutting extrusions], [Safety glasses, work gloves],
  [Soldering / wiring], [Safety glasses, fume extraction],
  [Battery handling], [Insulated gloves, safety glasses],
  [Motor testing], [Safety glasses, hearing protection],
  [Operation], [None required (stay clear of rover)],
)

#v(1em)

*Safe Operating Distance:*
- Operator: minimum 2m from rover during teleop
- Bystanders: minimum 5m from operating rover
- During charging: check every 15 min, do not leave unattended

#pagebreak()

// =============================================================================

= Hazard Zones

#procedure([Know the danger zones], time: "reference", difficulty: 1)

#v(1em)

#danger[
  Stay clear of marked zones during operation. Serious injury possible.
]

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Rover top view
    rect((-4, -3), (4, 3), stroke: 2pt + diagram-black, radius: 4pt)
    content((0, 0), text(size: 10pt, weight: "bold")[BVR0])

    // Wheels with hazard zones
    for (x, y) in ((-4, 2.2), (4, 2.2), (-4, -2.2), (4, -2.2)) {
      // Wheel
      rect((x - 0.5, y - 0.8), (x + 0.5, y + 0.8), fill: diagram-black, radius: 2pt)
      // Hazard zone circle
      circle((x, y), radius: 1.3, stroke: 2pt + muni-danger, fill: none)
    }

    // Tool area hazard (front)
    rect((-2.5, 3.3), (2.5, 4.8), stroke: 2pt + muni-danger, fill: rgb("#FEE2E2"), radius: 2pt)
    content((0, 4.05), text(size: 9pt, fill: muni-danger, weight: "bold")[TOOL ZONE])

    // Front indicator
    motion-arrow((0, 3.1), (0, 3.3))

    // Legend
    circle((-3, -5), radius: 0.3, stroke: 2pt + muni-danger, fill: none)
    content((-1, -5), text(size: 7pt)[Pinch/Crush hazard])
    rect((1.5, -5.2), (2.5, -4.8), stroke: 2pt + muni-danger, fill: rgb("#FEE2E2"), radius: 2pt)
    content((4, -5), text(size: 7pt)[Tool operation zone])
  }),
  caption: [Keep hands, feet, and loose clothing clear of marked zones.],
)

#v(1em)

*Hazard Types:*

#spec-table(
  [*Zone*], [*Hazard*], [*Injury Type*],
  [Wheel areas], [Rotating wheels, motor torque], [Crush, pinch, friction burn],
  [Tool zone], [Rotating auger/blade], [Laceration, amputation],
  [Underside], [50mm ground clearance], [Crush if rover tips],
  [Battery area], [Electrical, thermal], [Shock, burns],
)

#pagebreak()

// =============================================================================

= Battery Safety

#procedure([Handle lithium batteries safely], time: "reference", difficulty: 2)

#v(1em)

#warning[
  Li-ion batteries can catch fire if damaged, punctured, or short-circuited.
]

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Do's and Don'ts
    let dos = (
      ("15-25°C", "Store temp"),
      ("50-60%", "Storage charge"),
      ("Inspect", "Before each use"),
    )
    let donts = (
      ("No water", "Keep dry"),
      ("No puncture", "Protect case"),
      ("No fire", "Never burn"),
    )

    content((-3.5, 3), text(size: 9pt, weight: "bold", fill: muni-success)[DO])
    for (i, (icon, label)) in dos.enumerate() {
      let y = 1.5 - i * 1.5
      circle((-3.5, y), radius: 0.6, fill: rgb("#DCFCE7"), stroke: 1pt + muni-success)
      content((-3.5, y), text(size: 7pt)[#icon])
      content((-1.5, y), text(size: 7pt)[#label])
    }

    content((3.5, 3), text(size: 9pt, weight: "bold", fill: muni-danger)[DON'T])
    for (i, (icon, label)) in donts.enumerate() {
      let y = 1.5 - i * 1.5
      circle((3.5, y), radius: 0.6, fill: rgb("#FEE2E2"), stroke: 1pt + muni-danger)
      content((3.5, y), text(size: 7pt)[#icon])
      content((5.5, y), text(size: 7pt)[#label])
    }
  }),
  caption: none,
)

#v(1em)

*Signs of Battery Damage:*
- Swelling or bulging
- Unusual heat
- Hissing or venting
- Visible damage to case
- Reduced capacity

#v(1em)

*In Case of Battery Fire:*

+ *Evacuate* the immediate area (minimum 10m / 30ft)
+ Call fire department: *911*
+ If small and contained: use *CO2* or *ABC dry chemical* extinguisher
+ If large or spreading: *do not attempt to extinguish* (let professionals handle)
+ Ventilate area (toxic fluoride fumes)
+ Water in *large quantities* can cool adjacent cells and prevent spread, but small amounts can make it worse

#v(0.5em)

#warning[
  Li-ion fires re-ignite. Monitor for at least 1 hour after fire appears out. Do not move battery until cool.
]

#v(0.5em)

#danger[
  Never attempt to charge a damaged battery. Dispose at authorized battery recycling facility (Call2Recycle, Best Buy, etc.).
]

#pagebreak()
