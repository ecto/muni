"""Poses (from kin_v2.ik) + frames for cell_v2_working.mp4.
usage: python3 make_frames_v2.py table                       -> joint table + TCP check for every pose
       python3 make_frames_v2.py pose NAME BASE.vcad OUT.vcad -> one posed doc
       python3 make_frames_v2.py [frame ids...]              -> frames into frames_v2/ (panel-free doc)"""
import json, os, sys, subprocess, time
from multiprocessing import Pool
from kin_v2 import ik, fk, margin
S = os.path.dirname(os.path.abspath(__file__)); R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render')
FPS = 24
J = ['J1_yaw', 'J2_sho_pitch', 'J4_elbow', 'J5_fore_roll', 'J6_wrist_pitch', 'J7_tool_roll']
def arm(side, q): return {'arm_%s_%s' % (side, k): q.get(k, 0.0) for k in J}
from kin_v2 import shoulder
import math
def solve(c, side, p, tp, ref=None):
    # IK branches within limits (margin >= 10 deg) whose elbow and wrist stay above z 380 (clear of the deck,
    # fixture tops, door frames); pick the one nearest `ref` in joint space (smooth, short moves), else the highest.
    sols = [q for q in ik(c, side, p, tp, all_sols=True) if margin(q) >= 10]
    hi = lambda q: min(fk(c, side, q)[0][2], fk(c, side, q)[1][2])
    sols = [q for q in sols if hi(q) >= 380] or sols
    assert sols, ('unreachable', c, side, p, tp)
    if ref: return min(sols, key=lambda q: sum((q[k] - ref.get(k, 0)) ** 2 for k in q))
    return max(sols, key=hi)
def backoff(c, side, p, tp, d=120):
    # tool always points away from the shoulder in plan; back the TCP off d mm along the tool axis
    sx, sy, _ = shoulder(c, side); ux, uy = p[0] - sx, p[1] - sy; n = math.hypot(ux, uy); t = math.radians(tp)
    return (p[0] - d * ux / n * math.cos(t), p[1] - d * uy / n * math.cos(t), p[2] - d * math.sin(t))
# tuck: upper arm 25 deg up, forearm folded back down beside it, tool pointing straight up; folded over the plinth,
# L angled toward +X/-Y and R toward -X/+Y so the two folded arms sit side by side, 236 mm apart in Y
TQ = {'J2_sho_pitch': -20, 'J4_elbow': 145, 'J6_wrist_pitch': 140}
TUCK_L, TUCK_R = arm('l', {**TQ, 'J1_yaw': -120}), arm('r', {**TQ, 'J1_yaw': 60})
C_MILL, C_BENCH, C_OVEN, C_HOME = 340, 435, 870, 600
P = {}   # name -> (carriage, {side: (target, tool pitch)})
P['TUCK'] = (C_HOME, {})
MILL_T, OVEN_T = (280, 780, 330), (1170, 780, 300)             # 80 mm inside each door
P['MILL'] = (C_MILL, {'l': (MILL_T, -40)})                      # tool 40 deg down, entering off the door normal
P['MILL_PRE'] = (C_MILL, {'l': (backoff(C_MILL, 'l', MILL_T, -40, 80), -40)})
P['BENCH'] = (C_BENCH, {'l': ((690, 170, 270), -90), 'r': ((810, 170, 270), -90)})   # two-hand work on the bench, 120 apart
P['HANDOFF'] = (C_BENCH, {'l': ((728, 200, 420), -40), 'r': ((772, 200, 420), -40)})   # jaws meet 150 above the bench, tools 40 deg down, ~70 deg apart
P['OVEN'] = (C_OVEN, {'r': (OVEN_T, -40)})
P['OVEN_PRE'] = (C_OVEN, {'r': (backoff(C_OVEN, 'r', OVEN_T, -40, 60), -50)})   # 60 back, tool -50: further back the elbow hits its +150 stop on this branch
TQS = {'l': {**TQ, 'J1_yaw': -120}, 'r': {**TQ, 'J1_yaw': 60}}
REF = {'MILL_PRE': 'MILL', 'OVEN_PRE': 'OVEN', 'BENCH': 'HANDOFF'}     # solved near this pose (same IK branch)
POSES, QS = {}, {}
def pose(name):
    c, tg = P[name]; st = {**TUCK_L, **TUCK_R, 'carriage_x': c}
    for side, (p, tp) in tg.items():
        q = solve(c, side, p, tp, QS.get((REF.get(name), side), TQS[side])); QS[(name, side)] = q; st.update(arm(side, q))
    return st
