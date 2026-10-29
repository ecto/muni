"""GPSM memory benchmark. Our own code; the qwenputer repo is imported read-only for the GPSM arm.
usage: bench.py base ARM DS [max_items] | gpsm-write DS INST | gpsm-ask DS [max_items] | gpsm-list DS INST"""
import os, sys, json, time, random, re
sys.path.append("/home/cam/gpsm-bench/pylib")
ROOT = "/home/cam/gpsm-bench"; OUT = f"{ROOT}/out"; DATA = f"{ROOT}/data"
BASE = "/home/cam/gpsm-models/Qwen3.5-4B"; FS = "/home/cam/gpsm-models/Qwen3.5-4B-Qwenputer-Filesystem"
BGE = "/home/cam/gpsm-models/bge-small-en-v1.5"
A1_BUDGET = 16384; RAG_K = 5; RAG_TURNS = 6; SEED = 0
N_CONV = 3; N_Q = 150
INSTR = {"locomo": "Answer the question with a short phrase, using exact words from the conversation whenever possible. If the question asks when, give the date.",
         "facts": "Reply with the exact value only."}
os.makedirs(OUT, exist_ok=True)

def fmt_turn(t):
    s = f"{t['speaker']}: {t['text']}"
    if t.get("blip_caption"): s += f" [shares a photo: {t['blip_caption']}]"
    return s

def load(ds):
    """-> list of instances {id, who, sessions:[{n,date,turns}]}, questions [{qid,inst,question,answer,category}]"""
    if ds == "facts":
        d = json.load(open(f"{DATA}/facts.json"))
        ss = [{"n": i + 1, "date": s["date"], "turns": [{"speaker": a, "text": b} for a, b in s["turns"]]} for i, s in enumerate(d["sessions"])]
        inst = [{"id": "facts", "who": "the user and the assistant", "sessions": ss}]
        qs = [{"qid": f"f{f['id']}", "inst": "facts", "question": f["ask"], "answer": f["answer"], "category": f["kind"]} for f in d["facts"]]
        return inst, qs
    d = json.load(open(f"{DATA}/locomo10.json"))
    R = random.Random(SEED); chosen = sorted(R.sample(range(len(d)), N_CONV))
    inst, qs = [], []
    for i in chosen:
        x = d[i]; c = x["conversation"]
        ns = sorted(int(k.split("_")[1]) for k in c if re.fullmatch(r"session_\d+", k))
        ss = [{"n": n, "date": c[f"session_{n}_date_time"], "turns": c[f"session_{n}"]} for n in ns]
        inst.append({"id": x["sample_id"], "who": f"{c['speaker_a']} and {c['speaker_b']}", "sessions": ss})
        for j, q in enumerate(x["qa"]):
            if q["category"] == 5: continue
            qs.append({"qid": f"{x['sample_id']}-{j}", "inst": x["sample_id"], "question": q["question"],
                       "answer": str(q["answer"]), "category": q["category"]})
    R2 = random.Random(SEED + 1); qs = sorted(R2.sample(qs, min(N_Q, len(qs))), key=lambda q: q["qid"])
    return inst, qs

def session_text(s):
    return f"[Session {s['n']}, {s['date']}]\n" + "\n".join(fmt_turn(t) for t in s["turns"])

def done_ids(path):
    if not os.path.exists(path): return set()
    return {json.loads(l)["qid"] for l in open(path)}

def write_prompt(who, s, mode):
    head = (f"Below is a conversation between {who} (session {s['n']}, {s['date']}). After this you will not see it "
            f"again: later you will be asked questions about it and will have only your ")
    if mode == "notes":
        tail = ("notes. Write notes on everything you might need later: short bullet points that keep names, "
                "dates and exact values. At most 200 words.")
        return head + tail + "\n\n" + session_text(s)
    tail = ("memory. Save everything you might need later with put: one short, distinct name per fact (names are "
            "told apart by their first few words) and a short text that keeps names, dates and exact values. "
            "When you are done, reply done.")
    return head + tail + "\n\n" + session_text(s)

