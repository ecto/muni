# Labels-only doc for cell_v2: drop arm links, frame, deck, floor and panels; shrink every other named instance
# (the frame stays: it sets the auto-framing, so the label layer lines up with the full render) to a 2 mm tag at a chosen point (default: top centre of its box) so the label leader starts there.
import json, sys
sys.path.insert(0, __file__.rsplit('/', 1)[0])
from cellp import *
d = json.load(open(sys.argv[1]))
# electronics sit under the deck (hidden in iso/top): labelled in cell_v2_bay.png instead
import os
arms = lambda i: i.startswith(('l_', 'r_', 'panel'))
drop = (lambda i: i in ('deck', 'floor') or arms(i)) if os.environ.get('BAY') else (lambda i: i in ('deck', 'floor', 'psu', 'canhub', 'controller') or arms(i))
d['instances'] = [i for i in d['instances'] if not drop(i['id'])]
d['joints'] = [j for j in d['joints'] if not drop(j['childInstanceId'])]
nodes = d['nodes']
# mill pokes 50 mm out of the back of the frame: tag its far top edge so the label doc keeps the same bbox (= framing)
TAG = {'mill': (200, 349, 380), 'plinth': (0, 0, plinth_h), 'rail_f': (60, 20, 33), 'rail_b': (rail_l - 60, 20, 33), 'chain': (420, aisle_y, deck_z + 28),
       'light': (1150, 15, 0), 'winder': (100, 0, 0)}
for k, inst in enumerate(d['instances']):
    f = inst['id']; pd = d['partDefs'][inst['partDefId']]
    op = nodes[str(pd['root'])]['op']
    if op['type'] == 'Translate': op = nodes[str(op['child'])]['op']     # gen wraps boxes in [translate 0 0 0 ...]
    if f == 'frame': continue
    if f in TAG: p = TAG[f]
    elif op['type'] == 'Cube': c = op['size']; p = (c['x'] / 2, c['y'] / 2, c['z'])
    else: continue
    nid = 90000 + k
    nodes[str(nid)] = {'id': nid, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 2.0, 'y': 2.0, 'z': 2.0}}}
    pd['root'] = nid
    j = next(j for j in d['joints'] if j['childInstanceId'] == f)
    a = j['childAnchor']; j['childAnchor'] = {'x': a['x'] + 1 - p[0], 'y': a['y'] + 1 - p[1], 'z': a['z'] - p[2]}
json.dump(d, open(sys.argv[2], 'w'))
