#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Quick Reference Section
// Emergency Stop, Pre-Flight Checklist, Controls
// These are the pages operators flip to most often.

= Quick Reference

These three pages are the ones you'll use daily. Memorize the e-stop methods. Run the pre-flight checklist every time. Keep the controls layout in your head. Everything else in this manual is for building and fixing. This section is for operating.

= Emergency Stop

#danger[
  *Know this page.* If anything goes wrong, use one of these three methods immediately.
]

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Three E-Stop methods - large, clear
    let methods = (
      ((-4, 0), "PHYSICAL\nBUTTON", "Red mushroom\nbutton on rover", "1"),
      ((0, 0), "SPACEBAR", "Press spacebar\non controller", "2"),
      ((4, 0), "DISCONNECT", "Connection loss\nauto-triggers", "3"),
    )

    for (pos, title, desc, num) in methods {
      circle(pos, radius: 1.5, fill: rgb("#FEE2E2"), stroke: 2pt + muni-danger)
      content((pos.at(0), pos.at(1) + 0.3), text(size: 8pt, weight: "bold", fill: muni-danger)[#title])
      content((pos.at(0), pos.at(1) - 0.5), text(size: 6pt)[#desc])
      circle((pos.at(0) - 1.2, pos.at(1) + 1.2), radius: 0.3, fill: muni-danger, stroke: none)
      content((pos.at(0) - 1.2, pos.at(1) + 1.2), text(size: 8pt, fill: white, weight: "bold")[#num])
    }

    line((0, -2), (0, -2.8), stroke: 2pt + muni-danger, mark: (end: ">"))
    rect((-2.5, -4.2), (2.5, -3), fill: muni-danger, stroke: none, radius: 4pt)
    content((0, -3.6), text(fill: white, weight: "bold", size: 12pt)[ALL MOTORS STOP])
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *When to E-Stop:*
    - Person in path of rover
    - Unexpected movement
    - Smoke, sparks, or fire
    - Loss of control
    - Any doubt about safety
  ],
  [
    *To resume after E-Stop:*
    + Resolve the cause
    + Release physical button (if used)
    + Reconnect controller
    + Verify telemetry on dashboard
    + Resume operation
  ]
)

#pagebreak()

// =============================================================================

= Pre-Flight Checklist

#procedure([Daily inspection before operation], time: "2 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    rect((-3, -2), (3, 2), stroke: 1.5pt + diagram-black, radius: 4pt)

    for (x, y) in ((-3, 1.5), (3, 1.5), (-3, -1.5), (3, -1.5)) {
      rect((x - 0.4, y - 0.6), (x + 0.4, y + 0.6), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    }

    let points = (
      ((-3.8, 0), "1"),
      ((3.8, 0), "2"),
      ((0, 0), "3"),
      ((0, -2.8), "4"),
      ((0, 2.8), "5"),
    )

    for (pos, num) in points {
      circle(pos, radius: 0.4, fill: muni-orange, stroke: none)
      content(pos, text(size: 10pt, fill: white, weight: "bold")[#num])
    }

    content((0, 2.3), text(size: 6pt)[FRONT])
  }),
  caption: none,
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 1em,
  [
    #checklist(
      [*1* Wheels spin freely, no debris],
      [*2* All wheel bolts tight],
      [*3* E-Stop button not stuck],
    )
  ],
  [
    #checklist(
      [*4* Battery voltage > 40V],
      [*5* Camera and LiDAR clean],
      [*6* All connectors secure],
    )
  ]
)

#v(1em)

#note[
  If any check fails, do not operate. Resolve the issue first.
]

#v(0.5em)

#lesson[
  We once operated with a loose wheel bolt. 10 minutes in, the wheel nearly came off mid-turn. The 2-minute checklist beats a 2-hour field repair.
]

#pagebreak()

// =============================================================================

= Controls

#procedure([Controller mapping reference], time: "1 min read", difficulty: 1)

#figure(
  cetz.canvas({
    import cetz.draw: *

    rect((-5, -2.5), (5, 2.5), stroke: 1.5pt + diagram-black, radius: 12pt)

    circle((-3, 0), radius: 1, stroke: 1.5pt + diagram-black, fill: diagram-light)
    circle((-3, 0.3), radius: 0.3, fill: diagram-black)

    circle((3, 0), radius: 1, stroke: 1.5pt + diagram-black, fill: diagram-light)
    circle((3, 0), radius: 0.3, fill: diagram-black)

    content((-3, -2), text(size: 9pt, weight: "bold")[MOVEMENT])
    content((3, -2), text(size: 9pt, weight: "bold")[CAMERA])

    content((-3, 1.8), text(size: 6pt)[↑ Forward])
    content((-3, -1.3), text(size: 6pt)[← Turn → ])

    rect((-4.5, 2), (-1.5, 2.3), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    rect((1.5, 2), (4.5, 2.3), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((-3, 2.7), text(size: 7pt)[LB: Speed −])
    content((3, 2.7), text(size: 7pt)[RB: Speed +])

    circle((0, 0.5), radius: 0.6, fill: muni-danger, stroke: none)
    content((0, 0.5), text(fill: white, size: 7pt, weight: "bold")[STOP])
    content((0, -0.5), text(size: 7pt)[Guide Button])

    content((-4.5, 2.7), text(size: 6pt)[LT: Brake])
    content((4.5, 2.7), text(size: 6pt)[RT: Throttle])
  }),
  caption: none,
)

#v(1em)

#spec-table(
  [*Input*], [*Action*],
  [Left Stick Up/Down], [Forward / Reverse],
  [Left Stick Left/Right], [Turn left / right],
  [Right Stick], [Pan camera view],
  [Left Bumper (LB)], [Decrease max speed],
  [Right Bumper (RB)], [Increase max speed],
  [Left Trigger (LT)], [Brake / slow down],
  [Right Trigger (RT)], [Throttle (overrides stick)],
  [Guide Button (center)], [*Emergency Stop*],
  [Spacebar (keyboard)], [*Emergency Stop*],
)

#pagebreak()
