"""Labels-only doc: drop arm links / panels / deck / plate+leg parts, keep frame + table + human at full size (they
set the auto-framing, so the label layer lines up with the full render), shrink every other named instance to a
2 mm tag at the top centre of its bbox so the label leader starts there.  usage: label_doc.py in.vcad out.vcad"""
import json, sys, os
d = json.load(open(sys.argv[1])); nodes = d['nodes']
ORIG = json.loads(json.dumps(d))
KEEP = ('frame',)
HIDE = ('deck,bay1_plate,bay2_plate,bay3_plate,bench_legs,bay3_legs,mill_stand,foot_holes,psu,canhub,controller,chain,busbar,bay2_pins,bay3_pins,bay2_dock,bay3_dock,bay2_coupler,bay3_coupler,bay3_shelf,' + os.environ.get('HIDE', '')).split(',')
drop = lambda i: i.startswith(('l_', 'r_', 'panel')) or i in HIDE
d['instances'] = [i for i in d['instances'] if not drop(i['id'])]
d['joints'] = [j for j in d['joints'] if not drop(j['childInstanceId'])]
def bbox(nid):
    op = nodes[str(nid)]['op']; t = op['type']
    if t == 'Cube': s = op['size']; return [0, 0, 0, s['x'], s['y'], s['z']]
    if t == 'Cylinder': r = op['radius']; return [-r, -r, 0, r, r, op['height']]
    if t == 'Sphere': r = op['radius']; return [-r, -r, -r, r, r, r]
    if t == 'Translate':
        b = bbox(op['child']); o = op['offset']; return [b[0] + o['x'], b[1] + o['y'], b[2] + o['z'], b[3] + o['x'], b[4] + o['y'], b[5] + o['z']]
    if t in ('Union', 'Difference'):
        a = bbox(op['left']); b = bbox(op['right']) if t == 'Union' else a
        return [min(a[0], b[0]), min(a[1], b[1]), min(a[2], b[2]), max(a[3], b[3]), max(a[4], b[4]), max(a[5], b[5])]
    if t == 'Rotate':
        import math, itertools
        b = bbox(op['child']); ax, ay, az = (math.radians(op['angles'][k]) for k in 'xyz')
        def rot(p):
            x, y, z = p
            y, z = y * math.cos(ax) - z * math.sin(ax), y * math.sin(ax) + z * math.cos(ax)
            x, z = x * math.cos(ay) + z * math.sin(ay), -x * math.sin(ay) + z * math.cos(ay)
            x, y = x * math.cos(az) - y * math.sin(az), x * math.sin(az) + y * math.cos(az)
            return (x, y, z)
        P = [rot(c) for c in itertools.product((b[0], b[3]), (b[1], b[4]), (b[2], b[5]))]
        return [min(p[i] for p in P) for i in range(3)] + [max(p[i] for p in P) for i in range(3)]
    return [0, 0, 0, 0, 0, 0]
for k, inst in enumerate(d['instances']):
    f = inst['id']
    if f in KEEP: continue
    pd = d['partDefs'][inst['partDefId']]; b = bbox(pd['root'])
    p = ((b[0] + b[3]) / 2, (b[1] + b[4]) / 2, b[5])
    nid = 90000 + k
    nodes[str(nid)] = {'id': nid, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 2.0, 'y': 2.0, 'z': 2.0}}}
    pd['root'] = nid
    j = next(j for j in d['joints'] if j['childInstanceId'] == f)
    a = j['childAnchor']; j['childAnchor'] = {'x': a['x'] + 1 - p[0], 'y': a['y'] + 1 - p[1], 'z': a['z'] - p[2]}
# keep the framing identical to the full render: two unlabelled 1 mm markers at the full doc's bbox corners
WB = [1e9] * 3 + [-1e9] * 3
for j in ORIG['joints']:
    if j['parentInstanceId'] != 'frame' or j['kind']['type'] != 'Fixed': continue
    inst = next(i for i in ORIG['instances'] if i['id'] == j['childInstanceId'])
    b = bbox(ORIG['partDefs'][inst['partDefId']]['root']); pa, ca = j['parentAnchor'], j['childAnchor']
    o = (pa['x'] - ca['x'], pa['y'] - ca['y'], pa['z'] - ca['z'])
    for i in range(3): WB[i] = min(WB[i], b[i] + o[i]); WB[i + 3] = max(WB[i + 3], b[i + 3] + o[i])
fb = bbox(ORIG['partDefs']['frame']['root'])
for i in range(3): WB[i] = min(WB[i], fb[i]); WB[i + 3] = max(WB[i + 3], fb[i + 3])
nodes['99990'] = {'id': 99990, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 1.0, 'y': 1.0, 'z': 1.0}}}
d['partDefs']['bbmark'] = {'id': 'bbmark', 'name': '', 'root': 99990, 'defaultMaterial': 'alu'}
for k, (x, y, z) in enumerate(((WB[0], WB[1], WB[2]), (WB[3] - 1, WB[4] - 1, WB[5] - 1))):
    d['instances'].append({'id': 'bbm%d' % k, 'partDefId': 'bbmark', 'name': ''})
    d['joints'].append({'id': 'jbbm%d' % k, 'name': 'bbm%d' % k, 'parentInstanceId': 'frame', 'childInstanceId': 'bbm%d' % k,
                        'parentAnchor': {'x': x, 'y': y, 'z': z}, 'childAnchor': {'x': 0, 'y': 0, 'z': 0}, 'kind': {'type': 'Fixed'}, 'state': 0.0})
json.dump(d, open(sys.argv[2], 'w'))
