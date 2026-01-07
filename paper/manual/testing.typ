#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Testing Section
// Pre-power checks, VESC config, First power-up, Motor testing

= Testing

This is the moment of truth. You've built the mechanical system, wired the electronics, and configured the software. Now you find out if it all works together.

Go slow. The pre-power checks exist because we've made every mistake in this section at least once. A reversed connector, a missed termination resistor, a VESC with the wrong ID, these are easy to fix *before* you apply power and much harder after.

The first power-up should be anticlimactic. If you smell burning, hear crackling, or see smoke: power off immediately. Something is wrong, and continuing will make it worse.

= Pre-Power Checks

#procedure([Verify connections before first power-on], time: "10 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Multimeter
    tool-multimeter((0, 0), size: 2)
    content((0, -2.5), text(size: 9pt, weight: "bold")[Multimeter Tests])
  }),
  caption: none,
)

#v(1em)

*Continuity Tests (power OFF):*

#spec-table(
  [*Test*], [*Probe Points*], [*Expected*],
  [48V+ to GND], [Battery connector pins], [Open (no beep)],
  [12V+ to GND], [DC-DC output], [Open (no beep)],
  [CAN_H to CAN_L], [CAN connector], [~60Ω (two 120Ω in parallel)],
  [Phase A to B], [Motor connector], [Low resistance (motor windings)],
)

#v(1em)

*Visual Inspection:*

#checklist(
  [No exposed wire or bare conductors],
  [All connectors fully seated],
  [Polarity correct (red to +, black to -)],
  [No pinched wires],
  [Fuse installed and correct rating],
  [E-Stop button in pressed (safe) position],
)

#v(1em)

#danger[
  If any continuity test shows a short (beep) between power and ground, DO NOT APPLY POWER. Find and fix the short first.
]

#v(0.5em)

#lesson[
  A reversed XT60 connector on the DC-DC input fried a \$35 converter instantly. The 30-second continuity check would have caught it.
]

#pagebreak()

// =============================================================================

= First Power-Up

#procedure([Initial power-on sequence], time: "5 min", difficulty: 2)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    let steps = (
      (0, 4.5, "1", "E-Stop\nPressed"),
      (3.5, 4.5, "2", "Connect\nBattery"),
      (7, 4.5, "3", "Check\nVoltage"),
      (10.5, 4.5, "4", "Release\nE-Stop"),
      (14, 4.5, "5", "Watch\nLEDs"),
    )

    for (x, y, num, label) in steps {
      process-box((x, y), label, width: 2.8, height: 1.3)
      callout((x - 1.1, y + 0.5), num)
    }

    for i in range(4) {
      flow-arrow((i * 3.5 + 1.4, 4.5), ((i + 1) * 3.5 - 1.4, 4.5))
    }

    // Second row
    let steps2 = (
      (0, 1.5, "6", "Jetson\nBoots"),
      (3.5, 1.5, "7", "Wait\n30 sec"),
      (7, 1.5, "8", "SSH\nConnect"),
      (10.5, 1.5, "9", "Check\nServices"),
    )

    for (x, y, num, label) in steps2 {
      process-box((x, y), label, width: 2.8, height: 1.3)
      callout((x - 1.1, y + 0.5), num)
    }

    for i in range(3) {
      flow-arrow((i * 3.5 + 1.4, 1.5), ((i + 1) * 3.5 - 1.4, 1.5))
    }

    // Down arrow between rows
    line((14, 3.9), (14, 3), stroke: 1.5pt + diagram-black)
    line((14, 3), (0, 3), stroke: 1.5pt + diagram-black)
    line((0, 3), (0, 2.1), stroke: 1.5pt + diagram-black, mark: (end: ">"))

    // Final check
    check-mark((12.5, 1.5), size: 0.5)
  }),
  caption: none,
)

#v(1em)

*What to Watch:*

#spec-table(
  [*Indicator*], [*Normal*], [*Problem*],
  [VESC LEDs], [Solid green], [Red = fault, none = no power],
  [Jetson LED], [Solid then blinking], [None = power issue],
  [DC-DC LED], [Green (if equipped)], [None = input voltage issue],
  [Smell], [None], [Burning = immediate power off],
  [Sound], [Quiet hum], [Buzzing = loose connection],
)

