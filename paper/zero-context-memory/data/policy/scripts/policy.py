"""Policy arms B1-B3 on the same GPSM store (qwenputer filesystem checkpoint, author's driver).
usage: policy.py readback-a4 DS | write-b2 DS | readback-b2 DS | answer ARM DS | ask-b2 DS BUDGET [max_items]
Driver-issued store calls are fed token for token through the checkpoint's chip layers (the store circuit);
the circuit answers them itself (its own records and printed bodies), exactly as when Qwen writes a call."""
import os, sys, json, time, re, math
from collections import Counter
sys.path.insert(0, "/home/cam/gpsm-bench/scripts")
import bench as B
P = "/home/cam/gpsm-bench/policy"; OUT = f"{P}/out"; STORE = f"{P}/store"
SNAKE = os.environ.get("NAMES") == "snake"  # diagnostic: snake_case names (the checkpoint's own style), own store/arm
if SNAKE: STORE += "_snake"
os.makedirs(OUT, exist_ok=True); os.makedirs(STORE, exist_ok=True)
RAG_K = 5; RAG_TURNS = 6; MAXTXT = 103  # a put/append keeps <=104 tokens, the closing newline is one of them
STOP = set("""about above after again against also always anything because been before being below between both
could did does doing down during each even ever every from further going gonna have having here hers herself
himself into itself just like made make more most much myself never only other ought ours ourselves over really
same should some such than that thats their theirs them themselves then there these they thing things this those
through too under until very want was were what when where which while will with would your yours yourself
yourselves yeah great good thanks thank glad know think feel love sounds sure maybe right well still lots awesome
amazing totally sometimes something someone people time cool nice wow hope wish things being doing quick note
later""".split())

def compact_date(d):
    d = d.split(" on ")[-1] if " on " in d else d
    return d.replace(",", "").strip()

# ------------------------------------------------------------------ engine chunking and naming (B2/B3)
def plan_entries(inst, tok):
    """-> [{name, name_ids, pieces:[ids], text, session}] one entry per 6-turn chunk (A2's chunking)."""
    chunks = []
    for s in inst["sessions"]:
        for k, a in enumerate(range(0, len(s["turns"]), RAG_TURNS)):
            turns = s["turns"][a:a + RAG_TURNS]
            lines = [f"[Session {s['n']}, {s['date']}]"] + [B.fmt_turn(t) for t in turns]
            chunks.append(dict(s=s, k=k + 1, lines=lines, speakers=sorted({t["speaker"] for t in turns})))
    words = lambda c: [w.lower() for w in re.findall(r"[A-Za-z]{4,}", " ".join(c["lines"][1:]))]
    spk = {w.lower() for c in chunks for x in c["speakers"] for w in x.split()}
    df = Counter(w for c in chunks for w in set(words(c)))
    N = len(chunks); keys = set(); out = []
    for c in chunks:
        tf = Counter(w for w in words(c) if w not in STOP and w not in spk)
        ranked = sorted(tf, key=lambda w: (-tf[w] * math.log(N / df[w]), w))
        single = [w for w in ranked if len(tok.encode(w, add_special_tokens=False)) == 1
                  and len(tok.encode(" " + w, add_special_tokens=False)) == 1]
        # rev3: a name is two salient words, <=5 tokens, unique in the store. list prints only a name's first
        # 6 tokens and only names of <=5 tokens end in a newline (rev1's "<kw> <kw> s<n>c<k> <date> <speakers>"
        # ran together in the listing); the model dropped rev2's " s<n>" suffix when it called get. The
        # session and date are the first line of the text.
        cands = [(a, b) for i, a in enumerate(single[:10]) for b in single[i + 1:10]]
        cands += [(a, f"{b} {d}") for i, a in enumerate(single[:6]) for b in single[i + 1:6] for d in single[:6] if d not in (a, b)]
        cands += [(f"s{c['s']['n']}c{c['k']}", None)]
        for a, b in cands:
            name = (f"{a}_{b.replace(' ', '_')}" if SNAKE else f"{a} {b}") if b else a
            ids = tok.encode(name, add_special_tokens=False)
            if len(ids) > 5: continue
            key = (min(len(ids) + 1, 7), tuple((ids + [198, 510, 0, 0, 0, 0, 0])[:6]))
            if key not in keys: keys.add(key); break
        else: raise SystemExit(f"no unique name for s{c['s']['n']} c{c['k']}")
        # pieces: whole lines packed into <=103 tokens; a longer line is split at spaces
        pieces, cur = [], []
        def enc(ls): return tok.encode("\n".join(ls), add_special_tokens=False)
        flat = []
        for L in c["lines"]:
            if len(enc([L])) <= MAXTXT: flat.append(L); continue
            ws, part = L.split(" "), []
            for w in ws:
                if part and len(enc([" ".join(part + [w])])) > MAXTXT: flat.append(" ".join(part)); part = []
                part.append(w)
            flat.append(" ".join(part))
        for L in flat:
            if cur and len(enc(cur + [L])) > MAXTXT: pieces.append(enc(cur)); cur = []
            cur.append(L)
        pieces.append(enc(cur))
        assert all(len(p) <= MAXTXT for p in pieces)
        out.append(dict(name=name, name_ids=ids, pieces=pieces, session=c["s"]["n"],
                        text="\n".join(c["lines"])))
    return out

