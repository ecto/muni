"""Poses (kin_aisle IK) + stills + frames for aisle_working.mp4.
usage: python3 make_frames_aisle.py table | pose NAME OUT.vcad | frames [ids...]"""
import json, os, sys, subprocess, time, math
from multiprocessing import Pool
from kin_aisle import ik, fk, margin, shoulder, T_X0
from pose_util import arm, J, TQ, posed
S = os.path.dirname(os.path.abspath(__file__)); R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render')
FPS = 24
TUCKQ = {'l': {**TQ, 'J1_yaw': -30}, 'r': {**TQ, 'J1_yaw': 150}}       # cell_v2 tuck, re-expressed for the +X zero pose
def solve(x, lift, side, p, tp, ref, zmin=150):
    best = None
    for d in (0, 5, -5, 10, -10, 15, -15, 20, -20, 30, -30, 40, -40):
        sols = [q for q in (ik(x, lift, side, p, tp + d, all_sols=True) or []) if margin(q) >= 10]
        hi = lambda q: min(fk(x, lift, side, q)[0][2], fk(x, lift, side, q)[1][2])
        sols = [q for q in sols if hi(q) >= zmin]
        if sols: return min(sols, key=lambda q: sum((q[k] - ref.get(k, 0)) ** 2 for k in q))
    raise SystemExit('unreachable %s' % ((x, lift, side, p, tp),))
TX = lambda cx: cx - T_X0                    # tender_x for plinth centre X
# pose name -> (tender, centre X, lift, {side: (target, tool pitch)})
P = {}
P['T1_HOME'] = (1, 1200, 0, {}); P['T2_HOME'] = (2, 4200, 0, {})
# tender_1: machines (mill -> router -> saw), moving -X
P['T1_MILL_PRE'] = (1, 2250, 0, {'r': ((2250, 1700, 880), -25)})
P['T1_MILL'] = (1, 2250, 0, {'r': ((2250, 1840, 880), -25)})
P['T1_ROUTER'] = (1, 1500, 0, {'r': ((1500, 1950, 560), -50), 'l': ((1300, 1900, 560), -50)})
P['T1_SAW'] = (1, 500, 150, {'r': ((600, 1900, 950), -20), 'l': ((400, 1900, 950), -20)})
# tender_2: tray from the rack (highest shelf reachable at 600 lift = shelf 7, top z 1486) -> bench bay
SH7 = 200 + 6 * 1500 / 7
P['T2_RACK_PRE'] = (2, 3500, 600, {'l': ((3350, 1690, SH7 + 40), 0), 'r': ((3650, 1690, SH7 + 40), 0)})
P['T2_RACK'] = (2, 3500, 600, {'l': ((3350, 1860, SH7 + 30), 0), 'r': ((3650, 1860, SH7 + 30), 0)})
P['T2_LIFTTRAY'] = (2, 3500, 600, {'l': ((3350, 1860, SH7 + 70), 0), 'r': ((3650, 1860, SH7 + 70), 0)})
P['T2_CARRY'] = (2, 3500, 600, {'l': ((3350, 1620, SH7 + 70), 0), 'r': ((3650, 1620, SH7 + 70), 0)})
P['T2_BENCH_PRE'] = (2, 1750, 0, {'l': ((1660, 1000, 520), -40), 'r': ((1840, 1000, 520), -40)})
P['T2_BENCH'] = (2, 1750, 0, {'l': ((1690, 830, 286), -60), 'r': ((1810, 830, 286), -60)})
P['T2_BENCH_HI'] = (2, 1750, 0, {'l': ((1728, 800, 476), -40), 'r': ((1772, 800, 476), -40)})
P['T1_TOPSHELF_MAX'] = (1, 3500, 600, {'l': ((3350, 1860, SH7 + 30), 0), 'r': ((3650, 1860, SH7 + 30), 0)})
REF = {'T1_MILL': 'T1_MILL_PRE', 'T2_RACK': 'T2_RACK_PRE', 'T2_LIFTTRAY': 'T2_RACK', 'T2_CARRY': 'T2_LIFTTRAY', 'T2_BENCH': 'T2_BENCH_PRE', 'T2_BENCH_HI': 'T2_BENCH'}
QS, ST = {}, {}
for n, (t, cx, lift, tg) in P.items():
    st = {'tender_%d_x' % t: TX(cx), 'tender_%d_lift' % t: lift}
    for s in 'lr':
        q = TUCKQ[s]
        if s in tg:
            ref = QS.get((REF.get(n), s), {**TUCKQ[s], 'J1_yaw': 90 if tg[s][0][1] > 1350 else -90})
            q = solve(TX(cx), lift, s, tg[s][0], tg[s][1], ref)
        QS[(n, s)] = q; st.update(arm('arm_%d%s' % (t, s), q))
    ST[n] = st
