// Research post: Gate-Pinned State Memory (GPSM), zero-context compiled memory in tang and frog
// Same treatment as paper/vision/vision.typ (muon "Pilot Light", dark), trimmed to what this post uses.
// One source, two outputs: `--features html --format html --input target=html` builds the web
// post; without the input it builds the PDF. See web/scripts/build-posts.mjs.

#let web = sys.inputs.at("target", default: "pdf") == "html"
#show math.equation: it => if web { html.elem("span", attrs: (class: "math"), html.frame(it)) } else { it }
#show math.equation: set text(font: "New Computer Modern Math", fallback: true)
#show math.equation: it => { show regex("[A-Za-z0-9]"): set text(font: "IBM Plex Sans"); it }

// ---- color roles (as vision.typ) -------------------------------------------
#let c-bg      = rgb("#0b0907")
#let c-surface = rgb("#15110d")
#let c-line    = rgb("#2a2119")
#let c-line-hi = rgb("#4a3a2a")
#let c-text    = rgb("#e8e1d2")
#let c-label   = rgb("#a08d70")
#let c-sel     = rgb("#ff5f00")
#let c-ok      = rgb("#39ff14")
#let c-warn    = rgb("#ffd400")
#let c-fault   = rgb("#ff2052")
#let c-info    = rgb("#00f0ff")

// product names in their family color, as in the vision paper's periodic table
#let brand-names = (("tang", "make"), ("frog", "mind"), ("pond", "mind"))
#let fam-color = (make: c-ok, mind: c-sel)
#let brand-re = regex("(?i)\\b(tang|frog|pond)\\b")
#metadata(brand-names) <brand-names>
#let brand-family(name) = brand-names.find(n => n.at(0) == lower(name)).at(1)
#show brand-re: it => if web { it } else { text(fill: fam-color.at(brand-family(it.text)), it) }

// ---- type -------------------------------------------------------------------
#let sans    = ("IBM Plex Sans",)
#let sans-m  = ("IBM Plex Sans Medm", "IBM Plex Sans")
#let sans-sb = ("IBM Plex Sans SmBld", "IBM Plex Sans")
#let mono    = ("IBM Plex Mono", "IBM Plex Sans")
#let mono-m  = ("IBM Plex Mono Medm", "IBM Plex Sans Medm")
#let mono-sb = ("IBM Plex Mono SmBld", "IBM Plex Sans SmBld")
#let track = 0.12em
#let s-xs = 6.5pt
#let s-sm = 7.5pt
#let body-size = 9pt
#let s-md = 10.5pt
#let s-lg = 13pt
#let s-display = 40pt
#let nb = sym.space.nobreak.narrow

#let caps(body, size: s-xs, fill: c-label, font: mono-m) = text(font: font, size: size, tracking: track, fill: fill, upper(body))
#let lead(body) = text(font: sans-m, fill: c-text, body)
#let aside(title, body) = if web { html.elem("aside", attrs: (class: "aside"), { html.elem("div", attrs: (class: "aside-title"), title); body }) } else {
  pad(left: 0.35in, right: 0.35in, block(width: 100%, inset: (left: 12pt, y: 2pt), stroke: (left: 0.6pt + c-line-hi), above: 2.2em, below: 2.2em, breakable: false, {
    set text(size: s-sm, fill: c-label)
    set par(spacing: 1.1em, leading: 0.68em)
    show strong: set text(fill: c-text)
    block(below: 0.9em, text(font: mono-m, size: s-xs, fill: c-sel, tracking: track, upper([Aside · ] + title)))
    body
  }))
}

#set document(title: "Gate-Pinned State Memory", author: "Cam Pedersen (Municipal Robotics)")

#let hf-out() = { let m = page.margin; let l = if type(m) == dictionary { m.at("left", default: m.at("x", default: 0.75in)) } else { 0.75in }; calc.max(0in, l - 0.7in) }

#set page(
  paper: "us-letter",
  fill: c-bg,
  foreground: place(center + horizon, rotate(-32deg, text(font: mono-sb, size: 128pt, tracking: 0.12em, fill: c-sel.transparentize(93%), "DRAFT"))),
  margin: (x: 1.85in, top: 1.1in, bottom: 1.05in),
  header-ascent: 0.45in,
  footer-descent: 0.45in,
  numbering: "1",
  header: context {
    if counter(page).get().first() > 1 {
      pad(x: -hf-out(), grid(columns: (1fr, auto), caps[Gate-Pinned State Memory], caps[Municipal Robotics]))
    }
  },
  footer: context {
    if counter(page).get().first() > 1 {
      pad(x: -hf-out(), grid(columns: (1fr, auto, 1fr),
        caps[Draft · October 2026],
        text(font: mono-m, size: s-xs, fill: c-label, number-width: "tabular")[#{ let n = counter(page).get().first(); if n < 10 { "0" + str(n) } else { str(n) } } / #counter(page).final().first()],
        align(right, caps[Research])))
    }
  },
)

#set text(font: sans, weight: 400, size: body-size, fill: c-text, fallback: false, lang: "en", region: "us", hyphenate: false, costs: (runt: 1000%, widow: 200%, orphan: 200%))
#set par(justify: false, leading: 0.84em, spacing: 1.4em, linebreaks: "optimized")
#let web-num(pattern) = if web { (..n) => html.elem("span", attrs: (class: "sec-num"), numbering(pattern, ..n)) } else { pattern }
#set heading(numbering: web-num("1.1"))
#show strong: set text(font: sans-sb)
#show link: set text(fill: c-text)
#show heading: set text(font: sans, weight: 400, fill: c-text, hyphenate: false)
#let numlabel(it) = text(font: mono-m, size: s-xs, fill: c-label, tracking: track, counter(heading).display(it.numbering))
#show heading.where(level: 1): it => if web { it } else { block(above: 2.2em, below: 1.1em, sticky: true, width: 100%, {
  if it.numbering != none { place(left, dx: -1.15in, dy: 0.32em, box(width: 1.0in, align(right, numlabel(it)))) }
  text(font: sans-sb, size: s-lg, tracking: -0.01em, it.body)
}) }
#show heading.where(level: 2): it => if web { it } else { block(above: 1.65em, below: 0.55em, sticky: true, {
  if it.numbering == none { return text(font: sans-m, size: s-md, it.body) }
  place(left, dx: -1.15in, dy: 0.22em, box(width: 1.0in, align(right, numlabel(it))))
  text(font: sans-m, size: s-md, it.body)
}) }
#show raw: set text(font: mono, size: 1em)
#set list(marker: box(baseline: -0.07em, rect(width: 0.36em, height: 0.36em, fill: c-label, stroke: none)), indent: 0em, body-indent: 0.85em, spacing: 1.15em)

