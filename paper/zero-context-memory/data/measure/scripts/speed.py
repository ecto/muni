"""Prefill/decode throughput and peak memory; recurrent-state bytes. usage: speed.py base_hf|fs_hf|fs_rvchat"""
import sys, os, json, time, gc
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import torch, numpy as np
mode = sys.argv[1]; NP, NG, RUNS = 512, 256, 4  # run 0 = warmup
BASE, FS = os.environ["BASE"], os.environ["FS"]
text = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "text.txt")).read() * 2
OUT = os.path.expanduser("~/gpsm-measure/out")
def sync(): torch.cuda.synchronize()
def state_bytes(cache, layers):
    rec = conv = kv = 0
    for L in layers:
        lay = cache.layers[L]
        if hasattr(lay, "recurrent_states") and lay.recurrent_states[0] is not None:
            rec += lay.recurrent_states[0].numel() * lay.recurrent_states[0].element_size()
            conv += lay.conv_states[0].numel() * lay.conv_states[0].element_size()
        elif getattr(lay, "keys", None) is not None and torch.is_tensor(lay.keys):
            kv += lay.keys.numel() * lay.keys.element_size() + lay.values.numel() * lay.values.element_size()
    return {"recurrent_bytes": rec, "conv_bytes": conv, "kv_bytes": kv}
res = {"mode": mode, "prompt_tokens": NP, "gen_tokens": NG, "runs": RUNS - 1, "gpu": torch.cuda.get_device_name(),
       "torch": torch.__version__}
import transformers; res["transformers"] = transformers.__version__
if mode in ("base_hf", "fs_hf"):
    from transformers import AutoModelForCausalLM, AutoTokenizer
    path = BASE if mode == "base_hf" else FS
    tok = AutoTokenizer.from_pretrained(BASE)
    if mode == "base_hf":
        from transformers import AutoModelForImageTextToText
        model = AutoModelForImageTextToText.from_pretrained(path, dtype=torch.bfloat16).to("cuda").eval()
    else:
        from transformers import Qwen3_5ForCausalLM
        model = Qwen3_5ForCausalLM.from_pretrained(path, dtype=torch.bfloat16).to("cuda").eval()
    ids = tok(text).input_ids[:NP]
    res["weights_bytes_gpu"] = sum(p.numel() * p.element_size() for p in model.parameters())
    sync(); res["mem_after_load"] = torch.cuda.memory_allocated(); torch.cuda.reset_peak_memory_stats()
    pre, dec = [], []
    with torch.no_grad():
        for r in range(RUNS):
            sync(); t0 = time.time()
            out = model(input_ids=torch.tensor([ids], device="cuda"), use_cache=True, logits_to_keep=1)
            sync(); tp = time.time() - t0
            cache = out.past_key_values; nxt = int(out.logits[0, -1].argmax()); gen = []
            sync(); t0 = time.time()
            for k in range(NG):
                o = model(input_ids=torch.tensor([[nxt]], device="cuda"), past_key_values=cache, use_cache=True)
                nxt = int(o.logits[0, -1].argmax()); gen.append(nxt)
            sync(); td = time.time() - t0
            if r: pre.append(NP / tp); dec.append(NG / td)
        nl = len(model.config.layer_types) if hasattr(model.config, "layer_types") else model.config.text_config.num_hidden_layers
        lt = getattr(model.config, "layer_types", None) or model.config.text_config.layer_types
        res["layer_types_count"] = {t: lt.count(t) for t in set(lt)}
        res["state_after_768_tokens"] = state_bytes(cache, range(len(lt)))
    res["sample"] = tok.decode(gen[:40])
else:
    from repro import build_chat, machine, prompt, core, hf, CH
    chat = build_chat(); tok = chat.tok
    ids = tok(text).input_ids[:NP]
    res["weights_bytes_gpu"] = sum(p.numel() * p.element_size() for p in chat.model.parameters())
    sync(); res["mem_after_load"] = torch.cuda.memory_allocated(); torch.cuda.reset_peak_memory_stats()
    pre, dec, fired = [], [], 0
    with torch.no_grad():
        for r in range(RUNS):
            sync(); t0 = time.time()
            out = chat._prefill(ids, chunk=512); cache = out.past_key_values
            core.snapshot(chat.c, machine()); chat.load_store(None); cc = chat._chip_cache()
            for a in range(0, len(ids), 128):
                h = chat.cm.embed_tokens(torch.tensor([ids[a:a+128]], device=chat.dev))
                for L in range(chat.n_chip):
                    h = chat.cm.layers[L](h, position_embeddings=None, attention_mask=None, position_ids=None, past_key_values=cc)
            hf.inject(chat.c, cc)
            if chat.graph is None: chat._capture()
            sync(); tp = time.time() - t0
            nxt = int(out.logits[0, -1][: chat.v0].argmax()); gen = []; g = pos = NP
            sync(); t0 = time.time()
            for k in range(NG):
                hc = chat._chip_step(nxt, g, cc); g += 1
                ct = chat.circuit_token(hc)
                lg = chat._qwen_step(hc, cache, pos); pos += 1
                if ct is not None: fired += 1
                nxt = ct if ct is not None else int(lg[: chat.v0].argmax()); gen.append(nxt)
            sync(); td = time.time() - t0
            if r: pre.append(NP / tp); dec.append(NG / td)
        lt = chat.model.config.layer_types
        res["layer_types_count"] = {t: lt.count(t) for t in set(lt)}
        res["state_after_768_tokens_qwen_cache"] = state_bytes(cache, range(len(lt)))
        res["state_chip_cache"] = state_bytes(cc, range(chat.n_chip))
        res["circuit_fired"] = fired
        r_ = chat.c.target.value_heads // chat.c.target.key_heads
        res["store_heads_value_heads"] = sum(len(n.vheads()) for n in chat.c.store.persistent.values())
        res["store_heads_dense_fp32_bytes"] = res["store_heads_value_heads"] * 128 * 128 * 4
    res["sample"] = tok.decode([t for t in gen[:40] if t < chat.v0])
res["prefill_tok_s"] = pre; res["decode_tok_s"] = dec
res["prefill_mean"] = float(np.mean(pre)); res["prefill_std"] = float(np.std(pre, ddof=1))
res["decode_mean"] = float(np.mean(dec)); res["decode_std"] = float(np.std(dec, ddof=1))
res["peak_mem_allocated"] = torch.cuda.max_memory_allocated(); res["peak_mem_reserved"] = torch.cuda.max_memory_reserved()
print(json.dumps(res, indent=1))
json.dump(res, open(f"{OUT}/speed_{mode}.json", "w"), indent=1)
