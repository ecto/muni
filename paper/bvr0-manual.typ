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
= Specifications
// =============================================================================

== Overview

The BVR0 (Base Vectoring Rover, Revision 0) is a compact sidewalk-scale robotic platform designed for snow clearing and grounds maintenance. The rover measures 600mm square and stands 400mm tall, sized to navigate standard sidewalks while remaining small enough for a single person to lift.

Four independently-driven hub motors provide omnidirectional control without mechanical steering. A modular tool interface at the front accepts snow augers, brine sprayers, and sweeper attachments. The operator controls the rover remotely via LTE teleoperation, viewing a 360° video feed from the onboard camera.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Chassis frame
    rect((-3, -3), (3, 3), stroke: 1.5pt + diagram-black, radius: 4pt)

    // Wheels at corners
    for (x, y) in ((-3.25, 2.75), (3.25, 2.75), (-3.25, -2.75), (3.25, -2.75)) {
      rect((x - 0.5, y - 0.75), (x + 0.5, y + 0.75), fill: diagram-black, radius: 2pt)
    }

    // Electronics area
    rect((-2, -2), (2, 1), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 2pt)

    // Tool mount (front)
    rect((-1.5, 2.5), (1.5, 3), fill: diagram-light, stroke: 0.5pt + diagram-gray)

    // Sensor mast
    circle((0, 1.5), radius: 0.3, fill: diagram-black)

    // Dimension lines using library helpers
    dim-h(-3, -3, 3, "600 mm", offset: 1.5)
    dim-v(3, -3, 3, "600 mm", offset: 1.5)

    // Numbered callouts with leaders
    callout-leader((-3.25, 2.75), (-5, 3.5), "1")
    callout-leader((0, -0.5), (-4, -1.5), "2")
    callout-leader((0, 2.75), (3.5, 4), "3")
    callout-leader((0, 1.5), (2.5, 2.5), "4")

    // Direction indicator
    motion-arrow((0, 3.5), (0, 4.5))
    content((0, 4.8), text(size: 7pt)[FRONT])
  }),
  caption: [BVR0 top view: (1) Hub motor wheels, (2) Electronics bay, (3) Tool mount, (4) Sensor mast],
)

== Physical Specifications

The chassis is constructed from 2020 aluminum extrusion, providing a rigid yet lightweight frame. The electronics plate mounts centrally, keeping the center of gravity low. Hub motors integrate directly into the wheels, eliminating drivetrain complexity.

#spec-table(
  [*Dimension*], [*Value*],
  [Length], [600 mm],
  [Width], [600 mm],
  [Height], [400 mm (without sensor mast)],
  [Weight], [~25 kg (without battery)],
  [Ground Clearance], [50 mm],
  [Wheel Diameter], [165 mm (6.5")],
)

== Electrical Specifications

The rover operates on a 48V nominal battery pack, providing sufficient voltage for efficient motor operation while remaining within safe handling limits. A DC-DC converter steps voltage down to 12V for the compute module and accessories.

#spec-table(
  [*Parameter*], [*Value*],
  [Battery], [13S4P Li-ion, 48V nominal, 20Ah],
  [Voltage Range], [39V - 54.6V],
  [Motor Power], [4× 350W hub motors (1.4 kW total)],
  [Continuous Current], [60A per motor controller],
  [Control Voltage], [12V (via DC-DC converter)],
  [Communication], [CAN bus 500 kbps],
)

== Performance

Operating speed is intentionally limited to human walking pace. This enables safe sidewalk operation, reduces stopping distance, and improves operator situational awareness. The 4-hour runtime covers typical snow clearing shifts.

#spec-table(
  [*Metric*], [*Value*],
  [Max Speed], [2.5 m/s (5.6 mph)],
  [Operating Speed], [1.0 - 1.5 m/s],
  [Runtime], [~4 hours at working speed],
  [Max Grade], [15%],
  [Operating Temperature], [-20°C to 40°C],
)

== Sensors

