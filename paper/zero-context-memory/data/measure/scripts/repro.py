"""Our own driver for the qwenputer Filesystem checkpoint (kept outside the untrusted repo)."""
import sys, time, json, os
REPO = os.path.expanduser("~/qwenputer-repro/repo/experimental/riscv")
sys.path.insert(0, REPO)
import numpy as np, torch
from pathlib import Path
from chipformer.rv import chat as CH, core, elf, hf, ref, bridge as BR, spec as S, store as ST
from safetensors.numpy import save_file

M = os.environ["FS"]
OUT = Path(os.path.expanduser("~/gpsm-measure/out")); OUT.mkdir(exist_ok=True)
FW = REPO + "/tests/data/rv/calc-bridge.elf"
log = open(OUT / "run.log", "a")
def P(*a):
    s = " ".join(str(x) for x in a); print(s, flush=True); log.write(s + "\n"); log.flush()

def machine():
    m = ref.Machine.from_program(elf.program(FW)); BR.park(m); return m

def tname(chat, t):
    return S.name(t - chat.v0) if t >= chat.v0 else repr(chat.tok.decode([t]))

def prompt(chat, text, system=True):
    msgs = ([{"role": "system", "content": ST.MEM_SYSTEM}] if system else []) + [{"role": "user", "content": text}]
    return chat.prompt_ids(msgs, (CH.CALC_TOOL, *ST.TOOLS))

@torch.no_grad()
def forced(chat, ids, ops, snap=None, dump=None, free_max=0):
    """Teacher-forced: Qwen's positions take the next forced token; circuit-owned positions take
    the circuit's token. ops: list of (label, token list). Returns per-op stats."""
    out = chat._prefill(ids, chunk=128); cache = out.past_key_values
    first = int(torch.argmax(out.logits[0, -1][: chat.v0]))
    core.snapshot(chat.c, machine()); chat.load_store(snap)
    ccache = chat._chip_cache()
    for a in range(0, len(ids), 128):
        h = chat.cm.embed_tokens(torch.tensor([ids[a:a+128]], device=chat.dev))
        for L in range(chat.n_chip):
            h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=ccache)
    del h; torch.cuda.empty_cache()
    hf.inject(chat.c, ccache)
    if chat.graph is None: chat._capture()
    g = pos = len(ids)
    stats, steps = [], []
    for label, toks in ops:
        queue = list(toks); nxt = queue.pop(0) if queue else first; n_free = 0
        torch.cuda.synchronize(); t0 = time.time(); n_chip = n_circ_text = n_forced = 0
        while True:
            chip_in = nxt >= chat.v0
            hc = chat._chip_step(nxt, g, ccache); g += 1
            steps.append((label, nxt, "chip" if chip_in else "text"))
            if chip_in:
                n_chip += 1; nxt = chat.circuit_token(hc); assert nxt is not None
                continue
            logits = chat._qwen_step(hc, cache, pos); pos += 1
            ct = chat.circuit_token(hc)
            if ct is not None:
                if ct < chat.v0: n_circ_text += 1
                nxt = ct; continue
            if queue: nxt = queue.pop(0); n_forced += 1; continue
            if n_free < free_max and nxt not in chat.eos:
                nxt = int(torch.argmax(logits[: chat.v0])); n_free += 1; continue
            break
        torch.cuda.synchronize(); dt = time.time() - t0
        seq = [s for s in steps if s[0] == label]
        st = {"op": label, "seconds": round(dt, 4), "forced_tokens": len(toks), "chip_tokens": n_chip,
              "circuit_text_tokens": n_circ_text, "qwen_free_tokens": n_free, "positions": len(seq),
              "trace": [tname(chat, s[1]) for s in seq]}
        stats.append(st)
        P(json.dumps({k: v for k, v in st.items() if k != "trace"}))
        P("  ", "".join(x if x.startswith("'") is False else x[1:-1] for x in st["trace"][len(toks)-1:]))
        if dump is not None: dump(label, ccache, seq)
    return stats, ccache, steps

def build_chat():
    return CH.RvChat(M)

def verify(chat, prompt_ids, gen):
    """The reference StoreSession fed Qwen's own tokens must emit exactly the circuit's tokens."""
    ses = ST.StoreSession(chat.T, machine(), chat.store_cfg); ses.history(prompt_ids)
    stream = chat.to_stream(gen); pos = 0; n_owned = 0
    while pos < len(stream):
        t = stream[pos]
        if not S.is_text(t): return {"ok": False, "where": pos, "why": "chip token where reference has none"}
        k = len(ses.toks); ses.qwen([t - S.TEXT_TOKEN]); pos += 1
        added = ses.toks[k + 1:]
        if added:
            got = stream[pos:pos + len(added)]
            if got != added[:len(got)]: return {"ok": False, "where": pos, "got": got[:5], "want": added[:5]}
            n_owned += len(added); pos += len(added)
    return {"ok": True, "circuit_owned_checked": n_owned, "runs": len(ses.runs)}
