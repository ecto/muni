"""Synthetic exact-facts probe: 60 facts, 5 sessions x 12, seeded."""
import json, random, sys, uuid
R = random.Random(20261028)
projects = ["orchid", "basalt", "tundra", "kestrel", "lumen", "quartz", "ferrous", "nimbus", "saffron", "harbor"]
words = ["cache", "queue", "router", "ledger", "billing", "auth", "ingest", "render", "search", "sync"]
def hexs(n): return "".join(R.choice("0123456789abcdef") for _ in range(n))
facts = []
for i, p in enumerate(projects):
    w = words[i]
    u = str(uuid.UUID(int=R.getrandbits(128), version=4))
    facts.append(dict(kind="uuid", answer=u, say=f"the tenant ID for the {p} account is {u}",
                      ask=f"What is the tenant ID for the {p} account?"))
    path = f"/srv/{p}/{R.choice(['etc','conf','data','var'])}/{w}_{R.randint(10,99)}.{R.choice(['toml','yaml','json','ini'])}"
    facts.append(dict(kind="path", answer=path, say=f"the {p} {w} config file lives at {path}",
                      ask=f"What is the path of the {p} {w} config file?"))
    port = str(R.randint(1025, 65000))
    facts.append(dict(kind="port", answer=port, say=f"the {p} {w} service listens on port {port}",
                      ask=f"Which port does the {p} {w} service listen on?"))
    key = f"{w}.{R.choice(['max_retries','pool_size','timeout_ms','batch_size','shard_count'])}"
    val = str(R.choice([R.randint(2, 999), R.randint(1000, 90000)]))
    facts.append(dict(kind="config", answer=f"{key}={val}", say=f"in {p} we set {key}={val}",
                      ask=f"What config key=value setting did we use for {w} in {p}? Give it as key=value."))
    fn = f"{w}_{R.choice(['flush','merge','resolve','apply','encode'])}"
    a1, a2 = R.choice(["ctx", "buf", "req", "cfg"]), R.choice(["limit", "depth", "offset", "level"])
    ty = R.choice(["u32", "usize", "i64", "u16"])
    sig = f"fn {fn}({a1}: &mut {a1.capitalize()}, {a2}: {ty}) -> Result<(), Error>"
    facts.append(dict(kind="signature", answer=sig, say=f"the {p} {w} entry point has the signature {sig}",
                      ask=f"What is the exact signature of the {p} {w} entry point?"))
    h = hexs(10)
    facts.append(dict(kind="hash", answer=h, say=f"the last good {p} build was commit {h}",
                      ask=f"What is the commit hash of the last good {p} build?"))
R.shuffle(facts)
filler_u = ["Morning! Coffee machine is broken again.", "Did you see the game last night?", "I'm heading out for lunch soon.",
            "The standup ran long today.", "My cat knocked my monitor over this morning.", "Weather is gross, rain all week.",
            "Thinking about getting a new keyboard.", "Reminder that Friday is a half day.", "Ugh, so many meetings today."]
filler_a = ["Ha, that's rough.", "Sounds good.", "Oh no!", "Nice.", "Got it.", "Fair enough.", "Haha, classic."]
sessions = []
dates = ["2 March 2026", "9 March 2026", "17 March 2026", "24 March 2026", "1 April 2026"]
for s in range(5):
    turns = []
    for k, f in enumerate(facts[s*12:(s+1)*12]):
        f["session"] = s + 1
        if R.random() < 0.7:
            turns.append(("User", R.choice(filler_u))); turns.append(("Assistant", R.choice(filler_a)))
        turns.append(("User", f"Quick note for later: {f['say']}."))
        turns.append(("Assistant", "Noted."))
    turns.append(("User", "Ok, that's it for today, thanks!")); turns.append(("Assistant", "Anytime, see you."))
    sessions.append(dict(date=dates[s], turns=turns))
for i, f in enumerate(facts): f["id"] = i
json.dump(dict(seed=20261028, facts=facts, sessions=sessions), open(sys.argv[1], "w"), indent=1)
print(len(facts))