The sensor suite prioritizes situational awareness for teleoperation. The 360° camera provides immersive video for the operator. LiDAR enables future autonomous capabilities and provides depth information for obstacle detection.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of rover with sensor mast
    rect((-3, 0), (3, 1), stroke: 1pt + black, radius: 2pt)
    content((0, 0.5), text(size: 7pt)[Chassis])

    // Wheels
    circle((-2.5, 0), radius: 0.6, stroke: 1pt + black)
    circle((2.5, 0), radius: 0.6, stroke: 1pt + black)

    // Sensor mast
    line((0, 1), (0, 3), stroke: 1.5pt + black)

    // LiDAR
    rect((-0.4, 2.5), (0.4, 3), fill: muni-light-gray, stroke: 1pt + black, radius: 2pt)
    content((1.2, 2.75), text(size: 6pt)[LiDAR])

    // Camera
    circle((0, 3.3), radius: 0.25, fill: black)
    content((1, 3.3), text(size: 6pt)[360° Camera])

    // Field of view arcs
    arc((0, 2.75), start: 20deg, stop: 160deg, radius: 2, stroke: 0.5pt + gray)
    arc((0, 2.75), start: 200deg, stop: 340deg, radius: 2, stroke: 0.5pt + gray)
  }),
  caption: [Sensor mast carries LiDAR and 360° camera above obstacle height],
)

#spec-table(
  [*Sensor*], [*Purpose*],
  [Livox Mid-360 LiDAR], [3D mapping, obstacle detection],
  [Insta360 X4], [360° video for teleoperation],
  [IMU (integrated)], [Orientation, motion estimation],
  [GPS (optional)], [Georeferenced positioning],
)

#pagebreak()

// =============================================================================
= Bill of Materials
// =============================================================================

The BVR0 is designed for approximately \$4,000 in components, prioritizing availability and replaceability over optimization. All parts are commercially available; custom fabrication is limited to simple cut-and-drill operations on aluminum plate and extrusion.

== Cost Summary

#spec-table(
  [*Category*], [*Est. Cost*],
  [Chassis], [\$150],
  [Drivetrain], [\$800],
  [Electronics], [\$900],
  [Perception], [\$1,800],
  [Power], [\$400],
  [Wiring/Misc], [\$100],
  [*Total*], [*~\$4,150*],
)

== Chassis

The chassis uses 2020 aluminum extrusion for its balance of strength, weight, and ease of modification. T-slot construction allows components to be repositioned without drilling.

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [2020 extrusion 600mm], [8], [\$5], [\$40],
  [2020 corner bracket], [16], [\$2], [\$32],
  [M5×10 BHCS], [100], [\$0.10], [\$10],
  [M5 T-nut], [100], [\$0.15], [\$15],
  [Electronics plate (1/4" AL)], [1], [\$50], [\$50],
  [*Subtotal*], [], [], [*\$147*],
)

#figure(
  cetz.canvas({
    import cetz.draw: *
    
    // Hardware reference at approximate scale
    content((0, 2), text(size: 8pt, weight: "bold")[Hardware Reference])
    
    // M5x10 bolt
    screw-actual-size((-3, 0), thread: "M5", length: 10)
    
    // M5x16 bolt
    screw-actual-size((-1, 0), thread: "M5", length: 16)
    
    // M4x8 bolt
    screw-actual-size((1, 0), thread: "M4", length: 8)
    
    // T-nut
    tnut-side((3, 0), size: 0.5)
    content((3, -0.7), text(size: 6pt)[M5 T-Nut])
    
    // Corner bracket
    corner-bracket((5, 0), size: 0.9)
    content((5, -0.9), text(size: 6pt)[Bracket])
    
    // Scale bar
    scale-bar((-2, -1.8), length: 3, real-length: "20 mm", divisions: 4)
  }),
  caption: [Chassis hardware reference. Use this scale bar to verify print size.],
)

== Drivetrain

Hub motors eliminate chains, belts, and gearboxes. Each motor contains a brushless DC motor, planetary gearbox, and wheel tire in a single unit. VESC motor controllers provide precise torque control and regenerative braking.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Hub motor cross-section
    circle((0, 0), radius: 2, stroke: 1.5pt + black)
    circle((0, 0), radius: 1.2, stroke: 1pt + black)
    circle((0, 0), radius: 0.4, fill: black)

    // Labels
    content((0, 2.5), text(size: 7pt)[Tire])
    content((0, 1.6), text(size: 6pt)[Stator])
    content((0, 0.8), text(size: 6pt)[Rotor])
    content((0, 0), text(size: 5pt, fill: white)[Axle])

    // Dimension
    line((-2.3, -2), (-2.3, 2), stroke: 0.5pt + gray)
    line((-2.5, -2), (-2.1, -2), stroke: 0.5pt + gray)
    line((-2.5, 2), (-2.1, 2), stroke: 0.5pt + gray)
    content((-3, 0), text(size: 7pt)[165mm])
  }),
  caption: [Hub motor integrates motor, gearbox, and wheel in one unit],
)

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [Hoverboard hub motor 350W], [4], [\$50], [\$200],
  [VESC 6.6], [4], [\$120], [\$480],
  [Motor mount (custom)], [4], [\$20], [\$80],
  [Wheel spacer (custom)], [4], [\$10], [\$40],
  [*Subtotal*], [], [], [*\$800*],
)

