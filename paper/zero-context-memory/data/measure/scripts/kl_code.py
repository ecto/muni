"""KL to base Qwen3.5-4B on a text file, exp_kl.py's method. usage: kl_code.py base|fs FILE LABEL"""
import sys, os, json, gc
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import torch, numpy as np
from pathlib import Path
WORK = Path(os.path.expanduser("~/gpsm-measure/work")); WORK.mkdir(exist_ok=True)
OUTD = Path(os.path.expanduser("~/gpsm-measure/out"))
BASE = os.environ["BASE"]; mode, fn, label = sys.argv[1:4]; NT = 512
NQ = 248320
def lsm(lg): return torch.log_softmax(lg[..., :NQ].float(), -1)
if mode == "base":
    from transformers import AutoModelForImageTextToText, AutoTokenizer
    tok = AutoTokenizer.from_pretrained(BASE)
    ids = tok(open(fn).read()).input_ids[:NT]
    np.save(WORK / f"{label}_ids.npy", np.array(ids))
    for impl in ("sdpa", "eager"):
        m = AutoModelForImageTextToText.from_pretrained(BASE, dtype=torch.bfloat16, attn_implementation=impl).to("cuda").eval()
        with torch.no_grad():
            lp = lsm(m(input_ids=torch.tensor([ids], device="cuda")).logits[0])
        torch.save(lp[:-1].half().cpu(), WORK / f"{label}_base_{impl}.pt"); print(impl, len(ids), flush=True)
        del m; gc.collect(); torch.cuda.empty_cache()
else:
    from repro import *
    chat = build_chat()
    ids = [int(x) for x in np.load(WORK / f"{label}_ids.npy")]
    base = torch.load(WORK / f"{label}_base_sdpa.pt").float(); eager = torch.load(WORK / f"{label}_base_eager.pt").float()
    def kl(p, q): return (p.exp() * (p - q)).sum(-1)
    res = {"file": fn, "n_tokens": len(ids), "positions_compared": len(ids) - 1,
           "noise_floor_sdpa_vs_eager": float(kl(base, eager).mean()), "noise_floor_excl_pos0": float(kl(base, eager)[1:].mean()),
           "noise_top1_agree": float((base.argmax(-1) == eager.argmax(-1)).float().mean())}
    conds = [("empty_store", None), ("store_3files_memory_npz", CH.load_store_file(os.path.expanduser("~/qwenputer-repro/traces/memory.npz")))]
    if (OUTD / "full_small.npz").exists(): conds.append(("store_full_512_files", CH.load_store_file(str(OUTD / "full_small.npz"))))
    for lab, snap in conds:
        out = chat._prefill(ids[:1]); cache = out.past_key_values
        core.snapshot(chat.c, machine()); chat.load_store(snap)
        ccache = chat._chip_cache()
        with torch.no_grad():
            h = chat.cm.embed_tokens(torch.tensor([ids[:1]], device=chat.dev))
            for L in range(chat.n_chip):
                h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=ccache)
        hf.inject(chat.c, ccache)
        if chat.graph is None: chat._capture()
        lps = [lsm(out.logits[0, -1]).cpu()]; fired = 0
        with torch.no_grad():
            for p in range(1, len(ids) - 1):
                hc = chat._chip_step(ids[p], p, ccache)
                if chat.circuit_token(hc) is not None: fired += 1
                lps.append(lsm(chat._qwen_step(hc, cache, p)).cpu())
        mg = torch.stack(lps); k = kl(base, mg)
        res[lab] = {"kl_mean": float(k.mean()), "kl_max": float(k.max()), "kl_median": float(k.median()),
                    "kl_mean_excl_pos0": float(k[1:].mean()),
                    "top1_agree": float((base.argmax(-1) == mg.argmax(-1)).float().mean()), "circuit_fired": fired}
        print(lab, res[lab], flush=True)
    json.dump(res, open(OUTD / f"kl_{label}.json", "w"), indent=1); print(json.dumps(res, indent=1))