# ------------------------------------------------------------------ the driver
_D = []
def setup():
    if _D: return _D[0]
    chat, CH, S, run, prompt = B.gpsm_setup()
    from chipformer.rv import core, elf, hf, ref, bridge as BR, store as ST
    FW = B.REPO + "/tests/data/rv/calc-bridge.elf"
    def machine():
        m = ref.Machine.from_program(elf.program(FW)); BR.park(m); return m
    import torch
    @torch.no_grad()
    def drive(ids, snap, calls):
        """Prefill ids through the chip layers with the store snap loaded, then feed each driver call's tokens.
        Positions the circuit owns take the circuit's token (as in bench.run). Qwen's layers are not run:
        no position here is Qwen's choice. -> (gen, new snap, responses[list of segments], conflicts)"""
        core.snapshot(chat.c, machine()); chat.load_store(snap)
        cc = chat._chip_cache()
        for a in range(0, len(ids), 128):
            h = chat.cm.embed_tokens(torch.tensor([ids[a:a + 128]], device=chat.dev))
            for L in range(chat.n_chip):
                h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=cc)
        del h
        hf.inject(chat.c, cc)
        if chat.graph is None: chat._capture()
        g = len(ids); gen = []; conflicts = 0
        pending = list(calls); queue = list(pending.pop(0)); qi = 0
        nxt = queue[0]; qi = 1
        while True:
            gen.append(nxt)
            hc = chat._chip_step(nxt, g, cc); g += 1
            ct = chat.circuit_token(hc)
            if nxt >= chat.v0:
                assert ct is not None; nxt = ct; continue
            if ct is not None:
                if qi < len(queue):
                    if queue[qi] == ct: qi += 1
                    else: conflicts += 1
                nxt = ct; continue
            if qi < len(queue): nxt = queue[qi]; qi += 1; continue
            if pending:
                queue = [ST.Q_NL] + list(pending.pop(0)); nxt = queue[0]; qi = 1; continue
            break
        # split responses: from each <tool_response> to </tool_response>, keeping chip tokens as None markers
        resp, cur = [], None
        for t in gen:
            if t == ST.Q_TRO: cur = []
            elif t == ST.Q_TRC and cur is not None: resp.append(cur); cur = None
            elif cur is not None: cur.append(None if t >= chat.v0 else t)
        return gen, chat.save_store(("hf", cc)), resp, conflicts
    def call(op, name=None, text=None, raw_name=False):
        """Qwen3.5's spelling of a call (store.StoreSession.call_tokens). raw_name: name ids already end in a
        newline-holding token (as list printed them), so no extra newline before the closer."""
        t = [ST.Q_TCO, ST.Q_NL, ST.Q_LT, ST.Q_FUNCTION]
        t += [ST.OP_JOINED[op]] if op in ST.OP_JOINED else [ST.Q_EQ, ST.OP_BARE[op]]
        t += [ST.Q_GT, ST.Q_NL]
        for p, val in (("name", name), ("text", text)):
            if val is None: continue
            t += [ST.Q_LT, ST.Q_PARAMETER, ST.PARAM_JOINED[p], ST.Q_GT, ST.Q_NL, *val]
            if not (p == "name" and raw_name): t += [ST.Q_NL]
            t += [ST.Q_LTSLASH, ST.Q_PARAMETER, ST.Q_GT, ST.Q_NL]
        return t + [ST.Q_LTSLASH, ST.Q_FUNCTION, ST.Q_GT, ST.Q_NL, ST.Q_TCC]
    nl = {q for q in range(len(chat.tok)) if q < 248044 and "\n" in chat.tok.decode([q])}
    _D.append(dict(chat=chat, CH=CH, ST=ST, run=run, prompt=prompt, drive=drive, call=call, nl=nl))
    return _D[0]

