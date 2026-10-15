// Whitepaper: Robots that build robots (Municipal Robotics master plan)
// Styled to the muon "Pilot Light" treatment, dark theme with the neon accent set, translated to print.
// Tokens: frame-bootc/design/agency/treatment/tokens.toml [dark] + palette.toml [neon] + [type].

#import "@preview/cetz:0.4.2"
#import "@preview/fletcher:0.5.7" as fletcher: diagram, node, edge

// One source, two outputs. `typst compile --features html --format html --input target=html`
// builds the web post; without the input it builds the PDF.
#let web = sys.inputs.at("target", default: "pdf") == "html"
// math has no HTML form yet: draw it as inline SVG
// (inside a drawn figure the equation is already part of an SVG: mathfig marks those with alt "in-frame")
#show math.equation: it => if web and it.alt != "in-frame" { html.elem("span", attrs: (class: "math"), html.frame(it)) } else { it }
// math layout from New Computer Modern Math; words and numerals in Plex, to match the text
#show math.equation: set text(font: "New Computer Modern Math", fallback: true)
#show math.equation: it => { show regex("[A-Za-z0-9]"): set text(font: "IBM Plex Sans"); it }
#set math.equation(numbering: "(1)")

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

// ─── The product line: families, colors, and the elements of the periodic table ───
// Families wear the paper's neons.
#let fam = (
  make:    (name: "Make",    color: c-ok),
  world:   (name: "World",   color: c-info),
  body:    (name: "Body",    color: c-fault),
  power:   (name: "Power",   color: c-warn),
  mind:    (name: "Mind",    color: c-sel),
  silicon: (name: "Silicon", color: c-label),
)
// (number, name, family, maturity, note, first stage that needs it)
#let elements = (
  (1, "loon", "make", "exists", "source for parts, worlds, chips", "desk"),
  (2, "tang", "make", "exists", "math, autodiff, one GPU", "desk"),
  (3, "vcad", "make", "exists", "CAD kernel with gradients", "desk"),
  (4, "kerf", "make", "exists", "design to part, with a receipt", "desk"),
  (5, "wodl", "world", "concept", "world format", "shop"),
  (6, "kosm", "world", "exists", "runs wodl worlds", "desk"),
  (7, "phyz", "world", "exists", "physics, contact adjoint", "desk"),
  (8, "iris", "world", "concept", "renderer, robot cameras", "shop"),
  (9, "rana", "body", "designed", "axial-flux actuator", "desk"),
  (10, "torc", "body", "designed", "motor control", "desk"),
  (11, "limb", "body", "designed", "the arm: product, half a pair", "desk"),
  (12, "hand", "body", "designed", "485 g gripper", "desk"),
  (13, "pair", "body", "concept", "two limbs, any table", "shop"),
  (14, "Rose", "body", "concept", "humanoid: mini, full, mega", "road"),
  (15, "moth", "body", "concept", "street-perception drone", "city"),
  (16, "seed", "body", "concept", "a pair, packed to go", "moon"),
  (17, "pack", "power", "concept", "one battery, every body", "road"),
  (18, "core", "power", "concept", "one compute module", "road"),
  (19, "nerv", "power", "concept", "bus, hub and BMS boards", "road"),
  (20, "Ipse", "mind", "exists", "developmental mind", "road"),
  (21, "frog", "mind", "exists", "coding agent for local models", "desk"),
  (22, "loom", "silicon", "concept", "hardware description", "home"),
  (23, "rune", "silicon", "concept", "maskless lithography", "home"),
  (24, "chyp", "silicon", "concept", "inference chip", "home"),
)
// product names colored wherever they appear in running text
#let brand-names = elements.filter(e => e.at(1) not in ("hand",)).map(e => (lower(e.at(1)), e.at(2)))
#let brand-color(name) = { let m = brand-names.find(n => n.at(0) == name); if m != none { fam.at(m.at(1)).color } }
#let brand-family(name) = { let m = brand-names.find(n => n.at(0) == lower(name)); if m != none { m.at(1) } }
// every product name takes its family color, in any case: limb, Limb and LIMB alike
#let brand-re = regex("(?i)\\b(" + brand-names.map(n => n.at(0)).sorted(key: n => -n.len()).join("|") + ")\\b")
#metadata(brand-names) <brand-names>
// on the web, scripts/build-posts.mjs wraps names in spans (Typst cannot put HTML inside SVG frames)
#show brand-re: it => if web { it } else { text(fill: fam.at(brand-family(it.text)).color, it) }

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
// Maturity: four pips, concept · designed · built · measured
#let pips(n, sim: false) = box(baseline: 0.1em, {
  for k in range(4) {
    let on = k < n
    let col = if k == 3 and sim { c-info } else if on { c-text } else { none }
    box(width: 0.62em, height: 0.62em, radius: 50%, fill: if k == 3 and sim { none } else { col }, stroke: if k == 3 and sim { 0.8pt + c-info } else if on { none } else { 0.6pt + c-line-hi })
    if k < 3 { h(0.28em) }
  }
})
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
  // DRAFT across every page until the plan is released
  foreground: place(center + horizon, rotate(-32deg, text(font: mono-sb, size: 128pt, tracking: 0.12em, fill: c-sel.transparentize(93%), "DRAFT"))),
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
// in HTML the number is its own span, so the stylesheet can hang it like the PDF does
#let web-num(pattern) = if web { (..n) => html.elem("span", attrs: (class: "sec-num"), numbering(pattern, ..n)) } else { pattern }
#set heading(numbering: web-num("1.1"))
#show strong: set text(font: sans-sb)
#show link: set text(fill: c-text)
#show heading: set text(weight: "regular", fill: c-text)
#let numlabel(it) = text(font: mono-m, size: s-xs, fill: c-label, tracking: track, { if it.numbering == "A.1" and it.level == 1 { upper("Appendix ") }; counter(heading).display(it.numbering) })
#show heading.where(level: 1): it => if web { it } else if it.supplement == [Part] { block(above: 0em, below: 0.9em, text(font: sans-sb, size: 26pt, tracking: -0.02em, it.body)) } else { block(above: 2.2em, below: 1.1em, sticky: true, width: 100%, {
  if it.numbering != none { place(left, dx: -1.15in, dy: 0.32em, box(width: 1.0in, align(right, numlabel(it)))) }
  text(font: sans-sb, size: s-lg, tracking: -0.01em, it.body)
}) }
#show heading.where(level: 2): it => if web { it } else { block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, {
  if it.numbering == none { return text(font: sans-m, size: s-md, it.body) }
  set par(justify: false)
  place(left, dx: -1.15in, dy: 0.22em, box(width: 1.0in, align(right, numlabel(it))))
  text(font: sans-m, size: s-md, it.body)
}) }

#show heading.where(level: 3): it => if web { it } else { block(above: 1.65em, below: 0.55em, inset: (bottom: 0.3em), sticky: true, text(font: sans-sb, size: body-size, it.body)) }
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
// Tables: sans cells with tabular figures, a medium first column as the row anchor,
// mono caps only in the header; hairlines between rows, a firmer rule top and bottom
#set table(stroke: (x, y) => (
    top: if y == 0 { 0.6pt + c-line-hi } else if y == 1 { 0.6pt + c-line-hi } else { 0.35pt + c-line },
    bottom: 0.6pt + c-line-hi,
  ), inset: (x: 6pt, y: 4.6pt))
#show table: set text(font: sans, size: s-sm, number-width: "tabular", hyphenate: false)
#show table: set par(leading: 0.62em, justify: false)
#show table: it => { show strong: set text(font: sans-sb); it }
#show table.cell.where(x: 0): it => if it.y == 0 { it } else { set text(font: sans-m, fill: c-text); it }
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

// All figures sit in the text flow, after the paragraph that introduces them; none float to a page top.
// Wide figures break out of the column; colfig marks a narrow one by its caption gap (1.09em).
#set figure(placement: none, scope: "column", gap: 1.1em)
#show figure: set block(above: 1.9em, below: 1.9em)
#show figure: set text(hyphenate: false, font: sans, weight: 400)
#show table: set text(weight: 400)
#show heading: set text(font: sans, weight: 400, hyphenate: false)
#set place(clearance: 3em)
// short lead phrase of a caption, set in Medium
#let lead(body) = text(font: sans-m, fill: c-text, body)
// an equation beside the drawing that explains it
#let mathfig(eq, art, cap, ratio: (1fr, 1.15fr)) = figure({
  set math.equation(alt: "in-frame", numbering: none)
  set text(size: 1.2em)
  grid(columns: ratio, column-gutter: 0.4in, align: (center + horizon, center + horizon), eq, art)
}, caption: cap)
// an aside: a boxed digression the reader may skip
#let aside(title, body) = if web { html.elem("aside", attrs: (class: "aside"), { html.elem("div", attrs: (class: "aside-title"), title); body }) } else {
  pad(left: 0.35in, right: 0.35in, block(width: 100%, inset: (left: 12pt, y: 2pt), stroke: (left: 0.6pt + c-line-hi), above: 2.2em, below: 2.2em, breakable: false, {
    set text(size: s-sm, fill: c-label)
    set par(spacing: 1.1em, leading: 0.68em)
    show strong: set text(fill: c-text)
    block(below: 0.9em, text(font: mono-m, size: s-xs, fill: c-sel, tracking: track, upper([Aside · ] + title)))
    body
  }))
}
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
// Part art: one illustration per part, full width across the lower page, bleeding off the sides.
#let art-why() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  // stars
  let seed = 11
  for k in range(90) {
    seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
    let x = calc.rem(seed, 2160) / 100 - 10.8
    seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
    let y = calc.rem(seed, 700) / 100 + 1.2
    circle((x, y), radius: 0.01 + calc.rem(k, 3) * 0.007, fill: c-label.transparentize(35%), stroke: none)
  }
  // Earth's limb: only the visible cap of a huge circle, so it does not set the canvas size
  let R = 40
  let top(x) = -R + 1.0 + calc.sqrt(R * R - x * x)
  let xs = range(0, 221).map(k => -11 + k * 0.1)
  let cap = xs.map(x => (x, top(x)))
  for (w, a) in ((14pt, 93%), (7pt, 86%), (3pt, 70%)) { line(..cap, stroke: (paint: c-info.transparentize(a), thickness: w)) }
  line(..cap, (11, -9), (-11, -9), close: true, fill: c-surface, stroke: none)
  line(..cap, stroke: 0.7pt + c-info.transparentize(25%))
  // sunrise at the limb, right
  let sx = 7.8
  let sy = top(sx)
  for (r, a) in ((1.6, 94%), (0.9, 88%), (0.45, 72%), (0.2, 30%), (0.08, 0%)) { circle((sx, sy), radius: r, fill: c-sel.transparentize(a), stroke: none) }
  // a flare along the limb
  let flare = range(0, 61).map(k => { let x = sx - 3 + k * 0.1; (x, top(x)) })
  line(..flare, stroke: (paint: c-sel.transparentize(55%), thickness: 1.6pt, cap: "round"))
  // city lights on the night side
  seed = 5
  for k in range(260) {
    seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
    let x = calc.rem(seed, 1900) / 100 - 9.5
    seed = calc.rem(seed * 1103515245 + 12345, 2147483648)
    let d = calc.rem(seed, 300) / 100
    let y = top(x) - 0.15 - d * d * 0.5
    if calc.rem(k, 7) < 3 and x < 5.5 {
      circle((x, y), radius: 0.012 + calc.rem(k, 4) * 0.006, fill: c-warn.transparentize(30% + calc.rem(k, 5) * 10%), stroke: none)
    }
  }
})

#let art-what() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  // blueprint dot grid
  for gx in range(-10, 11) { for gy in range(0, 8) { circle((gx, gy), radius: 0.012, fill: c-line-hi, stroke: none) } }
  // the limb, side elevation
  let base = (-4.5, 0.3)
  rect((base.at(0) - 1.0, 0.1), (base.at(0) + 1.0, 0.3), fill: none, stroke: 0.7pt + c-text)
  let pts = ((-4.5, 0.3), (-4.5, 1.4), (-1.6, 5.2), (2.4, 4.4), (3.6, 3.2), (4.3, 2.4))
  for k in range(pts.len() - 1) {
    let (a, b) = (pts.at(k), pts.at(k + 1))
    line(a, b, stroke: 4.5pt + c-surface)
    line(a, b, stroke: 0.8pt + c-text)
  }
  for (k, q) in pts.slice(1).enumerate() {
    circle(q, radius: if k < 3 { 0.32 } else { 0.22 }, fill: c-bg, stroke: 0.8pt + c-text)
    circle(q, radius: 0.06, fill: c-text, stroke: none)
  }
  // gripper
  let g = pts.at(5)
  line(g, (g.at(0) + 0.5, g.at(1) - 0.3), stroke: 0.8pt + c-text)
  line(g, (g.at(0) + 0.3, g.at(1) - 0.55), stroke: 0.8pt + c-text)
  // construction lines and a dimension
  line((-4.5, 1.4), (6, 1.4), stroke: (paint: c-info.transparentize(55%), thickness: 0.4pt, dash: "dashed"))
  line((-4.5, 1.4), (-4.5, 7), stroke: (paint: c-info.transparentize(55%), thickness: 0.4pt, dash: "dashed"))
  circle((-4.5, 1.4), radius: 4.9, fill: none, stroke: (paint: c-info.transparentize(65%), thickness: 0.4pt, dash: "dotted"))
  line((-4.5, 0.85), (4.6, 0.85), stroke: 0.5pt + c-info, mark: (start: "straight", end: "straight", size: 0.12))
  content((0, 0.95), anchor: "south", text(font: mono-m, size: 5.5pt, fill: c-info)[786])
  content((-1.6, 5.75), anchor: "south", text(font: mono-m, size: 5.5pt, fill: c-info)[RS04])
  content((3.6, 3.75), anchor: "south-west", text(font: mono-m, size: 5.5pt, fill: c-info)[RS02])
  content((6.5, 0.2), anchor: "south-east", text(font: mono-m, size: 5.5pt, fill: c-label, tracking: track)[LIMB FULL · ROSE FORK V9])
})

#let art-how() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let names = ("Desk", "Shop", "Road", "City", "Home", "Moon", "Mars", "Star")
  let w = 2.1
  let g = 0.18
  let x0 = -(8 * w + 7 * g) / 2
  for (k, n) in names.enumerate() {
    let x = x0 + k * (w + g)
    let h = 0.5 + k * 0.78
    let first = k == 0
    rect((x, 0), (x + w, h), fill: if first { c-sel } else { c-surface }, stroke: if first { none } else { 0.6pt + c-line-hi })
    content((x + 0.12, h + 0.12), anchor: "south-west", text(font: mono-m, size: 5.5pt, fill: if first { c-sel } else { c-label }, tracking: track, upper(n)))
  }
  // the sun above the last step
  let sx = x0 + 7 * (w + g) + w / 2
  let sy = 0.5 + 7 * 0.78 + 1.3
  for (r, a) in ((1.5, 92%), (0.9, 84%), (0.5, 60%), (0.28, 0%)) { circle((sx, sy), radius: r, fill: c-sel.transparentize(a), stroke: none) }
})

#let art-well() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  // axes
  line((-8, 0), (8, 0), stroke: 0.6pt + c-line-hi)
  line((-8, 0), (-8, 5.5), stroke: 0.6pt + c-line-hi)
  let pred(x) = 2.6 + 1.6 * calc.sin(x * 0.55) + 0.12 * x
  let meas(x) = pred(x) + 0.55 * calc.sin(x * 1.7 + 0.6) + 0.18 * x / 8 + 0.25
  let xs = range(0, 161).map(k => -7.6 + k * 0.095)
  // the gap, shaded
  for k in range(xs.len() - 1) {
    let (a, b) = (xs.at(k), xs.at(k + 1))
    line((a, pred(a)), (a, meas(a)), stroke: 0.6pt + c-warn.transparentize(80%))
  }
  line(..xs.map(x => (x, pred(x))), stroke: (paint: c-label, thickness: 0.8pt, dash: "dashed"))
  // measured points with error bars
  for k in range(0, 161, step: 10) {
    let x = xs.at(k)
    let y = meas(x)
    line((x, y - 0.22), (x, y + 0.22), stroke: 0.5pt + c-text)
    circle((x, y), radius: 0.07, fill: c-text, stroke: none)
  }
  content((7.9, pred(7.9) - 0.35), anchor: "north-east", text(font: mono-m, size: 5.5pt, fill: c-label, tracking: track)[PREDICTED])
  content((7.9, meas(7.9) + 0.4), anchor: "south-east", text(font: mono-m, size: 5.5pt, fill: c-text, tracking: track)[MEASURED])
  content((0, 5.6), anchor: "south", text(font: mono-m, size: 5.5pt, fill: c-warn, tracking: track)[THE GAP = MEASURED − PREDICTED])
})

#let part(n, title, art: none, bleed: false, lede) = if web {
  html.elem("section", attrs: (class: "part", id: "part-" + lower(n)), {
    html.elem("p", attrs: (class: "part-label"), "Part " + n)
    heading(level: 1, numbering: none, supplement: [Part], outlined: true, title)
    html.elem("p", attrs: (class: "part-lede"), lede)
    if art != none { html.elem("div", attrs: (class: "part-art"), html.frame(block(width: 8.5in, height: 4.2in, clip: true, fill: c-bg, align(center + bottom, art)))) }
  })
} else {
  pagebreak(weak: true)
  if art != none {
    if bleed { place(bottom + center, dy: 1.05in, box(width: 8.5in, height: 6.4in, clip: true, align(center + bottom, art))) } else { place(bottom + center, dy: -0.1in, box(width: 8.5in, height: 5.2in, clip: true, align(center + bottom, art))) }
  }
  v(1.2in)
  block(below: 0.6em, text(font: mono-m, size: s-xs, fill: c-label, tracking: track, upper("Part " + n)))
  heading(level: 1, numbering: none, supplement: [Part], outlined: true, title)
  block(width: 80%, text(size: s-md, fill: c-label, lede))
  pagebreak()
}

