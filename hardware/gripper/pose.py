"""pose a gripper .vcad: python3 pose.py IN OUT opening_mm [lift_mm] -- sets finger_l/finger_r = opening/2,
pinion = -(opening/2)/r rad (rack_l rides on top of the pinion), lift/can_lift = lift."""
import json, sys, math
def pose(d, opening, lift=0.0, r=8.0):
    q = max(0.0, min(50.0, opening / 2))
    st = {'finger_l': q, 'finger_r': q, 'pinion': -math.degrees(q / r), 'lift': lift, 'can_lift': lift}
    for j in d['joints']:
        if j['name'] in st: j['state'] = st[j['name']]
    return d
if __name__ == '__main__':
    d = json.load(open(sys.argv[1]))
    import os
    if os.environ.get('STRIP'):   # labelled stills: drop the tiny dowel/magnet instances so their labels don't bury the rest
        drop = lambda i: i.startswith(('dowel_', 'magnet_'))
        d['instances'] = [i for i in d['instances'] if not drop(i['id'])]
        d['joints'] = [j for j in d['joints'] if not drop(j['childInstanceId'])]
    json.dump(pose(d, float(sys.argv[3]), float(sys.argv[4]) if len(sys.argv) > 4 else 0.0), open(sys.argv[2], 'w'))
