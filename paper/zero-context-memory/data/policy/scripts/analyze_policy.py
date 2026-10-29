"""Score B1-B3 (+ A2, A4 rescored from the first run for reference) -> results.json + per-arm CSVs. usage: analyze_policy.py DST"""
import sys, json, os, re, string, csv
sys.path.append("/home/cam/gpsm-bench/pylib")
import numpy as np, regex
from collections import Counter
from nltk.stem import PorterStemmer
ps = PorterStemmer()
P = "/home/cam/gpsm-bench/policy/out"; A = "/home/cam/gpsm-bench/out"; DST = sys.argv[1]; os.makedirs(DST, exist_ok=True)
# --- LoCoMo official scorer, identical to the first run's analyze.py
# LoCoMo scorer: our reimplementation of the scoring logic in snap-research/locomo
# task_eval/evaluation.py (LoCoMo is CC BY-NC 4.0; Maharana et al., arXiv:2402.17753),
# in the style of the SQuAD evaluation script. Not a copy of their file.
def normalize_answer(s):
    s = s.replace(',', "")
    s = s.lower(); s = ''.join(ch for ch in s if ch not in set(string.punctuation))
    s = regex.sub(r'\b(a|an|the|and)\b', ' ', s); return ' '.join(s.split())
def exact_match_score(p, g): return set(normalize_answer(p).split()) == set(normalize_answer(g).split())
def f1_score(p, g):
    pt = [ps.stem(w) for w in normalize_answer(p).split()]; gt = [ps.stem(w) for w in normalize_answer(g).split()]
    c = sum((Counter(pt) & Counter(gt)).values())
    if c == 0: return 0.0
    pr, rc = c / len(pt), c / len(gt); return 2 * pr * rc / (pr + rc)
def f1_multi(p, g):
    return float(np.mean([max(f1_score(pp.strip(), gg.strip()) for pp in p.split(',')) for gg in g.split(',')]))
def locomo_f1(pred, ans, cat):
    if cat == 3: ans = ans.split(';')[0].strip()
    return f1_multi(pred, ans) if cat == 1 else f1_score(pred, ans)
def lev(a, b):
    d = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        prev, d[0] = d[0], i
        for j, cb in enumerate(b, 1):
            prev, d[j] = d[j], min(d[j] + 1, d[j - 1] + 1, prev + (ca != cb))
    return d[len(b)]
def clean(p):
    p = p.strip().splitlines()[0] if p.strip() else ""
    return p.strip().strip("`'\"").rstrip(".").strip().strip("`")
def boot(x, B=2000, seed=0):
    x = np.asarray(x, float); n = len(x)
    if n == 0: return None
    R = np.random.default_rng(seed); m = x[R.integers(0, n, (B, n))].mean(1)
    return [round(float(x.mean()), 4), round(float(np.percentile(m, 2.5)), 4), round(float(np.percentile(m, 97.5)), 4)]
def toks(s): return [ps.stem(w) for w in normalize_answer(s).split()]
def holds(ans, text, ds):
    if ds == "facts": return ans in text
    a = toks(ans); return bool(len(a)) and sum((Counter(a) & Counter(toks(text))).values()) / len(a) >= 0.5
