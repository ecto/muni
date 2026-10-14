"""seed_growth.mp4 (1280x720, 24 fps, 12 s) + seed_growth.gif (6 s): bench_2_frame bars appear one at a time
while the right arm cycles reach/return; camera orbits az 200 -> 220, el 20.
Rendered at 12 fps (144 frames) and doubled to 24 fps to keep render time sane."""
import json, os, subprocess, math, time
from multiprocessing import Pool
import pose_seed
R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render'); L2V = __import__('os').environ.get('LOON2VCAD', 'loon2vcad')
N, FPS = 144, 12
def nbars(f): return min(12, 1 + f * 12 // (N - 12))          # all 12 in place for the last second
def build(n):
    subprocess.run(f'python3 seed.py {n} > fv/b{n}.loon && {L2V} fv/b{n}.loon > fv/b{n}.vcad', shell=True, check=True)
def frame(f):
    u = 0.5 - 0.5 * math.cos(2 * math.pi * f / 48)              # reach/return every 4 s
    d = pose_seed.pose(json.load(open(f'fv/b{nbars(f)}.vcad')), u)
    v = f'fv/{f:04d}.vcad'; json.dump(d, open(v, 'w'))
    subprocess.run([R, v, '--azimuth', '%.2f' % (200 + 20 * f / (N - 1)), '--elevation', '20', '--size', '1280x720',
                    '-o', f'frames/{f:04d}.png'], check=True, stderr=subprocess.DEVNULL)
    os.remove(v)
if __name__ == '__main__':
    os.makedirs('fv', exist_ok=True); os.makedirs('frames', exist_ok=True); t = time.time()
    with Pool(6) as p: p.map(build, range(1, 13))
    with Pool(int(os.environ.get('JOBS', 8))) as p: p.map(frame, range(N))
    print('frames %.1fs' % (time.time() - t))
    subprocess.run(f'ffmpeg -y -loglevel error -framerate {FPS} -i frames/%04d.png -vf "fps=24,format=yuv420p" -c:v libx264 -crf 20 seed_growth.mp4', shell=True, check=True)
    subprocess.run('ffmpeg -y -loglevel error -i seed_growth.mp4 -vf "setpts=0.5*PTS,fps=12,scale=640:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse" seed_growth.gif', shell=True, check=True)
