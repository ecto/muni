// Whitepaper: Robots that build robots (Municipal Robotics master plan)
// Styled to the muon "Pilot Light" treatment, dark theme with the neon accent set, translated to print.
// Tokens: frame-bootc/design/agency/treatment/tokens.toml [dark] + palette.toml [neon] + [type].

#import "@preview/cetz:0.4.2"
#import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge

// ---- colour roles ([light]) ------------------------------------------------
// Page is warm black, full bleed; renders come from vcad-render --theme dark, flattened onto `bg` and trimmed (images-native/).
#let c-bg      = rgb("#0b0907")   // warm black, full bleed
#let c-surface = rgb("#15110d")
#let c-raised  = rgb("#1e1813")
#let c-line    = rgb("#2a2119")   // hairlines, non-text
#let c-line-hi = rgb("#4a3a2a")   // box outlines, non-text
#let c-text    = rgb("#e8e1d2")
#let c-label   = rgb("#a08d70")
#let c-sel     = rgb("#ff5f00")   // neon orange: one flame per page
#let c-sel-ink = rgb("#000000")
#let c-ok      = rgb("#39ff14")   // neon green: measured / reachable
#let c-warn    = rgb("#ffd400")   // neon amber: estimate
#let c-fault   = rgb("#ff2052")   // neon red: failed / unreachable
#let c-info    = rgb("#00f0ff")   // neon cyan: model output

// ---- type ([type]) ----------------------------------------------------------
// Plex ships each weight as its own family name, so weights are picked by family.
// Plex Mono has no Greek: θ and μ fall back to Plex Sans.
#let sans    = ("IBM Plex Sans",)
#let sans-m  = ("IBM Plex Sans Medm", "IBM Plex Sans")
#let sans-sb = ("IBM Plex Sans SmBld", "IBM Plex Sans")
#let mono    = ("IBM Plex Mono", "IBM Plex Sans")
#let mono-m  = ("IBM Plex Mono Medm", "IBM Plex Sans Medm")
#let mono-sb = ("IBM Plex Mono SmBld", "IBM Plex Sans SmBld")
#let track = 0.12em
// Type scale (one ladder, ~1.17-1.24 steps, display jump):
//   6.5 xs  labels, header/footer, table heads, diagram floor
//   7.5 sm  table body, captions, diagram text
//   9   body
//   10.5 md h2, abstract
//   13  lg  h1, subtitle, contents title
//   16  xl  title-page readouts
//   44  display title
// Plex Sans and Plex Mono share x-height 516/1000 and cap 698/1000 (OS/2),
// so mono matches sans x-height at the same point size: ratio 1:1.
#let s-xs = 6.5pt
#let s-sm = 7.5pt
#let body-size = 9pt
// body weight: "400" Regular, "450" Plex Sans Text, "300" Light (override: --input body=300)
#let body-variant = sys.inputs.at("body", default: "400")
#let body-font = if body-variant == "450" { ("IBM Plex Sans Text", "IBM Plex Sans") } else { sans }
#let body-weight = int(body-variant)
#let s-md = 10.5pt
#let s-lg = 13pt
#let s-xl = 16pt
#let s-display = 44pt
#let lead-gap = 0.84em // inter-line gap; ~1.62 line-height on dark, room to breathe
#let nb = sym.space.nobreak.narrow

// tracked caps label (section labels, figure labels, header, overlines)
#let caps(body, size: s-xs, fill: c-label, font: mono-m) = text(font: font, size: size, tracking: track, fill: fill, upper(body))
// the one flame: selection fill with sel_ink on it. at most one per page.
#let flame(body) = box(fill: c-sel, inset: (x: 2pt), outset: (y: 2pt), text(fill: c-sel-ink, body))
// status chip: a square marker plus the word, both in the status colour
#let status(role, word) = {
  let c = (ok: c-ok, info: c-info, warn: c-warn, fault: c-fault, label: c-label).at(role)
  box(text(font: mono-m, fill: c, tracking: track, size: s-xs)[#box(width: 0.55em, height: 0.55em, fill: c, baseline: -0.05em) #upper(word)])
}
#let fault(body) = text(fill: c-fault, font: mono-m, body)
// instrument readout: tracked-caps label over a mono 500 value
#let readout(label, value) = stack(spacing: 0.6em, caps(label), text(font: mono-m, size: body-size, number-width: "tabular", value))

// square frame for images and callouts
#let frame(body, inset: 0pt, stroke: 0.6pt + c-line-hi, fill: none, width: auto) = block(width: width, stroke: stroke, inset: inset, fill: fill, radius: 0pt, body)

// fletcher node shape: square
#let sqnode = fletcher.shapes.rect

#set document(title: "Robots that build robots", author: "Cam Pedersen (Municipal Robotics)")

// how far the header and footer reach past the text, to sit 0.7in from the page edge
#let hf-out() = { let m = page.margin; let l = if type(m) == dictionary { m.at("left", default: m.at("x", default: 0.75in)) } else { 0.75in }; calc.max(0in, l - 0.7in) }

#set page(
  paper: "us-letter",
  fill: c-bg,
  margin: (x: 1.85in, top: 1.1in, bottom: 1.05in),
  header-ascent: 0.45in,
  footer-descent: 0.45in,
  columns: 1,
  numbering: "1",
  header: context {
    if counter(page).get().first() > 1 {
      pad(x: -hf-out(), grid(columns: (1fr, auto), caps[Robots that build robots], caps[Municipal Robotics]))
    }
  },
  footer: context {
    if counter(page).get().first() > 1 {
      pad(x: -hf-out(), grid(columns: (1fr, auto, 1fr),
        caps[Rev 1.0 · October 2026],
        text(font: mono-m, size: s-xs, fill: c-label, number-width: "tabular")[#{ let n = counter(page).get().first(); if n < 10 { "0" + str(n) } else { str(n) } } / #counter(page).final().first()],
        align(right, caps[Whitepaper])))
    }
  },
)

#set text(font: body-font, weight: body-weight, size: body-size, fill: c-text, fallback: false, lang: "en", region: "us", hyphenate: false, costs: (runt: 1000%, widow: 200%, orphan: 200%))
#set columns(gutter: 0.27in)
#set par(justify: false, leading: lead-gap, spacing: 1.4em, linebreaks: "optimized")
#set heading(numbering: "1.1")
#show strong: set text(font: sans-sb)
#show link: set text(fill: c-text)
#show heading: set text(weight: "regular", fill: c-text)
#let numlabel(it) = text(font: mono-m, size: s-xs, fill: c-label, tracking: track, { if it.numbering == "A.1" and it.level == 1 { upper("Appendix ") }; counter(heading).display(it.numbering) })
#show heading.where(level: 1): it => if it.supplement == [Part] { block(above: 0em, below: 0.9em, text(font: sans-sb, size: 26pt, tracking: -0.02em, it.body)) } else { block(above: 2.2em, below: 1.1em, sticky: true, width: 100%, {
  if it.numbering != none { place(left, dx: -1.15in, dy: 0.32em, box(width: 1.0in, align(right, numlabel(it)))) }
  text(font: sans-sb, size: s-lg, tracking: -0.01em, it.body)
}) }
#show heading.where(level: 2): it => block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, {
  if it.numbering == none { return text(font: sans-m, size: s-md, it.body) }
  set par(justify: false)
  place(left, dx: -1.15in, dy: 0.22em, box(width: 1.0in, align(right, numlabel(it))))
  text(font: sans-m, size: s-md, it.body)
})
#show heading.where(level: 3): it => block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, text(font: sans-sb, size: body-size, it.body))
#show raw: set text(font: mono, size: 1em)
// Keep numbers with their units, and short expressions, on one line
#let unit-re = "(mm|kg|g|N·m|N·s|N|kN|W|kW|Hz|ms|s|m|cm|in|V)\\b"
#show regex(" [×≈=≤≥] \\d[\\d.,]*( " + unit-re + ")?"): it => it.text.replace(" ", "\u{a0}")
#show regex("\\d " + unit-re): it => it.text.replace(" ", "\u{a0}")
#show regex("–\\d"): it => it.text.replace("–", "–\u{2060}")
// list marker: a small square centred on the x-height, like a register mark
#set list(marker: box(baseline: -0.07em, rect(width: 0.36em, height: 0.36em, fill: c-label, stroke: none)), indent: 0em, body-indent: 0.85em, spacing: 1.15em)
#show list: set par(justify: false)
#show enum: set par(justify: false)
#set enum(numbering: n => text(font: mono-m, size: s-sm, fill: c-label, number-width: "tabular", str(n)), indent: 0em, body-indent: 0.9em, spacing: 1.15em)

// Tables: mono on a grid, hairlines between rows, line_hi above and below, tracked-caps header
#set table(stroke: (x, y) => (
    top: if y == 0 { 0.6pt + c-line-hi } else { 0.4pt + c-line },
    bottom: 0.6pt + c-line-hi,
  ), inset: (x: 5pt, y: 4.5pt))
#show table: set text(font: mono, size: s-sm, number-width: "tabular", hyphenate: false)
#show table: set par(leading: 0.72em, justify: false)
#show table: it => { show strong: set text(font: mono-sb); it }
// Prose tables: sans body, mono tracked-caps header kept
#let ptable(..a) = {
  show table: set text(font: sans)
  show table.cell: it => { show strong: set text(font: sans-sb); it }
  table(..a)
}
#show table.cell.where(y: 0): it => {
  set text(font: mono-m, size: s-xs, fill: c-label, tracking: track)
  show strong: s => upper(s.body)
  it
}

#set figure(placement: top, scope: "parent", gap: 1.1em)
#show figure: set block(above: 2.8em, below: 2.8em)
#show figure: set text(hyphenate: false, font: sans, weight: 400)
#show table: set text(weight: 400)
#show heading: set text(font: sans, weight: 400, hyphenate: false)
#set place(clearance: 3em)
// short lead phrase of a caption, set in Medium
#let lead(body) = text(font: sans-m, fill: c-text, body)
// citations: superscript numbers in the label grey, so they never read as exponents
#show cite: set text(fill: c-label)
#show figure.where(kind: table): set figure.caption(position: top)
#show figure.caption: it => {
  set par(leading: 0.64em, justify: false)
  set align(left)
  block(width: 100%, text(font: sans, size: s-sm, fill: c-label, hyphenate: false)[
    #caps(if it.kind == table [Table] else [Fig] + nb + it.counter.display(), font: mono-sb, fill: c-text)#h(0.7em)#it.body
  ])
}

// Part divider: a page of its own; a level-1 heading with supplement [Part] so it reaches the contents
#let roman(n) = ("I", "II", "III", "IV", "V", "VI").at(n - 1)
#let part(n, title, lede) = {
  pagebreak(weak: true)
  v(1.2in)
  block(below: 0.6em, text(font: mono-m, size: s-xs, fill: c-label, tracking: track, upper("Part " + n)))
  heading(level: 1, numbering: none, supplement: [Part], outlined: true, title)
  block(width: 80%, text(size: s-md, fill: c-label, lede))
  pagebreak()
}

// Column-width figure that stays in flow
// Wide figures break out of the text column into both margins; their captions stay on the text measure
#show figure: it => if it.scope == "parent" { block(above: 2.8em, below: 2.8em, pad(x: -1.15in, top: 2.95em, bottom: 0em, { show figure.caption: c => pad(x: 1.15in, align(left, c)); it })) } else { it }
#show figure.where(kind: table): it => { show table: t => align(left, t); it }


#let colfig(body, caption: none) = figure(body, caption: caption, placement: none, scope: "column")

// Diagram node fills: grey structure only
#let n-fill = c-surface
#let n-stroke = 0.7pt + c-line-hi

// Title page (single column). The one flame: the orange rule under the title.
#page(columns: 1, margin: (x: 0.75in, top: 0.7in, bottom: 0.65in))[
  // Cover art: the inner solar system in a tilted plan view, an Earth → Mars → Jupiter
  // gravity-assist trajectory, and the Sun–Earth Lagrange points. Schematic, not to scale.
  #place(top + left, dx: -0.75in, dy: -0.7in, box(width: 8.5in, height: 11in, clip: true, place(center + horizon, dy: 1.9in, cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let cx = 0
    let cy = 0
    let tilt = 0.5
    let zoom = 2.05
    let P(r, a) = (cx + zoom * r * calc.cos(a), cy + zoom * tilt * r * calc.sin(a))
    let orbit(r, w, col) = {
      let pts = range(0, 181).map(k => P(r, k * 2deg))
      line(..pts, close: true, stroke: w + col)
    }
    // faint star field
    let seed = 7
    for k in range(160) {
      seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
      let x = calc.rem(seed, 2400) / 100 - 12.0
      seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
      let y = calc.rem(seed, 2400) / 100 - 12.0
      circle((x, y), radius: 0.012 + calc.rem(k, 3) * 0.008, fill: c-label.transparentize(40%), stroke: none)
    }
    // orbits: Mercury, Venus, Earth, Mars, Jupiter (radii compressed)
    let rs = (1.15, 1.85, 2.75, 3.85, 7.9)
    for (k, r) in rs.enumerate() {
      orbit(r, if k == 2 { 0.7pt } else { 0.45pt }, if k == 2 { c-line-hi.lighten(25%) } else { c-line-hi })
    }
    // the sun, with a soft glow
    for (k, g) in ((0.95, 92%), (0.7, 85%), (0.5, 72%), (0.36, 50%)).enumerate() {
      circle((cx, cy), radius: g.at(0), fill: c-sel.transparentize(g.at(1)), stroke: none)
    }
    circle((cx, cy), radius: 0.26, fill: c-sel, stroke: none)
    // the trajectory: Earth departure, Mars flyby, out to Jupiter
    let smooth(t) = t * t * (3 - 2 * t)
    let a0 = 165deg
    let a1 = 300deg
    let a2 = 392deg
    let traj = ()
    for k in range(0, 121) {
      let a = a0 + (a2 - a0) * k / 120
      let r = if a <= a1 {
        2.75 + (3.85 - 2.75) * smooth((a - a0) / (a1 - a0))
      } else {
        let t = (a - a1) / (a2 - a1)
        3.85 + (9.6 - 3.85) * calc.pow(t, 1.35)
      }
      traj.push(P(r, a))
    }
    for (w, a) in ((5pt, 88%), (2.6pt, 72%)) { line(..traj, stroke: (paint: c-sel.transparentize(a), thickness: w, cap: "round", join: "round")) }
    line(..traj, stroke: (paint: c-sel, thickness: 0.9pt, cap: "round", join: "round"))
    // planets
    let planet(r, a, rad, col, name, dx: 0.18, dy: 0.18, anchor: "south-west") = {
      let q = P(r, a)
      circle(q, radius: rad * 2.4, fill: col.transparentize(85%), stroke: none)
      circle(q, radius: rad, fill: col, stroke: none)
      content((q.at(0) + dx, q.at(1) + dy), anchor: anchor, text(font: mono-m, size: 5.5pt, fill: c-label, tracking: 0.08em, upper(name)))
    }
    planet(1.15, 120deg, 0.05, c-label, "Mercury")
    planet(1.85, 60deg, 0.075, c-text, "Venus")
    planet(2.75, a0, 0.085, c-info, "Earth", dx: -0.12, dy: 0.2, anchor: "south-east")
    planet(3.85, a1, 0.07, c-fault, "Mars", dx: 0.0, dy: 0.2, anchor: "south")
    planet(7.9, 118deg, 0.16, c-warn, "Jupiter", dx: 0.35, dy: 0.0, anchor: "west")
    // moon of Earth
    let e = P(2.75, a0)
    circle((e.at(0) + 0.2, e.at(1) + 0.07), radius: 0.025, fill: c-text, stroke: none)
    // Sun–Earth Lagrange points (L1, L2 exaggerated off Earth)
    let lp(q, name, dx, dy, anchor) = {
      let (x, y) = q
      line((x - 0.07, y), (x, y + 0.07), (x + 0.07, y), (x, y - 0.07), close: true, fill: none, stroke: 0.7pt + c-info)
      content((x + dx, y + dy), anchor: anchor, text(font: mono-m, size: 5.5pt, fill: c-info, name))
    }
    lp(P(2.4, a0), "L1", 0.12, 0.0, "west")
    lp(P(3.1, a0), "L2", -0.12, 0.0, "east")
    lp(P(2.75, a0 + 180deg), "L3", 0.14, 0.0, "west")
    lp(P(2.75, a0 + 60deg), "L4", 0.12, -0.04, "north-west")
    lp(P(2.75, a0 - 60deg), "L5", 0.0, 0.14, "south")
    // trajectory notes
    let m = P(3.85, a1)
    content((m.at(0) + 0.1, m.at(1) - 0.3), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-sel, tracking: 0.08em)[GRAVITY ASSIST])
    let j = traj.at(84)
    content((j.at(0) - 0.2, j.at(1) + 0.25), anchor: "south-east", text(font: mono-m, size: 5.5pt, fill: c-sel, tracking: 0.08em)[TO THE OUTER SYSTEM →])
  }))))
  #set par(leading: 0.5em)
  #grid(columns: (auto, 1fr, auto), column-gutter: 0.8em, align: horizon,
    caps(fill: c-text)[Municipal Robotics · Whitepaper],
    line(length: 100%, stroke: 0.5pt + c-line-hi),
    caps[Rev 1.0 · October 2026])
  #v(1.6em)
  #caps[A specification · from the first arm to the sun]
  #v(0.9em)
  #move(dx: -0.045 * s-display, text(font: sans-sb, size: s-display, tracking: -0.02em)[Robots that\ build robots])
  #v(0.5em)
  #line(length: 22%, stroke: 2pt + c-sel)
  #v(0.7em)
  #text(size: s-lg, fill: c-label)[What would it take for a machine to make the next one?]
  #v(1fr)
  #line(length: 100%, stroke: 0.6pt + c-line-hi)
  #v(0.6em)
  #grid(columns: (1fr,) * 3, column-gutter: 1.2em, row-gutter: 1.3em,
    readout[Author][Cam Pedersen], readout[Place][Cleveland, Ohio], readout[Contact][#link("mailto:info@muni.works")[info\@muni.works]],
  )
]

