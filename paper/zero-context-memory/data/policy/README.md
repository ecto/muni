# GPSM policy arms: does a better write or lookup policy close the gap?

## Licensing and what is published

LoCoMo (Maharana et al., 2024, arXiv:2402.17753; github.com/snap-research/locomo) is licensed CC BY-NC 4.0.
This folder publishes only our measurements of it: per-question scores, token counts and timings keyed by
LoCoMo question id, plus aggregate results. It does not include LoCoMo's conversations, questions or answers,
or model transcripts of them; those raw files were removed before publication. To reproduce, download
`data/locomo10.json` from the LoCoMo repository and run the scripts here. The scorer in `scripts/` is our
reimplementation of LoCoMo's F1 logic, not a copy of their code. The exact-facts probe is our own data and is
published in full.


Run 2026-10-29 on `mew` (RTX 3090 24 GB), the same machine, software, models, revisions, data subsets, scorer, greedy
decoding and bootstrap as `../bench/README.md` (read that first). Our code is in `scripts/`; it imports `bench.py`
from the first run unchanged (data loading, prompts, the A4 driver `run`, `gpsm_setup`). Every GPU phase ran under
`flock ~/tang-gpu.lock`; summed locked time was 3.51 GPU-hours (12,640 s, `raw/drive.log`), about 0.95 h of which went
to the two abandoned naming revisions (below). That is over the 3 h target.

## The store and the driver
- Store: the `Qwen3.5-4B-Qwenputer-Filesystem` checkpoint's circuit (512 names, 512 chunks, <=104 tokens per put or
  append, names keyed on their first 6 tokens plus a length class). Between sessions the store persists through the
  author's `save_store`/`load_store`, exactly as A4: one stream per session, from the previous session's snapshot.
