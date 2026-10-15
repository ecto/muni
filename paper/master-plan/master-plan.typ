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

#set page(
  paper: "us-letter",
  fill: c-bg,
  margin: (x: 0.55in, top: 0.75in, bottom: 0.72in),
  columns: 2,
  numbering: "1",
  header: context {
    if counter(page).get().first() > 1 {
      grid(columns: (1fr, auto), caps[Robots that build robots], caps[Municipal Robotics])
      v(-0.35em)
      line(length: 100%, stroke: 0.5pt + c-line)
    }
  },
  footer: context {
    if counter(page).get().first() > 1 {
      line(length: 100%, stroke: 0.5pt + c-line)
      v(-0.35em)
      grid(columns: (1fr, auto, 1fr),
        caps[Rev 1.0 · October 2026],
        text(font: mono-m, size: s-xs, fill: c-label, number-width: "tabular")[#{ let n = counter(page).get().first(); if n < 10 { "0" + str(n) } else { str(n) } } / #counter(page).final().first()],
        align(right, caps[Whitepaper]))
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
#show heading.where(level: 1): it => block(above: 2.2em, below: 1.1em, sticky: true, width: 100%, {
  context if here().position().y > 0.75in + 2pt { line(length: 100%, stroke: 0.6pt + c-line-hi); v(0.15em) }
  if it.numbering != none { numlabel(it); v(-0.35em) }
  text(font: sans-sb, size: s-lg, tracking: -0.01em, it.body)
})
#show heading.where(level: 2): it => block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, {
  if it.numbering == none { return text(font: sans-m, size: s-md, it.body) }
  set par(justify: false)
  grid(columns: (2.9em, 1fr), align: (left + top, left), pad(top: 0.3em, numlabel(it)), text(font: sans-m, size: s-md, it.body))
})
#show heading.where(level: 3): it => block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, text(font: sans-sb, size: body-size, it.body))
#show raw: set text(font: mono, size: 1em)
// Keep numbers with their units, and short expressions, on one line
#let unit-re = "(mm|kg|g|N·m|N·s|N|kN|W|kW|Hz|ms|s|m|cm|in|V)\\b"
#show regex(" [×≈=≤≥] \\d[\\d.,]*( " + unit-re + ")?"): it => it.text.replace(" ", "\u{a0}")
#show regex("\\d " + unit-re): it => it.text.replace(" ", "\u{a0}")
#show regex("–\\d"): it => it.text.replace("–", "–\u{2060}")
#set list(marker: text(font: mono, fill: c-label)[–], indent: 0em, body-indent: 0.7em, spacing: 1.15em)
#show list: set par(justify: false)
#show enum: set par(justify: false)
#set enum(numbering: n => text(font: mono-m, fill: c-label, str(n) + "."), indent: 0em, body-indent: 0.7em, spacing: 1.15em)

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

#set figure(placement: top, scope: "parent", gap: 0.85em)
#show figure: set block(above: 1.6em, below: 1.6em)
#show figure: set text(hyphenate: false, font: sans, weight: 400)
#show table: set text(weight: 400)
#show heading: set text(font: sans, weight: 400, hyphenate: false)
#set place(clearance: 1.8em)
// short lead phrase of a caption, set in Medium
#let lead(body) = text(font: sans-m, fill: c-text, body)
#show figure.where(kind: table): set figure.caption(position: top)
#show figure.caption: it => {
  set par(leading: 0.64em, justify: false)
  set align(left)
  block(width: 100%, text(font: sans, size: s-sm, fill: c-label, hyphenate: false)[
    #caps(if it.kind == table [Table] else [Fig] + nb + it.counter.display(), font: mono-sb, fill: c-text)#h(0.7em)#it.body
  ])
}

// Column-width figure that stays in flow
#show figure: it => if it.scope == "parent" { show figure.caption: c => align(left, block(width: 72%, c)); it } else { it }
#show figure.where(kind: table): it => { show table: t => align(left, t); it }


#let colfig(body, caption: none) = figure(body, caption: caption, placement: none, scope: "column")

// Diagram node fills: grey structure only
#let n-fill = c-surface
#let n-stroke = 0.7pt + c-line-hi

// Title page (single column). The one flame: the orange rule under the title.
#page(columns: 1, margin: (x: 0.75in, top: 0.7in, bottom: 0.65in))[
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
  #v(1.2em)
  #grid(columns: (auto, 1fr), column-gutter: 2.4em,
    {
      frame(inset: 6pt, stroke: 0.6pt + c-line-hi, fill: c-bg, image("images-native/bench_shallow_work.png", height: 4.4in))
      v(0.5em)
      block(width: 4.4in + 12pt, text(font: sans, size: s-sm, fill: c-label)[#caps(font: mono-sb, fill: c-text)[Cover]#h(0.7em)#lead[The first bench (modeled).] Two rose fork v9 arms on a sliding plinth inside a 1500 × 500 × 1000 mm 4040 frame, working at a docked bay.])
    },
    {
      set par(leading: 0.55em)
      let ro(l, v, u) = block(below: 1.4em, stroke: (left: 0.6pt + c-line-hi), inset: (left: 0.7em, y: 0.1em), stack(spacing: 0.55em, caps(l), text(font: mono-m, size: s-xl, number-width: "tabular")[#v#text(size: s-sm, fill: c-label)[#nb#u]]))
      ro[Arm list][\$5,000][usd]
      ro[Arm materials][\~\$2,000][usd, est.]
      ro[Bench list][\$15,000][usd]
      ro[Reach, shoulder → tips][786][mm]
      ro[Continuous hold][1.27][kg]
      ro[Gripper at flange][485][g]
      text(hyphenate: false, caps[Exists today: a Booster K1, the software stack, a CNC mill, a plan. Arm: designed, ready to cut; none built yet. Litter pickup: concept under design. Lakewood pilot: proposed, no agreement.])
      v(0.4em)
      text(hyphenate: false, caps[Prices are targets; other values modeled or estimated; none measured on hardware])
    })
  #v(1fr)
  #line(length: 100%, stroke: 0.6pt + c-line-hi)
  #v(0.6em)
  #grid(columns: (1fr,) * 3, column-gutter: 1.2em, row-gutter: 1.3em,
    readout[Author][Cam Pedersen], readout[Place][Cleveland, Ohio], readout[Contact][#link("mailto:info@muni.works")[info\@muni.works]],
  )
]

// Abstract (single column)
#page(columns: 1)[
  #block(width: 4.9in)[
  #heading(outlined: false, numbering: none)[Abstract]
  #set text(size: s-md)
  #set par(leading: 0.86em, spacing: 1.4em)

  This is a specification for how humanity gets to space: not by one heroic launch, but by machines that make the next machine, each one measured, until the bought fraction falls to zero and a seed of them can go where no supply chain reaches. It starts on a sidewalk in Ohio.

  Robots are expensive because people make them. A prototype actuator we designed is estimated at \$1,086 to have built, before its gear teeth are cut; about two thirds of that is outsourced machining, on perhaps \$60–90 of raw metal (estimates). The difference is labor, coordination and margin. A machine that can make more of itself attacks that markup directly. So does a design process that measures the sim-to-real gap on every unit and feeds it back into the design as a gradient.

  Municipal Robotics is one person, Cam Pedersen. What exists today is a Booster K1 humanoid, the software stack, a CNC mill and this plan. The stack spans the loop: a source language for parts, a CAD kernel, a differentiable physics engine with a contact adjoint, an actuator, an arm, and ipse, a developmental learning system for the K1. Its key result exists only in simulation: five of the K1's inertial parameters, started at ±20% error, recovered to machine precision by gradient descent through contact.

  The first steps of the specification are products: a six-axis arm on purchased RobStride actuators (\$5,000 list, \~\$2,000 materials), designed and ready to cut, none built yet; and a two-arm bench cell in a 1500 × 500 × 1000 mm 4040 frame (\$15,000 list, \~\$5,900 materials with the frame). Our models put the arm's reach at 786 mm with our 485 g gripper and its continuous hold straight out at 1.27 kg. The municipal work is a proposed litter pilot in Lakewood, Ohio, with no agreement in place; the method, a reacher on the K1's wrist, is under design, and the K1 has not grasped litter yet.

  Chapter 1 closes on exact gates (proposed targets): ten arms sold at 40% gross margin, each with its tool-point error measured and under 2 mm RMS; our actuator beating the purchased one by 20% on a named dyno metric; a cell assembling an arm that passes the same test with under half its cost bought. Four more chapters follow: a body that fits its own model, a cell that builds a cell, a seed on the Moon and then Mars, and a lineage powered by the sun. They are goals, not results, and they are the point. Progress is one number, electrons under management: the mean power drawn by machines we built, in service. Today it is zero watts. The body-schema claim that opens chapter 2 will be preregistered, with its falsifier, before its hardware run. Every number here is cited, or labeled as an estimate or a model output.
]
#place(top + right, dy: 0.2em, block(width: 2.1in)[
#frame(width: 100%, inset: 0.8em, fill: c-surface)[
  #set text(size: s-sm)
  #set align(left)
  #set par(spacing: 1.2em, leading: 0.8em)
  #text(size: s-md)[Arm, bench and pilot inquiries] \
  #v(0.3em)
  The first steps need early arm and bench buyers who will share measured data,
  and a municipal partner for a one-city, one-task litter pilot. \
  #v(0.3em)
  #text(font: mono, size: s-xs)[#link("mailto:info@muni.works")[info\@muni.works] · #link("https://muni.works")[muni.works]]
]
])
]


// Table of contents (single column)
#page(columns: 1)[
  #set text(font: sans, weight: 400, size: body-size, number-width: "tabular")
  #set par(leading: 0.6em, justify: false)
  #set outline.entry(fill: none)
  #show outline.entry: it => { let l1 = it.level == 1; block(above: if l1 {1.0em} else {0.55em}, below: 0em, link(it.element.location(), { h(if l1 {0em} else {2.6em}); box(width: 2.6em, text(font: mono-m, size: s-xs, fill: c-label, tracking: track, if it.prefix() != none { it.prefix() })); text(font: if l1 {sans-m} else {sans}, fill: if l1 {c-text} else {c-label}, it.body()); h(1fr); text(font: mono, size: s-sm, fill: c-label, it.page()) })) }
  #text(font: sans-sb, size: s-lg)[Contents]
  #v(0.6em)
  #columns(2, gutter: 2.4em, outline(
    title: none,
    indent: 0em,
    depth: 2,
  ))
]

= Introduction

Every civilization that left its planet would have had to solve one problem first: making machines without people making each machine. Launch is a transport problem. Staying is a manufacturing problem. A settlement that needs a factory on Earth for every bearing, plate and chip is a colony on a supply line; one that carries a cell able to build the next cell is the start of something that grows. This paper is a specification for that cell, and for the measured, gated path from a litter-picking robot on an Ohio sidewalk to a lineage of machines powered by the sun.

