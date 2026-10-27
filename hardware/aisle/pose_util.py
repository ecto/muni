import json
J = ['J1_yaw', 'J2_sho_pitch', 'J4_elbow', 'J5_fore_roll', 'J6_wrist_pitch', 'J7_tool_roll']
TQ = {'J2_sho_pitch': -20, 'J4_elbow': 145, 'J6_wrist_pitch': 140}
def arm(pfx, q): return {'%s_%s' % (pfx, k): q.get(k, 0.0) for k in J}
TUCK = {}
for p, y in (('arm_1l', -120), ('arm_1r', 60), ('arm_2l', -120), ('arm_2r', 60), ('bench_arm_l', -120), ('bench_arm_r', 60)):
    TUCK.update(arm(p, {**TQ, 'J1_yaw': y}))
BASE = {**TUCK, 'tender_1_x': 900, 'tender_2_x': 3900, 'tender_1_lift': 0, 'tender_2_lift': 0, 'bench_carriage_x': 600}
def posed(base, st):
    d = json.loads(json.dumps(base))
    for j in d['joints']:
        if j['name'] in st: j['state'] = st[j['name']]
    return d
