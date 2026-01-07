// BVR0 Technical Manual
// Base Vectoring Rover - Revision 0
//
// Philosophy: One thing per page. Each page answers one question.
// Structure: Content split into section files for maintainability.

#import "lib/template.typ": *
#import "lib/diagrams.typ": *

#show: manual.with(
  title: "BVR0",
  subtitle: "Base Vectoring Rover",
  revision: "0.1",
  date: "December 2025",
  doc-type: "Technical Manual",
  cover-image: "../images/bvr0-disassembled.jpg",
)

// =============================================================================
// VERSION HISTORY
// =============================================================================

= About This Manual

This manual is written by the people who built and operate BVR rovers. It's not a marketing document or a wish list: it's the real procedures we use, with the mistakes we've made and the lessons we've learned.

If something is unclear, wrong, or missing, let us know. This is a living document that improves with every build.

== Revision History

#version-history(
  [0.1], [December 2025], [Initial release],
  [0.2], [January 2026], [Added time estimates, lessons learned, BOM, glossary],
)

#v(1em)

== Document Conventions

Throughout this manual you'll see:

#table(
  columns: (auto, 1fr),
  stroke: none,
  inset: 6pt,
  [#box(fill: rgb("#FEFCE8"), inset: 4pt, radius: 2pt)[#text(size: 7pt, fill: rgb("#854D0E"))[_"We learned..."_]]], [Lessons from real build experience],
  [#box(fill: rgb("#FEF3C7"), inset: 4pt, radius: 2pt)[#text(size: 7pt, fill: rgb("#92400E"))[⚡ COMMON MISTAKE]]], [Errors we've seen (and made)],
  [#difficulty-dots(2)], [Difficulty rating (1-3)],
  [~15 min], [Estimated time for procedure],
)

#v(1em)

#note[
  This is a living document. Report errors or suggestions at #link("https://github.com/muni-works/muni")[github.com/muni-works/muni].
]

#pagebreak()

// =============================================================================
// CHEAT SHEET (print and laminate)
// =============================================================================
#include "manual/reference-card.typ"

// =============================================================================
// QUICK REFERENCE (pages accessed most often)
// =============================================================================
#include "manual/quick-reference.typ"

// =============================================================================
// BEFORE YOU BEGIN
// =============================================================================
#include "manual/before-you-begin.typ"

// =============================================================================
// OVERVIEW
// =============================================================================
#include "manual/overview.typ"

// =============================================================================
// TOOLS & MATERIALS
// =============================================================================
#include "manual/tools-materials.typ"

// =============================================================================
// CHASSIS ASSEMBLY
// =============================================================================
#include "manual/chassis.typ"

// =============================================================================
// ELECTRONICS PLATE
// =============================================================================
#include "manual/electronics-plate.typ"

// =============================================================================
// DRIVETRAIN
// =============================================================================
#include "manual/drivetrain.typ"

// =============================================================================
// POWER SYSTEM
// =============================================================================
#include "manual/power.typ"

// =============================================================================
// ELECTRONICS
// =============================================================================
#include "manual/electronics.typ"

// =============================================================================
// SENSOR MAST
// =============================================================================
#include "manual/sensor-mast.typ"

// =============================================================================
// WIRING
// =============================================================================
#include "manual/wiring.typ"

// =============================================================================
// TESTING & COMMISSIONING
// =============================================================================
#include "manual/testing.typ"

// =============================================================================
// OPERATION
// =============================================================================
#include "manual/operation.typ"

// =============================================================================
// SAFETY
// =============================================================================
#include "manual/safety.typ"

// =============================================================================
// FIRMWARE
// =============================================================================
#include "manual/firmware.typ"

// =============================================================================
// MAINTENANCE
// =============================================================================
#include "manual/maintenance.typ"

// =============================================================================
// APPENDIX A: BILL OF MATERIALS
// =============================================================================
#include "manual/bom.typ"

// =============================================================================
// APPENDIX B: HARDWARE REFERENCE (1:1 Scale)
// =============================================================================
#include "manual/hardware-reference.typ"

// =============================================================================
// APPENDIX C: GLOSSARY & INDEX
// =============================================================================
#include "manual/glossary.typ"

// =============================================================================
// BACK MATTER
// =============================================================================

#v(2em)
#align(center)[
  #text(size: 10pt)[
    *Municipal Robotics* \
    Cleveland, Ohio \
    #link("https://muni.works")[muni.works]
  ]
]
