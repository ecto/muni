// Municipal Robotics Document Template
// Shared styling for whitepapers, manuals, and technical documentation

#import "@preview/cetz:0.4.2"
#import "@preview/zero:0.5.0": num, ztable, set-num

// =============================================================================
// Brand Colors
// See docs/design-language.md for full specification
// =============================================================================

#let muni-orange = rgb("#ff6600")      // Primary accent (safety orange) - matches web
#let muni-gray = rgb("#5C5C5C")        // Secondary (cool gray)
#let muni-light-gray = rgb("#F5F5F5")  // Alternating rows, code backgrounds
#let muni-bg = rgb("#FAFAFA")          // Warm white background
#let muni-danger = rgb("#C41E3A")      // Critical warnings (life safety, damage risk)
#let muni-note = rgb("#2563EB")        // Information callouts
#let muni-success = rgb("#22c55e")     // Success/complete - matches web

// =============================================================================
// Brand Typography
// See docs/design-language.md for full specification
// Compile with: typst compile --font-path fonts/ <file>.typ
// =============================================================================

#let muni-font = "Berkeley Mono" // Primary font (terminal aesthetic)
#let muni-font-mono = "Berkeley Mono" // Monospace (same as primary)
#let muni-font-fallback = ("SF Mono", "Courier New", "Courier")
#let muni-font-size = 9pt
#let muni-leading = 0.9em
#let muni-tracking = 0em
#let muni-justify = false

// Configure zero for large numbers with comma grouping
#set-num(group: (threshold: 4, separator: ","))

// =============================================================================
// Main Document Template
// =============================================================================

#let manual(
  title: "Document Title",
  subtitle: none,
  revision: "1.0",
  date: datetime.today().display("[month repr:long] [year]"),
  doc-type: "Technical Manual",
  cover-image: none,
  body
) = {
  // Document metadata
  set document(
    title: title,
    author: "Municipal Robotics",
  )

  // Page setup: Portrait, single-column, generous margins
  // Right margin extra wide for handwritten notes in print
  set page(
    paper: "us-letter",
    margin: (left: 0.75in, right: 1.25in, top: 0.75in, bottom: 0.75in),
    numbering: "1",
    number-align: right,
    header: context {
      if counter(page).get().first() > 2 [
        #set text(size: 8pt, fill: gray)
        #title #h(1fr) Rev #revision
      ]
    },
    footer: context {
      if counter(page).get().first() > 2 [
        #set text(size: 8pt, fill: muni-gray)
        Municipal Robotics
        #h(1fr)
        #counter(page).display()
      ]
    },
  )

  // Figures: inline, not floating
  set figure(placement: none)

  // Typography (Berkeley Mono for terminal aesthetic)
  set text(font: (muni-font, ..muni-font-fallback), size: 9pt, tracking: muni-tracking)
  set par(justify: muni-justify, leading: 0.8em, spacing: 1em)
  // No heading numbers for cleaner section titles
  set heading(numbering: none)

  // Level 1 headings: Orange left border
  show heading.where(level: 1): it => {
    v(1em)
    block(
      width: 100%,
      inset: (left: 10pt, y: 8pt),
      stroke: (left: 3pt + muni-orange),
    )[
      #text(size: 14pt, weight: "bold")[#it.body]
    ]
    v(0.5em)
  }

  // Level 2 headings
  show heading.where(level: 2): it => {
    v(0.8em)
    text(size: 11pt, weight: "bold")[#it.body]
    v(0.2em)
  }

  // Level 3 headings
  show heading.where(level: 3): it => {
    v(0.5em)
    text(size: 9pt, weight: "bold")[#it.body]
    v(0.1em)
  }

  // Code blocks
  show raw: set text(font: (muni-font-mono, ..muni-font-fallback), size: 8pt)
  show raw.where(block: true): it => {
    block(
      width: 100%,
      fill: muni-light-gray,
      inset: 12pt,
      radius: 4pt,
    )[#it]
  }

  // Cover page
  page(
    margin: 1in,
    header: none,
    footer: none,
  )[
    #align(center)[
      #v(0.3in)

      // Document type (subtle)
      #text(size: 9pt, fill: gray, tracking: 0.1em)[
        #upper(doc-type)
      ]

      #v(0.2in)

      // Main title
      #text(size: 32pt, weight: "bold")[
        #title
      ]

      #if subtitle != none [
        #v(0.2em)
        #text(size: 12pt, fill: muni-gray)[#subtitle]
      ]

      #v(0.4in)

      // Cover image
      #if cover-image != none [
        #image(cover-image)
      ]

      #v(1fr)

      // Revision and date
      #text(size: 9pt, fill: gray)[
        Revision #revision
        #h(2em)
        #date
      ]

      #v(0.2in)

      // Company info
      #text(size: 9pt)[
        *Municipal Robotics* \
        Cleveland, Ohio \
        #link("https://muni.works")[muni.works]
      ]
    ]
  ]

  // Table of contents
  page(
    header: none,
    footer: none,
  )[
    #v(0.3in)
    #text(size: 14pt, weight: "bold")[Contents]
    #v(1em)
    #outline(
      title: none,
      indent: 1.5em,
      depth: 2,
    )
  ]

  // Body content
  body
}