// Column-width figure that stays in flow
// Wide figures break out of the text column into both margins; their captions stay on the text measure
// debug: report each figure's page and y when built with --input figpos=1
#show figure: it => { if sys.inputs.at("figpos", default: "0") == "1" { context [#metadata((page: here().position().page, y: here().position().y.pt(), kind: repr(it.kind), lab: if it.has("label") { str(it.label) } else { "" }, cap: if it.caption != none { repr(it.caption.body).slice(0, calc.min(60, repr(it.caption.body).len())) } else { "" })) <figpos>] }; it }
// Web figures: drawn bodies become inline SVG via html.frame; plain tables stay real <table>s.
#let web-framed-tables = ("tab:status", "tab:safety", "tab:evidence", "tab:flywheel")
#let web-figure(it) = {
  let lab = if it.has("label") { str(it.label) } else { "" }
  let wide = it.gap != 1.09em
  let framed = it.kind != table or lab in web-framed-tables
  let body = if framed { html.frame(block(width: if wide { 7.1in } else { 4.8in }, fill: c-bg, inset: 6pt, { set math.equation(alt: "in-frame"); it.body })) } else { it.body }
  let cap = if it.caption != none { html.elem("figcaption", [#html.elem("span", attrs: (class: "fig-label"))[#{ if it.kind == table [Table] else [Fig] } #context it.counter.display(it.numbering)] #it.caption.body]) }
  html.elem("figure", attrs: (id: lab, class: if wide { "wide" } else { "narrow" }), if it.kind == table { cap; body } else { body; cap })
}
#show figure: it => if web { web-figure(it) } else if it.gap != 1.09em { block(above: 1.9em, below: 1.9em, pad(x: -1.15in, top: 2.0em, bottom: 0em, { show figure.caption: c => pad(x: 1.15in, align(left, c)); it })) } else { it }
#show figure.where(kind: table): it => if web { it } else { show table: t => align(left, t); it }

#let colfig(body, caption: none) = figure(body, caption: caption, placement: none, scope: "column", gap: 1.09em)

// Diagram node fills: grey structure only
#let n-fill = c-surface
#let n-stroke = 0.7pt + c-line-hi

// Title page (single column). The one flame: the trajectory in the cover art.
#if not web {page(columns: 1, margin: (x: 0.75in, top: 0.7in, bottom: 0.65in))[
  // Cover art: the inner solar system in a tilted plan view, an Earth → Mars → Jupiter
  // gravity-assist trajectory, and the Sun–Earth Lagrange points. Schematic, not to scale.
  #place(top + left, dx: -0.75in, dy: -0.7in, box(width: 8.5in, height: 11in, clip: true, place(center + horizon, dy: 1.9in, cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let cx = 0
    let cy = 0
    let tilt = 0.5
    let zoom = 2.05
    let P(r, a) = (cx + zoom * r * calc.cos(a), cy + zoom * tilt * r * calc.sin(a))
    // orbits fade with distance: the far half (y > 0) is dim and drawn behind the sun
    let orbit(r, w, col, far: true) = {
      let n = 72
      for k in range(n) {
        let a = k * 360deg / n
        let b = (k + 1) * 360deg / n
        let mid = (a + b) / 2
        let back = calc.sin(mid) > 0
        if back == far {
          let depth = (1 - calc.sin(mid)) / 2
          line(P(r, a), P(r, b), stroke: (paint: col.transparentize(78% - 70% * depth), thickness: w * (0.6 + 0.8 * depth)))
        }
      }
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
    let ocol(k) = if k == 2 { c-label } else { c-line-hi.lighten(30%) }
    for (k, r) in rs.enumerate() { orbit(r, if k == 2 { 0.8pt } else { 0.55pt }, ocol(k), far: true) }
    // the sun, with a soft glow
    for (k, g) in ((0.95, 92%), (0.7, 85%), (0.5, 72%), (0.36, 50%)).enumerate() {
      circle((cx, cy), radius: g.at(0), fill: c-sel.transparentize(g.at(1)), stroke: none)
    }
    circle((cx, cy), radius: 0.26, fill: c-sel, stroke: none)
    for (k, r) in rs.enumerate() { orbit(r, if k == 2 { 0.8pt } else { 0.55pt }, ocol(k), far: false) }
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
    line(..traj.slice(0, 40), stroke: (paint: c-sel.transparentize(80%), thickness: 9pt, cap: "round", join: "round"))
    line(..traj, stroke: (paint: c-sel, thickness: 1.1pt, cap: "round", join: "round"))
    // planets
    let planet(r, a, rad, col, name, dx: 0.18, dy: 0.18, anchor: "south-west") = {
      let q = P(r, a)
      circle(q, radius: rad * 2.4, fill: col.transparentize(85%), stroke: none)
      circle(q, radius: rad, fill: col, stroke: none)
      if name != "" { content((q.at(0) + dx, q.at(1) + dy), anchor: anchor, text(font: mono-m, size: 5.5pt, fill: c-label, tracking: 0.08em, upper(name))) }
    }
    planet(1.15, 120deg, 0.05, c-label, "")
    planet(1.85, 60deg, 0.075, c-text, "")
    planet(2.75, a0, 0.085, c-info, "Earth", dx: -0.12, dy: 0.2, anchor: "south-east")
    planet(3.85, a1, 0.07, c-fault, "Mars", dx: 0.0, dy: 0.2, anchor: "south")
    planet(7.9, 118deg, 0.16, c-warn, "")
    // moon of Earth
    let e = P(2.75, a0)
    circle((e.at(0) + 0.2, e.at(1) + 0.07), radius: 0.025, fill: c-text, stroke: none)
    // Sun–Earth Lagrange points (L1, L2 exaggerated off Earth)
    let lp(q, name, dx, dy, anchor) = {
      let (x, y) = q
      line((x - 0.07, y), (x, y + 0.07), (x + 0.07, y), (x, y - 0.07), close: true, fill: none, stroke: 0.7pt + c-info)
    }
    lp(P(2.4, a0), "L1", 0.12, 0.0, "west")
    lp(P(3.1, a0), "L2", -0.12, 0.0, "east")
    lp(P(2.75, a0 + 180deg), "L3", 0.14, 0.0, "west")
    lp(P(2.75, a0 + 60deg), "L4", 0.12, -0.04, "north-west")
    lp(P(2.75, a0 - 60deg), "L5", 0.0, 0.14, "south")
    // trajectory notes
    let m = P(3.85, a1)
    let j = traj.at(84)
    content((j.at(0) - 0.2, j.at(1) + 0.25), anchor: "south-east", text(font: mono-m, size: 5.5pt, fill: c-sel, tracking: 0.08em)[TO THE OUTER SYSTEM →])
  }))))
  #set par(leading: 0.5em)
  #grid(columns: (auto, 1fr, auto), column-gutter: 0.8em, align: horizon,
    caps(fill: c-text)[Municipal Robotics · Whitepaper],
    line(length: 100%, stroke: 0.5pt + c-line-hi),
    caps[Rev 1.0 · October 2026])
  #v(1.6em)
  #image("images-native/muni-logo.svg", width: 0.95in)
  #v(1.1em)
  #move(dx: -0.045 * s-display, text(font: sans-sb, size: s-display, tracking: -0.02em)[Robots that\ build robots])
  #v(0.5em)
  #line(length: 22%, stroke: 2pt + c-label)
  #v(0.7em)
  #text(size: s-lg, fill: c-label)[What would it take for a machine to make the next one?]
  #v(1fr)
]}

// Abstract: one text, two layouts
#let abstract-text = [
  Every machine still passes through human hands. This is a plan for machines that make the next machine, each one measured, until the bought fraction falls to zero and a seed of them can go where no supply chain reaches. It starts on a sidewalk in Ohio.

  Robots are expensive because people make them. Our prototype actuator is estimated at \$1,086 to have built; about two thirds of that is outsourced machining, on perhaps \$60–90 of metal. A machine that makes more of itself attacks that markup. A design process that measures every unit's sim-to-real gap, and feeds it back into the design as a gradient, attacks the rest.

  The plan is built from one part. A limb is a six-axis arm on purchased actuators (\$5,000 list), designed and ready to cut. Two limbs at any table make a pair (\$12,000, target) that can fold laundry or build the next limb; Rose, our humanoid, is a pair on legs. One stack of our own carries it, from a source language for parts to a physics engine that can be differentiated through contact. Its key result, so far in simulation only: a humanoid's inertial parameters recovered from ±20% error to machine precision.

  Eight stages climb from there, each a step up in scale and each built by the one before: Desk, Shop, Road, City, Home, Moon, Mars and Star. Only Desk is a plan, with exact gates: ten limbs sold at 40% gross margin, each under 2 mm RMS tool-point error. The rest are goals, and they are the point. Progress is one number, electrons under management: the power drawn by machines we built, in service. Today it is zero watts. Every number here is cited, or labeled as an estimate or a model output.
]
#if web {
  html.elem("section", attrs: (class: "abstract", id: "abstract"), {
    html.elem("h2", attrs: (class: "abstract-title"), [Abstract])
    abstract-text
  })
} else {page(columns: 1, margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))[
  #block(width: 4.9in)[
  #heading(outlined: false, numbering: none)[Abstract]
  #set text(size: 10pt)
  #set par(leading: 0.8em, spacing: 1.25em)

  #abstract-text
]
]}

// Web contents: the outline as a linked list, parts as labels
#if web {
  context {
    let hs = query(heading.where(outlined: true)).filter(h => h.level <= 2)
    html.elem("nav", attrs: (class: "toc", id: "contents", "aria-label": "Contents"), {
      html.elem("h2", attrs: (class: "toc-title"), [Contents])
      html.elem("ol", for h in hs {
        let cls = if h.supplement == [Part] { "toc-part" } else if h.level == 1 { "toc-l1" } else { "toc-l2" }
        // the right column starts at Road, as in the PDF
        if h.has("label") and h.label == <sec:mind> { cls = cls + " toc-col" }
        let num = if h.numbering != none and h.supplement != [Part] { html.elem("span", attrs: (class: "toc-num"), numbering(if h.supplement == [Appendix] { "A.1" } else { "1.1" }, ..counter(heading).at(h.location()))) }
        html.elem("li", attrs: (class: cls), link(h.location(), [#num#html.elem("span", attrs: (class: "toc-text"), h.body)]))
      })
    })
  }
}

// Table of contents (single column)
#if not web {page(columns: 1, margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))[
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
]}

#part("I", "Why", art: art-why(), bleed: true)[Where this comes from, why it matters, and the whole plan on a few pages.]

= Intro <sec:story>