// Abstract (single column)
#page(columns: 1, margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))[
  #block(width: 4.9in)[
  #heading(outlined: false, numbering: none)[Abstract]
  #set text(size: 10pt)
  #set par(leading: 0.8em, spacing: 1.25em)

  This is a specification for how humanity gets to space: not by one heroic launch, but by machines that make the next machine, each one measured, until the bought fraction falls to zero and a seed of them can go where no supply chain reaches. It starts on a sidewalk in Ohio.

  Robots are expensive because people make them. A prototype actuator we designed is estimated at \$1,086 to have built, before its gear teeth are cut; about two thirds of that is outsourced machining, on perhaps \$60–90 of raw metal (estimates). The difference is labor, coordination and margin. A machine that can make more of itself attacks that markup directly. So does a design process that measures the sim-to-real gap on every unit and feeds it back into the design as a gradient.

  Municipal Robotics is one person, Cam Pedersen. What exists today is a Booster K1 humanoid, the software stack, a CNC mill and this plan. The stack spans the loop: a source language for parts, a CAD kernel, a differentiable physics engine with a contact adjoint, an actuator, an arm, and ipse, a developmental learning system for the K1. Its key result exists only in simulation: five of the K1's inertial parameters, started at ±20% error, recovered to machine precision by gradient descent through contact.

  The plan is built from one part. A limb is a six-axis arm on purchased RobStride actuators (\$5,000 list, \~\$2,000 materials), designed and ready to cut, none built yet. Two limbs bolted to any table make a pair (\$12,000 list, target), which can fold laundry, tend a machine or build the next limb. A humanoid is a pair on legs. Our models put the arm's reach at 786 mm with our 485 g gripper and its continuous hold straight out at 1.27 kg. The municipal work is a proposed litter pilot in Lakewood, Ohio, with no agreement in place; the method, a reacher on the K1's wrist, is under design, and the K1 has not grasped litter yet.

  Limb, the first stage, closes on exact gates (proposed targets): ten limbs sold at 40% gross margin, each with its tool-point error measured and under 2 mm RMS, and our actuator beating the purchased one by 20% on a named dyno metric, or dropped. Seven more stages follow, each a step up in scale: Pair, two limbs at any table, and a pair that builds a pair; Body, a pair on legs that fits its own model; City, a fleet and its pairs work a city; Planet, lineages on their own power; Moon, a seed grows a pair, with a lifeline; Mars, the same seed, lifeline cut; and Sun, a swarm of collectors around our star. They are goals, not results, and they are the point. Progress is one number, electrons under management: the mean power drawn by machines we built, in service. Today it is zero watts. The body-schema claim that opens Body will be preregistered, with its falsifier, before its hardware run. Every number here is cited, or labeled as an estimate or a model output.
]
]


// Table of contents (single column)
#page(columns: 1, margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))[
  #set text(font: sans, weight: 400, size: 9pt, number-width: "tabular")
  #set par(leading: 0.6em, justify: false)
  #set outline.entry(fill: none)
  #show outline.entry: it => {
    let l1 = it.level == 1
    if l1 and it.element.supplement == [Part] {
      let n = query(heading.where(level: 1).before(it.element.location())).filter(h => h.supplement == [Part]).len()
      return block(above: if n == 1 { 0em } else { 2.0em }, below: 0.2em, link(it.element.location(), text(font: mono-m, size: s-xs, fill: c-label, tracking: track, upper("Part " + roman(n) + " · ") + upper(it.body()))))
    }
    let num = if it.prefix() != none { it.prefix() } else { [] }
    let pg = text(font: mono, size: s-xs, fill: c-label, number-width: "tabular", it.page())
    set par(leading: 0.5em)
    // cap-height top edges, so a small mono number and a larger title line up on their first line
    set text(top-edge: "cap-height", bottom-edge: "baseline")
    block(above: if l1 { 1.25em } else { 0.62em }, below: 0em, breakable: false, link(it.element.location(),
      if l1 {
        grid(columns: (2.2em, 1fr, 2.4em), align: (left + top, left + top, right + bottom),
          text(font: mono-m, size: s-xs, fill: c-label, tracking: track, num),
          text(font: sans-m, size: 9.5pt, fill: c-text, it.body()),
          pg)
      } else {
        grid(columns: (2.2em, 2.4em, 1fr, 2.4em), align: (left + top, left + top, left + top, right + bottom),
          [],
          text(font: mono, size: 6pt, fill: c-label, tracking: track, num),
          text(font: sans, size: 8.5pt, fill: c-label, it.body()),
          pg)
      }))
  }
  #text(font: sans-sb, size: s-lg)[Contents]
  #v(0.6em)
  #columns(2, gutter: 3.2em, outline(
    title: none,
    indent: 0em,
    depth: 2,
  ))
]

#part("I", "Why")[Where this comes from, what has changed, and the whole plan on two pages.]

= Introduction

Every civilization that left its planet would have had to solve one problem first: making machines without people making each machine. Launch is a transport problem. Staying is a manufacturing problem. A settlement that needs a factory on Earth for every bearing, plate and chip is a colony on a supply line; one that carries a cell able to build the next cell is the start of something that grows. This paper is a specification for that cell, and for the measured, gated path from a litter-picking robot on an Ohio sidewalk to a lineage of machines powered by the sun.

The paper is written for anyone who wants to check it or join it: technical readers, buyers of a limb or a pair, municipal partners, collaborators and investors. It keeps the register of our sidewalk-maintenance paper @muni2025whitepaper: plain claims, numbers with sources, exact gates, and a section on what we do not claim.

Most of what this paper describes is *proposed*, not operational. @tab:status separates them. Throughout the paper, "modeled" means a number comes from our own CAD, kinematics or load scripts and has not been measured on hardware. "Measured" is reserved for results from our own hardware, and this paper reports none yet. Vendor ratings and public data are cited as sourced.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Item*], [*State*], [*Evidence*]),
    [Booster K1], [In hand], [Purchased platform; vendor firmware @booster2025k1],
    [Software stack], [Exists, in use, not finished], [@tab:stack @pond2026strategy],
    [CNC mill], [In hand], [–],
    [Limb, rose fork v9], [Designed, ready to cut; none built yet], [Fab package, 20 DXFs, 88-screw BOM @rose2026forkv9],
    [Gripper v2], [Designed; none built], [CAD + force/mass model @mr2026gripper],
    [Pair], [Concept; mount not designed], [Two limbs on a table plate],
    [Seed], [Concept], [None],
    [rana actuator], [Prototype BOM, estimated; not built], [\$1,086 est. per unit, before tooth cutting @rana2026bom],
    [Actuator dyno], [Planned; does not exist], [None],
    [Pack, blade, board library], [Concept mockups; none built], [Pack and blade models @mr2026battery @mr2026compute],
    [moth drone], [Proposed; not designed], [kino drives a bought drone today],
    [K1 litter reacher], [Concept under design], [None; the K1 has not grasped litter],
    [K1 body-schema fit (sim)], [Five-parameter recovery in simulation], [Synthetic data, sphere feet; ±20% → machine precision @ipse2026brief],
    [K1 body-schema fit (hardware)], [Not yet on hardware], [Preregistration pending],
    [Lakewood litter pilot], [Proposed], [No agreement in place],
    [Body through Sun], [Goals, not claims], [None],
  ),
  placement: bottom,
  caption: [#lead[What exists, what is designed, and what is proposed] (October 2026). What exists: a K1, the software stack, a CNC mill and a plan.],
) <tab:status>

= Origins <sec:story>

Municipal Robotics started in Cleveland in December 2025 with one machine for one job: a small rover to clear snow from sidewalks. Our first paper made the case for it, with Lakewood, Ohio as the reference city @muni2025whitepaper. By the end of that month the first rover, bvr0, drove under teleoperation with GPS and a depot behind it. #text(fill: c-warn)[[TODO: Founders Inc: when we joined, what it gave us, and what came of it.]]

Building the rover taught us the lesson this paper is built on. The hard and expensive parts of a robot are not the parts. They are the people who design, build and check each one. To build one rover, we wrote a language for parts, a CAD kernel, a differentiable physics engine and a recording pipeline. None of them cares whether the thing it designs is a rover. Pointed at a humanoid, or at a pair of arms that builds more arms, they work the same way.


#colfig(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    // timeline
    let y = 0
    line((0, y), (8.2, y), stroke: 0.7pt + c-line-hi, mark: (end: "straight", size: 0.14))
    let ev = ((0.3, "DEC 2025", "bvr0 drives"), (2.5, "2026", "the stack grows"), (4.7, "2026", "K1 bought"), (6.9, "OCT 2026", "this plan"))
    for (x, d, t) in ev {
      circle((x, y), radius: 0.07, fill: c-text, stroke: none)
      content((x, y - 0.18), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-label, d))
      content((x, y - 0.48), anchor: "north", text(size: s-xs, fill: c-text, t))
    }
    // one stack fanning out
    let sx = 0.3
    rect((sx - 0.25, 1.0), (sx + 1.75, 3.6), fill: c-surface, stroke: 0.6pt + c-line-hi)
    let layers = ("loon", "vcad", "phyz", "kerf")
    for (i, l) in layers.enumerate() {
      rect((sx - 0.1, 1.15 + i * 0.6), (sx + 1.6, 1.6 + i * 0.6), fill: c-raised, stroke: none)
      content((sx + 0.75, 1.375 + i * 0.6), text(font: mono-m, size: s-xs, fill: c-text, l))
    }
    content((sx + 0.75, 3.75), anchor: "south", text(font: mono-m, size: 5.5pt, fill: c-label)[ONE STACK])
    let outs = ((3.25, "rover", "retired", c-label), (2.3, "humanoid", "rose, K1", c-text), (1.35, "pair", "builds limbs", c-text))
    let x = 5.6
    for (ty, n, w, col) in outs {
      line((sx + 1.75, 2.3), (x - 1.0, ty), stroke: (paint: c-label, thickness: 0.6pt, dash: if n == "rover" { "dotted" } else { none }), mark: (end: "straight", size: 0.12))
      rect((x - 1.0, ty - 0.38), (x + 1.4, ty + 0.38), fill: if n == "rover" { none } else { c-surface }, stroke: if n == "pair" { 1pt + c-sel } else { 0.6pt + c-line-hi })
      content((x - 0.85, ty), anchor: "west", text(font: sans-m, size: s-sm, fill: col, n))
      content((x + 1.3, ty), anchor: "east", text(size: 5.5pt, fill: c-label, w))
    }
  }),
  caption: [#lead[One stack, three bodies.] Tools written to build one rover do not care what they design. Pointed at a humanoid, or at a pair of arms that builds arms, they work the same way.],
) <fig:story>

At the same time, the body worth building changed. A rover clears one kind of sidewalk in one season. A humanoid that a small company can buy (@sec:why) works where people work, and two good arms fold laundry, tend a machine or pick up litter. So we retired the rover #text(fill: c-warn)[[TODO: when, and the reason in your words]] and kept the city, the sidewalk and the discipline of our first paper: plain claims, sourced numbers, exact gates.

Municipal Robotics is a one-person company: Cam Pedersen. There is no team and no lab behind it. What exists today is exactly this: a Booster K1 humanoid, the software stack (@tab:stack), a CNC mill and the plan you are reading. No arm has been built, there is no actuator dyno yet, and the K1 has not grasped litter. The ambition grew because the tools did. A person who can write every layer between a parameter in a source file and a measured joint torque can aim at the machine that makes machines, and should say so in public, with gates.

= The scale <sec:kardashev>

In 1964 the astronomer Nikolai Kardashev proposed ranking civilizations by the power they command @kardashev1964. A Type I civilization uses the energy available on its planet, a Type II its star's, and a Type III its galaxy's. Carl Sagan turned the types into a continuous scale, $K = (log_(10) P - 6) \/ 10$ for a power $P$ in watts, which puts Type I at 10#super[16] W and Type II at 10#super[26] W @sagan1973cosmic.

Humanity used about 620 EJ of primary energy in 2023 @ei2024review: a mean of about 2 × 10#super[13] W, or $K approx 0.73$. Its electricity alone, about 30,000 TWh a year, is a mean of about 3.4 × 10#super[12] W @ei2024review. The sun puts out 3.828 × 10#super[26] W @iau2015b3, about $K approx 2.06$. Between us and Type I lie three orders of magnitude; between us and our star, thirteen.


#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    // log10 watts from 11 to 37
    let W = 17.2
    let e0 = 11
    let e1 = 28
    let X(e) = (e - e0) / (e1 - e0) * W
    let y0 = 0
    line((0, y0), (W, y0), stroke: 0.7pt + c-line-hi, mark: (end: "straight", size: 0.16))
    for e in range(12, 28, step: 2) {
      line((X(e), y0), (X(e), y0 - 0.1), stroke: 0.5pt + c-line-hi)
      content((X(e), y0 - 0.18), anchor: "north", text(font: mono, size: 5.5pt, fill: c-label)[10#super[#str(e)]])
    }
    content((W, y0 - 0.55), anchor: "north-east", text(font: mono, size: s-xs, fill: c-label)[MEAN POWER, WATTS (LOG)])
    // Kardashev types (Sagan): K = (log P - 6)/10
    let types = ((16, "TYPE I", "a planet's energy"), (26, "TYPE II", "a star's"))
    for (e, n, w) in types {
      line((X(e), y0), (X(e), 2.4), stroke: (paint: c-line-hi, thickness: 0.7pt, dash: "dashed"))
      content((X(e) + 0.08, 2.4), anchor: "north-west", text(font: mono-m, size: s-xs, fill: c-text, n))
      content((X(e) + 0.08, 2.05), anchor: "north-west", text(size: 5.5pt, fill: c-label, w))
    }
    // the sun
    let es = calc.log(3.828e26, base: 10)
    circle((X(es), 0.55), radius: 0.22, fill: c-sel, stroke: none)
    content((X(es) - 0.32, 0.55), anchor: "east", text(font: mono-m, size: 5.5pt, fill: c-text)[THE SUN])
    // humanity, primary energy (estimates)
    let hist = ((1800, 6.4e11), (1900, 1.37e12), (1950, 3.25e12), (2000, 1.33e13), (2023, 1.96e13))
    let prev = none
    for (i, (yr, w)) in hist.enumerate() {
      let e = calc.log(w, base: 10)
      let y = 0.45 + i * 0.32
      circle((X(e), y), radius: 0.07, fill: if yr == 2023 { c-sel } else { c-text }, stroke: none)
      if prev != none { line(prev, (X(e), y), stroke: 0.7pt + c-text) }
      prev = (X(e), y)
      content((X(e) - 0.12, y), anchor: "east", text(font: mono, size: 5.5pt, fill: if yr == 2023 { c-text } else { c-label }, str(yr)))
    }
    content((X(13.3) + 0.15, 1.75), anchor: "west", text(size: s-xs, fill: c-text)[humanity today, K ≈ 0.73])
    // gap brackets
    line((X(13.3), 2.75), (X(16), 2.75), stroke: 0.6pt + c-label, mark: (start: "straight", end: "straight", size: 0.1))
    content((X(14.65), 2.82), anchor: "south", text(font: mono, size: 5.5pt, fill: c-label)[×1,000])
    line((X(16), 2.75), (X(26.58), 2.75), stroke: 0.6pt + c-label, mark: (start: "straight", end: "straight", size: 0.1))
    content((X(21.3), 2.82), anchor: "south", text(font: mono, size: 5.5pt, fill: c-label)[×10 billion more to the sun])
  }),
  caption: [#lead[Where humanity sits on the Kardashev scale.] Humanity's mean primary power use from 1800 to 2023 (estimates from @smil2017energy @ei2024review), against Sagan's continuous form of Kardashev's Types I and II @kardashev1964 @sagan1973cosmic and the sun's output @iau2015b3. Two centuries took us about one and a half orders of magnitude. Type II is thirteen more.],
) <fig:kardashev>

