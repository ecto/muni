"""subset_doc.py IN OUT X0 X1 Y0 Y1 [drop,names]: keep floor-mounted subtrees whose plan centre lies in the box (framing control
for close-up stills; vcad-render --focus matched rose sub-part names instead of our instances). Floor stays (cropped by section)."""
import json, sys, os
S = os.path.dirname(os.path.abspath(__file__)); tags = json.load(open(S + '/aisle_tags.json'))
d = json.load(open(sys.argv[1])); x0, x1, y0, y1 = map(float, sys.argv[3:7]); drop = set(sys.argv[7].split(',')) if len(sys.argv) > 7 else set()
par = {j['childInstanceId']: j for j in d['joints']}
def root(i):
    while par[i]['parentInstanceId'] != 'floor': i = par[i]['parentInstanceId']
    return i
def centre(r):
    j = par[r]; a = j['parentAnchor']; t = tags.get(r, (0, 0, 0)); x, y = a['x'] + t[0], a['y'] + t[1]
    if j['kind']['type'] == 'Slider': x += j['kind']['axis']['x'] * j['state']; y += j['kind']['axis']['y'] * j['state']
    if r.startswith('bench') and r not in tags: x, y = 1750, 500
    if r == 'bench_plinth': x, y = 1750, 500
    return x, y
def keep(i):
    if i == 'floor': return True
    r = root(i)
    if r in drop: return False
    x, y = centre(r); return x0 <= x <= x1 and y0 <= y <= y1
d['instances'] = [i for i in d['instances'] if keep(i['id'])]
d['joints'] = [j for j in d['joints'] if keep(j['childInstanceId'])]
ids = {i['id'] for i in d['instances']}
# floor: replace by a local patch so the frame is tight
fl = next(i for i in d['instances'] if i['id'] == 'floor'); pd = d['partDefs'][fl['partDefId']]
nid = 99999; d['nodes'][str(nid)] = {'id': nid, 'name': None, 'op': {'type': 'Translate', 'offset': {'x': x0, 'y': y0, 'z': -300.0}, 'child': nid - 1}}
d['nodes'][str(nid - 1)] = {'id': nid - 1, 'name': None, 'op': {'type': 'Cube', 'size': {'x': x1 - x0, 'y': y1 - y0, 'z': 320.0}}}
pd['root'] = nid
json.dump(d, open(sys.argv[2], 'w')); print(len(ids), 'instances kept', file=sys.stderr)