def body(seg):
    """Printed text ids of a response (chip markers dropped, the header newline dropped)."""
    ids = [t for t in seg if t is not None]
    return ids[1:] if ids and ids[0] == 198 else ids

def readback(snap, label):
    """list, then get every listed name, all through the store. -> files [{name_ids, ids, found}]"""
    D = setup(); chat, drive, call = D["chat"], D["drive"], D["call"]
    ids = D["prompt"]("List the names in your memory.")
    t0 = time.time()
    _, _, resp, cf = drive(ids, snap, [call("list")])
    seg = resp[0][1:] if resp[0] and resp[0][0] == 198 else resp[0]
    names, cur = [], None
    for t in seg:  # each name follows its SL record
        if t is None:
            if cur: names.append(cur)
            cur = []
        elif cur is not None: cur.append(t)
    if cur: names.append(cur)
    files = []; conflicts = cf; BATCH = 40
    def variant(n, alt):
        raw = n[-1] in D["nl"] and len(n) <= 6
        if alt: raw = not raw
        return call("get", n, raw_name=raw)
    todo = [(i, False) for i in range(len(names))]; res = {}
    for attempt in (0, 1):
        for a in range(0, len(todo), BATCH):
            part = todo[a:a + BATCH]
            _, _, resp, cf = drive(ids, snap, [variant(names[i], alt) for i, alt in part]); conflicts += cf
            assert len(resp) == len(part), (len(resp), len(part))
            for (i, alt), r in zip(part, resp):
                b = body(r); nf = chat.tok.decode(b).strip() == "not found"
                if not nf or attempt == 1: res[i] = dict(found=not nf, ids=b)
        todo = [(i, True) for i in range(len(names)) if i in res and not res[i]["found"] or i not in res]
        todo = [(i, True) for i, _ in todo if i not in res or not res[i]["found"]]
        if not todo: break
    for i, n in enumerate(names):
        r = res[i]; files.append(dict(name_ids=n, name=chat.tok.decode(n).strip(), ids=r["ids"],
                                      text=chat.tok.decode(r["ids"]), found=r["found"]))
    print(label, "names", len(names), "found", sum(f["found"] for f in files), "conflicts", conflicts,
          "s", round(time.time() - t0, 1), flush=True)
    return files, conflicts, time.time() - t0

def cmd_readback_a4(ds):
    insts, _ = B.load(ds); out = {}
    for i in insts:
        snap = B.CH_load(B.snap_path(ds, i["id"], i["sessions"][-1]["n"])) if hasattr(B, "CH_load") else None
        D = setup(); snap = D["CH"].load_store_file(B.snap_path(ds, i["id"], i["sessions"][-1]["n"]))
        files, cf, sec = readback(snap, f"B1 {ds} {i['id']}")
        out[i["id"]] = dict(files=files, conflicts=cf, seconds=sec)
    json.dump(out, open(f"{OUT}/{ds}_B1_files.json", "w"))