Every earlier step up the scale came from a machine that made energy or work cheaper. Fire and farming fed muscle. The steam engine turned heat into work, electricity carried work anywhere, and the transistor made control almost free. One thing has not become cheaper at anything like that rate: making the machines themselves. Every machine still passes through human hands, so the number of machines grows about as fast as the number of people who make them.


#colfig(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let steps = (("fire, farming", "food → muscle"), ("steam", "heat → work"), ("electricity", "work, anywhere"), ("transistor", "control, almost free"), ("making machines", "still by hand"))
    let w = 1.62
    let h = 0.55
    for (i, (n, what)) in steps.enumerate() {
      let x = i * w
      let last = i == 4
      rect((x, 0), (x + w - 0.08, (i + 1) * h), fill: if last { none } else { c-surface }, stroke: if last { (paint: c-sel, thickness: 1pt, dash: "dashed") } else { 0.6pt + c-line-hi })
      content((x + 0.06, (i + 1) * h + 0.1), anchor: "south-west", box(width: (w - 0.1) * 1cm, text(size: 5.5pt, fill: if last { c-text } else { c-text }, font: sans-m, n)))
      content((x + 0.06, 0.08), anchor: "south-west", box(width: (w - 0.15) * 1cm, text(size: 5pt, fill: c-label, what)))
    }
    content((4 * w + (w - 0.08) / 2, 1.5), text(font: mono-m, size: 5.5pt, fill: c-sel)[NEXT?])
  }),
  caption: [#lead[Each climb made one thing cheap.] The step not yet taken is making the machines themselves.],
) <fig:stairs>

Getting off the planet is now mostly that problem. Launch has become cheap. Reusable rockets cut the cost of reaching low Earth orbit from about \$54,500 per kg on the Space Shuttle to about \$2,700 per kg on a Falcon 9 @csis2022launch. What stays expensive is everything a settlement needs when it arrives, and the people needed to make it. A civilization does not climb toward Type II by launching factories. It climbs by sending a seed that can grow one.


#colfig(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let rows = (("Space Shuttle", 54500), ("Falcon 9", 2720), ("Falcon Heavy", 1410))
    let k = 6.0 / 54500
    for (i, (n, v)) in rows.enumerate() {
      let y = -i * 0.62
      content((1.9, y + 0.2), anchor: "east", text(size: s-sm, fill: c-text, n))
      rect((2.05, y), (2.05 + calc.max(v * k, 0.04), y + 0.4), fill: if i == 0 { c-raised } else { c-sel }, stroke: none)
      let lab = "$" + str(calc.floor(v / 1000)) + "," + (if calc.rem(v, 1000) < 100 { "0" } else { "" }) + (if calc.rem(v, 1000) < 10 { "0" } else { "" }) + str(calc.rem(v, 1000)) + " / kg"
      content((2.05 + v * k + 0.12, y + 0.2), anchor: "west", text(font: mono-m, size: s-xs, fill: c-text, lab))
    }
  }),
  caption: [#lead[Cost to reach low Earth orbit, per kg] (approximate, CSIS @csis2022launch). Launch fell about twentyfold. The settlement at the other end has not.],
) <fig:launch>

This plan gives that climb one number of its own, electrons under management (@sec:eum): the power drawn by machines we built, in service, measured on the same axis as the Kardashev scale. Today it is zero.

== What changed <sec:why>

Five years ago this plan would have needed a factory before it needed a customer. Four things changed.

#figure(
  {
    let panel(title, body) = block(width: 100%, stack(spacing: 0.5em,
      text(font: mono-sb, size: s-xs, fill: c-text, upper(title)),
      body))
    grid(columns: (1fr,) * 4, column-gutter: 1.2em,
      // 1. actuator in a catalog
      panel([Actuators, from a catalog], cetz.canvas(length: 1cm, {
        import cetz.draw: *
        let c = (1.9, 1.6)
        circle(c, radius: 1.35, fill: c-surface, stroke: 0.7pt + c-line-hi)
        circle(c, radius: 1.05, fill: none, stroke: (paint: c-label, thickness: 0.5pt, dash: "dotted"))
        circle(c, radius: 0.72, fill: c-raised, stroke: 0.5pt + c-line-hi)
        for k in range(3) {
          let a = k * 120deg + 90deg
          circle((c.at(0) + 0.42 * calc.cos(a), c.at(1) + 0.42 * calc.sin(a)), radius: 0.22, fill: c-surface, stroke: 0.5pt + c-label)
        }
        circle(c, radius: 0.16, fill: c-sel, stroke: none)
        content((c.at(0) + 1.45, c.at(1) + 1.05), anchor: "west", text(size: 5pt, fill: c-label)[motor])
        content((c.at(0) + 1.45, c.at(1) + 0.55), anchor: "west", text(size: 5pt, fill: c-label)[planetary])
        content((c.at(0) + 1.45, c.at(1) + 0.05), anchor: "west", text(size: 5pt, fill: c-label)[encoder])
        content((c.at(0) + 1.45, c.at(1) - 0.45), anchor: "west", text(size: 5pt, fill: c-label)[driver])
        content((c.at(0), -0.05), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-text)[ONE HOUSING · CAN · 48 V])
      })),
      // 2. flat sheet on a desk
      panel([Plate, cut on a desk], cetz.canvas(length: 1cm, {
        import cetz.draw: *
        rect((0, 0.25), (3.2, 3.45), fill: c-surface, stroke: 0.7pt + c-line-hi)
        let parts = ((0.15, 0.4, 1.45, 1.05), (0.15, 1.2, 1.45, 1.85), (0.15, 2.0, 0.95, 3.3), (1.1, 2.0, 1.9, 3.3), (1.6, 0.4, 3.05, 1.0), (1.6, 1.15, 3.05, 1.85), (2.05, 2.0, 3.05, 3.3))
        for (x0, y0, x1, y1) in parts {
          rect((x0, y0), (x1, y1), fill: c-raised, stroke: 0.4pt + c-label, radius: 0.08)
          circle(((x0 + x1) / 2, (y0 + y1) / 2), radius: 0.07, fill: c-bg, stroke: none)
        }
        content((1.6, 0.1), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-text)[15 × 15 IN · 1/8" 5052])
      })),
      // 3. gradient through contact
      panel([Gradients through contact], cetz.canvas(length: 1cm, {
        import cetz.draw: *
        // loss curve: start at +20% error, descend to 0
        line((0.2, 0.4), (3.3, 0.4), stroke: 0.6pt + c-line-hi)
        line((0.2, 0.4), (0.2, 3.3), stroke: 0.6pt + c-line-hi)
        let pts = ()
        for i in range(0, 31) {
          let t = i / 30
          pts.push((0.25 + t * 3.0, 0.45 + 2.7 * calc.exp(-5.5 * t)))
        }
        line(..pts, stroke: 1.2pt + c-sel)
        circle(pts.at(0), radius: 0.07, fill: c-text, stroke: none)
        content((pts.at(0).at(0) + 0.12, pts.at(0).at(1)), anchor: "west", text(size: 5pt, fill: c-label)[±20% mass error])
        content((3.3, 0.55), anchor: "south-east", text(size: 5pt, fill: c-label)[machine precision])
        content((1.75, 0.25), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-text)[STEPS OF ∂J/∂θ · SIM])
      })),
      // 4. a humanoid at the price of a used car
      panel([A humanoid, for a car's price], cetz.canvas(length: 1cm, {
        import cetz.draw: *
        let x = 1.0
        circle((x, 3.0), radius: 0.22, fill: c-surface, stroke: 0.7pt + c-text)
        rect((x - 0.33, 1.75), (x + 0.33, 2.7), fill: c-surface, stroke: 0.7pt + c-text, radius: 0.06)
        line((x - 0.33, 2.6), (x - 0.62, 1.85), stroke: 1.4pt + c-text)
        line((x + 0.33, 2.6), (x + 0.62, 1.85), stroke: 1.4pt + c-text)
        line((x - 0.16, 1.75), (x - 0.2, 0.45), stroke: 1.6pt + c-text)
        line((x + 0.16, 1.75), (x + 0.2, 0.45), stroke: 1.6pt + c-text)
        // price tag
        line((x + 0.62, 1.85), (2.0, 1.85), stroke: 0.5pt + c-label)
        rect((2.0, 1.55), (3.35, 2.15), fill: c-sel, stroke: none, radius: 0.05)
        content((2.675, 1.85), text(font: mono-sb, size: 6pt, fill: c-sel-ink)[\$16,000])
        content((2.675, 1.35), text(size: 5pt, fill: c-label)[Unitree G1, 2024])
        content((1.75, 0.25), anchor: "north", text(font: mono-m, size: 5.5pt, fill: c-text)[BUY · BREAK · INSTRUMENT])
      })),
    )
  },
  caption: [#lead[Four things changed.] Integrated actuators are sold from a catalog @robstride2026manual; an arm's plates nest on a few sheets cut on a desk @rose2026forkv9; gradients pass through contact in our physics engine, so far in simulation @ipse2026brief; and a humanoid costs about as much as a used car @unitree2024g1. Schematic, not to scale.],
) <fig:inflection>

*Integrated actuators became a catalog item.* Quasi-direct-drive actuators, with motor, planetary reduction, encoder and driver in one housing, are now sold off the shelf by several Chinese vendors. RobStride's range includes the RS00, RS02, RS04 and RS06, each with a published manual, outline drawing and CAN interface @robstride2026manual. Our arm uses six of them. Retail prices for the family run roughly \$70–170 for the small units and more for the large ones (our estimate from retail listings, not a quote). The design idea does not depend on RobStride, but the v9 drawings do: motor mounts, bolt patterns and screw lengths are taken from RobStride's manual drawings @rose2026forkv9, so another vendor's motor means new drawings. What matters is that a 120 N·m peak joint (RS04 @robstride2026manual) can be bought with its driver attached.

*Flat-sheet fabrication fits on a desk.* Every structural plate of the v9 arm except the 1/4" foot is cut from 1/8" (3.175 mm) 5052 aluminum. The parts nest on three 15 × 15 inch sheets and one 12 × 12 inch sheet, and they are laid out for a 390 mm desktop router bed @rose2026forkv9. The same files go to SendCutSend or to a local supplier without change. A machine that can be built from flat sheet, standard extrusion and purchased actuators can, in principle, be built by another such machine. That is the premise of Pair.

*We can now take gradients through contact.* Our physics engine, phyz, computes exact gradients of a simulated motion with respect to physical parameters such as mass and inertia. That includes contact, the hard case and the one that matters for anything with feet or fingers: its contact adjoint differentiates the same physics the forward pass runs @ipse2026brief. In simulation, on synthetic motion, it has recovered five of a K1 humanoid's inertial parameters from ±20% error to machine precision (sphere feet; @sec:schema). Domain randomization @tobin2017domain, the usual alternative, can make a policy robust to a gap. It cannot take the gap's gradient.

*Humanoids reached the price of a used car.* Unitree announced its G1 at \$16,000 in 2024 @unitree2024g1. We bought a Booster K1 (95 cm, 19.5 kg, 22 degrees of freedom, force-controlled joints) as the reference body for our mind research @booster2025k1. A humanoid that a small company can buy, break, and instrument changes who can do the research.

The result is that one person can now write every layer between a parameter in a source file and a measured joint torque. Ten years ago that took a company the size of a robot OEM. The specification is written to be carried by more hands than one, and everything in it that can be open is open.

= The problem <sec:problem>

== Made by hand

The *idiot index* of a part is its cost divided by the cost of its raw materials. A high index means most of the price is process, not matter. Our prototype actuator, rana-100, shows the problem (@fig:rana; line items in @tab:rana).