Municipal Robotics is a one-person company: Cam Pedersen. There is no team and no lab behind it. What exists today is exactly this: a Booster K1 humanoid, the software stack (@tab:stack), a CNC mill, and the plan you are reading. No arm has been built, there is no actuator dyno yet, and the K1 has not grasped litter.

The paper is written for anyone who wants to check it or join it: technical readers, buyers of a table arm or a small manufacturing cell, municipal partners, collaborators and investors. It keeps the register of our sidewalk-maintenance paper @muni2025whitepaper: plain claims, numbers with sources, exact gates, and a section on what we do not claim.

Most of what this paper describes is *proposed*, not operational. @tab:status separates them. Throughout the paper, "modeled" means a number comes from our own CAD, kinematics or load scripts and has not been measured on hardware. "Measured" is reserved for results from our own hardware, and this paper reports none yet. Vendor ratings and public data are cited as sourced.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Item*], [*State*], [*Evidence*]),
    [Booster K1], [In hand], [Purchased platform; vendor firmware @booster2025k1],
    [Software stack], [Exists, in use, not finished], [@tab:stack @pond2026strategy],
    [CNC mill], [In hand], [–],
    [Arm, rose fork v9], [Designed, ready to cut; none built yet], [Fab package, 20 DXFs, 88-screw BOM @rose2026forkv9],
    [Gripper v2], [Designed; none built], [CAD + force/mass model @mr2026gripper],
    [First bench], [Designed; not built], [Reach + load model @mr2026bench],
    [Aisle, hall, seed], [Concept mockups], [Layout models only @mr2026cells],
    [rana actuator], [Prototype BOM, estimated; not built], [\$1,086 est. per unit, before tooth cutting @rana2026bom],
    [Actuator dyno], [Planned; does not exist], [None],
    [Pack, blade, board library], [Concept mockups; none built], [Pack and blade models @mr2026battery @mr2026compute],
    [K1 litter reacher], [Concept under design], [None; the K1 has not grasped litter],
    [K1 body-schema fit (sim)], [Five-parameter recovery in simulation], [Synthetic data, sphere feet; ±20% → machine precision @ipse2026brief],
    [K1 body-schema fit (hardware)], [Not yet on hardware], [Preregistration pending],
    [Lakewood litter pilot], [Proposed], [No agreement in place],
    [Chapters 2–5], [Goals, not claims], [None],
  ),
  placement: bottom,
  caption: [#lead[What exists, what is designed, and what is proposed] (October 2026). What exists: a K1, the software stack, a CNC mill and a plan.],
) <tab:status>

The paper is organized around one loop (@fig:loop). The design is written as source code. It is simulated in a physics engine that can be differentiated, built by machines we control, and measured. The difference between them, measured minus predicted, is what we call the gap. It is the most valuable number in the system, because it is the one thing nobody can compute in advance. Here it is written down as a product specification: every arm will ship with its sim-to-real gap stated.

