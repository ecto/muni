// Municipal Robotics Graphics Standards Manual
// Defining the visual identity for autonomous public works vehicles

#import "lib/template.typ": *

#set document(
  title: "Graphics Standards Manual",
  author: "Municipal Robotics",
)

#set page(
  paper: "us-letter",
  margin: 0.75in,
  numbering: "1",
  number-align: center,
  header: context {
    if counter(page).get().first() > 1 [
      #set text(font: "Berkeley Mono", size: 7pt, fill: muni-gray)
      MUNICIPAL ROBOTICS #h(1fr) GRAPHICS STANDARDS MANUAL
    ]
  },
  footer: context {
    if counter(page).get().first() > 1 [
      #set text(font: "Berkeley Mono", size: 7pt, fill: muni-gray)
      #h(1fr) #counter(page).display() #h(1fr)
    ]
  },
)

#set text(font: "Helvetica Neue", size: 9pt)
#set par(justify: false, leading: 0.9em, spacing: 2em)

// Berkeley Mono for non-prose text (tables, code, metadata)
#show table: it => {
  set text(font: "Berkeley Mono", size: 7pt)
  it
  v(1.5em)
}

#show raw: it => {
  set text(font: "Berkeley Mono")
  it
}

#show figure: it => {
  it
  v(1.5em)
}

#show raw.where(block: true): it => {
  block(
    width: 100%,
    fill: muni-light-gray,
    inset: 12pt,
    radius: 4pt,
  )[#it]
  v(1.5em)
}

#show grid: it => {
  it
  v(1.5em)
}

// Heading spacing: large space before (section breaks), minimal space after (tight grouping)
#show heading.where(level: 1): it => {
  v(3em)
  block(
    width: 100%,
    inset: (left: 10pt, y: 8pt),
    stroke: (left: 3pt + muni-orange),
  )[
    #text(font: "Helvetica Neue", size: 14pt, weight: "bold")[#it.body]
  ]
  v(0.3em)
}

#show heading.where(level: 2): it => {
  v(2.5em)
  text(font: "Helvetica Neue", size: 11pt, weight: "medium")[#it.body]
  v(0.15em)
}

#show heading.where(level: 3): it => {
  v(1.5em)
  text(font: "Helvetica Neue", size: 9pt, weight: "bold")[#it.body]
  v(-0.8em)
}

// =============================================================================
// Cover Page
// =============================================================================

#page(
  margin: 1in,
  header: none,
  footer: none,
)[
  #v(1.5in)

  #align(center)[
    // Logo
    #image("muni-logo-light.svg", width: 3in)

    #v(0.3in)

    #text(font: "Berkeley Mono", size: 9pt, fill: muni-gray, tracking: 0.15em)[
      MUNICIPAL ROBOTICS
    ]

    #v(1in)

    #text(size: 24pt, weight: "bold")[
      Graphics Standards Manual
    ]

    #v(0.3in)

    #text(font: "Berkeley Mono", size: 8pt, fill: muni-gray)[
      Visual identity guidelines for \
      autonomous public works vehicles
    ]

    #v(1fr)

    #text(font: "Berkeley Mono", size: 8pt, fill: muni-gray)[
      Version 1.0 \
      #datetime.today().display("[month repr:long] [year]")
    ]
  ]
]

// =============================================================================
// Table of Contents
// =============================================================================

#page(header: none, footer: none)[
  #v(0.5in)
  #text(size: 14pt, weight: "bold")[Contents]
  #v(1em)

  // TOC uses Berkeley Mono
  #set text(font: "Berkeley Mono", size: 8pt)

  // Make level 1 entries bold in TOC
  #show outline.entry.where(level: 1): it => {
    strong(it)
  }

  #outline(title: none, indent: 1.5em, depth: 2)
]

// =============================================================================
// Introduction
// =============================================================================

= Introduction

This manual defines the visual identity standards for Municipal Robotics and its autonomous public works vehicles. Consistent application of these standards builds recognition, trust, and professionalism.

== Core Principles