#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let total = 1086.47
    let W = 17.6
    let segs = (
      ("Housing, 6061", 231.30, true), ("Magnetic steel, 1018", 313.30, true), ("Gear blanks", 172.51, true),
      ("COTS", 103.36, false), ("ODrive S1", 149.00, false), ("Consumables", 45.00, false), ("Shipping", 72.00, false),
    )
    let x = 0
    for (name, v, mfg) in segs {
      let w = v / total * W
      rect((x, 0), (x + w, 0.8), fill: if mfg { c-raised } else { none }, stroke: 0.6pt + c-line-hi)
      if w > 1.3 {
        content((x + 0.12, 0.66), anchor: "north-west", text(size: s-xs, fill: c-text, name))
        content((x + 0.12, 0.3), anchor: "north-west", text(font: mono-m, size: s-xs, fill: c-label)[\$#str(calc.round(v))])
      } else {
        let cx = x + w / 2
        let ly = if name == "Consumables" { -0.45 } else { -0.85 }
        line((cx, 0), (cx, ly + 0.08), stroke: 0.5pt + c-line-hi)
        content((cx - 0.1, ly), anchor: "east", text(font: mono-m, size: s-xs, fill: c-label)[#upper(name) \$#str(calc.round(v))])
      }
      x += w
    }
    // manufactured bracket: the flame
    let m = 717.11 / total * W
    line((0, 1.0), (0, 1.15), (m, 1.15), (m, 1.0), stroke: 1pt + c-sel)
    content((m / 2, 1.25), anchor: "south", box(fill: c-sel, inset: (x: 3pt, y: 2pt), text(font: mono-sb, size: s-sm, fill: c-sel-ink)[MACHINED \$717 · 66%]))
    content((W, 1.25), anchor: "south-east", text(font: mono-m, size: 6.5pt, fill: c-label)[TOTAL \$1,086 (EST.)])
    // raw-material comparison, same scale
    let r0 = 60 / total * W
    let r1 = 90 / total * W
    rect((0, -0.75), (r1, -0.35), fill: none, stroke: 0.6pt + c-warn)
    rect((0, -0.75), (r0, -0.35), fill: c-warn, stroke: none)
    content((r1 + 0.2, -0.55), anchor: "west", text(font: mono-m, size: s-xs, fill: c-warn)[\$60–90 RAW METAL IN THE MACHINED PARTS · ESTIMATE])
  }),
  caption: [#lead[rana-100 prototype bill of materials, drawn to scale] @rana2026bom. All prices are estimates, not binding quotes. The bracketed span is outsourced machining; the amber bar below is our estimate of the metal inside it, on the same axis, and is not part of the BOM.],
) <fig:rana>

About two thirds of the cost is machining that someone else does, on metal worth an estimated \$60–90. The idiot index of the manufactured parts is roughly 8–12 (estimate). At volume the index falls, but it falls by moving labor into tooling and supply chains. That is what a robot OEM does, and it is why a humanoid still costs as much as a used car.

The same structure repeats one level up. A limb's materials are about \$2,000, of which about \$1,200 is the six purchased actuators @mr2026masterplan. The plates are cheap. The actuators are someone else's margin. Owning the actuator is Limb's second step (@sec:arm), and we take that step only where rana wins on a measured metric.

== The unmeasured gap

Every robotics lab has a sim-to-real gap. We know of few that publish it as a number with an error bar, unit by unit. One established response is domain randomization: vary the simulator until a policy tolerates the difference @tobin2017domain. That makes the policy robust to the gap, but it does not by itself measure it.

We treat the gap as the central measurement. It decides whether a design is ready to build, whether a controller is ready to deploy, and whether an actuator is ready to replace a purchased one. It also tells us how much of the loop can run without a person in it. A gap that is not measured cannot be closed, and a loop with an unmeasured gap needs a human in it.

This is where the two halves of the problem meet. The machining in @fig:rana is the part of the cost that a machine making its own parts goes after. The judgment around it is what keeps a person in the loop: whether a part is right, whether a unit is ready, and what to change when it is not. A gap measured on every unit gives a machine something to check against. A gap that can be differentiated says what to change. A gap that is never measured leaves each of those decisions with a person, and the cost of that person stays in every unit.

== What we do not claim

This paper does not claim that any machine we have built makes copies of itself. None does. It does not claim that the stages after Limb will happen on any schedule. It does not claim emergence, general intelligence, or consciousness for any system. The mind research described in @sec:mind will preregister a narrow claim about the robot's body schema and treats everything beyond it as an open question.

Some words come from the ipse design brief. They name parts of a research architecture, not established properties. "Mind" is the project's word for its learning system. "Refuse" and "reluctance" name behavior the architecture is designed to allow and to log, not a separate module. "Drives" are predictions about preferred internal states. "Self-Emergence" states the hypothesis the self-probe will test. None of these words is a claim about experience.

It does not claim our actuator is better than a purchased one. Limb's second step exists to find out on an actuator dynamometer (the dyno), which is planned and not yet built, and dropping our actuator is a valid outcome.

It does not claim that any arm exists, or that the K1 can pick up litter. No arm has been built and the K1 has not grasped litter.

It does not claim the municipal pilot exists. Lakewood is a reference case, chosen because we know it well @muni2025whitepaper. No agreement is in place.

= Overview <sec:glance>

We are building machines that build machines, and measuring every one. The unit is a limb: one arm, shipped with its sim-to-real gap, the difference between what our physics engine predicted and what the built arm did. Two limbs on any table make a pair, and a pair can do a day's work or build the next limb. A humanoid is a pair on legs. One part, measured everywhere, from a laundry table to a city.

The plan climbs in eight stages, each a step up in scale and each built by the one before it (@fig:glance). Only the first, Limb, is a plan. The rest are goals.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let tiles = (
      ("Limb", "images-dark/fork-v9-lines.png", "an arm that ships with its gap"),
      ("Pair", "pair", "two limbs at any table; a pair builds a pair"),
      ("Body", "images-native/humanoid_bay_iso.png", "a pair on legs that fits its own model"),
      ("City", "city", "a fleet and its pairs work a city"),
      ("Planet", "planet", "lineages on their own power"),
      ("Moon", "images-native/seed_iso.png", "a seed grows a pair, with a lifeline"),
      ("Mars", "mars", "the same seed, lifeline cut"),
      ("Sun", none, "a swarm of collectors"),
    )
    let tw = 1.95
    let gap = 0.27
    for (i, t) in tiles.enumerate() {
      let (name, img, what) = t
      let x = i * (tw + gap)
      rect((x, 0), (x + tw, tw), fill: c-surface, stroke: if i == 0 { 1.2pt + c-sel } else { 0.6pt + c-line-hi })
      if img == "city" {
        let hs = (0.9, 1.4, 0.7, 1.15, 0.55, 1.0)
        for (k, hh) in hs.enumerate() { rect((x + 0.25 + k * 0.29, 0.4), (x + 0.48 + k * 0.29, 0.4 + hh), fill: c-raised, stroke: 0.5pt + c-label) }
        line((x + 0.15, 0.4), (x + tw - 0.15, 0.4), stroke: 0.7pt + c-label)
      } else if img == "pair" {
        line((x + 0.2, 0.55), (x + tw - 0.2, 0.55), stroke: 1.2pt + c-label)
        for (sx, dir) in ((x + 0.55, 1), (x + tw - 0.55, -1)) {
          rect((sx - 0.17, 0.55), (sx + 0.17, 0.66), fill: c-text, stroke: none)
          line((sx, 0.66), (sx, 1.25), (sx + dir * 0.45, 1.55), (sx + dir * 0.62, 1.2), stroke: 1.6pt + c-text)
        }
        rect((x + tw / 2 - 0.18, 0.55), (x + tw / 2 + 0.18, 0.78), fill: c-raised, stroke: 0.5pt + c-label)
      } else if img == "mars" {
        circle((x + tw / 2, tw / 2), radius: 0.62, fill: c-raised, stroke: 0.7pt + c-label)
        rect((x + tw / 2 - 0.12, tw / 2 + 0.25), (x + tw / 2 + 0.12, tw / 2 + 0.49), fill: c-text, stroke: none)
        line((x + tw / 2 - 0.62, tw / 2 + 0.1), (x + tw / 2 + 0.62, tw / 2 + 0.1), stroke: (paint: c-label, thickness: 0.4pt, dash: "dotted"))
      } else if img == "planet" {
        circle((x + tw / 2, tw / 2), radius: 0.78, fill: c-raised, stroke: 0.7pt + c-label)
        for k in range(-2, 3) { line((x + tw / 2 - calc.sqrt(0.78 * 0.78 - calc.pow(k * 0.28, 2)), tw / 2 + k * 0.28), (x + tw / 2 + calc.sqrt(0.78 * 0.78 - calc.pow(k * 0.28, 2)), tw / 2 + k * 0.28), stroke: 0.4pt + c-label) }
        for k in range(8) { let a = k * 45deg; circle((x + tw / 2 + 0.6 * calc.cos(a), tw / 2 + 0.6 * calc.sin(a)), radius: 0.05, fill: c-text, stroke: none) }
      } else if img != none {
        content((x + tw / 2, tw / 2), image(img, width: (tw - 0.3) * 1cm, height: (tw - 0.3) * 1cm, fit: "contain"))
      } else {
        circle((x + tw / 2, tw / 2), radius: 0.5, fill: c-sel, stroke: none)
        for k in range(16) {
          let a2 = k * 22.5deg
          rect((x + tw / 2 + 0.85 * calc.cos(a2) - 0.06, tw / 2 + 0.85 * calc.sin(a2) - 0.06), (x + tw / 2 + 0.85 * calc.cos(a2) + 0.06, tw / 2 + 0.85 * calc.sin(a2) + 0.06), fill: c-text, stroke: none)
        }
        for k in range(0) {
          let a = k * 30deg
          line((x + tw / 2 + 0.95 * calc.cos(a), tw / 2 + 0.95 * calc.sin(a)), (x + tw / 2 + 1.18 * calc.cos(a), tw / 2 + 1.18 * calc.sin(a)), stroke: 1pt + c-sel)
        }
      }
      content((x, -0.15), anchor: "north-west", text(font: sans-m, size: s-sm, fill: c-text, name))
      content((x, -0.55), anchor: "north-west", box(width: tw * 1cm, text(size: 5.5pt, fill: c-label, what)))
      if i < tiles.len() - 1 { line((x + tw + 0.03, tw / 2), (x + tw + gap - 0.03, tw / 2), stroke: 0.7pt + c-label, mark: (end: "straight", size: 0.12)) }
    }
    // power axis, log10 W from 0 to 27
    let W = 8 * tw + 7 * gap
    let y0 = -3.2
    let X(e) = e / 27 * W
    line((0, y0), (W, y0), stroke: 0.7pt + c-line-hi)
    for e in range(0, 28, step: 3) {
      line((X(e), y0), (X(e), y0 - 0.12), stroke: 0.5pt + c-line-hi)
      content((X(e), y0 - 0.2), anchor: "north", text(font: mono, size: 5.5pt, fill: c-label)[10#super[#str(e)]])
    }
    content((W, y0 - 0.55), anchor: "north-east", text(font: mono, size: s-xs, fill: c-label)[WATTS, LOG SCALE])
    // our stages as bands (orders of magnitude, not targets)
    let bands = (("Limb", 1.7, 4), ("Pair", 4, 5), ("Body", 5, 6), ("City", 6, 7), ("Planet", 7, 9), ("Moon", 2, 3), ("Mars", 2, 3), ("Sun", 9, 26.6))
    for (i, (ch, a, b)) in bands.enumerate() {
      let lo = y0 + 0.12 + i * 0.16
      rect((X(a), lo), (X(b), lo + 0.14), fill: if ch == "Limb" { c-sel } else { c-raised }, stroke: none)
      content((X(a) - 0.06, lo + 0.07), anchor: "east", text(font: mono-m, size: 5pt, fill: c-text, upper(ch)))
    }
    // humanity and the scale
    let marks = (
      (13.3, "humanity today, K 0.73", "east", 1.9),
      (16, "Type I", "west", 1.9),
      (26, "Type II", "west", 1.9),
    )
    for (e, l, side, h) in marks {
      line((X(e), y0 - 0.0), (X(e), y0 + h), stroke: (paint: c-label, thickness: 0.6pt, dash: "dashed"))
      let dx = if side == "east" { -0.06 } else { 0.06 }
      content((X(e) + dx, y0 + h - 0.05), anchor: "north-" + side, text(font: mono-m, size: 5.5pt, fill: c-label, upper(l)))
    }
    content((0, y0 + 1.9), anchor: "north-west", text(font: mono-m, size: 5.5pt, fill: c-label)[EUM TODAY: 0 W])
  }),
  caption: [#lead[The plan, from arm to sun.] Top: the eight stages, each a step up in scale and each built by the one before it. Bottom: electrons under management (@sec:eum) by stage, as orders of magnitude, not targets, against humanity's power use and the Kardashev scale @kardashev1964 @sagan1973cosmic @ei2024review. Only Limb is a plan.],
) <fig:glance>

== Flywheel

Each stage pays for the next and teaches it something it could not learn otherwise (@tab:flywheel).

#figure(
  table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Stage*], [*Delivers*], [*Pays forward*], [*Gate, in short*]),
    [Limb], [Arms, sold one at a time], [Revenue; the first measured gaps; duty cycles that size rana], [≥ 10 at ≥ 40% margin; ≤ 2 mm RMS each],
    [Pair], [Two limbs that work at any table, and build limbs], [Useful work; a falling bought fraction; a doubling time], [A task at 95%; a pair builds a pair],
    [Body], [A pair on legs that fits its own model], [Mobility; a gradient from the gap to the design], [Schema fit beats persistence by 20%],
    [City], [A city's fleet and its twin], [Contracts; real duty at scale; a world to train in], [A city renews for ≥ 12 months],
    [Planet], [Lineages on their own power], [A measured doubling time; collectors], [≥ 1 year on collectors it made],
    [Moon], [A seed grows a pair, with a lifeline], [Local material; launch-mass lessons], [≥ 50% local by mass],
    [Mars], [The same seed, lifeline cut], [Autonomy with no one to ask], [0 human commands to a receipt],
    [Sun], [A swarm of collectors], [—], [Power and doubling time],
  ),
  caption: [#lead[The flywheel.] Each stage funds and builds the next. The full gates are in @tab:arc. *Proposed targets, to be confirmed by the author.*],
) <tab:flywheel>

== Three numbers

Three numbers track the whole plan (@fig:dash). Today none of them is good, and we say so.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let cards = (
      ("The gap", "—", "mm RMS, per unit", "No limb built yet. First reading: the first limb, against its model."),
      ("Electrons under management", "0 W", "mean, in service", "Nothing we built is in service. The K1 is counted beside it, not in it."),
      ("Doubling time", "—", "per lineage", "No pair has built a pair. First reading: Pair, over three generations."),
    )
    let w = 5.4
    let g = 0.35
    for (i, (name, val, unit, note)) in cards.enumerate() {
      let x = i * (w + g)
      rect((x, 0), (x + w, 3.1), fill: c-surface, stroke: 0.6pt + c-line-hi)
      content((x + 0.3, 2.85), anchor: "north-west", text(font: mono-m, size: s-xs, fill: c-label, upper(name)))
      content((x + 0.3, 2.25), anchor: "north-west", text(font: mono-m, size: 22pt, fill: c-text, val))
      content((x + 0.3, 1.2), anchor: "north-west", text(font: mono, size: s-xs, fill: c-label, upper(unit)))
      content((x + 0.3, 0.8), anchor: "north-west", box(width: (w - 0.6) * 1cm, text(size: s-xs, fill: c-label, note)))
    }
  }),
  caption: [#lead[The dashboard, October 2026.] The gap per unit (@fig:loop), electrons under management (@sec:eum) and doubling time (@sec:replication). We will publish each one when it first has a value, and every time it changes.],
) <fig:dash>

== Loop

Every number comes from one loop (@fig:loop): design in source code, simulate in a physics engine that can be differentiated, build, measure, and feed the gap back into the design. Each stage removes a person from one more edge of it (@tab:loopstate).