// =============================================================================
// Callout Boxes
// Hierarchy: Danger > Warning > Note > Tip
// Use sparingly - boxes draw attention, reserve for important callouts
// =============================================================================

// Danger box for critical safety notices (life safety, damage risk)
#let danger(body) = {
  block(
    width: 100%,
    fill: rgb("#FEF2F2"),
    stroke: (left: 3pt + muni-danger),
    inset: (left: 10pt, right: 8pt, y: 8pt),
    radius: (right: 3pt),
  )[
    #text(size: 8pt)[
      #text(weight: "bold", fill: muni-danger)[⚠ DANGER ] #body
    ]
  ]
}

// Warning box for safety notices
#let warning(body) = {
  block(
    width: 100%,
    fill: rgb("#FFF7ED"),
    stroke: (left: 3pt + muni-orange),
    inset: (left: 10pt, right: 8pt, y: 8pt),
    radius: (right: 3pt),
  )[
    #text(size: 8pt)[
      #text(weight: "bold", fill: muni-orange)[⚠ WARNING ] #body
    ]
  ]
}

// Note box for tips and information
#let note(body) = {
  block(
    width: 100%,
    fill: rgb("#EFF6FF"),
    stroke: (left: 3pt + muni-note),
    inset: (left: 10pt, right: 8pt, y: 8pt),
    radius: (right: 3pt),
  )[
    #text(size: 8pt)[
      #text(weight: "bold", fill: muni-note)[ℹ NOTE ] #body
    ]
  ]
}

// Success/tip box
#let tip(body) = {
  block(
    width: 100%,
    fill: rgb("#F0FDF4"),
    stroke: (left: 3pt + muni-success),
    inset: (left: 10pt, right: 8pt, y: 8pt),
    radius: (right: 3pt),
  )[
    #text(size: 8pt)[
      #text(weight: "bold", fill: muni-success)[✓ TIP ] #body
    ]
  ]
}

// =============================================================================
// Tables
// =============================================================================

// Specification table (two columns, clean styling)
#let spec-table(..args) = {
  set text(size: 8pt)
  table(
    columns: (1fr, 1fr),
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 5pt,
    fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
    ..args,
  )
}

// BOM table (4 columns: Part, Qty, Unit, Total)
#let bom-table(..args) = {
  set text(size: 8pt)
  table(
    columns: (2fr, auto, auto, auto),
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 5pt,
    fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
    ..args.pos().enumerate().map(((i, cell)) => {
      if i < 4 { text(weight: "bold")[#cell] } else { cell }
    }),
  )
}

// =============================================================================
// Checklists
// =============================================================================

// Styled checkbox item
#let checkbox(checked: false, body) = {
  let box-style = if checked {
    box(
      width: 10pt,
      height: 10pt,
      fill: muni-orange,
      radius: 2pt,
      align(center + horizon)[
        #text(fill: white, size: 7pt, weight: "bold")[✓]
      ]
    )
  } else {
    box(
      width: 10pt,
      height: 10pt,
      stroke: 1pt + muni-gray,
      radius: 2pt,
    )
  }

  grid(
    columns: (auto, 1fr),
    column-gutter: 6pt,
    row-gutter: 4pt,
    align: horizon,
    box-style,
    text(size: 8pt)[#body],
  )
}

// Checklist block
#let checklist(..items) = {
  block(
    width: 100%,
    fill: muni-light-gray,
    inset: 8pt,
    radius: 3pt,
  )[
    #stack(
      dir: ttb,
      spacing: 5pt,
      ..items.pos().map(item => checkbox(checked: false, item))
    )
  ]
}

// =============================================================================
// Step Markers
// =============================================================================

// Numbered step marker (orange circle with white number)
#let step(number) = {
  box(
    width: 18pt,
    height: 18pt,
    fill: muni-orange,
    radius: 9pt,
    align(center + horizon)[
      #text(fill: white, size: 9pt, weight: "bold")[#number]
    ]
  )
}

// =============================================================================
// Figures
// =============================================================================