The visual identity reflects a *civic* character---public-minded and service-oriented---rather than corporate. Our robots should feel like public infrastructure, not consumer gadgets or corporate intrusions.

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 1em,
  box(
    width: 100%,
    inset: 12pt,
    stroke: (left: 3pt + muni-orange),
    fill: muni-light-gray,
  )[
    #set text(font: "Berkeley Mono")
    #text(weight: "bold")[People > Task]

    #v(0.3em)
    #text(size: 8pt)[
      Design that yields to pedestrians, never intimidates.
    ]
  ],
  box(
    width: 100%,
    inset: 12pt,
    stroke: (left: 3pt + muni-orange),
    fill: muni-light-gray,
  )[
    #set text(font: "Berkeley Mono")
    #text(weight: "bold")[Predictability Wins]

    #v(0.3em)
    #text(size: 8pt)[
      Fewer signals, clearer meanings, consistent behavior.
    ]
  ],
  box(
    width: 100%,
    inset: 12pt,
    stroke: (left: 3pt + muni-orange),
    fill: muni-light-gray,
  )[
    #set text(font: "Berkeley Mono")
    #text(weight: "bold")[Quiet Competence]

    #v(0.3em)
    #text(size: 8pt)[
      No "cute"---be competent, boring, helpful.
    ]
  ],
)

// =============================================================================
// Logo
// =============================================================================

= Logo

The muni wordmark is a continuous stroke forming the letters "m-u-n-i" with a distinctive dot on the "i". The letterforms flow into each other, representing the connected nature of municipal infrastructure.

== Primary Wordmark

#align(center)[
  #box(
    width: 80%,
    inset: 2em,
    fill: white,
    stroke: 0.5pt + rgb("#e0e0e0"),
  )[
    #align(center)[
      #image("muni-logo-light.svg", width: 40%)
      #v(0.5em)
      #text(size: 7pt, fill: muni-gray)[Primary wordmark (for light backgrounds)]
    ]
  ]
]

#v(1em)

#align(center)[
  #box(
    width: 80%,
    inset: 2em,
    fill: rgb("#1a1a1a"),
  )[
    #align(center)[
      #image("muni-logo-dark.svg", width: 40%)
      #v(0.5em)
      #text(size: 7pt, fill: rgb("#999999"))[Reversed wordmark (for dark backgrounds)]
    ]
  ]
]

== Construction

The wordmark is constructed from a single continuous stroke with rounded terminals. The stroke weight is proportional to the x-height.

#table(
  columns: (1fr, 2fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Element*], [*Specification*],
  [Stroke width], [40 units (relative to 280-unit height)],
  [Terminals], [Round (stroke-linecap: round)],
  [Joins], [Round (stroke-linejoin: round)],
  [Dot radius], [28 units],
  [Viewbox], [880 × 400 units (with 60-unit padding)],
)

== Clear Space

Minimum clear space around the logo equals the height of the dot on the "i" (28 units, or approximately 10% of the total height).

#align(center)[
  #box(
    stroke: (dash: "dashed", paint: muni-orange, thickness: 1pt),
    inset: 1.5em,
  )[
    #box(
      stroke: 0.5pt + muni-gray,
      inset: 1em,
    )[
      #image("muni-logo-light.svg", width: 2in)
    ]
  ]
]

#align(center)[
  #text(size: 7pt, fill: muni-gray)[
    Dashed line = clear space boundary \
    Inner box = logo bounding box
  ]
]

== Minimum Size

#table(
  columns: (1fr, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Medium*], [*Minimum Width*],
  [Print], [1 inch (25mm)],
  [Digital], [80 pixels],
  [Embroidery], [1.5 inches (38mm)],
)

== Prohibited Uses

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  [
    *Do not:*
    - Stretch or distort proportions
    - Rotate the logo
    - Add drop shadows or effects
    - Place on busy backgrounds
    - Outline the wordmark
    - Change the stroke weight
  ],
  [
    *Do not:*
    - Use unauthorized colors
    - Add taglines to the logo
    - Recreate with alternate fonts
    - Crop or partially obscure
    - Animate individual letters
    - Add gradients or patterns
  ],
)

// =============================================================================
// Color
// =============================================================================

= Color

The color palette is minimal and functional. Safety orange serves as the primary brand color, reinforcing the public works context.

== Primary Colors