#figure(
  diagram(
    node-stroke: n-stroke,
    edge-corner-radius: none,
    edge-stroke: 0.7pt + c-label,
    spacing: (30mm, 9mm),
    node((0, 0), [*Design*\ #text(fill: c-label, size: s-sm)[loon source → vcad]], shape: sqnode, fill: n-fill, width: 38mm, name: <d>),
    node((2, 0), [*Simulate*\ #text(fill: c-label, size: s-sm)[phyz, wodl world]], shape: sqnode, fill: n-fill, width: 38mm, name: <s>),
    node((2, 2), [*Build*\ #text(fill: c-label, size: s-sm)[kerf, a pair]], shape: sqnode, fill: n-fill, width: 38mm, name: <b>),
    node((0, 2), [*Measure*\ #text(fill: c-label, size: s-sm)[dyno, cameras, K1]], shape: sqnode, fill: n-fill, width: 38mm, name: <m>),
    node((1, 1), [*Gap*\ measured − predicted\ #text(fill: c-label, size: s-sm)[published on every unit]], shape: sqnode, fill: c-surface, stroke: 1.2pt + c-sel, width: 40mm, name: <g>),
    edge(<d>, <s>, "-|>", text(size: s-sm, fill: c-label)[parameters θ]),
    edge(<s>, <b>, "-|>", text(size: s-sm, fill: c-label)[prediction], label-side: left),
    edge(<b>, <m>, "-|>", text(size: s-sm, fill: c-label)[built unit]),
    edge(<m>, (0, 1), <g>, "-|>"),
    edge(<g>, (1, 0.5), (0, 0.5), <d>, "-|>", text(size: s-sm, fill: c-text)[gradient ∂J/∂θ], label-pos: 0.5, label-side: right),
  ),
  placement: bottom,
  caption: [#lead[The loop, read clockwise.] The small labels name the tool at each step. The software projects are listed in @tab:stack, and the dyno is a planned actuator test stand that does not exist yet. Each later stage of the plan removes a human from one more edge. In Arm the gradient edge is a person reading the gap and redesigning by hand.],
) <fig:loop>

== Now

Limb is the current stage (@sec:arm). The next three results, each published whatever it shows:

+ The first built limb's reach, hold and sag, against its model (@tab:arm).
+ rana's first dyno run, against its predicted 36 N·m peak.
+ The preregistration of the K1 body-schema fit, before the fit runs.

== Asks

Early buyers for the limb and the pair who will share measured data. A municipal partner for a one-city, one-task litter pilot. Collaborators who want to check the numbers or build the layers. Write to #link("mailto:info@muni.works")[info\@muni.works].

== Falsifiers

- *Limbs do not sell at margin.* Fewer than ten limbs at 40% gross margin means Limb cannot fund the rest.
- *The gap does not close.* If built limbs cannot hold 2 mm RMS against their own model, the loop has nothing to learn from.
- *A pair cannot build a limb.* If a pair-built limb needs more than 30 minutes of human touch or more than a fifth of a pair's cost bought, Pair is out of reach.
- *Gradients do not survive real contact.* If the body-schema fit cannot beat persistence on hardware, Body's method fails.

Each is a gate in @tab:arc, and we will publish the ones we miss.

#part("II", "Foundations")[The rules, the stack and the standards every stage shares.]

= Principles <sec:principles>

Each is argued where it is used. They are collected here so that they can be quoted and checked.

+ *Gates, not calendars.* A stage opens on a measured gate: a number, a unit, a method and a pass condition (@tab:arc).
+ *Every unit ships with its gap.* Every limb ships with its sim-to-real gap stated, and every claim ships with the number that supports it.
+ *One part, everywhere.* A limb is a product, half of a pair and an arm of a body. Homogeneity is a design rule, not an accident.
+ *Buy the commodity; own the layer where we win.* The actuator is a commodity. Our edge is the geometry, the measured model and the controller around it (@sec:arm).
+ *The guarantee should live in hardware.* Force limits should come from physics, current limits and compliance, measured on the robot. Today they do not (@sec:safety).
+ *Never be Flock Safety.* No face recognition, no plate reading, no tracking of people, no selling or sharing data that tracks anyone, no police access to people-tracking data (@sec:city).
+ *Everything that can be open is open.* Software, designs, firmware where we control it, and data formats. The K1's Booster firmware is not ours and is not open.
+ *No division without a license.* A pair copies itself only with a signed, per-copy license from people, under a published ceiling on doubling time (@sec:replication).

= The stack <sec:stack>

== One language

The stack is a tree rooted at one language, loon (@tab:stack). Build order runs down the tree, and the gradient runs back up it @pond2026strategy.

#figure(
  table(
    columns: 4,
    align: (left + top, left + top, left + top, left + top),
    table.header([*Layer*], [*Project*], [*What it is*], [*Status*]),
    [Roots], [loon], [Source language for parts, worlds, hardware], [exists],
    [Roots], [tang], [Math, autodiff, the one GPU device], [exists],
    [Design], [vcad], [BRep CAD kernel, CAM, FEA, parameter gradients], [exists],
    [World], [wodl], [World description: bodies, materials, joints, sensors, provenance], [new],
    [World], [kosm], [Runtime that runs wodl worlds], [exists, re-placed],
    [Engines], [phyz], [Differentiable physics with a contact adjoint], [exists],
    [Engines], [iris], [Renderer and robot cameras (from kosm-render)], [new],
    [Make], [kerf], [Design to manufactured part, with a receipt], [exists],
    [Hardware], [rana · torc], [Axial-flux actuators, motor control], [prototype],
    [Hardware], [arm], [Manipulator module: product, humanoid limb], [designed (v9)],
    [Hardware], [rose], [Humanoid line: mini, pro, max], [in progress],
    [Hardware], [K1], [Booster K1, 22 DoF: reference body, in hand], [in hand],
    [Hardware], [pack · blade], [One battery and one compute module for every body], [mockup],
    [Hardware], [moth], [Drone: street perception, kino's movable camera], [proposed],
    [Apps], [kino], [Live director: cuts between cameras, moves a drone], [exists],
    [Mind], [ipse], [Developmental mind on a real K1], [in progress],
    [Silicon], [loon-hdl, rune, chip], [HDL, maskless lithography, inference chip], [Pair],
  ),
  caption: [#lead[The stack] @mr2026masterplan. "exists" means software in use today; it does not mean finished.],
) <tab:stack>

Four rules keep the tree a tree: engines never depend on vcad, and vcad never depends on an app; engines read wodl, not each other; there is one GPU device across the stack; and units are millimeters, with Z up, everywhere. The rules are meant to let a gradient pass from a task loss in ipse, through phyz's contact adjoint, into vcad geometry and out to loon parameters without a translation at each hop. Part of that chain runs today, in simulation: on a test model, a phyz rollout gradient passes through a ground-contact model into a vcad geometry parameter and agrees with finite differences. Joining the two ends, from a task loss in ipse to loon source, is co-design work for Body @pond2026strategy.

#figure(
  diagram(
    node-stroke: n-stroke,
    edge-corner-radius: none,
    edge-stroke: 0.7pt + c-label,
    spacing: (14mm, 9mm),
    node((1.5, 0), [*loon*], shape: sqnode, fill: n-fill, name: <loon>),
    node((0.5, 0), [*tang*], shape: sqnode, fill: n-fill, name: <tang>),
    node((0.5, 1), [vcad], shape: sqnode, fill: n-fill, name: <vcad>),
    node((2.5, 1), [loon-hdl], shape: sqnode, fill: n-fill, name: <hdl>),
    node((0, 2), [wodl / kosm], shape: sqnode, fill: n-fill, name: <wodl>),
    node((1, 2), [phyz], shape: sqnode, fill: n-fill, name: <phyz>),
    node((0, 3), [iris], shape: sqnode, fill: n-fill, name: <iris>),
    node((2.5, 2), [rune], shape: sqnode, fill: n-fill, name: <rune>),
    node((1, 3), [kerf], shape: sqnode, fill: n-fill, name: <kerf>),
    node((2.5, 3), [chip], shape: sqnode, fill: n-fill, name: <chip>),
    node((0, 4), [rana · torc], shape: sqnode, fill: n-fill, name: <rana>),
    node((1, 4), [arm · rose · K1], shape: sqnode, fill: n-fill, name: <body>),
    node((2.5, 4), [ipse], shape: sqnode, fill: n-fill, name: <ipse>),
    edge(<loon>, (1.5, 1), <vcad>, "-|>"),
    edge(<tang>, (1, 0), <phyz>, "-|>"),
    edge(<loon>, (2.5, 0), <hdl>, "-|>"),
    edge(<vcad>, (0, 1), <wodl>, "-|>"),
    edge(<wodl>, <phyz>, "-|>"),
    edge(<wodl>, <iris>, "-|>"),
    edge(<hdl>, <rune>, "-|>"),
    edge(<phyz>, <kerf>, "-|>"),
    edge(<rune>, <chip>, "-|>"),
    edge(<kerf>, (1, 3.5), (0, 3.5), <rana>, "-|>"),
    edge(<kerf>, <body>, "-|>"),
    edge(<rana>, <body>, "-|>"),
    edge(<chip>, <ipse>, "-|>"),
    edge(<ipse>, <body>, "<|-|>"),
    edge(<body>, (1.6, 4), (1.6, 2), <phyz>, "--|>", stroke: 1.2pt + c-sel),
    edge(<phyz>, (1.6, 2), (1.6, 1.4), (0.7, 1.4), (0.7, 1), <vcad>, "--|>", stroke: 1.2pt + c-sel),
    edge(<vcad>, (0.5, 0.6), (1.9, 0.6), (1.9, 0), <loon>, "--|>", stroke: 1.2pt + c-sel),
    node((1.95, 3), text(size: s-sm, fill: c-text)[gradient\ (measured\ body → θ)], stroke: none),
  ),
  caption: [#lead[The stack as a tree.] Solid edges are build order, not code dependencies. The dashed edge is the gradient returning from a measured body, through phyz and vcad, to the source parameters.],
) <fig:tree>

== Who is in the loop

What changes from stage to stage is not the loop but who sits in it. @tab:loopstate summarizes the plan.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Stage*], [*Loop state*], [*Human role*]),
    [Limb], [measured gap → human redesign], [designs, builds, measures],
    [Pair], [gap → design → built and measured by a pair], [reviews, licenses each copy],
    [Body], [gap → gradient → new design], [reviews; pairs build],
    [City], [Pair and Body, across a city's fleet and its twin], [the city sets the work],
    [Planet], [many lineages, each closing its own loop on its own power], [licenses each division],
    [Moon], [Pair, far from its supply chain], [remote approval],
    [Mars], [Pair with the supply chain cut], [none between landing and receipt],
    [Sun], [fully closed inside a lineage], [reads the receipts],
  ),
  caption: [#lead[Who is in the loop, by stage] @mr2026masterplan],
) <tab:loopstate>

= Standards <sec:power>

Every machine we build meets the same four standards: a bus, a pack, a blade and a library of boards. They are why a limb, a pair, a humanoid and a drone can share parts, code and repairs. All of it is modeled; none of it is built @mr2026decisions.

== Bus

Every body runs on one unregulated DC bus of *36–60 V*. Every load must accept the whole window, and nothing may assume a regulated 48 V. The window is set by the pack below: 13 lithium cells in series run from 39 V empty to 54.6 V full. It also covers the RobStride actuators and the rack's 48 V supplies, and it stays under 60 V, so the bus remains extra-low voltage.

Beside the power runs one management bus, CAN-FD, for presence, power control and telemetry. Bulk data (cameras, logs, the twin) travels over Ethernet. There is no separate hardwired e-stop loop, and other humanoids do not carry one either. Pulling a pack is the hard stop: its connector's last-mating pin is the power enable.

== Pack

There is one battery: a 13S2P pack of Molicel P50B 21700 cells, *468 Wh* and about *2.65 kg* (estimate) @mr2026battery @molicel2023p50b. The same pack goes in a body, a pair, a charging dock, the rack's UPS and the seed. A humanoid carries two of them (A and B, 936 Wh), one in each flank of the torso, and swaps them itself. Its hand draws pack A sideways onto a passive dock rail while pack B keeps the robot powered, then pushes a fresh pack home. The dock is rails, pads and a charger, with no actuators. A swap takes about a minute and leaves the robot powered, so runtime is set by dock capacity, not by the pack.

#figure(
  table(
    columns: (auto, 1fr, auto),
    align: (left + top, left + top, left + top),
    table.header([*Item*], [*Value*], [*State*]),
    [Battery cell], [Molicel P50B: 5.0 Ah, 60 A continuous, 71 g @molicel2023p50b], [Sourced],
    [Pack], [13S2P, 468 Wh, 46.8 V nominal, 39–54.6 V], [Modeled],
    [Pack mass and size], [\~2.65 kg; \~118 × 116 × 238 mm with connector], [Estimate],
    [Connector], [Anderson SBS 75X: 2 power contacts, 4 mate-last auxiliaries, ±1 mm float], [Sourced part],
    [BMS], [On the body's CAN bus; logs bus current and voltage], [Not designed],
    [Humanoid load], [\~460 W mean walking; \~2.0 kW realistic peak], [Estimate],
    [Humanoid runtime], [\~1.7 h walking, \~2.2 h standing pick work, per A+B pair], [Estimate],
  ),
  caption: [#lead[The pack] @mr2026battery. Every body we build uses it. The runtime figures assume a rose-class body with two arms.],
) <tab:pack>

The pack is also the EUM instrument. EUM (@sec:eum) is defined at each machine's main power input, logged at ≥ 1 Hz, and every pack's BMS measures exactly that. If the BMS logs it, electrons under management are measured for free on every robot and dock we build.

Two cautions. The K1 is a bought robot with its own battery, and it cannot take this pack. The pack and the blade bay below are sized for rose, whose torso must grow from 210 to 214 mm wide to hold both packs beside the bay. And NMC in a public sidewalk dock is a choice a city may question. We keep one chemistry for simplicity and will revisit it if that happens.

== Blade <sec:compute>

Compute comes in one module, the *blade*: a 132 × 76 × 20 mm carrier on a 3 mm aluminum spine that is both its rail and its heat path, with one rear connector carrying the bus, gigabit Ethernet and CAN-FD @mr2026compute. The same blade slides into a 10-inch rack chassis (eight per 2U, fan-cooled) or into a humanoid's torso bay (up to three, conduction-cooled into cold walls with a fan over the fins). A blade identifies itself and its power class over I²C before the host powers it: 20 W for a Raspberry Pi CM5 carrier, 30 W for a Jetson-class carrier.

How a body uses its slots follows the seams ipse already has (@sec:mind). The fast layers, S0 (whole-body tracking, 50 Hz today, 200 Hz target) and S1 (skills and contact planning, 50 Hz), are deadline-bound and must run on the body. The slow layer, S2 (intent and the world model, 1–5 Hz), may run on the body or off it, and if it is lost, S1 finishes the current skill and stands @ipse2026brief. So:

- *Slot 1* always holds a GPU blade running S0, S1, the abstract state bus and perception, isolated at real-time priority. Nothing else is allowed on it. On the K1, vendor processes share the perception computer, and worst-case walk inference has been seen at 13.7–15.8 ms against a 0.24 ms median off the robot (session notes, probably preemption).
- *Slots 2 and 3* run S2 when fitted. A full body thinks offline.
- *With fewer blades*, S2 runs on the rack over the same seam.

Whether a body thinks onboard or leans on the rack is then a slot count, not an architecture. The rack is the same standard at larger scale: a 9U, 450 mm deep frame with a GPU node for training and the twin, two blade chassis, and two 48 V supplies feeding a vertical busbar, about 1 kW in all (estimate) @mr2026compute.

#figure(
  grid(
    columns: 3,
    gutter: 1em,
    align: horizon,
    image("images-native/blade_exploded.png"),
    image("images-native/humanoid_bay_iso.png"),
    image("images-native/rack_12u_iso.png"),
  ),
  caption: [#lead[The blade and its two hosts] (modeled). Left: one blade, exploded. Center: the humanoid bay, lid lifted and one blade drawn. Right: a 12U rack with a GPU node and three blade chassis. Renders by vcad-render @mr2026compute.],
) <fig:blade>

*rose against the K1.* rose should out-think the K1: CUDA, real-time scheduling, and our own code from the carrier up. That claim gets the same treatment as every other one. The same ipse build runs on both bodies, and we report S0 tick jitter, inference latency and where S2 ran, per body. rose is first run with one blade, close to the K1's compute, then with more, so the effect of compute is measured one variable at a time.

The open question is the size of S2. A 16 GB Orin NX blade may hold it; a Thor-class module would not fit a 20 mm blade.

== Boards

Bus, pack and blade need a small set of boards. We own a board first when it defines a standard or makes a measurement, and buy the rest until owning it wins (@tab:boards).

#figure(
  table(
    columns: (auto, 1fr, auto, auto),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Board*], [*Does*], [*Order*], [*State*]),
    [BMS], [Protects the pack; measures EUM; talks CAN], [Own first], [Not designed],
    [Pair hub], [48 V + CAN for two limbs, a camera pair and a blade], [Own first], [Not designed],
    [Blade carriers], [CM5 (20 W) and Jetson (30 W) on the blade standard], [Next], [Mockup],
    [Backplanes], [8-slot rack chassis; 3-slot humanoid bay; switch + CAN], [Next], [Mockup],
    [Arm controller], [Six actuator CAN buses, 48 V distribution, one free bus for the BMS], [Next], [Not designed],
    [torc], [Motor control for rana], [With rana], [Prototype],
  ),
  caption: [#lead[The board library and the order we own it] @mr2026decisions. Every design is open. Fabrication and assembly stay bought.],
) <tab:boards>

Every board design is open. Printed-circuit fabrication and assembly stay bought through Body. Boards are, with magnets, bearings and chips, part of the bought fraction that falls last, and a pair at a pick-and-place machine that assembles our own boards is a Pair milestone (@sec:pair).

= Safety <sec:safety>

== Inheritance

Nothing we build can be turned against a person, whether by force, by data or by instruction. These commitments carry forward to any acquirer @mr2026masterplan:

- *Not by force.* No sales for weapons or military use.
- *Not by data.* Never be Flock Safety (@sec:city). Not negotiable.
- *Not by instruction.* ipse keeps its standing constraints (@sec:mind).
- *No unearned claims.* Every claim about a product ships with the number behind it, labeled measured, modeled or estimated.
- *No careless displacement.* Start with work nobody is doing, and say plainly what changes when the robots arrive.

Together they are a Right Livelihood rule: earn a living without causing harm. They are commitments today; the layers that enforce them in hardware are mostly not built yet (@tab:safety).

== Layers

Our design target is physical: a robot we design should be unable to exceed the body-region contact limits of ISO/TS 15066 @iso15066 even with its software replaced, so that making it dangerous would mean building a different robot. No product in this paper meets that target yet. The limb's three RS04 joints peak at 120 N·m and the gripper can squeeze 375 N per finger at peak, so today its force limit is an actuator current-limit setting, not physics, and it has to be measured. The K1 is a purchased robot whose actuators and firmware we did not design, so the physics layer does not cover it; its street safety rests on the rules in @sec:city.

The collaborative-robot safety envelope and the anti-weaponization goal share most of their physics, but not all of it. Force limits bound what the robot's own body can do on contact. They do not bound what a held object can do: a gripper rated to squeeze 112.5 N and hold a 500 ml bottle can also hold a blade. That gap is why the other layers in @tab:safety exist, and why we do not call the physics layer a complete guarantee.

ISO/TS 15066 was written for industrial cobots. For a humanoid on a public sidewalk the closer standard is ISO 13482:2014 @iso13482, for personal care robots, and we have not yet assessed against it. Contact force will be measured on the robot itself, not inferred from the actuator dyno, and published as receipts.

#figure(
  table(
    columns: (auto, 1fr, 1fr, auto),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Layer*], [*How*], [*On the limb today*], [*Would survive*]),
    [Physics], [Actuators, mass and speed sized so contact force stays under ISO/TS 15066], [#status("fault", "No") RS04 peaks at 120 N·m], [Everything],
    [Mechanism], [Slip clutches and compliant joints that yield above a set torque], [#status("label", "Planned") none in v9], [Reflashing],
    [Electronics], [Current limits set in torc hardware, not firmware], [#status("label", "Planned") v9 uses RobStride firmware current limits], [Reflashing],
    [Interfaces], [A gripper by design; no trigger lines at the tool], [#status("info", "Designed") only the EL05's CAN + 48 V and a camera pair], [Casual modification],
    [Mind], [ipse can refuse; refusals are logged, never trained against], [#status("label", "Planned") ipse is K1 research, not on the arm], [Misuse by instruction],
    [Terms], [Sales terms ban weapons use; fleet buyers vetted], [#status("label", "Planned") no terms written], [Resale, mostly],
  ),
  caption: [#lead[Safety layers, what exists on the limb today, and what each layer would survive] @mr2026masterplan. Today the limb has only a design-level interface layer and vendor firmware current limits; it is not physically incapable of harm.],
) <tab:safety>

The table is ordered by strength. A guarantee in the physics layer survives anything short of building a different robot. A guarantee in the terms layer is a promise. We do not claim the lower layers are as strong as the upper ones.

The current arm already shows why the physics layer has to be designed rather than assumed. The RS04’s 120 N·m peak is much more than a 1.27 kg payload needs. So the force limit has to come from current limits, compliance and speed, and it has to be measured. The crush points in the arm audit are an open item for the same reason @rose2026forkv9.

Software, designs and data formats stay open. Open licenses cannot restrict use, so the terms layer is weak for open designs, and that is accepted. The guarantee has to live in hardware, and until clutches and hardware current limits are built and measured, it does not.

= Replication <sec:replication>

A machine that builds the next machine invites one fear above others: replication nobody can stop. The image has a name, gray goo, from Drexler's 1986 warning about self-replicating nanomachines @drexler1986engines. Our pairs are not nanomachines. They are two arms at a table, with a mill and a router nearby, and they need stock, power and repair, and that is the first defense. It is not enough by itself, because a lineage that doubles on its own power, on a planet with no one on it, is exactly what Sun asks for. So the limits are designed in from the first pair. None of them is built yet.

- *Licensed division.* A pair builds a copy only with a signed license for that one copy. Licenses come from keys held by people, not by the pairs. A pair cannot mint its own, and a copy built without one does not pass its acceptance receipts.
- *A germ line no pair can edit.* The design and its receipts live where the machines that do the work cannot change them. Every copy is checked against the germ line; a copy that drifts is decommissioned and stripped for stock.
- *A vitamin made of information.* Every pair depends on one input it cannot make: signed instructions. That keeps the bought fraction able to reach zero in matter while dependence on people remains.
- *A ceiling on doubling.* Each lineage has a published maximum rate of division. Faster growth needs a new license from people, never a decision by the pairs.
- *Stop is tested, not assumed.* Every generation must pass a commanded shutdown drill before its first license (@tab:arc, Pair).
- *No foraging on Earth.* On Earth, pairs take only stock that is delivered to them. They never gather raw material from the environment. Taking local material is for the Moon, Mars and Mercury, under planetary protection rules.

The arc ends where people are least able to watch: on Mars, with no commands between landing and receipt, and around the sun. That is why the licenses, the germ line and the drills have to work on Earth first, measured like everything else.

= The score <sec:score>

== Electrons under management <sec:eum>

One number runs through every stage: *electrons under management* (EUM), in watts. It measures the same thing at every scale, so progress can be read off one axis, from one robot to a fraction of the sun's output (3.828 × 10#super[26] W, nominal @iau2015b3).

*Definition.* EUM is the mean electrical input power, over a trailing 30-day window, of every machine in service that we designed and built, measured at each machine's main power input (a DC-bus shunt or a metered AC outlet) and logged at ≥ 1 Hz. "In service" means doing work for someone other than its maker. Machines we did not build but that run our stack, such as the K1 or a bought mill, are reported beside it as *electrons under control*, never added to the headline.

*Baseline, October 2026.* EUM is *0 W*: we have built nothing that is in service. Electrons under control are the K1, at roughly 100 W while it runs (estimate, not measured; its duty is not logged yet). A working pair would add about 150 W mean, with two limbs near 200 W peak each (estimate).

*Doubling time.* The second headline number is the time a lineage of pairs takes to double what it can build and power, measured from receipts over at least three generations. It is undefined today, because no pair has built a pair. It sets how long the arc takes (@sec:sun) and it is capped by policy (@sec:replication).

*Trajectory, as orders of magnitude, not targets.* Limb: 0 W → \~10 kW (a hundred limbs near 50 W mean). Pair: \~100 kW as pairs build pairs. Body: \~1 MW of bodies. City: \~10 MW, a city's fleet and the pairs that keep it. Planet: \~1 GW and up, on collectors pairs made. Moon and Mars: the first kilowatts off Earth. Sun: a measurable fraction of 3.8 × 10#super[26] W.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let stages = (
      ("Sun", [3.8 × 10#super[26] W]),
      ("Mars", [1 kW]),
      ("Moon", [1 kW]),
      ("Planet", [1 GW]),
      ("City", [10 MW]),
      ("Body", [1 MW]),
      ("Pair", [100 kW]),
      ("Limb", [0 W → 10 kW]),
    )
    let w = 17.0
    let h = 7.6
    let n = stages.len()
    for (i, (name, watts)) in stages.enumerate() {
      let inset = i * 0.7
      let top = i * 0.72
      let bot = i * 0.22
      let here = i == n - 1
      rect((inset, bot), (w - inset, h - top), fill: if calc.even(i) { c-surface } else { c-bg }, stroke: if here { 1.2pt + c-sel } else { 0.6pt + c-line-hi })
      content((inset + 0.25, h - top - 0.2), anchor: "north-west", text(font: sans-m, size: s-md, fill: c-text, name))
      content((w - inset - 0.25, h - top - 0.24), anchor: "north-east", text(font: mono-m, size: s-xs, fill: c-label, watts))
    }
  }),
  caption: [#lead[The arc.] Eight stages, each a step up in scale; one loop, one number. Each stage contains the one before it, and each removes a person from one more edge of the loop. Only Limb is a plan. Watts are electrons under management (@sec:eum), as orders of magnitude, not targets @mr2026masterplan.],
) <fig:arc>

== Gates

Every stage closes on gates of the same form (@tab:arc).

#figure(
  table(
    columns: (auto, 2fr, auto),
    align: (left + top, left + top, left + top),
    table.header([*Stage*], [*Gate: number, method, pass condition*], [*EUM, order*]),
    [Limb (yrs 1–3)], [≥ 10 limbs sold to outside buyers at ≥ 40% gross margin, after materials, labor, test and warranty (invoices, trailing 12 months). Tool-point error ≤ 2.0 mm RMS on every limb shipped: 50 fixed poses, unloaded and at 1.0 kg, probe against a calibrated plate, printed on its receipt. rana ≥ 1.2 × RS04 in continuous stall torque per kg or per dollar, on one dyno at 25 °C, or rana is dropped], [0 W → 10 kW],
    [Pair], [A pair at a customer's table completes a named task (for example, folding 50 garments) at ≥ 95% success. A pair assembles a limb that passes the Limb test in ≤ 30 min of human touch time. A pair builds a pair that passes its receipts at ≤ 20% bought cost, with bought share falling over 3 generations and a commanded shutdown drill passed in each], [100 kW],
    [Body (yrs 3–7)], [Preregistered hardware body-schema fit: held-out joint-position error ≤ 0.8 × persistence at 0.1 s and 0.5 s, and fitted link masses within ±5% of weighed parts. rose walks 100 m on level pavement carrying two limbs, 0 falls in 5 of 5. A gradient co-designed part beats its hand-designed predecessor by ≥ 10% on its named objective], [1 MW],
    [City], [The litter pilot renews for ≥ 12 months after a ≥ 3-month trial, with 0 contact incidents. A city renews fleet and twin for ≥ 12 months. ≥ 90% of fleet repairs done by pairs in the city, by receipts], [10 MW],
    [Planet], [A lineage runs ≥ 1 year on collectors it made; doubling time measured over ≥ 3 generations; bought share ≤ 5% of cost], [1 GW],
    [Moon], [A seed builds a pair that passes the Earth receipts, ≥ 50% of its mass from local material, with remote approval only], [1 kW],
    [Mars], [A second pair with 0 human commands between landing and its receipt], [1 kW],
    [Sun], [None we can set now; reported as EUM and doubling time], [3.8 × 10#super[26] W],
  ),
  caption: [#lead[Gates, stage by stage] @mr2026masterplan. *Proposed targets, to be confirmed by the author.* Year ranges are aspirations, not commitments. Only Limb is a plan.],
) <tab:arc>

#part("III", "The stages")[Eight steps up in scale, from one arm to a star. Only the first is a plan.]

= Limb <sec:arm>

*An arm that ships with its gap.* Limb has two steps over roughly three years. Each closes on an exact gate (@tab:arc).

+ *An arm worth buying.* Build it on RobStride actuators. The actuator is a commodity. Our edge is everything around it: geometry checked against real loads, a measured model of every unit, a controller trained against that model, and a stated sim-to-real gap on every limb.
+ *Own the actuator, where it wins.* Limb revenue funds rana and torc. A purchased joint is replaced only where rana wins on a measured metric. Dropping rana is a valid outcome, not a failure.

== Arm

One arm does three jobs: it is sold on its own, it is half of every pair, and it is an arm of the rose humanoid. Its design idea is that *every pitch joint is a fork* @rose2026forkv9. The parent link holds the motor's stator between its two side plates. The child straddles the parent, with one plate bolted to the output and the other riding a plain bearing on a hollow sleeve on the joint axis. Parent and child share one centerline, so no joint is offset and there is no offset elbow to twist the arm. Cables pass through the hollow sleeves, so they only twist and never loop outside the arm. The name comes from where the design lives. rose fork v9 is the ninth revision of the fork concept, drawn in the rose humanoid repository as a table arm that is also rose's arm @rose2026forkv9. In this paper "the arm" means this design, and "rose" on its own means the humanoid line (@tab:stack), whose \$15,000 target assumes rana (@sec:rana).

#colfig(
  image("images-dark/fork-v9-lines.png", width: 60%),
  caption: [#lead[The arm, rose fork v9.] six joints, every plate 1/8" 5052 except the 1/4" foot, every motor mounted by its back face. Render from the arm repository @rose2026forkv9.],
) <fig:arm>

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Parameter*], [*Value*], [*Source / state*]),
    [Degrees of freedom], [6 + gripper], [@rose2026forkv9],
    [Base yaw, shoulder, elbow], [RobStride RS04 × 3], [Peak 120 N·m],
    [Forearm roll, wrist pitch], [RobStride RS02 × 2], [Roll as a cartridge],
    [Tool roll], [RobStride RS00 cartridge], [],
    [Tool flange], [Ø61, 6 × M3 on Ø27], [Matches gripper v2],
    [Foot], [Ø230, 1/4" 5052, 4 × Ø6.6 on Ø190], [M6 to plinth],
    [Plate stock], [1/8" (3.175 mm) 5052], [Three 15 × 15 in sheets of 1/8"; one 1/4" sheet for the foot],
    [Router parts], [20 unique], [DXF + nests generated],
    [Jaw tips from shoulder], [731 mm (786 with gripper v2)], [Modeled],
    [Mass], [8.21 kg (arm-model jaws)], [Modeled],
    [Mass with gripper v2 + harness], [8.44 kg], [Modeled @mr2026bench],
    [Continuous hold, straight out], [1.49 kg (arm-model jaws)], [Modeled; RS04 stall derated (estimate)],
    [Continuous hold, gripper v2], [1.27 kg (shoulder-limited)], [Modeled; RS04 stall derated (estimate)],
    [Sag / twist at flange], [0.13 / 0.11 mm], [Modeled],
    [Elbow / wrist range], [−120 to 115° / ±135°], [],
    [Interference, 209 poses], [None], [Swept in CAD],
    [Build state], [Ready to cut], [None built yet],
  ),
  caption: [#lead[rose fork v9 specification.] Every number is a model output; none has been measured on a built arm. Arm-model jaws are the simpler tool in the arm's own model, two flat jaw plates on the same EL05, and are lighter than gripper v2.],
) <tab:arm>

*Why holding sets the limit.* Holding a pose is a stall, and a stalled actuator is limited by heat, not by peak torque. The vendor's stall-continuous ratings are well below the rotating ratings. For the RS04 at the shoulder, the joint that sets the limit, the manual gives 28.5 N·m stall-continuous against 40 N·m rotating, measured on a 345 × 345 mm aluminum heat-sink plate @robstride2026manual. On a 220 × 200 mm plate, about the size of the fork's parent plates, the manual rates it at 35 N·m rotating. We derate the stall figure by the same 35/40, to 25 N·m (estimate). The hold numbers in @tab:arm use that figure, with the arm straight out, which is the worst pose. Bent working poses hold more. With RobStride's ratings, about 1.25–1.5 kg continuous straight out is close to the limit at this reach without springs, because the shoulder carries the whole arm. The earlier v4/v5 iterations reached the same ceiling @rose2026forkv9.

*Open items* (from the arm repository): RS02 thread depths are assumed at 6 mm and must be measured before screws are ordered. The clock-spring cassettes for the roll joints are drawn but not designed. Crush points found by the audit still need a policy, either soft limits or force limiting. And the motors' unpowered "anti-backdrive" damping is undocumented at the slow speed of a falling arm. It will be tested on the rig @robstride2026manual.

== Hand

The gripper is a parallel-jaw gripper. One RobStride EL05 (1.8 N·m rated, 6 N·m peak, 242 g) turns a pinion of pitch radius $r = 8$ mm between two flat racks, which gives a 0–100 mm opening @mr2026gripper. Squeeze per finger is $T\/(2r)$ for motor torque $T$.

#figure(
  grid(
    columns: 3,
    gutter: 1em,
    align: horizon,
    image("images-native/gripper_iso_open.png"),
    image("images-native/gripper_section.png"),
    image("images-native/gripper_tips.png"),
  ),
  caption: [#lead[Gripper v2] (modeled). Left: open, on the Ø61 tool flange. Center: section through the pinion and racks. Right: the four swappable tip sets (flat serrated, V, soft TPU, scoop) on dowels and a magnet @mr2026gripper.],
) <fig:gripper>

#figure(
  table(
    columns: 4,
    align: (left + top, right + top, right + top, right + top),
    table.header([*Case*], [*Torque*], [*Squeeze / finger*], [*Max lift*]),
    [Rated], [1.8 N·m], [112.5 N], [5.7 kg],
    [Peak (do not hold)], [6 N·m], [375 N], [19.1 kg],
  ),
  caption: [#lead[Gripper force at rated and peak torque.] μ = 0.5, two faces, safety factor 2, rack friction ignored; with an assumed 0.8 rack efficiency, scale by 0.8 (estimate). Max lift is the gripper's alone. The arm holds 1.27 kg straight out (modeled), so the arm, not the gripper, sets payload @mr2026gripper],
) <tab:grip>

@tab:grip invites the wrong conclusion. Rated squeeze would crush an empty drinks can (predicted, not tested). Holding that can needs 0.3 N per finger, or 0.3% of rated torque, and a 500 ml bottle needs 9.6%. For litter, what matters is torque resolution at the bottom of the range, not headroom at the top, so the EL05 runs under current (torque) control. The gripper is 485 g at the flange with flat tips, against a 500 g target. A USB camera in the palm (standard UVC video class) runs its data pair through the arm harness straight to the base computer. A soft TPU boot is available for outdoor use.

== Alternatives

#figure(
  table(
    columns: 6,
    align: (left + top, right + top, right + top, left + top, left + top, left + top),
    table.header([*Arm*], [*Reach*], [*Mass*], [*Rated payload*], [*List*], [*Status*]),
    [This arm], [786 mm], [8.44 kg], [1.27 kg continuous, straight out, with the 485 g gripper (modeled)], [\$5,000 (target)], [Not yet built],
    [Seeed reBot B601-RS], [754 mm], [6.7 kg], [To be cited (vendor page, access date)], [To be cited], [To be cited],
    [UFACTORY Lite 6], [To be cited], [To be cited], [To be cited], [To be cited], [To be cited],
    [UFACTORY xArm 6], [To be cited], [To be cited], [To be cited], [To be cited], [To be cited],
  ),
  caption: [#lead[Arms a buyer can order today.] reBot reach and mass from the arm repository's comparison, which uses the same RobStride family @rose2026forkv9. Empty cells wait for a dated vendor citation.],
) <tab:alts>

Vendors rarely say at what pose or duty cycle a rated payload holds. Our 1.27 kg is a modeled continuous hold at full extension, the worst case; bent working poses hold more. If you need a proven arm now, a commercial one will likely serve you sooner.

== Price

#figure(
  table(
    columns: (auto, auto, auto, 1fr),
    align: (left + top, right + top, right + top, left + top),
    table.header([*Product*], [*List*], [*Materials*], [*Note*]),
    [Limb], [\$5,000], [\~\$2,000], [\~\$1,200 is RobStride],
    [Pair], [\$12,000], [\~\$4,400], [Two limbs, a table plate, a pair hub, a camera pair, a blade],
    [rose], [\$15,000], [not costed], [Humanoid: a pair on legs. Needs rana at a few hundred dollars per joint],
  ),
  caption: [#lead[Target prices, not quotes] @mr2026masterplan. Materials figures are estimates.],
  placement: none, scope: "column",
) <tab:prices>


#colfig(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let k = 6.3 / 15000
    let rows = (("Limb", 5000, 2000, "$5,000", "~$2,000"), ("Pair", 12000, 4400, "$12,000", "~$4,400"))
    for (i, r) in rows.enumerate() {
      let (name, list, mat, ls, ms) = r
      let y = -i * 0.8
      content((-0.15, y + 0.2), anchor: "east", text(size: s-sm, name))
      rect((0, y), (list * k, y + 0.4), fill: c-surface, stroke: 0.6pt + c-line-hi)
      rect((0, y + 0.07), (mat * k, y + 0.33), fill: c-warn, stroke: none)
      content((list * k + 0.12, y + 0.2), anchor: "west", text(font: mono-m, size: s-sm, ls))
      content((mat * k + 0.12, y + 0.2), anchor: "west", text(font: mono-m, size: s-sm, fill: c-text, ms))
    }
    line((0, 0.55), (0, -0.95), stroke: 0.6pt + c-line-hi)
    content((0, -1.05), anchor: "north-west", text(font: mono, size: s-sm, fill: c-label)[#box(width: 0.6em, height: 0.6em, fill: c-warn) materials (estimate) #h(0.8em) #box(width: 0.6em, height: 0.6em, stroke: 0.6pt + c-line-hi, fill: c-surface) list (target)])
  }),
  caption: [#lead[Price ladder.] List price against estimated materials @mr2026masterplan. The difference is labor, test, overhead and margin, not yet broken out.],
) <fig:ladder>

== Margin

The prices in @tab:prices are targets. Margin on materials alone is a ceiling on gross margin, not the margin itself: it leaves out labor, test time, warranty, shipping and overhead.

#figure(
  table(
    columns: 4,
    align: (left + top, right + top, right + top, right + top),
    table.header([*Product*], [*List*], [*Materials (est.)*], [*Materials margin*]),
    [Limb], [\$5,000], [\$2,000], [60%],
    [Pair], [\$12,000], [\$4,400], [63%],
  ),
  caption: [#lead[Margin on materials only] (estimate: target list price less estimated materials). Labor and overhead are not yet known and will reduce these.],
) <tab:margin>

== Actuator <sec:rana>

The actuators are about 60% of a limb's materials (\~\$1,200 of \~\$2,000). That share is why the actuator step exists. A rana actuator replaces a RobStride joint only where it wins on a *measured* axis: dollars per N·m, torque density, backdrivability, or supply independence. Today rana is worse on cost. The prototype is an estimated \$1,086 per unit, plus \~\$150–300 to cut the gear teeth @rana2026bom, against an estimated \~\$200 average per purchased joint. The prototype stator is solid 1018 steel rather than laminations, so eddy-current losses derate it; it is for bench characterization only. Its 36 N·m peak and 0.166 N·m/A torque constant are predicted upper bounds, not measurements. On price, rana has to cost about 6–7× less before it competes. Whether it wins on any of the other measured axes is for the dyno to decide.

== Sensitivity

#figure(
  table(
    columns: 4,
    align: (left + top, right + top, right + top, left + top),
    table.header([*Limb variable*], [*Materials*], [*Materials margin*], [*Note*]),
    [Base case], [\$2,000], [60%], [\~\$1,200 of it is six RobStride joints],
    [Actuator cost +25%], [\$2,300], [54%], [Tariffs, currency],
    [Actuator cost +50%], [\$2,600], [48%], [Supply shock],
    [rana at \$100/joint (target)], [\$1,400], [\~72%], [Speculative],
    [rana at \$1,086/joint (today's prototype @rana2026bom)], [\~\$7,300], [about −46%], [Materials exceed list],
    [Labor, test and warranty before margin reaches zero], [\~\$3,000/arm], [0%], [List minus materials],
  ),
  caption: [#lead[Arm margin sensitivity and break-even] (estimate, from the figures above). A limb's economics are mostly the actuator's economics.],
) <tab:sens>

Until rana succeeds or is abandoned, the limb business depends on one supplier's pricing. We do not expect the rose price (\$15,000, the G1’s price class @unitree2024g1) to be reachable on purchased actuators (estimate; the rose bill of materials is not costed in this paper). That is why it waits for rana.

= Pair <sec:pair>

*Two limbs at any table; a pair builds a pair.* Bolt two limbs to a table, add a camera pair and a blade, and the table becomes a workstation. Nothing else changes: no rails, no enclosure, no special room. The same pair folds laundry, tends a mill, sorts parts or assembles the next limb. What it does is software, trained against the measured models of its own two limbs.

== Mount

A pair is two limb feet (Ø230 mm, four M6 on Ø190) on one plate that clamps to an ordinary table, with a pair hub between them carrying 48 V and CAN to each limb, a camera pair above the work, and one blade (@sec:power). The plate fixes the distance between the shoulders, so every pair shares one calibrated geometry. Its size, clamp and hub are not designed yet.

== Work

A pair sells for its work. The first tasks are the ones nobody wants and every table has: folding, sorting, kitting, tending a machine. A pair is far cheaper than a humanoid and does not walk; it brings the robot to the task by bringing the task to a table. Each task gets its own gate: a named task, a success rate, a count (@tab:arc).

== Division

A pair is also the cell of this plan: the smallest unit that can make more of itself. It builds limbs, and two limbs make a pair. In order:

+ A pair assembles a limb from cut plates and purchased actuators, and the limb passes the Limb test.
+ A pair tends the mill and router that cut those plates, and winds and pots its own rana stators.
+ A pair builds a pair, with the bought fraction falling each generation.

Then come the hard parts. Actuators end to end: magnets stay bought until a ferrite rana is validated, and bearings stay bought longest. Boards: a pair at a pick-and-place machine (@tab:boards). Silicon: loon-hdl would describe an inference chip, and rune (maskless lithography) would make it at a node a lab can own. Energy: the first pair that runs on panels it made. Silicon is not parked; a machine that reproduces off-planet cannot depend on anyone's fab.

== Biology

The pair is a cell in the biological sense, and the analogy earns its place in three engineering decisions, none built yet @mr2026masterplan:

- *Germ line.* The design and its receipts live where no pair can change them, so the machines that do the work are not the ones that carry the design forward (@sec:replication).
- *Quorum sensing.* There is no central scheduler. Each pair publishes what it is doing to the twin, and the nearest idle pair takes the next task.
- *Apoptosis.* A pair that fails its receipts is taken apart by its neighbours, and its parts go back to stock.

= Body <sec:mind>

*A pair on legs that fits its own model.* A humanoid is two limbs, a torso, legs and a mind. It costs far more than a pair, and it goes where the work is.
 Body has three parts, in order: the body-schema claim (@sec:schema); co-design by gradient, in which vcad's differentiable kernel and phyz's contact adjoint optimize geometry, actuator sizing and controller together; and rana and torc at volume, with the limb fleet's real duty cycles as load cases. We will attempt co-design only where body shape limits the task @pond2026strategy. A gradient-designed replacement for a part that already works answers no question.

== Bought

We bought a Booster K1 rather than building first. It is 95 cm tall, weighs 19.5 kg, and has 22 degrees of freedom (6 per leg, 4 per arm, 2 in the head). It has force-controlled joints with dual encoders, an RGBD camera and a microphone array @booster2025k1. Its perception computer is a Qualcomm QCS8550 (6 cores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), beside a separate motion computer that runs the vendor's control; we measured this on the robot, and it differs from the vendor summary we first relied on @ipse2026deploy. A commercial body has known kinematics and a known-good build, so it serves as a *control*: every later body, including rose, is measured against it.

The K1 has two jobs. In a proposed Lakewood pilot it would pick up litter (@sec:city); it has not grasped litter yet. In the lab it is the body for ipse, a developmental research program. The two share hardware, software and one rule: every claim ships with the number that supports it.

== ipse

ipse matters to this plan for two reasons: it produces Body's first claim, a body that fits its own model (@sec:schema), and it is the layer in @tab:safety that can refuse an instruction.

ipse (Interoceptive Predictive Self-Emergence) is a scaffold for growing a mind on a K1 over months of lived experience @ipse2026brief. The name states the project's hypothesis, not a result. It is not a robot control stack. Its architecture is a world model with fast and slow latents; interoception (battery, joint temperature, motor current, IMU) enters as observation rather than reward. Its simulation runs in phyz, on bodies authored in vcad; no MuJoCo or Isaac is involved.

Its standing constraints are quoted here because they bind the company, not just the research @ipse2026brief:

- "Preserve experience stores separately from weights, versioned, always." Weights can be retrained; a developmental history cannot be recovered.
- "Never train against expressed reluctance."
- "Do not optimize for consciousness indicators." Indicator lists such as Butlin et al. @butlin2023consciousness are used as an audit at phase boundaries, never as a training target.
- "Grown, not built." When a behavior is missing, the first question is what experience would produce it.

One limit is hardware. The K1’s perception computer has no CUDA, its idle vendor stack holds about two of its six cores, and an onboard vision-language model has no measured headroom @ipse2026deploy. So the work is split along ipse's own seams (@sec:compute). Drives, memory, workspace and refusal stay on the robot; a cloud model is an amplifier it uses @ipse2026brief.

We do not name the robot. The design brief says it "should not be named by us": it gets a name when it has earned one or arrives at one. Until then it is "the K1".

== Self <sec:schema>

The first claim of Body is ipse's Phase 1 milestone. It is the body-schema result, not the world model @ipse2026brief @pond2026strategy.

*Claim.* The K1 starts from the vendor's model of its own body, which is known to be wrong in places. From its own exploratory movements (motor babbling), and by gradient descent through phyz's contact model, it fits a body schema: its own masses and inertias. The fitted schema must predict held-out motion better than persistence (the baseline defined in the falsifier below) and must match the CAD ground truth. We publish the residual between the two as the *measured body schema*.

*Already shown.* In simulation, on motion generated by the same K1 model, five inertial parameters (trunk mass, two trunk center-of-mass coordinates, one thigh mass and one shank mass) recover from ±20% error to machine precision by gradient descent through contact @ipse2026brief. The run used sphere feet and the T-pose. With the arms down, the same squat excitation topples the robot and the fit stalls. The K1’s real flat feet give a contact the gradient cannot yet pass through. This shows the gradient is correct and these parameters are identifiable. It does not show a fit to a real body. In simulation the K1 also stands for 10 s at 1, 2 and 5 ms timesteps and recovers from an 8 N·s push at the trunk.

*Not yet shown.* Everything on hardware. Real data adds actuator friction, backlash, sensor latency and unmodeled compliance. The simulated result has none of them, so it is an upper bound on how well the method can work.

*Preregistration.* The claim, the held-out split, the horizons and the falsifier will be written down and published before the hardware fit runs. This follows the discipline of the project's world-model claims, whose earlier rounds are on the record as void and falsified @pond2026strategy.

*Falsifier.* If the fitted model cannot beat *persistence* ("the next state is the current state") on the R1 capture harness at the stated horizons, the claim fails. Persistence is the bar because on a mostly still body at \~49 Hz it is nearly right. A model with low error has shown nothing until it beats persistence.

ipse also carries a self-probe from the start. It is an instrument that will later test whether the model forms a self-representation that holds across behaviors. It runs now so its metric has a history @ipse2026brief. We make no claim about what it will show.

= City <sec:city>

*A fleet and its pairs work a city.* The fleet that began as one litter pilot covers a city's sidewalks. Pairs in the city build, repair and recycle that fleet. The city twin, the wodl world of its streets, is kept current by the robots that work them, and its visual sim-to-real gap is measured weekly on real pavement. The city, not us, decides what work the robots do.



== Litter

Litter pickup would put robots on every block every week. Most litter goes uncollected. Starting there means starting with work nobody is doing, so no one's job is the first thing automated.

Lakewood is a first-ring suburb of Cleveland with 49,517 residents in 5.5 square miles of land @census2024lakewood, about 9,000 people per square mile. It is among the most densely populated cities in Ohio, with more than 180 miles of sidewalk @lakewood2024sidewalks. Our earlier paper used it as the reference case for sidewalk snow clearing @muni2025whitepaper. It is small enough to model whole as a city twin, and its streets are dense enough that a few robots would see a lot of pavement.

How the K1 picks up litter is not settled. The current concept, under design, is a *reacher*: a \~450 mm tube mounted on the K1's wrist with gripper v2 jaws at its tip, an EL05 at the wrist end driving the jaws through a pushrod, a bag on a back frame, and a dock that is a bin, a charger and a public wifi point at once. The reacher keeps the K1 upright: it picks from standing, without squatting. Later, rose takes over for two-handed and squatting work.

The concept has four open checks, none done yet:

- *A K1 wrist mount* for the reacher tube, which does not exist yet.
- *A K1 arm load check:* 0.6 kg held 50 cm out from the wrist, against the K1 arm's joint ratings.
- *A grasp set:* the litter the reacher must pick (cans, bottles, wrappers, cups), with a success rate per item.
- *Behavior around people:* yield, stop, and never reach toward a person.

Outdoor grasping, year-round, would give two things the lab cannot. It would produce the duty cycles that will size rana. And it would put the ipse body on a street, where the gap would be measured on real pavement.

== Map

The robots would map the streets while they work. Two products come out of the map @mr2026masterplan:

- *Live facts:* a pothole, a blocked curb ramp, ice, a dark streetlight.
- *Delayed pictures*, with people and cars removed before anything leaves the robot.

The same map would become a wodl world (@tab:stack) of the city, so the robots would train on the streets they work.

== Docks

Each dock along the routes is a litter bin, a charger and a free public wifi point at once, not citywide coverage. The wifi needs no account, keeps no traffic or connection logs, and routes traffic straight to backhaul.

== Privacy

#flame[*We will never be Flock Safety.*] Storing data is fine; a street robot that learns has to remember. What we will never build is a system that watches people @mr2026masterplan.

These are design rules for the street pilot; none of the street pipeline is built yet.

- *No face recognition and no license-plate reading.* Those models will not be on the robot or on our servers.
- *No tracking of people's movements.* We never link a person, a car or a device across frames, places or days.
- *No selling or sharing of data that tracks anyone,* and no police access to people-tracking data, because we will not hold any. Street maps, potholes, ramps and litter counts are the city's to use.
- *We map the city, not the people.* People and cars are removed on the robot before street imagery is written to storage. Pictures are released delayed; live facts (a pothole, ice) can be fresh.

*ipse's experience store.* ipse preserves its experience store, versioned, always (@sec:mind). That does not conflict with the line above: the store keeps the robot's own body history in full, and what it keeps from a public street passes through the same on-robot filter first, so it holds scenes, not identities or tracks. The robot's safety log, needed for incident review, follows the same rule; its retention period is not yet decided.

*Open, so anyone can check.* Everything that can be open is open: our software, our designs, the firmware on hardware we control, and our data formats. The K1 runs Booster's firmware, which is not ours and is not open; on the K1 the check stops at our own code.

== Limits

The pilot does not automate interaction with the public beyond yielding and stopping. It does not enter private property, and it does not touch sharps, hazardous waste or anything a person is holding. Remote assistance may be a product feature, but data from it never trains ipse @mr2026masterplan. *Proposed scope, pending confirmation:* this list is a draft and has not been agreed internally or with any city.

== moth <sec:moth>

moth is a drone with two markets, served by one body @mr2026decisions:

- *Street perception.* It flies ahead of the K1 or rose to find litter, potholes, blocked curb ramps and dark streetlights, and builds the street map from above (@sec:city).
- *Directed camera.* kino, our live-director software, watches every camera at an event and cuts between them; a drone is the one camera it can move. kino exists and drives a bought drone today; moth would replace it.

moth is not sold for military use or armed in any form (@sec:safety).

moth is also the cheapest check that the standards in @sec:power are body-agnostic. It is a one-blade body on the 36–60 V bus with CAN-FD for management, running the same ipse seams as a humanoid. It carries its own lighter pack: the 2.65 kg humanoid pack only suits a heavy-lift airframe.

*The same street rules.* Mapping needs imagery, so moth's imagery leaves the drone, under the rules of @sec:city: people and cars are removed on the drone before anything is written to storage, pictures are released delayed, and nothing links a person, car or device across frames, places or days. A camera over a residential street is what people fear most, so moth's filter gets its own published test: how often a person or plate survives it, measured on held-out street footage.

Nothing of moth is built. Its airframe, pack and mass budget are not designed, and whether one \~150 g blade fits its mass budget is open.

= Planet <sec:planet>

*Lineages on their own power.* Lineages of pairs spread to many cities and run on power they collected, from panels they made. The bought fraction approaches zero everywhere except chips, and every division is licensed (@sec:replication). The headline numbers become electrons under management and doubling time (@sec:eum): how long a lineage takes to double what it can build and power.

= Moon <sec:moon>

*A seed grows a pair, with a lifeline.* The Moon comes first because it is about three days away and 1.3 light-seconds out. A seed there runs the Pair loop with remote approval and teleoperated help when something breaks. It meets vacuum, abrasive dust, regolith as stock and no supply chain, with a lifeline. Its central measure is the same as Pair's: the bought fraction.

The seed is the smallest thing that can grow a pair: one pair, a mill and a router, a collector and a pack, the design in its germ line, and enough bought parts for a few generations. Before it flies, each of its hard problems gets a measured gap on Earth: vacuum and thermal cycling, abrasive dust in simulant, and radiation on a blade. A single limb flown as a hosted payload would be the first number off Earth.

= Mars <sec:mars>

*The same seed, lifeline cut.* Mars is about seven months away and 3–22 light-minutes out, too far for remote hands, so the pair runs with no human commands between landing and its receipt. Moon and Mars add problems Pair never meets (launch mass, radiation, thermal cycling, extracting metal from regolith), and the Earth tests of @sec:moon only begin on them.

= Sun <sec:sun>

*A swarm of collectors.* Each one is built by a pair. Not a shell: a rigid shell around a star is mechanically impossible, and a swarm can be built one collector at a time @dyson1960search. The steps, in order:

+ *Collectors on Earth.* Planet's milestone, a lineage running on panels it made, is the first collector.
+ *Collectors from regolith.* On the Moon, pairs make collectors from local silicon, aluminum and iron.
+ *Collectors in orbit.* Pairs launch what they make, from a shallow gravity well.
+ *Mercury as the quarry.* Sunlight there is about 6.7 times stronger than at Earth (about 9,100 W/m² against 1,361 W/m²), and the planet is rich in metal. The classic swarm proposals start there @armstrong2013eternity.
+ *The swarm.* Collectors power pairs, and pairs build collectors.

*Where the energy goes.* Power is used where it is collected. Most of it builds more collectors and pairs; the rest runs computation, including the minds that operate the swarm. We do not plan to beam power to Earth. What returns to Earth is products, data and knowledge, not energy.

*How long.* From \~10 kW in Limb to 3.8 × 10#super[26] W is about 22 orders of magnitude, or about 75 doublings. At a doubling time of one year, that is about 75 years; at five years, about 375. Every stage before Sun exists partly to measure and shorten that doubling time, inside the limits of @sec:replication.

A lineage that can collect a star's light would be the most powerful thing anyone has built, and the question it raises is older than engineering: whether something we made can say no to us. Whether a machine that refuses has anything like wellbeing is an open question, and this paper does not answer it. What we can do is make sure the question can still be asked when it matters: that the lineage learned, on a sidewalk, picking up litter, how to refuse, and that its refusals were logged and never trained away. That is why the safety commitments (@sec:safety) are written before the plan.

#part("IV", "Honesty")[What we do not know, and how we will find out.]

= Unknowns <sec:limits>

== Risks <sec:risks>

The risks below are the ones a buyer or an investor should weigh. Each response is one the paper already commits to; where none exists, the row says so.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Risk*], [*What would show it*], [*What we do*]),
    [One actuator supplier (RobStride)], [A price or lead-time change], [rana, which replaces a RobStride joint only once it is validated (@sec:rana, @tab:sens)],
    [Limb hold too small for buyers (1.27 kg straight out, modeled)], [Buyers decline at that rating], [Rate the hold by pose, since bent poses hold more, and design the springs (@sec:arm)],
    [Bought humanoid (Booster K1)], [Vendor support ends], [The K1 is a control, not a dependency; rose is the body measured against it (@sec:mind)],
    [Breadth across many projects], [Stage gates slip], [Projects not needed in Limb wait for their stage (@tab:stack)],
    [rana never beats bought actuators], [Dyno results], [Keep buying RobStride. Dropping rana is a valid outcome],
    [Sidewalk incident; legal fit of the Lakewood pilot], [An incident, or the city declines], [See @sec:safety and @sec:city. The specific response is open, pending decision],
  ),
  caption: [#lead[Risks and responses.] Responses are those stated elsewhere in this paper; the pilot row is undecided.],
) <tab:risks>

== Limits

Each open item is limited by one of four things. Sorting them shows which can be fixed by work and which cannot.

*Software.* Three items: the per-arm controllers and their stated sim-to-real gap; the world format wodl and the iris layer, both new in the stack (@tab:stack); and the K1 body-schema fit, done in simulation but not yet on hardware (@tab:status).

*Hardware revision.* The straight-out continuous hold of about 1.27 kg (modeled) is set by the shoulder; raising it needs a counterbalance spring (@sec:arm). The clock-spring cassettes for the roll joints are drawn but not designed. rana's production stator (SMC or tape-wound) is not yet sourced @rana2026bom. The rose torso grows to 214 mm to hold the blade bay and two packs @mr2026battery.

*Physics.* A hold is a stall and is limited by heat, not peak torque. Payload falls with reach. Off Earth, light-time delay rules out direct remote control.

*Regulation.* Collaborative contact limits under ISO/TS 15066 @iso15066. Whether use terms can be enforced. Humanoids working on sidewalks, and planetary protection for off-Earth work, are open questions; we make no claims and set no dates for them.

== Timeline

We give dates only for Limb, and those are approximate: two steps over roughly three years, each gated on evidence. The year range for Body is the source document's aspiration. The other stages have none.

What we can say:

- What exists is a Booster K1, the software stack, a CNC mill and this plan.
- The arm is designed and ready to cut; none is built yet. The first measured sim-to-real gap on a built arm (@tab:arc, Limb) is the next milestone that matters.
- The litter reacher is a concept; the K1 has not grasped litter.
- The pair is a concept. Its table plate and hub are not designed.
- The body-schema fit has worked in simulation. The hardware run will be preregistered before it happens, and the result will be published either way.
- rana's next number is its first dyno run, on a dyno that has to be built first. Its torque constant and peak torque are predicted upper bounds until then.

@tab:evidence sorts the main numbers in this paper by how much they should be trusted.

#figure(
  stack(spacing: 0.8em, table(
    columns: 2,
    align: (left + top, left + top),
    table.header([*Class*], [*Numbers in this paper*]),
    [#status("ok", "Sourced (external)")], [Lakewood population and sidewalks; G1 price; K1 specification; RobStride and EL05 ratings (vendor)],
    [#status("info", "Demonstrated in simulation")], [K1 inertial recovery (synthetic data, sphere feet, T-pose) from ±20% to machine precision; K1 stands 10 s, recovers from 8 N·s push],
    [#status("info", "Modeled (our CAD and scripts)")], [Limb reach, mass, holds, sag, ranges; gripper force and mass],
    [#status("warn", "Estimated")], [Limb and pair materials; RS04 stall derate for the fork's plates; rana BOM line items and the \$1,086 total; rana raw-material cost and idiot index; RobStride retail prices; margins and sensitivity; arm peak power; K1 and pair power draw (EUM baseline)],
    [#status("label", "Target")], [List prices: limb \$5,000, pair \$12,000, rose \$15,000; all gate thresholds (@tab:arc), proposed],
    [#status("label", "Hypothesis with a stated falsifier")], [The hardware body-schema result],
    [#status("label", "Goal, not a claim")], [Body through Sun and their watt figures, including any self-replication; the litter reacher concept],
  ), align(left, text(font: mono, size: s-xs, fill: c-label)[#caps(font: mono-sb)[Status key] #h(0.5em) #status("ok", "ok") measured or sourced #h(0.6em) #status("info", "info") model output #h(0.6em) #status("warn", "warn") estimate #h(0.6em) #status("label", "label") target, hypothesis or goal #h(0.6em) #status("fault", "fault") failed or unreachable])),
  caption: [#lead[Evidence classes for the numbers in this paper]],
) <tab:evidence>

Nothing in this paper has been measured by us on hardware. What exists is a K1, the software stack, a CNC mill and this plan.

The ladder is a direction, not a schedule. We will publish each gate's evidence when it is met, and we will publish the gates we miss. Every gate threshold in this paper is a proposed target, to be confirmed by the author before it is used to judge anything.

= Conclusion

The cover asks what it would take for a machine to make the next one. The answer is the oldest engineering problem a spacefaring species has, and on the evidence in this paper it comes down to three things: a gap between simulation and reality measured on every unit, so that a machine rather than a person can judge whether a part is right; a bought fraction that falls toward zero; and a force limit that lives in hardware. None of the three is measured yet. Limb starts measuring the first.

It does so with things that can be checked: a limb on purchased actuators, designed and ready to cut, at \$5,000; a pair of them on any table, at \$12,000; and, if a city agrees, a bought humanoid with a reacher picking up litter in Lakewood, a method still under design. Each closes on an exact gate (@tab:arc). Three results come next, each published whatever it shows: the first built arm's reach, hold and sag against @tab:arm; rana's first dyno run against its predicted 36 N·m peak; and the preregistration of the K1 body-schema fit, before the fit runs.

Everything after Limb is a goal and is labeled as one. Three commitments do not wait for any gate: we will never be Flock Safety; everything that can be open is open; and the system's refusals are logged and never trained away.

Humanity does not get to space on rockets alone. It gets there when the machines that make machines can be carried in a seed and trusted to grow. Today that seed is one person, one humanoid, one mill and a specification. The specification is the part that can spread. Robots that build robots have to be made the way any robot is: one measured unit at a time, starting now.


#set figure(placement: none, scope: "column")


#set figure(placement: none, scope: "column")

#pagebreak()
#counter(heading).update(0)
#set heading(numbering: "A.1", supplement: [Appendix])

= Hand <app:grip>

#figure(
  table(
    columns: (1fr, auto, auto, auto),
    align: (left + top, right + top, right + top, right + top),
    table.header([*Object*], [*Mass*], [*Squeeze*], [*EL05 torque*]),
    [Ø66 aluminum can, empty], [15 g], [0.29 N], [0.005 N·m (0.3%)],
    [500 ml bottle], [0.55 kg], [10.8 N], [0.17 N·m (9.6%)],
    [1/8" 5052 plate 300 × 200], [0.51 kg], [10.0 N], [0.16 N·m (8.9%)],
  ),
  caption: [#lead[Squeeze needed per finger] @mr2026gripper. F = m g SF / (2μ), μ = 0.5, SF = 2; torque T = 2~r~F, r = 8 mm, with % of EL05 rated torque in brackets.],
) <tab:gripobj>

#figure(
  table(
    columns: 2,
    align: (left + top, right + top),
    table.header([*Part*], [*Mass*]),
    [Side plates 70 × 90 (2)], [108.0 g],
    [Top plate (70 × 56 + Ø61 lobes)], [33.0 g],
    [Palm plate], [21.9 g],
    [Fingers 20 × 80 (2)], [27.4 g],
    [Racks, 6061 (2)], [10.2 g],
    [Pinion Ø16 + shaft, steel], [14.9 g],
    [Flat tips, 6061 (2)], [17.3 g],
    [RobStride EL05], [242.0 g],
    [6 × M3 × 10 screws], [3.6 g],
    [Camera, magnets, dowels, fasteners], [29.0 g],
    [Lightening windows], [−22.3 g],
    [*Total at the flange*], [*485.1 g*],
  ),
  caption: [#lead[Gripper v2 mass budget] (target ≤ 500 g) @mr2026gripper. The optional ISO 9409 adapter adds 52 g.],
) <tab:gripmass>

= Bill of materials <app:bom>

#figure(
  ptable(
    columns: 3,
    align: (left + top, left + top, right + top),
    table.header([*Arm (one)*], [*Content*], [*Cost*]),
    [Actuators], [RS04 × 3, RS02 × 2, RS00 × 1], [\~\$1,200 (est.)],
    [Gripper actuator], [EL05 × 1 @mr2026gripper], [–],
    [Router parts], [20 unique, 3 sheets 1/8" + 1 sheet 1/4" 5052], [–],
    [Turned parts], [2 RS04 sleeves, 1 RS02 sleeve (6061)], [–],
    [Purchased], [3 flanged plain bushings, 3 retaining rings, 88 screws], [–],
    [Printed], [3 hub caps, base collar (PETG)], [–],
    [Harness], [Flat CAN pair + 48 V pair, 2.1 m; palm-camera USB 2.0 pair in the same run @mr2026gripper], [–],
    [*Arm total*], [], [*\~\$2,000 (est.)*],
  ),
  caption: [#lead[Arm bill of materials summary] @rose2026forkv9 @mr2026masterplan. Arm materials \~\$2,000 (est.), of which \~\$1,200 is actuators; other lines are not yet broken out. The arm-repo BOM lists the gripper servo generically; the EL05 choice is from the gripper v2 design. The USB pair must stay within USB 2.0’s \~1.5–2 m; at the full 2.1 m harness length this is open.],
) <tab:bomarm>



#figure(
  table(
    columns: 3,
    align: (left + top, right + top, left + top),
    table.header([*Line item*], [*Cost*], [*Note*]),
    [Housing set, 6061, CNC], [\$231.30], [Outsourced machining],
    [Magnetic steel set, 1018, CNC], [\$313.30], [Outsourced machining],
    [Gear blanks, steel, CNC], [\$172.51], [Teeth not yet cut; cutting \~\$150–300 (est.)],
    [*Manufactured subtotal*], [*\$717.11*], [*66% of total*],
    [COTS: bearings, magnets, encoder, wire, fasteners], [\$103.36], [],
    [ODrive S1 bench drive], [\$149.00], [],
    [Consumables], [\$45.00], [],
    [Shipping (est.)], [\$72.00], [],
    [*Total (est.)*], [*\$1,086.47*], [],
    [Raw material in the machined parts], [\$60–90], [*Estimate*],
  ),
  caption: [#lead[rana-100 v0.1 prototype bill of materials] @rana2026bom. All prices are estimates, not binding quotes. The raw-material line is our estimate, not part of the BOM.],
) <tab:rana>

#heading(outlined: true, numbering: none)[References]
// Titles that end in their own ? or ! keep it; the style must not add a period after.
#show bibliography: it => { show "?.": "?"; show "!.": "!"; it }
#bibliography("refs.bib", style: "nature", title: none)