BENCH_TUCK = {**arm('bench_arm_l', {**TQ, 'J1_yaw': -120}), **arm('bench_arm_r', {**TQ, 'J1_yaw': 60}), 'bench_carriage_x': 600}
def full(n1, n2, **kw): d = {**BENCH_TUCK, **ST[n1], **ST[n2]}; d.update(kw); return d
# ---- timeline (s): (t, T1 pose, T2 pose)
KEYS = [(0.0, 'T1_HOME', 'T2_HOME'), (0.5, 'T1_HOME', 'T2_HOME'),
        (2.0, 'T1_MILL_PRE', 'T2_RACK_PRE'),  (2.8, 'T1_MILL', 'T2_RACK_PRE'), (3.8, 'T1_MILL', 'T2_RACK'),
        (4.4, 'T1_MILL', 'T2_LIFTTRAY'), (5.0, 'T1_MILL_PRE', 'T2_CARRY'), (5.6, 'T1_MILL_PRE', 'T2_CARRY'),
        (7.0, 'T1_ROUTER', 'T2_CARRY_LOW'), (8.4, 'T1_ROUTER', 'T2_CARRY_LOW'), (10.0, 'T1_SAW', 'T2_CARRY_LOW'),
        (11.6, 'T1_SAW', 'T2_BENCH_PRE'), (12.6, 'T1_SAW', 'T2_BENCH'), (13.6, 'T1_SAW', 'T2_BENCH'),
        (14.4, 'T1_SAW', 'T2_BENCH_HI'), (15.4, 'T1_SAW', 'T2_BENCH_HI'), (16.0, 'T1_SAW', 'T2_BENCH_HI')]
ST['T2_CARRY_LOW'] = {**ST['T2_CARRY'], 'tender_2_lift': 0}          # lower with the tray held
def tl(t, n, P_):
    return ST[n]
K = [(t, full(a, b)) for t, a, b in KEYS]
NAMES = sorted(K[0][1])
ss = lambda u: u * u * (3 - 2 * u)
def at(t):
    for (t0, a), (t1, b) in zip(K, K[1:]):
        if t <= t1:
            u = ss(max(0, min(1, (t - t0) / (t1 - t0)))); return {n: a[n] + (b[n] - a[n]) * u for n in NAMES}
    return K[-1][1]
N = int(round(K[-1][0] * FPS)) + 1
def job(i):
    base = json.load(open(f"{S}/aisle_raw.vcad"))
    base['instances'] = [x for x in base['instances'] if x['id'] != 'human']      # human hides tender_2 at the rack from this camera
    base['joints'] = [j for j in base['joints'] if j['childInstanceId'] != 'human']
    v = f"{S}/fvcad/{i:04d}.vcad"; json.dump(posed(base, at(i / FPS)), open(v, 'w'))
    az = 200 + 15 * i / (N - 1)
    subprocess.run([R, v, '--azimuth', f"{az:.2f}", '--elevation', '20', '--size', '2560x1440', '-o', f"{S}/frames/{i:04d}.png"], check=True, capture_output=True)
    subprocess.run(['magick', f"{S}/frames/{i:04d}.png", '-background', 'white', '-flatten', '-crop', '1600x900+480+265', '+repage', '-resize', '1280x720', f"{S}/frames/{i:04d}.png"], check=True)
def gap(st):   # plinth centre spacing tender_2 - tender_1
    return st['tender_2_x'] - st['tender_1_x']
def table():
    print('| pose | centre X | lift | arm | J1 | J2 | J4 | J5 | J6 | J7 | TCP (X,Y,Z) | elbow z | margin |'); print('|' + '---|' * 13)
    for n, (t, cx, lift, tg) in P.items():
        for s in tg:
            q = QS[(n, s)]; e, w, tc = fk(TX(cx), lift, s, q)
            print('| %s | %d | %d | %s | %s | (%.0f,%.0f,%.0f) | %.0f | %.0f |' % (n, cx, lift, s.upper(), ' | '.join('%.1f' % q.get(k, 0) for k in J), *tc, e[2], margin(q)))
    g = min(gap(at(i / FPS)) for i in range(N)); print('\nmin tender centre spacing over the video: %.0f mm' % g)
if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'table': table()
    elif a[0] == 'pose':     # pose NAME OUT  (NAME = T1pose+T2pose)
        n1, n2 = a[1].split('+'); json.dump(posed(json.load(open(f'{S}/aisle_raw.vcad')), full(n1, n2)), open(a[2], 'w'))
    elif a[0] == 'frames':
        os.makedirs(f"{S}/frames", exist_ok=True); os.makedirs(f"{S}/fvcad", exist_ok=True)
        only = [int(x) for x in a[1:]] or range(N)
        t = time.time(); Pool(10).map(job, only); print(N, 'frames', round(time.time() - t, 1), 's')