#let color-swatch(color, name, hex, usage) = {
  box(
    width: 100%,
    stroke: 0.5pt + rgb("#e0e0e0"),
  )[
    #box(width: 100%, height: 50pt, fill: color)
    #box(
      width: 100%,
      inset: 8pt,
      fill: white,
    )[
      #text(size: 9pt, weight: "bold")[#name] \
      #text(font: "Berkeley Mono", size: 7pt, fill: muni-gray)[#hex] \
      #text(size: 7pt, fill: muni-gray)[#usage]
    ]
  ]
}

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 1em,
  color-swatch(muni-orange, "Safety Orange", "#FF6600", "Primary accent, CTAs"),
  color-swatch(black, "Black", "#000000", "Text, dark backgrounds"),
  color-swatch(white, "White", "#FFFFFF", "Backgrounds, reversed text"),
)

#v(1em)

== Secondary Colors

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 1em,
  color-swatch(muni-gray, "Cool Gray", "#5C5C5C", "Secondary text, borders"),
  color-swatch(muni-light-gray, "Light Gray", "#F5F5F5", "Backgrounds, fills"),
  color-swatch(muni-bg, "Warm White", "#FAFAFA", "Page backgrounds"),
)

#v(1em)

== Status Colors

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 1em,
  color-swatch(muni-success, "Success Green", "#22C55E", "Complete, operational"),
  color-swatch(muni-note, "Info Blue", "#2563EB", "Information, notes"),
  color-swatch(muni-danger, "Danger Red", "#C41E3A", "Critical warnings"),
)

== Color Usage

#table(
  columns: (auto, 1fr, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Context*], [*Primary*], [*Notes*],
  [Digital (light mode)], [Black on white], [Orange for accents only],
  [Digital (dark mode)], [White on black], [Orange for accents only],
  [Print documents], [Black text, orange accents], [High contrast for legibility],
  [Physical rovers], [Safety orange, gray panels], [Matte finish, no gloss],
  [Safety markings], [Orange, retroreflective], [Per MUTCD standards],
)

== Forbidden Colors

The following are explicitly prohibited in the brand identity:

- *Chrome or metallic finishes* --- too corporate/aggressive
- *Black gloss* --- intimidating, not civic
- *Gradients* --- breaks the flat, functional aesthetic
- *Neon or fluorescent variations* --- except approved safety materials
- *RGB rainbow effects* --- one meaning per color

// =============================================================================
// Typography
// =============================================================================

= Typography

The typographic system uses Helvetica Neue for prose (readable body text) and Berkeley Mono for technical content (data, code, specifications). Helvetica Neue also serves as the heading typeface. This creates clean, readable documents with technical precision where it matters.

== Technical Typeface: Berkeley Mono

#box(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + rgb("#e0e0e0"),
  fill: muni-light-gray,
)[
  #text(size: 28pt, weight: "regular")[Berkeley Mono]

  #v(0.5em)

  #text(size: 12pt)[
    ABCDEFGHIJKLMNOPQRSTUVWXYZ \
    abcdefghijklmnopqrstuvwxyz \
    0123456789 \
    !@\#\$%^&\*()-+=[]{}|;:',.<>?/
  ]
]

#v(0.5em)
#text(size: 7pt, fill: muni-gray)[
  Licensed from berkeleygraphics.com. Use for code, data labels, tables, specifications, and all technical content.
]

#v(1em)

== Primary Typeface: Helvetica Neue

#box(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + rgb("#e0e0e0"),
  fill: muni-light-gray,
)[
  #text(font: "Helvetica Neue", size: 28pt, weight: "medium")[Helvetica Neue]

  #v(0.5em)

  #text(font: "Helvetica Neue", size: 12pt)[
    ABCDEFGHIJKLMNOPQRSTUVWXYZ \
    abcdefghijklmnopqrstuvwxyz \
    0123456789
  ]
]

#v(0.5em)
#text(size: 7pt, fill: muni-gray)[
  System font (macOS). Use for prose body text and all headings. Creates readable, professional documents.
]

