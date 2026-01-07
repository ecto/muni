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

#lesson[
  One field test ended with a dead battery mid-session. The Jetson filesystem corrupted and needed a full reflash. Now we monitor voltage religiously and shut down at 42V.
]

#pagebreak()

// =============================================================================

= Tool Attachment

#procedure([Attach modular tools], time: "2 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Three-step process
    let pw = 4.5
    let ph = 3

    // Step 1: Approach
    step-panel((0, 0), size: (pw, ph), step-num: 1, title: "Align")
    rect((0.5, 0.5), (2, 2.2), stroke: 1pt + diagram-black, radius: 2pt)
    content((1.25, 1.35), text(size: 6pt)[Rover])
    rect((2.8, 0.5), (4.3, 2.2), stroke: 1pt + diagram-black, radius: 2pt)
    content((3.55, 1.35), text(size: 6pt)[Tool])
    motion-arrow((2.2, 1.35), (2.6, 1.35))

    panel-arrow-h((0, 0), from-size: (pw, ph), gap: 0.5)

    // Step 2: Connect mechanical
    step-panel((pw + 0.5, 0), size: (pw, ph), step-num: 2, title: "Latch")
    rect((pw + 1, 0.5), (pw + 4, 2.2), stroke: 1pt + diagram-black, radius: 2pt)
    line((pw + 2.5, 0.5), (pw + 2.5, 2.2), stroke: 0.5pt + diagram-gray)
    rect((pw + 2.3, 1.5), (pw + 2.7, 1.9), fill: muni-success, stroke: none, radius: 2pt)
    content((pw + 2.5, 2.5), text(size: 5pt, fill: muni-success)[Click!])

    panel-arrow-h((pw + 0.5, 0), from-size: (pw, ph), gap: 0.5)

    // Step 3: Connect electrical
    step-panel((2 * (pw + 0.5), 0), size: (pw, ph), step-num: 3, title: "Connect")
    rect((2 * (pw + 0.5) + 0.5, 0.5), (2 * (pw + 0.5) + 4, 2.2), stroke: 1pt + diagram-black, radius: 2pt)
    connector-dt((2 * (pw + 0.5) + 2.25, 0.2), pins: 4)
    content((2 * (pw + 0.5) + 2.25, -0.5), text(size: 5pt)[DT connector])
  }),
  caption: [Tool attachment: align rails, latch, then connect electrical.],
)

#v(1em)

*Attachment Procedure:*

+ Power OFF rover (E-Stop pressed)
+ Align tool mounting rails with rover interface
+ Slide tool forward until latch clicks (audible)
+ Verify latch indicator shows green/locked
+ Connect DT electrical connector (power + CAN)
+ Power ON rover
+ Tool announces itself automatically on CAN bus
+ Operator UI shows tool status

*Detachment:*

+ Power OFF rover
+ Disconnect DT electrical connector first
+ Release latch lever
+ Slide tool rearward to remove

#v(1em)

#note[
  Always disconnect electrical before unlatching mechanical. Prevents arcing.
]

#pagebreak()