== Electronics

The Jetson Orin NX provides GPU-accelerated compute for video encoding, sensor processing, and future autonomy features. The Sierra MC7455 LTE modem enables reliable cellular connectivity for teleoperation.

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [Jetson Orin NX 16GB], [1], [\$600], [\$600],
  [Jetson carrier board], [1], [\$100], [\$100],
  [USB CAN adapter], [1], [\$30], [\$30],
  [LTE modem (Sierra MC7455)], [1], [\$80], [\$80],
  [7" HDMI display], [1], [\$50], [\$50],
  [GPS module (optional)], [1], [\$30], [\$30],
  [*Subtotal*], [], [], [*\$890*],
)

== Perception

The Livox Mid-360 provides 360° LiDAR coverage in a compact, solid-state package. The Insta360 X4 captures 360° video that the operator can pan and tilt virtually, providing natural situational awareness.

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [Livox Mid-360 LiDAR], [1], [\$1,500], [\$1,500],
  [Insta360 X4], [1], [\$300], [\$300],
  [Sensor mount pole (1" AL)], [1], [\$20], [\$20],
  [*Subtotal*], [], [], [*\$1,820*],
)

== Power System

The 13S4P battery pack provides 960Wh of capacity. At typical operating loads of 200-300W, this yields 3-4 hours of runtime. The pack includes a battery management system (BMS) for cell balancing and protection.

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [13S4P battery pack 20Ah], [1], [\$300], [\$300],
  [48V→12V DCDC 20A], [1], [\$40], [\$40],
  [100A ANL fuse + holder], [1], [\$15], [\$15],
  [E-Stop relay 100A], [1], [\$25], [\$25],
  [E-Stop button], [1], [\$15], [\$15],
  [*Subtotal*], [], [], [*\$395*],
)

== Wiring & Connectors

Silicone wire handles the temperature extremes of outdoor operation. XT90 connectors are rated for the high currents of the main battery circuit. Deutsch DT connectors provide weatherproof connections for the tool interface.

#bom-table(
  [Part], [Qty], [Unit], [Total],
  [8 AWG silicone wire (red)], [2m], [\$3/m], [\$6],
  [8 AWG silicone wire (black)], [2m], [\$3/m], [\$6],
  [14 AWG wire assortment], [1], [\$15], [\$15],
  [22 AWG twisted pair], [5m], [\$1/m], [\$5],
  [XT90 connectors (5 pair)], [1], [\$12], [\$12],
  [XT30 connectors (10 pair)], [1], [\$8], [\$8],
  [Deutsch DT connector kit], [1], [\$25], [\$25],
  [Heat shrink kit], [1], [\$12], [\$12],
  [Cable management], [1], [\$10], [\$10],
  [*Subtotal*], [], [], [*\$99*],
)

#pagebreak()

// =============================================================================
= Assembly
// =============================================================================

Assembly proceeds in five phases: chassis frame, motor mounting, electronics installation, wiring, and testing. Each phase should be completed and verified before proceeding to the next.

== Required Tools

A basic set of hand tools is sufficient for assembly. No specialized equipment is required.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Tool icons with visual representations
    
    // Hex keys
    tool-hex-key((-5, 0), size: 1.2)
    content((-5, -1.2), text(size: 7pt)[Hex Keys])
    content((-5, -1.6), text(size: 5pt, fill: diagram-gray)[2.5, 3, 4, 5mm])
    
    // Screwdriver
    tool-screwdriver((-2.5, 0), size: 1.2, tip: "phillips")
    content((-2.5, -1.2), text(size: 7pt)[Screwdriver])
    content((-2.5, -1.6), text(size: 5pt, fill: diagram-gray)[Phillips #2])
    
    // Wrench
    tool-wrench((0, 0), size: 1.2)
    content((0, -1.2), text(size: 7pt)[Wrench])
    content((0, -1.6), text(size: 5pt, fill: diagram-gray)[8, 10, 13mm])
    
    // Multimeter
    tool-multimeter((2.5, 0), size: 1)
    content((2.5, -1.2), text(size: 7pt)[Multimeter])
    content((2.5, -1.6), text(size: 5pt, fill: diagram-gray)[Voltage/Cont.])
    
    // Torque reference
    torque-indicator((5.5, 0), value: "4 Nm", size: 1.2)
    content((5.5, -1.2), text(size: 7pt)[Torque])
    content((5.5, -1.6), text(size: 5pt, fill: diagram-gray)[M5 fasteners])
  }),
  caption: [Required tools: hex keys, screwdriver, wrenches, multimeter. M5 bolts torque to 4 Nm.],
)