#set table(stroke: (x, y) => (
    top: if y == 0 { 0.6pt + c-line-hi } else if y == 1 { 0.6pt + c-line-hi } else { 0.35pt + c-line },
    bottom: 0.6pt + c-line-hi,
  ), inset: (x: 6pt, y: 4.6pt))
#show table: set text(font: sans, size: s-sm, number-width: "tabular", hyphenate: false)
#show table: set par(leading: 0.62em, justify: false)
#show table.cell.where(x: 0): it => if it.y == 0 { it } else { set text(font: sans-m, fill: c-text); it }
#show table.cell.where(y: 0): it => {
  set text(font: mono-m, size: s-xs, fill: c-label, tracking: track)
  show strong: s => upper(s.body)
  it
}
#set figure(placement: none, gap: 1.1em)
#show figure: set block(above: 1.9em, below: 1.9em)
#show figure.where(kind: table): set figure.caption(position: top)
#show figure.caption: it => {
  set par(leading: 0.64em, justify: false)
  set align(left)
  block(width: 100%, text(font: sans, size: s-sm, fill: c-label, hyphenate: false)[
    #caps(if it.kind == table [Table] else [Fig] + nb + it.counter.display(), font: mono-sb, fill: c-text)#h(0.7em)#it.body
  ])
}
#show cite: set text(fill: c-label)