# ---------------------------------------------------------------- base arms
def run_base(arm, ds, max_items):
    import torch
    from transformers import AutoTokenizer, AutoModelForCausalLM
    torch.manual_seed(SEED)
    inst, qs = load(ds); byid = {i["id"]: i for i in inst}
    DRY = os.environ.get("GPSM_DRY"); dev = "cpu" if DRY else "cuda"
    path = f"{OUT}/{ds}_{arm}.jsonl" + (".dry" if DRY else ""); done = done_ids(path)
    tok = AutoTokenizer.from_pretrained(BASE)
    model = None if DRY else AutoModelForCausalLM.from_pretrained(BASE, dtype=torch.bfloat16).cuda().eval()
    def chat_ids(user):
        out = tok.apply_chat_template([{"role": "user", "content": user}], add_generation_prompt=True,
                                      enable_thinking=False, tokenize=True)
        return out if isinstance(out, list) else out["input_ids"]
    @torch.no_grad()
    def gen(ids, n):
        if DRY: return "dry"
        x = torch.tensor([ids], device="cuda")
        y = model.generate(x, max_new_tokens=n, do_sample=False, attention_mask=torch.ones_like(x))
        return tok.decode(y[0, len(ids):], skip_special_tokens=True).strip()
    ctx = {}
    if arm == "A3":  # notes, written once per instance after each session (cached)
        npath = f"{OUT}/{ds}_A3_notes.json" + (".dry" if DRY else "")
        notes = json.load(open(npath)) if os.path.exists(npath) else {}
        for i in inst:
            if i["id"] in notes and len(notes[i["id"]]) == len(i["sessions"]): continue
            acc = notes.get(i["id"], [])
            for s in i["sessions"][len(acc):]:
                t0 = time.time(); ids = chat_ids(write_prompt(i["who"], s, "notes"))
                acc.append({"n": s["n"], "date": s["date"], "notes": gen(ids, 400), "prompt_tokens": len(ids),
                            "seconds": time.time() - t0})
                notes[i["id"]] = acc; json.dump(notes, open(npath, "w"), indent=1)
                print("notes", i["id"], s["n"], flush=True)
        for i in inst:
            ctx[i["id"]] = "\n\n".join(f"[Notes from session {a['n']}, {a['date']}]\n{a['notes']}" for a in notes[i["id"]])
    if arm == "A1":
        for i in inst:
            lines, n = [], 0
            for s in reversed(i["sessions"]):
                block = [f"[Session {s['n']}, {s['date']}]"] + [fmt_turn(t) for t in s["turns"]]
                stop = False
                for L in reversed(block[1:]):
                    k = len(tok.encode(L + "\n", add_special_tokens=False))
                    if n + k > A1_BUDGET: stop = True; break
                    lines.append(L); n += k
                lines.append(block[0]); n += 12
                if stop: break
            ctx[i["id"]] = "\n".join(reversed(lines))
    if arm == "A2":
        from transformers import AutoModel
        btok = AutoTokenizer.from_pretrained(BGE); bm = AutoModel.from_pretrained(BGE).to(dev).eval()
        @torch.no_grad()
        def emb(texts):
            out = []
            for a in range(0, len(texts), 64):
                b = btok(texts[a:a + 64], padding=True, truncation=True, max_length=512, return_tensors="pt").to(dev)
                v = bm(**b).last_hidden_state[:, 0]; out.append(torch.nn.functional.normalize(v, dim=-1))
            return torch.cat(out)
        chunks = {}
        for i in inst:
            cs = []
            for s in i["sessions"]:
                for a in range(0, len(s["turns"]), RAG_TURNS):
                    cs.append(f"[Session {s['n']}, {s['date']}]\n" + "\n".join(fmt_turn(t) for t in s["turns"][a:a + RAG_TURNS]))
            chunks[i["id"]] = (cs, emb(cs))
    n_run = 0
    with open(path, "a") as f:
        for q in qs:
            if q["qid"] in done or n_run >= max_items: continue
            t0 = time.time(); extra = {}
            if arm == "A0": user = f"{q['question']}\n{INSTR[ds]}"
            elif arm == "A1": user = f"Conversation history (the most recent part):\n{ctx[q['inst']]}\n\n{q['question']}\n{INSTR[ds]}"
            elif arm == "A3": user = f"Your notes from earlier conversations:\n{ctx[q['inst']]}\n\n{q['question']}\n{INSTR[ds]}"
            else:
                cs, E = chunks[q["inst"]]
                qv = emb(["Represent this sentence for searching relevant passages: " + q["question"]])[0]
                top = torch.topk(E @ qv, min(RAG_K, len(cs))).indices.tolist()
                extra["retrieved"] = top
                user = "Relevant excerpts from earlier conversations:\n" + "\n\n".join(cs[t] for t in top) + f"\n\n{q['question']}\n{INSTR[ds]}"
            ids = chat_ids(user); pred = gen(ids, 64)
            rec = dict(q, arm=arm, pred=pred, prompt_tokens=len(ids), seconds=time.time() - t0, **extra)
            f.write(json.dumps(rec) + "\n"); f.flush(); n_run += 1
            print(arm, q["qid"], repr(pred[:60]), "|", q["answer"][:40], flush=True)

# ---------------------------------------------------------------- GPSM arm (qwenputer filesystem checkpoint)
REPO = "/home/cam/qwenputer-repro/repo/experimental/riscv"
_SETUP = []
def gpsm_setup():
    if _SETUP: return _SETUP[0]
    _SETUP.append(_gpsm_setup()); return _SETUP[0]
