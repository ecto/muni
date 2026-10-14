"""Frames for cell_real_working.mp4. Poses from kin_real.ik (see the KEYS table); joint-space smoothstep between keys.
usage: python3 make_frames_real.py            -> all frames into frames_real/
       python3 make_frames_real.py pose NAME C OUT.vcad   -> write one posed doc (used by build_real.sh)"""
import json, os, sys, subprocess, time
from multiprocessing import Pool
S = os.path.dirname(os.path.abspath(__file__)); R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render')
FPS = 24
def arm(side, j1, j2, j4, j6, j5=0, j7=0):
    p = 'arm_%s_' % side
    return {p + 'J1_yaw': j1, p + 'J2_sho_pitch': j2, p + 'J4_elbow': j4, p + 'J5_fore_roll': j5, p + 'J6_wrist_pitch': j6, p + 'J7_tool_roll': j7}
TUCK_L, TUCK_R = arm('l', 90, 35, -160, 115), arm('r', 90, 35, -160, 115)   # folded toward -Y, lowest point z~570
TUCK = {**TUCK_L, **TUCK_R}
MILL_PRE = {**arm('l', 90, -13.0, -37.9, -99.2), **TUCK_R}       # TCP (280,300,300) at the door, tool +Y 30 deg down
MILL = {**arm('l', 90, -12.7, -61.9, -75.4), **TUCK_R}           # TCP (280,380,260), 60 mm inside the door
PLATE = {**arm('l', 38.7, -18.5, -104.1, 32.6), **arm('r', -141.3, -18.5, -104.1, 32.6)}  # top-down at (650,420,150) / (850,580,150)
OVEN_PRE = {**TUCK_L, **arm('r', 90, -15.9, -55.0, -79.2)}       # TCP (1170,370,250)
OVEN = {**TUCK_L, **arm('r', 90, -20.5, -77.6, -51.8)}           # TCP (1170,480,220), 80 mm inside the door
POSES = dict(TUCK=TUCK, MILL_PRE=MILL_PRE, MILL=MILL, PLATE=PLATE, OVEN_PRE=OVEN_PRE, OVEN=OVEN)
KEYS = [(0.0, 500, TUCK), (0.6, 500, TUCK), (2.0, 230, TUCK), (2.7, 230, MILL_PRE), (3.4, 230, MILL), (4.2, 230, MILL),
        (4.9, 230, MILL_PRE), (5.6, 230, TUCK), (6.6, 500, TUCK), (7.3, 500, PLATE), (7.9, 500, PLATE), (8.5, 500, TUCK),
        (9.5, 720, TUCK), (10.1, 720, OVEN_PRE), (10.7, 720, OVEN), (11.3, 720, OVEN), (11.9, 720, OVEN_PRE), (12.5, 720, TUCK)]
NAMES = sorted({k for _, _, p in KEYS for k in p} | {'carriage_x'})
def full(c, p): d = {n: 0.0 for n in NAMES}; d.update(p); d['carriage_x'] = c; return d
K = [(t, full(c, p)) for t, c, p in KEYS]
ss = lambda u: u * u * (3 - 2 * u)
def at(t):
    for (t0, a), (t1, b) in zip(K, K[1:]):
        if t <= t1:
            u = ss(max(0, min(1, (t - t0) / (t1 - t0)))); return {n: a[n] + (b[n] - a[n]) * u for n in NAMES}
    return K[-1][1]
base = json.load(open(f"{S}/cell_real_raw.vcad"))
def posed(st):
    d = json.loads(json.dumps(base))
    for j in d['joints']:
        if j['name'] in st: j['state'] = st[j['name']]
    return d
N = int(round(K[-1][0] * FPS)) + 1
def job(i):
    v = f"{S}/fvcad_real/{i:04d}.vcad"; json.dump(posed(at(i / FPS)), open(v, 'w'))
    az = 230 + 10 * i / (N - 1)
    subprocess.run([R, v, '--azimuth', f"{az:.2f}", '--elevation', '22', '--size', '1280x720', '-o', f"{S}/frames_real/{i:04d}.png"], check=True, capture_output=True)
if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'pose':
        json.dump(posed(full(float(sys.argv[3]), POSES[sys.argv[2]])), open(sys.argv[4], 'w')); sys.exit()
    os.makedirs(f"{S}/frames_real", exist_ok=True); os.makedirs(f"{S}/fvcad_real", exist_ok=True)
    only = [int(x) for x in sys.argv[1:]] or range(N)
    t = time.time(); Pool(12).map(job, only); print(N, 'frames', round(time.time() - t, 1), 's')