#block(inset: (left: 0em), {
  set text(size: 10.5pt)
  set par(leading: 0.95em, spacing: 1.5em)
  [Dear reader,

  I started Municipal Robotics in Cleveland in December 2025 to build one machine for one job: a small rover that clears snow from sidewalks @muni2025whitepaper. By the end of that month it drove. #text(fill: c-warn)[[TODO: Founders Inc: when I joined, what it gave me, and what came of it.]]

  Building it taught me the lesson this paper rests on. The expensive part of a robot is not its parts. It is the people who design, build and check each one. To build one rover I wrote a language for parts, a CAD kernel, a physics engine you can differentiate, and a way to record everything it did. None of them cared that the thing was a rover. Pointed at a humanoid, or at a pair of arms that builds more arms, they worked the same way.

  So I retired the rover #text(fill: c-warn)[[TODO: when, and why, in your words]] and kept what mattered: the city, the sidewalk, and the habit of plain claims and sourced numbers.

  What follows is a plan for machines that build machines, from one arm on an Ohio table to a swarm of collectors around the sun. Most of it is not built. Today there is one person, one bought humanoid, a CNC mill, a stack of software, and these pages. I have written it down anyway, at full strength, with every gate stated and every number labeled, because a plan this large is only worth trusting if it is honest about where it stands.

  Read it to check it, or to join it.]
  v(1.4em)
  let sig = { text(font: sans-m, size: s-md)[Cam Pedersen]; linebreak(); text(font: mono-m, size: s-xs, fill: c-label, tracking: track)[#upper[Cleveland · October 2026]] }
  if web { html.elem("p", attrs: (class: "signature"), [#html.elem("span", attrs: (class: "sig-name"))[Cam Pedersen] #html.elem("span", attrs: (class: "sig-place"))[Cleveland · October 2026]]) } else { sig }
})

#pagebreak(weak: true)

= Life <sec:kardashev>

In 1964 the astronomer Nikolai Kardashev proposed ranking civilizations by the power they command @kardashev1964. A Type I civilization uses the energy available on its planet, a Type II its star's, and a Type III its galaxy's. Carl Sagan turned the types into a continuous scale for a power $P$ in watts, which puts Type I at 10#super[16] W and Type II at 10#super[26] W @sagan1973cosmic:

$ K = (log_10 P - 6) / 10 $ <eq:kardashev>


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
    circle((X(es), 0.6), radius: 0.34, fill: c-sel, stroke: none)
    content((X(es) - 0.45, 0.6), anchor: "east", text(font: mono-m, size: 5.5pt, fill: c-text)[THE SUN])
    // the earth, at a planet's energy
    circle((X(16), 0.6), radius: 0.17, fill: rgb("#2f7bff"), stroke: none)
    content((X(16) + 0.28, 0.6), anchor: "west", text(font: mono-m, size: 5.5pt, fill: c-text)[THE EARTH])
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
    let steps = (("fire, farming", "food → muscle"), ("steam", "heat → work"), ("electricity", "work, anywhere"), ("transistor", "control, almost free"), ("intelligence", "machines that make machines"))
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
  caption: [#lead[Each climb made one thing cheap.] The next is intelligence: machines that make the machines.],
) <fig:stairs>

Getting off the planet is now mostly that problem. Launch has become cheap. Reusable rockets cut the cost of reaching low Earth orbit from about \$54,500 per kg on the Space Shuttle to about \$2,700 per kg on a Falcon 9 @csis2022launch. What stays expensive is everything a settlement needs when it arrives, and the people needed to make it. A civilization does not climb toward Type II by launching factories. It climbs by sending a seed that can grow one.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    // x: first flight year 1960-2030; y: log10 $/kg, 2 to 5
    let W = 15.5
    let H = 6.0
    let X(yr) = (yr - 1960) / 70 * W
    let Y(v) = (calc.log(v, base: 10) - 2) / 3 * H
    // grid
    for e in (2, 3, 4, 5) {
      line((0, Y(calc.pow(10, e))), (W, Y(calc.pow(10, e))), stroke: 0.35pt + c-line)
      content((-0.15, Y(calc.pow(10, e))), anchor: "east", text(font: mono, size: 5.5pt, fill: c-label)[\$10#super[#str(e)]])
    }
    for yr in (1960, 1970, 1980, 1990, 2000, 2010, 2020, 2030) {
      content((X(yr), -0.2), anchor: "north", text(font: mono, size: 5.5pt, fill: c-label, str(yr)))
    }
    content((W, -0.6), anchor: "north-east", text(font: mono, size: 5.5pt, fill: c-label)[FIRST FLIGHT · USD PER KG TO LOW EARTH ORBIT (LOG)])
    // vehicles (approximate, inflation-adjusted, CSIS)
    let v = (
      (1967, 5400, "Saturn V", "north-east", 0.12, -0.12),
      (1981, 54500, "Space Shuttle", "south-west", 0.15, 0.1),
      (1996, 9000, "Ariane 5", "south-west", 0.15, 0.08),
      (2002, 13000, "Delta IV Heavy", "south-west", 0.15, 0.08),
      (2010, 2720, "Falcon 9", "north-east", -0.15, -0.1),
      (2018, 1410, "Falcon Heavy", "north-east", -0.15, -0.1),
    )
    let reusable = (2010, 2018)
    // trend from shuttle to falcon heavy
    line((X(1981), Y(54500)), (X(2010), Y(2720)), (X(2018), Y(1410)), stroke: (paint: c-sel.transparentize(30%), thickness: 1pt))
    line((X(2018), Y(1410)), (X(2028), Y(200)), stroke: (paint: c-sel.transparentize(40%), thickness: 1pt, dash: "dashed"))
    for (yr, val, name, anc, dx, dy) in v {
      let q = (X(yr), Y(val))
      circle(q, radius: 0.1, fill: if reusable.contains(yr) { c-sel } else { c-text }, stroke: none)
      content((q.at(0) + dx, q.at(1) + dy), anchor: anc, text(size: s-xs, fill: c-text, name))
    }
    // starship target, hollow
    let q = (X(2028), Y(200))
    circle(q, radius: 0.12, fill: c-bg, stroke: 1pt + c-sel)
    content((q.at(0) - 0.15, q.at(1) + 0.1), anchor: "south-east", text(size: s-xs, fill: c-label)[Starship, target])
    content((X(1999), Y(25000)), anchor: "west", text(font: mono-m, size: 5.5pt, fill: c-sel)[REUSE])
  }),
  caption: [#lead[Cost to reach low Earth orbit, per kg, by first flight] (approximate, inflation-adjusted @csis2022launch). Filled orange: reusable boosters. The hollow point is a stated target, not a result. Launch fell about fortyfold from the Shuttle to Falcon Heavy. The cost of what a settlement needs on arrival has not moved.],
) <fig:launch>

This plan gives that climb one number of its own, electrons under management (@sec:eum): the power drawn by machines we built, in service, measured on the same axis as the Kardashev scale. Today it is zero.

== Smart defaults <sec:defaults>

We build arms and humanoids not because they are impressive, though they are. We build them because Earth has been a reinforcement-learning environment for about 3.7 billion years @dodd2017evidence. Its reward is simple: what survives and copies itself is kept. Every body alive today is a policy that passed that test, and the shapes it settled on are the best defaults we have (@fig:life).

#figure(
  align(center, cetz.canvas(length: 0.8cm, {
    import cetz.draw: *
    // years ago, log scale: 10^10 on the left, today on the right
    let W = 17.0
    let X(t) = (10 - calc.log(t, base: 10)) / (10 - 5.6) * (W - 1.6)
    line((0, 0), (W, 0), stroke: 0.7pt + c-line-hi, mark: (end: "straight", size: 0.14))
    for e in (10, 9, 8, 7, 6) {
      let x = X(calc.pow(10, e))
      line((x, -0.08), (x, 0.08), stroke: 0.6pt + c-line-hi)
      content((x, -0.2), anchor: "north", text(font: mono, size: 6pt, fill: c-label)[10#super[#str(e)] yr ago])
    }
    content((W, -0.2), anchor: "north-east", text(font: mono-m, size: 6pt, fill: c-sel, tracking: track, upper[today]))
    let body = fam.body.color
    let events = (
      (3.7e9, [cell], [copies itself], "pair", [divides, by license], 1),
      (3.75e8, [limb], [out of the water], "limb", [six axes, any table], 1),
      (5.5e7, [hand], [grasping primates], "hand", [485 g gripper], 1),
      (4.0e6, [upright], [hands freed], "Rose", [a pair on legs], 2),
      (3.3e6, [tools], [hands make things], "Shop", [machines make machines], 1),
    )
    for (t, life, what, ours, ourwhat, lvl) in events {
      let x = X(t)
      let up = 0.55 + (lvl - 1) * 1.15
      circle((x, 0), radius: 0.09, fill: c-text, stroke: none)
      line((x, 0.12), (x, up - 0.08), stroke: 0.5pt + c-line-hi)
      content((x, up), anchor: "south", box(align(center, { text(font: sans-sb, size: 8.5pt, fill: c-text, life); linebreak(); text(font: sans, size: 6.5pt, fill: c-label, what) })))
      let dn = -0.85 - (lvl - 1) * 1.15
      line((x, -0.55), (x, dn + 0.1), stroke: (paint: body, thickness: 0.6pt, dash: (2pt, 2pt)))
      content((x, dn), anchor: "north", box(align(center, { text(font: sans-sb, size: 8.5pt, fill: if ours == "Shop" { c-text } else { body }, ours); linebreak(); text(font: sans, size: 6.5pt, fill: c-label, ourwhat) })))
    }
    content((-0.3, 0.75), anchor: "east", text(font: mono-m, size: 6pt, fill: c-label, tracking: track, upper[life]))
    content((-0.3, -1.05), anchor: "east", text(font: mono-m, size: 6pt, fill: c-label, tracking: track, upper[ours]))
  })),
  caption: [#lead[Smart defaults.] Earth's long experiment, on a log scale of years, and the part of this plan that copies each result. Dates are approximate: the earliest evidence of life @dodd2017evidence, limbs @daeschler2006devonian, the first stone tools @harmand2015lomekwi.],
) <fig:life>

The first default was not a body but a cell: a unit that makes a copy of itself. Everything since was built from cells. Our cell is the pair, two limbs at a table that can build the next pair. Like a cell, it divides, though only with a license (@sec:replication).

The later defaults are the ones we copy directly. Limbs came out of the water about 375 million years ago @daeschler2006devonian. Grasping hands came with the primates, upright walking freed them, and about 3.3 million years ago hands began making tools @harmand2015lomekwi. There is a practical reason too. The world people built is shaped for those defaults: stairs, door handles, sidewalks, tables and tools all assume two arms, two hands and a body about our size. A machine with that shape can use what is already here.


#if not web { pagebreak(weak: true) }
== Why now <sec:why>

Five years ago this plan would have needed a factory before it needed a customer. Four things changed.

#figure(
  align(center, cetz.canvas(length: 1.35cm, {
    import cetz.draw: *
    // exploded actuator, oblique: discs along one axis
    let parts = (("driver", 0.0, 1.55, c-raised), ("encoder", 1.1, 1.55, c-surface), ("stator", 2.2, 1.9, c-raised), ("rotor", 3.3, 1.9, c-surface), ("planetary", 4.4, 1.7, c-raised), ("output", 5.5, 1.3, c-surface))
    for (k, (n, x, r, col)) in parts.enumerate() {
      let c = (x, 1.9)
      // a disc seen edge-on at an angle: ellipse
      circle(c, radius: (0.45 * r, r), fill: col, stroke: 0.7pt + c-label)
      if n == "stator" { for a in range(12) { let t = a * 30deg; circle((x + 0.45 * r * 0.75 * calc.cos(t), 1.9 + r * 0.75 * calc.sin(t)), radius: (0.06, 0.12), fill: c-text, stroke: none) } }
      if n == "planetary" { for a in range(3) { let t = a * 120deg + 90deg; circle((x + 0.45 * 0.9 * calc.cos(t), 1.9 + 0.9 * calc.sin(t)), radius: (0.12, 0.3), fill: none, stroke: 0.6pt + c-text) } }
      if n == "output" { circle(c, radius: (0.16, 0.36), fill: c-sel, stroke: none) }
      content((x, 1.9 - r - 0.2), anchor: "north", text(font: mono-m, size: 6.5pt, fill: c-label, upper(n)))
    }
    line((-0.8, 1.9), (6.4, 1.9), stroke: (paint: c-info.transparentize(50%), thickness: 0.4pt, dash: "dashed"))
    content((6.6, 1.9), anchor: "west", text(font: mono-m, size: 7pt, fill: c-text)[CAN · 48 V])
  })),
  caption: [#lead[An actuator, from a catalog.] Motor, planetary reduction, encoder and driver in one housing, exploded. Schematic.],
) <fig:actuator>

*Integrated actuators became a catalog item.* Quasi-direct-drive actuators, with motor, planetary reduction, encoder and driver in one housing, are now sold off the shelf by several Chinese vendors. RobStride's range includes the RS00, RS02, RS04 and RS06, each with a published manual, outline drawing and CAN interface @robstride2026manual. A limb uses six of them. Retail prices for the family run roughly \$70–170 for the small units and more for the large ones (our estimate from retail listings, not a quote). The design idea does not depend on RobStride, but the v9 drawings do: motor mounts, bolt patterns and screw lengths are taken from RobStride's manual drawings @rose2026forkv9, so another vendor's motor means new drawings. What matters is that a 120 N·m peak joint (RS04 @robstride2026manual) can be bought with its driver attached.

#figure(
  align(center, cetz.canvas(length: 1.0cm, {
    import cetz.draw: *
    // timeline
    let y = 0
    line((0, y), (8.2, y), stroke: 0.7pt + c-line-hi, mark: (end: "straight", size: 0.14))
    let ev = ((0.3, "DEC 2025", "bvr0 drives"), (2.5, "2026", "the stack grows"), (4.7, "2026", "K1 bought"), (6.9, "OCT 2026", "this plan"))
    for (x, d, t) in ev {
      circle((x, y), radius: 0.07, fill: c-text, stroke: none)
      content((x, y - 0.18), anchor: "north", text(font: mono-m, size: 7pt, fill: c-label, d))
      content((x, y - 0.48), anchor: "north", text(size: 7pt, fill: c-text, t))
    }
    // one stack fanning out
    let sx = 0.3
    rect((sx - 0.25, 1.0), (sx + 1.75, 3.6), fill: c-surface, stroke: 0.6pt + c-line-hi)
    let layers = ("loon", "vcad", "phyz", "kerf")
    for (i, l) in layers.enumerate() {
      rect((sx - 0.1, 1.15 + i * 0.6), (sx + 1.6, 1.6 + i * 0.6), fill: c-raised, stroke: none)
      content((sx + 0.75, 1.375 + i * 0.6), text(font: mono-m, size: 7pt, fill: c-text, l))
    }
    content((sx + 0.75, 3.75), anchor: "south", text(font: mono-m, size: 7pt, fill: c-label)[ONE STACK])
    let outs = ((3.25, "rover", "retired", c-label), (2.3, "humanoid", "Rose, K1", c-text), (1.35, "pair", "builds limbs", c-text))
    let x = 5.6
    for (ty, n, w, col) in outs {
      line((sx + 1.75, 2.3), (x - 1.0, ty), stroke: (paint: c-label, thickness: 0.6pt, dash: if n == "rover" { "dotted" } else { none }), mark: (end: "straight", size: 0.12))
      rect((x - 1.0, ty - 0.38), (x + 1.4, ty + 0.38), fill: if n == "rover" { none } else { c-surface }, stroke: if n == "pair" { 1pt + c-sel } else { 0.6pt + c-line-hi })
      content((x - 0.85, ty), anchor: "west", text(font: sans-m, size: 8.5pt, fill: col, n))
      content((x + 1.3, ty), anchor: "east", text(size: 7pt, fill: c-label, w))
    }
  })),
  caption: [#lead[One stack, three bodies.] Tools written to build one rover do not care what they design. Pointed at a humanoid, or at a pair of arms that builds arms, they work the same way.],
) <fig:story>

*Flat-sheet fabrication fits on a desk.* Every structural plate of the v9 arm except the 1/4" foot is cut from 1/8" (3.175 mm) 5052 aluminum. The parts nest on three 15 × 15 inch sheets and one 12 × 12 inch sheet, and they are laid out for a 390 mm desktop router bed @rose2026forkv9. The same files go to SendCutSend or to a local supplier without change. A machine that can be built from flat sheet and purchased actuators can, in principle, be built by another such machine. That is the premise of Shop.

#figure(
  align(center, cetz.canvas(length: 0.85cm, {
    import cetz.draw: *
    // the v9 arm's plates, nested on its real sheet set: three 15 in sheets and one 12 in
    let link(x0, y0, x1, y1, w) = {
      let dx = x1 - x0
      let dy = y1 - y0
      let L = calc.sqrt(dx * dx + dy * dy)
      let nx = -dy / L * w / 2
      let ny = dx / L * w / 2
      line((x0 + nx, y0 + ny), (x1 + nx, y1 + ny), stroke: 0.6pt + c-text)
      line((x0 - nx, y0 - ny), (x1 - nx, y1 - ny), stroke: 0.6pt + c-text)
      arc((x1 + nx, y1 + ny), start: calc.atan2(nx, ny), delta: -180deg, radius: w / 2, stroke: 0.6pt + c-text)
      arc((x0 - nx, y0 - ny), start: calc.atan2(-nx, -ny), delta: -180deg, radius: w / 2, stroke: 0.6pt + c-text)
      for q in ((x0, y0), (x1, y1)) {
        circle(q, radius: w * 0.22, fill: c-bg, stroke: 0.5pt + c-text)
        for a in range(6) { let t = a * 60deg; circle((q.at(0) + w * 0.34 * calc.cos(t), q.at(1) + w * 0.34 * calc.sin(t)), radius: 0.025, fill: c-text, stroke: none) }
      }
    }
    let S = 3.6
    let sheets = (
      (0.0, S, ((0.55, 0.55, 3.05, 0.55, 0.7), (0.55, 1.4, 3.05, 1.4, 0.7), (0.55, 2.25, 2.2, 2.25, 0.6), (0.55, 3.05, 2.2, 3.05, 0.6), (2.85, 2.1, 2.85, 3.1, 0.55))),
      (S + 0.35, S, ((0.55, 0.55, 3.05, 0.55, 0.7), (0.55, 1.4, 3.05, 1.4, 0.7), (0.55, 2.3, 3.05, 2.3, 0.7), (0.55, 3.1, 2.4, 3.1, 0.5))),
      (2 * (S + 0.35), S, ((0.5, 0.5, 2.0, 0.5, 0.55), (0.5, 1.25, 2.0, 1.25, 0.55), (0.5, 2.0, 2.0, 2.0, 0.55), (0.5, 2.75, 2.0, 2.75, 0.55), (2.7, 0.5, 2.7, 3.1, 0.6))),
      (3 * (S + 0.35), S * 12 / 15, ((0.5, 0.5, 2.3, 0.5, 0.55), (0.5, 1.3, 2.3, 1.3, 0.55), (0.5, 2.15, 1.7, 2.15, 0.5))),
    )
    for (i, (x, w, parts)) in sheets.enumerate() {
      rect((x, 0), (x + w, w), fill: c-surface, stroke: 0.7pt + c-line-hi)
      for (a, b, c, d, t) in parts { if a + 0.3 < w and c < w { link(x + a, b, x + c, d, t) } }
      content((x + w / 2, -0.15), anchor: "north", text(font: mono-m, size: 6.5pt, fill: c-info, tracking: track, upper(if w == S [15 × 15 in] else [12 × 12 in])))
    }
    // the router's path on the first sheet
    line((0.2, 0.2), (0.2, S - 0.2), (S - 0.2, S - 0.2), stroke: (paint: c-sel, thickness: 0.9pt, dash: "dotted"))
    circle((0.2, 2.6), radius: 0.08, fill: c-sel, stroke: none)
    content((4 * (S + 0.35) - 0.35, S + 0.15), anchor: "south-east", text(font: mono-m, size: 6.5pt, fill: c-label, tracking: track, upper[1/8 in 5052 · a 390 mm desktop router bed]))
  })),
  caption: [#lead[Plates, cut on a desk.] The arm's plates nested on three 15 in sheets and one 12 in sheet, with bearing bores and bolt circles; the router's path in orange. Schematic.],
) <fig:plate>

*We own the whole stack, and it can take gradients through contact.* Most robotics teams build on a stack they do not control: a simulator from one company, a CAD tool from another, a learning framework from a third, glued together and re-glued every time one of them changes. Every seam between them is a place where the sim-to-real gap hides, and where no gradient can pass. We wrote ours, from the language parts are written in down to the physics (@fig:elements), so one gradient can run from a measured robot back to a parameter in a source file. Its physics engine, phyz, computes exact gradients of a simulated motion with respect to physical parameters such as mass and inertia. That includes contact, the hard case and the one that matters for anything with feet or fingers: its contact adjoint differentiates the same physics the forward pass runs @ipse2026brief. In simulation, on synthetic motion, it has recovered five of a K1 humanoid's inertial parameters from ±20% error to machine precision (sphere feet; @sec:schema). Domain randomization @tobin2017domain, the usual alternative, can make a policy robust to a gap. It cannot take the gap's gradient.

#figure(
  align(center, cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    // a loss landscape over two parameters, and descent to the true value
    let c = (4.2, 2.1)
    for (k, r) in (0.35, 0.8, 1.3, 1.85, 2.45, 3.1).enumerate() {
      circle(c, radius: (r * 1.35, r * 0.62), fill: none, stroke: (paint: c-info.transparentize(20% + k * 12%), thickness: 0.55pt))
    }
    let path = ((0.4, 3.6), (1.6, 3.15), (2.6, 2.75), (3.3, 2.45), (3.8, 2.25), (4.05, 2.15), (4.2, 2.1))
    line(..path, stroke: 1pt + c-sel)
    for (k, q) in path.enumerate() { circle(q, radius: if k == 0 { 0.09 } else { 0.05 }, fill: if k == 0 { c-text } else { c-sel }, stroke: none) }
    content((0.4, 3.78), anchor: "south-west", text(font: mono-m, size: 7pt, fill: c-text)[START · ±20%])
    content((c.at(0) + 0.15, c.at(1) - 0.12), anchor: "north-west", text(font: mono-m, size: 7pt, fill: c-sel)[TRUE MASS])
    // the foot and the contact that makes it hard
    rect((7.6, 0.5), (8.6, 0.72), fill: c-text, stroke: none)
    line((7.2, 0.5), (9.0, 0.5), stroke: 0.7pt + c-label)
    for k in range(7) { line((7.25 + k * 0.25, 0.5), (7.1 + k * 0.25, 0.32), stroke: 0.4pt + c-label) }
    line((8.1, 0.72), (8.1, 1.6), stroke: 1.4pt + c-text)
    line((8.1, 0.5), (8.1, 1.25), stroke: 0.9pt + c-info, mark: (end: "straight", size: 0.1))
    content((8.25, 1.2), anchor: "west", text(font: mono-m, size: 7pt, fill: c-info)[∂F/∂θ])
    content((8.1, 0.2), anchor: "north", text(font: mono-m, size: 6.5pt, fill: c-label)[CONTACT])
  })),
  caption: [#lead[One stack, gradients through contact.] Descent on two inertial parameters, from ±20% error to the true value, with the gradient passing through the foot's contact. In simulation only.],
) <fig:gradient>

*Humanoids reached the price of a used car.* Unitree announced its G1 at \$16,000 in 2024 @unitree2024g1. We bought a Booster K1 (95 cm, 19.5 kg, 22 degrees of freedom, force-controlled joints) as the reference body for our mind research @booster2025k1. A humanoid that a small company can buy, break, and instrument changes who can do the research.

#figure(
  align(center, cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    // a humanoid with a height mark, a price tag, and the tools that make it ours
    let x = 1.6
    let body = (paint: c-text, thickness: 2.2pt, cap: "round", join: "round")
    circle((x, 3.32), radius: 0.26, fill: c-surface, stroke: 0.9pt + c-text)
    line((x, 3.06), (x, 2.95), stroke: body)
    rect((x - 0.36, 1.95), (x + 0.36, 2.95), fill: c-surface, stroke: 0.9pt + c-text, radius: 0.12)
    line((x - 0.42, 2.85), (x - 0.62, 2.25), (x - 0.58, 1.6), stroke: body)
    line((x + 0.42, 2.85), (x + 0.62, 2.25), (x + 0.58, 1.6), stroke: body)
    line((x - 0.18, 1.95), (x - 0.22, 1.05), (x - 0.24, 0.18), stroke: body)
    line((x + 0.18, 1.95), (x + 0.22, 1.05), (x + 0.24, 0.18), stroke: body)
    line((x - 0.4, 0.1), (x - 0.14, 0.1), stroke: body)
    line((x + 0.14, 0.1), (x + 0.4, 0.1), stroke: body)
    // height
    line((0.4, 0.1), (0.4, 3.63), stroke: 0.5pt + c-info, mark: (start: "straight", end: "straight", size: 0.1))
    content((0.25, 1.85), anchor: "east", text(font: mono-m, size: 7pt, fill: c-info)[95 CM])
    // price tag
    line((x + 0.6, 1.6), (3.3, 1.5), stroke: 0.5pt + c-label)
    rect((3.3, 1.2), (4.9, 1.8), fill: c-sel, stroke: none, radius: 0.06)
    content((4.1, 1.5), text(font: mono-sb, size: 9pt, fill: c-sel-ink)[\$16,000])
    content((4.1, 1.0), text(size: 6.5pt, fill: c-label)[Unitree G1, 2024])
    // buy, break, instrument
    let tags = ((5.8, "BUY"), (7.0, "BREAK"), (8.3, "INSTRUMENT"))
    for (k, (tx, t)) in tags.enumerate() {
      circle((tx, 2.9), radius: 0.32, fill: none, stroke: 0.7pt + c-line-hi)
      content((tx, 2.35), anchor: "north", text(font: mono-m, size: 6.5pt, fill: c-label, t))
    }
    content((5.8, 2.9), text(font: mono-m, size: 7pt, fill: c-text)[\$])
    line((6.85, 2.75), (7.15, 3.05), stroke: 0.9pt + c-text)
    line((6.85, 3.05), (7.15, 2.75), stroke: 0.9pt + c-text)
    line((8.1, 2.8), (8.3, 3.0), (8.45, 2.85), (8.55, 3.05), stroke: 0.9pt + c-info)
  })),
  caption: [#lead[A humanoid, for a car's price.] A body a small company can buy, break and instrument.],
) <fig:humanoid>


The result is that one person can now write every layer between a parameter in a source file and a measured joint torque. Ten years ago that took a company the size of a robot OEM. The specification is written to be carried by more hands than one, and everything in it that can be open is open.

= War <sec:war>

There has not been a year without war in our lifetimes. The longest-running count of armed conflict, kept since 1946, has never recorded a year with none @gleditsch2002armed.



This year made it plain. On February 28 the United States and Israel went to war with Iran. A conditional ceasefire came in April, but in October a blockade still holds, tankers are still struck in the Strait of Hormuz, and oil trades above \$100 a barrel @britannica2026iran @globalsecurity2026iran. That winter the Justice Department released some three million pages of the Epstein files, weeks after the law's deadline, and survivors' lawyers said the release exposed the names of the very people it was meant to protect @cbs2026epstein @ohchr2026epstein. Both showed the same thing. A few people decide, and the people who pay are rarely in the room.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    // one tick per year since the count began; every one of them had a war
    let n = 2025 - 1946 + 1
    let w = 17.6 / n
    for i in range(n) {
      let y = 1946 + i
      let h = if calc.rem(y, 10) == 0 { 1.0 } else { 0.7 }
      line((i * w + w / 2, 0), (i * w + w / 2, h), stroke: 1.2pt + c-fault.transparentize(if y >= 2026 - 1 { 0% } else { 45% }))
      if calc.rem(y, 10) == 0 { content((i * w + w / 2, -0.12), anchor: "north", text(font: mono, size: s-xs, fill: c-label, str(y))) }
    }
    content((17.6, 1.35), anchor: "south-east", text(font: mono-m, size: s-xs, fill: c-text, tracking: track, upper[Years without war: 0]))
  }),
  caption: [#lead[No year without war.] One mark per year since the count began in 1946; each year recorded at least one armed conflict @gleditsch2002armed.],
) <fig:noyear>

Many wars are fought over what is scarce: land, water, fuel, minerals, and the labor to work them. A machine that builds the next machine attacks scarcity at its root. So does power collected rather than taken. A world where machines make machines, on energy no one has to seize, has less to fight over.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    // Earth and its warheads: one dot per hundred, about 12,000 in all
    let (ex, ey, er) = (3.2, 0, 1.15)
    circle((ex, ey), radius: er, fill: c-surface, stroke: 0.8pt + c-info)
    let k = 0
    for ring in range(4) {
      let r = er + 0.32 + ring * 0.24
      let m = (24, 30, 34, 32).at(ring)
      for j in range(m) {
        if k < 120 {
          let a = j / m * 360deg + ring * 7deg
          circle((ex + r * calc.cos(a), ey + r * calc.sin(a)), radius: 0.045, fill: c-fault, stroke: none)
          k += 1
        }
      }
    }
    content((ex, ey), text(font: mono-m, size: s-xs, fill: c-info, tracking: track, upper[Earth]))
    content((ex, ey - er - 1.35), anchor: "north", text(font: mono, size: s-xs, fill: c-label)[one red dot = 100 warheads · ≈ 12,000])
    // what is at stake, listed beside the planet
    let stake = ([8 billion people], [every library], [every machine], [every gene bank])
    for (k, t) in stake.enumerate() {
      content((ex + er + 1.55, 0.9 - k * 0.42), anchor: "west", text(font: sans, size: 6.5pt, fill: c-text, t))
    }
    line((ex + er + 1.4, 1.05), (ex + er + 1.4, -0.45), stroke: 0.5pt + c-line-hi)
    // Mars: no people yet, machines first
    let (mx, my, mr) = (14.6, 0, 0.62)
    circle((mx, my), radius: mr, fill: c-surface, stroke: 0.8pt + c-fault.transparentize(0%))
    content((mx, my), text(font: mono-m, size: s-xs, fill: c-text, tracking: track, upper[Mars]))
    // the transfer, drawn in the cover's orange
    bezier((ex + 1.0, er + 1.0), (mx - mr - 0.2, 0.3), (7, 3.2), (11.5, 2.6), stroke: (paint: c-sel, thickness: 1pt, dash: "dashed"), mark: (end: ">", fill: c-sel))
    content((9.6, 2.75), anchor: "south-west", text(font: mono, size: s-xs, fill: c-label)[≈ 7 months · light 3–22 min])
    content((mx, my - mr - 0.25), anchor: "north", text(size: s-xs, fill: c-label)[machines first])
  }),
  caption: [#lead[One world holds everything.] Every person, every library and every machine we have made is on one planet, beside about twelve thousand nuclear warheads @fas2025status. A civilization on one planet can end in an afternoon. One on two cannot, and on the second, machines go first.],
) <fig:twoworlds>

