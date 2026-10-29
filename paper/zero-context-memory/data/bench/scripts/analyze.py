"""Score every arm, bootstrap CIs, A4 reach stats -> results.json + per-arm CSVs."""
import sys, json, os, re, string, csv, glob
sys.path.append("/home/cam/gpsm-bench/pylib")
import numpy as np, regex
from collections import Counter
from nltk.stem import PorterStemmer
ps = PorterStemmer()
OUT = "/home/cam/gpsm-bench/out"; DST = sys.argv[1]; os.makedirs(DST, exist_ok=True)
# --- LoCoMo scorer: our reimplementation of the scoring logic in snap-research/locomo
#     task_eval/evaluation.py (LoCoMo is CC BY-NC 4.0; Maharana et al., arXiv:2402.17753),
#     itself in the style of the SQuAD evaluation script. Not a copy of their file.
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
CATS = {1: "multi-hop", 2: "temporal", 3: "open-domain", 4: "single-hop"}
res = {"locomo": {}, "facts": {}}
_a4 = f"{OUT}/locomo_A4.jsonl"
SUB = {json.loads(l)["qid"] for l in open(_a4)} if os.path.exists(_a4) else None
for ds in ["locomo", "facts"]:
    for arm in ["A0", "A1", "A2", "A3", "A4"]:
        p = f"{OUT}/{ds}_{arm}.jsonl"
        if not os.path.exists(p): continue
        rows = [json.loads(l) for l in open(p)]
        if ds == "locomo" and SUB is not None and arm != "A4":
            full = [dict(r) for r in rows]
            for r in full: r["f1"] = locomo_f1(r["pred"], r["answer"], r["category"]); r["em"] = float(exact_match_score(r["pred"], r["answer"]))
            res.setdefault("locomo_all150_base_arms", {})[arm] = {"n": len(full), "f1": boot([r["f1"] for r in full]), "em": boot([r["em"] for r in full]),
                "prompt_tokens_mean": round(float(np.mean([r["prompt_tokens"] for r in full])), 1)}
            rows = [r for r in rows if r["qid"] in SUB]
        for r in rows:
            if arm == "A4":
                r["pred_raw"] = r["pred"]; r["budget_exhausted"] = r.get("qwen_free", 0) >= 300
                if "<tool_call>" in r["pred"] or "<function=" in r["pred"]: r["pred"] = ""
            if ds == "locomo":
                r["f1"] = locomo_f1(r["pred"], r["answer"], r["category"]); r["em"] = float(exact_match_score(r["pred"], r["answer"]))
            else:
                c = clean(r["pred"]); r["em"] = float(c == r["answer"]); r["cer"] = lev(c, r["answer"]) / len(r["answer"])
                r["em_contains"] = float(r["answer"] in r["pred"])
        keys = ["f1", "em"] if ds == "locomo" else ["em", "em_contains", "cer"]
        out = {"n": len(rows)}
        for k in keys: out[k] = boot([r[k] for r in rows])
        out["prompt_tokens_mean"] = round(float(np.mean([r["prompt_tokens"] for r in rows])), 1)
        if arm == "A4": out["context_tokens_mean"] = round(float(np.mean([r["context_tokens"] for r in rows])), 1)
        out["seconds_mean"] = round(float(np.mean([r["seconds"] for r in rows])), 2)
        bycat = {}
        for c in sorted({r["category"] for r in rows}, key=str):
            rr = [r for r in rows if r["category"] == c]
            bycat[CATS.get(c, c) if ds == "locomo" else c] = {"n": len(rr), **{k: boot([r[k] for r in rr]) for k in keys}}
        out["by_category"] = bycat
        res[ds][arm] = out
        with open(f"{DST}/{ds}_{arm}.csv", "w", newline="") as f:
            cols = ["qid", "category", "question", "answer", "pred"] + keys + ["prompt_tokens", "seconds"] + (["context_tokens", "n_get", "n_list"] if arm == "A4" else [])
            w = csv.writer(f); w.writerow(cols)
            for r in rows:
                if arm == "A4":
                    r["n_get"] = r["text"].count("<function=get>"); r["n_list"] = r["text"].count("<function=list>")
                w.writerow([r.get(c, "") if not isinstance(r.get(c), float) else round(r[c], 4) for c in cols])
        res[ds][arm]["_rows"] = rows
