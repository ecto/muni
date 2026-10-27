import subprocess, sys, os, time, math
from multiprocessing import Pool
sys.path.insert(0, '.')
import gen_hall as G
from PIL import Image, ImageDraw, ImageFont
R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render'); L = __import__('os').environ.get('LOON2VCAD', 'loon2vcad')
F = '/System/Library/Fonts/SFNS.ttf'
YS = G.ROW_D + 2350
ALL = set(G.TRIPLES) | set(G.SINGLES)
# install order: (slot, kind)
EV = [(('A', 1, 15), 'bench'), (('B', -1, 27), 'bench'), (('A', -1, 22), 'mill'),
      (('C', 1, 33), 'bench'), (('A', 1, 55), 'rack'), (('B', 1, 20), 'mill'), (('B', -1, 50), 'winder'), (('C', -1, 14), 'saw'), (('C', 1, 45), 'mill')]
def state(n_done):
    done = EV[:n_done]
    emp = ALL - {k for k, _ in done}
    fill = {k: v for k, v in done if v != 'bench'}
    return emp, fill
def n_empty(emp): return sum(3 if k in G.TRIPLES else 1 for k in emp)
def n_bench(emp): return 10 + sum(1 for k in G.TRIPLES if k not in emp)
def target(slot):
    r, s, i = slot; x0, y0 = G.bay_origin(r, s, i)
    return x0 + (750 if slot in G.TRIPLES else 250), G.ROWS[r]
def path(slot, u):
    """dock (0,7000) -> (1500,7000) -> (1500,rowY) -> (bayX,rowY). u 0..1 by arc length. returns x, y, along_y"""
    tx, ty = target(slot)
    pts = [(200, 7000), (G.SP_X0, 7000), (G.SP_X0, ty), (tx, ty)]
    segs = [math.dist(a, b) for a, b in zip(pts, pts[1:])]; T = sum(segs); d = u * T
    for (a, b), sl in zip(zip(pts, pts[1:]), segs):
        if d <= sl and sl > 0:
            f = d / sl; return a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[0] == b[0]
        d -= sl
    return tx, ty, False
FPS, N = 24, 240
EVF = 20; T0 = 12   # frames per install, start
def frame_spec(f):
    if f < T0: return 0, None, 0.0
    k = (f - T0) // EVF
    if k < len(EV):
        u = ((f - T0) % EVF) / (EVF - 4)
        if u >= 1: return k + 1, None, 0.0
        slot, kind = EV[k]; x, y, ay = path(slot, u * u * (3 - 2 * u))
        return k, (x, y, ay, kind), 0.0
    rd = min(1.0, (f - T0 - EVF * len(EV)) / 30)
    return len(EV), None, max(rd, 0.02)
def doc(n_done, carry, rd):
    emp, fill = state(n_done)
    d, ymax, ne = G.build(empties=emp, fill=fill, row_d=rd, carry=carry, show_static_carry=False, labels=False, stake=YS)
    return d, emp
def caption(img, emp, rd, title=None, size=24):
    im = Image.open(img).convert('RGBA'); o = Image.new('RGBA', im.size, (250, 250, 247, 255)); o.alpha_composite(im)
    d = ImageDraw.Draw(o); f = ImageFont.truetype(F, size)
    t = 'empty bays %2d   ·   benches %d   ·   %s' % (n_empty(emp), n_bench(emp), 'row D being laid' if rd > 0 else '3 rows')
    d.text((20, o.height - 16), t, font=f, fill=(15, 15, 25, 255), anchor='ls')
    if title: d.text((20, 14), title, font=ImageFont.truetype(F, int(size * 1.2)), fill=(15, 15, 25, 255), anchor='lt')
    o.convert('RGB').save(img)
def render(d, base, size):
    open(base + '.loon', 'w').write(d.loon(ymax=YS))
    subprocess.run('%s %s.loon > %s.vcad' % (L, base, base), shell=True, check=True)
    subprocess.run([R, base + '.vcad', '--view', 'top', '--size', size, '--fill', '0.92', '-o', base + '.png'], check=True, stderr=subprocess.DEVNULL)
def do_frame(f):
    n, carry, rd = frame_spec(f); d, emp = doc(n, carry, rd)
    b = 'frames/f%04d' % f; render(d, b, '1280x720'); caption(b + '.png', emp, rd, 'the hall grows a bay at a time', 22)
    os.remove(b + '.loon'); os.remove(b + '.vcad')
if __name__ == '__main__':
    t = time.time()
    for name, n, rd, ttl in (('hall_grow_1', 0, 0, '1 · 15 empty bays'), ('hall_grow_2', 3, 0, '2 · 8 empty bays (two benches + a mill installed)'),
                             ('hall_grow_3', len(EV), 0.5, '3 · 0 empty bays, row D spine being laid, membrane moved out')):
        d, emp = doc(n, None, rd); render(d, name, '2000x1100'); caption(name + '.png', emp, rd, ttl, 30)
        print(name, n_empty(emp), n_bench(emp))
    print('stills %.1fs' % (time.time() - t))
    if 'stills' in sys.argv: sys.exit()
    os.makedirs('frames', exist_ok=True); t = time.time()
    with Pool(8) as p: p.map(do_frame, range(N))
    print('frames %.1fs' % (time.time() - t))
    subprocess.run('ffmpeg -y -loglevel error -framerate 24 -i frames/f%04d.png -c:v libx264 -pix_fmt yuv420p -crf 20 hall_growth.mp4', shell=True, check=True)
    # 6 s gif: every 2nd frame of the 10 s -> 12 fps... speed up 10->6 s: take frames at 10/6 step, 12 fps
    subprocess.run('ffmpeg -y -loglevel error -i hall_growth.mp4 -vf "setpts=0.6*PTS,fps=12,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse=dither=bayer" hall_growth.gif', shell=True, check=True)