// ---- figures ----------------------------------------------------------------
#import "@preview/cetz:0.4.2"
// Web figures: drawn bodies become inline SVG via html.frame (as vision.typ); tables stay <table>s.
#show figure: it => if web and it.kind != table {
  let lab = if it.has("label") { str(it.label) } else { "" }
  let body = html.frame(block(width: 4.8in, fill: c-bg, inset: 6pt, { set math.equation(alt: "in-frame"); it.body }))
  let cap = if it.caption != none { html.elem("figcaption", [#html.elem("span", attrs: (class: "fig-label"))[Fig #context it.counter.display(it.numbering)] #it.caption.body]) }
  html.elem("figure", attrs: if lab == "" { (class: "narrow") } else { (id: lab, class: "narrow") }, { body; cap })
} else { it }
#let lbl(pos, body, anchor: "center", fill: c-label, size: s-xs, font: mono-m) = cetz.draw.content(pos, anchor: anchor, text(font: font, size: size, fill: fill, tracking: 0.04em, body))

// a 16-row miniature of a 128×128 state, with its key bar on the left
#let delta-grid(pinned: true, quiet: false, title: none, cell: 0.21, cols: 12) = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let rows = 16
  let hit = 5
  let h = rows * cell
  // key bar
  for r in range(rows) {
    let y = h - (r + 1) * cell
    let f = if pinned { if r == hit { c-sel } else { c-surface } } else {
      c-label.transparentize(100% - (20% + calc.rem(r * 37 + 11, 61) * 1%))
    }
    rect((0, y), (cell, y + cell), fill: f, stroke: 0.3pt + c-line)
  }
  let x0 = cell * 2.2
  for r in range(rows) {
    let y = h - (r + 1) * cell
    for c in range(cols) {
      let base = 30% + calc.rem(r * 13 + c * 7, 40) * 1%
      let f = if pinned {
        if r == hit { c-sel.transparentize(calc.rem(c * 29, 45) * 1%) } else { c-label.transparentize(100% - base * (if quiet { 0.5 } else { 0.7 })) }
      } else {
        // decayed and smeared: every row dims and shifts a little
        let sm = calc.rem(r * 5 + c * 3, 11) * 1%
        c-label.transparentize(100% - base * 0.45 - sm)
      }
      rect((x0 + c * cell, y), (x0 + (c + 1) * cell, y + cell), fill: f, stroke: 0.3pt + if quiet { c-bg } else { c-line })
    }
  }
  if not quiet {
    let w = x0 + cols * cell
    lbl((cell / 2, h + 0.22), [k])
    lbl((x0 + cols * cell / 2, h + 0.22), [S#sub[t]])
    if title != none { lbl((w / 2, -0.32), title, fill: c-text) }
    if pinned { lbl((w + 0.12, h - (hit + 0.5) * cell), [← row i = v], anchor: "west", fill: c-sel) }
    else { lbl((w + 0.12, h / 2), [all rows ×g,\ all shift], anchor: "west") }
  }
})

#let fig-gates() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let sx = 1.12            // cm per decade of 1 − g
  let xo = 2.15            // left gutter for row labels
  let X(d) = xo + d * sx   // d = −log10(1 − g)
  let L(g) = -calc.log(1 - g, base: 10)
  let rows = (
    ("Early", "layers 0–10", (0.04, 0.08), (0.99967, 0.999994)),
    ("Mid", "layers 12–26", (0.06, 0.35), (0.99955, 0.999999)),
    ("Late", "layers 28–44", (0.05, 0.15), (0.99981, 0.999997)),
  )
  let top = 3.0
  // axis
  line((X(0), 0), (X(7), 0), stroke: 0.6pt + c-line-hi)
  for d in range(8) {
    line((X(d), 0), (X(d), -0.08), stroke: 0.6pt + c-line-hi)
    line((X(d), 0), (X(d), top), stroke: 0.3pt + c-line)
    lbl((X(d), -0.3), if d == 0 [1] else [10#super[#("−" + str(d))]])
  }
  lbl((X(3.5), -0.72), [1 − g, per-channel minimum over the run (log)])
  lbl((X(1), top + 0.25), [g = 0.9])
  lbl((X(4), top + 0.25), [g = 0.9999])
  // the unreachable line
  line((X(7.25), -0.1), (X(7.25), top + 0.1), stroke: (paint: c-fault, thickness: 0.6pt, dash: "dashed"))
  lbl((X(7.25), top + 0.55), [exact RAM needs 1 − g = 0\ (never reached)], fill: c-fault, anchor: "south")
  for (i, (name, layers, med, best)) in rows.enumerate() {
    let y = top - 0.55 - i * 0.95
    lbl((0, y + 0.12), name, anchor: "west", fill: c-text)
    lbl((0, y - 0.18), layers, anchor: "west")
    // median channel: hollow
    rect((X(L(med.at(0))), y - 0.07), (calc.max(X(L(med.at(1))), X(L(med.at(0))) + 0.06), y + 0.07), fill: none, stroke: 0.7pt + c-ok)
    // best channel: solid
    rect((X(L(best.at(0))), y - 0.07), (X(L(best.at(1))), y + 0.07), fill: c-ok, stroke: none)
    if i == 0 {
      lbl((X(L(med.at(1))) + 0.1, y + 0.28), [median channel], anchor: "west", fill: c-ok)
      lbl((X(L(best.at(0))), y + 0.28), [best channel], anchor: "west", fill: c-ok)
    }
    if i == 1 {
      circle((X(6), y), radius: 0.09, fill: c-bg, stroke: 1pt + c-ok)
      lbl((X(6) - 0.15, y - 0.3), [layer 14, head 33, channel 83: 1 − g = 10#super[−6]], anchor: "north-east", fill: c-text)
    }
  }
})

#let fig-where() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let sh = 0.07  // slab height
  // host column: 48 slabs, attention every fourth
  let hx = 2.0
  for i in range(48) {
    let y = i * sh
    let att = calc.rem(i, 4) == 3
    rect((hx, y), (hx + 1.6, y + sh * 0.78), fill: if att { c-label } else { c-surface }, stroke: 0.3pt + c-line-hi)
  }
  let H = 48 * sh
  lbl((hx + 0.8, -0.3), [host model], fill: c-text)
  lbl((hx + 0.8, -0.6), [Qwen3-Coder-Next, 48 layers])
  lbl((hx - 0.15, H * 0.8), [GDN ×36\ (dark)], anchor: "east")
  lbl((hx - 0.15, H * 0.45), [attention ×12\ (light, every 4th)], anchor: "east")
  // chip column: 12 slabs, orange
  let cx = 9.6
  let csh = 0.2
  let cy0 = 0.6
  for i in range(12) {
    let y = cy0 + i * csh
    rect((cx, y), (cx + 1.6, y + csh * 0.78), fill: c-sel.transparentize(15%), stroke: none)
  }
  let cH = cy0 + 12 * csh
  lbl((cx + 0.8, -0.3), [compiled store], fill: c-sel)
  lbl((cx + 0.8, -0.6), [12 chip layers, +1,920 dims])
  // engine box
  let ex = 5.4
  rect((ex, H - 0.9), (ex + 2.2, H - 0.1), fill: c-surface, stroke: 0.7pt + c-line-hi)
  lbl((ex + 1.1, H - 0.5), [engine], fill: c-text)
  // tool call leaves top of host
  line((hx + 0.8, H + 0.05), (hx + 0.8, H + 0.35), (ex + 0.6, H + 0.35), (ex + 0.6, H - 0.1), mark: (end: "stealth", fill: c-text), stroke: 0.6pt + c-text)
  lbl((hx + 0.95, H + 0.55), [tool call], anchor: "west", fill: c-text)
  // engine → chip tokens → chip column
  line((ex + 2.2, H - 0.4), (cx + 0.8, H - 0.4), (cx + 0.8, cH + 0.08), mark: (end: "stealth", fill: c-sel), stroke: 0.7pt + c-sel)
  lbl((ex + 2.35, H - 0.3), [chip tokens], anchor: "south-west", fill: c-sel)
  lbl((cx + 0.8, cH + 0.45), [never the base layers,\ the LM head or the KV cache], anchor: "south", fill: c-label)
  // result returns as text
  line((cx, cy0 + 0.6), (hx + 1.75, cy0 + 0.6), mark: (end: "stealth", fill: c-text), stroke: (paint: c-text, thickness: 0.6pt, dash: "dashed"))
  lbl(((cx + hx + 1.6) / 2, cy0 + 0.82), [result, as text], fill: c-text)
  // state block
  let sx = 5.0
  rect((sx, cy0 - 0.5), (sx + 3.0, cy0 + 0.25), fill: none, stroke: (paint: c-line-hi, thickness: 0.6pt, dash: "dotted"))
  lbl((sx + 1.5, cy0 + 0.02), [store state, per session: 26 MB])
  lbl((sx + 1.5, cy0 - 0.24), [user's blob file: 512 KiB])
})

#let fig-latency() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let xo = 1.5
  let sx = 2.6  // cm per decade, from 10 ms
  let X(ms) = xo + (calc.log(ms, base: 10) - 1) * sx
  let ops = (("put", 1650, 51), ("get", 1270, 40), ("list", 1020, 25))
  let top = 3.4
  for (ms, t) in ((10, [10 ms]), (100, [100 ms]), (1000, [1 s])) {
    line((X(ms), -0.05), (X(ms), top), stroke: 0.3pt + c-line)
    lbl((X(ms), -0.3), t)
  }
  line((xo, 0), (X(2500), 0), stroke: 0.6pt + c-line-hi)
  for (i, (op, a, t)) in ops.enumerate() {
    let y = top - 0.5 - i * 1.05
    lbl((xo - 0.2, y - 0.17), op, anchor: "east", fill: c-text, size: s-sm)
    rect((xo, y), (X(a), y + 0.24), fill: c-label.transparentize(40%), stroke: none)
    lbl((X(a) + 0.1, y + 0.12), [#a ms], anchor: "west")
    rect((xo, y - 0.42), (X(t), y - 0.18), fill: c-ok, stroke: none)
    lbl((X(t) + 0.1, y - 0.3), [#t ms], anchor: "west", fill: c-ok)
  }
  rect((xo, top + 0.05), (xo + 0.3, top + 0.2), fill: c-label.transparentize(40%), stroke: none)
  lbl((xo + 0.4, top + 0.12), [author's PyTorch stack], anchor: "west")
  rect((xo + 4.2, top + 0.05), (xo + 4.5, top + 0.2), fill: c-ok, stroke: none)
  lbl((xo + 4.6, top + 0.12), [tang chip window], anchor: "west", fill: c-ok)
})

#let fig-arch() = cetz.canvas(length: 1cm, {
  import cetz.draw: *
  let W = 3.25
  let xs = (0.0, 4.47, 8.94)
  let top = 5.0
  let bot = 0.0
  let mid(k) = (xs.at(k) + W + xs.at(k + 1)) / 2
  let tier(x, title, sub) = {
    rect((x, bot), (x + W, top), fill: none, stroke: 0.6pt + c-line-hi)
    lbl((x + W / 2, top - 0.32), title, fill: c-text, size: s-sm, font: mono-sb)
    lbl((x + W / 2, top - 0.62), sub)
  }
  let part(x, y, h, title, sub, hot: false) = {
    rect((x + 0.2, y), (x + W - 0.2, y + h), fill: if hot { c-sel.transparentize(82%) } else { c-surface }, stroke: 0.6pt + if hot { c-sel } else { c-line-hi })
    lbl((x + W / 2, y + h - 0.3), title, fill: if hot { c-sel } else { c-text })
    if sub != none { lbl((x + W / 2, y + h - 0.6), sub) }
  }
  tier(xs.at(0), [frog], [the client])
  tier(xs.at(1), [tang serve], [the session])
  tier(xs.at(2), [tang engine], [the model])
  part(xs.at(0), 2.6, 1.2, [agent], [Memory tool])
  part(xs.at(0), 0.3, 1.5, [store file], [512 KiB], hot: true)
  lbl((xs.at(0) + W / 2, 0.65), [per project])
  part(xs.at(1), 2.6, 1.2, [/v1/memory], [load · op · end])
  part(xs.at(1), 0.3, 1.5, [session], [per cache key])
  lbl((xs.at(1) + W / 2, 0.65), [zeroed at end])
  part(xs.at(2), 3.0, 0.85, [host model], [layers, KV cache])
  part(xs.at(2), 1.65, 0.85, [chip window], [12 compiled layers], hot: true)
  part(xs.at(2), 0.3, 0.85, [pinned heads], [store state, 26 MB], hot: true)
  let solid(c) = (mark: (end: "stealth", fill: c), stroke: 0.6pt + c)
  let dashed(c) = (mark: (end: "stealth", fill: c), stroke: (paint: c, thickness: 0.6pt, dash: "dashed"))
  // frog <-> serve: op and result
  line((xs.at(0) + W - 0.2, 3.4), (xs.at(1) + 0.2, 3.4), ..solid(c-text))
  lbl((mid(0), 3.62), [op])
  line((xs.at(1) + 0.2, 2.95), (xs.at(0) + W - 0.2, 2.95), ..dashed(c-text))
  lbl((mid(0), 2.72), [result])
  // frog <-> serve: the store file
  line((xs.at(0) + W - 0.2, 1.3), (xs.at(1) + 0.2, 1.3), ..solid(c-sel))
  lbl((mid(0), 1.52), [load], fill: c-sel)
  line((xs.at(1) + 0.2, 0.85), (xs.at(0) + W - 0.2, 0.85), ..dashed(c-sel))
  lbl((mid(0), 0.62), [export], fill: c-sel)
  // serve -> engine: ops become chip tokens; the store goes to the pinned heads
  line((xs.at(1) + W - 0.2, 3.2), (xs.at(2) + 0.2, 2.07), ..solid(c-text))
  lbl((mid(1), 3.25), [chip\ tokens], anchor: "south")
  line((xs.at(1) + W - 0.2, 1.05), (xs.at(2) + 0.2, 0.72), ..solid(c-sel))
  // the chip window reads and writes the pinned heads
  line((xs.at(2) + W / 2, 1.65), (xs.at(2) + W / 2, 1.15), mark: (start: "stealth", end: "stealth", fill: c-sel), stroke: 0.6pt + c-sel)
  // the separation between store traffic and the host model
  line((xs.at(2) + 0.2, 2.8), (xs.at(2) + W - 0.2, 2.8), stroke: (paint: c-fault, thickness: 0.7pt, dash: "dashed"))
})

// ---- title page ---------------------------------------------------------------
#if not web {page(margin: (x: 0.75in, top: 0.7in, bottom: 0.65in))[
  #set par(leading: 0.5em)
  #grid(columns: (auto, 1fr, auto), column-gutter: 0.8em, align: horizon,
    caps(fill: c-text)[Municipal Robotics · Research],
    line(length: 100%, stroke: 0.5pt + c-line-hi),
    caps[Draft · October 2026])
  #v(1.6em)
  #image("images/muni-logo.svg", width: 0.95in)
  #v(0.2in)
  #box(height: 2.0in, width: 100%, align(right + horizon, delta-grid(quiet: true, cell: 0.16, cols: 16)))
  #v(0.2in)
  #move(dx: -0.045 * s-display, text(font: sans-sb, size: s-display, tracking: -0.02em)[Gate-Pinned\ State Memory])
  #v(0.5em)
  #line(length: 22%, stroke: 2pt + c-label)
  #v(0.7em)
  #text(size: s-lg, fill: c-label)[Exact storage in a model's recurrent state, and how far a model gets using it]
  #v(1fr)
  #caps[Cam Pedersen · Municipal Robotics]
]}

#let abstract-text = [
  A coding agent remembers by pasting text into its prompt, and pays for it on every turn. Divyansh Agrawal's qwenputer showed another way: a Gated DeltaNet head with its gates pinned is exact, addressable RAM, and a filesystem can be compiled into extra layers of a model without training @agrawal2026qwenputer. We call the method Gate-Pinned State Memory (GPSM), checked every link of it on our own stack, and then measured whether it helps an agent remember.

  The storage holds up. tang's kernel stores and reads one-hot rows bitwise over a 50,000-step free run, while no trained head qualifies: on GLM-5.3-Flash, none of 8,192 channels per layer holds its gate at exactly 1. On an RTX 3090 the author's model drove its store through tool calls and read files back from a small saved file in an empty conversation, with its output on prose and code within the noise floor of the base model. tang ran the compiled layers bitwise in its reference math, emitted all 168 of their tensors bitwise from its own compiler, and served them from a CUDA window inside the engine at about 0.9 ms per token. The author's compiled CPU also reproduced: 39 of 40 large multiplications against 22 for the base model.

  As memory for an agent, it does not yet work. On a cross-session benchmark (LoCoMo, 125 questions) the GPSM model scored an answer F1 of 0.05, below a model with no memory at all (0.10), while retrieval over the old transcripts scored 0.44 at a similar prompt size. On 60 exact facts it recalled 23 percent against 92 percent for retrieval. The store was not the problem: when the right file existed and the model fetched it, the exact text came back. The failures were in the model's policy: what it chose to write, a store it filled and could not reclaim, and lookups that looped until its budget ran out. GPSM gives exact storage that costs no prompt tokens to hold; turning that into memory needs a better policy for using it.
]
#if web {
  html.elem("section", attrs: (class: "abstract", id: "abstract"), {
    html.elem("h2", attrs: (class: "abstract-title"), [Abstract])
    abstract-text
  })
} else {page(margin: (x: 0.75in, top: 1.1in, bottom: 1.05in))[
  #block(width: 4.9in)[
    #heading(outlined: false, numbering: none)[Abstract]
    #set text(size: 10pt)
    #set par(leading: 0.8em, spacing: 1.25em)
    #abstract-text
  ]
]}

