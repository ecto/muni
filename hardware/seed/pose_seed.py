"""Pose the seed doc. python3 pose_seed.py IN.vcad OUT.vcad U  (U 0 = tucked, 1 = right arm reaching out the doors toward bench_2)"""
import json, sys
J = ['J1_yaw', 'J2_sho_pitch', 'J4_elbow', 'J5_fore_roll', 'J6_wrist_pitch', 'J7_tool_roll']
TQ = {'J2_sho_pitch': -20, 'J4_elbow': 145, 'J6_wrist_pitch': 140}           # make_frames_v2 TUCK
TUCK = {**{'arm_l_' + k: v for k, v in {**TQ, 'J1_yaw': -120}.items()}, **{'arm_r_' + k: v for k, v in {**TQ, 'J1_yaw': 60}.items()}}
REACH_R = {'J1_yaw': 170, 'J2_sho_pitch': -15, 'J4_elbow': 5, 'J6_wrist_pitch': 10}   # zero pose = +Y; yaw 170 -> out the -Y doors
CAR = 435
def state(u):
    s = {('arm_%s_%s' % (sd, k)): 0.0 for sd in 'lr' for k in J}; s.update(TUCK); s['carriage_x'] = CAR
    for k, v in REACH_R.items():
        a = TUCK.get('arm_r_' + k, 0.0); s['arm_r_' + k] = a + (v - a) * u
    return s
def pose(d, u):
    st = state(u)
    for j in d['joints']:
        if j['name'] in st: j['state'] = st[j['name']]
    return d
if __name__ == '__main__':
    json.dump(pose(json.load(open(sys.argv[1])), float(sys.argv[3])), open(sys.argv[2], 'w'))
