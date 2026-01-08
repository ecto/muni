#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Bill of Materials Appendix
// Complete parts list with vendors and approximate costs

= Bill of Materials

This is everything you need to build one BVR0 from scratch. Prices are approximate as of late 2025 and vary by region and vendor.

A few notes on sourcing: the expensive items (Jetson, LiDAR, camera) are worth buying from authorized distributors for warranty support. The commodity items (extrusions, fasteners, wire) can come from anywhere. The hub motors are sourced from AliExpress because that's where they're cheapest; allow 2-3 weeks for shipping.

The total cost (~\$4,200) is roughly split: one-third mechanical, one-third power/drive, one-third compute/sensors. If you're building multiple units, the mechanical and power costs drop significantly with bulk ordering.

#v(1em)

== Structural

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [2020 Aluminum Extrusion, 1m], [6], [\$8], [\$48], [Amazon / 8020.net],
  [90° Corner Bracket], [16], [\$1.50], [\$24], [Amazon],
  [M5×10 Button Head Bolt], [64], [\$0.15], [\$10], [McMaster],
  [M5 T-Nut (drop-in)], [64], [\$0.20], [\$13], [Amazon],
  [M5 T-Nut (pre-load)], [32], [\$0.15], [\$5], [Amazon],
  [25mm Aluminum Tube (mast)], [1], [\$15], [\$15], [Local metal supplier],
)

#v(1em)

== Drivetrain

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [Hub Motor 350W 48V], [4], [\$85], [\$340], [AliExpress / ODrive],
  [VESC 6.7 (Flipsky)], [4], [\$120], [\$480], [Flipsky],
  [M5×12 Button Head Bolt], [8], [\$0.15], [\$2], [McMaster],
  [4mm Bullet Connectors], [24], [\$0.50], [\$12], [Amazon],
)

#v(1em)

== Power

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [48V 20Ah Downtube Battery], [1], [\$400], [\$400], [Unit Pack Power / Luna],
  [E-Stop Mushroom Button], [1], [\$15], [\$15], [Amazon],
  [XT60 Connector Pair], [8], [\$2], [\$16], [Amazon],
  [DC-DC 48V→12V 10A], [1], [\$35], [\$35], [Amazon],
  [10 AWG Silicone Wire (red)], [3m], [\$2/m], [\$6], [Amazon],
  [10 AWG Silicone Wire (black)], [3m], [\$2/m], [\$6], [Amazon],
)

#pagebreak()

== Electronics

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [Jetson Orin NX 16GB], [1], [\$600], [\$600], [Seeed / Arrow],
  [Carrier Board w/ CAN], [1], [\$130], [\$130], [Seeed / Waveshare],
  [USB 3.0 Hub (powered)], [1], [\$25], [\$25], [Amazon],
  [LTE Modem (USB)], [1], [\$50], [\$50], [Amazon],
  [22 AWG Wire (CAN, assorted)], [10m], [\$0.50/m], [\$5], [Amazon],
  [JST-XH 4-pin Connector], [10], [\$0.50], [\$5], [Amazon],
  [Electrical Tape], [2 rolls], [\$3], [\$6], [Amazon],
  [Velcro Strips], [1 pack], [\$8], [\$8], [Amazon],
)

#v(1em)

== Sensors

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [Livox Mid-360 LiDAR], [1], [\$1,000], [\$1,000], [Livox],
  [Insta360 X4 Camera], [1], [\$500], [\$500], [Insta360],
  [RTK GPS Module (optional)], [1], [\$200], [\$200], [SparkFun],
)

#v(1em)

== Miscellaneous

#table(
  columns: (2fr, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 6pt,
  fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
  [*Part*], [*Qty*], [*Unit*], [*Total*], [*Source*],
  [Zip Ties (assorted)], [1 pack], [\$8], [\$8], [Amazon],
  [Heat Shrink Tubing], [1 kit], [\$12], [\$12], [Amazon],
  [Loctite 243 (blue)], [1], [\$8], [\$8], [Amazon],
  [Dielectric Grease], [1], [\$6], [\$6], [Amazon],
  [Cable Sleeve (split loom)], [5m], [\$1/m], [\$5], [Amazon],
)

#v(2em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Subtotals*
    #table(
      columns: (1fr, auto),
      stroke: none,
      inset: 4pt,
      [Structural], [\$115],
      [Drivetrain], [\$834],
      [Power], [\$478],
      [Electronics], [\$829],
      [Sensors], [\$1,700],
      [Misc], [\$39],
    )
  ],
  [
    #v(1em)
    #box(
      width: 100%,
      fill: muni-light-gray,
      inset: 12pt,
      radius: 4pt,
    )[
      #text(size: 14pt, weight: "bold")[Total: ~\$3,995]
      #v(0.3em)
      #text(size: 8pt, fill: muni-gray)[
        Excludes tools, shipping, and taxes. \
        Prices vary by region and vendor.
      ]
    ]
  ]
)

#pagebreak()