CATS = {1: "multi-hop", 2: "temporal", 3: "open-domain", 4: "single-hop"}
TOOL = {"A4", "B2b300", "B2b1000", "B2snakeb300"}
res = {"locomo": {}, "facts": {}, "exactness": {}, "paired": {}, "store": {}}
rows_by = {}
SUB = [json.loads(l)["qid"] for l in open(f"{A}/locomo_A4.jsonl")]
for ds in ["locomo", "facts"]:
    for arm in ["A2", "A4", "B1", "B2b300", "B2b1000", "B2snakeb300", "B3"]:
        p = f"{A if arm in ('A2', 'A4') else P}/{ds}_{arm}.jsonl"
        if not os.path.exists(p): continue
        rows = [json.loads(l) for l in open(p)]
        if ds == "locomo": rows = [r for r in rows if r["qid"] in set(SUB)]
        for r in rows:
            if arm in TOOL:
                bud = r.get("budget", 300)
                r["pred_raw"] = r["pred"]; r["budget_exhausted"] = r.get("qwen_free", 0) >= bud
                if "<tool_call>" in r["pred"] or "<function=" in r["pred"]: r["pred"] = ""
                t = r["text"]; r["n_get"] = t.count("<function=get>"); r["n_list"] = t.count("<function=list>")
                got = re.findall(r"<function=get>.*?<tool_response>\s*(.*?)\s*</tool_response>", t, re.S)
                r["got_relevant"] = any(holds(r["answer"], v, ds) for v in got)
            if arm in ("B1", "B3", "A2") and "retrieved_text" in r:
                r["retrieved_relevant"] = any(holds(r["answer"], v, ds) for v in r["retrieved_text"])
            if ds == "locomo":
                r["f1"] = locomo_f1(r["pred"], r["answer"], r["category"]); r["em"] = float(exact_match_score(r["pred"], r["answer"]))
            else:
                c = clean(r["pred"]); r["em"] = float(c == r["answer"]); r["cer"] = lev(c, r["answer"]) / len(r["answer"])
                r["em_contains"] = float(r["answer"] in r["pred"])
        keys = ["f1", "em"] if ds == "locomo" else ["em", "em_contains", "cer"]
        out = {"n": len(rows)}
        for k in keys: out[k] = boot([r[k] for r in rows])
        out["prompt_tokens_mean"] = round(float(np.mean([r["prompt_tokens"] for r in rows])), 1)
        if arm in TOOL:
            out["context_tokens_mean"] = round(float(np.mean([r["context_tokens"] for r in rows])), 1)
            out["frac_budget_exhausted"] = round(float(np.mean([r["budget_exhausted"] for r in rows])), 3)
            out["frac_no_final_answer"] = round(float(np.mean([r["pred"] == "" for r in rows])), 3)
            out["frac_looked"] = round(float(np.mean([(r["n_get"] + r["n_list"]) > 0 for r in rows])), 3)
            out["gets_mean"] = round(float(np.mean([r["n_get"] for r in rows])), 2)
            out["frac_a_get_returned_answer_bearing_text"] = round(float(np.mean([r["got_relevant"] for r in rows])), 3)
        if any("retrieved_relevant" in r for r in rows):
            out["frac_top5_holds_answer"] = round(float(np.mean([r.get("retrieved_relevant", False) for r in rows])), 3)
        out["seconds_mean"] = round(float(np.mean([r["seconds"] for r in rows])), 2)
        bycat = {}
        for c in sorted({r["category"] for r in rows}, key=str):
            rr = [r for r in rows if r["category"] == c]
            bycat[CATS.get(c, c) if ds == "locomo" else c] = {"n": len(rr), **{k: boot([r[k] for r in rr]) for k in keys}}
        out["by_category"] = bycat
        res[ds][arm] = out; rows_by[(ds, arm)] = {r["qid"]: r for r in rows}
        if arm in ("A2", "A4"): continue
        with open(f"{DST}/{ds}_{arm}.csv", "w", newline="") as f:
            cols = ["qid", "category", "question", "answer", "pred"] + keys + ["prompt_tokens", "seconds"]
            cols += ["context_tokens", "n_get", "n_list", "budget_exhausted", "got_relevant"] if arm in TOOL else ["retrieved_relevant"]
            w = csv.writer(f); w.writerow(cols)
            for r in rows: w.writerow([round(r[c], 4) if isinstance(r.get(c), float) else r.get(c, "") for c in cols])
# --- paired differences (per question, same qids), bootstrap CI
def paired(ds, a, b, k):
    if (ds, a) not in rows_by or (ds, b) not in rows_by: return None
    q = sorted(set(rows_by[(ds, a)]) & set(rows_by[(ds, b)]))
    d = [rows_by[(ds, a)][x][k] - rows_by[(ds, b)][x][k] for x in q]
    same = sum(rows_by[(ds, a)][x]["pred"] == rows_by[(ds, b)][x]["pred"] for x in q)
    return {"n": len(q), "mean_diff": boot(d), "identical_predictions": int(same)}
