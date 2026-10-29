"""Qwen deciding, with the store full (out/full_small.npz loaded): what does the model do when put answers 'full'?"""
import sys, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from repro import *
chat = build_chat()
snap = CH.load_store_file(str(OUT / "full_small.npz"))
res = {}
for lab, ask in [("save_new", 'Please remember that my dog is named Biscuit. Save it with put under the name "dog".'),
                 ("update_existing", 'Use put to replace the text of the file "f0" with "updated note".')]:
    ids = prompt(chat, ask)
    st, cc, steps = forced(chat, ids, [(lab, [])], snap=snap, free_max=400)
    gen = [s[1] for s in steps]
    res[lab] = {"text": chat.text_of(gen), "chip_trace": [S.name(t - chat.v0) for t in gen if t >= chat.v0]}
    P(lab, json.dumps(res[lab], indent=1))
json.dump(res, open(OUT / "behave_full.json", "w"), indent=1)
