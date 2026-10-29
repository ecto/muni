"""Store sizes and capacity of the qwenputer Filesystem checkpoint (chip-only forced fills)."""
import sys, os, json, time, io
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from repro import *
OUTD = OUT
chat = build_chat(); T = chat.tok
enc = lambda s: T.encode(s, add_special_tokens=False)
SS = ST.StoreSession
R = {"store_cfg": chat.store_cfg.__dict__ if chat.store_cfg else None,
     "store_heads": {k: len(n.vheads()) for k, n in chat.c.store.persistent.items()}}
print(R, flush=True)

@torch.no_grad()
def start(snap=None):
    ids = prompt(chat, "Use the memory tools.")
    core.snapshot(chat.c, machine()); chat.load_store(snap)
    cc = chat._chip_cache()
    for a in range(0, len(ids), 128):
        h = chat.cm.embed_tokens(torch.tensor([ids[a:a+128]], device=chat.dev))
        for L in range(chat.n_chip):
            h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=cc)
    hf.inject(chat.c, cc)
    if chat.graph is None: chat._capture()
    return cc, len(ids)

@torch.no_grad()
def op(st, toks, maxpos=200000):
    """Feed forced tokens through the chip layers only; circuit-owned positions take its token.
    Returns the circuit's printed text and chip records."""
    cc, g = st["cc"], st["g"]
    queue = list(toks); nxt = queue.pop(0); out = []
    n = 0
    while n < maxpos:
        hc = chat._chip_step(nxt, g, cc); g += 1; n += 1
        ct = chat.circuit_token(hc)
        if nxt >= chat.v0:
            assert ct is not None; nxt = ct; out.append(ct); continue
        if ct is not None: nxt = ct; out.append(ct); continue
        if queue: nxt = queue.pop(0); continue
        break
    st["g"] = g
    txt = chat.text_of(out); chips = [S.name(t - chat.v0) for t in out if t >= chat.v0]
    body = txt.split("<tool_response>\n")[-1].split("</tool_response>")[0] if "<tool_response>" in txt else txt
    return body, chips, n

def snapbytes(snap):
    p = OUTD / "tmp.npz"; CH.save_store_file(str(p), snap); b = os.path.getsize(p)
    nk = sum(len(r) for vs in snap.values() for r in vs)
    return {"npz_bytes": b, "rows_saved": nk, "raw_float64_bytes": nk * 128 * 8}

def save(st, name):
    snap = chat.save_store(("hf", st["cc"]))
    CH.save_store_file(str(OUTD / f"{name}.npz"), snap)
    return snap, snapbytes(snap)

# --- sizes: empty, 3 files (the repro's three small files)
st = {}; st["cc"], st["g"] = start()
R["size_empty"] = save(st, "empty")[1]
files = [("a.txt", "apples are red"), ("b.txt", "bananas are yellow"), ("c.txt", "cherries are dark")]
for n_, t_ in files: op(st, SS.call_tokens("put", enc(n_), enc(t_)))
R["size_3files"] = save(st, "three")[1]
# truncation and name-prefix checks
long_text = " ".join(f"w{i}" for i in range(150))
op(st, SS.call_tokens("put", enc("long"), enc(long_text)))
body, _, _ = op(st, SS.call_tokens("get", enc("long")))
R["text_truncation"] = {"put_tokens": len(enc(long_text)), "get_tokens": len(enc(body.rstrip("\n"))), "cmax": ST.CMAX, "get_equals_first_cmax": body.rstrip("\n") == T.decode(enc(long_text)[:ST.CMAX]),
                        "put_response": None}
nA, nB = "notes_alpha_project_one", "notes_alpha_project_two"
R["name_prefix"] = {"name_tokens": [len(enc(nA)), len(enc(nB))], "nmax": ST.NMAX}
op(st, SS.call_tokens("put", enc(nA), enc(" first")))
op(st, SS.call_tokens("put", enc(nB), enc(" second")))
R["name_prefix"]["get_first_name"] = op(st, SS.call_tokens("get", enc(nA)))[0]
R["name_prefix"]["list"] = op(st, SS.call_tokens("list"))[0]
print(R, flush=True)

# --- capacity, small files
st = {}; st["cc"], st["g"] = start(); t0 = time.time(); fill = []
i = 0
while True:
    body, chips, n = op(st, SS.call_tokens("put", enc(f"f{i}"), enc(" short note")))
    fill.append(body.strip())
    if body.strip() != "ok" or i > 1100: break
    i += 1
R["small_fill"] = {"text": " short note", "text_tokens": len(enc(" short note")), "ok_puts": sum(b == "ok" for b in fill),
                   "first_non_ok_index": i, "first_non_ok_response": fill[-1], "seconds": time.time() - t0}
print(R["small_fill"], flush=True)
after = {}
for lab, toks in [("put_new", SS.call_tokens("put", enc("zz"), enc(" more"))),
                  ("put_overwrite_f0", SS.call_tokens("put", enc("f0"), enc(" changed"))),
                  ("append_f1", SS.call_tokens("append", enc("f1"), enc(" more"))),
                  ("get_f0", SS.call_tokens("get", enc("f0"))),
                  ("get_f511", SS.call_tokens("get", enc(f"f{i-1}"))),
                  ("delete_f2", SS.call_tokens("delete", enc("f2"))),
                  ("put_after_delete", SS.call_tokens("put", enc("yy"), enc(" more")))]:
    after[lab] = op(st, toks)[0][:200]
lst = op(st, SS.call_tokens("list"))[0]
after["list_names"] = len([x for x in lst.split("\n") if x.strip()])
R["small_full_behavior"] = after
snap, R["size_full_small"] = save(st, "full_small")
print(after, flush=True)

# --- capacity, one big file grown by 104-token appends, and big puts
st = {}; st["cc"], st["g"] = start(); t0 = time.time()
chunk = enc(" " + " ".join(f"z{k}" for k in range(80)))[:ST.CMAX]
assert len(chunk) == ST.CMAX
resp = []
j = 0
while True:
    body, _, _ = op(st, SS.call_tokens("append", enc("big"), chunk))
    resp.append(body.strip())
    if body.strip() != "ok" or j > 1100: break
    j += 1
R["big_fill"] = {"chunk_tokens": len(chunk), "ok_appends": sum(b == "ok" for b in resp), "first_non_ok_response": resp[-1],
                 "seconds": time.time() - t0}
t1 = time.time(); body, chips, n = op(st, SS.call_tokens("get", enc("big")), maxpos=400000)
gt = enc(body.rstrip("\n"))
R["big_fill"]["get_tokens"] = len(gt); R["big_fill"]["get_positions"] = n; R["big_fill"]["get_seconds"] = time.time() - t1
R["big_fill"]["get_exact"] = body.rstrip("\n") == T.decode(chunk * R["big_fill"]["ok_appends"])
snap, R["size_full_big"] = save(st, "full_big")
print(R["big_fill"], flush=True)
json.dump(R, open(OUTD / "capacity.json", "w"), indent=1)
