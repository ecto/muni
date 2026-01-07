#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Full-Page Cheat Sheet
// Everything you need at a glance - print this and keep it handy

#pagebreak()

#align(center)[
  #text(size: 16pt, weight: "bold")[BVR0 Cheat Sheet]
  #v(-0.3em)
  #text(size: 8pt, fill: muni-gray)[Print this page. Laminate it. Tape it to your workbench.]
]

#v(0.5em)

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 1em,
  row-gutter: 1em,

  // ===== Column 1 =====
  [
    #box(
      width: 100%,
      stroke: 2pt + muni-danger,
      inset: 8pt,
      radius: 4pt,
      fill: rgb("#FEF2F2"),
    )[
      #text(size: 9pt, weight: "bold", fill: muni-danger)[Emergency Stop]
      #v(0.3em)
      #text(size: 7.5pt)[
        *1.* Red button on rover \
        *2.* Spacebar on controller \
        *3.* Guide button (gamepad) \
        *4.* Connection loss (auto)
      ]
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[CAN Bus IDs]
      #v(0.3em)
      #table(
        columns: (auto, 1fr),
        stroke: none,
        inset: 2pt,
        align: (right, left),
        text(size: 7.5pt, weight: "bold")[0], text(size: 7.5pt)[VESC Front Left],
        text(size: 7.5pt, weight: "bold")[1], text(size: 7.5pt)[VESC Front Right],
        text(size: 7.5pt, weight: "bold")[2], text(size: 7.5pt)[VESC Rear Left],
        text(size: 7.5pt, weight: "bold")[3], text(size: 7.5pt)[VESC Rear Right],
        text(size: 7.5pt, weight: "bold")[10+], text(size: 7.5pt)[Tool Attachments],
        text(size: 7.5pt, weight: "bold")[0x0B00], text(size: 7.5pt)[LED Controller],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Battery Voltages]
      #v(0.3em)
      #table(
        columns: (1fr, auto),
        stroke: none,
        inset: 2pt,
        align: (left, right),
        text(size: 7.5pt)[Full charge], text(size: 7.5pt, weight: "bold")[54.6V],
        text(size: 7.5pt)[Nominal], text(size: 7.5pt)[48V],
        text(size: 7.5pt, fill: muni-orange)[Low warning], text(size: 7.5pt, fill: muni-orange)[42V],
        text(size: 7.5pt, fill: muni-danger)[Cutoff], text(size: 7.5pt, fill: muni-danger, weight: "bold")[39V],
        text(size: 7.5pt)[12V rail], text(size: 7.5pt)[11.5-12.5V],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Torque Specs]
      #v(0.3em)
      #table(
        columns: (1fr, auto),
        stroke: none,
        inset: 2pt,
        align: (left, right),
        text(size: 7.5pt)[Frame (M5)], text(size: 7.5pt, weight: "bold")[4 Nm],
        text(size: 7.5pt)[Motor mounts (M5)], text(size: 7.5pt, weight: "bold")[4 Nm],
        text(size: 7.5pt)[Electronics (M3)], text(size: 7.5pt, weight: "bold")[0.5 Nm],
        text(size: 7.5pt)[Wheel axle], text(size: 7.5pt)[Hand tight],
      )
    ]
  ],

  // ===== Column 2 =====
  [
    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Wire Colors]
      #v(0.3em)
      #table(
        columns: (auto, 1fr),
        stroke: none,
        inset: 2pt,
        align: (center, left),
        box(width: 10pt, height: 10pt, fill: rgb("#ff6600"), radius: 2pt), text(size: 7.5pt)[48V Power (+)],
        box(width: 10pt, height: 10pt, fill: rgb("#1a1a1a"), radius: 2pt), text(size: 7.5pt)[Ground (-)],
        box(width: 10pt, height: 10pt, fill: rgb("#dc2626"), radius: 2pt), text(size: 7.5pt)[12V Power],
        box(width: 10pt, height: 10pt, fill: rgb("#3b82f6"), radius: 2pt), text(size: 7.5pt)[Motor Phase A],
        box(width: 10pt, height: 10pt, fill: rgb("#22c55e"), radius: 2pt), text(size: 7.5pt)[Motor Phase B],
        box(width: 10pt, height: 10pt, fill: rgb("#eab308"), radius: 2pt), text(size: 7.5pt)[Motor Phase C],
        box(width: 10pt, height: 10pt, fill: rgb("#a855f7"), radius: 2pt), text(size: 7.5pt)[CAN High],
        box(width: 10pt, height: 10pt, fill: rgb("#06b6d4"), radius: 2pt), text(size: 7.5pt)[CAN Low],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[LED Status Codes]
      #v(0.3em)
      #table(
        columns: (auto, auto, 1fr),
        stroke: none,
        inset: 2pt,
        align: (center, left, left),
        box(width: 8pt, height: 8pt, fill: rgb("#22c55e"), radius: 4pt), text(size: 7.5pt)[Solid], text(size: 7.5pt)[Ready],
        box(width: 8pt, height: 8pt, fill: rgb("#22c55e"), radius: 4pt), text(size: 7.5pt)[Pulse], text(size: 7.5pt)[Teleop],
        box(width: 8pt, height: 8pt, fill: rgb("#3b82f6"), radius: 4pt), text(size: 7.5pt)[Pulse], text(size: 7.5pt)[Autonomous],
        box(width: 8pt, height: 8pt, fill: rgb("#eab308"), radius: 4pt), text(size: 7.5pt)[Blink], text(size: 7.5pt)[Low Battery],
        box(width: 8pt, height: 8pt, fill: rgb("#dc2626"), radius: 4pt), text(size: 7.5pt)[Solid], text(size: 7.5pt)[E-Stop],
        box(width: 8pt, height: 8pt, fill: rgb("#dc2626"), radius: 4pt), text(size: 7.5pt)[Fast], text(size: 7.5pt)[Fault],
        box(width: 8pt, height: 8pt, fill: rgb("#9ca3af"), radius: 4pt), text(size: 7.5pt)[Off], text(size: 7.5pt)[No Power],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[CLI Commands]
      #v(0.3em)
      #table(
        columns: (1fr, auto),
        stroke: none,
        inset: 2pt,
        align: (left, right),
        text(size: 6.5pt, font: "Berkeley Mono")[muni status], text(size: 6.5pt, fill: muni-gray)[Health],
        text(size: 6.5pt, font: "Berkeley Mono")[muni can scan], text(size: 6.5pt, fill: muni-gray)[Devices],
        text(size: 6.5pt, font: "Berkeley Mono")[muni motors test], text(size: 6.5pt, fill: muni-gray)[Spin],
        text(size: 6.5pt, font: "Berkeley Mono")[muni leds set idle], text(size: 6.5pt, fill: muni-gray)[Reset],
        text(size: 6.5pt, font: "Berkeley Mono")[muni logs -f], text(size: 6.5pt, fill: muni-gray)[Live logs],
        text(size: 6.5pt, font: "Berkeley Mono")[systemctl restart bvrd], text(size: 6.5pt, fill: muni-gray)[Restart],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Network Ports]
      #v(0.3em)
      #table(
        columns: (1fr, auto),
        stroke: none,
        inset: 2pt,
        align: (left, right),
        text(size: 7.5pt)[Hostname], text(size: 7.5pt, font: "Berkeley Mono")[bvr-XX],
        text(size: 7.5pt)[SSH], text(size: 7.5pt, font: "Berkeley Mono")[22],
        text(size: 7.5pt)[WebSocket], text(size: 7.5pt, font: "Berkeley Mono")[8080],
        text(size: 7.5pt)[Video], text(size: 7.5pt, font: "Berkeley Mono")[5600],
        text(size: 7.5pt)[Metrics], text(size: 7.5pt, font: "Berkeley Mono")[8086],
      )
    ]
  ],

  // ===== Column 3 =====
  [
    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Pre-Flight (2 min)]
      #v(0.3em)
      #text(size: 7.5pt)[
        ☐ Battery > 42V \
        ☐ E-Stop released \
        ☐ Wheels spin free \
        ☐ Wheel bolts tight \
        ☐ Connectors secure \
        ☐ Camera/LiDAR clean \
        ☐ Controller paired
      ]
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Quick Troubleshooting]
      #v(0.3em)
      #text(size: 7pt)[
        *No power* \
        → Check breaker, battery \

        *Motors not responding* \
        → Release E-Stop, `muni can scan` \

        *Erratic movement* \
        → Check motor IDs, phase order \

        *Video lag* \
        → Check WiFi, reduce resolution \

        *GPS no fix* \
        → Open sky, check antenna
      ]
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-gray,
      inset: 8pt,
      radius: 4pt,
    )[
      #text(size: 9pt, weight: "bold")[Dimensions]
      #v(0.3em)
      #table(
        columns: (1fr, auto),
        stroke: none,
        inset: 2pt,
        align: (left, right),
        text(size: 7.5pt)[Frame L × W], text(size: 7.5pt)[60 × 50 cm],
        text(size: 7.5pt)[Height], text(size: 7.5pt)[45 cm],
        text(size: 7.5pt)[Wheelbase], text(size: 7.5pt)[45 cm],
        text(size: 7.5pt)[Track width], text(size: 7.5pt)[55 cm],
        text(size: 7.5pt)[Clearance], text(size: 7.5pt)[8 cm],
        text(size: 7.5pt)[Weight (empty)], text(size: 7.5pt)[~15 kg],
      )
    ]

    #v(0.5em)

    #box(
      width: 100%,
      stroke: 1pt + muni-orange,
      inset: 8pt,
      radius: 4pt,
      fill: rgb("#FFF7ED"),
    )[
      #text(size: 9pt, weight: "bold")[Support]
      #v(0.3em)
      #table(
        columns: (auto, 1fr),
        stroke: none,
        inset: 2pt,
        align: (left, left),
        text(size: 7.5pt, weight: "bold")[Web], text(size: 7.5pt)[muni.works],
        text(size: 7.5pt, weight: "bold")[Docs], text(size: 7.5pt)[muni.works/docs],
        text(size: 7.5pt, weight: "bold")[GitHub], text(size: 7.5pt)[github.com/muni-works],
      )
      #v(0.3em)
      #line(length: 100%, stroke: 0.5pt + muni-gray)
      #v(0.3em)
      #grid(
        columns: (1fr, 1fr),
        column-gutter: 0.5em,
        text(size: 6.5pt, fill: muni-gray)[Serial: #box(width: 2.5em, stroke: (bottom: 0.5pt + muni-gray))],
        text(size: 6.5pt, fill: muni-gray)[Build: #box(width: 2.5em, stroke: (bottom: 0.5pt + muni-gray))],
      )
    ]
  ]
)

#pagebreak()
