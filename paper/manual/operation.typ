#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Operation Section
// Startup, Shutdown, Tool attachment

= Operation

Operating the rover is straightforward once it's built and tested. The startup sequence takes about 3 minutes. Shutdown takes 1 minute. Most of that time is waiting for the Jetson to boot.

The key habit is consistency. Use the same startup sequence every time. Check the same indicators. Park in the same spot. Consistent routines catch problems early, when they're small and easy to fix.

= Startup Procedure

#procedure([Power on and connect], time: "3 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    let steps = (
      (0, 4), (0, 2.5), (0, 1), (0, -0.5), (0, -2),
    )
    let labels = (
      "Pre-flight\nChecklist",
      "Connect\nBattery",
      "Wait for\nBoot (30s)",
      "Connect\nController",
      "Verify\nTelemetry",
    )
    let times = ("2 min", "5 sec", "30 sec", "10 sec", "verify")

    for (i, ((x, y), label)) in steps.zip(labels).enumerate() {
      rect((x - 1.4, y - 0.6), (x + 1.4, y + 0.6), fill: diagram-light, stroke: 1pt + diagram-black, radius: 4pt)
      content((x, y), text(size: 8pt)[#label])
      content((2.5, y), text(size: 7pt, fill: diagram-gray)[#times.at(i)])
    }

    for i in range(4) {
      let y1 = 4 - i * 1.5 - 0.6
      let y2 = 4 - (i + 1) * 1.5 + 0.6
      line((0, y1), (0, y2), stroke: 1.5pt + diagram-black, mark: (end: ">"))
    }
  }),
  caption: [Startup takes approximately 3 minutes.],
)

#v(1em)

*Detailed Steps:*

+ *Pre-flight:* Complete checklist on page 2
+ *Battery:* Connect XT90 (hear click). E-Stop should be pressed.
+ *Boot:* Release E-Stop. Wait for Jetson to boot (30s). VESC LEDs turn green.
+ *Controller:* Power on controller. Connect to operator station.
+ *Telemetry:* Verify video feed, voltage reading, and mode indicator.

#v(1em)

#note[
  Do not operate if telemetry shows errors or video feed is absent.
]

#pagebreak()

// =============================================================================

= Shutdown Procedure

#procedure([Safe power-off sequence], time: "1 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    let steps = (
      (0, 3.5), (0, 2), (0, 0.5), (0, -1),
    )
    let labels = (
      "Return to\nHome",
      "Release\nControls",
      "Press\nE-Stop",
      "Disconnect\nBattery",
    )

    for ((x, y), label) in steps.zip(labels) {
      rect((x - 1.4, y - 0.6), (x + 1.4, y + 0.6), fill: diagram-light, stroke: 1pt + diagram-black, radius: 4pt)
      content((x, y), text(size: 8pt)[#label])
    }

    for i in range(3) {
      let y1 = 3.5 - i * 1.5 - 0.6
      let y2 = 3.5 - (i + 1) * 1.5 + 0.6
      line((0, y1), (0, y2), stroke: 1.5pt + diagram-black, mark: (end: ">"))
    }
  }),
  caption: [Always press E-Stop before disconnecting battery.],
)

#v(2em)

*Shutdown Checklist:*

#checklist(
  [Rover parked in designated area],
  [Controller set down / powered off],
  [E-Stop button pressed (red button down)],
  [Wait 5 seconds for Jetson to save state],
  [Disconnect battery (pull XT90)],
  [Store battery properly (50-60% charge for long storage)],
)

#v(1em)

#warning[
  Never disconnect battery while Jetson is running. This can corrupt the filesystem.
]

#v(0.5em)

#pagebreak()

// =============================================================================

= Tool Attachment

#procedure([Attach tools to chassis], time: "15 min", difficulty: 1)

#v(1em)

BVR0 uses direct bolt mounting for tools: no quick-release, no modular interface. Tools attach to the 2020 extrusion frame using T-nuts and bolts, just like everything else on the rover.

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Rover frame (top view)
    rect((-4, -2), (4, 2), stroke: 1.5pt + diagram-black, fill: diagram-light, radius: 2pt)
    content((0, 0), text(size: 8pt, weight: "bold")[BVR0 Frame])

    // Tool mounting area (front)
    rect((-3, 2), (3, 3.5), stroke: 1.5pt + diagram-black, fill: white, radius: 2pt)
    content((0, 2.75), text(size: 7pt)[Tool (e.g., plow)])

    // T-nuts and bolts
    for x in (-2, 0, 2) {
      circle((x, 2), radius: 0.15, fill: diagram-black)
    }
    content((0, 1.6), text(size: 5pt)[M5 bolts + T-nuts])

    // Arrow showing attachment direction
    motion-arrow((0, 4), (0, 3.7))
    content((0, 4.3), text(size: 5pt)[Bolt down])
  }),
  caption: [Tool bolts directly to frame using T-slot hardware.],
)

#v(1em)

*Attachment Procedure:*

+ Power OFF rover (E-Stop pressed)
+ Position tool on frame, align with T-slots
+ Insert M5 bolts through tool mounting holes
+ Thread into T-nuts in frame extrusion
+ Tighten with 4mm hex key (hand-tight + 1/4 turn)
+ Connect power cable (if tool is powered)
+ Power ON rover

*Removal:*

+ Power OFF rover
+ Disconnect power cable first
+ Loosen M5 bolts
+ Lift tool off frame

#v(1em)

#note[
  BVR1 adds a quick-release rail system with electrical pass-through. BVR0 keeps it simple: bolts work.
]

#pagebreak()
