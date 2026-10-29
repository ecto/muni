# GPSM measurements: qwenputer Filesystem and CPU checkpoints

Run on mew: one RTX 3090 24 GB, torch 2.10.0+cu128, transformers 5.20.2, bf16 weights. flash-linear-attention and causal_conv1d are not installed, so the HF reference GDN kernels were used. Dates: 2026-10-29 UTC. Every number is in `results.json`. Raw outputs are in `raw/`, and the scripts are in `scripts/`. The scripts ran from ~/gpsm-measure on mew, invoked as `python3 -P`, with each GPU job under `flock ~/tang-gpu.lock`.

## Revisions
- Base: Qwen/Qwen3.5-4B @ 851bf6e8
- Filesystem: divagr1925/Qwen3.5-4B-Qwenputer-Filesystem @ 9112e41f
- CPU: divagr1925/Qwen3.5-4B-Qwenputer @ 2d906850
- Code: github.com/divagr18/qwenputer, branch rv-store @ 01cf657d (MIT). Its `chipformer` package was imported and not modified.

## 1. Sizes (bytes, exact)
Store file format: `save_store_file` writes a `np.savez_compressed` archive. It holds one `<head>/n` count per store head (49 heads, 93 value heads) and one float64[128] per nonzero state row. Conv states, firmware state and the KV cache are not included.
- The author's memory.npz, as produced by the earlier reproduction (a.txt and c.txt live, b.txt tombstoned): **14,276 B**. It has 20 rows and 233 nonzero values.
- Empty store: **9,746 B**. That is the zip overhead of 49 `/n` entries.
- After 3 puts (a/b/c.txt): **13,855 B**, from 18 rows.
- Full with 512 small files: **604,338 B**, from 2,565 rows (2,961,544 B uncompressed).
- Full with one 53,248-token file: **2,163,655 B**, from 8,710 rows.
- Dense store heads: 93 × 128 × 128 × fp32 = **6,094,848 B**.
- Recurrent state per conversation (fp32, independent of length):
  - Qwen's 24 GDN layers: 50,331,648 B
  - The 12 chip layers: 21,495,808 B
  - All 36 GDN layers: **71,827,456 B**
  - Conv states: 2,244,608 B (all 36 layers), 1,572,864 B (base)
  - KV cache: 32,768 B/token (8 full-attention layers, bf16)

## 2. Capacity
Method: `capacity.py` feeds teacher-forced `StoreSession.call_tokens` calls through the chip layers only (the HF modules as a CUDA graph). Positions the circuit owns take the circuit's token. Qwen's layers are skipped because every Qwen token was forced. The limits are the ones in `store.Config`: 1024 directory slots, 512 chunks, 512 names, CMAX = 104 tokens per call, NMAX = 6 name tokens compared.
- Small files ("fN", text " short note", 2 tokens): 512 puts return `ok`, and the 513th returns `full`.
- Large file (104-token appends to one file): 512 appends return `ok`, so **53,248 tokens** are stored. The 513th returns `full`. A `get` printed all 53,248 tokens exactly (53,795 positions, 160 s).
- When the store is full:
  - put new, put overwrite and append all return `full`.
  - get still works.
  - delete returns `ok`, but the next put still returns `full`. Chunks are never reclaimed, so 512 is a lifetime count of puts and appends per store, not a count of live files.
- A put longer than 104 tokens (490 tokens) returns no error. The text is silently truncated to its first 104 tokens.
- Model behaviour at the limit (`behave.py`, Qwen choosing freely, greedy, n=1 per prompt, full store loaded): in both cases the tool answered `full` and Qwen then told the user it had saved or replaced the text. It is a false success.
- Not tested: name-prefix collisions past NMAX. The test names were only 4 tokens long.

## 3. Speed (512-token prose prompt, 256 greedy tokens, batch 1, 1 warmup + 3 runs, mean ± sample sd)
| | prefill tok/s | decode tok/s | peak alloc |
|---|---|---|---|
| base Qwen3.5-4B, HF forward per token | 4303 ± 9 | 45.38 ± 0.34 | 9,471,702,016 B |
| Filesystem, author's RvChat path | 992 ± 37 | 31.04 ± 0.16 | 18,860,999,168 B |

- The Filesystem path is the author's RvChat: chip layers compacted and run as a CUDA graph, Qwen layers in HF. Its prefill is a full-model prefill plus a chip-view re-prefill in chunks of 128.
- The two paths are not identical stacks. Loading the Filesystem checkpoint plainly in HF ran out of memory on 24 GB (above 20.4 GiB of bf16 weights), so a like-for-like HF number does not exist.
- Base weights include the vision tower (9.08 GB).
- On this text the first 40 greedy tokens were identical for both models, and the circuit fired 0 times.

## 4. KL to base (method of exp_kl.py)
Method: the first 512 tokens of each file. Base log-probs come from an sdpa forward; the noise floor is sdpa vs eager. The merged model ran token by token through RvChat (chip step + Qwen step). The value is the mean KL(base‖merged) over 511 positions.

Sources:
- Rust: itoa 1.0.18 src/lib.rs from line 54 (MIT OR Apache-2.0).
- Python: CPython 3.12 Lib/textwrap.py from line 17 (PSF-2.0).
- Prose: the reproduction's text.txt.

| text | noise floor | empty store | memory.npz loaded | full store (512 files) | top-1 agree |
|---|---|---|---|---|---|
| Rust | 4.10e-4 | 5.11e-4 | 5.11e-4 | 5.11e-4 | 0.998 |
| Python | 6.12e-4 | 7.31e-4 | 7.31e-4 | 7.31e-4 | 0.992 |
| prose | 7.23e-4 | 6.69e-4 | 6.69e-4 | 6.69e-4 | 0.994 |

The circuit fired 0 times in every run, and the store contents had no effect: the numbers are identical to every printed digit.

## 5. Arithmetic (the author's chipformer.rv.evalarith)
Problems: DeepMind mathematics_dataset v1.0 extrapolate `arithmetic__mul_big` and `arithmetic__div_big`, n=40, seed 0. These are the same problems as the author's table. Answers were scored by exact rational match, with a budget of 4096 Qwen tokens.

| set | CPU model (rv1, batch 4) | base, no tool (base_nudge, batch 8) | author rv1 / base_nudge |
|---|---|---|---|
| mul_big | **39/40** | **22/40** | 39/40 / 22/40 |
| div_big | **40/40** | **29/40** | 38/40 (v1), 39/40 (v2) / 29/40 |

- Lockstep errors: 0.
- Latency, as wall-clock per problem inside a batch:
  - rv1: mean 15.1 s / 16.5 s, median about 9.2 s.
  - base: mean 291 s / 304 s.
  - Throughput: rv1 562 and 544 problems/h; base 225 and 248 problems/h.
- Mean tokens per problem: rv1 used about 185–229 Qwen tokens, 1–2 calc calls and about 2.3k chip tokens; base used about 1,890 Qwen tokens.
- Batched wall time is not single-stream latency.

## What failed or was not done
- The plain HF Filesystem speed run ran out of memory, as described above.
- The name-collision test did not exercise NMAX.
- `capacity.py`'s final json.dump crashed on an ndarray in the config. Its results were taken from `raw/logs/capacity.log`, which printed every value.
- The GPU lock was heavily contended, which made wall time long. The actual GPU time was about 1 h.