# --- A4 reach statistics
PUT = re.compile(r"<function=(put|append)>\s*<parameter=name>\s*(.*?)\s*</parameter>\s*<parameter=text>\s*(.*?)\s*</parameter>\s*</function>\s*</tool_call>\s*<tool_response>\s*(.*?)\s*</tool_response>", re.S)
OPS = re.compile(r"<function=(put|append|delete|get|list)>(.*?)</function>\s*</tool_call>\s*<tool_response>(.*?)</tool_response>", re.S)
def toks(s): return [ps.stem(w) for w in normalize_answer(s).split()]
for ds in ["locomo", "facts"]:
    wp = f"{OUT}/{ds}_A4_write.jsonl"
    if not os.path.exists(wp) or "A4" not in res[ds]: continue
    W = [json.loads(l) for l in open(wp)]
    store = {}; per = []; malformed_w = 0
    for w in W:
        calls = PUT.findall(w["text"]); per.append(len(calls))
        malformed_w += max(0, w["text"].count("<tool_call>") - w["text"].count("<tool_response>"))
        d = store.setdefault(w["inst"], {})
        for m in OPS.finditer(w["text"]):
            op, body, resp = m.group(1), m.group(2), m.group(3).strip()
            nm = re.search(r"<parameter=name>\s*(.*?)\s*</parameter>", body, re.S)
            tx = re.search(r"<parameter=text>\s*(.*?)\s*</parameter>", body, re.S)
            if not nm: continue
            name = nm.group(1)
            if op == "put" and resp == "ok" and tx: d[name] = tx.group(1)
            elif op == "append" and resp == "ok" and tx: d[name] = d.get(name, "") + " " + tx.group(1)
            elif op == "delete" and resp == "ok": d.pop(name, None)
    finals = {}
    for fp in glob.glob(f"{OUT}/{ds}_A4_final_*.json"):
        j = json.load(open(fp)); body = j["text"].split("<tool_response>")[-1].split("</tool_response>")[0] if "<tool_response>" in j["text"] else ""
        finals[j["inst"]] = [x for x in body.strip().splitlines() if x.strip()]
    rows = res[ds]["A4"]["_rows"]
    looked, rel, rel_looked, got_right = [], [], [], []
    fail = Counter()
    for r in rows:
        t = r["text"]; ng = t.count("<function=get>"); nl = t.count("<function=list>")
        lk = (ng + nl) > 0; looked.append(lk)
        a = toks(r["answer"]) if ds == "locomo" else None
        files = store.get(r["inst"], {})
        if ds == "locomo":
            ex = any(len(a) and sum((Counter(a) & Counter(toks(v))).values()) / len(a) >= 0.5 for v in files.values())
        else:
            ex = any(r["answer"] in v for v in files.values())
        rel.append(ex)
        got = re.findall(r"<function=get>\s*<parameter=name>\s*(.*?)\s*</parameter>.*?<tool_response>\s*(.*?)\s*</tool_response>", t, re.S)
        resp_has = any((r["answer"] in v) if ds == "facts" else (len(a) and sum((Counter(a) & Counter(toks(v))).values()) / len(a) >= 0.5) for _, v in got)
        r["reach"] = dict(looked=lk, n_get=ng, n_list=nl, relevant_file_exists=ex, retrieved_relevant=bool(resp_has))
        if t.count("<tool_call>") > t.count("<tool_response>"): fail["unfinished_tool_call_at_end"] += 1
        if r["budget_exhausted"]: fail["tool_loop_hit_300_token_budget"] += 1
        if r["pred"] == "": fail["no_final_answer"] += 1
        good = r.get("f1", r["em"]) >= 0.5
        if ex and not lk: fail["stored_not_looked_up"] += 1
        elif ex and lk and not resp_has: fail["looked_up_wrong_file"] += 1
        elif ex and resp_has and not good: fail["retrieved_but_answered_wrong"] += 1
        elif not ex and lk: fail["looked_up_but_not_stored"] += 1
        elif not ex and not lk: fail["not_stored_not_looked_up"] += 1
    n = len(rows); L = np.array(looked); E = np.array(rel)
    allnames = [nm for d in store.values() for nm in d]
    res[ds]["A4_reach"] = {
        "write_sessions": len(W), "puts_per_session_mean": round(float(np.mean(per)), 2),
        "sessions_with_zero_puts": int(sum(p == 0 for p in per)), "puts_total": int(sum(per)),
        "malformed_write_tool_calls": malformed_w,
        "distinct_names_parsed": {k: len(v) for k, v in store.items()},
        "final_list_lines_undercounts_long_names": {k: len(v) for k, v in finals.items()},
        "store_fill_names_of_512": {k: round(len(v) / 512, 3) for k, v in finals.items()},
        "store_fill_chunks_of_512_upper_bound": {k: round(sum(1 for w in W if w["inst"] == k for _ in PUT.findall(w["text"])) / 512, 3) for k in store},
        "stored_text_tokens_approx": {k: int(sum(len(v.split()) * 1.3 for v in d.values())) for k, d in store.items()},
        "write_sessions_hitting_token_cap": int(sum(w["qwen_free"] >= (1500 if ds == "facts" else 1000) for w in W)),
        "write_deletes_total": int(sum(w["text"].count("<function=delete>") for w in W)),
        "write_seconds_mean": round(float(np.mean([w["seconds"] for w in W])), 1),
        "n_questions": n, "frac_looked": round(float(L.mean()), 3),
        "frac_relevant_file_exists": round(float(E.mean()), 3),
        "frac_looked_given_relevant": round(float(L[E].mean()), 3) if E.any() else None,
        "frac_looked_given_not_relevant": round(float(L[~E].mean()), 3) if (~E).any() else None,
        "frac_relevant_given_looked": round(float(E[L].mean()), 3) if L.any() else None,
        "frac_retrieved_relevant_given_relevant_and_looked": round(float(np.mean([r["reach"]["retrieved_relevant"] for r in rows if r["reach"]["looked"] and r["reach"]["relevant_file_exists"]])), 3) if (L & E).any() else None,
        "failure_modes": dict(fail),
        "relevance_rule": ("locomo: a stored file (parsed from the model's own put calls) holds >=50% of the gold answer's stemmed tokens; "
                           "facts: a stored file contains the exact answer string"),
    }
for ds in res:
    for arm in list(res[ds]):
        if isinstance(res[ds][arm], dict): res[ds][arm].pop("_rows", None)
json.dump(res, open(f"{DST}/results.json", "w"), indent=1)
print(json.dumps({ds: {a: {k: v for k, v in x.items() if k != "by_category"} for a, x in res[ds].items()} for ds in res}, indent=1))
