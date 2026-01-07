#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Hardware Reference
// 1:1 scale drawings for identification

= Hardware Reference

#scale-indicator()

Print this page at 100% scale. Use to verify hardware sizes.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Title
    content((0, 6), text(size: 10pt, weight: "bold")[Bolt Sizes (Socket Head Cap Screw)])

    // M3
    screw-actual-size((-4, 3), thread: "M3", length: 8)
    content((-4, 1.5), text(size: 7pt)[Electronics])

    // M4
    screw-actual-size((-1.5, 3), thread: "M4", length: 10)
    content((-1.5, 1.5), text(size: 7pt)[Motors])

    // M5
    screw-actual-size((1, 3), thread: "M5", length: 12)
    content((1, 1.5), text(size: 7pt)[Frame])

    // M5 long
    screw-actual-size((3.5, 3), thread: "M5", length: 16)
    content((3.5, 1.5), text(size: 7pt)[Standoffs])
  }),
  caption: none,
)

#v(2em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    content((0, 3), text(size: 10pt, weight: "bold")[Washer and Nut Sizes])

    // M3 washer
    washer((-4, 0), outer: 0.2, inner: 0.08)
    content((-4, -0.8), text(size: 7pt)[M3 washer])

    // M4 washer
    washer((-1.5, 0), outer: 0.25, inner: 0.1)
    content((-1.5, -0.8), text(size: 7pt)[M4 washer])

    // M5 washer
    washer((1, 0), outer: 0.32, inner: 0.13)
    content((1, -0.8), text(size: 7pt)[M5 washer])

    // M5 nut
    nut-top((3.5, 0), size: 0.28)
    content((3.5, -0.8), text(size: 7pt)[M5 nut])
  }),
  caption: none,
)

#v(2em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Wire Gauges (Cross-Section)*

    #figure(
      cetz.canvas({
        import cetz.draw: *

        // 10 AWG (power)
        circle((-1.5, 0), radius: 0.13, fill: diagram-accent, stroke: none)
        content((-1.5, -0.5), text(size: 6pt)[10 AWG])
        content((-1.5, -0.8), text(size: 5pt, fill: gray)[48V Power])

        // 18 AWG (signal power)
        circle((0, 0), radius: 0.05, fill: rgb("#3b82f6"), stroke: none)
        content((0, -0.5), text(size: 6pt)[18 AWG])
        content((0, -0.8), text(size: 5pt, fill: gray)[12V])

        // 22 AWG (signal)
        circle((1.5, 0), radius: 0.03, fill: diagram-black, stroke: none)
        content((1.5, -0.5), text(size: 6pt)[22 AWG])
        content((1.5, -0.8), text(size: 5pt, fill: gray)[CAN/Signal])
      }),
      caption: none,
    )
  ],
  [
    *Connector Sizes*

    #figure(
      cetz.canvas({
        import cetz.draw: *

        // XT90
        connector-xt((-1.5, 0), size: "90")

        // XT60
        connector-xt((0.5, 0), size: "60")

        // XT30
        connector-xt((2, 0), size: "30")
      }),
      caption: none,
    )
  ]
)

#v(1em)

#note[
  If hardware doesn't match these silhouettes at 100% print scale, verify your print settings. Some PDF viewers default to "Fit to Page" which scales incorrectly.
]

#pagebreak()