for n in ['TUCK', 'MILL', 'MILL_PRE', 'HANDOFF', 'BENCH', 'OVEN', 'OVEN_PRE']: POSES[n] = pose(n)
KEYS = [(0.0, 'TUCK', C_HOME), (0.6, 'TUCK', C_HOME), (1.8, 'TUCK', C_MILL), (2.5, 'MILL_PRE', C_MILL), (3.2, 'MILL', C_MILL),
        (4.0, 'MILL', C_MILL), (4.6, 'MILL_PRE', C_MILL), (5.2, 'TUCK', C_MILL), (6.0, 'TUCK', C_BENCH), (6.7, 'BENCH', C_BENCH),
        (7.3, 'BENCH', C_BENCH), (7.9, 'HANDOFF', C_BENCH), (8.6, 'HANDOFF', C_BENCH), (9.2, 'TUCK', C_BENCH), (10.0, 'TUCK', C_OVEN),
        (10.6, 'OVEN_PRE', C_OVEN), (11.2, 'OVEN', C_OVEN), (11.9, 'OVEN', C_OVEN), (12.4, 'OVEN_PRE', C_OVEN), (13.0, 'TUCK', C_OVEN),
        (14.0, 'TUCK', C_HOME)]
def keyst(n, c): d = dict(POSES[n]); d['carriage_x'] = c; return d
K = [(t, keyst(n, c)) for t, n, c in KEYS]
NAMES = sorted(K[0][1])
ss = lambda u: u * u * (3 - 2 * u)
def at(t):
    for (t0, a), (t1, b) in zip(K, K[1:]):
        if t <= t1:
            u = ss(max(0, min(1, (t - t0) / (t1 - t0)))); return {n: a[n] + (b[n] - a[n]) * u for n in NAMES}
    return K[-1][1]
def posed(base, st):
    d = json.loads(json.dumps(base))
    for j in d['joints']:
        if j['name'] in st: j['state'] = st[j['name']]
    return d
N = int(round(K[-1][0] * FPS)) + 1
def job(i):
    base = json.load(open(f"{S}/cell_v2_np_raw.vcad"))
    v = f"{S}/fvcad_v2/{i:04d}.vcad"; json.dump(posed(base, at(i / FPS)), open(v, 'w'))
    az = 230 + 10 * i / (N - 1)
    subprocess.run([R, v, '--azimuth', f"{az:.2f}", '--elevation', '22', '--size', '1280x720', '-o', f"{S}/frames_v2/{i:04d}.png"], check=True, capture_output=True)
def table():
    print('| pose | q (mm) | arm | J1 | J2 | J4 | J5 | J6 | J7 | TCP (X,Y,Z) | elbow z | min limit margin |')
    print('|---|---|---|---|---|---|---|---|---|---|---|---|')
    for n, st in POSES.items():
        c = st['carriage_x']
        for s in 'lr':
            q = {k: st['arm_%s_%s' % (s, k)] for k in J}
            e, w, t = fk(c, s, q)
            print('| %s | %d | %s | %s | (%.0f, %.0f, %.0f) | %.0f | %.0f |' % (n, c, s.upper(), ' | '.join('%.1f' % q[k] for k in J), *t, e[2], margin(q)))
    print('\nkeyframes (t s, pose, carriage q):', ', '.join('%.1f %s %d' % k for k in KEYS))
if __name__ == '__main__':
    a = sys.argv[1:]
    if a and a[0] == 'table': table(); sys.exit()
    if a and a[0] == 'pose':
        json.dump(posed(json.load(open(a[2])), POSES[a[1]]), open(a[3], 'w')); sys.exit()
    os.makedirs(f"{S}/frames_v2", exist_ok=True); os.makedirs(f"{S}/fvcad_v2", exist_ok=True)
    only = [int(x) for x in a] or range(N)
    t = time.time(); Pool(12).map(job, only); print(N, 'frames', round(time.time() - t, 1), 's')