= Motivation

A coding agent's memory today is text. Notes, summaries and retrieved chunks go into the prompt, are prefilled on every turn, and compete with code for the context window. On a 16 GB laptop running a 4B model, context is the scarcest thing there is. The alternatives in the literature each pay somewhere else. KV caches are exact but large and tied to one model @yao2024cacheblend. Learned fixed-size states cost no context but forget @stuffedmamba2024 @behrouz2025titans. Test-time training writes facts into weights, at the price of optimizer state and lost general ability @beyondppl2026.

Gated DeltaNet (GDN) updates a matrix state per head with the delta rule @yang2024gdn:

$ S_t = g_t S_(t-1) + beta_t k_t (v_t - S_(t-1)^top k_t)^top $

With the decay $g = 1$, the write strength $beta = 1$ and a one-hot key $k = e_i$, the update replaces row $i$ of $S$ with $v$ and leaves every other row alone.

#figure(
  grid(columns: 2, column-gutter: 0.5in, delta-grid(pinned: false, title: [trained head: g < 1, dense k]), delta-grid(pinned: true, title: [pinned head: g = 1, β = 1, one-hot k])),
  caption: [#lead[The delta rule, drawn.] Sixteen rows stand for a 128 × 128 head state. A trained head decays every row and smears the write across all of them. A pinned head with a one-hot key replaces exactly one row and leaves the rest untouched.],
)