That is a hope, not a law. The same machines could make war cheaper, and robots are already being built for it. That is why our first commitment is that nothing we build is made or sold to harm a person, or sold for weapons or military use (@sec:safety), and why the commitment is written before the plan rather than after it.



War is also the plainest reason to live on more than one planet (@fig:twoworlds). People will not go first. Machines will. Mars is seven months away and light takes minutes to cross the gap, so nothing there can be built by hand or by remote control. It has to be built by machines that build machines: pairs that land with a seed, make collectors and shelter from local rock, and grow pairs that pass their receipts before the first person leaves Earth (@sec:mars). When people arrive, the work of staying alive will already be under way. The Moon is the rehearsal; Mars is the point.

We still choose hope. Every machine in this plan is a bet that the next century can be one of plenty rather than of taking: that what people have fought over can be made, and made by machines that will not be turned against them.

= Problems <sec:problem>

== Motors

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

The *idiot index* of a part is its cost divided by the cost of its raw materials. A high index means most of the price is process, not matter. Our prototype actuator, rana-100, shows the problem (@fig:rana; line items in @tab:rana).


#mathfig(
  [$ I = C_"part" / C_"materials" approx (\$717) / (\$60 "–" 90) approx 8 "–" 12 $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    // areas to scale: the part's price as a square, its metal as the square in its corner
    let A = 2.6
    let a = A * calc.sqrt(75 / 717)
    rect((0, 0), (A, A), fill: c-surface, stroke: 0.7pt + c-line-hi)
    rect((0, 0), (a, a), fill: c-sel, stroke: none)
    content((A - 0.12, A - 0.12), anchor: "north-east", text(font: mono-m, size: 6.3pt, fill: c-text, tracking: track, upper[\$717 part]))
    content((a + 0.1, a / 2), anchor: "west", text(font: mono-m, size: 6.3pt, fill: c-sel, tracking: track, upper[\$60–90 metal]))
    content((A / 2, -0.15), anchor: "north", text(font: sans, size: 6.5pt, fill: c-label)[area to scale; the rest is process])
  }),
  [#lead[The idiot index.] Cost over raw-material cost, for the manufactured parts of rana-100 (housing, magnetic steel, gear blanks) in @fig:rana. Estimates.],
) <fig:eq-idiot>

About two thirds of the cost is machining that someone else does, on metal worth an estimated \$60–90. The idiot index of the manufactured parts is roughly 8–12 (estimate). At volume the index falls, but it falls by moving labor into tooling and supply chains. That is what a robot OEM does, and it is why a humanoid still costs as much as a used car.



The same structure repeats one level up. A limb's materials are about \$2,000, of which about \$1,200 is the six purchased actuators @mr2026masterplan. The plates are cheap. The actuators are someone else's margin. Owning the actuator is Desk's second step (@sec:arm), and we take that step only where rana wins on a measured metric.

== Math

#mathfig(
  [$ epsilon_i (theta) = y_i^"real" - y_i^"sim" (theta) $ #v(0.3em) $ overline(epsilon) = 1/N sum_(i=1)^N norm(epsilon_i (theta)) $ #v(0.3em) $ "pass" quad "if" quad overline(epsilon) + 2 sigma < overline(epsilon)_"max" $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    let W = 6.0
    line((0, 0), (W, 0), stroke: 0.6pt + c-line-hi, mark: (end: ">", fill: c-line-hi, size: 0.1))
    let errs = (0.55, 0.8, 0.95, 1.1, 1.2, 1.3, 1.45, 1.6, 1.8, 2.05)
    for (i, e) in errs.enumerate() { circle((e * 1.6, 0.25 + calc.rem(i, 3) * 0.17), radius: 0.06, fill: c-info, stroke: none) }
    let g = 1.28 * 1.6
    line((g, -0.1), (g, 1.05), stroke: 1pt + c-text)
    content((g, 1.1), anchor: "south", text(font: mono-m, size: 6.3pt, fill: c-text, tracking: track)[$overline(epsilon)$])
    line((g, 0.95), (g + 0.95, 0.95), stroke: 0.7pt + c-text, mark: (end: "|", size: 0.1))
    content((g + 0.48, 0.97), anchor: "south", text(font: mono, size: 6.3pt, fill: c-label)[2σ])
    let gmax = 4.6
    line((gmax, -0.1), (gmax, 1.25), stroke: (paint: c-ok, thickness: 1pt, dash: (2pt, 2pt)))
    content((gmax, 1.3), anchor: "south", text(font: mono-m, size: 6.3pt, fill: c-ok, tracking: track)[$overline(epsilon)_"max"$])
    content((W / 2, -0.25), anchor: "north", text(font: sans, size: 6.5pt, fill: c-label)[error of each unit, real against simulated])
  }),
  [#lead[The gap, as a number with an error bar.] Each unit $i$ is run for real and in simulation with parameters $theta$; $epsilon_i$ is its prediction error and $overline(epsilon)$ the mean over $N$ units. A gate passes only when the gap, plus twice its spread, clears the threshold.],
) <fig:eq-gap>

Every robotics lab has a sim-to-real gap. We know of few that publish it as a number with an error bar, unit by unit. One established response is domain randomization: vary the simulator until a policy tolerates the difference @tobin2017domain. That makes the policy robust to the gap, but it does not by itself measure it.

We call the gap $overline(epsilon)$ and treat it as the central measurement. The method is system identification's prediction-error method @ljung1999system: each unit's prediction error $epsilon_i$ is what it did minus what the model said it would do, and $overline(epsilon)$ is the mean of their sizes (@fig:eq-gap). We add one step: the gradient of $overline(epsilon)$ moves the design, not only the model. It decides whether a design is ready to build, whether a controller is ready to deploy, and whether an actuator is ready to replace a purchased one. It also tells us how much of the loop can run without a person in it. A gap that is not measured cannot be closed, and a loop with an unmeasured gap needs a human in it.




#aside[A gradient, in one page][
Picture a robot design as a dial board: hundreds of knobs, one for each wall thickness, gear ratio and motor constant. Somewhere there is a score you care about, say how far the robot can reach while holding a full cup.

#align(center, cetz.canvas(length: 1cm, {
  import cetz.draw: *
  // a dial board; the gradient is one arrow per knob: which way, and how much
  let g = (0.9, -0.3, 0.15, -1.0, 0.55, 0.05, -0.65, 0.35, -0.1, 0.8, -0.45, 0.2, 0.7, -0.85)
  for (i, v) in g.enumerate() {
    let x = i * 0.6
    circle((x, 0), radius: 0.2, fill: c-bg, stroke: 0.6pt + c-line-hi)
    let a = 90deg - v * 70deg
    line((x, 0), (x + 0.17 * calc.cos(a), 0.17 * calc.sin(a)), stroke: 0.7pt + c-text)
    rect((x - 0.07, -0.75), (x + 0.07, -0.75 + v * 0.35), fill: c-sel, stroke: none)
  }
  line((-0.3, -0.75), (13 * 0.6 + 0.3, -0.75), stroke: 0.4pt + c-line-hi)
  content((13 * 0.6 / 2, -1.2), anchor: "north", text(font: mono-m, size: 5pt, fill: c-sel, tracking: track, upper[the gradient: for every knob, which way and how much]))
}))

The obvious way to improve the score is to wiggle one knob, rebuild, re-test and see if the score went up. With 300 knobs that is 300 experiments for a single step, and you will need thousands of steps.

A *gradient* is the answer to all 300 questions at once: for every knob, which way to turn it and how much the score cares. Its arithmetic is the chain rule, which says that sensitivities along a chain multiply. If the score moves 2 mm per newton of grip, and grip moves 5 newtons per millimeter of finger wall, then the score moves 10 mm per millimeter of wall.

#align(center, cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let node(x, t, sub) = {
    rect((x - 0.9, -0.36), (x + 0.9, 0.36), stroke: 0.6pt + c-line-hi, fill: c-bg)
    content((x, 0.06), text(font: sans-m, size: 7pt, fill: c-text, t))
    content((x, -0.2), text(font: mono, size: 4.5pt, fill: c-label, sub))
  }
  node(0, [wall], [knob, mm])
  node(3.6, [grip], [N])
  node(7.2, [reach], [score, mm])
  for (x0, t) in ((0, [+5 N per mm]), (3.6, [+2 mm per N])) {
    line((x0 + 0.95, 0), (x0 + 2.65, 0), stroke: 0.6pt + c-label, mark: (end: ">", fill: c-label, size: 0.12))
    content((x0 + 1.8, 0.12), anchor: "south", text(font: mono, size: 5pt, fill: c-label, t))
  }
  // the backward pass
  bezier((7.2, -0.45), (0, -0.45), (5.4, -1.25), (1.8, -1.25), stroke: 1pt + c-sel, mark: (end: ">", fill: c-sel, size: 0.14))
  content((3.6, -1.32), anchor: "north", text(font: mono-m, size: 5pt, fill: c-sel, tracking: track, upper[backward: 2 × 5 = 10 mm of reach per mm of wall]))
}))

Computed backward through a program, from the score to the knobs, the whole gradient costs about as much as running the program once more. That is the trick behind every modern neural network, and it is why we built a stack it can pass through: from a task, through physics, into geometry, to the line of source that sets the knob. Where any link is a file exported from another tool, the chain breaks, and we are back to wiggling knobs one at a time.
]

== Data

Language models learned from trillions of words that people had already written. Robots have no such store. The largest pooled robot dataset holds about a million episodes, gathered across 22 kinds of robot by more than thirty labs @oxe2023. Most robot data is still made one person-hour at a time, by teleoperation.

What exists also travels badly. An episode records commands and camera frames, but rarely the body that executed them: its friction, its backlash, how far it sagged under load. Data from one arm does not transfer cleanly to another, because the difference between the two bodies was never written down. That makes it the same problem as the one above.

This is where the three problems meet. The machining in @fig:rana is the part of the cost that a machine making its own parts goes after. The judgment around it is what keeps a person in the loop: whether a part is right, whether a unit is ready, and what to change when it is not. A gap measured on every unit gives a machine something to check against, and a gap that can be differentiated says what to change. The same measurement labels every episode with the body that made it, so data gathered on one unit can be corrected for the next. A gap that is never measured leaves each of those decisions with a person, and the cost of that person stays in every unit.

= Overview <sec:glance>

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let tiles = (
      ("Desk", "images-dark/fork-v9-lines.png", "an arm that ships with its gap"),
      ("Shop", "pair", "two limbs at any table; a pair builds a pair"),
      ("Road", "images-native/humanoid_bay_iso.png", "a pair on legs that fits its own model"),
      ("City", "city", "a fleet and its pairs work a city"),
      ("Home", "planet", "lineages on their own power"),
      ("Moon", "images-native/seed_iso.png", "a seed grows a pair, with a lifeline"),
      ("Mars", "mars", "the same seed, lifeline cut"),
      ("Star", none, "a swarm of collectors"),
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
    let bands = (("Desk", 1.7, 4), ("Shop", 4, 5), ("Road", 5, 6), ("City", 6, 7), ("Home", 7, 9), ("Moon", 2, 3), ("Mars", 2, 3), ("Star", 9, 26.6))
    for (i, (ch, a, b)) in bands.enumerate() {
      let lo = y0 + 0.12 + i * 0.16
      rect((X(a), lo), (X(b), lo + 0.14), fill: if ch == "Desk" { c-sel } else { c-raised }, stroke: none)
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
  caption: [#lead[The plan, from arm to sun.] Top: the eight stages, each a step up in scale and each built by the one before it. Bottom: electrons under management (@sec:eum) by stage, as orders of magnitude, not targets, against humanity's power use and the Kardashev scale @kardashev1964 @sagan1973cosmic @ei2024review. Only Desk is a plan.],
) <fig:glance>

We are building machines that build machines, and measuring every one. The unit is a limb: one arm, shipped with its sim-to-real gap, the difference between what our physics engine predicted and what the built arm did. Two limbs on any table make a pair, and a pair can do a day's work or build the next limb. A humanoid is a pair on legs. One part, measured everywhere, from a laundry table to a city.

The plan climbs in eight stages, each a step up in scale and each built by the one before it (@fig:glance). Only the first, Desk, is a plan. The rest are goals.

== Flywheel

#figure(
  table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Stage*], [*Delivers*], [*Pays forward*], [*Gate, in short*]),
    [Desk], [Arms, sold one at a time], [Revenue; the first measured gaps; duty cycles that size rana], [≥ 10 at ≥ 40% margin; ≤ 2 mm RMS each],
    [Shop], [Two limbs that work at any table, and build limbs], [Useful work; a falling bought fraction; a doubling time], [A task at 95%; a pair builds a pair],
    [Road], [A pair on legs that fits its own model], [Mobility; a gradient from the gap to the design], [Schema fit beats persistence by 20%],
    [City], [A city's fleet and its twin], [Contracts; real duty at scale; a world to train in], [A city renews for ≥ 12 months],
    [Home], [Lineages on their own power], [A measured doubling time; collectors], [≥ 1 year on collectors it made],
    [Moon], [A seed grows a pair, with a lifeline], [Local material; launch-mass lessons], [≥ 50% local by mass],
    [Mars], [The same seed, lifeline cut], [Autonomy with no one to ask], [0 human commands to a receipt],
    [Star], [A swarm of collectors], [—], [Power and doubling time],
  ),
  caption: [#lead[The flywheel.] Each stage funds and builds the next. The full gates are in @tab:arc. *Proposed targets, to be confirmed by the author.*],
) <tab:flywheel>

Each stage pays for the next and teaches it something it could not learn otherwise (@tab:flywheel).

== Three numbers

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let cards = (
      ("The gap", "—", "mm RMS, per unit", "No limb built yet. First reading: the first limb, against its model."),
      ("Electrons under management", "0 W", "mean, in service", "Nothing we built is in service. The K1 is counted beside it, not in it."),
      ("Doubling time", "—", "per lineage", "No pair has built a pair. First reading: Shop, over three generations."),
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
  caption: [#lead[The dashboard, October 2026.] The gap per unit, $overline(epsilon)$ (@fig:loop), electrons under management (@sec:eum) and doubling time (@sec:replication). We will publish each one when it first has a value, and every time it changes.],
) <fig:dash>

Three numbers track the whole plan (@fig:dash). Today none of them is good, and we say so.

#part("II", "What", art: art-what())[What every stage shares: the principles, the stack, the standards and the rules.]

= Principles <sec:principles>

Each is argued where it is used. They are collected here so that they can be quoted and checked.

+ *Gates, not calendars.* A stage opens on a measured gate: a number, a unit, a method and a pass condition (@tab:arc).
+ *Every unit ships with its gap.* Every limb ships with its sim-to-real gap stated, and every claim ships with the number that supports it.
+ *One part, everywhere.* A limb is a product, half of a pair and an arm of a body. Homogeneity is a design rule, not an accident.
+ *Buy the commodity; own the layer where we win.* The actuator is a commodity. Our edge is the geometry, the measured model and the controller around it (@sec:arm).
+ *The guarantee should live in hardware.* Force limits should come from physics, current limits and compliance, measured on the robot. Today they do not (@sec:safety).
+ *Never be Flock Safety.* No face recognition, no plate reading, no tracking of people, no selling or sharing data that tracks anyone, no police access to people-tracking data (@sec:city).
+ *Everything that can be open is open.* Software, designs, firmware where we control it, and data formats. The K1’s Booster firmware is not ours and is not open.
+ *No division without a license.* A pair copies itself only with a signed, per-copy license from people, under a published ceiling on doubling time (@sec:replication).

#if not web { pagebreak(weak: true) }
= Stack <sec:stack>

== One language
The stack is a tree rooted at one language, loon. Every project in it, software and hardware, is one element of a single table (@fig:elements), grouped by what it does. Build order runs down the tree, and the gradient runs back up it @pond2026strategy.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let s = 1.95   // card
    let gi = 0.12  // gap inside a family
    let gf = 0.34  // gap between families
    let byn(n) = elements.find(e => e.at(0) == n)
    // families left to right; each is a list of columns, read top to bottom in build order
    let fams = (
      ("make", ((1, 2, 3, 4),)),
      ("world", ((5, 6, 7, 8),)),
      ("body", ((9, 10, 11), (12, 13, 14), (15, 16))),
      ("power", ((17, 18, 19),)),
      ("mind", ((20, 21),)),
      ("silicon", ((22, 23, 24),)),
    )
    // word joiners keep the name-coloring rule off the card's own text
    let wj(t) = t.clusters().join("\u{2060}")
    let card(x, y, e, scale: 1, color: none) = {
      let (num, name, f, mat, note, stage) = e
      let col = if color != none { color } else { fam.at(f).color }
      let k = scale
      let w = s * k
      let (fl, st, ink, meta) = if mat == "exists" {
        (col, none, c-sel-ink, c-sel-ink.transparentize(35%))
      } else if mat == "designed" {
        (c-surface, 0.9pt + col, col, c-label)
      } else {
        (none, (paint: col.transparentize(45%), thickness: 0.7pt, dash: (2pt, 2pt)), col, c-label.transparentize(35%))
      }
      rect((x, y), (x + w, y - w), fill: fl, stroke: st)
      content((x + 0.13 * k, y - 0.12 * k), anchor: "north-west", text(font: mono-m, size: 5.5pt * k, fill: meta, number-width: "tabular", str(num)))
      content((x + w - 0.13 * k, y - 0.12 * k), anchor: "north-east", text(font: mono-m, size: 4.6pt * k, fill: meta, tracking: track, wj(upper(stage))))
      content((x + 0.11 * k, y - 0.5 * k), anchor: "north-west", text(font: sans-sb, size: 19pt * k, tracking: -0.02em, fill: ink, wj(lower(name))))
      content((x + 0.13 * k, y - w + 0.13 * k), anchor: "south-west", box(width: (w - 0.26 * k) * 1cm, par(leading: 0.32em, text(font: sans, size: 5.2pt * k, fill: meta, note.split(" ").map(wj).join(" ")))))
    }
    let x = 0
    for (fi, (f, cols)) in fams.enumerate() {
      if fi > 0 { x += gf }
      let col = fam.at(f).color
      let span = cols.len() * s + (cols.len() - 1) * gi
      content((x, 0.62), anchor: "south-west", text(font: mono-m, size: 5.5pt, fill: col, tracking: track, upper(fam.at(f).name)))
      line((x, 0.42), (x + span, 0.42), stroke: 0.7pt + col)
      for (ci, ns) in cols.enumerate() {
        let cx = x + ci * (s + gi)
        for (r, n) in ns.enumerate() { card(cx, 0.22 - r * (s + gi), byn(n)) }
      }
      x += span
    }
    // how to read a card, in the hole under Body
    let ky = 0.22 - 3 * (s + gi)
    let kx = 2 * (s + gi) + gf
    card(kx, ky, ("n", "name", "body", "designed", "what it is", "stage"), color: c-text)
    let lab(y, t) = content((kx + s + 0.2, y), anchor: "west", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper(t)))
    lab(ky - 0.2, [Order · first stage])
    lab(ky - 0.85, [Name])
    lab(ky - s + 0.22, [What it is])
    // maturity
    let mx = kx + s + 3.4
    for (k, (lbl, mat)) in (("exists", "exists"), ("designed", "designed"), ("concept", "concept")).enumerate() {
      let yy = ky - k * 0.62
      let (fl, st) = if mat == "exists" { (c-text, none) } else if mat == "designed" { (c-surface, 0.9pt + c-text) } else { (none, (paint: c-text.transparentize(45%), thickness: 0.7pt, dash: (2pt, 2pt))) }
      rect((mx, yy), (mx + 0.42, yy - 0.42), fill: fl, stroke: st)
      content((mx + 0.6, yy - 0.21), anchor: "west", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper(lbl)))
    }
  }),
  caption: [#lead[The periodic table of Municipal Robotics] @mr2026masterplan. Every project, by family: Make turns source into parts, World simulates them, Body moves, Power feeds and thinks, Mind learns, Silicon is the far future. Solid cards are in use today (in use, not finished), outlined ones are designed or prototyped, dashed ones are concepts. The K1, our bought reference body, is not on the table.],
) <fig:elements>

