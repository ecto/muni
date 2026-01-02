// Municipal Robotics Pre-Seed Pitch Deck
// Q1 2026

#import "lib/template.typ": *

#set document(title: "Municipal Robotics - Pre-Seed Pitch Deck", author: "Municipal Robotics")

#set page(
  paper: "presentation-16-9",
  margin: 0.5in,
  numbering: none,
)

// Presentation text size
#set text(font: muni-font, size: 11pt, tracking: muni-tracking)
#set par(justify: false, leading: muni-leading)  // Left-aligned for slides

// =============================================================================
// Slide 1: Title
// =============================================================================

#page[
  #align(center + horizon)[
    #text(size: 14pt, fill: gray)[Pre-Seed Investment Opportunity]

    #v(1em)

    #text(size: 16pt, style: "italic")[The] \
    #v(0.05in)
    #text(size: 48pt, weight: "bold")[Municipal Robotics] \
    #v(0.1in)
    #text(size: 16pt, style: "italic")[Corporation of] \
    #text(size: 24pt)[Cleveland, Ohio]

    #v(0.1in)

    #text(size: 18pt)[builds _autonomous_ vehicles \ *to improve public spaces*]

    #v(1em)

    #text(size: 14pt, fill: gray)[
      Cam Pedersen, Founder \
      info\@muni.works
    ]
  ]
]

// =============================================================================
// Slide 2: Problem
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[The Problem]

  #v(0.3in)

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.5in,
    [
      #text(size: 18pt, weight: "bold")[Sidewalks are failing Americans]

      - *1 million+ slip-and-fall injuries* per year from icy sidewalks
      - *\$35 billion* in annual liability costs for municipalities
      - *20% of Americans* have mobility challenges requiring clear paths
      - Parents, elderly, disabled forced into the street

      #v(0.3in)

      #text(size: 18pt, weight: "bold")[Why it's not getting fixed]

      - Manual labor: expensive, unavailable, unsafe
      - Existing equipment: too big for sidewalks
      - Property owner mandates: unenforceable
      - *No good solution exists today*
    ],
    [
      #align(center)[
        #image("images/pedestrian-road.jpg", width: 100%)

        #v(0.1in)

        #text(size: 12pt, fill: gray)[
          Cities face impossible tradeoffs: \
          clear sidewalks or fund other services
        ]
      ]
    ]
  )
]

// =============================================================================
// Slide 3: Solution
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Our Solution]

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.5in,
    [
      #align(center)[
        #image("images/rover.jpg", width: 85%)
      ]
    ],
    [
      #text(size: 20pt, weight: "bold")[BVR: Base Vectoring Rover]

      Electric rover designed for sidewalk-scale work. bvr1 in development.

      #v(0.2in)

      - *Sidewalk-sized:* 24" wide, fits anywhere pedestrians walk
      - *Electric:* Zero emissions, quiet operation
      - *Modular tools:* Auger, plow, spreader, mower
      - *Safe by design:* LiDAR stops on any obstacle
      - *Teleoperated:* One operator monitors 10+ rovers

      #v(0.3in)

      #text(size: 16pt, weight: "bold", fill: muni-orange)[
        10x more efficient than manual clearing
      ]
    ]
  )
]

// =============================================================================
// Slide 4: Demo
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Progress]

  #v(0.2in)

  #align(center)[
    #grid(
      columns: 3,
      gutter: 16pt,
      [
        #image("images/prototype-drift.png", height: 1.6in)
        #v(4pt)
        #text(size: 11pt, weight: "bold")[BVR0 Prototype]
        #v(2pt)
        #text(size: 10pt, fill: gray)[Operational December 2025]
      ],
      [
        #image("images/bvr0-disassembled.jpg", height: 1.6in)
        #v(4pt)
        #text(size: 11pt, weight: "bold")[Field-Repairable]
        #v(2pt)
        #text(size: 10pt, fill: gray)[Off-the-shelf parts, \$5k BOM]
      ],
      [
        #image("images/wheel-snow.jpg", height: 1.6in)
        #v(4pt)
        #text(size: 11pt, weight: "bold")[Winter Tested]
        #v(2pt)
        #text(size: 10pt, fill: gray)[Real snow, real conditions]
      ],
    )

    #v(0.2in)

    #box(
      fill: muni-light-gray,
      radius: 6pt,
      inset: 12pt,
    )[
      #text(size: 12pt)[
        #text(fill: muni-orange, weight: "bold")[Proven:] Drivetrain #sym.dot Teleoperation #sym.dot GPS #sym.dot MCU firmware #sym.dot Depot deployment \
        #text(fill: muni-orange, weight: "bold")[Now (Artifact):] BVR1 R&D #sym.dot Supervised autonomy #sym.dot Production unit
      ]
    ]
  ]
]