A one-hot query reads row $i$ back. Compile weights that produce those gates and keys, and the recurrent state is RAM. We call fixing the gates this way _gate pinning_, a head built this way a _pinned head_, and memory kept in pinned heads *Gate-Pinned State Memory* (GPSM). The gates are pinned, not the state: rows change when they are rewritten, and stay exactly as written until then. qwenputer @agrawal2026qwenputer compiled a RISC-V CPU and a 512-file store into 12 extra layers in front of Qwen3.5 4B, without training. It reported a KL divergence to the base model of $6 times 10^(-4)$, and recall of facts across chats 90 percent of the time, against 30 percent when the old chats were pasted back in.

tang is our inference engine. It runs Qwen3-Coder-Next, a mixture of experts with about 3B parameters active per token and a hybrid of 36 GDN layers and 12 attention layers, and already snapshots the fixed-size recurrent state to disk. Its GDN kernel is specified for the shapes of Flash-Next, another Qwen hybrid tang serves, with 16 key and 48 value heads per layer; Coder-Next and Qwen3.5 4B have 32 value heads. frog, our coding agent, talks to it. So the question was concrete: what of this transfers to an engine we own?

= Exact RAM in tang's kernel

tang pins the GDN recurrence in a CPU specification that the CUDA kernel is verified against bitwise. We tested the claim there.

- *The gates reach exactly 1.* Gate pre-activations of $a + "dt" = -30$ for the decay and $b = 40$ for the write strength round to exactly `1.0f`.
- *A write into an empty row stores $v$ bitwise* and touches no other row of any of a Flash-Next layer's 48 value heads.
- *An overwrite stores $"fma"(1, v - s, s)$*, which is exactly $v$ only when $v - s$ is exact. On random floats, 8,643 overwrites were exact and 15,933 were off by one rounding. Stored values must be integers below $2^24$.
- *A 50,000-token free run*, driving the kernel as a register machine, stayed bitwise exact on the CPU and on an RTX 3090.
- *One token with $g < 1$ ends exactness for good.* It scales every row by a non-power-of-two, and the integers turn fractional.

= Trained heads cannot do it

So a pretrained head can be borrowed for storage only if its gate is bitwise 1 on every token. We measured the real gates of GLM-5.3-Flash, whose recurrent layers use a per-channel-gated delta rule, over 427 tokens and all 34 recurrent layers.

#figure(
  table(
    columns: 5,
    align: (left, right, right, right, right),
    table.header([*Layers*], [*Mean g*], [*Best channel min g*], [*Mean / max beta*], [*At 1.0 every token*]),
    [Early (0–10)], [0.51–0.68], [0.999994], [0.54–0.71 / ≤0.997], [0 of 8,192],
    [Mid (12–26)], [0.73–0.87], [0.999999], [0.64–0.77 / ≤1.000], [0 of 8,192],
    [Late (28–44)], [0.67–0.79], [0.999997], [0.55–0.84 / 1.000], [0 of 8,192],
  ),
  caption: [#lead[Gates of a trained hybrid model.] No channel qualifies as exact storage. Measured on GLM-5.3-Flash UD-Q4_K_XL, RTX 3090.],
)

A few channels are remarkably slow. Between 1 and 8 percent per layer have a mean gate above 0.999, and the best has a mean gate of $1 - 2 times 10^(-7)$, a half-life from the gate alone of about three million tokens. #figure(fig-gates(),
  caption: [#lead[No channel reaches 1.] Each bar is a range across the layers in its group: hollow for the median channel's minimum gate over the run, solid for the best channel's. GLM-5.3-Flash on an RTX 3090 over 427 tokens; per-layer data was not kept.],
)