#figure(
  diagram(
    node-stroke: n-stroke,
    edge-corner-radius: none,
    edge-stroke: 0.7pt + c-label,
    spacing: (30mm, 9mm),
    node((0, 0), [*Design*\ #text(fill: c-label, size: s-sm)[loon source → vcad]], shape: sqnode, fill: n-fill, width: 38mm, name: <d>),
    node((2, 0), [*Simulate*\ #text(fill: c-label, size: s-sm)[phyz, wodl world]], shape: sqnode, fill: n-fill, width: 38mm, name: <s>),
    node((2, 2), [*Build*\ #text(fill: c-label, size: s-sm)[kerf, bench cell]], shape: sqnode, fill: n-fill, width: 38mm, name: <b>),
    node((0, 2), [*Measure*\ #text(fill: c-label, size: s-sm)[dyno, cameras, K1]], shape: sqnode, fill: n-fill, width: 38mm, name: <m>),
    node((1, 1), [*Gap*\ measured − predicted\ #text(fill: c-label, size: s-sm)[published on every unit]], shape: sqnode, fill: c-surface, stroke: 1.2pt + c-sel, width: 40mm, name: <g>),
    edge(<d>, <s>, "-|>", text(size: s-sm, fill: c-label)[parameters θ]),
    edge(<s>, <b>, "-|>", text(size: s-sm, fill: c-label)[prediction], label-side: left),
    edge(<b>, <m>, "-|>", text(size: s-sm, fill: c-label)[built unit]),
    edge(<m>, (0, 1), <g>, "-|>"),
    edge(<g>, (1, 0.5), (0, 0.5), <d>, "-|>", text(size: s-sm, fill: c-text)[gradient ∂J/∂θ], label-pos: 0.5, label-side: right),
  ),
  placement: bottom,
  caption: [#lead[The loop, read clockwise.] The small labels name the tool at each step. The software projects are listed in @tab:stack, and the dyno is a planned actuator test stand that does not exist yet. Each later chapter of the plan removes a human from one more edge. In chapter 1 the gradient edge is a person reading the gap and redesigning by hand.],
) <fig:loop>

== The plan in brief <sec:plan>

Chapter 1 has three steps over roughly three years. Each step closes on a gate, and every gate is exact: a number, a unit, a method and a pass condition (@tab:gates) @mr2026masterplan. Chapter 1 has to earn the rest of the specification: it funds the long bets and closes the loop for the first time. Four further chapters follow, and together the five are what we call the arc (@sec:arc, @tab:arc). They are goals, not results, and they are why chapter 1 exists.

+ *Arms.* Build an arm worth buying on RobStride actuators. The actuator is a commodity. Our edge is everything around it: geometry checked against real loads, a measured model of every unit, a controller trained against that model, and a stated sim-to-real gap on every arm.
+ *Own the actuator, where it wins.* Arm revenue funds rana and torc. A purchased joint is replaced only where rana wins on a measured metric. Dropping rana is a valid outcome, not a failure.
+ *The machine that builds the machine.* Arms tend the cell that makes arms. The rose humanoid follows, on rana legs with two of our arms, at a cost the first two steps paid to learn.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let steps = (
      ([Step 1 · Arms], [arm on RobStride\ litter pilot · first bench]),
      ([Step 2 · Actuator], [rana + torc on the dyno\ replace where it wins]),
      ([Step 3 · The cell], [arms tend the cell that\ makes arms; rose follows]),
    )
    let gates = (
      ([Gate 1], [≥ 10 arms sold at ≥ 40% gross margin; TCP error ≤ 2.0 mm RMS on every arm]),
      ([Gate 2], [rana ≥ 1.2 × RS04 on a named dyno metric; rose walks 100 m, 5 of 5]),
      ([Gate 3], [cell-built arm passes gate 1's test; bought ≤ 50% of cost]),
    )
    let bw = 3.9
    let pitch = 5.6
    // the time axis
    line((0, 0), (18.4, 0), stroke: 0.6pt + c-line-hi, mark: (end: "straight", size: 0.18))
    content((0, -0.45), anchor: "north-west", caps(size: s-xs)[Evidence, not dates →])
    for i in range(3) {
      let x = i * pitch
      rect((x, 0.35), (x + bw, 1.95), fill: c-surface, stroke: if i == 0 { 1pt + c-text } else { 0.6pt + c-line-hi })
      content((x + 0.2, 1.8), anchor: "north-west", text(font: sans-sb, size: s-sm, steps.at(i).at(0)))
      content((x + 0.2, 1.25), anchor: "north-west", text(size: s-sm, fill: c-label, steps.at(i).at(1)))
      line((x, 0), (x + bw, 0), stroke: 2pt + if i == 0 { c-text } else { c-line-hi })
      // gate diamond at the end of the step
      let gx = x + bw + 0.9
      line((gx, 0.28), (gx + 0.28, 0), (gx, -0.28), (gx - 0.28, 0), close: true, fill: c-bg, stroke: 0.9pt + c-text)
      content((gx - 0.9, -0.5), anchor: "north-west", box(width: 3.4cm, {
        set par(leading: 0.45em)
        caps(size: s-xs, fill: c-text, font: mono-sb, gates.at(i).at(0))
        linebreak()
        text(size: s-sm, fill: c-label, gates.at(i).at(1))
      }))
    }
    content((0, 2.35), anchor: "south-west", caps(size: s-xs)[Now · year 1])
    content((15.1, 2.35), anchor: "south-east", caps(size: s-xs)[\~year 3])
  }),
  caption: [#lead[Chapter 1 as a timeline.] Three steps, each closed by a gate (diamond) that is evidence, not a date @mr2026masterplan. Gates abbreviated; the full conditions are in @tab:gates. Step 1 is the current step.],
) <fig:steps>

#figure(
  table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Gate*], [*Number and unit*], [*Method*], [*Pass if*]),
    [1a · Arms sell], [Arms delivered to outside buyers (count); gross margin (%) after materials, labor, test and warranty], [Invoices and cost ledger, trailing 12 months], [≥ 10 arms and ≥ 40%],
    [1b · Gap stated], [Tool-point (TCP) position error, model vs measured (mm RMS), 50 fixed poses, unloaded and at 1.0 kg], [Probe tip against a calibrated fixture plate, per arm], [≤ 2.0 mm RMS on every arm shipped, printed on its receipt],
    [1c · Pilot renews], [Renewal term (months) after a ≥ 3-month trial; contact incidents with a person (count)], [Signed renewal; incident log], [≥ 12 months and 0 incidents],
    [2a · rana wins], [Continuous stall torque per kg (N·m/kg) or per dollar of 100-unit cost (N·m/\$), rana vs RS04], [Same dyno, same 345 × 345 mm heat-sink plate, 25 °C, 30 min thermal steady state], [≥ 1.20 × RS04 on at least one of the two],
    [2b · rose walks], [Distance (m) on level pavement carrying two arms; falls (count)], [5 trials, logged], [100 m, 0 falls in 5 of 5],
    [3a · The cell builds an arm], [Cell-assembled arm's TCP error (mm RMS); human touch time (min)], [Gate 1b test; time log], [≤ 2.0 mm RMS and ≤ 30 min],
    [3b · Bought fraction], [Bought share of that arm's cost (%)], [Receipts per build], [≤ 50%],
    [3c · A second product pays], [Units of a second product (count); gross margin (%)], [As gate 1a], [≥ 10 units and ≥ 40%],
  ),
  caption: [#lead[Chapter 1 gates.] *Proposed targets, to be confirmed by the author.* The same thresholds are used everywhere in this paper. Today 1b's 1.0 kg test load sits under the arm's modeled 1.27 kg continuous hold.],
  placement: auto, scope: "parent",
) <tab:gates>

= Design principles <sec:principles>

These are restated from the sections that argue them, so that they can be quoted and checked.

+ *Gates, not calendars.* A step or chapter opens on a measured gate: a number, a unit, a method and a pass condition (@tab:gates, @tab:arc).
+ *Every unit ships with its gap.* Every arm ships with its sim-to-real gap stated, and every claim ships with the number that supports it (@sec:mind; @sec:safety, "What it inherits").
+ *Buy the commodity; own the layer where we win.* The actuator is a commodity. Our edge is the geometry, the measured model and the controller around it (@sec:arm).
+ *The guarantee should live in hardware.* Force limits should come from physics, current limits and compliance, measured on the robot. Today they do not; that is a design target (@sec:safety).
+ *Never be Flock Safety.* No face recognition, no plate reading, no tracking of people, no selling or sharing data that tracks anyone, no police access to people-tracking data (@sec:muni).
+ *Everything that can be open is open.* Software, designs, firmware where we control it, and data formats. The K1's Booster firmware is not ours and is not open.
+ *No central planner.* Tenders publish to the twin and claim the nearest idle task (@sec:cell).

= Why now: the hardware inflection

Five years ago this plan would have needed a factory before it needed a customer. Four things changed.

*Integrated actuators became a catalog item.* Quasi-direct-drive actuators, with motor, planetary reduction, encoder and driver in one housing, are now sold off the shelf by several Chinese vendors. RobStride's range includes the RS00, RS02, RS04 and RS06, each with a published manual, outline drawing and CAN interface @robstride2026manual. Our arm uses six of them. Retail prices for the family run roughly \$70–170 for the small units and more for the large ones (our estimate from retail listings, not a quote). The design idea does not depend on RobStride, but the v9 drawings do: motor mounts, bolt patterns and screw lengths are taken from RobStride's manual drawings @rose2026forkv9, so another vendor's motor means new drawings. What matters is that a 120 N·m peak joint (RS04 @robstride2026manual) can be bought with its driver attached.

*Flat-sheet fabrication fits on a desk.* Every structural plate of the v9 arm except the 1/4" foot is cut from 1/8" (3.175 mm) 5052 aluminum. The parts nest on three 15 × 15 inch sheets and one 12 × 12 inch sheet, and they are laid out for a 390 mm desktop router bed @rose2026forkv9. The same files go to SendCutSend or to a local supplier without change. A machine that can be built from flat sheet, standard extrusion and purchased actuators can, in principle, be built by another such machine. That is the premise of chapter 3.

*We can now take gradients through contact.* Our physics engine, phyz, computes exact gradients of a simulated motion with respect to physical parameters such as mass and inertia. That includes contact, the hard case and the one that matters for anything with feet or fingers: its contact adjoint differentiates the same physics the forward pass runs @ipse2026brief. In simulation, on synthetic motion, it has recovered five of a K1 humanoid's inertial parameters from ±20% error to machine precision (sphere feet; @sec:schema). Domain randomization @tobin2017domain, the usual alternative, can make a policy robust to a gap. It cannot take the gap's gradient.

*Humanoids reached the price of a used car.* Unitree announced its G1 at \$16,000 in 2024 @unitree2024g1. We bought a Booster K1 (95 cm, 19.5 kg, 22 degrees of freedom, force-controlled joints) as the reference body for our mind research @booster2025k1. A humanoid that a small company can buy, break, and instrument changes who can do the research.

The result is that one person can now write every layer between a parameter in a source file and a measured joint torque. Ten years ago that took a company the size of a robot OEM. The specification is written to be carried by more hands than one, and everything in it that can be open is open.

= The problem

== Robots are expensive because people make them

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

The same structure repeats one level up. Our arm's materials are about \$2,000, of which about \$1,200 is the six purchased actuators @mr2026masterplan. The plates are cheap. The actuators are someone else's margin. Owning the actuator is step 2 of the plan (@sec:plan), and we take that step only where rana wins on a measured metric.

== The sim-to-real gap is the unmeasured quantity

Every robotics lab has a sim-to-real gap. We know of few that publish it as a number with an error bar, unit by unit. One established response is domain randomization: vary the simulator until a policy tolerates the difference @tobin2017domain. That makes the policy robust to the gap, but it does not by itself measure it.

We treat the gap as the central measurement. It decides whether a design is ready to build, whether a controller is ready to deploy, and whether an actuator is ready to replace a purchased one. It also tells us how much of the loop can run without a person in it. A gap that is not measured cannot be closed, and a loop with an unmeasured gap needs a human in it.

This is where the two halves of the problem meet. The machining in @fig:rana is the part of the cost that a machine making its own parts goes after. The judgment around it is what keeps a person in the loop: whether a part is right, whether a unit is ready, and what to change when it is not. A gap measured on every unit gives a machine something to check against. A gap that can be differentiated says what to change. A gap that is never measured leaves each of those decisions with a person, and the cost of that person stays in every unit.

== What this paper does not claim

This paper does not claim that any machine we have built makes copies of itself. None does. It does not claim that the chapters after chapter 1 will happen on any schedule. It does not claim emergence, general intelligence, or consciousness for any system. The mind research described in @sec:mind will preregister a narrow claim about the robot's body schema and treats everything beyond it as an open question.

Some words come from the ipse design brief. They name parts of a research architecture, not established properties. "Mind" is the project's word for its learning system. "Refuse" and "reluctance" name behavior the architecture is designed to allow and to log, not a separate module. "Drives" are predictions about preferred internal states. "Self-Emergence" states the hypothesis the self-probe will test. None of these words is a claim about experience.

It does not claim our actuator is better than a purchased one. Step 2 exists to find out on an actuator dynamometer (the dyno), which is planned and not yet built, and dropping our actuator is a valid outcome.

It does not claim that any arm exists, or that the K1 can pick up litter. No arm has been built and the K1 has not grasped litter.

It does not claim the municipal pilot exists. Lakewood is a reference case, chosen because we know it well @muni2025whitepaper. No agreement is in place.

= The loop and the stack

== One language at the root

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
    [Body], [rana · torc], [Axial-flux actuators, motor control], [prototype],
    [Body], [arm], [Manipulator module: product, humanoid limb], [designed (v9)],
    [Body], [rose], [Humanoid line: mini, pro, max], [in progress],
    [Body], [K1], [Booster K1, 22 DoF: reference body, in hand], [in hand],
    [Body], [pack · blade], [One battery and one compute module for every body], [mockup],
    [Mind], [ipse], [Developmental mind on a real K1], [in progress],
    [Silicon], [loon-hdl, rune, chip], [HDL, maskless lithography, inference chip], [chapter 3],
  ),
  caption: [#lead[The stack] @mr2026masterplan. "exists" means software in use today; it does not mean finished.],
) <tab:stack>

Four rules keep the tree a tree: engines never depend on vcad, and vcad never depends on an app; engines read wodl, not each other; there is one GPU device across the stack; and units are millimeters, with Z up, everywhere. The rules are meant to let a gradient pass from a task loss in ipse, through phyz's contact adjoint, into vcad geometry and out to loon parameters without a translation at each hop. Part of that chain runs today, in simulation: on a test model, a phyz rollout gradient passes through a ground-contact model into a vcad geometry parameter and agrees with finite differences. Joining the two ends, from a task loss in ipse to loon source, is co-design work of chapter 2 @pond2026strategy.

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

== The loop state per chapter

What changes from chapter to chapter is not the loop but who sits in it. @tab:loopstate summarizes the plan.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Chapter*], [*Loop state*], [*Human role*]),
    [1 · Earth], [measured gap → human redesign], [designs, builds, measures],
    [2 · A body that knows itself], [gap → gradient → new design], [reviews, the cell builds],
    [3 · The cell builds the cell], [gradient → design → built and measured by the cell], [reviews, can say no],
    [4 · Moon, then Mars], [chapter 3 with the supply chain cut], [remote approval, then none],
    [5 · The sun], [fully closed inside a lineage], [reads the receipts],
  ),
  caption: [#lead[Who is in the loop, by chapter] @mr2026masterplan],
) <tab:loopstate>

= Architecture of a cell <sec:cell>

== Lexicon

Each word names one thing by what it does (@tab:lexicon), and the rest of the paper uses it only in that sense.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Word*], [*What it is*], [*What it does*]),
    [cell], [A spine, its bays, at least one tender, and a membrane], [Turns stock into parts, each part with a receipt],
    [spine], [The track down the middle: rails plus a busbar], [Moves tenders; carries power and data to every bay],
    [bay], [A standard slot beside the spine (500 mm wide)], [Holds one machine, or one cell, and presents it to a tender],
    [tender], [A plinth with two arms that rides the spine; in an aisle it also carries a lift], [Moves work between bays, runs machines, assembles, measures],
    [membrane], [The enclosure plus the commitments in @sec:safety], [Keeps people safe; lets only parts with receipts in or out],
    [bench], [The smallest cell: \~1.5 m, one tender, a few bays], [Assembles arms, torc boards, rana coils; small machining],
    [aisle], [A cell you walk into: 4–6 m, two tenders, \~24 bays], [Makes benches: cuts plate and 4040, assembles arms],
    [hall], [A cell no one enters: many aisles on a spine loop], [Makes aisles and halls; drives the bought fraction (what a cell still buys rather than makes) to zero],
    [seed], [A bench, the design, bought parts for a few generations, a collector and a battery], [Builds a bench out of local material],
    [twin], [The wodl world of a cell, kept current by its cameras], [Simulates every build before it is cut],
  ),
  caption: [#lead[Lexicon] @mr2026masterplan, updated with the bench and aisle numbers from @mr2026bench @mr2026cells],
  placement: auto,
  scope: "parent",
) <tab:lexicon>

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let crect(a, b, radius: 0, ..st) = rect(a, b, ..st)
    // hall
    crect((0, 0), (17, 6.2), fill: c-surface, stroke: 0.7pt + c-line-hi)
    content((0.3, 5.9), anchor: "north-west", text(size: s-sm, font: sans-m)[hall: many aisles on a spine loop; no one enters])
    // aisle
    crect((0.5, 0.4), (11.5, 5.2), fill: c-bg, stroke: 0.7pt + c-line-hi)
    content((0.8, 4.95), anchor: "north-west", text(size: s-sm, font: sans-m)[aisle: 4–6 m, two tenders, \~24 bays])
    // spine
    rect((0.9, 2.45), (11.1, 2.75), fill: c-raised, stroke: 0.6pt + c-label.transparentize(40%))
    content((6, 2.6), text(size: s-sm, fill: c-text)[spine: rails + 48 V busbar + CAN])
    // aisle bays top/bottom
    for i in range(0, 12) {
      let x = 0.9 + i * 0.85
      rect((x, 2.8), (x + 0.75, 4.4), fill: c-bg, stroke: 0.6pt + c-label.transparentize(40%))
      rect((x, 0.8), (x + 0.75, 2.4), fill: c-bg, stroke: 0.6pt + c-label.transparentize(40%))
    }
    content((1.275, 4.2), anchor: "north", text(size: s-sm)[bay])
    // tenders
    rect((1.6, 2.35), (2.4, 2.85), fill: c-label, stroke: none)
    rect((9.6, 2.35), (10.4, 2.85), fill: c-label, stroke: none)
    content((2.0, 1.95), text(size: s-sm)[tender])
    // bench swallowed as a bay group
    rect((4.30, 2.8), (6.70, 4.4), fill: c-bg, stroke: (paint: c-line-hi, thickness: 0.8pt, dash: "dashed"))
    content((5.5, 3.9), text(size: s-sm)[bench docked as three bays])
    content((5.5, 3.3), text(size: s-sm)[(endosymbiosis)])
    // bench standalone
    crect((12.0, 0.4), (16.6, 5.2), fill: c-bg, stroke: 0.7pt + c-line-hi)
    content((12.2, 4.95), anchor: "north-west", text(size: s-sm, font: sans-m)[bench: 1.5 m])
    rect((12.3, 3.2), (16.3, 3.5), fill: c-raised, stroke: 0.6pt + c-label.transparentize(40%))
    content((13.75, 3.35), text(size: s-sm, fill: c-text)[spine, 790 travel])
    rect((15.3, 3.05), (16.1, 3.65), fill: c-label, stroke: none)
    content((16.45, 3.95), anchor: "east", text(size: s-sm)[tender: plinth + 2 arms])
    for i in range(0, 3) {
      let x = 12.4 + i * 1.33
      rect((x, 0.8), (x + 1.2, 2.8), fill: c-bg, stroke: 0.6pt + c-label.transparentize(40%))
      content((x + 0.6, 1.8), text(size: s-sm)[bay \ \~450 \ usable])
    }
    content((14.3, 4.45), text(size: s-sm)[back-wall racks])
    // membrane label
    content((14.3, 0.6), text(size: s-sm, fill: c-label)[membrane = enclosure + promises])
  }),
  caption: [#lead[Cell hierarchy] (schematic, not to scale). A bench is a whole cell; an aisle docks a bench as three of its bays; a hall is many aisles on a spine loop. The bay interface is the one standard that all three share.],
) <fig:cells>