- Driver-issued calls (`policy.py: drive`): the prompt is prefilled through the chip layers with the snapshot loaded
  (as `bench.run`). The driver then feeds each call's tokens in Qwen3.5's spelling (the author's
  `StoreSession.call_tokens`). At every position the circuit owns, the circuit's token is taken, as in `bench.run`. So
  the circuit captures the call and prints its own answer (records, `ok`, a get's text, a list's names). Between
  calls the driver writes the newline the template leaves to Qwen. Qwen's own layers are not run in driver streams,
  because no position there is Qwen's choice; the store lives entirely in the chip layers.
  - "Conflicts" (the circuit claiming a position with a token other than the driver's) were 0 in every stream.
- Read-back (`readback`): a `list` call, split into names at the circuit's SL records, then a `get` per name in
  batches of 40 per stream. A name that list printed with its newline (<=5 tokens) is sent as printed; a longer one is
  sent as its 6 printed tokens plus a newline. Every listed name was found on the first variant: A4 stores
  258/400/356/60 names, B2 stores 75/133/107/36.

## Arms (questions: the same 125 LoCoMo and 60 facts as A4, unless cut below)
- B1, model writes + engine search: A4's saved final stores (`~/gpsm-bench/out/gpsm_store/*_s<last>.npz`, not re-run).
  Every file is read back from the store by driver get calls. Each file is embedded as `[<name>]\n<text>` with
  bge-small (the name is a model-written file's only label), and the top 5 by cosine go in A2's prompt. Base Qwen3.5-4B
  answers with no tools, by A2's code path. The checkpoint's Qwen layers are the base's; base was used so that B1/B3
  differ from A2 only in where the text comes from.
- B2, engine writes + model lookup: see "Engine writes". At question time the checkpoint gets A4's prompt, tools and
  `bench.run`, with Qwen budgets of 300 (`B2b300`) and 1000 (`B2b1000`) free tokens. Circuit tokens are uncapped, as in
  A4.
- B3, engine writes + engine search: B2's store, read back by driver gets. Each entry's text (it starts with its
  `[Session n, date]` line) is embedded as stored; top 5 go in A2's prompt; base Qwen3.5-4B answers.

### Prompts (verbatim)
- B1 and B3: A2's: "Relevant excerpts from earlier conversations:\n{5 texts joined by blank lines}\n\n{question}\n{INSTR}".
  Query prefix: "Represent this sentence for searching relevant passages: ". max_new_tokens 64.
- B2: A4's. System MEM_SYSTEM and tools CALC_TOOL + put/append/get/list/delete; user
  "{question}\n{INSTR}\nYou may look in your memory with list and get." INSTR is unchanged from the first run.
- Engine write streams are prefilled with A4's write prompt (`bench.write_prompt(..., "store")`) for parity. The
  model does not choose anything there.

## Engine writes (B2/B3)
- Chunking: A2's. Each entry holds 6 consecutive turns of one session as `[Session n, date]\n` + `Speaker: text` lines,
  with photo captions. The text is cut into pieces of whole lines of at most 103 tokens, plus the newline the circuit
  stores (104). The first piece is put; the rest are appended under the same name.
  - A line over 103 tokens is split at spaces (3 lines in conv-50; their split point becomes a newline).
  - The pieces concatenated equal A2's chunk text plus a final newline, so B3's prompts are byte-identical to A2's.
- Capacity: no reduction was needed. Writes per store were 256/349/343 (conv-26/47/50) and 50 (facts), all of 512.
  Stored tokens were 18,730/25,760/25,484 and 3,498, all of 53,248. Entries were 75/133/107 and 36. Every write
  answered `ok`.
- Names (final, rev3): two salient words of the chunk, <=5 tokens, unique per store, e.g. `search offset`, `class
  programming`.
  - The words are lowercase alphabetic, 4+ letters, not a stopword or speaker name, and a single Qwen token both with
    and without a leading space. They are ranked by tf x log(N/df) over that conversation's chunks.
  - The first unused pair among the top 10 is taken, then word pairs from triples. Failing that, `s<n>c<k>` is used
    (6 conv-47, 1 conv-50, 4 facts entries).
  - The session and date are not in the name; they are the first line of the text.

### Revisions (both abandoned and kept in `raw/rev1`, `raw/rev2`)
1. rev1 named entries `<kw> <kw> s<n>c<k> <date> <speakers>`, with the distinguishing part first, as asked.
   `list` prints only a name's first 6 tokens. Only names of 5 tokens or fewer end with their newline, so the listing
   ran together ("search offset s1c1last game s1c2billing ..."). The model then called get on fragments such as
   `s5c1billing`.
   - B2b300 facts (20 q): 0 EM, 20/20 budget exhausted.
2. rev2 used `<kw> <kw> s<n>` (<=5 tokens; the listing is readable). The model dropped the ` s<n>` part when it called
   get ("search offset" for "search offset s1").
   - B2b300 facts (16 q, stopped): 1 answered.
3. rev3 (final) uses two words only. The store contents (texts) are identical across revisions; B3 gave identical
   predictions in all three.
- A diagnostic `B2snakeb300` (facts, 15 q) uses rev3's words joined snake_case (`search_offset`), the checkpoint's own
  naming style from A4.

## Results (95 % bootstrap CIs, 2000 resamples, seed 0; full numbers in results.json)
| arm | LoCoMo n | F1 | EM | facts n | EM | contains | CER | prompt tok (LoCoMo) | context tok | s/q (LoCoMo) |
|---|---|---|---|---|---|---|---|---|---|---|
| A2 RAG (first run) | 125 | 0.44 [0.37, 0.51] | 0.18 [0.11, 0.25] | 60 | 0.92 [0.83, 0.98] | 0.92 | 0.06 | 1127 | - | 0.7 |
| A4 GPSM (first run) | 125 | 0.05 [0.02, 0.08] | 0.00 | 60 | 0.23 [0.13, 0.35] | 0.67 | 2.29 | 854 | 3565 | 77 |
| B1 model writes + engine search | 125 | 0.27 [0.21, 0.33] | 0.11 [0.06, 0.18] | 60 | 0.68 [0.57, 0.80] | 0.68 | 0.35 | 175 | - | 0.3 |
| B2 engine writes + model lookup, 300 | 42 | 0.03 [0.00, 0.07] | 0.00 | 30 | 0.07 [0.00, 0.17] | 0.27 | 1.61 | 854 | 2902 | 63 |
| B2, 1000 | 9 | 0.00 [0, 0] | 0.00 | 15 | 0.00 [0, 0] | 0.20 | 1.02 | 854 | 7465 | 208 |
| B3 engine writes + engine search | 125 | 0.44 [0.37, 0.51] | 0.18 [0.11, 0.25] | 60 | 0.92 [0.83, 0.98] | 0.92 | 0.06 | 1127 | - | 0.6 |
| B2snake, 300 (diagnostic) | - | - | - | 15 | 0.00 | 0.13 | 1.03 | - | - | - |

- B1 and B3 seconds/question exclude the one-off read-back: 52-98 s per store, 12-17 s for facts.
- Per-category F1 is in results.json. B1: single-hop 0.34, multi-hop 0.23, temporal 0.05, open 0.27. B3 equals A2 per
  category.

### Exactness (B3)
- Every one of the 351 engine entries read back from the store byte- and token-id-identical to what was written:
  0 mismatches (conv-26 75, conv-47 133, conv-50 107, facts 36).
- All 1,074 A4 files were also found by name, and all 351 B2 names were listed.

### Paired (per-question difference, bootstrap CI)
- B3 - A2: +0.000 [0, 0] F1, with identical predictions on 125/125 LoCoMo and 60/60 facts.
- B1 - A2: -0.17 [-0.25, -0.10] F1 and -0.23 [-0.35, -0.12] facts EM.
- B1 - A4: +0.22 [+0.16, +0.28] F1 and +0.45 [+0.32, +0.57] facts EM.
- B2b300 - A4: -0.01 [-0.07, +0.05] F1 (42 q) and -0.07 [-0.20, +0.07] facts EM (30 q).
- B2b1000 - B2b300 on the same questions: 0 (9 LoCoMo, 15 facts; identical predictions).

### Where B2 failed (from the transcripts)
- Every B2 question starts with `list`.
- On facts, all 255 gets named real files: 0 not found. 37 % of questions got an answer-bearing file, yet 67 % ran out
  of the budget, re-getting files (21/30 questions repeated a get).
- On LoCoMo, 44 % of gets were `not found` (A4: 19 %). The model split two-word names into single words (`stories`,
  `group`) and invented `s30c4`-style names.
- With 1000 tokens the model made 30-40 gets per question and still did not stop (73 % / 100 % exhausted).
- Snake_case names did not help: 0/15 EM, 80 % exhausted.

## Cuts (GPU time)
- B2b300 LoCoMo: every 3rd of the 125 in qid order, 42 questions (`STRIDE=3`).
- B2b1000 LoCoMo: every 15th, 9 questions, a subset of the 42.
- B2b300 facts: f0-f29, 30 questions (the second half was dropped).
- B2b1000 facts: f0-f14.
- B2snake: facts f0-f14 only.
- B1 and B3 ran in full. One seed, greedy decoding throughout.

## Files
- `results.json`: all metrics. The `paired`, `exactness` and `store` sections hold the paired differences, the B3
  exactness check and the A4 store read-back.
- `{locomo,facts}_{B1,B2b300,B2b1000,B3}.csv` and `facts_B2snakeb300.csv`: per question.
- `raw/`:
  - question jsonl, with full B2 transcripts and B1/B3 retrieved texts;
  - `*_B1_files.json` / `*_B3_files.json`: everything read back from the stores, as token ids and text;
  - `*_B2_write.json`: the engine's entries, names, pieces and per-session write answers;
  - `drive.log` (per-phase GPU seconds), `run.log`, and `rev1/`, `rev2/`.
- The engine-written stores (`.npz` per session) are on mew in `~/gpsm-bench/policy/store{,_snake,_rev1,_rev2}/`.
- `scripts/`:
  - `policy.py` (with `readback-a4 | write-b2 | readback-b2 | answer B1/B3 | ask-b2 DS BUDGET [N]`, env `STRIDE`,
    `NAMES=snake`);
  - `analyze_policy.py` (the scorer, copied from the first run's `analyze.py`);
  - `drive.sh`.
