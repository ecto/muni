# GPSM memory benchmark: LoCoMo cross-session QA and an exact-facts probe

## Licensing and what is published

LoCoMo (Maharana et al., 2024, arXiv:2402.17753; github.com/snap-research/locomo) is licensed CC BY-NC 4.0.
This folder publishes only our measurements of it: per-question scores, token counts and timings keyed by
LoCoMo question id, plus aggregate results. It does not include LoCoMo's conversations, questions or answers,
or model transcripts of them; those raw files were removed before publication. To reproduce, download
`data/locomo10.json` from the LoCoMo repository and run the scripts here. The scorer in `scripts/` is our
reimplementation of LoCoMo's F1 logic, not a copy of their code. The exact-facts probe is our own data and is
published in full.


Run 2026-10-29 on `mew` (one RTX 3090 24 GB, driver 580.196.07, torch 2.10.0+cu128, transformers 5.20.2,
Python 3.12). The GPU was shared, so every phase ran under `flock ~/tang-gpu.lock`. Summed compute time
was about 4.4 GPU-hours: 3.6 h of it was the GPSM arm, plus model loads. That is over the 3 h target;
see "Cuts". Wall time was about 10 h (03:07 to 13:05), mostly spent waiting in the lock queue.

## Models (all greedy, bf16, `enable_thinking=False` chat template)
- A0 to A3: `Qwen/Qwen3.5-4B` at rev `851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a`, loaded as `Qwen3_5ForCausalLM`
  (text weights only, 0 missing keys).
- A4: `divagr1925/Qwen3.5-4B-Qwenputer-Filesystem` at rev `9112e41f03516ca4d0125f5bfa975008487a8458`. Its card says
  `base_model: Qwen/Qwen3.5-4B, relation: merge`: 12 chip layers in front of Qwen's layers, store of 512 names and
  512 chunks, at most 104 text tokens per put, names compared on their first 6 tokens. It is driven by our own copy of
  the author's fast path (`chipformer.rv.chat.RvChat` from github divagr18/qwenputer, branch rv-store, commit
  `01cf657d353bdc0c795c2d57a28a3e3f483f20a1`). That driver uses chunked prefill and lets Qwen pick each token,
  except at positions the circuit owns. It is the same loop as `~/qwenputer-repro/scripts/repro.py:forced`.
  Between sessions the store persists through the author's `save_store`/`load_store` (recurrent-state rows only).
- A2 embedder: `BAAI/bge-small-en-v1.5` at rev `5c38ec7c405ec4b44b94cc5a9bb96e735b38267a`. It uses CLS pooling with L2
  normalisation. Queries get the prefix "Represent this sentence for searching relevant passages: ".

## Arms
- A0: question only.
- A1: full history, budget 16,384 tokens of transcript. The most recent turns are kept: sessions are walked backward
  and turns added until the budget is full. Mean prompt is 16.7k tokens.
- A2: RAG. Chunks are 6 consecutive turns of one session with a `[Session n, date]` header and no overlap. Retrieval
  is top-k with k=5 by cosine.
- A3: text notes. After each session the base model writes notes (max 400 new tokens). The question prompt gets all
  accumulated notes.
- A4: GPSM. After each session the checkpoint is shown the session and its own tools (author's `MEM_SYSTEM` system
  prompt plus `CALC_TOOL` and the store tools put/append/get/list/delete). It then decides what to put. The cap is
  1500 Qwen-chosen tokens per session on the facts probe and 1000 on LoCoMo; circuit-owned tokens don't count. At
  question time the context holds only the system prompt, the tools and the question. Qwen may produce 300 tokens of
  its own; circuit tokens such as list output and file contents are uncapped.

Session transcripts are formatted as `[Session n, date]` followed by `Speaker: text`, plus ` [shares a photo: <blip_caption>]`
where present.

### Prompts (verbatim; `{}` are fields)
Notes (A3) and store (A4) after each session share this head: "Below is a conversation between {who} (session {n},
{date}). After this you will not see it again: later you will be asked questions about it and will have only your "
- A3 continues: "notes. Write notes on everything you might need later: short bullet points that keep names, dates and exact values. At most 200 words.\n\n{transcript}"
- A4 continues: "memory. Save everything you might need later with put: one short, distinct name per fact (names are told apart by their first few words) and a short text that keeps names, dates and exact values. When you are done, reply done.\n\n{transcript}"

`{who}` is "Caroline and Melanie" (and so on) for LoCoMo, and "the user and the assistant" for the facts probe.

Question instruction INSTR:
- LoCoMo: "Answer the question with a short phrase, using exact words from the conversation whenever possible. If the question asks when, give the date."
- facts: "Reply with the exact value only."