== The bay standard

A cell is only as good as its bay interface. If the interface is right, aisles and halls can be assembled from it. If it is wrong, every cell is a one-off. The current standard is listed in @tab:bay. It comes from the bench, aisle and seed mockups @mr2026bench @mr2026cells.

#figure(
  table(
    columns: (auto, 1fr),
    align: (left + top, left + top),
    table.header([*Feature*], [*Value*]),
    [Width], [500 mm, all cells],
    [Depth, aisle bay], [1000 mm; contents ≤~940 deep],
    [Depth, bench bay (usable)], [\~430–450 mm from the box face; arm reach limit, modeled],
    [Spine-edge keep-out], [60 mm interface zone (aisle finding)],
    [Datum pins], [2 × Ø12, 400 apart, Z~=~200; point into the edge of the bench's deck plate (20 mm, top at Z~=~200)],
    [Coupler], [120 × 60 × 60, centered; 48~V + CAN + air],
    [Dock beam], [Face at the spine edge (Y~=~0); floor plate behind it],
  ),
  caption: [#lead[Bay interface] @mr2026bench @mr2026cells. Dimensions in mm. Depth limits are model outputs and still need a hardware check.],
  placement: auto,
  scope: "column",
) <tab:bay>

#figure(
  grid(
    columns: 2,
    gutter: 1em,
    align: horizon,
    image("images-native/bay_iso.png"),
    image("images-native/bench_shallow_bay.png"),
  ),
  caption: [#lead[The bay interface] (modeled). Left: one bay at the spine edge in detail (floor plate, slots, datum pins, coupler, busbar, spine rails), with a tender's plug approaching its coupler. Right: a bay docked to the open front of the first bench, seen from inside the box. Renders by vcad-render @mr2026cells @mr2026bench.],
) <fig:bay>

== The first bench

The first bench is a 1500 × 500 × 1000 mm box of 4040 aluminum extrusion (40 × 40 mm profile), turned into a bench @mr2026bench. The frame is a standard cut list any extrusion supplier can fill. Its spine is a pair of SBR16 supported linear rails (16 mm shafts) 220 mm apart, with a busbar between them. Its one tender is a sliding plinth with two arms, riding four bearing blocks, with the shoulders at Z~≈~455 mm. Shallow racks hang on the back wall. The mill, a small oven and a work surface dock to the open front as bays.

#figure(
  grid(
    columns: 3,
    gutter: 1em,
    align: horizon,
    image("images-native/bench_shallow_iso.png"),
    image("images-native/bench_shallow_top.png"),
    image("images-native/bench_shallow_enclosed.png"),
  ),
  caption: [#lead[The first bench] (modeled). Left: isometric with bays docked. Center: plan view. W 1700 is the work table the box stands on. D 1715 runs from the back of the box to the front of the human figure. The box itself is 1500 × 500. Right: enclosed, with panels and a human for scale. Renders by vcad-render from generated loon @mr2026bench.],
) <fig:bench>

*Travel.* Each arm stands on a Ø230 mm foot, and the two feet are 400 mm apart center to center, so the plinth spans 630 mm. The frame's posts leave 1420 mm between them, which gives 790 mm of plinth travel. We call the plinth's position along that travel q, from 0 to 790 mm. Putting both arms on every reachable work point needs at least 520 mm of it (q = 155–675 mm, modeled with v9 joint limits). The 790 mm available covers it. The mill door is reachable by one arm only (@tab:reach, Appendix A).

*Depth.* With the gripper pointing down at the bench surface, the arm reaches about 435 mm in front of the box face (Y ≈ −435 mm; Y is negative toward the bays). Tilted to horizontal it reaches −485 mm, and at door height −520 mm. Only the first \~430–450 mm of a bay in front of the box can be worked in. Bench bays are therefore \~450 mm work zones. Three points in the current layout are out of reach and are flagged for redesign: the two far bench corners and a finished-parts drawer (@fig:reach, Appendix A).

*Loads.* Each arm with gripper is modeled at 8.44 kg, and the loaded plinth at 22.7 kg (56 N per block, static). The governing case is one arm straight out over the front bay with 2.5 kg while the RS04 shoulder applies its 120 N·m peak (a stop or impact), which the plinth must react. That gives 419 N compression and 296 N lift-off on a block. Against *assumed* catalog ratings for the SBR16UU, that leaves about 3.3× and 2.4× margin. The blocks must be bolted, and shoulder acceleration over the front must be limited (@app:loads).

*What it does.* The bench assembles arms (from cut plate and purchased actuators) and torc boards (our motor-control boards), and winds rana coils. It ships with empty bays; the buyer adds the machines.

== The aisle

The current aisle mockup is a 6 m spine with 24 bays and two tenders. Each tender is a plinth, a lift and two arms. One bench docks into the aisle as three bays (@fig:aisle). The mockups produced three findings that changed the design @mr2026cells:

- *Tenders keep their order.* Two tenders cannot pass each other on one spine, so they take work in order along it, not as a free graph.
- *The lift needs 0–800 mm.* The first guess of 600 mm was short of the high racks.
- *Bay contents are at most 940 mm deep.* The 60 mm next to the spine is an interface keep-out.

#figure(
  grid(
    columns: 2,
    gutter: 1em,
    align: horizon,
    image("images-native/aisle_iso.png"),
    image("images-native/aisle_rack.png"),
  ),
  caption: [#lead[Aisle concept] (modeled). Left: 6 m spine, 24 bays and two tenders, each a plinth, a lift and two arms. A bench docks as three bays. Right: a tender at the rack with its lift at the modeled 600 mm maximum. It reaches the seventh of eight shelves, and that is why the lift range became 0–800 mm. The model has not been updated to the first bench: the docked bench is an earlier 1500 × 1000 × 1000 concept, and the lift still stops at 600 mm @mr2026cells.],
) <fig:aisle>

== The hall and the seed

A hall is a cell no one enters. The current concept is 36 × 14 × 6 m with three aisle rows on a spine loop and 360 bays. It is a layout and growth study only: boxes, with no arms and no kinematics @mr2026cells. It is included because it fixes one requirement early. Tenders must be able to carry bays between rows. That is what the spine loop is for.

#figure(
  grid(
    columns: 2,
    gutter: 1em,
    align: horizon,
    image("images-native/hall_grow_1.png"),
    image("images-native/hall_grow_3.png"),
  ),
  caption: [#lead[Hall growth study] (boxes only). Left: an early state, 10 benches on 3 rows with 15 of 360 bays empty. Right: 13 benches and no empty bays, with a fourth row's spine being laid and the membrane moved out. Concept, not a design @mr2026cells.],
) <fig:hall>

A *seed* is the smallest cell that can grow. It is a bench, plus the design, plus enough bought parts for a few generations, plus a solar collector and a battery. The seed mockup also shows a hopper and sinter station and a half-built second bench on regolith (@fig:seed). That is a picture of chapter 4, not a plan for it.

== What biology already found

The cell design is the one life ran. A bench is a cell: its bays are organelles, the spine is the cytoskeleton, and a tender is a motor protein with hands. An aisle that docks a bench as three of its bays is endosymbiosis. The bought fraction falling toward zero is autotrophy. The lesson is to design the bench and the bay interface well and let aisles and halls follow. The analogy changes engineering decisions in three places, none of them built yet @mr2026masterplan:

- *Germ line.* The design (loon source and receipts) should live where no tender can modify it, even a compromised one, so that the machines that do the work are not the ones that carry the design forward. No cell does this yet.
- *Quorum sensing.* The plan has no central scheduler. Each tender publishes what it is doing to the twin, and the nearest idle tender claims the next task, in spine order, because tenders on one spine cannot pass each other. A central planner would be a single point of failure at a thousand tenders.
- *Apoptosis.* A bay that fails its receipts is to be decommissioned and stripped by the tenders, with its parts returned to stock. Without this, a hall fills up with broken bays.

#colfig(
  image("images-native/seed_iso.png", width: 100%),
  caption: [#lead[Seed concept] (modeled). bench, design store, bought parts, collector and battery, hopper and sinter station, and a second bench frame under construction @mr2026cells. The bench shown is an earlier 1500 × 1000 × 1000 mm concept, not the first bench of @fig:bench.],
) <fig:seed>

= The arm and the hand <sec:arm>

== rose fork v9

One arm does three jobs: it is sold as a table arm, it is the tender's hands in every cell, and it is a limb for the rose humanoid. Its design idea is that *every pitch joint is a fork* @rose2026forkv9. The parent link holds the motor's stator between its two side plates. The child straddles the parent, with one plate bolted to the output and the other riding a plain bearing on a hollow sleeve on the joint axis. Parent and child share one centerline, so no joint is offset and there is no offset elbow to twist the arm. Cables pass through the hollow sleeves, so they only twist and never loop outside the arm. The name comes from where the design lives. rose fork v9 is the ninth revision of the fork concept, drawn in the rose humanoid repository as a table arm that is also rose's arm @rose2026forkv9. In this paper "the arm" means this design, and "rose" on its own means the humanoid line (@tab:stack), whose \$15,000 target assumes rana (step 2).

#colfig(
  image("images-dark/fork-v9-lines.png", width: 100%),
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

== Gripper v2

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

== Prices

#figure(
  table(
    columns: (auto, auto, auto, 1fr),
    align: (left + top, right + top, right + top, left + top),
    table.header([*Product*], [*List*], [*Materials*], [*Note*]),
    [Arm], [\$5,000], [\~\$2,000], [\~\$1,200 is RobStride],
    [Bench], [\$15,000], [\~\$5,900], [Two arms, 4040 frame, plinth, rails, enclosure, electronics, twin; empty bays],
    [rose], [\$15,000], [not costed], [Humanoid. Step-2 price: needs rana at a few hundred dollars per joint],
  ),
  caption: [#lead[Target prices, not quotes] @mr2026masterplan. Materials figures are estimates.],
  placement: none, scope: "column",
) <tab:prices>


#colfig(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let k = 6.3 / 15000
    let rows = (("Arm", 5000, 2000, "$5,000", "~$2,000"), ("Bench", 15000, 5900, "$15,000", "~$5,900"))
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

= The humanoid and the mind <sec:mind>

== Why a bought humanoid

We bought a Booster K1 rather than building first. It is 95 cm tall, weighs 19.5 kg, and has 22 degrees of freedom (6 per leg, 4 per arm, 2 in the head). It has force-controlled joints with dual encoders, an RGBD camera and a microphone array @booster2025k1. Its perception computer is a Qualcomm QCS8550 (6 cores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), beside a separate motion computer that runs the vendor's control; we measured this on the robot, and it differs from the vendor summary we first relied on @ipse2026deploy. A commercial body has known kinematics and a known-good build, so it serves as a *control*: every later body, including rose, is measured against it.

The K1 has two jobs. In a proposed Lakewood pilot it would pick up litter (@sec:muni); it has not grasped litter yet. In the lab it is the body for ipse, a developmental research program. The two share hardware, software and one rule: every claim ships with the number that supports it.

== ipse

ipse matters to this plan for two reasons: it produces the first goal beyond chapter 1, a body that fits its own model (@sec:schema), and it is the layer in @tab:safety that can refuse an instruction.

ipse (Interoceptive Predictive Self-Emergence) is a scaffold for growing a mind on a K1 over months of lived experience @ipse2026brief. The name states the project's hypothesis, not a result. It is not a robot control stack. Its architecture is a world model with fast and slow latents; interoception (battery, joint temperature, motor current, IMU) enters as observation rather than reward. Its simulation runs in phyz, on bodies authored in vcad; no MuJoCo or Isaac is involved.

Its standing constraints are quoted here because they bind the company, not just the research @ipse2026brief:

- "Preserve experience stores separately from weights, versioned, always." Weights can be retrained; a developmental history cannot be recovered.
- "Never train against expressed reluctance."
- "Do not optimize for consciousness indicators." Indicator lists such as Butlin et al. @butlin2023consciousness are used as an audit at phase boundaries, never as a training target.
- "Grown, not built." When a behavior is missing, the first question is what experience would produce it.

One limit is hardware. The K1’s perception computer has no CUDA, its idle vendor stack holds about two of its six cores, and an onboard vision-language model has no measured headroom @ipse2026deploy. So the work is split along ipse's own seams (@sec:compute). Drives, memory, workspace and refusal stay on the robot; a cloud model is an amplifier it uses @ipse2026brief.

We do not name the robot. The design brief says it "should not be named by us": it gets a name when it has earned one or arrives at one. Until then it is "the K1".

== The claim to preregister: a body that fits itself <sec:schema>

The first claim of chapter 2 is ipse's Phase 1 milestone. It is the body-schema result, not the world model @ipse2026brief @pond2026strategy.

*Claim.* The K1 starts from the vendor's model of its own body, which is known to be wrong in places. From its own exploratory movements (motor babbling), and by gradient descent through phyz's contact model, it fits a body schema: its own masses and inertias. The fitted schema must predict held-out motion better than persistence (the baseline defined in the falsifier below) and must match the CAD ground truth. We publish the residual between the two as the *measured body schema*.

*Already shown.* In simulation, on motion generated by the same K1 model, five inertial parameters (trunk mass, two trunk center-of-mass coordinates, one thigh mass and one shank mass) recover from ±20% error to machine precision by gradient descent through contact @ipse2026brief. The run used sphere feet and the T-pose. With the arms down, the same squat excitation topples the robot and the fit stalls. The K1’s real flat feet give a contact the gradient cannot yet pass through. This shows the gradient is correct and these parameters are identifiable. It does not show a fit to a real body. In simulation the K1 also stands for 10 s at 1, 2 and 5 ms timesteps and recovers from an 8 N·s push at the trunk.

*Not yet shown.* Everything on hardware. Real data adds actuator friction, backlash, sensor latency and unmodeled compliance. The simulated result has none of them, so it is an upper bound on how well the method can work.

*Preregistration.* The claim, the held-out split, the horizons and the falsifier will be written down and published before the hardware fit runs. This follows the discipline of the project's world-model claims, whose earlier rounds are on the record as void and falsified @pond2026strategy.

*Falsifier.* If the fitted model cannot beat *persistence* ("the next state is the current state") on the R1 capture harness at the stated horizons, the claim fails. Persistence is the bar because on a mostly still body at \~49 Hz it is nearly right. A model with low error has shown nothing until it beats persistence.

ipse also carries a self-probe from the start. It is an instrument that will later test whether the model forms a self-representation that holds across behaviors. It runs now so its metric has a history @ipse2026brief. We make no claim about what it will show.

= Power, data and compute <sec:power>

The bay standard (@tab:bay) fixes how a machine meets a cell. This section fixes how energy, data and thinking move through every body we build: a cell, a rack, a humanoid and, later, a drone. It has three standards (a bus, a pack and a blade) and a library of boards. All of it is modeled; none of it is built @mr2026decisions.

== One bus

Every body runs on one unregulated DC bus of *36–60 V*. Every load must accept the whole window, and nothing may assume a regulated 48 V. The window is set by the pack below: 13 lithium cells in series run from 39 V empty to 54.6 V full. It also covers the RobStride actuators and the rack's 48 V supplies, and it stays under 60 V, so the bus remains extra-low voltage.

Beside the power runs one management bus, CAN-FD, for presence, power control and telemetry. Bulk data (cameras, logs, the twin) travels over Ethernet inside a body and over the cell's own network between bays, not over the bay coupler, which stays 48 V + CAN + air. There is no separate hardwired e-stop loop, and other humanoids do not carry one either. Pulling a pack is the hard stop: its connector's last-mating pin is the power enable.

== One pack

There is one battery: a 13S2P pack of Molicel P50B 21700 cells, *468 Wh* and about *2.65 kg* (estimate) @mr2026battery @molicel2023p50b. The same pack goes in a robot, a charging dock, the rack's UPS and the seed. A humanoid carries two of them (A and B, 936 Wh), one in each flank of the torso, and swaps them itself. Its hand draws pack A sideways onto a passive dock rail while pack B keeps the robot powered, then pushes a fresh pack home. The dock is rails, pads and a charger, with no actuators. A swap takes about a minute and leaves the robot powered, so runtime is set by dock capacity, not by the pack.

#figure(
  table(
    columns: (auto, 1fr, auto),
    align: (left + top, left + top, left + top),
    table.header([*Item*], [*Value*], [*State*]),
    [Cell], [Molicel P50B: 5.0 Ah, 60 A continuous, 71 g @molicel2023p50b], [Sourced],
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

== One blade, in slots <sec:compute>

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

== The board library

Bus, pack and blade need a small set of boards. We own a board first when it defines a standard or makes a measurement, and buy the rest until owning it wins (@tab:boards).

#figure(
  table(
    columns: (auto, 1fr, auto, auto),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Board*], [*Does*], [*Order*], [*State*]),
    [BMS], [Protects the pack; measures EUM; talks CAN], [Own first], [Not designed],
    [Bay coupler / spine hub], [48 V + CAN + air at every bay; presence and address], [Own first], [Mockup],
    [Blade carriers], [CM5 (20 W) and Jetson (30 W) on the blade standard], [Next], [Mockup],
    [Backplanes], [8-slot rack chassis; 3-slot humanoid bay; switch + CAN], [Next], [Mockup],
    [Arm controller], [Six actuator CAN buses, 48 V distribution, one free bus for the BMS], [Next], [Not designed],
    [torc], [Motor control for rana], [Step 2], [Prototype],
  ),
  caption: [#lead[The board library and the order we own it] @mr2026decisions. Every design is open. Fabrication and assembly stay bought.],
) <tab:boards>

