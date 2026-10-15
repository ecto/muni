"""Poses (kin.ik) + keyframes + frames for bench_shallow_working.mp4.
usage: python3 frames.py table                        -> joint table + TCP check for every pose
       python3 frames.py pose NAME BASE.vcad OUT.vcad -> one posed doc
       python3 frames.py                              -> frames/ (unique poses rendered once, holds reuse them)"""
import json, os, sys, subprocess, time, math, shutil
from multiprocessing import Pool
from kin import ik, fk, margin, shoulder
from bsp import *
S = os.path.dirname(os.path.abspath(__file__)); R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render')
FPS = 24
J = ['J1_yaw', 'J2_sho_pitch', 'J4_elbow', 'J5_fore_roll', 'J6_wrist_pitch', 'J7_tool_roll']
def arm(side, q): return {'arm_%s_%s' % (side, k): q.get(k, 0.0) for k in J}
def solve(c, side, p, band, ref=None):
    """IK branches within limits (margin >= 10) for any pitch in band; prefer elbow+wrist above z 330 (clear of the
    deck edge / bay fixtures), then nearest to ref in joint space."""
    sols = [(tp, q) for tp in band for q in ik(c, side, p, tp, all_sols=True) if margin(q) >= 10]
    hi = lambda q: min(fk(c, side, q)[0][2], fk(c, side, q)[1][2])
    good = [s for s in sols if hi(s[1]) >= 330] or sols
    assert good, ('unreachable', c, side, p)
    if ref: return min(good, key=lambda s: sum((s[1][k] - ref.get(k, 0)) ** 2 for k in s[1]))
    return good[0]
def lift(p, d): return (p[0], p[1], p[2] + d)
def back(c, side, p, d):           # back off d mm toward the shoulder in plan (for door / rack approaches)
    sx, sy, _ = shoulder(c, side); ux, uy = p[0] - sx, p[1] - sy; n = math.hypot(ux, uy)
    return (p[0] - d * ux / n, p[1] - d * uy / n, p[2])
DOWN = [-90, -80, -70, -60, -50, -45]; FWD = [0, -10, -20, -30, -40]
# tuck: upper arm 20 deg up, forearm folded back down, tool up; L angled -30 deg, R 150 deg -> folded over the plinth
TQ = {'J2_sho_pitch': -20, 'J4_elbow': 145, 'J6_wrist_pitch': 140}
TQS = {'l': {**TQ, 'J1_yaw': -30}, 'r': {**TQ, 'J1_yaw': 150}}
C_MILL, C_BENCH, C_RACK, C_WIND, C_HOME = 0, 395, 200, 790, 395
MILL_T = (250, -110, 390); WIND_T = (1360, -160, 365); PEG_T = (290, 345, 740)
P = {'TUCK': (C_HOME, {}),
     'MILL': (C_MILL, {'l': (MILL_T, FWD)}), 'MILL_PRE': (C_MILL, {'l': ((250, -40, 470), FWD + DOWN)}),
     'BENCH': (C_BENCH, {'l': ((690, -200, 215), DOWN), 'r': ((810, -200, 215), DOWN)}),
     'HANDOFF': (C_BENCH, {'l': ((725, -150, 430), [-30, -40, -20]), 'r': ((775, -150, 430), [-30, -40, -20])}),
     'RACK': (C_RACK, {'l': (PEG_T, FWD)}), 'RACK_PRE': (C_RACK, {'l': (lift(back(C_RACK, 'l', PEG_T, 0), -0) if False else (PEG_T[0], PEG_T[1] - 60, PEG_T[2] + 0), FWD)}),
     'RACK_OFF': (C_RACK, {'l': ((PEG_T[0], PEG_T[1] - 60, PEG_T[2] - 30), FWD)}),
     'WIND': (C_WIND, {'r': (WIND_T, DOWN)}), 'WIND_PRE': (C_WIND, {'r': (lift(WIND_T, 120), DOWN)})}
REF = {'MILL_PRE': 'MILL', 'BENCH': 'HANDOFF', 'RACK_PRE': 'RACK', 'RACK_OFF': 'RACK', 'WIND_PRE': 'WIND'}
POSES, QS, TP = {}, {}, {}
def pose(name):
    c, tg = P[name]; st = {**arm('l', TQS['l']), **arm('r', TQS['r']), 'carriage_x': c}
    for side, (p, band) in tg.items():
        tp, q = solve(c, side, p, band, QS.get((REF.get(name), side), TQS[side])); QS[(name, side)] = q; TP[(name, side)] = tp; st.update(arm(side, q))
    return st