// =============================================================================
// Slide 5: Market
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Market Opportunity]

  #text(size: 18pt, weight: "bold")[Total Addressable Market: \$14B+]

  #table(
    columns: (2fr, 1fr, 2fr),
    stroke: none,
    inset: 8pt,
    fill: (_, row) => if calc.odd(row) { muni-light-gray } else { white },

    [*Segment*], [*TAM*], [*Entry Strategy*],
    [Municipal sidewalks], [\$2B], [Pilot program (current focus)],
    [Municipal parks/paths], [\$3B], [Same customer, new use case],
    [University campuses], [\$1B], [Robotics labs + facilities],
    [Corporate campuses], [\$2B], [Tech companies, HQs],
    [Shopping centers/retail], [\$1.5B], [Property management cos],
    [Airports (airside)], [\$500M], [Specialized, high-value],
    [HOAs/residential], [\$4B], [Volume play, later phase],
  )

  #v(0.2in)

  #align(center)[
    #text(size: 20pt, weight: "bold")[
      At just 1% market share: *\$140M revenue*
    ]
  ]
]

// =============================================================================
// Slide 6: Business Model
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Business Model]

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.5in,
    [
      #text(size: 18pt, weight: "bold")[Hardware + Subscription]

      #table(
        columns: (2fr, 1fr),
        stroke: 0.5pt + muni-light-gray,
        inset: 10pt,

        [BVR rover (bvr1)], [*\$18,000*],
        [Depot base station], [\$6,000],
        [Annual software subscription], [\$3,600/yr],
        [5-year LTV per rover], [*\$36,000*],
      )

      #v(0.3in)

      #text(size: 14pt)[
        *Gross margin:* 65% hardware, 85% software \
        *Recurring revenue* creates predictable growth
      ]
    ],
    [
      #text(size: 18pt, weight: "bold")[Fleet Packages]

      #table(
        columns: (1fr, 1fr, 1fr),
        stroke: 0.5pt + muni-light-gray,
        inset: 8pt,
        fill: (_, row) => if row == 0 { muni-light-gray } else { white },

        [*Package*], [*Rovers*], [*Price*],
        [Pilot], [2], [\$50k],
        [Small], [10], [\$220k],
        [Medium], [25], [\$500k],
        [Large], [50], [\$950k],
        [Enterprise], [100+], [\$2M+],
      )

      #v(0.2in)

      #text(size: 14pt, fill: muni-gray)[
        One Chicago-sized deal = entire year's revenue
      ]
    ]
  )
]

// =============================================================================
// Slide 7: Traction
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Traction]

  #grid(
    columns: (1fr, 1fr, 1fr),
    gutter: 0.4in,
    [
      #align(center)[
        #text(size: 48pt, weight: "bold", fill: muni-orange)[1]
        #v(0.1in)
        #text(size: 16pt, weight: "bold")[Working Prototype]
        #v(0.1in)
        #text(size: 12pt, fill: gray)[
          bvr0 complete \
          December 2025
        ]
      ]
    ],
    [
      #align(center)[
        #text(size: 48pt, weight: "bold", fill: muni-orange)[3-5]
        #v(0.1in)
        #text(size: 16pt, weight: "bold")[Pilot Partners Sought]
        #v(0.1in)
        #text(size: 12pt, fill: gray)[
          Midwest municipalities \
          Winter 2026
        ]
      ]
    ],
    [
      #align(center)[
        #text(size: 48pt, weight: "bold", fill: muni-orange)[100%]
        #v(0.1in)
        #text(size: 16pt, weight: "bold")[Open Source]
        #v(0.1in)
        #text(size: 12pt, fill: gray)[
          Build community \
          Earn trust
        ]
      ]
    ],
  )

  #v(0.5in)

  #align(center)[
    #box(
      fill: muni-light-gray,
      inset: 16pt,
      radius: 8pt,
    )[
      #text(size: 16pt)[
        *Milestone:* bvr1 production units shipping to pilot partners Summer 2026
      ]
    ]
  ]
]