== Phase 1: Chassis Frame

The chassis forms a 600mm square base with vertical supports for the electronics plate. Corner brackets provide rigidity without welding.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Isometric view of frame
    let iso = (x, y, z) => (x * 0.7 - y * 0.7, x * 0.4 + y * 0.4 + z)

    // Base rectangle
    line(iso(-2, -2, 0), iso(2, -2, 0), stroke: 1.5pt + black)
    line(iso(2, -2, 0), iso(2, 2, 0), stroke: 1.5pt + black)
    line(iso(2, 2, 0), iso(-2, 2, 0), stroke: 1.5pt + black)
    line(iso(-2, 2, 0), iso(-2, -2, 0), stroke: 1.5pt + black)

    // Vertical posts
    line(iso(-2, -2, 0), iso(-2, -2, 1.5), stroke: 1.5pt + black)
    line(iso(2, -2, 0), iso(2, -2, 1.5), stroke: 1.5pt + black)
    line(iso(2, 2, 0), iso(2, 2, 1.5), stroke: 1.5pt + black)
    line(iso(-2, 2, 0), iso(-2, 2, 1.5), stroke: 1.5pt + black)

    // Corner brackets (simplified)
    for pos in ((-2, -2), (2, -2), (2, 2), (-2, 2)) {
      circle(iso(pos.at(0), pos.at(1), 0), radius: 0.15, fill: gray)
    }

    // Labels
    content(iso(0, -2.5, 0), text(size: 7pt)[Base extrusions])
    content(iso(2.5, 0, 0.75), text(size: 7pt)[Vertical supports])
  }),
  caption: [Chassis frame assembly sequence],
)

*Assembly steps:*

+ Cut extrusions to length if not pre-cut. Deburr all cuts with a file.
+ Assemble the base rectangle (600×600mm) using corner brackets.
+ Verify the frame is square by measuring diagonals (should be equal).
+ Add corner gussets at each joint for additional rigidity.
+ Mount the four vertical supports at the corners.

== Phase 2: Motor Mounting

Each hub motor mounts to a custom bracket that bolts to the extrusion. The bracket positions the wheel axis at the correct height for ground clearance.

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
    content((0, -4), text(size: 6pt)[165mm])
    content((1.8, -4), text(size: 7pt)[Tire])
    
    // Explode arrow from tire to motor
    explode-arrow((0, -2.8), (0, -2.5))
    
    // Assembly direction indicator
    line((3, -4), (3, 4.5), stroke: 1pt + diagram-accent, mark: (end: ">"))
    content((3.8, 0), text(size: 7pt, fill: diagram-accent)[Assembly])
  }),
  caption: [Exploded view: (1) Insert T-nuts into extrusion, (2) Bolt bracket to frame, (3) Attach motor and tire],
)

*Assembly steps:*

#step(1) Slide M5 T-nuts into the extrusion slot at each motor position.

#step(2) Align the motor bracket holes with the T-nuts and secure with M5×10 bolts. Torque to 4 Nm.

#step(3) Mount the hub motor to the bracket using four M4×8 bolts. Ensure the axle is centered.

#step(4) Press the tire onto the hub motor rim. Spin by hand to verify free rotation.

== Phase 3: Electronics Mounting

The electronics plate serves as both a mounting surface and heat sink. VESCs mount with thermal pads to conduct heat into the aluminum plate.

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

*Assembly steps:*

+ Mount the electronics plate to the vertical chassis supports.
+ Install the Jetson module using M3 standoffs.
+ Mount VESCs with thermal pads between the VESC and plate.
+ Install the DC-DC converter and main fuse holder.
+ Route all power wiring before securing with cable ties.

== Phase 4: Wiring

Wiring divides into two domains: high-current power wiring and low-current signal wiring. Keep these separated to reduce electrical noise.

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

*Power wiring:*

+ Connect the battery positive to the main fuse.
+ Wire from fuse output to E-Stop relay input.
+ Connect E-Stop relay output to all four VESCs in parallel.
+ Wire battery to DC-DC input; DC-DC output to Jetson and accessories.
+ Install XT90 connector inline for battery disconnect.