for ds, k in (("locomo", "f1"), ("facts", "em")):
    for a, b in (("B3", "A2"), ("B1", "A2"), ("B1", "A4"), ("B2b300", "A4"), ("B2b1000", "B2b300"), ("B3", "B2b300"), ("B3", "B1"), ("B2snakeb300", "B2b300")):
        res["paired"][f"{ds}:{a}-{b}:{k}"] = paired(ds, a, b, k)
    if ("B2b1000", "B2b300") in [(x, y) for x, y in []]: pass
# B2b1000 ran on a subset: B2b300 on that same subset for a like-for-like budget comparison
for ds in ["locomo", "facts"]:
    if (ds, "B2b1000") in rows_by and (ds, "B2b300") in rows_by:
        q = set(rows_by[(ds, "B2b1000")]); k = "f1" if ds == "locomo" else "em"
        res[ds]["B2b300_on_B2b1000_subset"] = {"n": len(q), k: boot([rows_by[(ds, "B2b300")][x][k] for x in sorted(q)])}
# --- exactness of the engine-written store (B3 readback vs B2 write log), and B1 readback stats
for ds in ["locomo", "facts"]:
    wp, rp = f"{P}/{ds}_B2_write.json", f"{P}/{ds}_B3_files.json"
    if os.path.exists(wp) and os.path.exists(rp):
        W, R = json.load(open(wp)), json.load(open(rp)); ex = {}
        from transformers import AutoTokenizer
        _tk = AutoTokenizer.from_pretrained("/home/cam/gpsm-models/Qwen3.5-4B-Qwenputer-Filesystem")
        bytes_of = lambda ids: _tk.decode(ids).encode("utf-8")
        for iid in W:
            want = {tuple(e["name_ids"][:6]): [t for p in e["pieces"] for t in p + [198]] for e in W[iid]["entries"]}
            got = {tuple((f["name_ids"][:-1] if f["name_ids"][-1:] == [198] else f["name_ids"])[:6]): f["ids"] for f in R[iid]["files"] if f["found"]}
            mism = sum(1 for k in want if got.get(k) != want[k])
            ex[iid] = {"entries_written": len(want), "entries_listed": len(R[iid]["files"]), "entries_found": len(got),
                       "token_id_mismatches": mism,
                       "byte_mismatches": sum(1 for e in W[iid]["entries"] if bytes_of(got.get(tuple(e["name_ids"][:6]), [])) != b"".join(bytes_of(p) + b"\n" for p in e["pieces"])),
                       "writes": W[iid]["writes"], "tokens_stored": W[iid]["tokens_stored"],
                       "write_answers": dict(sum((Counter(s["answers"]) for s in W[iid]["sessions"]), Counter())),
                       "driver_conflicts_write": sum(s["conflicts"] for s in W[iid]["sessions"]),
                       "driver_conflicts_read": R[iid]["conflicts"], "readback_seconds": round(R[iid]["seconds"], 1)}
        res["exactness"][ds] = ex
    bp = f"{P}/{ds}_B1_files.json"
    if os.path.exists(bp):
        R = json.load(open(bp))
        res["store"][f"{ds}_A4_store_readback"] = {iid: {"names_listed": len(v["files"]), "found": sum(f["found"] for f in v["files"]),
            "text_tokens": sum(len(f["ids"]) for f in v["files"] if f["found"]), "driver_conflicts": v["conflicts"],
            "readback_seconds": round(v["seconds"], 1)} for iid, v in R.items()}
json.dump(res, open(f"{DST}/results.json", "w"), indent=1)
print(json.dumps({k: ({a: {kk: vv for kk, vv in x.items() if kk != "by_category"} for a, x in v.items()} if k in ("locomo", "facts") else v) for k, v in res.items()}, indent=1))