== Type Weights

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  [
    *Berkeley Mono Weights:*

    #v(0.3em)

    #text(weight: "regular")[Regular] \
    #text(size: 7pt, fill: muni-gray)[Body text, code, all content]

    #v(0.3em)

    #text(weight: "bold")[Bold] \
    #text(size: 7pt, fill: muni-gray)[Strong emphasis, data labels]
  ],
  [
    *Helvetica Neue Weights:*

    #v(0.3em)

    #text(font: "Helvetica Neue", weight: "medium")[Medium] \
    #text(size: 7pt, fill: muni-gray)[H1 and H2 headings]

    #v(0.3em)

    #text(font: "Helvetica Neue", weight: "bold")[Bold] \
    #text(size: 7pt, fill: muni-gray)[H3 headings]
  ],
)

== Type Sizes

#table(
  columns: (auto, auto, auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Element*], [*Typeface*], [*Web*], [*Print*], [*Notes*],
  [Body], [Berkeley Mono], [14px], [8pt], [Regular weight, terminal aesthetic],
  [Small], [Berkeley Mono], [12px], [7pt], [Captions, secondary text],
  [Heading 1], [Helvetica Neue], [24px], [14pt], [Medium weight, visual break],
  [Heading 2], [Helvetica Neue], [18px], [11pt], [Medium weight],
  [Heading 3], [Helvetica Neue], [14px], [9pt], [Bold weight],
  [Code blocks], [Berkeley Mono], [13px], [8pt], [Same as body],
  [Data labels], [Berkeley Mono], [11px], [7pt], [ALL CAPS, tracked],
)

== Line Length & Leading

#table(
  columns: (1fr, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Parameter*], [*Value*],
  [Maximum line length], [80 characters],
  [Line height (web)], [1.5],
  [Line height (print)], [0.9em],
  [Paragraph spacing], [2em],
  [Text alignment], [Left (ragged right)],
)

== Fallback Stack

*Body Text:*
```
font-family: "Berkeley Mono", "Menlo", "Courier New", monospace;
```

*Headings Only:*
```
font-family: "Helvetica Neue", "Helvetica", "Arial";
```

== Usage Guidelines

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Use Berkeley Mono for:*], [*Use Helvetica Neue for:*],
  [
    - Code snippets \
    - Terminal output \
    - Data values and labels \
    - Technical specifications \
    - CAN IDs, hex values \
    - Timestamps \
    - Headers and footers \
    - Figure captions \
    - Table data \
    - Procedure titles \
    - Version history \
    - All content except headings
  ],
  [
    - H1 headings (section titles) \
    - H2 headings (subsections) \
    - H3 headings (minor headings) \
    - Body text (ALL paragraphs) \
    - All prose and descriptive text
  ],
)

#v(0.5em)

*Design Philosophy:* Helvetica Neue creates readable prose for general consumption. Berkeley Mono is reserved for technical content where precision and monospace formatting matter: code, specifications, data tables, and system output. This separation keeps documents professional while maintaining technical authenticity.

// =============================================================================
// Physical Design Language
// =============================================================================

= Physical Design Language

Standards for the appearance and behavior of autonomous rovers in public spaces.

== Vehicle Colors

#table(
  columns: (auto, 1fr, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Role*], [*Specification*], [*Rationale*],
  [Primary], [Safety orange, matte finish], [Reads "public works," not brand ad],
  [Secondary], [Cool gray or white panels], [Neutral, serviceable],
  [Forbidden], [Chrome, black gloss, gradients], [Too corporate/aggressive],
)

== Graphics & Markings

*Required elements:*

- Large, legible block lettering: *"SIDEWALK SERVICE ROBOT"*
- Unit ID prominently displayed (e.g., "Beaver-12")
- City/department seal or "Municipal Robotics" badge
- 24/7 hotline phone number
- QR code to incident report form

*Restrictions:*

- Logos limited to one small mark
- No ad wraps, mascots, or lifestyle imagery
- No anthropomorphic features (eyes, faces)

== Conspicuity Elements

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Element*], [*Specification*],
  [Chevrons], [Retroreflective, mounted low and lateral],
  [Marker lights], [Amber, steady (not flashy), at corners],
  [Work light], [White, active only when tool is operating],
)

== Status Display

Top-mounted e-ink panel (sun-readable) cycling:

#box(
  width: 100%,
  inset: 1em,
  fill: muni-light-gray,
  stroke: 0.5pt + rgb("#e0e0e0"),
)[
  #text(size: 8pt)[
    ```
    ╔═══════════════════════════════╗
    ║  SIDEWALK SERVICE - CLEARING  ║
    ║  ───────────────────────────  ║
    ║  Unit: Beaver-12              ║
    ║  ETA this block: 4 min        ║
    ║  ───────────────────────────  ║
    ║  Issues? Call 216-555-0100    ║
    ║  or scan QR →  [QR]           ║
    ╚═══════════════════════════════╝
    ```
  ]
]

== Light Semantics

#table(
  columns: (auto, auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Color*], [*State*], [*Meaning*],
  [Amber steady], [Normal], [Operating],
  [Amber slow pulse], [Caution], [Yielding/waiting],
  [White workbar], [Tool active], [Work in progress],
  [Red solid], [Stopped], [Fault or emergency],
)

*Rule: One meaning per color. No RGB rainbows.*

== Human Cues

The rover should communicate state through simple, non-anthropomorphic cues:

- Front "face" panel with soft geometry (not aggressive)
- Simple status glyph on display:

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Glyph*], [*Meaning*],
  [☺], [Idle, ready],
  [•], [Working],
  [!], [Attention needed],
)

*Avoid:* Skeuomorphic eyes, faces, or anthropomorphic features. The rover is a tool, not a character.

== Materials & Finish

- Textured polymer and powder-coat metal
- Rounded edges, soft fillets throughout
- Visible fasteners acceptable (serviceable vibe)
- No aggressive "sports" angles
- Low profile: hip-height or below for main body

// =============================================================================
// Digital Design Patterns
// =============================================================================

= Digital Design Patterns

The web and application presence follows an "engineering terminal" aesthetic: precise, functional, readable.

== Layout Philosophy

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  [
    *Boxes* (bordered containers)
    
    #box(
      width: 100%,
      stroke: 1pt + black,
    )[
      #box(width: 100%, fill: black, inset: 6pt)[
        #text(fill: white, size: 8pt, weight: "bold")[TITLE]
      ]
      #box(width: 100%, inset: 8pt)[
        #text(size: 8pt)[Reserved for important callouts: CTAs, safety info, key downloads.]
      ]
    ]
    
    #v(0.5em)
    #text(size: 7pt, fill: muni-gray)[Use sparingly---draws attention]
  ],
  [
    *Sections* (simple titles)

    #box(width: 100%)[
      #text(weight: "bold", size: 9pt)[Section Title]
      #v(0.3em)
      #text(size: 8pt)[Standard content containers. Title followed directly by content.]
    ]

    #v(0.5em)
    #text(size: 7pt, fill: muni-gray)[Primary layout unit]
  ],
)

== Interactive States

#table(
  columns: (auto, 1fr, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*State*], [*Treatment*], [*Example*],
  [Default], [Bold text, inherit color], [#text(weight: "bold")[Link text]],
  [Hover], [Inverted (fg ↔ bg)], [#box(fill: black, inset: 3pt)[#text(fill: white, weight: "bold")[Link text]]],
  [CTA], [Orange background, black text], [#box(fill: muni-orange, inset: 3pt)[#text(fill: black, weight: "bold")[Action]]],
)

== Status Indicators

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 1em,
  [
    #align(center)[
      #text(size: 24pt, fill: muni-success)[■]
      
      #text(size: 8pt)[Complete]
    ]
  ],
  [
    #align(center)[
      #text(size: 24pt, fill: muni-orange)[◐]
      
      #text(size: 8pt)[In Progress]
    ]
  ],
  [
    #align(center)[
      #text(size: 24pt, fill: muni-gray)[○]
      
      #text(size: 8pt)[Future]
    ]
  ],
)

== ASCII & Box Drawing

Use box-drawing characters for hierarchies and structure:

#box(
  width: 100%,
  inset: 1em,
  fill: muni-light-gray,
)[
  #text(size: 8pt)[
    ```
    depot/
    ├── console/     Web application
    ├── discovery/   Rover registration
    ├── dispatch/    Mission planning
    └── grafana/     Dashboards
    ```
  ]
]