def _gpsm_setup():
    sys.path.insert(0, REPO)
    import torch
    from chipformer.rv import chat as CH, core, elf, hf, ref, bridge as BR, spec as S, store as ST
    FW = REPO + "/tests/data/rv/calc-bridge.elf"
    chat = CH.RvChat(FS)
    def machine():
        m = ref.Machine.from_program(elf.program(FW)); BR.park(m); return m
    @torch.no_grad()
    def run(ids, snap, free_max):
        """Greedy, Qwen deciding; circuit-owned positions take the circuit's token (after repro.forced)."""
        out = chat._prefill(ids, chunk=1024); cache = out.past_key_values
        nxt = int(torch.argmax(out.logits[0, -1][: chat.v0]))
        core.snapshot(chat.c, machine()); chat.load_store(snap)
        cc = chat._chip_cache()
        for a in range(0, len(ids), 128):
            h = chat.cm.embed_tokens(torch.tensor([ids[a:a + 128]], device=chat.dev))
            for L in range(chat.n_chip):
                h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=cc)
        del h; torch.cuda.empty_cache()
        hf.inject(chat.c, cc)
        if chat.graph is None: chat._capture()
        g = pos = len(ids); gen = []; n_free = 0
        while True:
            gen.append(nxt)
            hc = chat._chip_step(nxt, g, cc); g += 1
            if nxt >= chat.v0:
                nxt = chat.circuit_token(hc); assert nxt is not None; continue
            logits = chat._qwen_step(hc, cache, pos); pos += 1
            ct = chat.circuit_token(hc)
            if ct is not None: nxt = ct; continue
            if n_free < free_max and nxt not in chat.eos:
                nxt = int(torch.argmax(logits[: chat.v0])); n_free += 1; continue
            break
        return gen, chat.save_store(("hf", cc)), n_free
    def prompt(user):
        return chat.prompt_ids([{"role": "system", "content": ST.MEM_SYSTEM}, {"role": "user", "content": user}],
                               (CH.CALC_TOOL, *ST.TOOLS))
    return chat, CH, S, run, prompt

def snap_path(ds, inst, n): return f"{OUT}/gpsm_store/{ds}_{inst}_s{n}.npz"

def gpsm_write(ds, inst_id):
    insts, _ = load(ds); i = [x for x in insts if x["id"] == inst_id][0]
    os.makedirs(f"{OUT}/gpsm_store", exist_ok=True)
    lpath = f"{OUT}/{ds}_A4_write.jsonl"
    have = set()
    if os.path.exists(lpath): have = {(r["inst"], r["n"]) for r in map(json.loads, open(lpath))}
    chat, CH, S, run, prompt = gpsm_setup()
    snap = None
    for s in i["sessions"]:
        if (inst_id, s["n"]) in have:
            snap = CH.load_store_file(snap_path(ds, inst_id, s["n"])); continue
        ids = prompt(write_prompt(i["who"], s, "store")); t0 = time.time()
        gen, snap, n_free = run(ids, snap, 1500 if ds == "facts" else 1000)
        CH.save_store_file(snap_path(ds, inst_id, s["n"]), snap)
        text = chat.text_of(gen)
        rec = dict(inst=inst_id, n=s["n"], prompt_tokens=len(ids), gen_positions=len(gen), qwen_free=n_free,
                   chip_tokens=sum(t >= chat.v0 for t in gen), seconds=time.time() - t0, text=text)
        open(lpath, "a").write(json.dumps(rec) + "\n")
        print("write", inst_id, s["n"], text.count("<function=put>"), "puts", round(rec["seconds"], 1), "s", flush=True)
    # authoritative listing of the final store
    ids = prompt("List the names in your memory."); gen, _, _ = run(ids, snap, 60)
    json.dump({"inst": inst_id, "text": chat.text_of(gen),
               "rows": {k: [len(v) for v in vs] for k, vs in snap.items()}},
              open(f"{OUT}/{ds}_A4_final_{inst_id}.json", "w"), indent=1)

def gpsm_ask(ds, max_items):
    insts, qs = load(ds)
    path = f"{OUT}/{ds}_A4.jsonl"; done = done_ids(path)
    cap = 125 if ds == "locomo" else len(qs)  # GPU-time cut: A4 runs the first 125 of the 150 (qid order)
    todo = [q for q in qs[:cap] if q["qid"] not in done][:max_items]
    if not todo: return
    chat, CH, S, run, prompt = gpsm_setup()
    snaps = {i["id"]: CH.load_store_file(snap_path(ds, i["id"], i["sessions"][-1]["n"])) for i in insts}
    with open(path, "a") as f:
        for q in todo:
            t0 = time.time()
            user = f"{q['question']}\n{INSTR[ds]}\nYou may look in your memory with list and get."
            ids = prompt(user); gen, _, n_free = run(ids, snaps[q["inst"]], 300)
            text = chat.text_of(gen)
            pred = text.split("</think>")[-1].replace("<|im_end|>", "").strip()
            rec = dict(q, arm="A4", pred=pred, prompt_tokens=len(ids), context_tokens=len(ids) + len(gen),
                       seconds=time.time() - t0, text=text, qwen_free=n_free)
            f.write(json.dumps(rec) + "\n"); f.flush()
            print("A4", q["qid"], repr(pred[:60]), "|", q["answer"][:40], flush=True)

if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "base": run_base(sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 10**9)
    elif cmd == "gpsm-write": gpsm_write(sys.argv[2], sys.argv[3])
    elif cmd == "gpsm-ask": gpsm_ask(sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 10**9)
    elif cmd == "gpsm-write-all":
        for i in load(sys.argv[2])[0]: gpsm_write(sys.argv[2], i["id"])
    elif cmd == "insts": print(" ".join(i["id"] for i in load(sys.argv[2])[0]))