*Signal wiring:*

+ Connect CAN bus in a daisy chain: Jetson → VESC1 → VESC2 → VESC3 → VESC4.
+ Install 120Ω termination resistors at each end of the CAN bus.
+ Wire the E-Stop button in series with the relay coil.
+ Connect the LTE modem to Jetson via USB.

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

Testing proceeds from basic power verification to full system operation. Never skip steps; catching problems early prevents damage.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Testing flowchart
    let boxes = (
      (0, 4, "Power Off\nMultimeter Check"),
      (0, 2.5, "Power On\nNo Motors"),
      (0, 1, "VESC\nInitialization"),
      (0, -0.5, "E-Stop\nTest"),
      (0, -2, "Motor Spin\n(Wheels Up)"),
    )

    for (x, y, label) in boxes {
      rect((x - 1.2, y - 0.5), (x + 1.2, y + 0.5), fill: muni-light-gray, stroke: 1pt + black, radius: 4pt)
      content((x, y), text(size: 6pt)[#label])
    }

    for i in range(4) {
      let y1 = 4 - i * 1.5 - 0.5
      let y2 = 4 - (i + 1) * 1.5 + 0.5
      line((0, y1), (0, y2), stroke: 1pt + black, mark: (end: ">"))
    }
  }),
  caption: [Testing sequence from power-off checks to motor operation],
)

*Testing steps:*

+ *Power off:* Use a multimeter to verify no shorts between battery terminals.
+ *Power on (no motors):* Connect battery, verify Jetson boots, check 12V rail.
+ *VESC initialization:* Confirm all four VESCs show green status LEDs.
+ *E-Stop test:* Press E-Stop, verify power to VESCs is cut.
+ *Motor spin:* With wheels off the ground, command each motor individually.

== Quality Checklist

Before considering the build complete, verify each item on this checklist.

#checklist(
  [All bolts torqued to specification],
  [No exposed wiring or bare conductors],
  [CAN bus termination verified with oscilloscope or by VESC status],
  [E-Stop cuts power within 100ms],
  [All wheels spin freely without rubbing],
  [Battery is secure and protected from impact],
  [All connectors fully seated with positive click],
  [Thermal management verified under load],
)

#pagebreak()

// =============================================================================
= Electrical System
// =============================================================================

The electrical system distributes power from the 48V battery to motors and electronics. This section details the power topology, CAN bus network, and connector pinouts.

== Power Distribution

Power flows from the battery through a single 100A fuse, then splits to three subsystems: motor controllers (VESCs), emergency stop circuit, and DC-DC converter. The E-Stop relay can cut power to the VESCs while leaving the Jetson powered for diagnostics.

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

== Main Components

Each component in the power system serves a specific protective or conversion function.

#spec-table(
  [*Component*], [*Specification*],
  [Battery], [13S4P Li-ion, 48V 20Ah with BMS],
  [Main Fuse], [100A ANL at battery positive],
  [E-Stop], [Normally closed contactor, cuts 48V to VESCs],
  [DCDC], [48V→12V, 20A for Jetson + accessories],
  [VESCs], [4× VESC 6, 60A continuous each],
)

== CAN Bus Topology

The CAN bus connects all motor controllers and the tool interface in a daisy chain. Each end of the bus requires a 120Ω termination resistor to prevent signal reflections.

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

The CAN bus operates at 500 kbps using twisted pair wiring (CANH and CANL). Termination resistors are essential: without them, signal reflections cause communication errors.

== Connectors

Standardized connectors enable quick assembly and field replacement.

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

#spec-table(
  [*Connector*], [*Use*],
  [XT90], [Main battery power (90A rated)],
  [5.5mm bullet], [Motor phase wires (60A rated)],
  [XT30], [12V accessories (30A rated)],
  [JST-XH], [Sensors and buttons (signal level)],
  [Deutsch DT06-6S], [Tool interface (weatherproof)],
)

== VESC Configuration

Each VESC requires configuration via the VESC Tool software before first use. The CAN ID must be unique for each motor controller.

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

Operating the BVR0 requires completing startup procedures, understanding teleoperation controls, and following shutdown protocols. This section covers each phase of operation.

== Startup Procedure

Startup follows a consistent sequence that verifies system health before enabling motor control.

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

*Pre-flight check:* Before connecting power, verify the battery is charged (>40V), the E-Stop is not engaged, wheels are clear of obstructions, and the LTE antenna is connected.