for n in ['TUCK', 'MILL', 'MILL_PRE', 'HANDOFF', 'BENCH', 'RACK', 'RACK_PRE', 'RACK_OFF', 'WIND', 'WIND_PRE']: POSES[n] = pose(n)
KEYS = [(0.0, 'TUCK', C_HOME), (0.5, 'TUCK', C_HOME), (1.3, 'TUCK', C_MILL), (1.9, 'MILL_PRE', C_MILL), (2.5, 'MILL', C_MILL),
        (3.3, 'MILL', C_MILL), (3.8, 'MILL_PRE', C_MILL), (4.3, 'TUCK', C_MILL), (5.0, 'TUCK', C_BENCH), (5.6, 'BENCH', C_BENCH),
        (6.2, 'BENCH', C_BENCH), (6.8, 'HANDOFF', C_BENCH), (7.6, 'HANDOFF', C_BENCH), (8.1, 'TUCK', C_BENCH), (8.6, 'TUCK', C_RACK),
        (9.1, 'RACK_PRE', C_RACK), (9.6, 'RACK', C_RACK), (10.1, 'RACK', C_RACK), (10.5, 'RACK_OFF', C_RACK), (10.9, 'TUCK', C_RACK),
        (11.6, 'TUCK', C_WIND), (12.1, 'WIND_PRE', C_WIND), (12.6, 'WIND', C_WIND), (13.3, 'WIND', C_WIND), (13.7, 'WIND_PRE', C_WIND),
        (14.1, 'TUCK', C_WIND), (14.6, 'TUCK', C_HOME)]
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
VIEW = ['--azimuth', '250', '--elevation', '40', '--size', '1600x900', '--fill', '0.9', '--aa', '2']   # cropped to 1280x720 in build.sh
def strip_holes(d):
    # Bolt/lightening holes are [difference cut body] (loon: B - A -> body = right). They are what makes a render
    # slow (~90 s CPU per arm vs <1 s without), so the video uses hole-free arms; stills keep them.
    N = d['nodes']
    def res(i):
        op = N[str(i)]['op']; return res(op['right']) if op['type'] == 'Difference' else i
    for pd in d['partDefs'].values(): pd['root'] = res(pd['root'])
    return d
def job(i):
    base = strip_holes(json.load(open(f"{S}/video_base.vcad")))   # np doc minus human/table (they hide the bays at this angle)
    v = f"{S}/fvcad/{i:04d}.vcad"; json.dump(posed(base, at(i / FPS)), open(v, 'w'))
    subprocess.run([R, v, *VIEW, '-o', f"{S}/vfr2/{i:04d}.jpg"], check=True, capture_output=True)
def table():
    print('| pose | q | arm | J1 | J2 | J4 | J5 | J6 | J7 | tool pitch | TCP (X,Y,Z) | elbow z | wrist z | min margin |')
    print('|---|---|---|---|---|---|---|---|---|---|---|---|---|---|')
    for n, st in POSES.items():
        c = st['carriage_x']
        for s in 'lr':
            q = {k: st['arm_%s_%s' % (s, k)] for k in J}; e, w, t = fk(c, s, q)
            print('| %s | %d | %s | %s | %s | (%.0f, %.0f, %.0f) | %.0f | %.0f | %.0f |' % (n, c, s.upper(), ' | '.join('%.1f' % q[k] for k in J),
                  TP.get((n, s), 'tuck'), *t, e[2], w[2], margin(q)))
    print('\nkeyframes (t s, pose, carriage q):', ', '.join('%.1f %s %d' % k for k in KEYS))
if __name__ == '__main__':
    a = sys.argv[1:]
    if a and a[0] == 'table': table(); sys.exit()
    if a and a[0] == 'pose': json.dump(posed(json.load(open(a[2])), POSES[a[1]]), open(a[3], 'w')); sys.exit()
    os.makedirs(f"{S}/vfr2", exist_ok=True); os.makedirs(f"{S}/fvcad", exist_ok=True)
    # holds: identical state -> render once, copy
    uniq, first = {}, {}
    for i in range(N):
        key = tuple(round(v, 4) for v in at(i / FPS).values()); first.setdefault(key, i); uniq[i] = first[key]
    todo = sorted(set(uniq.values())); todo = [i for i in todo if not os.path.exists(f"{S}/vfr2/{i:04d}.jpg")]
    t = time.time(); Pool(int(os.environ.get('NP', 8))).map(job, todo)
    for i, j in uniq.items():
        if i != j: shutil.copy(f"{S}/vfr2/{j:04d}.jpg", f"{S}/vfr2/{i:04d}.jpg")
    print(N, 'frames,', len(set(uniq.values())), 'unique,', round(time.time() - t, 1), 's')