// Styled figure with muted caption
#show figure.caption: it => {
  text(size: 8pt, fill: rgb("#999999"))[
    #text(weight: "bold")[#it.supplement #it.counter.display():]
    #it.body
  ]
}

// =============================================================================
// Procedure Headers with Time & Difficulty
// =============================================================================

// Difficulty indicator (1-3 dots)
#let difficulty-dots(level) = {
  stack(
    dir: ltr,
    spacing: 2pt,
    ..range(3).map(i => circle(
      radius: 3pt,
      fill: if i < level { muni-orange } else { rgb("#E5E5E5") },
      stroke: none,
    ))
  )
}

// Procedure header with time estimate and difficulty
#let procedure(title, time: none, difficulty: none) = {
  v(0.3em)
  block(
    width: 100%,
    inset: (y: 4pt),
    stroke: (bottom: 0.5pt + rgb("#E5E5E5")),
  )[
    #grid(
      columns: (1fr, auto, auto),
      column-gutter: 8pt,
      align: (left, center, center),
      text(size: 9pt, fill: muni-gray)[#title],
      if time != none { text(size: 7pt, fill: muni-gray)[~#time] },
      if difficulty != none { difficulty-dots(difficulty) },
    )
  ]
  v(0.2em)
}

// =============================================================================
// Lessons Learned & Experience Callouts
// =============================================================================

// "We learned this the hard way" callout
#let lesson(body) = {
  block(
    width: 100%,
    fill: rgb("#FEFCE8"),
    stroke: (left: 2pt + rgb("#CA8A04")),
    inset: (left: 10pt, right: 8pt, y: 6pt),
    radius: (right: 3pt),
  )[
    #text(size: 7pt, style: "italic", fill: rgb("#854D0E"))[
      "We learned this the hard way:" #body
    ]
  ]
}

// Common mistake / pitfall warning
#let pitfall(body) = {
  block(
    width: 100%,
    fill: rgb("#FEF3C7"),
    stroke: (left: 2pt + rgb("#D97706")),
    inset: (left: 10pt, right: 8pt, y: 6pt),
    radius: (right: 3pt),
  )[
    #text(size: 7pt)[
      #text(weight: "bold", fill: rgb("#92400E"))[⚡ COMMON MISTAKE ] #body
    ]
  ]
}

// =============================================================================
// Video Links
// =============================================================================

// Video link (inline, compact)
#let video-link(url, caption) = {
  box(
    inset: 6pt,
    radius: 3pt,
    fill: muni-light-gray,
    stroke: 0.5pt + rgb("#E5E5E5"),
  )[
    #text(size: 7pt, fill: muni-orange)[▶]
    #h(4pt)
    #link(url)[#text(size: 7pt)[#caption]]
  ]
}

// =============================================================================
// Before/After Comparison
// =============================================================================

#let before-after(before-content, after-content, caption: none) = {
  figure(
    grid(
      columns: (1fr, 1fr),
      column-gutter: 1em,
      box(
        width: 100%,
        stroke: 1pt + rgb("#E5E5E5"),
        inset: 8pt,
        radius: 4pt,
      )[
        #align(center)[
          #text(size: 6pt, fill: muni-gray, weight: "bold")[BEFORE]
        ]
        #v(0.3em)
        #before-content
      ],
      box(
        width: 100%,
        stroke: 1pt + muni-success,
        inset: 8pt,
        radius: 4pt,
      )[
        #align(center)[
          #text(size: 6pt, fill: muni-success, weight: "bold")[AFTER]
        ]
        #v(0.3em)
        #after-content
      ],
    ),
    caption: caption,
  )
}

// =============================================================================
// Hardware Reference (1:1 Scale)
// =============================================================================

// 1:1 scale indicator box
#let scale-indicator() = {
  box(
    inset: 6pt,
    radius: 4pt,
    stroke: 1pt + muni-orange,
    fill: rgb("#FFF7ED"),
  )[
    #text(size: 7pt, fill: muni-orange, weight: "bold")[1:1 SCALE]
    #text(size: 6pt, fill: muni-gray)[ — Print at 100%]
  ]
}

// =============================================================================
// Version History Table
// =============================================================================

#let version-history(..revisions) = {
  table(
    columns: (auto, auto, 1fr),
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 8pt,
    fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
    [*Rev*], [*Date*], [*Changes*],
    ..revisions.pos().flatten(),
  )
}

// =============================================================================
// Glossary Entry
// =============================================================================

#let glossary-entry(term, definition) = {
  grid(
    columns: (auto, 1fr),
    column-gutter: 1em,
    text(weight: "bold")[#term],
    text(size: 9pt)[#definition],
  )
  v(0.3em)
}
