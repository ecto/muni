# Design Language

How a sidewalk robot should look and move so it feels civic—public-minded, service-oriented—rather than corporate.

## Core Principles

- **People > task**: always yield; never nudge
- **Predictability wins**: fewer signals, clearer meanings, consistent choreography
- **Quiet competence**: no "cute"; be competent, boring, helpful

---

## Visual Language

### Colors

| Role      | Specification                                                   | Rationale                          |
| --------- | --------------------------------------------------------------- | ---------------------------------- |
| Primary   | High-visibility safety orange or municipal yellow, matte finish | Reads "public works," not brand ad |
| Secondary | Cool gray or white for panels                                   | Neutral, serviceable               |
| Forbidden | Chrome, black gloss, gradients                                  | Too corporate/aggressive           |

### Graphics & Markings

- Large, legible block lettering: **"SIDEWALK SERVICE ROBOT"**
- Unit ID prominently displayed (e.g., "Beaver-12")
- City/department seal or "Municipal Robotics" badge
- Logos ≤1 small mark; no ad wraps, mascots, or lifestyle imagery

### Conspicuity

| Element       | Specification                             |
| ------------- | ----------------------------------------- |
| Chevrons      | Retroreflective, mounted low and lateral  |
| Marker lights | Amber, steady (not flashy), at corners    |
| Work light    | White, active only when tool is operating |

### Human Cues

- Front "face" panel with soft geometry
- Simple status glyph:
  - ☺ idle
  - • working
  - ! attention needed
- Avoid skeuomorphic eyes or anthropomorphic features

### Materials & Finish

- Textured polymer and powder-coat metal
- Rounded edges, soft fillets
- Visible fasteners acceptable (serviceable vibe)
- No aggressive "sports" angles

---

## Motion Language

### Speed

| Context                    | Speed              | Notes                      |
| -------------------------- | ------------------ | -------------------------- |
| Default                    | 0.8–1.2 m/s        | Human walking pace         |
| Near people                | Auto-throttle down | Reduce speed automatically |
| Near driveways/storefronts | Auto-throttle down | Increase caution           |

### Yield Choreography

1. Slow to a crawl
2. Give lateral space
3. Stop 1 m ahead of pedestrian
4. Slight 2–3° "bow" (short front suspension dip) to signal yielding

### Crosswalk Behavior

- Wait a beat after WALK signal appears
- Enter decisively
- No mid-cross stops unless emergency

### Overtaking Pedestrians

- **Don't.** Trail politely until safe passing width ≥1 m
- Announce intent with soft chime + text display
- Wait for acknowledgment or clear path

### Acceleration Profile

- Ease-in/ease-out ramps on all starts and stops
- No snappy pivots near people

---

## Communication

### Labeling

Plain-English, purpose-first labeling:

> "City Sidewalk Service Robot—Clearing Snow"

> "City Sidewalk Service Robot—Sweeping Leaves"

Stating the purpose reduces unease.

### Status Display

Top-mounted e-ink panel (sun-readable) cycling:

- Current task
- ETA to clear this block
- Hotline phone number
- QR code for issue reporting

### Audio Palette

| Sound          | Use                | Notes                  |
| -------------- | ------------------ | ---------------------- |
| Soft tick-tock | Low-speed movement | Sub-2 kHz, unobtrusive |
| Two-note chime | Before passing     | Gentle, non-startling  |
| Spoken prompt  | Only when engaged  | "Passing on your left" |

Volume adapts to ambient noise level.

### Light Semantics

| Color            | State       | Meaning            |
| ---------------- | ----------- | ------------------ |
| Amber steady     | Normal      | Operating          |
| Amber slow pulse | Caution     | Yielding/waiting   |
| White workbar    | Tool active | Work in progress   |
| Red solid        | Stopped     | Fault or emergency |

**One meaning per color. No RGB rainbows.**

### Consent Cues

Before any close pass:

1. Play two-note chime
2. Display "I'll wait" if pedestrian doesn't respond
3. Remain stationary until path is clear

---

## Form & Proportions

- **Low profile**: hip-height or below for main body
- Tools may project forward but must keep sightlines clear
- Rounded corners, soft fillets throughout
- **Visible utility**: broom/auger/brine tank looks like a tool, not a mystery box

---

## Civic Identity Kit

### Unit Livery Template

- Color block placement (vector template)
- Seal placement guidelines
- Unit ID sizing and position

### Accessibility Badge

Display on chassis:

> "Maintains ≥36 in (915 mm) clear path"

Publish exact track width on the chassis.

### Contact & Accountability

Required on all sides:

- 24/7 phone/SMS hotline
- QR code to incident form
- Unique robot ID (large, legible)

### Open Data QR

Links to:

- Current route
- Uptime statistics
- Emissions avoided
- Maintenance logs

Trust through transparency.

---

## Behavior Rules Summary

1. **People > task** — always yield; never nudge
2. **Predictability wins** — fewer signals, clearer meanings, consistent choreography
3. **Quiet competence** — no "cute"; be competent, boring, helpful

---

## Digital Design Language

