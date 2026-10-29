// Research post: Zero-context compiled memory (tang, frog)
// Same treatment as paper/vision/vision.typ (muon "Pilot Light", dark), trimmed to what this post uses.
// One source, two outputs: `--features html --format html --input target=html` builds the web
// post; without the input it builds the PDF. See web/scripts/build-posts.mjs.

#let web = sys.inputs.at("target", default: "pdf") == "html"
#show math.equation: it => if web { html.elem("span", attrs: (class: "math"), html.frame(it)) } else { it }
#show math.equation: set text(font: "New Computer Modern Math", fallback: true)
#show math.equation: it => { show regex("[A-Za-z0-9]"): set text(font: "IBM Plex Sans"); it }

// ---- colour roles (as vision.typ) -------------------------------------------
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

// product names in their family colour, as in the vision paper's periodic table
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

#set document(title: "Zero-context compiled memory", author: "Cam Pedersen (Municipal Robotics)")

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
      pad(x: -hf-out(), grid(columns: (1fr, auto), caps[Zero-context compiled memory], caps[Municipal Robotics]))
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
    lbl((X(d), -0.3), if d == 0 [1] else [10#super[−#d]])
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
  lbl((ex + 2.35, H - 0.62), [chip tokens], anchor: "west", fill: c-sel)
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

#let fig-proof() = {
  let rungs = (
    ([Kernel one-hot write], [bitwise]),
    ([50,000-step free run], [bitwise on CPU and 3090]),
    ([Author's checkpoint, 3090], [160 tokens, 0 mismatches; recall from a 14 KB file]),
    ([tang reference math], [126 tokens, states bitwise]),
    ([tang compiler], [168 tensors bitwise]),
    ([GPU chip window], [0 mismatching layer steps]),
    ([In-engine isolation], [twin slot identical]),
  )
  let card(i, r) = block(width: 100%, height: 0.92in, inset: 5pt, fill: c-surface, stroke: 0.6pt + c-line-hi, {
    set par(leading: 0.45em)
    text(font: mono-m, size: s-xs, fill: c-label)[#(i + 1)]
    h(1fr)
    text(font: mono-sb, size: s-sm, fill: c-ok)[✓]
    v(0.3em)
    text(font: sans-m, size: s-sm, fill: c-text, r.at(0))
    linebreak()
    text(font: sans, size: s-xs, fill: c-label, r.at(1))
  })
  grid(columns: (1fr,) * 4, column-gutter: 5pt, row-gutter: 5pt,
    ..rungs.enumerate().map(((i, r)) => card(i, r)))
}

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
  #move(dx: -0.045 * s-display, text(font: sans-sb, size: s-display, tracking: -0.02em)[Zero-context\ compiled memory])
  #v(0.5em)
  #line(length: 22%, stroke: 2pt + c-label)
  #v(0.7em)
  #text(size: s-lg, fill: c-label)[Exact, persistent memory in a model's recurrent state, served by tang]
  #v(1fr)
  #caps[Cam Pedersen · Municipal Robotics]
]}

#let abstract-text = [
  A coding agent remembers by pasting text into its prompt, and pays for it on every turn. Divyansh Agrawal's qwenputer showed another way: a Gated DeltaNet layer with its gates pinned is exact, addressable RAM, and a filesystem can be compiled into extra layers of a model without training @agrawal2026qwenputer. We checked every link of that claim on our own stack.

  tang's kernel stores and reads one-hot rows bitwise, and stays exact over a 50,000-step free run. Trained heads cannot do it: on GLM-5.3-Flash, none of 8,192 channels per layer keeps its gate at exactly 1. We reproduced the author's filesystem on an RTX 3090, with the model issuing every store call itself and recalling files from a 14 KB state file in an empty conversation. tang's reference math then ran the author's compiled layers bitwise, tang's own Rust compiler emitted all 168 tensors of them bitwise, and a CUDA window in our engine ran them bitwise at about 0.9 ms per token, 25 to 40 times faster than the author's stack.

  We also tested two ways to get instant prefill from the same structure, and both fail. What remains is memory that costs no context, needs no training, leaves the model's text unchanged, and can be held by the user rather than the provider.
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
  grid(columns: 2, column-gutter: 0.5in, delta-grid(pinned: false, title: [trained head: g < 1, dense k]), delta-grid(pinned: true, title: [pinned head: g = 1, β = 1, k = e#sub[i]])),
  caption: [#lead[The delta rule, drawn.] Sixteen rows stand for a 128 × 128 head state. A trained head decays every row and smears the write across all of them. A pinned head with a one-hot key replaces exactly one row and leaves the rest untouched.],
)

A one-hot query reads row $i$ back. Compile weights that produce those gates and keys, and the recurrent state is RAM. qwenputer compiled a RISC-V CPU and a 512-file store into 12 extra layers in front of Qwen 3.5 4B, without training, and reported a KL divergence to the base model of $6 times 10^(-4)$ @agrawal2026qwenputer.

tang is our inference engine. It runs Qwen3-Coder-Next, a hybrid of 36 GDN layers and 12 attention layers, and already snapshots the fixed-size recurrent state to disk. frog, our coding agent, talks to it. So the question was concrete: what of this transfers to an engine we own?

= Exact RAM in tang's kernel

tang pins the GDN recurrence in a CPU specification that the CUDA kernel is verified against bitwise. We tested the claim there.

- *The gates reach exactly 1.* Projections of $a + "dt" = -30$ and $b = 40$ round to exactly `1.0f`.
- *A write into an empty row stores $v$ bitwise* and touches no other row of any of the 48 heads.
- *An overwrite stores $"fma"(1, v - s, s)$*, which is exactly $v$ only when $v - s$ is exact. On random floats, 8,643 overwrites were exact and 15,933 were off by one rounding. Stored values must be integers below $2^24$.
- *A 50,000-token free run*, driving the kernel as a register machine, stayed bitwise exact on the CPU and on an RTX 3090.
- *One token with $g < 1$ ends exactness for good.* It scales every row by a non-power-of-two, and the integers turn fractional.

= Trained heads cannot do it

So a pretrained head can be borrowed for storage only if its gate is bitwise 1 on every token. We measured the real gates of GLM-5.3-Flash, whose recurrent layers use a per-channel-gated delta rule, over 427 tokens and all 34 recurrent layers.

#figure(
  table(
    columns: 5,
    align: (left, right, right, right, right),
    table.header([*Layers*], [*Mean g*], [*Best channel min g*], [*β mean / max*], [*At 1.0 every token*]),
    [Early (0–10)], [0.51–0.68], [0.999994], [0.54–0.71 / ≤0.997], [0 of 8,192],
    [Mid (12–26)], [0.73–0.87], [0.999999], [0.64–0.77 / ≤1.000], [0 of 8,192],
    [Late (28–44)], [0.67–0.79], [0.999997], [0.55–0.84 / 1.000], [0 of 8,192],
  ),
  caption: [#lead[Gates of a trained hybrid model.] No channel qualifies as exact storage. Measured on GLM-5.3-Flash UD-Q4_K_XL, RTX 3090.],
)

A few channels are remarkably slow. Between 1 and 8 percent per layer have a mean gate above 0.999, and the best has a gate-only half-life near three million tokens. #figure(fig-gates(),
  caption: [#lead[No channel reaches 1.] Each bar is a range across the layers in its group: hollow for the median channel's minimum gate over the run, solid for the best channel's. GLM-5.3-Flash on an RTX 3090 over 427 tokens; per-layer data was not kept.],
)

But they are analog registers, and they leak. Exact storage needs dedicated heads with compiled gates, which is the qwenputer design.

= Two dead ends

The same structure suggests faster prefill. We tested both versions, and neither works.

*Poking state.* A trained head can only read the state its own prefill would have produced. An integer written into a row means nothing to it, so poking context into state turns back into prefill.

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

= The store, end to end

== An engine-driven store

We first built the store without compiled layers: a key-value file store in dedicated GDN heads of one layer, written and read through the real kernel with one-hot keys and pinned gates. The layout follows the author's: a hashed directory with linear probing, plus content rows of token ids. Reads pass the output norm and a division, so values stay below $2^18$.

#figure(
  table(
    columns: 4,
    align: (left, left, right, right),
    table.header([*Model shape*], [*Heads, k / v*], [*Directory*], [*Content*]),
    [Flash-Next], [16 / 48, tiled], [1,024 slots, 512 files], [65,024 tokens · 768 KiB],
    [Coder-Next, Qwen 3.5 4B], [16 / 32, grouped], [512 slots, 256 files], [65,024 tokens · 512 KiB],
  ),
  caption: [#lead[Store capacity] in a quarter of one GDN layer.],
)

A 512-file fill and a 10,000-operation random workload, checked against a hash map, both pass. The store persists through the bytes tang's snapshot already carries. It also moves into a different conversation's state while leaving that conversation's own heads bitwise untouched.

== The model drives it

*Reproduction.* We ran the author's code and checkpoint on an RTX 3090. In one conversation the model issued all nine store calls itself: three puts, a list, three gets, a delete and a list. Every answer was printed by the model, and 160 printed tokens matched the author's reference with no mismatches. We saved the state to a 14 KB file and loaded it into an empty conversation. The model listed the two surviving files and read both back exactly. KL to the base model was $6.9 times 10^(-4)$ against a noise floor of $6.7 times 10^(-4)$, and bitwise identical with files loaded or not.

*tang runs the compiled layers.* tang's reference math ran the checkpoint's 12 compiled layers on the author's trace. All 126 tokens matched, and every store head's $128 times 128$ state was bitwise equal after each operation.

*tang compiles them.* `tang-chip`, a Rust port of the author's compiler, emits the layers from the circuit graph: 3,753 dimensions, 164 heads and 47,532 units. At Qwen 3.5 4B shapes, all 168 tensors equal the checkpoint bitwise, zero padding included: 2.44 billion elements, 1.24 million of them nonzero. Retargeted to Coder-Next, whose hidden size widens from 2,048 to 3,968, the store replays the trace bitwise and no weight touches the host model's dimensions.

*tang serves them.* A chip window in the engine uploads the emitted layers as sparse blocks and runs store calls without touching the base layers, the LM head or the KV cache. On an RTX 3090 it ran with 0 mismatching layer steps against the CPU path. Inside a running engine, on a slot holding a prompt, store calls returned the stored ids. The slot's tokens and greedy continuation stayed identical to a twin slot that never ran them.

#figure(fig-where(),
  caption: [#lead[Where the store lives.] The compiled layers sit beside the host model, not inside it. Store calls run only through the chip layers; the host sees the result as text.],
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

The compiled layers take 13.75 MB of sparse weights and 26 MB of state per session. They cost about 332 million multiply-adds per token, roughly 8 percent of the 4B model.

Each link of the claim was checked on its own, from the kernel up to a running engine.

#figure(fig-proof(),
  caption: [#lead[The proof chain.] Seven checks, each bitwise or exact, from the kernel's one-hot write to a store call inside a running engine.],
)

== Serving

tang serve loads a store per session, runs operations and exports it. An ephemeral mode zeroes the store when a session ends and never writes it to disk. frog loads the user's file at session start, saves after every write, and gives the model a `Memory` tool. `frog memory dump` prints exactly what is stored.

#aside[Privacy][
  The part of memory that persists becomes a file on the user's disk, about half a megabyte. In a hosted session the provider sees it in plaintext while the session runs, and with the ephemeral mode keeps nothing afterwards. This is not encryption: the layout is known, so anyone holding the file can read it. The model can still repeat what it reads, so egress controls matter as before. What changes is where memory piles up: in a file you can read and delete, not with us.
]

= Where it sits

Compiled memory occupies a corner nothing else holds: exact recall, no context, no training, and almost no change to the base model. Text memory such as MemGPT, Mem0 and A-MEM is portable and mature, but pays context every turn and recalls lossily @packer2023memgpt @chhikara2025mem0 @xu2025amem. KV reuse is exact but large @yao2024cacheblend. Learned states and test-time training cost no context but are lossy and need training @behrouz2025titans @inplacettt2026 @lcc2026. Knowledge editing and one-step LoRA memory are dominated: one study measured free-form recall at zero @beyondppl2026. Compiled transformers go back to Tracr @lindner2023tracr. What is new is that hybrid models have layers where compiled storage is exact.

= What is not done

- *No cross-session benchmark.* The 90 percent recall figure is the author's own fact set. Nobody has run compiled memory on LongMemEval or LoCoMo.
- *Does the model reach for it?* The 4B model used the store when asked. Whether Coder-Next uses memory at the right moments in a long coding session is unmeasured, and it is the main risk.
- *No build on real Coder-Next weights yet.* It needs the circuit re-exported with Coder-Next's tokenizer and a machine that holds the weights.
- *Fixed capacity.* 256 to 512 files.
- *Compute on every token.* About 8 percent at 4B, less on a 3B-active mixture of experts.

#heading(outlined: true, numbering: none)[References]
#show bibliography: it => { show "?.": "?"; show "!.": "!"; it }
#bibliography("refs.bib", style: "nature", title: none)
