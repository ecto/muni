// BVR1 Technical Manual
// Base Vectoring Rover - Revision 1
//
// Philosophy: One thing per page. Each page answers one question.
// Structure: Content split into section files for maintainability.

#import "lib/template.typ": *
#import "lib/diagrams.typ": *

#show: manual.with(
  title: "BVR1",
  subtitle: "Base Vectoring Rover",
  revision: "1.0",
  date: "January 2026",
  doc-type: "Technical Manual",
  cover-image: "../images/bvr1-render.png",
)

// =============================================================================
// VERSION HISTORY
// =============================================================================

= About This Manual

This manual is written by the people who built and operate BVR rovers. It's not a marketing document or a wish list: it's the real procedures we use, with the mistakes we've made and the lessons we've learned.

If something is unclear, wrong, or missing, let us know. This is a living document that improves with every build.

== Revision History

#version-history(
  [1.0], [January 2026], [Initial release (based on BVR0 manual)],
)

#v(1em)

== What's New in BVR1

BVR1 builds on the BVR0 platform with several upgrades:

- *Larger wheels*: 8" wheels (vs 6.5" in BVR0) for better ground clearance and obstacle handling
- *Motor brackets*: Bicycle-style mounting brackets for easier wheel service and alignment
- *Custom battery pack*: 13S4P 21700 cells in a dedicated tray (vs downtube battery in BVR0)
- *Lighting*: Integrated headlights and tail lights for visibility and low-light operation
- *Improved sensor mast*: Reinforced design with cable routing channels

Most assembly procedures are identical to BVR0. This manual highlights BVR1-specific differences.

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
// TODO: Create BVR1-specific overview with updated specs
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
// ELECTRONICS PLATE (BVR1-specific with custom plate)
// =============================================================================
#include "manual-bvr1/electronics-plate.typ"

// =============================================================================
// DRIVETRAIN (BVR1-specific with motor brackets)
// =============================================================================
#include "manual-bvr1/drivetrain.typ"

// =============================================================================
// POWER SYSTEM (BVR1-specific with custom battery and lighting)
// =============================================================================
#include "manual-bvr1/power.typ"

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
// TODO: Create BVR1-specific BOM with motor brackets and 8" wheels
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
