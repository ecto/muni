// BVR0 Datasheet
// Single-page product specification sheet

#import "lib/template.typ": *
#import "lib/diagrams.typ": *

#set document(
  title: "BVR0 Datasheet",
  author: "Municipal Robotics",
)

#set page(
  paper: "us-letter",
  margin: (x: 0.5in, y: 0.5in),
  header: none,
  footer: none,
)

#set text(font: "Times New Roman", size: 9pt)
#set par(justify: true, leading: 0.5em)

// Header
#grid(
  columns: (1fr, auto),
  align: (left, right),
  [
    #text(size: 24pt, weight: "bold")[BVR0]
    #h(1em)
    #text(size: 12pt)[Base Vectoring Rover]
  ],
  [
    #text(size: 9pt, fill: gray)[
      Municipal Robotics \
      Rev 0.1 / December 2025
    ]
  ]
)

#v(0.3em)
#line(length: 100%, stroke: 0.5pt + gray)
#v(0.5em)

// Two-column layout
#grid(
  columns: (1fr, 1fr),
  column-gutter: 1.5em,
  [
    // Left column

    == Description

    The BVR0 is a compact sidewalk-scale robotic platform for snow clearing and grounds maintenance. Four independently-driven hub motors provide omnidirectional control. A modular tool interface accepts snow augers, sweepers, and brine applicators. Remote operation via LTE teleoperation with 360° video.

    #v(0.5em)

    == Mechanical Specifications

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Parameter*], [*Value*],
      [Length], [600 mm (23.6")],
      [Width], [600 mm (23.6")],
      [Height], [400 mm (15.7")],
      [Ground Clearance], [50 mm (2.0")],
      [Wheel Diameter], [160 mm (6.3")],
      [Track Width], [550 mm (21.7")],
      [Wheelbase], [550 mm (21.7")],
      [Weight (dry)], [~25 kg (55 lb)],
      [Weight (w/ battery)], [~30 kg (66 lb)],
      [Max Payload], [10 kg (22 lb)],
    )

    #v(0.5em)

    == Electrical Specifications

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Parameter*], [*Value*],
      [Battery Chemistry], [Li-ion (13S4P)],
      [Nominal Voltage], [48V],
      [Voltage Range], [39V - 54.6V],
      [Capacity], [20 Ah (960 Wh)],
      [Motor Power], [4 × 350W (1.4 kW)],
      [Peak Current], [60A per motor],
      [Control Voltage], [12V DC],
      [CAN Bus], [500 kbps],
    )

    #v(0.5em)

    == Performance

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Parameter*], [*Value*],
      [Max Speed], [5.5 m/s (12 mph)],
      [Operating Speed], [1.0 - 1.5 m/s],
      [Max Wheel RPM], [650 RPM],
      [Max Angular Velocity], [2.5 rad/s],
      [Max Acceleration], [3.0 m/s²],
      [Max Deceleration], [8.0 m/s²],
      [Runtime], [3-4 hours],
      [Max Grade], [15%],
    )

    #v(0.5em)

    == Environmental

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Parameter*], [*Value*],
      [Operating Temp], [-20°C to +40°C],
      [Storage Temp], [-30°C to +50°C],
      [IP Rating], [IP54 (splash resistant)],
      [Precipitation], [Light rain/snow],
      [Max Wind], [40 km/h],
    )
  ],
  [
    // Right column

    == Sensors

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Sensor*], [*Specification*],
      [LiDAR], [Livox Mid-360],
      [  - FOV], [360° × 59° (-7° to +52°)],
      [  - Range], [0.1m - 70m],
      [  - Points], [200,000 pts/sec],
      [  - IMU], [Built-in, 200 Hz],
      [Camera], [Insta360 X4],
      [  - Resolution], [8K (5.7K @ 30fps)],
      [  - FOV], [360° spherical],
      [  - Interface], [USB-C (UVC)],
    )

    #v(0.5em)

    == Compute

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Component*], [*Specification*],
      [Processor], [Jetson Orin NX 16GB],
      [  - CPU], [8-core Arm Cortex-A78AE],
      [  - GPU], [1024-core Ampere],
      [  - AI], [100 TOPS],
      [Connectivity], [Sierra MC7455 LTE],
      [  - Bands], [LTE Cat 6],
      [  - Speed], [300 Mbps down],
      [Display], [7" HDMI (optional)],
    )

    #v(0.5em)

    == Motor Controllers

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Parameter*], [*Value*],
      [Controller], [VESC 6.6 × 4],
      [Continuous Current], [60A],
      [Peak Current], [150A],
      [Control Mode], [FOC (Field Oriented)],
      [Feedback], [Hall sensors],
      [Motor Type], [32-pole BLDC hub],
    )

    #v(0.5em)

    == Connectors

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Interface*], [*Connector*],
      [Battery], [XT90 (90A rated)],
      [Motor Phase], [5.5mm bullet],
      [12V Aux], [XT30 (30A rated)],
      [Tool Interface], [Deutsch DT06-6S],
      [CAN Bus], [JST-XH 4-pin],
    )

    #v(0.5em)

    == Tool Interface

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Pin*], [*Function*],
      [1], [12V Power],
      [2], [GND],
      [3], [CAN High],
      [4], [CAN Low],
      [5], [Reserved],
      [6], [Reserved],
    )

    #v(0.5em)

    == Safety Features

    - Physical E-Stop button (mushroom, NC relay)
    - Software E-Stop via operator interface
    - Automatic stop on connection loss (>2s)
    - LiDAR-based obstacle detection (1.5m radius)
    - 100A main fuse at battery
  ]
)