Question prompts (single user turn, no system prompt for A0 to A3):
- A0: "{question}\n{INSTR}"
- A1: "Conversation history (the most recent part):\n{history}\n\n{question}\n{INSTR}"
- A2: "Relevant excerpts from earlier conversations:\n{5 chunks joined by blank lines}\n\n{question}\n{INSTR}"
- A3: "Your notes from earlier conversations:\n{[Notes from session n, date]\n... for every session}\n\n{question}\n{INSTR}"
- A4 (system = MEM_SYSTEM): "{question}\n{INSTR}\nYou may look in your memory with list and get."

MEM_SYSTEM (author's, chipformer/rv/store.py): "You have a memory that keeps texts across conversations, with the
tools put, append, get, list and delete, and a calc tool for exact arithmetic. When the user tells you something about
themselves (facts, preferences, changes), save it in your memory with put, one short name per fact; when a fact
changes, put the new value under the same name. Before you answer a question about the user, look in your memory
first (list, then get)."

## Data and subsets
- LoCoMo: `data/locomo10.json` from github snap-research/locomo (main).
  - Conversations: 3 drawn with `random.Random(0).sample(range(10), 3)`: conv-26, conv-47 and conv-50, with 19, 31
    and 30 sessions. All 80 sessions were written to memory for A3 and A4.
  - Questions: category 5 (adversarial) is excluded. 150 questions were drawn with `random.Random(1)` and sorted by
    qid. That sample has 84 single-hop, 32 temporal, 28 multi-hop and 6 open-domain questions.
  - Headline table: A4 ran only the first 125 of those 150 in qid order (all of conv-26 and conv-47, and 26 of
    conv-50's 51). Every arm in `results.json["locomo"]` is scored on those same 125. A0 to A3 on all 150 are in
    `results.json["locomo_all150_base_arms"]`.
- Exact-facts probe: `facts.json`, made by `scripts/make_facts.py` with seed 20261028. It has 60 facts, 10 each of
  UUID, path, port, key=value, Rust signature and 10-hex hash. They are spread over 5 sessions of 12, mixed with
  filler chit-chat. The user states each fact as "Quick note for later: ...". Questions come in one new session.

## Metrics
- LoCoMo: the scorer logic is copied verbatim from snap-research/locomo `task_eval/evaluation.py`. It gives token F1
  with Porter stemming; category 1 uses the comma-split partial F1 and category 3 uses the answer before ';'. Its EM
  is the set-equality `exact_match_score`. Category names follow the common mapping 1 multi-hop, 2 temporal,
  3 open-domain, 4 single-hop.
- Facts:
  - `em` is strict: the first line of the reply, stripped of quotes, backticks and a trailing '.', equals the answer.
  - `em_contains`: the answer string appears in the reply.
  - `cer`: Levenshtein distance divided by answer length.
- A4 replies that end inside an unfinished tool call (the 300-token budget ran out) are scored as an empty answer.
- CIs: 95 % percentile bootstrap, 2000 resamples, seed 0. Each entry is [mean, lo, hi].
- A4 reach stats (`results.json[ds]["A4_reach"]`):
  - The store contents are rebuilt from the model's own put, append and delete calls; only those answered "ok" are
    applied.
  - "Relevant file exists" on LoCoMo means some stored text holds at least 50 % of the gold answer's stemmed tokens
    (a lenient rule). On the facts probe it means some stored text contains the answer exactly.
  - `final_list_lines` comes from a forced `list` at the end. Names longer than 6 tokens print without a newline,
    so it undercounts.

## Cuts (and why)
- LoCoMo was cut to 3 of 10 conversations and 150 questions, and A4 to 125 of those, to save GPU time. A4 took about
  77 s per question. Most of that is the circuit printing a `list` of 300 to 500 names token by token, plus Qwen's
  get loops.
- No arm was cut.
- The A4 question budget of 300 Qwen tokens was not varied. 101/125 LoCoMo and 15/60 facts questions ran out of it
  inside get loops, so A4 scores are partly a budget effect. Among the 24 LoCoMo questions where A4 did answer, mean
  F1 was 0.25. Among the 45 facts questions it answered, 89 % contained the exact value.
- Only one seed and greedy decoding were used, so CIs reflect question sampling only.

## Files
- `results.json`: every number.
- `{locomo,facts}_{A0..A4}.csv`: per question, with prediction, score, prompt tokens and seconds. The A4 files also
  have context tokens and get/list counts.
- `facts.json`: the facts probe.
- `raw/`: the unscored jsonl, including full A4 write and question transcripts, A3 notes, and the final store
  listings.
- `scripts/`: `bench.py`, `analyze.py`, `make_facts.py` and `drive.sh`, as run.