def cmd_write_b2(ds):
    D = setup(); chat, drive, call, CH = D["chat"], D["drive"], D["call"], D["CH"]
    insts, _ = B.load(ds); log = {}
    for i in insts:
        ents = plan_entries(i, chat.tok); snap = None; recs = []
        for s in i["sessions"]:
            es = [e for e in ents if e["session"] == s["n"]]
            calls = []
            for e in es:
                for j, p in enumerate(e["pieces"]):
                    calls.append(call("put" if j == 0 else "append", e["name_ids"], p))
            ids = D["prompt"](B.write_prompt(i["who"], s, "store")); t0 = time.time()
            gen, snap, resp, cf = drive(ids, snap, calls)
            CH.save_store_file(f"{STORE}/{ds}_{i['id']}_s{s['n']}.npz", snap)
            answers = [chat.tok.decode(body(r)).strip() for r in resp]
            recs.append(dict(n=s["n"], calls=len(calls), answers=Counter(answers), conflicts=cf,
                             positions=len(gen), seconds=time.time() - t0))
            print("write", ds, i["id"], s["n"], len(calls), dict(Counter(answers)), "cf", cf, flush=True)
        log[i["id"]] = dict(entries=[dict(name=e["name"], name_ids=e["name_ids"], pieces=e["pieces"],
                                          session=e["session"], text=e["text"]) for e in ents],
                            sessions=recs, writes=sum(len(e["pieces"]) for e in ents),
                            tokens_stored=sum(len(p) + 1 for e in ents for p in e["pieces"]))
    json.dump(log, open(f"{OUT}/{ds}_B2{'snake' if SNAKE else ''}_write.json", "w"))

def cmd_readback_b2(ds):
    D = setup(); insts, _ = B.load(ds); out = {}
    for i in insts:
        snap = D["CH"].load_store_file(f"{STORE}/{ds}_{i['id']}_s{i['sessions'][-1]['n']}.npz")
        files, cf, sec = readback(snap, f"B3 {ds} {i['id']}")
        out[i["id"]] = dict(files=files, conflicts=cf, seconds=sec)
    json.dump(out, open(f"{OUT}/{ds}_B3_files.json", "w"))

def cmd_answer(arm, ds):
    """B1/B3: bge-small top-5 over the files read back from the store; base Qwen3.5-4B answers (A2's code path)."""
    import torch
    from transformers import AutoTokenizer, AutoModelForCausalLM, AutoModel
    torch.manual_seed(B.SEED)
    insts, qs = B.load(ds)
    if ds == "locomo":
        sub = {json.loads(l)["qid"] for l in open(f"{B.OUT}/locomo_A4.jsonl")}; qs = [q for q in qs if q["qid"] in sub]
    files = json.load(open(f"{OUT}/{ds}_{arm}_files.json"))
    tok = AutoTokenizer.from_pretrained(B.BASE)
    model = AutoModelForCausalLM.from_pretrained(B.BASE, dtype=torch.bfloat16).cuda().eval()
    btok = AutoTokenizer.from_pretrained(B.BGE); bm = AutoModel.from_pretrained(B.BGE).cuda().eval()
    @torch.no_grad()
    def emb(texts):
        o = []
        for a in range(0, len(texts), 64):
            b = btok(texts[a:a + 64], padding=True, truncation=True, max_length=512, return_tensors="pt").to("cuda")
            o.append(torch.nn.functional.normalize(bm(**b).last_hidden_state[:, 0], dim=-1))
        return torch.cat(o)
    def chat_ids(user):
        o = tok.apply_chat_template([{"role": "user", "content": user}], add_generation_prompt=True, enable_thinking=False, tokenize=True)
        return o if isinstance(o, list) else o["input_ids"]
    docs = {}
    for iid, v in files.items():
        fs = [f for f in v["files"] if f["found"]]
        # B1: a model-written file is shown as its name then its text (the name is its only label);
        # B3: the engine's text already starts with its [Session n, date] header, shown as stored
        txt = [(f"[{f['name']}]\n" + f["text"].rstrip("\n")) if arm == "B1" else f["text"].rstrip("\n") for f in fs]
        docs[iid] = (txt, emb(txt))
    path = f"{OUT}/{ds}_{arm}.jsonl"; done = B.done_ids(path)
    with open(path, "a") as f:
        for q in qs:
            if q["qid"] in done: continue
            t0 = time.time(); cs, E = docs[q["inst"]]
            qv = emb(["Represent this sentence for searching relevant passages: " + q["question"]])[0]
            top = torch.topk(E @ qv, min(RAG_K, len(cs))).indices.tolist()
            user = "Relevant excerpts from earlier conversations:\n" + "\n\n".join(cs[t] for t in top) + f"\n\n{q['question']}\n{B.INSTR[ds]}"
            ids = chat_ids(user); x = torch.tensor([ids], device="cuda")
            y = model.generate(x, max_new_tokens=64, do_sample=False, attention_mask=torch.ones_like(x))
            pred = tok.decode(y[0, len(ids):], skip_special_tokens=True).strip()
            rec = dict(q, arm=arm, pred=pred, prompt_tokens=len(ids), seconds=time.time() - t0, retrieved=top,
                       retrieved_text=[cs[t] for t in top])
            f.write(json.dumps(rec) + "\n"); f.flush()
            print(arm, q["qid"], repr(pred[:60]), "|", q["answer"][:40], flush=True)