#v(0.5em)
#line(length: 100%, stroke: 0.5pt + gray)
#v(0.3em)

// Dimensions drawing placeholder
#grid(
  columns: (1fr, 1fr),
  column-gutter: 1em,
  [
    == Dimensions

    #figure(
      cetz.canvas({
        import cetz.draw: *

        // Chassis
        rect((-2, -2), (2, 2), stroke: 1pt + diagram-black, radius: 2pt)

        // Wheels at corners
        for (x, y) in ((-2.15, 1.85), (2.15, 1.85), (-2.15, -1.85), (2.15, -1.85)) {
          rect((x - 0.3, y - 0.5), (x + 0.3, y + 0.5), fill: diagram-black, radius: 1pt)
        }

        // Electronics bay
        rect((-1.2, -1.2), (1.2, 0.5), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 1pt)

        // Sensor mast
        circle((0, 0.8), radius: 0.2, fill: diagram-black)

        // Tool mount
        rect((-0.8, 1.7), (0.8, 2), fill: diagram-light, stroke: 0.5pt + diagram-gray)

        // Dimensions
        dim-h(-2, -2, 2, "600 mm", offset: 1)
        dim-v(2, -2, 2, "600 mm", offset: 1)

        // Front indicator
        motion-arrow((0, 2.3), (0, 2.8))
        content((0, 3.1), text(size: 6pt)[FRONT])
      }),
      caption: [Top view],
    )
  ],
  [
    == Ordering Information

    #table(
      columns: (1fr, 1fr),
      stroke: 0.5pt + gray,
      inset: 5pt,
      [*Part Number*], [*Description*],
      [BVR0-BASE], [Rover platform (no tools)],
      [BVR0-AUGER], [Snow auger attachment],
      [BVR0-BRINE], [Brine sprayer attachment],
      [BVR0-SWEEP], [Sweeper attachment],
    )

    #v(0.5em)

    == Included

    - BVR0 rover platform
    - 48V 20Ah battery pack
    - Battery charger
    - XT90 battery cable
    - Quick start guide

    #v(0.5em)

    == Not Included

    - Tool attachments (sold separately)
    - LTE SIM card
    - Operator workstation
  ]
)

#v(1em)

#align(center)[
  #text(size: 8pt, fill: gray)[
    Municipal Robotics · Cleveland, Ohio · muni.works · info\@muni.works \
    Specifications subject to change without notice. December 2025.
  ]
]