#pagebreak()

// =============================================================================

= VESC Configuration

#procedure([Configure motor controllers], time: "15 min per VESC", difficulty: 3)

#grid(
  columns: (1fr, auto),
  column-gutter: 1em,
  [
    Configure motor controllers using VESC Tool.

    *Connection:*
  ],
  video-link("https://muni.works/docs/vesc", [VESC Setup Walkthrough]),
)
+ Connect laptop to VESC via USB
+ Open VESC Tool
+ Select serial port, click Connect

*Motor Wizard:*
+ Navigate to Motor → Motor Wizard
+ Select motor type (usually "Large outrunner")
+ Run detection: VESC will spin motor briefly
+ Review detected parameters
+ Write configuration to VESC

*CAN Configuration:*
+ Navigate to App → CAN Status
+ Set unique Controller ID (0, 1, 2, 3)
+ Set CAN Baud to 500K
+ Enable "Send CAN Status"
+ Write configuration

#v(1em)

*Per-VESC Settings:*

#spec-table(
  [*VESC*], [*ID*], [*Motor Direction*],
  [Front Left], [0], [Forward = CCW],
  [Front Right], [1], [Forward = CW],
  [Rear Left], [2], [Forward = CCW],
  [Rear Right], [3], [Forward = CW],
)

#note[
  Left and right motors spin opposite directions for forward motion in skid-steer.
]

#v(0.5em)

#pitfall[
  Swapping VESC IDs 0↔1 or 2↔3 makes the rover spin in circles instead of driving straight. The CAN ID diagram on the Quick Reference Card saves debugging time.
]

#pagebreak()

// =============================================================================

= Motor Testing

#procedure([Verify motor response], time: "10 min", difficulty: 2)

#v(1em)

#warning[
  Elevate rover so all wheels are off the ground before motor testing.
]

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Rover on blocks
    rect((-3, 1), (3, 2), stroke: 1.5pt + diagram-black, fill: diagram-light, radius: 2pt)
    content((0, 1.5), text(size: 8pt)[BVR0])

    // Wheels spinning
    for (x, dir) in ((-3.5, "CCW"), (3.5, "CW")) {
      circle((x, 0.5), radius: 0.6, stroke: 1.5pt + diagram-black, fill: white)
      // Rotation arrow
      arc((x, 0.5), start: 45deg, stop: 315deg, radius: 0.8, stroke: 1pt + muni-orange, mark: (end: ">"))
      content((x, -0.8), text(size: 6pt)[#dir])
    }

    // Support blocks
    rect((-2, 0), (-1, 1), fill: diagram-gray, stroke: 1pt + diagram-black)
    rect((1, 0), (2, 1), fill: diagram-gray, stroke: 1pt + diagram-black)
    content((0, 0.5), text(size: 6pt)[Blocks])

    // Ground
    line((-4.5, 0), (4.5, 0), stroke: 1pt + diagram-black)
  }),
  caption: [Test with wheels elevated. Verify each motor spins correct direction.],
)

#v(1em)

*Test Procedure:*

+ Elevate rover on blocks (all wheels free)
+ Power on, release E-Stop
+ Connect controller
+ Command forward slowly: all wheels should spin "forward"
+ Command reverse: all wheels should spin "backward"
+ Command left turn: right wheels forward, left wheels backward
+ Test E-Stop: press button, verify immediate stop

*Direction Fix:*
If a motor spins wrong direction, swap any two phase wires on that motor.

#v(1em)

#checklist(
  [All 4 motors respond to commands],
  [Direction correct for each motor],
  [E-Stop stops all motors immediately],
  [No unusual sounds or vibration],
  [VESCs not overheating],
)

#v(0.5em)

#lesson[
  The first road test had the rover on flat ground. It took off at 2 m/s and hit a wall. Always test on blocks first, with throttle at minimum.
]

#pagebreak()
