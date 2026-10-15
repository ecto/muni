"""gripper_grasp.mp4 frames: open -> close on the can -> lift 60 -> lower -> open, slow orbit. 24 fps, 8 s."""
import json, os, subprocess
from multiprocessing import Pool
from pose import pose
S = os.path.dirname(os.path.abspath(__file__)); R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render')
FPS, DUR = 24, 8.0
K = [(0, 100, 0), (0.8, 100, 0), (2.4, 66, 0), (2.9, 66, 0), (4.5, 66, 60), (5.2, 66, 60), (6.2, 66, 0), (6.5, 66, 0), (7.4, 100, 0), (8, 100, 0)]
ss = lambda u: u * u * (3 - 2 * u)
def at(t):
    for (t0, o0, l0), (t1, o1, l1) in zip(K, K[1:]):
        if t <= t1:
            u = ss(max(0, min(1, (t - t0) / (t1 - t0)))); return o0 + (o1 - o0) * u, l0 + (l1 - l0) * u
    return K[-1][1:]
N = int(FPS * DUR)
def job(i):
    o, l = at(i / FPS); base = json.load(open(f'{S}/gripper_can_raw.vcad'))
    v = f'{S}/frames/{i:04d}.vcad'; json.dump(pose(base, o, l), open(v, 'w'))
    az = 200 + 50 * i / (N - 1)
    subprocess.run([R, v, '--azimuth', f'{az:.2f}', '--elevation', '18', '--size', '1280x720', '-o', f'{S}/frames/{i:04d}.png'], check=True, capture_output=True)
    os.remove(v)
if __name__ == '__main__':
    os.makedirs(f'{S}/frames', exist_ok=True)
    with Pool(8) as p: p.map(job, range(N))