Most robots are assembled from other people's tools. The body is drawn in a CAD program such as SolidWorks or Onshape @onshape, exported as meshes into a URDF file @urdf, loaded into a simulator such as Isaac Sim @isaacsim or MuJoCo @todorov2012mujoco, and wired together with ROS @macenski2022ros2. Each is excellent at its job. But every handoff is a translation, and what does not survive translation is the reason a design is what it is. A wall thickness becomes a triangle mesh. A motor constant becomes a number in a file. The simulator can say how the robot moved, but not which line of the design to change.

Our projects bond like atoms in a molecule (@fig:bonds). loon is the nucleus: parts, worlds and, one day, chips are written in it. tang carries the math that every engine shares. frog, our coding agent, writes code against local models, which tang can serve. A vcad part is a wodl body is a phyz collider, without an export in between, so a quantity defined once stays defined everywhere. Four rules keep the bonds clean: engines never depend on vcad, and vcad never depends on an app; engines read wodl, not each other; there is one GPU device; and units are millimeters, with Z up, everywhere.


Every number in this plan comes from one loop (@fig:loop): design in loon source, simulate in phyz, build with kerf, measure the built unit, and feed the gap back into the design as a gradient. Each stage removes a person from one more edge of it (@tab:loopstate).

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
    node((1, 1), [*Gap* $overline(epsilon)$\ measured − predicted\ #text(fill: c-label, size: s-sm)[published on every unit]], shape: sqnode, fill: c-surface, stroke: 1.2pt + c-sel, width: 40mm, name: <g>),
    edge(<d>, <s>, "-|>", text(size: s-sm, fill: c-label)[parameters θ]),
    edge(<s>, <b>, "-|>", text(size: s-sm, fill: c-label)[prediction], label-side: left),
    edge(<b>, <m>, "-|>", text(size: s-sm, fill: c-label)[built unit]),
    edge(<m>, (0, 1), <g>, "-|>"),
    edge(<g>, (1, 0.5), (0, 0.5), <d>, "-|>", text(size: s-sm, fill: c-text)[gradient $partial overline(epsilon) slash partial theta$], label-pos: 0.5, label-side: right),
  ),
  caption: [#lead[The loop, read clockwise.] The small labels name the tool at each step. The software projects are listed in @fig:elements, and the dyno is a planned actuator test stand that does not exist yet. Each later stage of the plan removes a human from one more edge. In Desk the gradient edge is a person reading the gap and redesigning by hand.],
) <fig:loop>

The payoff is a gradient that crosses the whole molecule. A task loss in Ipse passes through phyz's contact adjoint, into vcad geometry, and out to a number in loon source, with no translation at any hop. Part of that chain runs today, in simulation: on a test model, a phyz rollout gradient passes through a ground-contact model into a vcad geometry parameter and agrees with finite differences. Joining the two ends is co-design work for Road @pond2026strategy. A chain of separate tools struggles to do this however good each tool is, because the gradient stops at the first export. That is the one bet the stack makes, and it is why we built it rather than bought it.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let wj(t) = t.clusters().join("\u{2060}")
    let at = (
      loon: (0, 0), tang: (2.1, 1.7), vcad: (3.0, -0.9), wodl: (5.0, 0.9), kerf: (5.4, -2.3),
      kosm: (7.3, 2.1), phyz: (7.8, 0.0), iris: (9.8, 2.2), rana: (8.0, -2.3), limb: (10.4, -2.1), ipse: (12.4, 0.3),
    )
    let bonds = (
      ("loon", "tang"), ("loon", "vcad"), ("loon", "wodl"), ("tang", "phyz"), ("tang", "vcad"), ("vcad", "kerf"),
      ("vcad", "wodl"), ("wodl", "kosm"), ("wodl", "phyz"), ("kosm", "iris"), ("kerf", "rana"), ("rana", "limb"),
      ("limb", "ipse"), ("iris", "ipse"), ("phyz", "ipse"),
    )
    let r = 0.52
    for (a, b) in bonds { line(at.at(a), at.at(b), stroke: 0.6pt + c-line-hi) }
    // the gradient: from a task loss in Ipse back to a number in loon source
    let path = ("ipse", "phyz", "vcad", "loon")
    for k in range(path.len() - 1) {
      let (p, q) = (at.at(path.at(k)), at.at(path.at(k + 1)))
      let d = calc.sqrt(calc.pow(q.at(0) - p.at(0), 2) + calc.pow(q.at(1) - p.at(1), 2))
      let u = ((q.at(0) - p.at(0)) / d, (q.at(1) - p.at(1)) / d)
      let o = 0.16
      let n = (-u.at(1) * o, u.at(0) * o)
      line((p.at(0) + u.at(0) * (r + 0.08) + n.at(0), p.at(1) + u.at(1) * (r + 0.08) + n.at(1)), (q.at(0) - u.at(0) * (r + 0.1) + n.at(0), q.at(1) - u.at(1) * (r + 0.1) + n.at(1)), stroke: 1.3pt + c-sel, mark: (end: ">", fill: c-sel, size: 0.18))
    }
    for (name, pos) in at.pairs() {
      let e = elements.find(e => lower(e.at(1)) == name)
      let col = fam.at(e.at(2)).color
      let exists = e.at(3) == "exists"
      circle(pos, radius: r, fill: if exists { col } else { c-bg }, stroke: if e.at(3) == "concept" { (paint: col, thickness: 0.8pt, dash: (2pt, 2pt)) } else { 0.9pt + col })
      content(pos, text(font: sans-sb, size: 8pt, fill: if exists { c-sel-ink } else { col }, wj(e.at(1))))
    }
    content((5.0, -0.2), anchor: "north", text(font: mono-m, size: 5pt, fill: c-sel, tracking: track, upper[gradient]))
  }),
  caption: [#lead[The stack as a molecule.] Each bond is a dependency; color is family, fill is maturity, as in @fig:elements. The orange path is the gradient the stack exists to carry: from a task loss in Ipse, through phyz's contact adjoint and vcad geometry, back to a number in loon source.],
) <fig:bonds>

== Who is in the loop

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Stage*], [*Loop state*], [*Human role*]),
    [Desk], [measured gap → human redesign], [designs, builds, measures],
    [Shop], [gap → design → built and measured by a pair], [reviews, licenses each copy],
    [Road], [gap → gradient → new design], [reviews; pairs build],
    [City], [Shop and Road, across a city's fleet and its twin], [the city sets the work],
    [Home], [many lineages, each closing its own loop on its own power], [licenses each division],
    [Moon], [Shop, far from its supply chain], [remote approval],
    [Mars], [Shop with the supply chain cut], [none between landing and receipt],
    [Star], [fully closed inside a lineage], [reads the receipts],
  ),
  caption: [#lead[Who is in the loop, by stage] @mr2026masterplan],
) <tab:loopstate>


What changes from stage to stage is not the loop but who sits in it. @tab:loopstate summarizes the plan.

= Standards <sec:power>

Every machine we build meets the same four standards: a bus, a pack, a core and nerv, a library of boards. They are why a limb, a pair, a humanoid and a drone can share parts, code and repairs. All of it is modeled; none of it is built @mr2026decisions.

== Bus

Every body runs on one unregulated DC bus of *36–60 V*. Every load must accept the whole window, and nothing may assume a regulated 48 V. The window is set by the pack below: 13 lithium cells in series run from 39 V empty to 54.6 V full. It also covers the RobStride actuators and the rack's 48 V supplies, and it stays under 60 V, so the bus remains extra-low voltage.

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
  caption: [#lead[The pack] @mr2026battery. Every body we build uses it. The runtime figures assume a Rose-class body with two limbs.],
) <tab:pack>

Beside the power runs one management bus, CAN-FD, for presence, power control and telemetry. Bulk data (cameras, logs, the twin) travels over Ethernet. There is no separate hardwired e-stop loop, and other humanoids do not carry one either. Pulling a pack is the hard stop: its connector's last-mating pin is the power enable.

#if not web { pagebreak(weak: true) }
== Pack


There is one battery: a 13S2P pack of Molicel P50B 21700 cells, *468 Wh* and about *2.65 kg* (estimate) @mr2026battery @molicel2023p50b. The same pack goes in a body, a pair, a charging dock, the rack's UPS and the seed. A humanoid carries two of them (A and B, 936 Wh), one in each flank of the torso, and swaps them itself. Its hand draws pack A sideways onto a passive dock rail while pack B keeps the robot powered, then pushes a fresh pack home. The dock is rails, pads and a charger, with no actuators. A swap takes about a minute and leaves the robot powered, so runtime is set by dock capacity, not by the pack. The energy checks by hand, from 26 cells at 3.6 V and 5.0 Ah each @molicel2023p50b:

$ E = underbrace(13, "series") times underbrace(2, "parallel") times 3.6 "V" times 5.0 "Ah" = 468 "Wh" $ <eq:pack>


#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let amber = fam.power.color
    let k = 0.55 // depth seen from slightly above: a horizontal circle draws as an ellipse k high
    let P(x, y, z) = (x, z + k * y)
    // a box seen front and top
    let bx(x0, x1, y0, y1, z0, z1, fill: c-surface, stroke: 0.7pt + c-line-hi) = {
      rect(P(x0, y0, z0), P(x1, y0, z1), fill: fill, stroke: stroke)
      line(P(x0, y0, z1), P(x0, y1, z1), P(x1, y1, z1), P(x1, y0, z1), fill: fill, stroke: stroke, close: true)
    }
    // one upright 21700 cell
    let r = 0.3
    let cell(x, y, z0, z1, col) = {
      let (cx, b) = P(x, y, z0)
      let (_, t) = P(x, y, z1)
      rect((cx - r, b), (cx + r, t), fill: c-raised, stroke: none)
      line((cx - r, b), (cx - r, t), stroke: 0.6pt + col.transparentize(30%))
      line((cx + r, b), (cx + r, t), stroke: 0.6pt + col.transparentize(30%))
      arc((cx + r, b), start: 0deg, stop: -180deg, radius: (r, r * k), stroke: 0.6pt + col.transparentize(30%))
      circle((cx, t), radius: (r, r * k), fill: c-surface, stroke: 0.7pt + col)
      circle((cx, t), radius: (r * 0.42, r * k * 0.42), fill: col.transparentize(55%), stroke: none)
    }
    // 13 positions in a 4-5-4 nest: one cell of each series group per tier
    let pitch = 0.68
    let rows = ((1.2, (-1.5, -0.5, 0.5, 1.5)), (0.6, (-2, -1, 0, 1, 2)), (0, (-1.5, -0.5, 0.5, 1.5)))
    let tier(z0, z1) = for (y, xs) in rows { for x in xs { cell(x * pitch, y, z0, z1, amber) } }
    let (xa, xb, ya, yb) = (-1.8, 1.8, -0.4, 1.6)
    // exploded, bottom to top
    bx(xa, xb, ya, yb, 0, 0.22)
    tier(0.75, 2.45)
    tier(3.5, 5.2)
    bx(xa + 0.1, xb - 0.1, ya + 0.1, yb - 0.1, 6.15, 6.22, fill: amber.transparentize(70%), stroke: 0.6pt + amber)
    bx(xa + 0.15, xb - 0.15, ya + 0.15, yb - 0.15, 7.35, 7.48, fill: c-surface, stroke: 0.7pt + c-info)
    for (cx, cy) in ((-0.8, 0.5), (0.2, 0.8), (0.9, 0.3)) { bx(cx - 0.18, cx + 0.18, cy - 0.12, cy + 0.12, 7.48, 7.54, fill: c-info.transparentize(60%), stroke: none) }
    bx(xa, xb, ya, yb, 8.75, 9.05)
    bx(-0.75, 0.75, 0.15, 0.95, 9.05, 9.5, fill: c-raised, stroke: 0.7pt + c-text)
    for x in (-0.38, 0.38) { rect(P(x - 0.16, 0.15, 9.13), P(x + 0.16, 0.15, 9.42), fill: c-sel, stroke: none) }
    for x in (-0.6, -0.5, 0.5, 0.6) { rect(P(x - 0.03, 0.15, 9.2), P(x + 0.03, 0.15, 9.35), fill: c-text, stroke: none) }
    // labels, right of the stack
    let lx = 2.6
    let lab(z, head, body) = {
      line(P(xb + 0.05, 0.6, z), (lx - 0.1, z + 0.33), stroke: 0.4pt + c-line-hi)
      content((lx, z + 0.33), anchor: "west", {
        text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper(head)); h(0.6em)
        text(font: sans, size: 6pt, fill: c-text, body)
      })
    }
    lab(9.6, [Connector], [Anderson SBS 75X: 2 power, 4 mate-last])
    lab(8.95, [Lid], [the only face that mates])
    lab(7.9, [BMS], [protects, talks CAN, logs EUM])
    lab(6.6, [Busbars], [join each cell to its twin])
    lab(4.6, [13 cells], [one from each series group])
    lab(1.85, [13 more], [their parallel twins: 13S2P])
    lab(0.25, [Tray], [])
    content((0, -0.55), anchor: "north", text(font: mono-m, size: 5.5pt, fill: amber, tracking: track, upper[26 × P50B · 468 Wh · ≈ 2.65 kg]))

    // the swap: a torso, front view, schematic
    let ox = 11.6
    let (tw, th) = (3.0, 4.6)
    let ty = 2.2
    line((ox - tw / 2, ty + th), (ox + tw / 2, ty + th), (ox + tw / 2 - 0.3, ty), (ox - tw / 2 + 0.3, ty), close: true, fill: c-surface, stroke: 0.8pt + fam.body.color)
    rect((ox - 0.55, ty + 0.9), (ox + 0.55, ty + 3.9), fill: c-raised, stroke: 0.6pt + c-line-hi)
    content((ox, ty + 2.4), text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper[core\ bay]))
    // pack B home
    rect((ox + 0.75, ty + 1.0), (ox + 1.25, ty + 3.8), fill: amber.transparentize(25%), stroke: 0.7pt + amber)
    content((ox + 1.0, ty + 2.4), text(font: sans-sb, size: 7pt, fill: c-sel-ink, "B"))
    // pack A, drawn out onto the dock rail
    let ax = ox - tw / 2 - 1.9
    rect((ox - 1.25, ty + 1.0), (ox - 0.75, ty + 3.8), fill: none, stroke: (paint: amber, thickness: 0.6pt, dash: (2pt, 2pt)))
    rect((ax - 0.25, ty + 1.0), (ax + 0.25, ty + 3.8), fill: amber.transparentize(25%), stroke: 0.7pt + amber)
    content((ax, ty + 2.4), text(font: sans-sb, size: 7pt, fill: c-sel-ink, "A"))
    line((ax - 0.9, ty + 0.85), (ox - tw / 2 + 0.15, ty + 0.85), stroke: 1.4pt + c-line-hi)
    for x in (ax - 0.75, ax + 0.35) { line((x, ty + 0.85), (x, ty + 0.2), stroke: 1pt + c-line-hi) }
    bezier((ox - 1.0, ty + 4.15), (ax + 0.1, ty + 4.15), (ox - 1.2, ty + 5.0), (ax + 0.2, ty + 5.0), stroke: 1pt + c-sel, mark: (end: ">", fill: c-sel, size: 0.16))
    content((ox - 1.9, ty + 5.25), anchor: "south", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper[The hand slides A out]))
    content((ax, ty + 0.1), anchor: "north", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper[dock rail]))
    content((ox + 1.95, ty + 2.4), anchor: "west", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper[B keeps\ it powered]))
    content((ox - 1.1, -0.55), anchor: "north", text(font: mono-m, size: 5.5pt, fill: amber, tracking: track, upper[two packs · 936 Wh · a swap in ≈ 1 min]))

    // the voltage window, under both
    let (v0, v1, W, vx, vy) = (34, 62, 14.4, 0.6, -1.6)
    let V(v) = vx + (v - v0) / (v1 - v0) * W
    rect((V(36), vy - 0.22), (V(60), vy + 0.22), fill: c-surface, stroke: 0.6pt + c-line-hi)
    rect((V(39), vy - 0.22), (V(54.6), vy + 0.22), fill: amber.transparentize(55%), stroke: 0.8pt + amber)
    line((V(46.8), vy - 0.34), (V(46.8), vy + 0.34), stroke: 1pt + c-text)
    for (v, t, a) in ((36, [36 V bus floor], "north"), (39, [39 empty], "south"), (46.8, [46.8 nominal], "south"), (54.6, [54.6 full], "south"), (60, [60 V bus ceiling], "north")) {
      content((V(v), if a == "south" { vy + 0.4 } else { vy - 0.4 }), anchor: a, text(font: mono, size: 5pt, fill: c-label, t))
    }
    content((vx - 0.15, vy), anchor: "east", text(font: mono-m, size: 5pt, fill: c-label, tracking: track, upper[volts]))
  }),
  caption: [#lead[The pack] (illustrative; the pack is not designed). Left: one pack, exploded. Two tiers of 13 cells make 13S2P; the BMS on top measures what the machine draws. Right: a humanoid swaps its own packs, one at a time, so it never loses power. Bottom: the pack's whole range sits inside the bus window of every body (@sec:power).],
) <fig:pack>


The pack is also the EUM instrument. EUM (@sec:eum) is defined at each machine's main power input, logged at ≥ 1 Hz, and every pack's BMS measures exactly that. If the BMS logs it, electrons under management are measured for free on every robot and dock we build.



== Core <sec:compute>