But they are analog registers, and they leak. Exact storage needs dedicated pinned heads with compiled gates, which is the qwenputer design and the basis of GPSM.

= Two dead ends

The same structure suggests faster prefill. Writing context straight into state cannot work: a trained head can only read the state its own prefill would have produced, so an integer written into a row means nothing to it. We tested the two ideas that could work, and neither does.

*Chunked kernels.* tang steps the GDN recurrence one token at a time. A chunked tensor-core kernel has about 35 times the throughput on a 512-token window, but the recurrence is only 4.5 percent of a prefill window on an RTX 3090, and 5.5 percent on an H200. Prefill time lives in attention and the MoE layers.

*Precomputed chunk maps.* The delta rule is affine in $S$, so a chunk of text collapses to one matrix pair per head. If a chunk's keys and values did not depend on earlier context, prefill would be a few matrix products. We spliced context-free chunks into real GLM-5.3-Flash runs.

#figure(
  table(
    columns: 4,
    align: (left, right, right, right),
    table.header([*Spliced chunk*], [*State error, early / mid / late*], [*KL, mean / max (nats)*], [*Top-1 agree*]),
    [4], [0.34 / 0.65 / 0.54], [0.095 / 0.80], [30 / 32],
    [8], [0.30 / 0.62 / 0.46], [0.103 / 1.13], [27 / 32],
    [12], [0.44 / 0.98 / 0.60], [0.111 / 0.89], [31 / 32],
    [Skip (control)], [0.5–0.7], [0.30–0.58 / 3.7–5.5], [26–28 / 32],
  ),
  caption: [#lead[Context-free chunk precomputation.] KL is over the 32 tokens after each spliced chunk. About 0.1 nat per token: better than skipping the chunk, not lossless.],
)

Mid and late layers depend on context, not position. Fixing the error means recomputing most of the depth, which is prefill again. Instant restore of _old_ context is what tang already ships: a resumed 7,000-token session prefills in about 0.1 s from the snapshot disk tier.

= GPSM, end to end

== An engine-driven store

@fig:arch shows the whole path. frog keeps the store as a file on the user's disk and gives the model a `Memory` tool. tang serve loads the file into a session's pinned heads at the start and exports it after every write. Inside the engine, store operations run as their own windows through the compiled layers, apart from the host model's layers and KV cache.

#figure(fig-arch(),
  caption: [#lead[Architecture.] Store traffic, in orange, never passes through the host model: operations run as chip tokens through the compiled layers, which read and write the pinned heads. The red line marks that separation. The store file moves only between the user's disk and a live session.],
) <fig:arch>

We first built the store without compiled layers: a key-value file store in dedicated GDN heads of one layer, written and read through the real kernel with one-hot keys and pinned gates. The layout follows the author's: a hashed directory with linear probing, plus content rows of token ids. Values stay below $2^18$, tighter than the kernel's $2^24$, because a read passes the output norm and a division, about eight roundings.

#figure(
  table(
    columns: 4,
    align: (left, left, right, right),
    table.header([*Model shape*], [*Heads, k / v*], [*Directory*], [*Content*]),
    [Flash-Next], [16 / 48, tiled], [1,024 slots, 512 files], [65,024 tokens · 768 KiB],
    [Coder-Next, Qwen3.5 4B], [16 / 32, grouped], [512 slots, 256 files], [65,024 tokens · 512 KiB],
  ),
  caption: [#lead[Store capacity.] The store takes 4 of a layer's 16 key heads and their value heads, a quarter of one GDN layer. The heads column gives the whole layer.],
)

A 512-file fill and a 10,000-operation random workload, checked against a hash map, both pass. The store persists through the bytes tang's snapshot already carries. It also moves into a different conversation's state while leaving that conversation's own heads bitwise untouched.

== The model drives it

*Reproduction.* We ran the author's code and checkpoint on an RTX 3090. In one conversation the model issued all nine store calls itself: three puts, a list, three gets, a delete and a list. Every answer was printed by the model, and 160 printed tokens matched the author's reference with no mismatches. We saved the store the author's way, as a 14 KB file of its occupied rows, and loaded it into an empty conversation. The model listed the two surviving files and read both back exactly. KL to the base model was $6.9 times 10^(-4)$ against a noise floor of $6.7 times 10^(-4)$, and bitwise identical with files loaded or not.

*tang runs the compiled layers.* tang's reference math ran the checkpoint's 12 compiled layers on the author's trace. All 126 tokens of a shorter put, get and list trace matched, and every store head's $128 times 128$ state was bitwise equal after each operation.

*tang compiles them.* `tang-chip`, a Rust port of the author's compiler, emits the layers from the circuit graph: 3,753 dimensions, 164 heads and 47,532 units. The dimensions are live at different times, so the compiler packs them into far fewer slots of the residual stream. At Qwen3.5 4B shapes, all 168 tensors equal the checkpoint bitwise, zero padding included: 2.44 billion elements, 1.24 million of them nonzero. Retargeted to Coder-Next, whose hidden size widens from 2,048 to 3,968, the store replays the trace bitwise and no weight touches the host model's dimensions.

*tang serves them.* A chip window in the engine uploads the emitted layers as sparse blocks and runs store calls without touching the base layers, the LM head or the KV cache. On an RTX 3090 it ran with 0 mismatching layer steps against the CPU path. Inside a running engine, on a slot holding a prompt, store calls returned the stored ids. The slot's tokens and greedy continuation stayed identical to a twin slot that never ran them.

#figure(fig-where(),
  caption: [#lead[Where the store lives.] The compiled layers sit beside the host model, not inside it. The host model emits the tool call; the engine runs it as chip tokens through the chip layers only, and the host sees the result as text.],
)

#figure(
  table(
    columns: 4,
    align: (left, right, right, right),
    table.header([*Operation*], [*Tokens*], [*Per op*], [*Per token*]),
    [Put], [56], [51 ms], [0.92 ms],
    [Get], [43], [40 ms], [0.92 ms],
    [List], [27], [25 ms], [0.93 ms],
  ),
  caption: [#lead[The chip window on an RTX 3090], with CUDA graphs. The author's PyTorch stack took 1.0–1.65 s per operation.],
)