// =============================================================================
// Slide 8: Competition
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Competitive Landscape]

  #table(
    columns: (1.5fr, 1fr, 1fr, 1fr, 1fr),
    stroke: 0.5pt + muni-light-gray,
    inset: 10pt,
    fill: (col, row) => {
      if row == 0 { muni-light-gray }
      else if col == 0 { white }
      else { white }
    },

    [], [*Muni*], [*Manual Labor*], [*Toro RT-1000*], [*Yarbo*],
    [Sidewalk-sized], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]], [#text(fill: muni-danger)[✗]], [#text(fill: muni-success)[✓]],
    [All-weather], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]], [#text(fill: muni-danger)[✗]],
    [Municipal-grade], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]], [#text(fill: muni-danger)[✗]],
    [Autonomous], [#text(fill: muni-success)[✓]], [#text(fill: muni-danger)[✗]], [#text(fill: muni-success)[✓]], [#text(fill: muni-success)[✓]],
    [Affordable at scale], [#text(fill: muni-success)[✓]], [#text(fill: muni-danger)[✗]], [#text(fill: muni-danger)[✗]], [#text(fill: muni-danger)[✗]],
    [Available now], [2026], [Yes], [Yes], [Yes],
  )

  #v(0.3in)

  #text(size: 14pt)[
    *Key insight:* Toro's RT-1000 is ATV-sized (56" wide) and costs \$50k+. Yarbo targets residential driveways, not municipal infrastructure. No one serves sidewalk-scale municipal maintenance.
  ]
]

// =============================================================================
// Slide 9: Team
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Team]

  #grid(
    columns: (auto, 1fr),
    gutter: 0.5in,
    [
      #image("images/cam.jpg", width: 1.5in)
    ],
    [
      #text(size: 24pt, weight: "bold")[Cam Pedersen]
      #text(size: 16pt, fill: gray)[, Founder]

      #text(size: 14pt)[
        - Autonomous vehicle scheduling, Uber
        - CTO & Co-founder, DitchCarbon (Carbon data aggregation)
        - Director of Engineering, Vanilla
        - Based in Cleveland, Ohio
      ]
    ]
  )

  #v(0.5in)

  #text(size: 18pt, weight: "bold")[Hiring with this round:]

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.3in,
    [
      #box(fill: muni-light-gray, inset: 12pt, radius: 4pt, width: 100%)[
        *Robotics Engineer* \
        #text(size: 12pt, fill: gray)[Autonomy, perception, controls]
      ]
    ],
    [
      #box(fill: muni-light-gray, inset: 12pt, radius: 4pt, width: 100%)[
        *Business Development* \
        #text(size: 12pt, fill: gray)[Municipal sales, partnerships]
      ]
    ]
  )
]

// =============================================================================
// Slide 10: Roadmap
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Roadmap]

  #table(
    columns: (1fr, 2fr, 2fr),
    stroke: 0.5pt + muni-light-gray,
    inset: 12pt,
    fill: (_, row) => if row == 0 { muni-light-gray } else { white },

    [*When*], [*Milestone*], [*Capability*],
    [Dec 2025], [bvr0 prototype complete], [Drivetrain, teleop, GPS, depot],
    [Jan 2026], [Artifact residency], [bvr1 R&D, autonomy, production unit],
    [Q1 2026], [Pre-seed close], [Hire team, scale production],
    [Q2 2026], [bvr1 production], [First pilot units built],
    [Q3 2026], [Pilot deployments], [3-5 municipal partners],
    [Q3 2026], [Supervised autonomy], [1 operator : 10 rovers],
    [Q4 2026], [Seed round], [\$3M at \$12M post],
    [2027], [Scale production], [100+ rovers deployed],
    [2028], [Series A], [National expansion],
  )
]