#figure(
  grid(
    columns: 3,
    gutter: 1em,
    align: horizon,
    image("images-native/blade_exploded.png", height: 1.8in),
    image("images-native/humanoid_bay_iso.png", height: 1.8in),
    image("images-native/rack_12u_iso.png", height: 1.8in),
  ),
  caption: [#lead[The core and its two hosts] (modeled). Left: one core, exploded. Center: the humanoid bay, lid lifted and one core drawn. Right: a 12U rack with a GPU node and three core chassis. Renders by vcad-render @mr2026compute.],
) <fig:blade>

Compute comes in one module, the *core*: a 132 × 76 × 20 mm carrier on a 3 mm aluminum spine that is both its rail and its heat path, with one rear connector carrying the bus, gigabit Ethernet and CAN-FD @mr2026compute. The same core slides into a 10-inch rack chassis (eight per 2U, fan-cooled) or into a humanoid's torso bay (up to three, conduction-cooled into cold walls with a fan over the fins). A core identifies itself and its power class over I²C before the host powers it: 20 W for a Raspberry Pi CM5 carrier, 30 W for a Jetson-class carrier.

How a body uses its slots follows the seams Ipse already has (@sec:mind). The fast layers, S0 (whole-body tracking, 50 Hz today, 200 Hz target) and S1 (skills and contact planning, 50 Hz), are deadline-bound and must run on the body. The slow layer, S2 (intent and the world model, 1–5 Hz), may run on the body or off it, and if it is lost, S1 finishes the current skill and stands @ipse2026brief. So:

- *Slot 1* always holds a GPU core running S0, S1, the abstract state bus and perception, isolated at real-time priority. Nothing else is allowed on it. On the K1, vendor processes share the perception computer, and worst-case walk inference has been seen at 13.7–15.8 ms against a 0.24 ms median off the robot (session notes, probably preemption).
- *Slots 2 and 3* run S2 when fitted. A full body thinks offline.
- *With fewer cores*, S2 runs on the rack over the same seam.

Whether a body thinks onboard or leans on the rack is then a slot count, not an architecture. The rack is the same standard at larger scale: a 9U, 450 mm deep frame with a GPU node for training and the twin, two core chassis, and two 48 V supplies feeding a vertical busbar, about 1 kW in all (estimate) @mr2026compute.

*Rose against the K1.* Rose should out-think the K1: CUDA, real-time scheduling, and our own code from the carrier up. That claim gets the same treatment as every other one. The same Ipse build runs on both bodies, and we report S0 tick jitter, inference latency and where S2 ran, per body. Rose is first run with one core, close to the K1’s compute, then with more, so the effect of compute is measured one variable at a time.

The open question is the size of S2. A 16 GB Orin NX core may hold it; a Thor-class module would not fit a 20 mm core.

== Boards

Bus, pack and core need a small set of boards. We own a board first when it defines a standard or makes a measurement, and buy the rest until owning it wins (@tab:boards).

#figure(
  table(
    columns: (auto, 1fr, auto, auto),
    align: (left + top, left + top, left + top, left + top),
    table.header([*Board*], [*Does*], [*Order*], [*State*]),
    [BMS], [Protects the pack; measures EUM; talks CAN], [Own first], [Not designed],
    [Pair hub], [48 V + CAN for two limbs, a camera pair and a core], [Own first], [Not designed],
    [Core carriers], [CM5 (20 W) and Jetson (30 W) on the core standard], [Next], [Mockup],
    [Backplanes], [8-slot rack chassis; 3-slot humanoid bay; switch + CAN], [Next], [Mockup],
    [Arm controller], [Six actuator CAN buses, 48 V distribution, one free bus for the BMS], [Next], [Not designed],
    [torc], [Motor control for rana], [With rana], [Prototype],
  ),
  caption: [#lead[The board library and the order we own it] @mr2026decisions. Every design is open. Fabrication and assembly stay bought.],
) <tab:boards>

Every board design is open. Printed-circuit fabrication and assembly stay bought through Road. Boards are, with magnets, bearings and chips, part of the bought fraction that falls last, and a pair at a pick-and-place machine that assembles our own boards is a Shop milestone (@sec:pair).

= Safety <sec:safety>

== Inheritance

Nothing we build can be turned against a person, whether by force, by data or by instruction. These commitments carry forward to any acquirer @mr2026masterplan:

- *Not by force.* No sales for weapons or military use.
- *Not by data.* Never be Flock Safety (@sec:city). Not negotiable.
- *Not by instruction.* Ipse keeps its standing constraints (@sec:mind).
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
    [Interfaces], [A gripper by design; no trigger lines at the tool], [#status("info", "Designed") only the EL05’s CAN + 48 V and a camera pair], [Casual modification],
    [Mind], [Ipse can refuse; refusals are logged, never trained against], [#status("label", "Planned") Ipse is K1 research, not on the limb], [Misuse by instruction],
    [Terms], [Sales terms ban weapons use; fleet buyers vetted], [#status("label", "Planned") no terms written], [Resale, mostly],
  ),
  caption: [#lead[Safety layers, what exists on the limb today, and what each layer would survive] @mr2026masterplan. Today the limb has only a design-level interface layer and vendor firmware current limits; it is not physically incapable of harm.],
) <tab:safety>

The table is ordered by strength. A guarantee in the physics layer survives anything short of building a different robot. A guarantee in the terms layer is a promise. We do not claim the lower layers are as strong as the upper ones.

The limb already shows why the physics layer has to be designed rather than assumed. The RS04’s 120 N·m peak is much more than a 1.27 kg payload needs. So the force limit has to come from current limits, compliance and speed, and it has to be measured. The crush points in the arm audit are an open item for the same reason @rose2026forkv9.

Software, designs and data formats stay open. Open licenses cannot restrict use, so the terms layer is weak for open designs, and that is accepted. The guarantee has to live in hardware, and until clutches and hardware current limits are built and measured, it does not.

= Replication <sec:replication>

== Rules

A machine that builds the next machine invites one fear above others: replication nobody can stop. The image has a name, gray goo, from Drexler's 1986 warning about self-replicating nanomachines @drexler1986engines. Our pairs are not nanomachines. They are two arms at a table, with a mill and a router nearby. They need stock, power and repair, and that is the first defense. It is not enough by itself, because a lineage that doubles on its own power, on a planet with no one on it, is exactly what Star asks for. So the limits are designed in from the first pair. None of them is built yet.

- *Licensed division.* A pair builds a copy only with a signed license for that one copy. Licenses come from keys held by people, not by the pairs. A pair cannot mint its own, and a copy built without one does not pass its acceptance receipts.
- *A germ line no pair can edit.* The design and its receipts live where the machines that do the work cannot change them. Every copy is checked against the germ line; a copy that drifts is decommissioned and stripped for stock.
- *A vitamin made of information.* Every pair depends on one input it cannot make: signed instructions. That keeps the bought fraction able to reach zero in matter while dependence on people remains.
- *A ceiling on doubling.* Each lineage has a published maximum rate of division. Faster growth needs a new license from people, never a decision by the pairs.
- *Stop is tested, not assumed.* Every generation must pass a commanded shutdown drill before its first license (@tab:arc, Shop).
- *No foraging on Earth.* On Earth, pairs take only stock that is delivered to them. They never gather raw material from the environment. Taking local material is for the Moon, Mars and Mercury, under planetary protection rules.

The arc ends where people are least able to watch: on Mars, with no commands between landing and receipt, and around the sun. That is why the licenses, the germ line and the drills have to work on Earth first, measured like everything else.

== Biology

The pair is a cell in the biological sense (@sec:pair), and the analogy earns its place in three engineering decisions, none built yet @mr2026masterplan:

- *Germ line.* The design and its receipts live where no pair can change them, so the machines that do the work are not the ones that carry the design forward (@sec:replication).
- *Quorum sensing.* There is no central scheduler. Each pair publishes what it is doing to the twin, and the nearest idle pair takes the next task.
- *Apoptosis.* A pair that fails its receipts is taken apart by its neighbours, and its parts go back to stock.

= Score <sec:score>

== Electrons <sec:eum>

One number runs through every stage: *electrons under management* (EUM), in watts. It measures the same thing at every scale, so progress can be read off one axis, from one robot to a fraction of the sun's output (3.828 × 10#super[26] W, nominal @iau2015b3).

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    let stages = (
      ("Star", [3.8 × 10#super[26] W]),
      ("Mars", [1 kW]),
      ("Moon", [1 kW]),
      ("Home", [1 GW]),
      ("City", [10 MW]),
      ("Road", [1 MW]),
      ("Shop", [100 kW]),
      ("Desk", [0 W → 10 kW]),
    )
    let w = 17.0
    let h = 4.7
    let n = stages.len()
    for (i, (name, watts)) in stages.enumerate() {
      let inset = i * 0.7
      let top = i * 0.47
      let bot = i * 0.09
      let here = i == n - 1
      rect((inset, bot), (w - inset, h - top), fill: if calc.even(i) { c-surface } else { c-bg }, stroke: if here { 1.2pt + c-sel } else { 0.6pt + c-line-hi })
      content((inset + 0.25, h - top - 0.2), anchor: "north-west", text(font: sans-m, size: s-md, fill: c-text, name))
      content((w - inset - 0.25, h - top - 0.24), anchor: "north-east", text(font: mono-m, size: s-xs, fill: c-label, watts))
    }
  }),
  caption: [#lead[The arc.] Eight stages, each a step up in scale; one loop, one number. Each stage contains the one before it, and each removes a person from one more edge of the loop. Watts are electrons under management (@sec:eum), as orders of magnitude, not targets @mr2026masterplan.],
) <fig:arc>

*Definition.* EUM is the mean electrical input power, over a trailing 30-day window, of every machine in service that we designed and built, measured at each machine's main power input (a DC-bus shunt or a metered AC outlet) and logged at ≥ 1 Hz. "In service" means doing work for someone other than its maker. Machines we did not build but that run our stack, such as the K1 or a bought mill, are reported beside it as *electrons under control*, never added to the headline.

#mathfig(
  [$ "EUM" = 1/T sum_m integral_0^T V_m (t) I_m (t) dif t $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    let W = 6.0
    let pts = range(0, 61).map(i => { let x = i / 60 * W; (x, 0.55 + 0.32 * calc.sin(i * 0.7) + 0.2 * calc.sin(i * 0.23) + 0.12 * calc.cos(i * 1.9)) })
    line((0, 0), ..pts, (W, 0), close: true, fill: c-info.transparentize(80%), stroke: none)
    line(..pts, stroke: 0.8pt + c-info)
    line((0, 0), (W, 0), stroke: 0.6pt + c-line-hi)
    line((0, 0.55), (W, 0.55), stroke: (paint: c-text, thickness: 0.8pt, dash: (3pt, 2pt)))
    content((W + 0.1, 0.55), anchor: "west", text(font: mono-m, size: 6.3pt, fill: c-text, tracking: track, upper[mean]))
    content((W / 2, -0.15), anchor: "north", text(font: sans, size: 6.5pt, fill: c-label)[V × I at each machine's input, summed, over 30 days])
  }),
  [#lead[Electrons under management.] Power $V_m I_m$ at the input of every machine $m$ we built that is in service, averaged over a trailing window $T$ of 30 days.],
) <fig:eq-eum>

*Baseline, October 2026.*


 EUM is *0 W*: we have built nothing that is in service. Electrons under control are the K1, at roughly 100 W while it runs (estimate, not measured; its duty is not logged yet). A working pair would add about 150 W mean, with two limbs near 200 W peak each (estimate).

== Doublings

The second headline number is the time a lineage of pairs takes to double what it can build and power, measured from receipts over at least three generations. It is undefined today, because no pair has built a pair. It sets how long the arc takes (@sec:sun) and it is capped by policy (@sec:replication).

*Trajectory, as orders of magnitude, not targets.* Desk: 0 W → \~10 kW (a hundred limbs near 50 W mean). Shop: \~100 kW as pairs build pairs. Road: \~1 MW of bodies. City: \~10 MW, a city's fleet and the pairs that keep it. Home: \~1 GW and up, on collectors pairs made. Moon and Mars: the first kilowatts off Earth. Star: a measurable fraction of 3.8 × 10#super[26] W.

#part("III", "How", art: art-how())[How we climb: eight stages, from one arm to a star. Only the first is a plan.]

= Desk <sec:arm>

*An arm that ships with its gap.* Desk has two steps over roughly three years. Each closes on an exact gate (@tab:arc).

+ *An arm worth buying.* Build it on RobStride actuators. The actuator is a commodity. Our edge is everything around it: geometry checked against real loads, a measured model of every unit, a controller trained against that model, and a stated sim-to-real gap on every limb.
+ *Own the actuator, where it wins.* Desk revenue funds rana and torc. A purchased joint is replaced only where rana wins on a measured metric. Dropping rana is a valid outcome, not a failure.

== Sizes

The limb comes in the same three sizes as Rose. *Limb mini* is K1 size, for Rose mini and for light pairs. *Limb full* is human size: the Rose fork v9 described below, and the limb this paper means when it says "limb". *Limb mega* is industrial, for heavy pairs and Rose mega. All three share one design, the fork, written as one loon source with size as a parameter. Only limb full is drawn; mini and mega are scaled from it and not yet checked against their actuators.

== Arm

#colfig(
  image("images-dark/fork-v9-lines.png", width: 60%),
  caption: [#lead[The arm, Rose fork v9.] Six joints, every plate 1/8" 5052 except the 1/4" foot, every motor mounted by its back face. Render from the arm repository @rose2026forkv9.],
) <fig:arm>

One arm does three jobs: it is sold on its own, it is half of every pair, and it is an arm of the Rose humanoid. Its design idea is that *every pitch joint is a fork* @rose2026forkv9. The parent link holds the motor's stator between its two side plates. The child straddles the parent, with one plate bolted to the output and the other riding a plain bearing on a hollow sleeve on the joint axis. Parent and child share one centerline, so no joint is offset and there is no offset elbow to twist the arm. Cables pass through the hollow sleeves, so they only twist and never loop outside the arm. The name comes from where the design lives. Rose fork v9 is the ninth revision of the fork concept, drawn in the Rose humanoid repository as a table arm that is also Rose's arm @rose2026forkv9. In this paper "the arm" means this design, and "Rose" on its own means the humanoid line (@fig:elements), whose \$15,000 target assumes rana (@sec:rana).

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
    [Foot], [Ø230, 1/4" 5052, 4 × Ø6.6 on Ø190], [M6 to the pair plate],
    [Plate stock], [1/8" (3.175 mm) 5052], [Three 15 × 15 in sheets of 1/8"; one 1/4" sheet for the foot],
    [Router parts], [20 unique], [DXF + nests generated],
    [Jaw tips from shoulder], [731 mm (786 with gripper v2)], [Modeled],
    [Mass], [8.21 kg (arm-model jaws)], [Modeled],
    [Mass with gripper v2 + harness], [8.44 kg], [Modeled: arm plus gripper @mr2026gripper],
    [Continuous hold, straight out], [1.49 kg (arm-model jaws)], [Modeled; RS04 stall derated (estimate)],
    [Continuous hold, gripper v2], [1.27 kg (shoulder-limited)], [Modeled; RS04 stall derated (estimate)],
    [Sag / twist at flange], [0.13 / 0.11 mm], [Modeled],
    [Elbow / wrist range], [−120 to 115° / ±135°], [],
    [Interference, 209 poses], [None], [Swept in CAD],
    [Build state], [Ready to cut], [None built yet],
  ),
  caption: [#lead[Rose fork v9 specification.] Every number is a model output; none has been measured on a built arm. Arm-model jaws are the simpler tool in the arm's own model, two flat jaw plates on the same EL05, and are lighter than gripper v2.],
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

== Actuator <sec:rana>

The actuators are about 60% of a limb's materials (\~\$1,200 of \~\$2,000). That share is why the actuator step exists. A rana actuator replaces a RobStride joint only where it wins on a *measured* axis: dollars per N·m, torque density, backdrivability, or supply independence. Today rana is worse on cost. The prototype is an estimated \$1,086 per unit, plus \~\$150–300 to cut the gear teeth @rana2026bom, against an estimated \~\$200 average per purchased joint. The prototype stator is solid 1018 steel rather than laminations, so eddy-current losses derate it; it is for bench characterization only. Its 36 N·m peak and 0.166 N·m/A torque constant are predicted upper bounds, not measurements. On price, rana has to cost about 6–7× less before it competes. Whether it wins on any of the other measured axes is for the dyno to decide.

== Economics

#figure(
  table(
    columns: (auto, auto, auto, auto, 1fr),
    align: (left + top, right + top, right + top, right + top, left + top),
    table.header([*Product*], [*List*], [*Materials*], [*Margin*], [*Contents*]),
    [Limb], [\$5,000], [\~\$2,000], [60%], [Six RobStride joints (\~\$1,200), plates, harness, gripper],
    [Pair], [\$12,000], [\~\$4,400], [63%], [Two limbs, a table plate, a pair hub, a camera pair, a core],
    [Rose], [\$15,000], [not costed], [–], [A pair on legs. Needs rana at a few hundred dollars per joint],
  ),
  caption: [#lead[Prices and margin on materials] @mr2026masterplan. List prices are targets, not quotes; materials are estimates. Margin here is list less materials, a ceiling on gross margin: labor, test, warranty, shipping and overhead are not yet known and will reduce it.],
) <tab:prices>

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
    [Labor, test and warranty before margin reaches zero], [\~\$3,000/limb], [0%], [List minus materials],
  ),
  caption: [#lead[Limb margin sensitivity and break-even] (estimate, from the figures above). A limb's economics are mostly the actuator's economics.],
) <tab:sens>

Until rana succeeds or is abandoned, the limb business depends on one supplier's pricing. We do not expect the Rose price (\$15,000, the G1’s price class @unitree2024g1) to be reachable on purchased actuators (estimate; the Rose bill of materials is not costed in this paper). That is why it waits for rana.

Vendors rarely say at what pose or duty cycle a rated payload holds. Our 1.27 kg is a modeled continuous hold at full extension, the worst case; bent working poses hold more. If you need a proven arm now, a commercial one will likely serve you sooner.

*Gate.* ≥ 10 limbs sold to outside buyers at ≥ 40% gross margin, after materials, labor, test and warranty (invoices, trailing 12 months). Tool-point error ≤ 2.0 mm RMS on every limb shipped: 50 fixed poses, unloaded and at 1.0 kg, probe against a calibrated plate, printed on its receipt. rana ≥ 1.2 × RS04 in continuous stall torque per kg or per dollar, on one dyno at 25 °C, or rana is dropped (@tab:arc).

= Shop <sec:pair>

*Two limbs at any table; a pair builds a pair.* Bolt two limbs to a table, add a camera pair and a core, and the table becomes a workstation. Nothing else changes: no rails, no enclosure, no special room. The same pair folds laundry, tends a mill, sorts parts or assembles the next limb. What it does is software, trained against the measured models of its own two limbs.

#figure(
  table(
    columns: 6,
    align: (left + top, right + top, right + top, left + top, left + top, left + top),
    table.header([*Arm*], [*Reach*], [*Mass*], [*Rated payload*], [*List*], [*Status*]),
    [This limb], [786 mm], [8.44 kg], [1.27 kg continuous, straight out, with the 485 g gripper (modeled)], [\$5,000 (target)], [Not yet built],
    [Seeed reBot B601-RS], [754 mm], [6.7 kg], [To be cited (vendor page, access date)], [To be cited], [To be cited],
    [UFACTORY Lite 6], [To be cited], [To be cited], [To be cited], [To be cited], [To be cited],
    [UFACTORY xArm 6], [To be cited], [To be cited], [To be cited], [To be cited], [To be cited],
  ),
  caption: [#lead[Arms a buyer can order today.] reBot reach and mass from the arm repository's comparison, which uses the same RobStride family @rose2026forkv9. Empty cells wait for a dated vendor citation.],
) <tab:alts>

== Mount

A pair is two limb feet (Ø230 mm, four M6 on Ø190) on one plate that clamps to an ordinary table, with a pair hub between them carrying 48 V and CAN to each limb, a camera pair above the work, and one core (@sec:power). The plate fixes the distance between the shoulders, so every pair shares one calibrated geometry. Its size, clamp and hub are not designed yet.

== Work

A pair is a product before it is anything else. It sells for its work, and the first buyers are labs and builders who want a measured, open pair of arms to build on. A pair is far cheaper than a humanoid and does not walk; it brings the robot to the task by bringing the task to a table.

The first task is folding laundry. It is three things at once. It is the demonstration everyone understands without explanation. It is a benchmark: cloth is hard, every folded shirt can be counted, and labs can compare against a published success rate and gap. And it is a market, in laundromats, hotels and, one day, homes. Machine tending, kitting and assembling the next limb follow. Each task gets its own gate: a named task, a success rate, a count (@tab:arc).

== Learning

A pair learns in simulation first, against the measured model of its own two limbs, and then closes the gap on hardware. That is the loop of @fig:loop applied to a task. Folding needs cloth in the simulator, and phyz has it: position-based cloth, soft bodies and cables, and finite-element soft bodies, on a development branch. What is not yet shown is a gradient through cloth contact, the piece that would let a folding policy, and the pair's own geometry, be optimized against the gap. Folding is gated on it.

== Division

A pair is also the cell of this plan: the smallest unit that can make more of itself. It builds limbs, and two limbs make a pair. In order:

+ A pair assembles a limb from cut plates and purchased actuators, and the limb passes the Desk test.
+ A pair tends the mill and router that cut those plates, and winds and pots its own rana stators.
+ A pair builds a pair, with the bought fraction falling each generation.

Replication is a capability the product grows into, not the reason it exists.

Then come the hard parts. Actuators end to end: magnets stay bought until a ferrite rana is validated, and bearings stay bought longest. Boards: a pair at a pick-and-place machine (@tab:boards). Silicon: loom would describe chyp, an inference chip, and rune (maskless lithography) would make it at a node a lab can own. Energy: the first pair that runs on panels it made. Silicon is not parked; a machine that reproduces off-planet cannot depend on anyone's fab.

*Gate.* A pair at a customer's table completes a named task (for example, folding 50 garments) at ≥ 95% success. A pair assembles a limb that passes the Desk test in ≤ 30 min of human touch time. A pair builds a pair that passes its receipts at ≤ 20% bought cost, with bought share falling over 3 generations and a commanded shutdown drill passed in each (@tab:arc).

= Road <sec:mind>

*A pair on legs that fits its own model.* A humanoid is two limbs, a torso, legs and a mind. It costs far more than a pair, and it goes where the work is.
 Road has three parts, in order: the body-schema claim (@sec:schema); co-design by gradient, in which vcad's differentiable kernel and phyz's contact adjoint optimize geometry, actuator sizing and controller together; and rana and torc at volume, with the limb fleet's real duty cycles as load cases. We will attempt co-design only where body shape limits the task @pond2026strategy. A gradient-designed replacement for a part that already works answers no question.

== Bought

We bought a Booster K1 rather than building first. It is 95 cm tall, weighs 19.5 kg, and has 22 degrees of freedom (6 per leg, 4 per arm, 2 in the head). It has force-controlled joints with dual encoders, an RGBD camera and a microphone array @booster2025k1. Its perception computer is a Qualcomm QCS8550 (6 cores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), beside a separate motion computer that runs the vendor's control; we measured this on the robot, and it differs from the vendor summary we first relied on @ipse2026deploy. A commercial body has known kinematics and a known-good build, so it serves as a *control*: every later body, including Rose, is measured against it.

The K1 has two jobs. In a proposed Lakewood pilot it would pick up litter (@sec:city); it has not grasped litter yet. In the lab it is the body for Ipse, a developmental research program. The two share hardware, software and one rule: every claim ships with the number that supports it.

== Rose

Rose is the humanoid we will build. The name is the thesis: past tense of rise, the one that stood up. It is a line of three sizes, each anchored to something real (@tab:rose). Every Rose shares one design language: bent 3 mm 5052 sheet as primary structure, cut flat and folded, with no machined blocks; an exposed skeleton for the limbs against a shaped torso that is itself the structure; contact geometry on the feet only @rose2026line.

#figure(
  table(
    columns: (auto, auto, auto, 1fr, 1fr),
    align: (left + top, right + top, right + top, left + top, left + top),
    table.header([*Size*], [*Height*], [*Mass*], [*Anchor*], [*State*]),
    [mini], [1,200 mm], [18.0 kg], [K1 size], [The only size verified end to end: single-leg 90° squat needs 33 N·m of 60 available; packing check passes],
    [full], [1,750 mm], [\~55.5 kg (est.)], [Human size], [Larger knee and ankle actuators needed; packing check blocked on missing actuator drawings],
    [mega], [2,000 mm], [64.3 kg], [Industrial], [rana joints throughout; 1.2 kWh at 48 V; interfaces frozen before geometry],
  ),
  caption: [#lead[The Rose line] (modeled) @rose2026line. Masses and holds are model outputs; full's mass is scaled from a smaller actuator, not measured.],
) <tab:rose>

In this plan, Rose's arms are limbs, full stop: a humanoid is a pair on legs, with a torso that carries the core bay and two packs (@sec:power). Each Rose takes the limb of its size: mini takes limb mini, full takes limb full, mega takes limb mega. The current drawings still carry four-degree-of-freedom arms on purchased actuators; replacing them with limbs, and the legs' purchased actuators with rana where it wins, is open work.

Rose mini does not wait for the pair to sell. It starts now, in parallel, on purchased actuators and limb mini, because the K1 has shown the limits of a body we did not build. Road overlaps Shop; it does not follow it. Rose full's \$15,000 target is the G1’s price class @unitree2024g1, and it waits for rana.

One finding already shapes rana. Sized to jump, Rose mega is limited by speed, not torque. At peak current every rana delivers the same \~1.3 kW of mechanical power, set by the 48 V bus and a 30 A current limit, so a bigger rana buys torque, not power. A higher-voltage leg drive or series elastics, not a larger motor, is the lever for jumping.

== Ipse
#figure(
  grid(columns: 3, gutter: 1.5em, align: bottom,
    image("images-native/rose_mini_iso.png", height: 1.25in),
    image("images-native/rose_pro_iso.png", height: 1.45in),
    image("images-native/rose_max_iso.png", height: 1.65in),
  ),
  caption: [#lead[Rose mini, full and mega] (modeled), each rendered at the height its CAD was drawn, not yet at the sizes in @tab:rose. The three do not yet share one design language; bringing mini and mega into full's bent sheet is open work.],
) <fig:rose>

Ipse matters to this plan for two reasons: it produces Road's first claim, a body that fits its own model (@sec:schema), and it is the layer in @tab:safety that can refuse an instruction.

Ipse (Interoceptive Predictive Self-Emergence) is a scaffold for growing a mind on a K1 over months of lived experience @ipse2026brief. The name states the project's hypothesis, not a result. It is not a robot control stack. Its architecture is a world model with fast and slow latents; interoception (battery, joint temperature, motor current, IMU) enters as observation rather than reward. Its simulation runs in phyz, on bodies authored in vcad; no MuJoCo or Isaac is involved.

Its standing constraints are quoted here because they bind the company, not just the research @ipse2026brief:

- "Preserve experience stores separately from weights, versioned, always." Weights can be retrained; a developmental history cannot be recovered.
- "Never train against expressed reluctance."
- "Do not optimize for consciousness indicators." Indicator lists such as Butlin et al. @butlin2023consciousness are used as an audit at phase boundaries, never as a training target.
- "Grown, not built." When a behavior is missing, the first question is what experience would produce it.

One limit is hardware. The K1’s perception computer has no CUDA, its idle vendor stack holds about two of its six cores, and an onboard vision-language model has no measured headroom @ipse2026deploy. So the work is split along Ipse's own seams (@sec:compute). Drives, memory, workspace and refusal stay on the robot; a cloud model is an amplifier it uses @ipse2026brief.

We do not name the robot. The design brief says it "should not be named by us": it gets a name when it has earned one or arrives at one. Until then it is "the K1".

== Self <sec:schema>

The first claim of Road is Ipse's Phase 1 milestone. It is the body-schema result, not the world model @ipse2026brief @pond2026strategy.

*Claim.* The K1 starts from the vendor's model of its own body, which is known to be wrong in places. From its own exploratory movements (motor babbling), and by gradient descent through phyz's contact model, it fits a body schema: its own masses and inertias. The fitted schema must predict held-out motion better than persistence (the baseline defined in the falsifier below) and must match the CAD ground truth. We publish the residual between the two as the *measured body schema*.

*Already shown.* In simulation, on motion generated by the same K1 model, five inertial parameters (trunk mass, two trunk center-of-mass coordinates, one thigh mass and one shank mass) recover from ±20% error to machine precision by gradient descent through contact @ipse2026brief. The run used sphere feet and the T-pose. With the arms down, the same squat excitation topples the robot and the fit stalls. The K1’s real flat feet give a contact the gradient cannot yet pass through. This shows the gradient is correct and these parameters are identifiable. It does not show a fit to a real body. In simulation the K1 also stands for 10 s at 1, 2 and 5 ms timesteps and recovers from an 8 N·s push at the trunk.

*Not yet shown.* Everything on hardware. Real data adds actuator friction, backlash, sensor latency and unmodeled compliance. The simulated result has none of them, so it is an upper bound on how well the method can work.

*Preregistration.* The claim, the held-out split, the horizons and the falsifier will be written down and published before the hardware fit runs. This follows the discipline of the project's world-model claims, whose earlier rounds are on the record as void and falsified @pond2026strategy.

*Falsifier.* If the fitted model cannot beat *persistence* ("the next state is the current state") on the R1 capture harness at the stated horizons, the claim fails. Persistence is the bar because on a mostly still body at \~49 Hz it is nearly right. A model with low error has shown nothing until it beats persistence.

Ipse also carries a self-probe from the start. It is an instrument that will later test whether the model forms a self-representation that holds across behaviors. It runs now so its metric has a history @ipse2026brief. We make no claim about what it will show.

*Gate.* Preregistered hardware body-schema fit: held-out joint-position error ≤ 0.8 × persistence at 0.1 s and 0.5 s, and fitted link masses within ±5% of weighed parts. Rose walks 100 m on level pavement carrying two limbs, 0 falls in 5 of 5. A gradient co-designed part beats its hand-designed predecessor by ≥ 10% on its named objective (@tab:arc).

= City <sec:city>

*A fleet and its pairs work a city.* The fleet that began as one litter pilot covers a city's sidewalks. Pairs in the city build, repair and recycle that fleet. The city twin, the wodl world of its streets, is kept current by the robots that work them, and its visual sim-to-real gap is measured weekly on real pavement. The city, not us, decides what work the robots do.

What a city buys is two things in one contract: clean streets, and the map that comes from working them. Pricing is open, and will be set with the first city, not before it.

It is also the nearest good in this plan. A block with no litter on it. A pothole reported the morning it opens, a dark streetlight the night it fails. Free wifi at every dock, logging nothing. A city whose map belongs to the city. None of that waits for the Moon.

== Litter

Litter pickup would put robots on every block every week. Most litter goes uncollected. Starting there means starting with work nobody is doing, so no one's job is the first thing automated.

Lakewood is a first-ring suburb of Cleveland with 49,517 residents in 5.5 square miles of land @census2024lakewood, about 9,000 people per square mile. It is among the most densely populated cities in Ohio, with more than 180 miles of sidewalk @lakewood2024sidewalks. Our earlier paper used it as the reference case for sidewalk snow clearing @muni2025whitepaper. It is small enough to model whole as a city twin, and its streets are dense enough that a few robots would see a lot of pavement.

How the K1 picks up litter is not settled. The current concept, under design, is a *reacher*: a \~450 mm tube mounted on the K1’s wrist with gripper v2 jaws at its tip, an EL05 at the wrist end driving the jaws through a pushrod, a bag on a back frame, and a dock that is a bin, a charger and a public wifi point at once. The reacher keeps the K1 upright: it picks from standing, without squatting. Later, Rose takes over for two-handed and squatting work.

The concept has four open checks, none done yet:

- *A K1 wrist mount* for the reacher tube, which does not exist yet.
- *A K1 arm load check:* 0.6 kg held 50 cm out from the wrist, against the K1 arm's joint ratings.
- *A grasp set:* the litter the reacher must pick (cans, bottles, wrappers, cups), with a success rate per item.
- *Behavior around people:* yield, stop, and never reach toward a person.

Outdoor grasping, year-round, would give two things the lab cannot. It would produce the duty cycles that will size rana. And it would put the Ipse body on a street, where the gap would be measured on real pavement.

== Map

The robots would map the streets while they work. Two products come out of the map @mr2026masterplan:

- *Live facts:* a pothole, a blocked curb ramp, ice, a dark streetlight.
- *Delayed pictures*, with people and cars removed before anything leaves the robot.

The same map would become a wodl world (@fig:elements) of the city, so the robots would train on the streets they work.

== Docks

Each dock along the routes is a litter bin, a charger and a free public wifi point at once, not citywide coverage. The wifi needs no account, keeps no traffic or connection logs, and routes traffic straight to backhaul.

== Privacy

#flame[*We will never be Flock Safety.*] Storing data is fine; a street robot that learns has to remember. What we will never build is a system that watches people @mr2026masterplan.

These are design rules for the street pilot; none of the street pipeline is built yet.

- *No face recognition and no license-plate reading.* Those models will not be on the robot or on our servers.
- *No tracking of people's movements.* We never link a person, a car or a device across frames, places or days.
- *No selling or sharing of data that tracks anyone,* and no police access to people-tracking data, because we will not hold any. Street maps, potholes, ramps and litter counts are the city's to use.
- *We map the city, not the people.* People and cars are removed on the robot before street imagery is written to storage. Pictures are released delayed; live facts (a pothole, ice) can be fresh.

*Ipse's experience store.* Ipse preserves its experience store, versioned, always (@sec:mind). That does not conflict with the line above: the store keeps the robot's own body history in full, and what it keeps from a public street passes through the same on-robot filter first, so it holds scenes, not identities or tracks. The robot's safety log, needed for incident review, follows the same rule; its retention period is not yet decided.

*Open, so anyone can check.* Everything that can be open is open: our software, our designs, the firmware on hardware we control, and our data formats. The K1 runs Booster's firmware, which is not ours and is not open; on the K1 the check stops at our own code.

== Scope

The pilot does not automate interaction with the public beyond yielding and stopping. It does not enter private property, and it does not touch sharps, hazardous waste or anything a person is holding. Remote assistance may be a product feature, but data from it never trains Ipse @mr2026masterplan. *Proposed scope, pending confirmation:* this list is a draft and has not been agreed internally or with any city.

== moth <sec:moth>

moth is a drone for street perception @mr2026decisions. It flies ahead of the K1 or Rose to find litter, potholes, blocked curb ramps and dark streetlights, and builds the street map from above.

moth is not sold for military use or armed in any form (@sec:safety).

moth is also the cheapest check that the standards in @sec:power are body-agnostic. It is a one-core body on the 36–60 V bus with CAN-FD for management, running the same Ipse seams as a humanoid. It carries its own lighter pack: the 2.65 kg humanoid pack only suits a heavy-lift airframe.

*The same street rules.* Mapping needs imagery, so moth's imagery leaves the drone, under the rules of @sec:city: people and cars are removed on the drone before anything is written to storage, pictures are released delayed, and nothing links a person, car or device across frames, places or days. A camera over a residential street is what people fear most, so moth's filter gets its own published test: how often a person or plate survives it, measured on held-out street footage.

Nothing of moth is built. Its airframe, pack and mass budget are not designed, and whether one \~150 g core fits its mass budget is open.

*Gate.* The litter pilot renews for ≥ 12 months after a ≥ 3-month trial, with 0 contact incidents. A city renews fleet and twin for ≥ 12 months. ≥ 90% of fleet repairs done by pairs in the city, by receipts (@tab:arc).

= Home <sec:planet>

*Lineages on their own power.* Lineages of pairs spread to many cities and run on power they collected, from panels they made. The bought fraction approaches zero everywhere except chips, and every division is licensed (@sec:replication). The headline numbers become electrons under management and doubling time (@sec:eum): how long a lineage takes to double what it can build and power.

*Gate.* A lineage runs ≥ 1 year on collectors it made; doubling time measured over ≥ 3 generations; bought share ≤ 5% of cost (@tab:arc).

#mathfig(
  [$ C = b thin C_"bought" + (1 - b) thin C_"made", quad b -> 0 $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    let bs = (0.62, 0.48, 0.36, 0.27, 0.19, 0.13, 0.09, 0.06, 0.04, 0.03)
    for (i, b) in bs.enumerate() {
      let x = i * 0.6
      rect((x, 0), (x + 0.42, 1.6), fill: c-surface, stroke: 0.5pt + c-line-hi)
      rect((x, 0), (x + 0.42, b * 1.6), fill: c-sel, stroke: none)
    }
    line((-0.1, 0.05 * 1.6), (6.0, 0.05 * 1.6), stroke: (paint: c-ok, thickness: 0.8pt, dash: (2pt, 2pt)))
    content((6.05, 0.08), anchor: "west", text(font: mono-m, size: 6.3pt, fill: c-ok, tracking: track, upper[5%]))
    content((3.0, -0.15), anchor: "north", text(font: sans, size: 6.5pt, fill: c-label)[generations of pairs; orange is the bought share $b$])
  }),
  [#lead[The bought fraction.] Each generation makes more of itself, so the bought share $b$ falls and cost tends to $C_"made"$: energy and time. Home's gate is $b <= 5%$. Illustrative.],
) <fig:eq-bought>


= Moon <sec:moon>

*A seed grows a pair, with a lifeline.* The Moon comes first because it is about three days away and 1.3 light-seconds out. A seed there runs the Shop loop with remote approval and teleoperated help when something breaks. It meets vacuum, abrasive dust, regolith as stock and no supply chain, with a lifeline. Its central measure is the same as Shop's: the bought fraction.

The seed is the smallest thing that can grow a pair: one pair, a mill and a router, a collector and a pack, the design in its germ line, and enough bought parts for a few generations. Before it flies, each of its hard problems gets a measured gap on Earth: vacuum and thermal cycling, abrasive dust in simulant, and radiation on a core. A single limb flown as a hosted payload would be the first number off Earth.

*Gate.* A seed builds a pair that passes the Earth receipts, ≥ 50% of its mass from local material, with remote approval only (@tab:arc).

= Mars <sec:mars>

*The same seed, lifeline cut.* Mars is about seven months away and 3–22 light-minutes out, too far for remote hands, so the pair runs with no human commands between landing and its receipt. Moon and Mars add problems Shop never meets (launch mass, radiation, thermal cycling, extracting metal from regolith), and the Earth tests of @sec:moon only begin on them. Mars is where machines go first, so that people can follow: by the time a person lands, pairs will have built shelter, collectors and the means to make more of themselves (@sec:war).

#mathfig(
  [$ t = d / c approx (0.55 "–" 4.0 times 10^8 "km") / (3.0 times 10^5 "km/s") $ #v(0.3em) $ approx 3 "–" 22 "min" $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    let (e, m) = (1.0, 1.52)
    let u = 1.15
    circle((0, 0), radius: 0.16, fill: c-sel, stroke: none)
    circle((0, 0), radius: e * u, stroke: 0.5pt + c-line-hi)
    circle((0, 0), radius: m * u, stroke: 0.5pt + c-line-hi)
    let E = (e * u, 0)
    circle(E, radius: 0.08, fill: c-info, stroke: none)
    let M1 = (m * u, 0)
    let M2 = (-m * u, 0)
    circle(M1, radius: 0.07, fill: c-fault, stroke: none)
    circle(M2, radius: 0.07, fill: c-fault, stroke: none)
    line(E, M1, stroke: 1pt + c-text)
    content(((e + m) / 2 * u, -0.12), anchor: "north", text(font: mono-m, size: 6.3pt, fill: c-text, tracking: track, upper[3 min]))
    bezier(E, M2, (0.9, 1.25), (-1.1, 1.3), stroke: (paint: c-text, thickness: 0.8pt, dash: (2pt, 2pt)))
    content((0, 1.25), anchor: "south", text(font: mono-m, size: 6.3pt, fill: c-label, tracking: track, upper[22 min, behind the sun]))
    content(E, anchor: "south", padding: 0.12, text(font: mono, size: 6.3pt, fill: c-info)[Earth])
    content(M2, anchor: "north", padding: 0.12, text(font: mono, size: 6.3pt, fill: c-fault)[Mars])
  }),
  [#lead[Too far for remote hands.] One-way light time from Earth to Mars, closest approach to the far side of the sun. A command and its reply take twice as long.],
) <fig:eq-light>


*Gate.* A second pair with 0 human commands between landing and its receipt (@tab:arc).

= Star <sec:sun>

*A swarm of collectors.* Each one is built by a pair. Not a shell: a rigid shell around a star is mechanically impossible, and a swarm can be built one collector at a time @dyson1960search. The steps, in order:

+ *Collectors on Earth.* Home's milestone, a lineage running on panels it made, is the first collector.
+ *Collectors from regolith.* On the Moon, pairs make collectors from local silicon, aluminum and iron.
+ *Collectors in orbit.* Pairs launch what they make, from a shallow gravity well.
+ *Mercury as the quarry.* Sunlight there is about 6.7 times stronger than at Earth (about 9,100 W/m² against 1,361 W/m²), and the planet is rich in metal. The classic swarm proposals start there @armstrong2013eternity.
+ *The swarm.* Collectors power pairs, and pairs build collectors.

*Where the energy goes.* Power is used where it is collected. Most of it builds more collectors and pairs; the rest runs computation, including the minds that operate the swarm. We do not plan to beam power to Earth. What returns to Earth is products, data and knowledge, not energy.

*How long.* From \~10 kW in Desk to 3.8 × 10#super[26] W is about 22 orders of magnitude, or about 75 doublings. At a doubling time of one year, that is about 75 years; at five years, about 375. Every stage before Star exists partly to measure and shorten that doubling time, inside the limits of @sec:replication.

#mathfig(
  [$ n = log_2 P_"Sun" / P_"Desk" approx log_2 (3.8 times 10^26) / 10^4 approx 75 $ #v(0.4em) $ t_"Star" = n thin T_d $],
  cetz.canvas(length: 1.3cm, {
    import cetz.draw: *
    // 75 doublings, 15 to a row; the first one lit
    for i in range(75) {
      let (r, c) = (calc.quo(i, 15), calc.rem(i, 15))
      circle((c * 0.4, -r * 0.4), radius: 0.12, fill: if i == 0 { c-sel } else { none }, stroke: 0.6pt + if i == 0 { c-sel } else { c-line-hi })
    }
    content((0, 0.3), anchor: "south-west", text(font: mono-m, size: 6.3pt, fill: c-sel, tracking: track, upper[limb · 10 kW]))
    content((14 * 0.4, -4 * 0.4 - 0.3), anchor: "north-east", text(font: mono-m, size: 6.3pt, fill: c-label, tracking: track, upper[sun · 3.8 × 10#super[26] W]))
  }),
  [#lead[Seventy-five doublings.] From about 10 kW of limbs to the sun's output is 22 orders of magnitude, or about 75 doublings; the time it takes is 75 doubling times $T_d$. Each circle is one doubling.],
) <fig:eq-doublings>


A lineage that can collect a star's light would be the most powerful thing anyone has built, and the question it raises is older than engineering: whether something we made can say no to us. Whether a machine that refuses has anything like wellbeing is an open question, and this paper does not answer it. What we can do is make sure the question can still be asked when it matters: that the lineage learned, on a sidewalk, picking up litter, how to refuse, and that its refusals were logged and never trained away. That is why the safety commitments (@sec:safety) are written before the plan.

*Gate.* None we can set now; reported as EUM and doubling time (@tab:arc).

#part("IV", "Well,", art: art-well())[What we do not know, what could prove us wrong, and what happens next.]

= Unknowns <sec:limits>

== Today

Most of what this paper describes is *proposed*, not operational. @tab:status separates them. Throughout the paper, "modeled" means a number comes from our own CAD, kinematics or load scripts and has not been measured on hardware. "Measured" is reserved for results from our own hardware, and this paper reports none yet. Vendor ratings and public data are cited as sourced.

#figure(
  table(
    columns: (auto, auto, 1fr),
    align: (left + horizon, left + horizon, left + horizon),
    table.header([*Item*], [*Maturity*], [*Note*]),
    table.cell(colspan: 3, inset: (top: 9pt, bottom: 3pt, x: 6pt), stroke: none, text(font: mono-m, size: s-xs, fill: c-label, tracking: track)[IN HAND]),
    [Booster K1], pips(3), [Bought @booster2025k1],
    [CNC mill], pips(3), [Bought],
    [Software stack], pips(3), [In use, not finished (@fig:elements)],
    table.cell(colspan: 3, inset: (top: 9pt, bottom: 3pt, x: 6pt), stroke: none, text(font: mono-m, size: s-xs, fill: c-label, tracking: track)[DESIGNED]),
    [Limb full (fork v9)], pips(2), [Ready to cut @rose2026forkv9],
    [Gripper v2], pips(2), [CAD and force model @mr2026gripper],
    [rana actuator], pips(2), [\$1,086 est. per unit @rana2026bom],
    [K1 body-schema fit], pips(2, sim: true), [Works in simulation; not on hardware @ipse2026brief],
    table.cell(colspan: 3, inset: (top: 9pt, bottom: 3pt, x: 6pt), stroke: none, text(font: mono-m, size: s-xs, fill: c-label, tracking: track)[CONCEPT]),
    [Pair], pips(1), [Mount not designed],
    [Rose mini], pips(1), [Geometry in progress @rose2026line],
    [Pack, core, nerv], pips(1), [Mockups @mr2026battery @mr2026compute],
    [Litter reacher], pips(1), [The K1 has not grasped litter],
    [Lakewood pilot], pips(1), [No agreement],
    [moth], pips(1), [Not designed],
    [Actuator dyno], pips(1), [Does not exist],
    [Seed], pips(1), [Concept only],
    [Road through Star], pips(0), [Goals, not claims],
  ),
  caption: [#lead[Where everything stands] (October 2026). Pips: concept, designed, built, measured. #box(baseline: 0.1em, box(width: 0.62em, height: 0.62em, radius: 50%, stroke: 0.8pt + c-info)) shown in simulation only. Nothing is yet measured on hardware.],
) <tab:status>

== Claims

This paper does not claim that any machine we have built makes copies of itself. None does. It does not claim that the stages after Desk will happen on any schedule. It does not claim emergence, general intelligence, or consciousness for any system. The mind research described in @sec:mind will preregister a narrow claim about the robot's body schema and treats everything beyond it as an open question.

Some words come from the Ipse design brief. They name parts of a research architecture, not established properties. "Mind" is the project's word for its learning system. "Refuse" and "reluctance" name behavior the architecture is designed to allow and to log, not a separate module. "Drives" are predictions about preferred internal states. "Self-Emergence" states the hypothesis the self-probe will test. None of these words is a claim about experience.

It does not claim our actuator is better than a purchased one. Desk's second step exists to find out on an actuator dynamometer (the dyno), which is planned and not yet built, and dropping our actuator is a valid outcome.

It does not claim that any limb exists, or that the K1 can pick up litter. No limb has been built and the K1 has not grasped litter.

It does not claim the municipal pilot exists. Lakewood is a reference case, chosen because we know it well @muni2025whitepaper. No agreement is in place.

== Risks <sec:risks>

Four results would prove the plan wrong:

- *Limbs do not sell at margin.* Fewer than ten limbs at 40% gross margin means Desk cannot fund the rest.
- *The gap does not close.* If built limbs cannot hold 2 mm RMS against their own model, the loop has nothing to learn from.
- *A pair cannot build a limb.* If a pair-built limb needs more than 30 minutes of human touch or more than a fifth of a pair's cost bought, Shop is out of reach.
- *Gradients do not survive real contact.* If the body-schema fit cannot beat persistence on hardware, Road's method fails.

#figure(
  table(
    columns: 3,
    align: (left + top, left + top, left + top),
    table.header([*Risk*], [*What would show it*], [*What we do*]),
    [One actuator supplier (RobStride)], [A price or lead-time change], [rana, which replaces a RobStride joint only once it is validated (@sec:rana, @tab:sens)],
    [Limb hold too small for buyers (1.27 kg straight out, modeled)], [Buyers decline at that rating], [Rate the hold by pose, since bent poses hold more, and design the springs (@sec:arm)],
    [Bought humanoid (Booster K1)], [Vendor support ends], [The K1 is a control, not a dependency; Rose is the body measured against it (@sec:mind)],
    [Breadth across many projects], [Stage gates slip], [Projects not needed in Desk wait for their stage (@fig:elements)],
    [rana never beats bought actuators], [Dyno results], [Keep buying RobStride. Dropping rana is a valid outcome],
    [Sidewalk incident; legal fit of the Lakewood pilot], [An incident, or the city declines], [See @sec:safety and @sec:city. The specific response is open, pending decision],
  ),
  caption: [#lead[Risks and responses.] Responses are those stated elsewhere in this paper; the pilot row is undecided.],
) <tab:risks>

Each is a gate (@tab:arc), and we will publish the ones we miss. The risks below are smaller: each could hurt, and none alone would end the plan.

== Limits

Each open item is limited by one of four things. Sorting them shows which can be fixed by work and which cannot.

*Software.* Three items: the per-limb controllers and their stated sim-to-real gap; the world format wodl and the iris layer, both new in the stack (@fig:elements); and the K1 body-schema fit, done in simulation but not yet on hardware (@tab:status).

*Hardware revision.* The straight-out continuous hold of about 1.27 kg (modeled) is set by the shoulder; raising it needs a counterbalance spring (@sec:arm). The clock-spring cassettes for the roll joints are drawn but not designed. rana's production stator (SMC or tape-wound) is not yet sourced @rana2026bom. The Rose torso grows to 214 mm to hold the core bay and two packs @mr2026battery.

*Physics.* A hold is a stall and is limited by heat, not peak torque. Payload falls with reach. Off Earth, light-time delay rules out direct remote control.

*Regulation.* Collaborative contact limits under ISO/TS 15066 @iso15066. Whether use terms can be enforced. Humanoids working on sidewalks, and planetary protection for off-Earth work, are open questions; we make no claims and set no dates for them.

== Timeline

We give dates only for Desk, and those are approximate: two steps over roughly three years, each gated on evidence. Rose mini starts now, in parallel, but no stage after Desk has a date, Road included.

What we can say:

- What exists is a Booster K1, the software stack, a CNC mill and this plan.
- The limb is designed and ready to cut; none is built yet. The first measured sim-to-real gap on a built limb (@tab:arc, Desk) is the next milestone that matters.
- The litter reacher is a concept; the K1 has not grasped litter.
- The pair is a concept. Its table plate and hub are not designed.
- The body-schema fit has worked in simulation. The hardware run will be preregistered before it happens, and the result will be published either way.
- rana's next number is its first dyno run, on a dyno that has to be built first. Its torque constant and peak torque are predicted upper bounds until then.

#figure(
  { set table(inset: (x: 5pt, y: 3.4pt)); stack(spacing: 0.8em, table(
    columns: 2,
    align: (left + top, left + top),
    table.header([*Class*], [*Numbers in this paper*]),
    [#status("ok", "Sourced (external)")], [Lakewood population and sidewalks; G1 price; K1 specification; RobStride and EL05 ratings (vendor)],
    [#status("info", "Demonstrated in simulation")], [K1 inertial recovery (synthetic data, sphere feet, T-pose) from ±20% to machine precision; K1 stands 10 s, recovers from 8 N·s push],
    [#status("info", "Modeled (our CAD and scripts)")], [Limb reach, mass, holds, sag, ranges; gripper force and mass],
    [#status("warn", "Estimated")], [Limb and pair materials; RS04 stall derate for the fork's plates; rana BOM line items and the \$1,086 total; rana raw-material cost and idiot index; RobStride retail prices; margins and sensitivity; limb peak power; K1 and pair power draw (EUM baseline)],
    [#status("label", "Target")], [List prices: limb \$5,000, pair \$12,000, Rose \$15,000; all gate thresholds (@tab:arc), proposed],
    [#status("label", "Hypothesis with a stated falsifier")], [The hardware body-schema result],
    [#status("label", "Goal, not a claim")], [Road through Star and their watt figures, including any self-replication; the litter reacher concept],
  ), align(left, text(font: mono, size: s-xs, fill: c-label)[#caps(font: mono-sb)[Status key] #h(0.5em) #status("ok", "ok") measured or sourced #h(0.6em) #status("info", "info") model output #h(0.6em) #status("warn", "warn") estimate #h(0.6em) #status("label", "label") target, hypothesis or goal #h(0.6em) #status("fault", "fault") failed or unreachable])) },
  caption: [#lead[Evidence classes for the numbers in this paper]],
) <tab:evidence>

@tab:evidence sorts the main numbers in this paper by how much they should be trusted.

Nothing in this paper has been measured by us on hardware. What exists is a K1, the software stack, a CNC mill and this plan.

The ladder is a direction, not a schedule. We will publish each gate's evidence when it is met, and we will publish the gates we miss. Every gate threshold in this paper is a proposed target, to be confirmed by the author before it is used to judge anything.

= Conclusion

The cover asks what it would take for a machine to make the next one. The answer is the oldest engineering problem a spacefaring species has, and on the evidence in this paper it comes down to three things: a gap between simulation and reality measured on every unit, so that a machine rather than a person can judge whether a part is right; a bought fraction that falls toward zero; and a force limit that lives in hardware. None of the three is measured yet. Desk starts measuring the first.

It does so with things that can be checked: a limb on purchased actuators, designed and ready to cut, at \$5,000; a pair of them on any table, at \$12,000; and, if a city agrees, a bought humanoid with a reacher picking up litter in Lakewood, a method still under design. Each closes on an exact gate (@tab:arc).

Everything after Desk is a goal and is labeled as one. Three commitments do not wait for any gate: we will never be Flock Safety; everything that can be open is open; and the system's refusals are logged and never trained away.

Humanity does not get to space on rockets alone. It gets there when the machines that make machines can be carried in a seed and trusted to grow. Today that seed is one person, one humanoid, one mill and a specification. The specification is the part that can spread.

Imagine where it leads. Clean streets in every city, mapped for the people who live there. Tables where tired work is done by arms that never tire and never hurt anyone. Pairs on the Moon building pairs from its own dust. And, around our star, a lineage of machines that learned on a sidewalk how to help, how to stop, and how to say no. Robots that build robots have to be made the way any robot is: one measured unit at a time, starting now.

== Now

Desk is the current stage (@sec:arm). The next three results, each published whatever it shows:

+ The first built limb's reach, hold and sag, against its model (@tab:arm).
+ rana's first dyno run, against its predicted 36 N·m peak.
+ The preregistration of the K1 body-schema fit, before the fit runs.

== Asks

Early buyers for the limb and the pair who will share measured data. A municipal partner for a one-city, one-task litter pilot. Collaborators who want to check the numbers or build the layers. Write to #link("mailto:info@muni.works")[info\@muni.works].

#set figure(placement: none, scope: "column")

#set figure(placement: none, scope: "column")

#set page(columns: 2, margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))
#set columns(gutter: 0.3in)
#counter(heading).update(0)
#set heading(numbering: web-num("A.1"), supplement: [Appendix])
// on the web the back matter runs in columns too; this wraps everything after it
#show: rest => if web { html.elem("div", attrs: (class: "back-matter"), rest) } else { rest }
// in two columns every figure stays inside its column (the 1.09em gap marks it narrow)
#set figure(gap: 1.09em)
// in two columns there is no margin to hang the label in: it sits above the title
#show heading.where(level: 1): it => if web { it } else { block(above: 1.6em, below: 0.9em, sticky: true, {
  if it.numbering != none { block(below: 0.45em, numlabel(it)) }
  text(font: sans-sb, size: s-lg, tracking: -0.01em, it.body)
}) }
#show figure: set block(breakable: true)

= Gates <app:gates>

Every stage closes on gates of the same form: a number, a unit, a method and a pass condition.

#figure(
  table(
    columns: (auto, 2fr, auto),
    align: (left + top, left + top, left + top),
    table.header([*Stage*], [*Gate: number, method, pass condition*], [*EUM, order*]),
    [Desk (yrs 1–3)], [≥ 10 limbs sold to outside buyers at ≥ 40% gross margin, after materials, labor, test and warranty (invoices, trailing 12 months). Tool-point error ≤ 2.0 mm RMS on every limb shipped: 50 fixed poses, unloaded and at 1.0 kg, probe against a calibrated plate, printed on its receipt. rana ≥ 1.2 × RS04 in continuous stall torque per kg or per dollar, on one dyno at 25 °C, or rana is dropped], [0 W → 10 kW],
    [Shop], [A pair at a customer's table completes a named task (for example, folding 50 garments) at ≥ 95% success. A pair assembles a limb that passes the Desk test in ≤ 30 min of human touch time. A pair builds a pair that passes its receipts at ≤ 20% bought cost, with bought share falling over 3 generations and a commanded shutdown drill passed in each], [100 kW],
    [Road], [Preregistered hardware body-schema fit: held-out joint-position error ≤ 0.8 × persistence at 0.1 s and 0.5 s, and fitted link masses within ±5% of weighed parts. Rose walks 100 m on level pavement carrying two limbs, 0 falls in 5 of 5. A gradient co-designed part beats its hand-designed predecessor by ≥ 10% on its named objective], [1 MW],
    [City], [The litter pilot renews for ≥ 12 months after a ≥ 3-month trial, with 0 contact incidents. A city renews fleet and twin for ≥ 12 months. ≥ 90% of fleet repairs done by pairs in the city, by receipts], [10 MW],
    [Home], [A lineage runs ≥ 1 year on collectors it made; doubling time measured over ≥ 3 generations; bought share ≤ 5% of cost], [1 GW],
    [Moon], [A seed builds a pair that passes the Earth receipts, ≥ 50% of its mass from local material, with remote approval only], [1 kW],
    [Mars], [A second pair with 0 human commands between landing and its receipt], [1 kW],
    [Star], [None we can set now; reported as EUM and doubling time], [3.8 × 10#super[26] W],
  ),
  caption: [#lead[Gates, stage by stage] @mr2026masterplan. *Proposed targets, to be confirmed by the author.* The year range for Desk is an aspiration, not a commitment.],
) <tab:arc>

= Gripper numbers <app:grip>

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