#figure(fig-latency(),
  caption: [#lead[Latency per store operation], same RTX 3090, log scale. The author's time is mostly PyTorch fallback kernels, not the store.],
)

The compiled layers take 13.75 MB of sparse weights and 26 MB of state per session in tang, and about 332 million multiply-adds per token, roughly 8 percent of the 4B model's. That count understates the cost of the published artifact (§6.4).

== Serving

`tang serve` loads a store per session, runs operations and exports it. An ephemeral mode zeroes the store when a session ends and never writes it to disk. frog loads the user's file at session start, saves after every write, and gives the model a `Memory` tool. `frog memory dump` prints exactly what is stored.

#aside[Privacy][
  The part of memory that persists becomes a file on the user's disk, 512 to 768 KiB. In a hosted session the provider sees it in plaintext while the session runs, and with the ephemeral mode keeps nothing afterwards. This is not encryption: the layout is known, so anyone holding the file can read it. The model can still repeat what it reads, so egress controls matter as before. What changes is where memory piles up: in a file you can read and delete, not with us.
]

= Does it help an agent remember?

Sections 2 to 5 show that the storage is exact. This section asks the question that matters for an agent: given the store, does a model remember better across sessions? We ran the author's filesystem model against four memory arms on the same base weights, Qwen3.5 4B, on one RTX 3090 with greedy decoding. The data, prompts, model revisions and scripts are published with this post.

== Setup

- *A0, no memory.* The question alone.
- *A1, full history.* The most recent 16,384 tokens of the earlier sessions.
- *A2, retrieval.* The earlier sessions in chunks of six turns, embedded with bge-small, top five chunks in the prompt.
- *A3, text notes.* After each session the model writes notes of up to 400 tokens; all notes go into the prompt. This is how coding agents remember today.
- *A4, GPSM.* After each session the filesystem model is shown the session and its store tools, with the author's system prompt, and saves what it chooses, up to 1,000 of its own tokens per session. At question time it sees only the question and may list and get files, with 300 of its own tokens to spend.

Two tests. *LoCoMo* is a standard long-conversation benchmark; we used 3 of its 10 conversations (80 sessions), excluded its adversarial category, and scored 125 questions with its official F1. The *exact-facts probe* plants 60 seeded values (UUIDs, paths, ports, configuration values, function signatures and hashes) in five short sessions among filler, then asks for each one in a new session, scored by exact match. Intervals are bootstrap 95 percent.

== Results

#figure(
  table(
    columns: (1.5fr, 1.15fr, 0.7fr, 1fr, 0.9fr),
    align: (left, right, right, right, right),
    table.header([*Arm*], [*LoCoMo F1*], [*EM*], [*Prompt tokens*], [*s / question*]),
    [A0 no memory], [0.10 (0.06–0.13)], [0.01], [52], [0.3],
    [A1 full history], [0.46 (0.40–0.53)], [0.18], [16,711], [4.1],
    [A2 retrieval], [0.44 (0.37–0.51)], [0.18], [1,127], [0.7],
    [A3 text notes], [0.40 (0.34–0.46)], [0.14], [6,477], [1.7],
    [A4 GPSM], [0.05 (0.02–0.08)], [0.00], [854 (3,565 with tool output)], [76.7],
  ),
  caption: [#lead[Cross-session QA on LoCoMo], 125 questions from 3 conversations, Qwen3.5 4B, RTX 3090. Retrieval matches full history at a fifteenth of the prompt. GPSM scores below no memory.],
)

#figure(
  table(
    columns: (1.5fr, 1.2fr, 1fr, 1fr),
    align: (left, right, right, right),
    table.header([*Arm*], [*Exact match*], [*Char. error rate*], [*Prompt tokens*]),
    [A0 no memory], [0.00], [1.29], [33],
    [A1 full history], [0.97 (0.92–1.00)], [0.01], [3,063],
    [A2 retrieval], [0.92 (0.83–0.98)], [0.06], [591],
    [A3 text notes], [0.88 (0.80–0.95)], [0.16], [1,743],
    [A4 GPSM], [0.23 (0.13–0.35)], [2.29], [835],
  ),
  caption: [#lead[Exact-facts probe], 60 seeded values asked in a later session. GPSM's reply contained the right value in 67 percent of cases but matched exactly in 23.],
)

GPSM came last on both tests. Retrieval over the old transcripts, the simplest arm, came within the confidence interval of full history at about a fifteenth of the prompt.

== Where it fails

The logs separate the store from the model using it.

- *The store returns what was written.* On the facts probe, when the right file existed and the model looked for it, it fetched the right file 95 percent of the time and the exact value came back.
- *The model always writes and always looks.* It saved files in all 85 sessions, about 18 per LoCoMo session, and listed or fetched on every question, whether or not a relevant file existed.
- *What it writes loses information.* Notes were saved without session dates, so "when" questions returned "last year" instead of a year. Facts were filed under names unrelated to their content: one speaker's home country was stored inside a file about a necklace.
- *It runs out of room.* Every write consumes one of 512 chunks for good, even an overwrite. Two of the three LoCoMo conversations exhausted the store, which answered "full" 83 and 67 times. In the facts probe the model deleted all 12 of its own files during the first session, losing 30 percent of the facts before any question was asked.
- *Lookups loop.* With 300 to 500 file names, the model guessed names, fetched near misses and kept going. 101 of 125 LoCoMo questions ended mid-lookup with no answer. On the 24 it did answer, F1 was 0.25.
- *It reports success when it fails.* When the store answered "full", the model told the user the text was saved, both times we tested it.

== Cost and limits of the artifact