// =============================================================================
// Slide 11: Financials
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[Financial Projections]

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.5in,
    [
      #text(size: 18pt, weight: "bold")[Revenue Forecast]

      #table(
        columns: (1fr, 1fr, 2fr),
        stroke: 0.5pt + muni-light-gray,
        inset: 10pt,
        fill: (_, row) => if row == 0 { muni-light-gray } else { white },

        [*Year*], [*Revenue*], [*Driver*],
        [2026], [\$500k], [Early pilots],
        [2027], [\$4M], [University + enterprise],
        [2028], [\$15M], [Subscription + national],
        [2029], [\$50M], [Federal + platform],
        [2030], [\$160M], [International + RaaS],
      )
    ],
    [
      #text(size: 18pt, weight: "bold")[Path to Profitability]

      #table(
        columns: (1fr, 1fr, 1fr),
        stroke: 0.5pt + muni-light-gray,
        inset: 10pt,
        fill: (_, row) => if row == 0 { muni-light-gray } else { white },

        [*Year*], [*EBITDA*], [*Margin*],
        [2026], [(\$325k)], [-65%],
        [2027], [(\$700k)], [-18%],
        [2028], [(\$2M)], [-13%],
        [2029], [\$1.5M], [3%],
        [2030], [\$31M], [19%],
      )

      #v(0.2in)

      #text(size: 12pt, fill: gray)[
        Profitable by 2029 with subscription revenue
      ]
    ]
  )
]

// =============================================================================
// Slide 12: The Ask
// =============================================================================

#page[
  #text(size: 32pt, weight: "bold", fill: muni-orange)[The Ask]

  #v(-0.5in)

  #align(center)[
    #text(size: 48pt, weight: "bold")[\$500-600k Pre-Seed]
    #v(-0.5in)
    #text(size: 20pt, fill: gray)[at \$3M post-money valuation]
  ]

  #grid(
    columns: (1fr, 1fr),
    gutter: 0.5in,
    [
    #v(-0.5in)
      #text(size: 18pt, weight: "bold")[Use of Funds]

      #table(
        columns: (2fr, 1fr),
        stroke: 0.5pt + muni-light-gray,
        inset: 10pt,

        [Team (2 hires, 12 mo)], [\$220k],
        [Hardware (10 pilot units)], [\$100k],
        [Facilities], [\$40k],
        [R&D / prototyping], [\$50k],
        [Sales / marketing], [\$40k],
        [Legal / admin], [\$25k],
        [Buffer], [\$75k],
        [*Total*], [*\$550k*],
      )
    ],
    [
      #text(size: 18pt, weight: "bold")[Milestones to Seed]

      #box(fill: muni-light-gray, inset: 16pt, radius: 8pt, width: 100%)[
        #stack(
          dir: ttb,
          spacing: 12pt,
          [✓ 3 paying pilots (\$100k+ revenue)],
          [✓ 10 rovers deployed in field],
          [✓ Supervised autonomy working (1:10)],
          [✓ 2-3 LOIs from larger cities],
        )
      ]

      #text(size: 14pt)[
        *Seed target:* \$3M at \$12M post \
        *Timeline:* Q4 2026
      ]
    ]
  )
]

// =============================================================================
// Slide 13: Vision
// =============================================================================

#page[
  #align(center + horizon)[
    #text(size: 24pt, fill: gray)[Our Vision]

    #v(0.3in)

    #text(size: 36pt, weight: "bold")[
      The AWS of outdoor autonomy
    ]

    #v(0.3in)

    #text(size: 18pt)[
      Starting with sidewalk snow removal. \
      Expanding to all outdoor surface maintenance. \
      Becoming the platform that powers every robot that works outside.
    ]

    #v(0.5in)

    #box(
      fill: muni-light-gray,
      inset: 20pt,
      radius: 8pt,
    )[
      #text(size: 16pt)[
        *Exit potential:* \$400-600M acquisition by 2029-2030 \
        by John Deere, Caterpillar, or Husqvarna
      ]
    ]

    #v(0.5in)

    #text(size: 20pt, weight: "bold")[
      Cam Pedersen \
      info\@muni.works
    ]
  ]
]