def cmd_ask_b2(ds, budget, max_items):
    """B2: A4's question prompt and tools, Qwen deciding (bench.run), on the engine-written store."""
    D = setup(); chat, run, prompt, CH = D["chat"], D["run"], D["prompt"], D["CH"]
    insts, qs = B.load(ds)
    if ds == "locomo":
        sub = {json.loads(l)["qid"] for l in open(f"{B.OUT}/locomo_A4.jsonl")}; qs = [q for q in qs if q["qid"] in sub]
    arm = f"B2{'snake' if SNAKE else ''}b{budget}"; path = f"{OUT}/{ds}_{arm}.jsonl"; done = B.done_ids(path)
    stride = int(os.environ.get("STRIDE", "1"))  # GPU-time cut: every STRIDE-th question in qid order
    todo = [q for q in qs[::stride] if q["qid"] not in done][:max_items]
    snaps = {i["id"]: CH.load_store_file(f"{STORE}/{ds}_{i['id']}_s{i['sessions'][-1]['n']}.npz") for i in insts}
    with open(path, "a") as f:
        for q in todo:
            t0 = time.time()
            ids = prompt(f"{q['question']}\n{B.INSTR[ds]}\nYou may look in your memory with list and get.")
            gen, _, n_free = run(ids, snaps[q["inst"]], budget)
            text = chat.text_of(gen)
            pred = text.split("</think>")[-1].replace("<|im_end|>", "").strip()
            rec = dict(q, arm=arm, pred=pred, prompt_tokens=len(ids), context_tokens=len(ids) + len(gen),
                       seconds=time.time() - t0, text=text, qwen_free=n_free, budget=budget)
            f.write(json.dumps(rec) + "\n"); f.flush()
            print(arm, q["qid"], round(rec["seconds"], 1), repr(pred[:60]), "|", q["answer"][:40], flush=True)

def cmd_plan(ds):
    from transformers import AutoTokenizer
    tok = AutoTokenizer.from_pretrained(B.FS)
    for i in B.load(ds)[0]:
        e = plan_entries(i, tok)
        print(ds, i["id"], "entries", len(e), "writes", sum(len(x["pieces"]) for x in e),
              "tokens", sum(len(p) + 1 for x in e for p in x["pieces"]), "eg", [x["name"] for x in e[:3]])

if __name__ == "__main__":
    c = sys.argv[1]
    if c == "plan": cmd_plan(sys.argv[2])
    elif c == "readback-a4": cmd_readback_a4(sys.argv[2])
    elif c == "write-b2": cmd_write_b2(sys.argv[2])
    elif c == "readback-b2": cmd_readback_b2(sys.argv[2])
    elif c == "answer": cmd_answer(sys.argv[2], sys.argv[3])
    elif c == "ask-b2": cmd_ask_b2(sys.argv[2], int(sys.argv[3]), int(sys.argv[4]) if len(sys.argv) > 4 else 10**9)