*Power on:* Connect the battery via the XT90 connector. The Jetson will boot automatically, which takes approximately 30 seconds. The onboard display will show the dashboard when ready.

*Connect operator station:* Open the operator interface in a web browser and navigate to the rover's IP address. Verify the video feed is active and telemetry readings are nominal before commanding movement.

== Teleoperation

The operator controls the rover through a web interface that displays video, telemetry, and control inputs.

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

The rover accepts input from keyboard (WASD), gamepad, or touchscreen. Movement commands map the left stick to forward/backward and rotation. The right stick pans the 360° camera view. Speed is adjusted with bumpers or scroll wheel.

=== Control Modes

Three control modes provide different levels of automation.

#spec-table(
  [*Mode*], [*Description*],
  [Direct], [1:1 joystick to motor control, no assistance],
  [Assisted], [Obstacle avoidance prevents collisions],
  [Waypoint], [Autonomous path following between points],
)

== Shutdown Procedure

Proper shutdown protects the electronics and battery.

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

Release all controls so the rover comes to a stop. Disconnect from the operator interface. Press the physical E-Stop button. Finally, disconnect the battery using the XT90 connector and store the rover in a dry location.

== Tool Attachment

Tools connect via a quick-release mechanical mount and a Deutsch DT electrical connector.

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

Power off the rover before attaching tools. Align the tool mount with the front bracket and engage the quick-release latch. Connect the Deutsch DT connector for power and CAN communication. Power on and verify the tool appears in the dashboard.

#pagebreak()

// =============================================================================
= Safety
// =============================================================================

The BVR0 is a powered machine capable of causing injury. This section covers safety protocols, hazard awareness, and emergency procedures.

#danger[
  This is a heavy, powered machine. It can cause serious injury if mishandled. Always maintain situational awareness when operating.
]

== Hazard Awareness

Understanding potential hazards enables safe operation.

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

Keep hands and feet clear of wheels and moving parts at all times. The hub motors can generate significant torque instantly. Never reach under the rover while it is powered.

== Battery Safety

Lithium-ion batteries require careful handling to prevent fire or explosion.

#warning[
  Lithium-ion batteries can catch fire if damaged, overcharged, or short-circuited. Handle with care.
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

Store batteries at room temperature (15-25°C). Never charge batteries unattended. Inspect for physical damage before each use. Do not expose to water or extreme temperatures. Use only the provided charger. Dispose of damaged batteries through proper recycling channels.

== Emergency Stop

Multiple E-Stop mechanisms provide redundant safety.

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

The physical E-Stop button on the rover chassis immediately cuts power to all motors. The software E-Stop (spacebar in operator interface) sends a stop command over the network. If the network connection is lost for more than 2 seconds, the rover automatically stops.

#note[
  To reset after E-Stop: identify and resolve the cause, twist or pull the physical button to release, reconnect the operator interface, and confirm the rover is ready in the dashboard.
]

== Operating Conditions

Environmental limits ensure safe operation.

#spec-table(
  [*Condition*], [*Limit*],
  [Temperature], [-20°C to 40°C],
  [Precipitation], [Light rain/snow only],
  [Wind], [< 40 km/h],
  [Visibility], [Operator must see rover or camera feed],
)

Do not operate on slopes exceeding 15%. Do not operate in standing water deeper than 50mm. Always maintain a clear line of sight or reliable video feed.

#pagebreak()

// =============================================================================
= Maintenance
// =============================================================================

Regular maintenance ensures reliable operation and extends the life of components. This section covers inspection schedules, maintenance procedures, and troubleshooting.

== Regular Inspection

Perform these checks before each operation.

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

== Periodic Maintenance

Scheduled maintenance prevents failures and catches wear before it becomes critical.

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

*Weekly:* Clean debris from wheels and chassis. Wipe camera lenses. Check CAN bus connections. Verify LTE signal strength at operating locations.

*Monthly:* Inspect wiring for chafing or wear. Check bolt torque on motor mounts. Clean battery contacts. Update firmware if available.

*Seasonal:* Perform full electrical inspection. Check bearings on hub motors. Replace worn cables or connectors. Calibrate sensors if needed.

== Storage

Proper storage protects the battery and electronics during periods of non-use.

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

For extended storage (>2 weeks): charge the battery to 50-60% (storage charge), disconnect the battery from the rover, store in a dry location at 15-25°C, cover to protect from dust, and check battery monthly to top up if below 40%.

== Troubleshooting

Common issues and their solutions.

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