// =============================================================================
// Print Design Patterns
// =============================================================================

= Print Design Patterns

Guidelines for PDFs and printed materials generated with Typst.

== Page Structure

Each page should answer exactly one question. If a reader flips to a page, they should immediately understand what that page is for.

#box(
  width: 100%,
  inset: 1em,
  stroke: 0.5pt + rgb("#e0e0e0"),
)[
  #grid(
    columns: (auto, 1fr),
    gutter: 1em,
    [
      #box(width: 3pt, height: 100%, fill: muni-orange)
    ],
    [
      #text(weight: "bold")[Title] --- What is this page about?
      
      #v(0.3em)
      
      #box(width: 100%, height: 60pt, fill: muni-light-gray, stroke: 0.5pt + muni-gray)[
        #align(center + horizon)[Primary diagram or content]
      ]
      
      #v(0.3em)
      
      #text(size: 8pt, fill: muni-gray)[Supporting details, specs, notes...]
    ],
  )
]

== Content Priority

Order pages by frequency of access:

+ *Emergency procedures* --- first pages (crisis access)
+ *Daily checklists* --- next (used every session)  
+ *Controls reference* --- next (consulted during operation)
+ *Build instructions* --- middle (used once)
+ *Troubleshooting* --- end (consulted when needed)

== Callout Hierarchy

#v(0.5em)

#danger[
  Life safety or equipment damage risk. Ignoring may cause injury or destruction.
]

#warning[
  Caution required. May cause problems if ignored.
]

#note[
  Helpful information or tips. Good to know.
]

#tip[
  Best practices and recommendations.
]

== Tables

Use consistent table styling with header rows:

#spec-table(
  [*Parameter*], [*Value*],
  [Stroke], [0.5pt, #rgb("#e0e0e0")],
  [Header fill], [#muni-light-gray],
  [Inset], [5--8pt],
  [Font size], [8pt],
)

// =============================================================================
// Application Examples
// =============================================================================

= Application Examples

== Business Card

#align(center)[
  #box(
    width: 3.5in,
    height: 2in,
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 0.25in,
  )[
    #image("muni-logo-light.svg", width: 1in)

    #v(1fr)

    #text(size: 9pt)[
      *Cam Pedersen* \
      Founder
    ]

    #v(0.5em)

    #text(size: 8pt, fill: muni-gray)[
      cam\@muni.works \
      muni.works \
      Cleveland, Ohio
    ]
  ]
]

#align(center)[
  #text(size: 7pt, fill: muni-gray)[3.5" × 2" standard card size]
]

== Document Header

#box(
  width: 100%,
  stroke: (bottom: 0.5pt + rgb("#e0e0e0")),
  inset: (bottom: 8pt),
)[
  #grid(
    columns: (1fr, auto),
    [
      #text(size: 18pt, weight: "bold")[Document Title]

      #text(size: 8pt, fill: muni-gray)[Subtitle or description]
    ],
    [
      #image("muni-logo-light.svg", width: 0.6in)

      #text(size: 7pt, fill: muni-gray)[Rev 1.0]
    ],
  )
]

== Email Signature

#box(
  width: 100%,
  inset: 1em,
  fill: muni-light-gray,
)[
  #text(size: 8pt)[
    ```
    --
    Cam Pedersen
    Founder, Municipal Robotics
    cam@muni.works | muni.works
    Cleveland, Ohio
    ```
  ]
]

// =============================================================================
// Resources
// =============================================================================

= Resources

== File Locations

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt + rgb("#e0e0e0"),
  inset: 8pt,
  fill: (_, row) => if row == 0 { muni-light-gray } else { white },
  [*Asset*], [*Path*],
  [Logo (dark bg)], [`web/public/images/muni-logo-dark.svg`],
  [Logo (light bg)], [`web/public/images/muni-logo-light.svg`],
  [Berkeley Mono fonts], [`paper/fonts/`],
  [Typst template], [`paper/lib/template.typ`],
  [Design language source], [`docs/design-language.md`],
)

== Contact

For questions about brand usage or to request assets:

#align(center)[
  #box(
    inset: 1em,
    stroke: 1pt + muni-orange,
  )[
    *info\@muni.works* \
    muni.works
  ]
]
