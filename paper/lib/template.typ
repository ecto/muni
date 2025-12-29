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

  // Page setup with improved header/footer (landscape for manual, 2-column)
  set page(
    paper: "us-letter",
    flipped: true,
    margin: (x: 0.5in, top: 0.6in, bottom: 0.6in),
    columns: 2,
    numbering: "1",
    number-align: center,
    header: context {
      if counter(page).get().first() > 2 [
        #set text(size: 9pt, fill: gray)
        #title
        #h(1fr)
        Municipal Robotics
      ]
    },
    footer: context {
      if counter(page).get().first() > 2 [
        #set text(size: 8pt, fill: muni-gray)
        Rev #revision
        #h(1fr)
        #counter(page).display()
        #h(1fr)
        #date
      ]
    },
  )

  // Figures span both columns
  set figure(scope: "parent", placement: auto)

  // Typography (Berkeley Mono for terminal aesthetic)
  set text(font: (muni-font, ..muni-font-fallback), size: muni-font-size, tracking: muni-tracking)
  set par(justify: muni-justify, leading: muni-leading, spacing: 1.2em)
  // No heading numbers for cleaner section titles
  set heading(numbering: none)

  // Level 1 headings: Orange left border, spans both columns
  show heading.where(level: 1): it => {
    colbreak(weak: true)
    place(scope: "parent", float: true, top, block(
      width: 100%,
      inset: (left: 12pt, y: 8pt),
      stroke: (left: 4pt + muni-orange),
    )[
      #text(size: 18pt, weight: "bold")[#it.body]
    ])
    v(2em)
  }

  // Level 2 headings
  show heading.where(level: 2): it => {
    v(1em)
    text(size: 12pt, weight: "bold")[#it.body]
    v(0.2em)
  }

  // Level 3 headings
  show heading.where(level: 3): it => {
    v(0.6em)
    text(size: 10pt, weight: "bold")[#it.body]
    v(0.1em)
  }

  // Code blocks (same font as body, slightly smaller)
  show raw: set text(font: (muni-font-mono, ..muni-font-fallback), size: 9pt)
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
    margin: 0.5in,
    header: none,
    footer: none,
  )[
    #align(center)[
      #v(1.5in)

      // Document type (subtle)
      #text(size: 11pt, fill: gray)[
        #doc-type
      ]

      #v(0.5in)

      // Main title (large, bold)
      #text(size: 36pt, weight: "bold")[
        #title
      ]

      #if subtitle != none [
        #v(0.3em)
        #text(size: 16pt)[#subtitle]
      ]

      #v(0.5in)

      // Cover image
      #if cover-image != none [
        #image(cover-image, width: 80%)
      ]

      #v(1fr)

      // Revision and date
      #text(size: 11pt, fill: gray)[
        Revision #revision
        #h(2em)
        #date
      ]

      #v(0.5in)

      // Company info
      #text(size: 11pt)[
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
    #v(0.5in)
    #text(size: 18pt, weight: "bold")[Contents]
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
    stroke: (left: 4pt + muni-danger),
    inset: (left: 16pt, right: 12pt, y: 12pt),
    radius: (right: 4pt),
  )[
    #text(weight: "bold", fill: muni-danger)[⚠ DANGER ] #body
  ]
}

// Warning box for safety notices
#let warning(body) = {
  block(
    width: 100%,
    fill: rgb("#FFF7ED"),
    stroke: (left: 4pt + muni-orange),
    inset: (left: 16pt, right: 12pt, y: 12pt),
    radius: (right: 4pt),
  )[
    #text(weight: "bold", fill: muni-orange)[⚠ WARNING ] #body
  ]
}

// Note box for tips and information
#let note(body) = {
  block(
    width: 100%,
    fill: rgb("#EFF6FF"),
    stroke: (left: 4pt + muni-note),
    inset: (left: 16pt, right: 12pt, y: 12pt),
    radius: (right: 4pt),
  )[
    #text(weight: "bold", fill: muni-note)[ℹ NOTE ] #body
  ]
}

// Success/tip box
#let tip(body) = {
  block(
    width: 100%,
    fill: rgb("#F0FDF4"),
    stroke: (left: 4pt + muni-success),
    inset: (left: 16pt, right: 12pt, y: 12pt),
    radius: (right: 4pt),
  )[
    #text(weight: "bold", fill: muni-success)[✓ TIP ] #body
  ]
}

// =============================================================================
// Tables
// =============================================================================

// Specification table (two columns, clean styling)
#let spec-table(..args) = {
  table(
    columns: (1fr, 1fr),
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 8pt,
    fill: (_, row) => if row == 0 { rgb("#f8f8f8") } else { white },
    ..args,
  )
}

// BOM table (4 columns: Part, Qty, Unit, Total)
#let bom-table(..args) = {
  table(
    columns: (2fr, auto, auto, auto),
    stroke: 0.5pt + rgb("#e0e0e0"),
    inset: 8pt,
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
      width: 14pt,
      height: 14pt,
      fill: muni-orange,
      radius: 3pt,
      align(center + horizon)[
        #text(fill: white, size: 10pt, weight: "bold")[✓]
      ]
    )
  } else {
    box(
      width: 14pt,
      height: 14pt,
      stroke: 1.5pt + muni-gray,
      radius: 3pt,
    )
  }

  grid(
    columns: (auto, 1fr),
    column-gutter: 8pt,
    row-gutter: 6pt,
    align: horizon,
    box-style,
    body,
  )
}

// Checklist block
#let checklist(..items) = {
  block(
    width: 100%,
    fill: muni-light-gray,
    inset: 12pt,
    radius: 4pt,
  )[
    #stack(
      dir: ttb,
      spacing: 8pt,
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
    width: 24pt,
    height: 24pt,
    fill: muni-orange,
    radius: 12pt,
    align(center + horizon)[
      #text(fill: white, size: 12pt, weight: "bold")[#number]
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