#figure(
  table(
    columns: (1.6fr, 1fr, 1.25fr),
    align: (left, right, right),
    table.header([*Measure (RTX 3090)*], [*Base Qwen3.5 4B*], [*With compiled store*]),
    [Decode, tokens/s (n = 3)], [45.4 ± 0.3], [31.0 ± 0.2],
    [Prefill, tokens/s (n = 3)], [4,303 ± 9], [992 ± 37],
    [Peak GPU memory], [9.5 GB], [18.9 GB],
    [KL to base, Rust / Python / prose], [–], [5.1 / 7.3 / 6.7 × 10#super[−4]],
    [KL noise floor, Rust / Python / prose], [4.1 / 6.1 / 7.2 × 10#super[−4]], [–],
  ),
  caption: [#lead[The published filesystem model against its base.] The compiled model runs through the author's driver because it does not fit a plain load on 24 GB, so the speed rows compare two code paths as well as two models. KL is within the noise floor on code and prose; the prose here is a different 512-token text from the reproduction run in §5.2.],
)

The store's limits are fixed by its compilation: 512 writes over its lifetime, 53,248 tokens, and 104 tokens per write, beyond which a write is cut without an error. Its saved file grows from 9,746 bytes empty to 604,338 bytes with 512 small files and 2,163,655 bytes with one maximum-size file. tang's own store, which runs on the same primitive, reuses deleted slots; the author's does not.

== Exact computation

The same compiler emitted a RISC-V CPU into the model. On the author's arithmetic benchmark, 40 problems each, the CPU model solved 39 large multiplications and 40 large divisions; base Qwen3.5 4B solved 22 and 29. Here the model only has to hand a calculation to the circuit, and the exactness carries through.

= Implications

== Exact storage is solved; the policy is not

Everything below the tool call worked: rows were written and read bitwise, the store persisted across conversations, and the right file came back when it was asked for. Everything above it failed: what to write, under which name, when the store is full, and when to stop looking. Those are decisions a model makes in text, and an untrained 4B model made them badly. Better results will come from the policy, not the memory. Three directions follow from the failure modes, and none is tested: let the engine write dated, structured entries instead of the model, replace list-and-guess with a search operation, and train the model to use the tools.

== Memory leaves the prompt, lookups do not

Stored files cost no prompt tokens while they sit in the state. Using them does: in the GPSM arm the context grew from 854 tokens to 3,565 on average with list output and file contents, against 1,127 for retrieval. A store only saves context if lookups are short and targeted.

== Compiled, not trained

The compiled layers left the model's output within the noise floor on Rust, Python and prose, and a compiled CPU fixed arithmetic the base model gets wrong. Exact capabilities can be added without training and without disturbing the base model. The cost in the published artifact is real, 0.68 times the decode speed and twice the memory, though some of that is the driver rather than the layers.

== The engine owns the invariant

Exactness holds only while every gate in a pinned head stays bitwise at 1. So GPSM is a contract between compiled weights and an engine that routes store traffic separately and audits the store before export. tang can keep store tokens out of the base layers and the KV cache because it decides what runs where.

== What it does not change

GPSM does not make new context cheaper. Prefill still runs every token through the attention and MoE layers, and both of our attempts to shortcut it failed (§4).

= Comparison

#figure(
  table(
    columns: (1.4fr, 1fr, 1fr, 1fr, 1.1fr),
    align: (left, right, right, right, left),
    table.header([*Approach*], [*LoCoMo F1*], [*Facts EM*], [*Prompt tokens*], [*Needs*]),
    [Full history], [0.46], [0.97], [16,711], [Context to fit it],
    [Retrieval], [0.44], [0.92], [1,127], [An embedding model],
    [Text notes], [0.40], [0.88], [6,477], [Nothing extra],
    [GPSM, as published], [0.05], [0.23], [854–3,565], [Compiled layers, 2× memory],
  ),
  caption: [#lead[Measured, same base model and questions.] Prompt tokens are LoCoMo means.],
)

On our measurements, retrieval dominates: nearly the accuracy of full history at a fifteenth of the prompt, with no change to the model. Text notes come close behind. GPSM is last on accuracy, slowest by an order of magnitude per question, and the most expensive to run.

We did not run learned-state approaches such as Titans @behrouz2025titans, test-time training @inplacettt2026 @lcc2026, or parametric memory; their published results come from different models and benchmarks and are not comparable to the numbers above. One study of one-step LoRA memory measured free-form recall at zero @beyondppl2026. Agent memory systems in the MemGPT, Mem0 and A-MEM line are elaborations of the text-notes arm @packer2023memgpt @chhikara2025mem0 @xu2025amem.

GPSM's distinct property is exact storage that costs nothing to hold. Nothing in these results shows that property paying off in recall yet.

= What the data supports

- *Exact storage and transfer.* Supported. Rows round-trip bitwise, the store persists in a small file, and moves between conversations without disturbing the receiving one (§5).
- *Exact computation.* Supported on one benchmark: 39 of 40 against 22 of 40 for large multiplication (§6.5).
- *Leaving the base model unchanged.* Supported on code and prose: KL within the noise floor (§6.4).
- *Cross-session memory for an agent.* Not supported. GPSM scored below no memory on LoCoMo and recalled 23 percent of exact facts, against 92 percent for retrieval (§6.2).
- *Small models on small machines.* Not supported today. The published artifact doubles GPU memory and lowers decode speed, and its lookups added more context than retrieval did.

= Open questions

- *Can a better policy close the gap?* Engine-written entries, a search operation, or training the model on the tools are untested.
- *Does it hold at scale and on Coder-Next?* We ran one 4B model, one seed and 3 of 10 LoCoMo conversations. No build on Coder-Next's weights exists yet.
- *How much of the overhead is the driver?* The speed comparison mixes two code paths; tang's chip window ran at about 0.9 ms per token, but not yet beside a host model at full speed.
- *Is a lifetime write budget acceptable?* The author's store never frees space; tang's reuses slots, but that has not been run in this benchmark.

= Conclusion

A delta-rule head with its gates pinned is exact RAM. We confirmed that in tang's kernel, showed that no trained head reaches it on its own, and followed the idea from the author's checkpoint to our own compiler and a CUDA window inside our engine, each step bitwise. Then we measured what it buys an agent, and the answer today is little: the model driving the store remembered less than a model with no memory at all, while simple retrieval came close to full history. The storage is exact; the policy that uses it is not. Gate-Pinned State Memory is a sound primitive and a poor memory system as published, and the work that remains is teaching a model to use it.

#heading(outlined: true, numbering: none)[References]
#show bibliography: it => { show "?.": "?"; show "!.": "!"; it }
#bibliography("refs.bib", style: "nature", title: none)
