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

// ---- title page ---------------------------------------------------------------
#if not web {page(margin: (x: 0.75in, top: 0.7in, bottom: 0.65in))[
  #set par(leading: 0.5em)
  #grid(columns: (auto, 1fr, auto), column-gutter: 0.8em, align: horizon,
    caps(fill: c-text)[Municipal Robotics · Research],
    line(length: 100%, stroke: 0.5pt + c-line-hi),
    caps[Draft · October 2026])
  #v(1.6em)
  #image("images/muni-logo.svg", width: 0.95in)
  #v(2.4in)
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

With the decay $g = 1$, the write strength $beta = 1$ and a one-hot key $k = e_i$, the update replaces row $i$ of $S$ with $v$ and leaves every other row alone. A one-hot query reads row $i$ back. Compile weights that produce those gates and keys, and the recurrent state is RAM. qwenputer compiled a RISC-V CPU and a 512-file store into 12 extra layers in front of Qwen 3.5 4B, without training, and reported a KL divergence to the base model of $6 times 10^(-4)$ @agrawal2026qwenputer.

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

A few channels are remarkably slow. Between 1 and 8 percent per layer have a mean gate above 0.999, and the best has a gate-only half-life near three million tokens. But they are analog registers, and they leak. Exact storage needs dedicated heads with compiled gates, which is the qwenputer design.

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

The compiled layers take 13.75 MB of sparse weights and 26 MB of state per session. They cost about 332 million multiply-adds per token, roughly 8 percent of the 4B model.

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