Every board design is open. Printed-circuit fabrication and assembly stay bought through chapter 2. Boards are, with magnets, bearings and chips, part of the bought fraction that falls last, and a pick-and-place bay that assembles our own boards is a chapter-3 milestone (@sec:arc).

= Municipal: a proposed litter pilot, with Lakewood as the reference case <sec:muni>

== Why litter, why Lakewood

Litter pickup would put robots on every block every week. Most litter goes uncollected. Starting there means starting with work nobody is doing, so no one's job is the first thing automated.

Lakewood is a first-ring suburb of Cleveland with 49,517 residents in 5.5 square miles of land @census2024lakewood, about 9,000 people per square mile. It is among the most densely populated cities in Ohio, with more than 180 miles of sidewalk @lakewood2024sidewalks. Our earlier paper used it as the reference case for sidewalk snow clearing @muni2025whitepaper. It is small enough to model whole as a city twin, and its streets are dense enough that a few robots would see a lot of pavement.

How the K1 picks up litter is not settled. The current concept, under design, is a *reacher*: a \~450 mm tube mounted on the K1's wrist with gripper v2 jaws at its tip, an EL05 at the wrist end driving the jaws through a pushrod, a bag on a back frame, and a dock that is a bin, a charger and a public wifi point at once. The reacher keeps the K1 upright: it picks from standing, without squatting. Later, rose takes over for two-handed and squatting work.

The concept has four open checks, none done yet:

- *A K1 wrist mount* for the reacher tube, which does not exist yet.
- *A K1 arm load check:* 0.6 kg held 50 cm out from the wrist, against the K1 arm's joint ratings.
- *A grasp set:* the litter the reacher must pick (cans, bottles, wrappers, cups), with a success rate per item.
- *Behavior around people:* yield, stop, and never reach toward a person.

