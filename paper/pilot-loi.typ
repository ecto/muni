// Municipal Robotics - Pilot Program Letter of Intent
// Non-binding LOI template for pilot partnerships

#import "lib/template.typ": *

#set document(title: "Pilot Program Letter of Intent", author: "Municipal Robotics")

#set page(
  paper: "us-letter",
  margin: (x: 1in, y: 0.75in),
  numbering: none,
)

#set text(font: (muni-font, ..muni-font-fallback), size: 10pt)
#set par(justify: false, leading: 0.8em)

// Variables - fill these in for each LOI
#let partner-name = "[ORGANIZATION NAME]"
#let partner-contact = "[CONTACT NAME]"
#let partner-title = "[TITLE]"
#let partner-address = "[ADDRESS]"
#let pilot-rovers = "[X]"
#let pilot-duration = "90 days"
#let pilot-start = "Winter 2026-27"
#let pilot-locations = "[designated pilot areas]"
#let hardware-price = "$12,000"
#let today-date = datetime.today().display("[month repr:long] [year]")

// Header
#align(right)[
  #image("muni-logo-dark.svg", width: 1.5in)
]

#v(0.3in)

#text(size: 9pt, fill: muni-gray)[#today-date]

#v(0.2in)

#partner-contact \
#partner-title \
#partner-name \
#partner-address

#v(0.3in)

#align(center)[
  #text(size: 14pt, weight: "bold")[Letter of Intent] \
  #text(size: 11pt, fill: muni-gray)[Autonomous Snow Removal Pilot Program]
]

#v(0.3in)

Dear #partner-contact,

This Letter of Intent ("LOI") confirms the mutual interest between *Municipal Robotics* ("Muni") and *#partner-name* ("Partner") in establishing a pilot program for autonomous sidewalk snow removal.

#v(0.15in)

== Pilot Program Overview

#table(
  columns: (1.2in, 1fr),
  stroke: none,
  inset: (x: 0pt, y: 4pt),
  [*Equipment:*], [#pilot-rovers BVR autonomous snow removal rover(s)],
  [*Duration:*], [#pilot-duration from deployment],
  [*Target Start:*], [#pilot-start],
  [*Locations:*], [#partner-name #pilot-locations],
  [*Pilot Pricing:*], [#hardware-price per unit (discounted from \$18,000 MSRP)],
)

#v(0.15in)

== Muni Responsibilities

- Provide #pilot-rovers production BVR rover(s) configured for snow removal
- On-site deployment, training, and technical support
- Remote monitoring and software updates during pilot period
- Collect performance data and provide monthly reports
- Maintain liability insurance covering rover operations

== Partner Responsibilities

- Designate pilot area(s) suitable for autonomous operation
- Provide site access for deployment and maintenance
- Assign a point of contact for coordination
- Share feedback on operational performance
- Participate in joint case study upon successful completion

#v(0.15in)

== Non-Binding Nature

This LOI represents a statement of mutual intent and is *not a binding contract*. Either party may withdraw at any time prior to execution of a definitive Pilot Agreement. No financial commitment is required until a formal agreement is signed.

#v(0.15in)

== Next Steps

Upon indication of interest, Muni will:
1. Schedule an on-site demonstration
2. Conduct site assessment for pilot locations
3. Prepare a detailed Pilot Agreement for review

#v(0.4in)

#grid(
  columns: (1fr, 1fr),
  gutter: 0.5in,
  [
    *Municipal Robotics* \
    #v(0.5in)
    #line(length: 2in, stroke: 0.5pt) \
    Cam Pedersen, Founder \
    #text(size: 9pt, fill: muni-gray)[Date: #h(1in)]
  ],
  [
    *#partner-name* \
    #v(0.5in)
    #line(length: 2in, stroke: 0.5pt) \
    #partner-contact, #partner-title \
    #text(size: 9pt, fill: muni-gray)[Date: #h(1in)]
  ],
)

#v(0.4in)

#align(center)[
  #box(
    width: 100%,
    stroke: 0.5pt + muni-light-gray,
    inset: 12pt,
    radius: 4pt,
  )[
    #text(size: 9pt, fill: muni-gray)[
      *Municipal Robotics* · Cleveland, Ohio \
      cam\@muni.works · muni.works · (218) 851-9923
    ]
  ]
]
