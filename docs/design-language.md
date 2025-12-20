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