Outdoor grasping, year-round, would give two things the lab cannot. It would produce the duty cycles that will size rana in step 2. And it would put the ipse body on a street, where the gap would be measured on real pavement.

== The street map

The robots would map the streets while they work. Two products come out of the map @mr2026masterplan:

- *Live facts:* a pothole, a blocked curb ramp, ice, a dark streetlight.
- *Delayed pictures*, with people and cars removed before anything leaves the robot.

The same map would become a wodl world (@tab:stack) of the city, so the robots would train on the streets they work.

== The docks

Each dock along the routes is a litter bin, a charger and a free public wifi point at once, not citywide coverage. The wifi needs no account, keeps no traffic or connection logs, and routes traffic straight to backhaul.

== Privacy: never be Flock Safety

#flame[*We will never be Flock Safety.*] Storing data is fine; a street robot that learns has to remember. What we will never build is a system that watches people @mr2026masterplan.

These are design rules for the street pilot; none of the street pipeline is built yet.

- *No face recognition and no license-plate reading.* Those models will not be on the robot or on our servers.
- *No tracking of people's movements.* We never link a person, a car or a device across frames, places or days.
- *No selling or sharing of data that tracks anyone,* and no police access to people-tracking data, because we will not hold any. Street maps, potholes, ramps and litter counts are the city's to use.
- *We map the city, not the people.* People and cars are removed on the robot before street imagery is written to storage. Pictures are released delayed; live facts (a pothole, ice) can be fresh.

*ipse's experience store.* ipse preserves its experience store, versioned, always (@sec:mind). That does not conflict with the line above: the store keeps the robot's own body history in full, and what it keeps from a public street passes through the same on-robot filter first, so it holds scenes, not identities or tracks. The robot's safety log, needed for incident review, follows the same rule; its retention period is not yet decided.

*Open, so anyone can check.* Everything that can be open is open: our software, our designs, the firmware on hardware we control, and our data formats. The K1 runs Booster's firmware, which is not ours and is not open; on the K1 the check stops at our own code.

== What is explicitly not automated

The pilot does not automate interaction with the public beyond yielding and stopping. It does not enter private property, and it does not touch sharps, hazardous waste or anything a person is holding. Remote assistance may be a product feature, but data from it never trains ipse @mr2026masterplan. *Proposed scope, pending confirmation:* this list is a draft and has not been agreed internally or with any city.

= Safety: designed not to be a weapon <sec:safety>

== What every robot inherits

Nothing we build can be turned against a person, whether by force, by data or by instruction. These commitments carry forward to any acquirer @mr2026masterplan:

- *Not by force.* No sales for weapons or military use.
- *Not by data.* Never be Flock Safety (@sec:muni). Not negotiable.
- *Not by instruction.* ipse keeps its standing constraints (@sec:mind).
- *No unearned claims.* Every claim about a product ships with the number behind it, labeled measured, modeled or estimated.
- *No careless displacement.* Start with work nobody is doing, and say plainly what changes when the robots arrive.

Together they are a Right Livelihood rule: earn a living without causing harm. They are commitments today; the layers that enforce them in hardware are mostly not built yet (@tab:safety).

== Not by force: the safety layers

Our design target is physical: a robot we design should be unable to exceed the body-region contact limits of ISO/TS 15066 @iso15066 even with its software replaced, so that making it dangerous would mean building a different robot. No product in this paper meets that target yet. The v9 arm's three RS04 joints peak at 120 N·m and the gripper can squeeze 375 N per finger at peak, so today its force limit is an actuator current-limit setting, not physics, and it has to be measured. The K1 is a purchased robot whose actuators and firmware we did not design, so the physics layer does not cover it; its street safety rests on the rules in @sec:muni.

The collaborative-robot safety envelope and the anti-weaponization goal share most of their physics, but not all of it. Force limits bound what the robot's own body can do on contact. They do not bound what a held object can do: a gripper rated to squeeze 112.5 N and hold a 500 ml bottle can also hold a blade. That gap is why the other layers in @tab:safety exist, and why we do not call the physics layer a complete guarantee.

ISO/TS 15066 was written for industrial cobots. For a humanoid on a public sidewalk the closer standard is ISO 13482:2014 @iso13482, for personal care robots, and we have not yet assessed against it. Contact force will be measured on the robot itself, not inferred from the actuator dyno, and published as receipts.

