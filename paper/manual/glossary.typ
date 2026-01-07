#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Glossary and Index
// Common terms and abbreviations

= Glossary

This manual uses a lot of acronyms and technical terms. If you encounter an unfamiliar term, check here first.

#v(0.5em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    #glossary-entry([AWG], [American Wire Gauge. Lower numbers = thicker wire. 10 AWG for power, 22 AWG for signals.])

    #glossary-entry([BLDC], [Brushless DC motor. Uses electronic commutation instead of brushes.])

    #glossary-entry([BVR], [Base Vectoring Rover. Muni's first rover morphology.])

    #glossary-entry([CAN], [Controller Area Network. Industrial communication bus used for motor control.])

    #glossary-entry([CAN_H / CAN_L], [CAN High and CAN Low. Differential pair signals.])

    #glossary-entry([CCW / CW], [Counter-clockwise / Clockwise. Motor rotation direction.])

    #glossary-entry([DC-DC], [DC-DC converter. Steps voltage down (48V → 12V).])

    #glossary-entry([DT Connector], [Deutsch DT connector. Weatherproof automotive connector.])

    #glossary-entry([E-Stop], [Emergency Stop. Cuts power to motors immediately.])

    #glossary-entry([FOC], [Field-Oriented Control. Advanced motor control algorithm for smooth, efficient operation.])

    #glossary-entry([GPIO], [General Purpose Input/Output. Digital pins on the Jetson.])

    #glossary-entry([Hub Motor], [Motor integrated into the wheel hub. No external gears or chains.])
  ],
  [
    #glossary-entry([Jetson], [NVIDIA Jetson. Embedded AI computer. BVR0 uses Orin NX.])

    #glossary-entry([LiDAR], [Light Detection and Ranging. Laser-based 3D sensor.])

    #glossary-entry([LiPo], [Lithium Polymer battery. High energy density, requires careful handling.])

    #glossary-entry([LTE], [Long-Term Evolution. Cellular data connection.])

    #glossary-entry([Nm], [Newton-meter. Unit of torque.])

    #glossary-entry([RTK], [Real-Time Kinematic. GPS correction for centimeter accuracy.])

    #glossary-entry([Skid-Steer], [Steering method where left and right sides drive at different speeds.])

    #glossary-entry([T-Nut], [Threaded nut that slides into aluminum extrusion T-slots.])

    #glossary-entry([Teleop], [Teleoperation. Remote control of the rover by a human operator.])

    #glossary-entry([Termination], [120Ω resistor at CAN bus endpoints to prevent signal reflection.])

    #glossary-entry([VESC], [Vedder Electronic Speed Controller. Open-source motor controller.])

    #glossary-entry([XT Connector], [Amass XT series. Yellow power connectors (XT90, XT60, XT30).])
  ]
)

#pagebreak()

= Index

Page numbers reference the first significant mention of each topic. See the Table of Contents for section-level navigation.

#v(0.5em)

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 1.5em,
  row-gutter: 0.3em,
  [
    *A*
    - Aluminum extrusion, 7-9
    - AWG (wire gauge), 18

    *B*
    - Battery, 13-14, 28
    - Battery safety, 28
    - BOM (Bill of Materials), 30-31
    - Bolts, torque values, 9

    *C*
    - Cable management, 21
    - CAN bus, 17-18
    - CAN IDs, 15
    - Charging, 28
    - Checklist, pre-flight, 2
    - Connectors, 18-19
    - Controls, 3
    - Corner brackets, 8
  ],
  [
    *D*
    - DC-DC converter, 14
    - Diagnostics, 27
    - Difficulty ratings, page headers

    *E*
    - E-Stop, 1, 23
    - Electronics plate, 10
    - Extrusions, cutting, 7

    *F*
    - First power-up, 22
    - Firmware, 26
    - Frame assembly, 8

    *G*
    - GPIO, 16
    - Glossary, 32

    *H*
    - Hazard zones, 27
    - Hub motors, 11
  ],
  [
    *J*
    - Jetson mounting, 15

    *L*
    - LiDAR, 5

    *M*
    - Maintenance, 26
    - Motor testing, 23
    - Motor wiring, 19

    *P*
    - Phase wires, 19
    - PPE, 27
    - Power system, 13-14

    *Q*
    - Quick reference, 1-3

    *S*
    - Safety, 27-28
    - Sensor mast, 12
    - Shutdown, 24
    - Specifications, 5
    - Squareness check, 9
    - Startup, 24
    - Storage, 26

    *T*
    - T-nuts, 8
    - Tool attachment, 25
    - Troubleshooting, 26

    *V*
    - VESC config, 15, 22
    - VESC mounting, 14
    - Vertical posts, 8

    *W*
    - Wiring schematic, 17
  ]
)

#pagebreak()
