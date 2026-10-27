"""Labels-only doc for the aisle: framing parts stay full (floor, fences, curtains, light rail) so the bbox/framing match
the full render; every tagged instance (aisle_tags.json) shrinks to a 2 mm cube at its tag point; the rest is dropped.
BAYS=1 also labels the 24 bay floors (top view); datum/coupler parts are never labelled (96 tags = unreadable)."""
import json, sys, os
S = os.path.dirname(os.path.abspath(__file__))
d = json.load(open(sys.argv[1])); tags = json.load(open(S + '/aisle_tags.json'))
FULL = {'floor', 'fence_front', 'fence_back', 'curtain_in', 'curtain_out', 'light_rail'}
bays = os.environ.get('BAYS')
MAIN = {'saw', 'sheet_router', 'mill', 'reflow', 'tray_rack', 'winder', 'stock_drawers', 'dock_in', 'dock_out', 'assembly_table',
        'finished_rack', 'bench', 'tender_1', 'tender_2', 'human', 'curtain_in', 'curtain_out', 'spine_busbar', 'light_rail'}
def keep(i):
    if i in FULL: return True
    if i not in tags: return False
    if i.startswith('bay_'): return bool(bays) and i.count('_') == 1
    return i in MAIN
LAB = MAIN | {'light_rail', 'fence_front', 'fence_back', 'curtain_in', 'curtain_out'} | ({i for i in tags if i.startswith('bay_') and i.count('_') == 1} if bays else set())
d['instances'] = [i for i in d['instances'] if keep(i['id'])]
d['joints'] = [j for j in d['joints'] if keep(j['childInstanceId'])]
for k, inst in enumerate(d['instances']):
    f = inst['id']
    if f in FULL: continue
    p = tags[f]; pd = json.loads(json.dumps(d['partDefs'][inst['partDefId']])); nid = 90000 + k
    d['nodes'][str(nid)] = {'id': nid, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 2.0, 'y': 2.0, 'z': 2.0}}}
    pd['root'] = nid; key = 'lab_%d' % k; d['partDefs'][key] = pd; inst['partDefId'] = key
    j = next(j for j in d['joints'] if j['childInstanceId'] == f)
    a = j['childAnchor']; j['childAnchor'] = {'x': a['x'] + 1 - p[0], 'y': a['y'] + 1 - p[1], 'z': a['z'] - p[2]}
json.dump(d, open(sys.argv[2], 'w'))