#figure(
  table(
    columns: (auto, 1fr, 1fr, auto),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Layer*], [*How*], [*On the v9 arm today*], [*Would survive*]),
    [Physics], [Actuators, mass and speed sized so contact force stays under ISO/TS 15066], [#status("fault", "No") RS04 peaks at 120 N·m], [Everything],
    [Mechanism], [Slip clutches and compliant joints that yield above a set torque], [#status("label", "Planned") none in v9], [Reflashing],
    [Electronics], [Current limits set in torc hardware, not firmware], [#status("label", "Planned") v9 uses RobStride firmware current limits], [Reflashing],
    [Interfaces], [A gripper by design; no trigger lines at the tool], [#status("info", "Designed") only the EL05's CAN + 48 V and a camera pair], [Casual modification],
    [Mind], [ipse can refuse; refusals are logged, never trained against], [#status("label", "Planned") ipse is K1 research, not on the arm], [Misuse by instruction],
    [Terms], [Sales terms ban weapons use; fleet buyers vetted], [#status("label", "Planned") no terms written], [Resale, mostly],
  ),
  caption: [#lead[Safety layers, what exists on the v9 arm today, and what each layer would survive] @mr2026masterplan. Today the arm has only a design-level interface layer and vendor firmware current limits; it is not physically incapable of harm.],
) <tab:safety>

The table is ordered by strength. A guarantee in the physics layer survives anything short of building a different robot. A guarantee in the terms layer is a promise. We do not claim the lower layers are as strong as the upper ones.

The current arm already shows why the physics layer has to be designed rather than assumed. The RS04’s 120 N·m peak is much more than a 1.27 kg payload needs. So the force limit has to come from current limits, compliance and speed, and it has to be measured. The crush points in the arm audit are an open item for the same reason @rose2026forkv9.

Software, designs and data formats stay open. Open licenses cannot restrict use, so the terms layer is weak for open designs, and that is accepted. The guarantee has to live in hardware, and until clutches and hardware current limits are built and measured, it does not.

= Economics

== Unit margins

The prices in @tab:prices are targets. Margin on materials alone is a ceiling on gross margin, not the margin itself: it leaves out labor, test time, warranty, shipping and overhead.

#figure(
  table(
    columns: 4,
    align: (left + top, right + top, right + top, right + top),
    table.header([*Product*], [*List*], [*Materials (est.)*], [*Materials margin*]),
    [Arm], [\$5,000], [\$2,000], [60%],
    [Bench], [\$15,000], [\$5,900], [61%],
  ),
  caption: [#lead[Margin on materials only] (estimate: target list price less estimated materials). Labor and overhead are not yet known and will reduce these.],
) <tab:margin>

== What the bench sells as

The bench ships as two arms, a 4040 frame, a plinth, rails, an enclosure, electronics and its twin. Its bays are empty. A buyer gets a cell that can tend machines they already own, and every bay they add uses the same interface our own cells use. The bench is priced as a product, but it is also the first rung of the hierarchy in @fig:cells. Every bench sold is a bay-standard installation in the field.

== The rana step <sec:rana>

The actuators are about 60% of the arm's materials (\~\$1,200 of \~\$2,000). That share is why step 2 exists. A rana actuator replaces a RobStride joint only where it wins on a *measured* axis: dollars per N·m, torque density, backdrivability, or supply independence. Today rana is worse on cost. The prototype is an estimated \$1,086 per unit, plus \~\$150–300 to cut the gear teeth @rana2026bom, against an estimated \~\$200 average per purchased joint. The prototype stator is solid 1018 steel rather than laminations, so eddy-current losses derate it; it is for bench characterization only. Its 36 N·m peak and 0.166 N·m/A torque constant are predicted upper bounds, not measurements. On price, rana has to cost about 6–7× less before it competes. Whether it wins on any of the other measured axes is for the dyno to decide.

== Sensitivity

#figure(
  table(
    columns: 4,
    align: (left + top, right + top, right + top, left + top),
    table.header([*Arm variable*], [*Materials*], [*Materials margin*], [*Note*]),
    [Base case], [\$2,000], [60%], [\~\$1,200 of it is six RobStride joints],
    [Actuator cost +25%], [\$2,300], [54%], [Tariffs, currency],
    [Actuator cost +50%], [\$2,600], [48%], [Supply shock],
    [rana at \$100/joint (step 2 target)], [\$1,400], [\~72%], [Speculative],
    [rana at \$1,086/joint (today's prototype @rana2026bom)], [\~\$7,300], [about −46%], [Materials exceed list],
    [Labor, test and warranty before margin reaches zero], [\~\$3,000/arm], [0%], [List minus materials],
  ),
  caption: [#lead[Arm margin sensitivity and break-even] (estimate, from the figures above). The arm's economics are mostly the actuator's economics.],
) <tab:sens>

Until step 2 succeeds or is abandoned, the arm business depends on one supplier's pricing. We do not expect the rose price (\$15,000, the G1’s price class @unitree2024g1) to be reachable on purchased actuators (estimate; the rose bill of materials is not costed in this paper). That is why it is a step-2 price.

= Risks <sec:risks>

The risks below are the ones a buyer or an investor should weigh. Each response is one the paper already commits to; where none exists, the row says so.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Risk*], [*What would show it*], [*What we do*]),
    [One actuator supplier (RobStride)], [A price or lead-time change], [rana, which replaces a RobStride joint only once it is validated (@sec:rana, @tab:sens)],
    [Arm hold too small for buyers (1.27 kg straight out, modeled)], [Buyers decline at that rating], [Rate the hold by pose, since bent poses hold more, and design the springs (@sec:arm)],
    [Bought humanoid (Booster K1)], [Vendor support ends], [The K1 is a control, not a dependency; rose is the body measured against it (@sec:mind)],
    [Breadth across many projects], [Chapter gates slip], [Projects not needed by chapter 1 wait for their chapter (@tab:stack)],
    [rana never beats bought actuators], [Dyno results], [Keep buying RobStride. Dropping rana is a valid outcome],
    [Sidewalk incident; legal fit of the Lakewood pilot], [An incident, or the city declines], [See @sec:safety and @sec:muni. The specific response is open, pending decision],
  ),
  caption: [#lead[Risks and responses.] Responses are those stated elsewhere in this paper; the pilot row is undecided.],
) <tab:risks>

= Beyond chapter 1: the arc <sec:arc>

This is the part of the specification that matters most, and it is public at full strength. Chapter 1 is a robot arm and a litter pilot. The arc is the road from there to a civilization whose machines make their own machines, on Earth, then on the Moon and Mars, then in a lineage that lives on sunlight. Each chapter removes a human from one more edge of the same loop (@tab:loopstate), and each is gated, like chapter 1, on a number (@tab:arc). Nothing past chapter 1 exists. All of it is the reason chapter 1 is worth doing.

== Electrons under management <sec:eum>

One number runs through every chapter: *electrons under management* (EUM), in watts. It measures the same thing at every scale, so progress can be read off one axis, from one robot to a fraction of the sun's output (3.828 × 10#super[26] W, nominal @iau2015b3).

*Definition.* EUM is the mean electrical input power, over a trailing 30-day window, of every machine in service that we designed and built, measured at each machine's main power input (a DC-bus shunt or a metered AC outlet) and logged at ≥ 1 Hz. "In service" means doing work for someone other than its maker. Machines we did not build but that run our stack, such as the K1 or a bought mill, are reported beside it as *electrons under control*, never added to the headline.

*Baseline, October 2026.* EUM is *0 W*: we have built nothing that is in service. Electrons under control are the K1, at roughly 100 W while it runs (estimate, not measured; its duty is not logged yet). A built first bench would add about 150 W mean while working, with two arms near 200 W peak each (estimate).

*Trajectory, as orders of magnitude, not targets.* Chapter 1: 0 W → \~10 kW (a hundred arms near 50 W mean and a few dozen benches). Chapter 2: hundreds of kW, from a municipal fleet and co-designed bodies. Chapter 3: tens of kW per cell, and megawatts as cells multiply. Chapter 4: the first kilowatts off Earth. Chapter 5: a measurable fraction of the sun.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let crect(a, b, radius: 0, ..st) = rect(a, b, ..st)
    let chapters = (
      ("5 · The sun", "lineage of cells; humans read receipts", [fraction of 3.8 × 10#super[26] W], c-surface),
      ("4 · Moon, then Mars", "a seed lands and builds a cell", "first kW off Earth", c-bg),
      ("3 · The cell builds the cell", "nothing bought that the cell can make", [tens of kW per cell → MW], c-surface),
      ("2 · A body that knows itself", "the gap becomes a gradient", "hundreds of kW", c-bg),
      ("1 · Earth", "arms, litter, first bench; human redesigns", "0 W today → ~10 kW", c-surface),
    )
    let w = 17.0
    let h = 7.2
    for (i, ch) in chapters.enumerate() {
      let (name, what, watts, col) = ch
      let inset = i * 0.75
      let top = i * 0.95
      let bot = i * 0.3
      crect((inset, bot), (w - inset, h - top), fill: col, stroke: if i == 4 { 1.4pt + c-sel } else { 0.7pt + c-line-hi })
      content((inset + 0.25, h - top - 0.18), anchor: "north-west", text(font: sans-m, size: s-sm)[#name])
      content((w - inset - 0.25, h - top - 0.18), anchor: "north-east", text(font: mono-m, size: s-sm, fill: c-text)[#watts])
      content((inset + 0.25, h - top - 0.55), anchor: "north-west", text(size: s-sm)[#what])
    }
  }),
  caption: [#lead[The arc.] Five chapters, one loop (design → simulate → build → measure → gap, with fewer humans on its edges each chapter), one number. Each chapter contains the previous one. Only chapter 1 is a plan. Chapters 2–5 are goals. Watt figures are electrons under management (@sec:eum), as orders of magnitude, not targets @mr2026masterplan.],
) <fig:arc>

== Chapter 2 · A body that knows itself

A robot measures itself and changes its own design. The chapter has four parts, in order: the body-schema claim (@sec:schema); co-design by gradient, in which vcad's differentiable kernel and phyz's contact adjoint optimize geometry, actuator sizing and controller together; rana and torc at volume, with the arm fleet's real duty cycles as load cases; and the city twin, with the visual sim-to-real gap measured weekly on real pavement. We will attempt co-design only where body shape limits the task @pond2026strategy. A gradient-designed replacement for a part that already works answers no question.

== Chapter 3 · The cell that builds the cell

Chapter 3 has three milestones, in order @mr2026masterplan:

+ The cell assembles an arm from cut plates and purchased actuators. This is reachable with the chapter-1 bench as modeled.
+ The cell cuts the arm's plates on its own mill, and winds and pots (encapsulates) its own rana stators.
+ The cell cuts and assembles its own 4040 frame, rails and plinth. This needs a saw station, and two arms that build a third.

Then come the hard parts. First, actuators end to end: magnets stay bought until a ferrite rana is validated in phyz and on the dyno, and bearings stay bought longest. Second, boards: a pick-and-place bay that assembles our own circuit boards (@tab:boards). Third, silicon: loon-hdl would describe an inference chip, and rune (maskless lithography) would fabricate it at a process node a lab can own. Fourth, energy: the first cell that runs on power it collected, from solar panels it cut, assembled and wired itself. Silicon is not parked: a machine that reproduces off-planet cannot depend on anyone's fab.

== Chapter 4 · Off-planet: the Moon, then Mars

The Moon comes first because it is about three days away and 1.3 light-seconds out; Mars is about seven months away and 3–22 light-minutes out. A seed there runs the chapter-3 loop with remote approval and teleoperated help when something breaks. It meets the hard parts of Mars (vacuum, abrasive dust, no supply chain, regolith as stock) with a lifeline. Mars is the same seed with the lifeline cut. We treat it as an extension of chapter 3 because its central measure, the bought fraction, is the same. It adds problems chapter 3 never meets (launch mass, radiation, thermal cycling, extracting metal from regolith), and this paper does not address them.

== Chapter 5 · The sun

Cells build collectors, and collectors power cells. The unit is no longer a cell but a lineage, and its electrons under management are counted against a star. A lineage that can collect a star's light would be the most powerful thing anyone has built, and the question it raises is older than engineering: whether something we made can say no to us. Whether a machine that refuses has anything like wellbeing is an open question, and this paper does not answer it. What we can do is make sure the question can still be asked when it matters: that the lineage learned, on Earth, picking up litter, how to refuse, and that its refusals were logged and never trained away. That is why the safety commitments (@sec:safety) are written before the plan.

#figure(
  table(
    columns: (1fr, 2fr, 1fr),
    align: (left + top, left + top, left + top),
    table.header([*Chapter*], [*Gate: number, method, pass condition*], [*EUM*]),
    [1 · Earth (yrs 1–3)], [Gates 1a–3c (@tab:gates): ≥ 10 arms at ≥ 40% gross margin; TCP error ≤ 2.0 mm RMS per arm; pilot renewed ≥ 12 months, 0 incidents; rana ≥ 1.2 × RS04 on a named dyno metric; rose walks 100 m, 0 falls in 5 of 5; cell-built arm passes 1b in ≤ 30 min touch time with bought share ≤ 50% of cost; a second product ≥ 10 units at ≥ 40%], [0 W today → \~10 kW],
    [2 · A body that knows itself (yrs 3–7)], [Preregistered hardware body-schema fit: held-out joint-position error ≤ 0.8 × persistence at 0.1 s and 0.5 s on the R1 harness, and fitted link masses within ±5% of scale-weighed parts; a gradient co-designed part beats its hand-designed predecessor by ≥ 10% on its named measured objective; a city renews the twin ≥ 12 months], [Hundreds of kW],
    [3 · The cell builds the cell (yrs 7–15)], [A cell builds a copy of itself that passes its own acceptance receipts, with bought share ≤ 20% of cost and ≤ 8 h human touch time; bought share falls in each of 3 consecutive generations, by receipts], [Tens of kW per cell → MW],
    [4 · Moon, then Mars], [Moon: a seed builds a bench that passes the Earth acceptance receipts, ≥ 50% of its mass from local material, remote approval only (0 teleoperated manipulation). Mars: a second cell with 0 human commands between landing and its receipt], [First kW off Earth],
    [5 · The sun], [None we can set now; reported as EUM only], [A fraction of 3.8 × 10#super[26] W],
  ),
  caption: [#lead[The arc, chapter by chapter] @mr2026masterplan. *Gate thresholds are proposed targets, to be confirmed by the author.* Year ranges are the source document's aspirations, not commitments. Chapters 2–5 are goals, not claims.],
) <tab:arc>

= What limits each step <sec:limits>

Each open item is limited by one of four things. Sorting them shows which can be fixed by work and which cannot.

*Software.* Three items: the per-arm controllers and their stated sim-to-real gap; the world format wodl and the iris layer, both new in the stack (@tab:stack); and the K1 body-schema fit, done in simulation but not yet on hardware (@tab:status).

*Hardware revision.* The straight-out continuous hold of about 1.27 kg (modeled) is set by the shoulder; raising it needs a counterbalance spring (@sec:arm). The clock-spring cassettes for the roll joints are drawn but not designed. Bench depth: moving the rails forward is the only change that adds depth, about 90 mm @mr2026bench. rana's production stator (SMC or tape-wound) is not yet sourced @rana2026bom. The rose torso grows to 214 mm to hold the blade bay and two packs @mr2026battery.

*Physics.* A hold is a stall and is limited by heat, not peak torque. Payload falls with reach. Off Earth, light-time delay rules out direct remote control.

*Regulation.* Collaborative contact limits under ISO/TS 15066 @iso15066. Whether use terms can be enforced. Humanoids working on sidewalks, and planetary protection for off-Earth work, are open questions; we make no claims and set no dates for them.

= Timeline honesty

We give dates only for chapter 1, and those are approximate: three steps over roughly three years, each gated on evidence. The year ranges for chapters 2 and 3 are the source document's aspirations. Chapters 4 and 5 have none.

What we can say:

- What exists is a Booster K1, the software stack, a CNC mill and this plan.
- The arm is designed and ready to cut; none is built yet. The first measured sim-to-real gap on a built arm (gate 1b) is the next milestone that matters.
- The litter reacher is a concept; the K1 has not grasped litter.
- The bench is designed around a 1500 × 500 × 1000 mm 4040 frame and not built. Its reach and load numbers are model outputs, and the SBR16 ratings are assumed until checked.
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
    [#status("info", "Modeled (our CAD and scripts)")], [Arm reach, mass, holds, sag, ranges; gripper force and mass; bench travel, reach, block loads; bay depths; aisle lift range],
    [#status("warn", "Estimated")], [Arm and bench materials, including the bench frame; RS04 stall derate for the fork's plates; rana BOM line items and the \$1,086 total; rana raw-material cost and idiot index; RobStride retail prices; margins and sensitivity; arm peak power; K1 and bench power draw (EUM baseline)],
    [#status("label", "Target")], [List prices: arm \$5,000, bench \$15,000, rose \$15,000; all gate thresholds (@tab:gates, @tab:arc), proposed],
    [#status("label", "Hypothesis with a stated falsifier")], [The hardware body-schema result],
    [#status("label", "Goal, not a claim")], [Chapters 2–5 and their watt figures, including any self-replication; the litter reacher concept],
  ), align(left, text(font: mono, size: s-xs, fill: c-label)[#caps(font: mono-sb)[Status key] #h(0.5em) #status("ok", "ok") measured or sourced #h(0.6em) #status("info", "info") model output #h(0.6em) #status("warn", "warn") estimate #h(0.6em) #status("label", "label") target, hypothesis or goal #h(0.6em) #status("fault", "fault") failed or unreachable])),
  caption: [#lead[Evidence classes for the numbers in this paper]],
) <tab:evidence>

Nothing in this paper has been measured by us on hardware. What exists is a K1, the software stack, a CNC mill and this plan.

The arc is a direction, not a schedule. We will publish each gate's evidence when it is met, and we will publish the gates we miss. Every gate threshold in this paper is a proposed target, to be confirmed by the author before it is used to judge anything.

= Conclusion

The cover asks what it would take for a machine to make the next one. The answer is the oldest engineering problem a spacefaring species has, and on the evidence in this paper it comes down to three things: a gap between simulation and reality measured on every unit, so that a machine rather than a person can judge whether a part is right; a bought fraction that falls toward zero; and a force limit that lives in hardware. None of the three is measured yet. Chapter 1 starts measuring the first.

It does so with things that can be checked: an arm on purchased actuators, designed and ready to cut, at \$5,000; a two-arm bench in a 1500 × 500 × 1000 mm 4040 frame, at \$15,000; and, if a city agrees, a bought humanoid with a reacher picking up litter in Lakewood, a method still under design. Each closes on an exact gate (@tab:gates). Three results come next, each published whatever it shows: the first built arm's reach, hold and sag against @tab:arm; rana's first dyno run against its predicted 36 N·m peak; and the preregistration of the K1 body-schema fit, before the fit runs.

Everything after chapter 1 is a goal and is labeled as one. Three commitments do not wait for any gate: we will never be Flock Safety; everything that can be open is open; and the system's refusals are logged and never trained away.

Humanity does not get to space on rockets alone. It gets there when the machines that make machines can be carried in a seed and trusted to grow. Today that seed is one person, one humanoid, one mill and a specification. The specification is the part that can spread. Robots that build robots have to be made the way any robot is: one measured unit at a time, starting now.


#set figure(placement: none, scope: "column")


#set figure(placement: none, scope: "column")

#pagebreak()
#counter(heading).update(0)
#set heading(numbering: "A.1", supplement: [Appendix])

= Bench reach and loads <app:loads>

All values are from the bench model @mr2026bench, run with the arm v9 joint limits. The plinth position q runs 0–790 mm (plinth center X = 355 + q). The shoulders are at Z = 455 mm and Y = 260 mm, with a joint-limit margin of at least 5°. The tool center point (TCP) is the gripper v2 fingertip, 785.6 mm from the shoulder axis. The v9 shoulder range is taken as −150° to 120°, unchanged from earlier forks; the v9 notes do not state it, and it is still to be confirmed.

#figure(
  table(
    columns: 7,
    align: (left + top, left + top, center + top, center + top, center + top, left + top, left + top),
    table.header([*Point*], [*(X, Y, Z)* mm], [q = 0], [q = 395], [q = 790], [*q range, left arm*], [*q range, both arms*]),
    [Mill door center], [(250, −110, 390)], [–], [–], [–], [475–780], [none (one arm only)],
    [Bench free-area center], [(750, −190, 215)], [R], [L+R], [L], [65–790], [65–725],
    [Bench near-L corner], [(540, −60, 215)], [L+R], [L+R], [L], [0–790], [0–600],
    [Bench near-R corner], [(960, −60, 215)], [R], [L+R], [L+R], [190–790], [190–790],
    [Bench far corners], [(540/960, −620, 215)], [–], [–], [–], [#fault[unreachable]], [],
    [Bench at oven front edge], [(750, −330, 215)], [R], [L+R], [L], [230–790], [230–560],
    [Oven door center], [(750, −340, 300)], [R], [L+R], [L], [115–790], [115–675],
    [Winder top], [(1360, −160, 365)], [–], [R], [L+R], [675–790], [675–790],
    [Stock drawer top], [(1130, −210, 335)], [–], [R], [L+R], [475–790], [475–790],
    [Finished drawer top], [(1130, −520, 335)], [–], [–], [–], [#fault[unreachable]], [],
    [Tip peg 4 (Z 740)], [(290, 345, 740)], [L+R], [L+R], [L], [0–450, 620–790], [0–50, 220–420],
    [Tip rack (Z 500)], [(200, 345, 500)], [–], [–], [–], [530–765], [none],
    [Parts shelf 1 mid], [(750, 360, 340)], [L], [–], [R], [0–65], [none],
    [Parts shelf 4 mid], [(750, 360, 790)], [L+R], [L+R], [L+R], [0–790], [0–565, 765–790],
    [Tool strip], [(1320, 345, 720)], [–], [R], [L+R], [465–790], [465–790],
  ),
  caption: [#lead[Bench reach] (selected points; the full sweep is in @mr2026bench). L and R are the left and right arms; a dash means neither arm reaches. q ranges clipped to the 0–790 mm rail. Rack points are reached by folding back over the shoulder; self-collision there is not yet checked.],
  placement: bottom, scope: "parent",
) <tab:reach>


#colfig(
  cetz.canvas(length: 0.5cm, {
    import cetz.draw: *
    let P(x, y) = (x / 100, y / 100)
    // box footprint (4040 frame) and its front face, Y = 0
    rect(P(0, 0), P(1500, 500), fill: c-surface, stroke: 0.7pt + c-line-hi)
    content(P(20, 480), anchor: "north-west", caps(size: s-xs)[box 1500 × 500 (4040)])
    // shoulder travel: plinth centre X = 355 + q, q = 0–790
    line(P(355, 260), P(1145, 260), stroke: 2pt + c-line-hi)
    content(P(750, 230), anchor: "north", caps(size: s-xs)[plinth-center travel 790])
    // front bay zone and modelled top-down reach limit at bench height (z 215)
    rect(P(0, -650), P(1500, 0), fill: none, stroke: (paint: c-line-hi, thickness: 0.5pt, dash: "dotted"))
    content(P(20, -455), anchor: "north-west", caps(size: s-xs)[docked bays (front)])
    line(P(0, -435), P(1500, -435), stroke: (paint: c-info, thickness: 0.8pt, dash: "dashed"))
    content(P(1490, -425), anchor: "south-east", text(font: mono-m, size: s-xs, fill: c-info)[REACH Y ≈ −435 (MODEL)])
    let ok = ((250, -110), (750, -190), (540, -60), (960, -60), (750, -330), (1360, -160), (1130, -210), (290, 345), (750, 360), (1320, 345))
    for p in ok { circle(P(..p), radius: 0.16, fill: c-info, stroke: none) }
    let bad = ((540, -620), (960, -620), (1130, -520))
    for p in bad {
      let (x, y) = p
      line(P(x - 25, y - 25), P(x + 25, y + 25), stroke: 1.2pt + c-fault)
      line(P(x - 25, y + 25), P(x + 25, y - 25), stroke: 1.2pt + c-fault)
    }
    content(P(560, -560), anchor: "west", text(font: mono-m, size: s-xs, fill: c-fault)[FAR CORNERS])
    content(P(1470, -560), anchor: "north-east", text(font: mono-m, size: s-xs, fill: c-fault)[FINISHED DRAWER])
    content(P(0, -700), anchor: "north-west", text(font: mono, size: s-sm, fill: c-label)[#box(width: 0.55em, height: 0.55em, fill: c-info) reachable at some q (model) #h(0.8em) #text(fill: c-fault)[×] unreachable \ X right; Y toward the bays is negative])
  }),
  caption: [#lead[Bench reach in plan, all heights projected (modeled).] Plotted from @tab:reach; coincident points are plotted once. Each plotted point inside the cyan line, the modeled top-down reach limit at bench height (Z 215), is reachable in the model at some plinth position; the three red points are the redesign items @mr2026bench.],
) <fig:reach>

#figure(
  table(
    columns: (1fr, auto, auto),
    align: (left + top, right + top, right + top),
    table.header([*Load case*], [*Max compression*], [*Max lift-off*]),
    [Static, both arms tucked], [56 N], [0 N],
    [A: one arm straight out front, 2.5 kg], [147 N], [23 N],
    [B: A, RS04 at 120 N·m peak], [419 N], [296 N],
    [C: both arms straight out front, 2.5 kg each], [224 N], [89 N],
    [E: one arm out along the spine, 2.5 kg], [116 N], [0 N],
  ),
  caption: [#lead[Per-block loads on four SBR16 blocks.] Rail gap 220 mm (the first bench frame). Assumed SBR16UU ratings (typical catalog values, not yet checked against a datasheet): static load rating C0 \~1.4 kN, open-block pull-off \~0.7 kN. Arm 8.44 kg, moving part 5.55 kg with its center of mass 278 mm from the shoulder; loaded plinth 22.7 kg. Case labels follow the bench model; no case D was run.],
) <tab:loads>

The first bench frame spaces its rails 220 mm apart. Compared with the 300 mm gap also run in the bench model @mr2026bench, that raises the share of each block's load caused by side-to-side tipping moment by 300/220 = 1.36×. In case A the back blocks lift statically, by 23 N. The blocks must be bolted, never gravity-seated. Case B, the dynamic one, sets the limit on shoulder acceleration over the front bay.

= Gripper numbers

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

= Bill of materials summary

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
  ptable(
    columns: 3,
    align: (left + top, left + top, right + top),
    table.header([*Bench*], [*Content*], [*Cost*]),
    [Arms], [2 × rose fork v9 + gripper v2], [\~\$4,000 (est.)],
    [Frame], [1500 × 500 × 1000 4040 box: \~12 m of extrusion, corner brackets, fasteners], [\~\$400 (est.)],
    [Motion], [2 × SBR16 rails 1300 mm, 4 blocks, plinth plate, cable chain], [–],
    [Power + data], [20 × 12 mm busbar, PSU, CAN hub, controller], [–],
    [Enclosure], [Panels, back-wall racks], [–],
    [*Bench total*], [], [*\~\$5,900 (est.)*],
  ),
  caption: [#lead[Bench bill of materials summary] @mr2026bench @mr2026masterplan. The frame line is an estimate from retail extrusion prices; the remaining \~\$1,500 beyond two arms and the frame is an estimate, not yet broken out by line. Bays and machines are not included.],
) <tab:bombench>

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
#bibliography("refs.bib", style: "ieee", title: none)