The web and print presence follows an "engineering terminal" aesthetic: precise, functional, readable. Think CAD title blocks, terminal output, and technical documentation.

### Typography

| Element     | Specification                          |
| ----------- | -------------------------------------- |
| Primary     | Berkeley Mono                          |
| Fallback    | SF Mono, Courier New, system monospace |
| Size        | 12px web, 10pt print, 1.5 line-height  |
| Max width   | 80 characters                          |
| Rendering   | Subpixel antialiasing where available  |

**Font:** Berkeley Mono (licensed from [berkeleygraphics.com](https://berkeleygraphics.com/typefaces/berkeley-mono/))

Monospace conveys precision and technical credibility. All text uses the same font: no mixing of sans-serif headers with monospace body. The terminal aesthetic carries across web, print, and UI.

### Colors

| Role          | Light Mode              | Dark Mode               |
| ------------- | ----------------------- | ----------------------- |
| Background    | `#fff` (pure white)     | `#000` (pure black)     |
| Foreground    | `rgba(0,0,0,0.9)`       | `rgba(255,255,255,0.8)` |
| Accent        | `rgba(0,0,0,0.8)`       | `rgba(255,255,255,0.8)` |
| Muted         | `rgba(0,0,0,0.5)`       | `rgba(255,255,255,0.6)` |
| Safety Orange | `#ff6600`               | `#ff6600`               |
| Status Green  | `#22c55e`               | `#22c55e`               |

High contrast, pure backgrounds. Foreground uses slight transparency for softer readability. Safety orange for CTAs and brand accent.

### Layout Patterns

**Boxes** (bordered containers with inverted title bar):
- Reserved for important callouts: CTAs, safety info, key downloads
- Inverted title bar draws attention
- Use sparingly

**Sections** (title with underline, no border):
- Standard content containers
- Subtle underline separates title from content
- Primary layout unit

**Pre-formatted text**:
- All content uses `<pre>` for terminal-like rendering
- Box-drawing characters (├─ └─ │) for hierarchies and trees
- ASCII diagrams where appropriate

### Navigation

- Horizontal button row, centered
- Inverted text (light on dark)
- Hover: safety orange background
- No underlines, no brackets

### Status Indicators

| Symbol | Meaning     | Color        |
| ------ | ----------- | ------------ |
| ■      | Complete    | Green        |
| ◐      | In progress | Orange       |
| ○      | Future      | Muted        |

### Interaction States

| State   | Treatment                           |
| ------- | ----------------------------------- |
| Default | Bold text, inherit color            |
| Hover   | Inverted (foreground ↔ background)  |
| CTA     | Safety orange background, black text|

### Principles

1. **Terminal aesthetic** — monospace everywhere, box-drawing characters, pre-formatted layout
2. **High contrast** — pure black/white backgrounds, legible foregrounds
3. **Sparse emphasis** — boxes only for important callouts, not decoration
4. **Functional beauty** — the aesthetic emerges from precision, not ornamentation

---

## Print Design Language

For PDFs and printed materials (Typst-generated), we use a more traditional technical documentation style while maintaining brand consistency.

### One Thing Per Page

Technical manuals follow a strict "one concept per page" rule. Each page answers exactly one question. If a reader flips to a page, they should immediately understand what that page is for and find everything they need without turning.

**Page structure:**
- Title at top (what is this page about?)
- Primary diagram or content (the answer)
- Supporting details below (context, specs, notes)

**Prioritize by frequency of access:**
1. **Emergency procedures** - first pages after cover (crisis access)
2. **Daily checklists** - next (used every session)
3. **Controls reference** - next (consulted during operation)
4. **Build instructions** - middle (used once)
5. **Troubleshooting** - end (consulted when needed)

### Typography

| Element   | Specification                     |
| --------- | --------------------------------- |
| Body      | Berkeley Mono, 9pt                |
| Code      | Berkeley Mono, 9pt                |
| Headings  | Bold, with orange left border     |
| Leading   | 0.9em                             |
| Alignment | Left-aligned (ragged right)       |

All print documents use Berkeley Mono for consistent terminal aesthetic.

### Colors (Print)

| Role          | Hex       | Usage                    |
| ------------- | --------- | ------------------------ |
| Safety Orange | `#E86A33` | Accents, heading borders |
| Cool Gray     | `#5C5C5C` | Secondary text           |
| Light Gray    | `#F5F5F5` | Table backgrounds        |
| Danger Red    | `#C41E3A` | Critical warnings        |
| Note Blue     | `#2563EB` | Information callouts     |
| Success Green | `#16A34A` | Completion indicators    |

### Callout Hierarchy

| Type    | Left Border | Background  | Use Case                  |
| ------- | ----------- | ----------- | ------------------------- |
| Danger  | Red         | Light red   | Life safety, damage risk  |
| Warning | Orange      | Light orange| Caution, potential issues |
| Note    | Blue        | Light blue  | Tips, information         |
| Tip     | Green       | Light green | Best practices            |

### Diagrams

Technical diagrams use the cetz library with consistent styling:
- Black strokes (1-1.5pt)
- Light gray fills for components
- Orange callout numbers
- Dimension lines with measurements
