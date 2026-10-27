"""subset.py in.vcad out.vcad 'drop,list' | --keep 'keep,list' : copy of a doc without (or with only) the named instances (prefix* allowed)."""
import json, sys
d = json.load(open(sys.argv[1])); keep = sys.argv[3] == '--keep'; names = sys.argv[4 if keep else 3].split(',')
m = lambda i: any(i == n or (n.endswith('*') and i.startswith(n[:-1])) for n in names)
ok = (lambda i: m(i) or i == 'frame') if keep else (lambda i: not m(i))
d['instances'] = [i for i in d['instances'] if ok(i['id'])]
d['joints'] = [j for j in d['joints'] if ok(j['childInstanceId']) and ok(j['parentInstanceId'])]
if '--hide-frame' in sys.argv:   # ground must stay: shrink it to a 1 mm cube instead
    pd = d['partDefs']['frame']; d['nodes']['99999'] = {'id': 99999, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 1.0, 'y': 1.0, 'z': 1.0}}}; pd['root'] = 99999
json.dump(d, open(sys.argv[2], 'w'))
