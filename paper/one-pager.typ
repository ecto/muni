// Municipal Robotics One-Pager
// Pre-Seed Investment Summary

#import "lib/template.typ": muni-orange, muni-gray, muni-light-gray, muni-font, muni-font-size, muni-leading, muni-tracking, muni-justify

#set document(title: "Municipal Robotics - Investment Summary", author: "Municipal Robotics")

#set page(
  paper: "us-letter",
  margin: 0.4in,
  numbering: none,
)

// Smaller size to fit on one page
#set text(font: muni-font, size: 9pt, tracking: muni-tracking)
#set par(justify: muni-justify, leading: muni-leading)

// Header
#align(center)[
  #stack(dir: ttb, spacing: 1em)[
    #text(size: 24pt, weight: "bold")[Municipal Robotics]
  ][
    #text(size: 11pt, fill: gray)[Pre-Seed Investment Summary]
  ]
]

#v(0.15in)

// Two-column layout
#grid(
  columns: (1fr, 1fr),
  gutter: 0.3in,

  // LEFT COLUMN
  [
    // Company
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Company]

    Autonomous sidewalk maintenance vehicles for municipalities. Starting with snow removal, expanding to all outdoor surface maintenance.

    #v(0.12in)

    // Problem
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Problem]

    - 1M+ slip-and-fall injuries annually from icy sidewalks
    - \$35B in municipal liability costs
    - Manual clearing: expensive, unavailable, unsafe
    - No sidewalk-scale autonomous solution exists

    #v(0.12in)

    // Solution
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Solution]

    *BVR (Base Vectoring Rover):* Electric, sidewalk-sized rover with modular tools (auger, plow, spreader, mower). LiDAR safety system stops on any obstacle. One operator monitors 10+ rovers remotely.

    #v(0.12in)

    // Market
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Market]

    #table(
      columns: (2fr, 1fr),
      stroke: none,
      inset: 3pt,
      [Municipal sidewalks], [\$2B],
      [Parks / paths], [\$3B],
      [University campuses], [\$1B],
      [Corporate / retail], [\$3.5B],
      [HOAs / residential], [\$4B],
      [*Total addressable*], [*\$14B+*],
    )

    #v(0.12in)

    // Business Model
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Business Model]

    - Hardware: \$18k per rover (65% margin)
    - Software subscription: \$3,600/year (85% margin)
    - 5-year LTV: \$36k per rover
    - Fleet packages: \$50k (pilot) to \$2M+ (enterprise)

    #v(0.12in)

    // Traction
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Traction]

    - bvr0 prototype complete (Dec 2025)
    - bvr1 production units shipping Summer 2026
    - Seeking 3-5 Midwest municipal pilots
    - 100% open source (builds trust, community)
  ],

  // RIGHT COLUMN
  [
    // Team
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Team]

    *Cam Pedersen, Founder* \
    Autonomous vehicle scheduling at Uber. CTO & Co-founder at DitchCarbon. Based in Cleveland, Ohio.

    *Hiring:* Robotics engineer, Business development

    #v(0.12in)

    // Financials
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Financial Projections]

    #table(
      columns: (1fr, 1fr, 1fr),
      stroke: 0.5pt + muni-light-gray,
      inset: 4pt,
      fill: (_, row) => if row == 0 { muni-light-gray } else { white },
      [*Year*], [*Revenue*], [*EBITDA*],
      [2026], [\$500k], [(\$325k)],
      [2027], [\$4M], [(\$700k)],
      [2028], [\$15M], [(\$2M)],
      [2029], [\$50M], [\$1.5M],
      [2030], [\$160M], [\$31M],
    )

    #v(0.12in)

    // The Ask
    #text(size: 11pt, weight: "bold", fill: muni-orange)[The Ask]

    #box(
      width: 100%,
      fill: muni-light-gray,
      inset: 10pt,
      radius: 4pt,
    )[
      #align(center)[
        #text(size: 16pt, weight: "bold")[\$500-600k Pre-Seed]
        #v(0in)
        #text(size: 10pt)[\$3M post-money valuation]
      ]
    ]

    #v(0.1in)

    *Use of funds:*
    - Team: \$220k (2 hires, 12 months)
    - Hardware: \$100k (10 pilot units)
    - Operations: \$155k (facilities, R&D, sales)
    - Buffer: \$75k

    #v(0.12in)

    // Milestones
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Milestones to Seed (Q4 2026)]

    - 3 paying pilots (\$100k+ revenue)
    - 10 rovers deployed in field
    - Supervised autonomy (1 operator : 10 rovers)
    - Seed: \$3M at \$12M post

    #v(0.12in)

    // Exit
    #text(size: 11pt, weight: "bold", fill: muni-orange)[Exit Potential]

    \$400-600M acquisition by 2029-2030 by industrial OEM (John Deere, Caterpillar, Husqvarna) consolidating outdoor autonomy.

    #v(0.15in)

    // Contact
    #box(
      width: 100%,
      stroke: 1pt + muni-orange,
      inset: 10pt,
      radius: 4pt,
    )[
      #align(center)[
        *Cam Pedersen* \
        info\@muni.works · muni.works \
        Cleveland, Ohio
      ]
    ]
  ]
)

#v(0.1in)

#align(center)[
  #text(size: 8pt, fill: gray)[
    This document is confidential and for informational purposes only. Not an offer to sell securities.
  ]
]

