"""hall generator. build(state) -> (loon, labels_loon). state dict: empties (set of (row,side,i) or bench-slot keys filled), etc.
Z-up mm. Hall X 0..36000, Y 0..14000 (Y grows to 18650 when row D is laid)."""
import math, sys
HX, HY, HH, WT = 36000, 14000, 6000, 100
ROWS = {'A': 2350, 'B': 7000, 'C': 11650}
ROW_D = 16300
SP_X0, SP_X1 = 1500, 34500           # cross-spines
BAY0, PITCH, NB = 3000, 500, 60
LANE, WALK = 1000, 700
MATS = '''[material "floor" 0.80 0.80 0.78 0.0 0.9]
[material "track" 0.10 0.10 0.12 0.0 0.9]
[material "bus" 1.0 0.45 0.0 0.0 0.5]
[material "wall" 0.55 0.55 0.58 0.0 0.8]
[material "curtain" 0.95 0.25 0.20 0.0 0.4]
[material "mill" 0.30 0.45 0.62 0.6 0.5]
[material "saw" 0.70 0.30 0.25 0.5 0.5]
[material "router" 0.40 0.58 0.42 0.4 0.5]
[material "reflow" 0.78 0.62 0.25 0.5 0.5]
[material "rack" 0.55 0.55 0.58 0.7 0.5]
[material "winder" 0.55 0.40 0.62 0.5 0.5]
[material "stock" 0.62 0.50 0.36 0.0 0.8]
[material "bench" 0.62 0.48 0.32 0.0 0.8]
[material "arm" 0.15 0.15 0.17 0.6 0.5]
[material "plinth" 0.70 0.72 0.75 0.9 0.4]
[material "tender" 1.0 0.85 0.0 0.0 0.5]
[material "empty" 0.20 0.85 0.30 0.0 0.9]
[material "pin" 0.20 0.20 0.22 0.9 0.3]
[material "human" 0.20 0.35 0.75 0.0 0.7]
[material "tag" 0.5 0.5 0.5 0.0 0.5]
'''
# machine sizes (x along spine, y depth into lane, z)
M = {'saw': (450, 800, 900), 'router': (480, 950, 900), 'mill': (450, 700, 1500), 'reflow': (450, 800, 700),
     'rack': (450, 600, 1800), 'winder': (400, 600, 800), 'stock': (450, 900, 1200)}
NAMES = {'saw': 'saw', 'router': 'sheet router', 'mill': 'mill', 'reflow': 'reflow', 'rack': 'tray rack', 'winder': 'winder', 'stock': 'stock'}

def plan():
    """bay map: (row, side, i) -> kind. side +1 = +Y lane. benches occupy 3 bays: key 'bench' at first i, 'benchX' at others."""
    mix = ['mill', 'mill', 'mill', 'rack', 'mill', 'saw', 'router', 'mill', 'reflow', 'rack', 'winder', 'stock', 'mill', 'rack']
    B = {}
    k = 0
    for r in ROWS:
        for s in (-1, 1):
            for i in range(NB):
                B[(r, s, i)] = mix[k % len(mix)]; k += 1
    benches = [('A', -1, 6), ('A', 1, 30), ('A', -1, 48), ('B', -1, 12), ('B', 1, 40), ('B', 1, 3),
               ('C', 1, 9), ('C', -1, 27), ('C', 1, 51), ('C', -1, 42)]
    for (r, s, i) in benches:
        B[(r, s, i)] = 'bench'; B[(r, s, i + 1)] = B[(r, s, i + 2)] = 'benchX'
    # 15 empties: 3 bench-sized triples + 6 singles
    triples = [('A', 1, 15), ('B', -1, 27), ('C', 1, 33)]
    singles = [('A', -1, 22), ('A', 1, 55), ('B', 1, 20), ('B', -1, 50), ('C', -1, 14), ('C', 1, 45)]
    return B, benches, triples, singles
B0, BENCHES, TRIPLES, SINGLES = plan()
FILL = {('A', -1, 22): 'mill', ('A', 1, 55): 'rack', ('B', 1, 20): 'mill', ('B', -1, 50): 'winder', ('C', -1, 14): 'saw', ('C', 1, 45): 'mill'}

def bay_origin(r, s, i):
    y0 = ROWS[r] + WALK / 2 if s > 0 else ROWS[r] - WALK / 2 - LANE
    return BAY0 + PITCH * i, y0

class Doc:
    def __init__(s): s.parts = {}; s.inst = []; s.tags = []; s.bb = []
    def bounds(s):
        b = s.bb; return (tuple(min(q[i] for q in b) for i in range(3)), tuple(max(q[i + 3] for q in b) for i in range(3)))
    def part(s, name, expr, mat): s.parts.setdefault(name, '[part "%s" %s "%s"]' % (name, expr, mat))
    cut = ()
    def box(s, iid, sx, sy, sz, x, y, z, mat):
        if any(iid.startswith(c) for c in s.cut): return
        s.bb.append((x, y, z, x + sx, y + sy, z + sz))
        p = 'b_%d_%d_%d_%s' % (sx, sy, sz, mat); s.part(p, '[cube %d %d %d]' % (sx, sy, sz), mat); s.inst.append((iid, p, x, y, z))
    def cyl(s, iid, r, h, x, y, z, mat, ry=0):
        p = 'c_%d_%d_%d_%s' % (r, h, ry, mat)
        e = '[cylinder %d %d]' % (r, h) if not ry else '[rotate 0 %d 0 [cylinder %d %d]]' % (ry, r, h)
        s.part(p, e, mat); s.inst.append((iid, p, x, y, z))
    def tag(s, label, x, y, z): s.tags.append((label, x, y, z))
    def loon(s, tags_only=False, ymax=HY):
        if tags_only:
            parts = ['[part "t" [cube 2 2 2] "tag"]']
            inst = ['[instance "%s" "t" %g %g %g]' % (l, x - 1, y - 1, z - 2) for l, x, y, z in s.tags]
            inst += ['[instance "." "t" 0 0 0]', '[instance ".." "t" %d %d %d]' % (HX - 2, ymax - 2, HH - 2)]
        else:
            parts = list(s.parts.values())
            inst = ['[instance "%s" "%s" %g %g %g]' % i for i in s.inst]
        ground = inst[0].split('"')[1]
        return MATS + '[assembly\n #[%s]\n #[%s]\n #[]\n "%s"]\n' % ('\n   '.join(parts), '\n   '.join(inst), ground)

def spine_x(d, name, y, x0, x1, bus=True):
    L = int(x1 - x0)
    d.box(name + '_track', L, WALK, 20, x0, y - WALK / 2, 0, 'track')
    d.box(name + '_rail_a', L, 40, 40, x0, y - 170, 20, 'pin'); d.box(name + '_rail_b', L, 40, 40, x0, y + 130, 20, 'pin')
    if bus: d.box(name + '_bus', L, 30, 120, x0, y + 300, 300, 'bus')
def spine_y(d, name, x, y0, y1):
    L = int(y1 - y0)
    d.box(name + '_track', WALK, L, 20, x - WALK / 2, y0, 0, 'track')
    d.box(name + '_rail_a', 40, L, 40, x - 170, y0, 20, 'pin'); d.box(name + '_rail_b', 40, L, 40, x + 130, y0, 20, 'pin')
    d.box(name + '_bus', 30, L, 120, x + 300, y0, 300, 'bus')

def machine(d, iid, kind, x0, y0, s):
    sx, sy, sz = M[kind]
    x = x0 + (PITCH - sx) / 2; y = y0 + (LANE - sy if s > 0 else 0) if False else (y0 + 30 if s > 0 else y0 + LANE - 30 - sy)
    d.box(iid, sx, sy, sz, x, y, 0, kind)
    return x + sx / 2, y + sy / 2, sz
def bench(d, iid, x0, y0, s, z=0, along_y=False):
    if along_y: lx, ly = 1000, 1500
    else: lx, ly = 1500, 1000
    d.box(iid, lx, ly, 1000, x0, y0, z, 'bench')
    cx, cy = x0 + lx / 2, y0 + ly / 2
    d.box(iid + '_plinth', 520, 320, 10, cx - 260, cy - 160, z + 1000, 'plinth')
    for k, o in ((0, -200), (1, 200)):
        ax, ay = (cx + o, cy) if not along_y else (cx, cy + o)
        d.box(iid + '_arm%d' % k, 110, 110, 550, ax - 55, ay - 55, z + 1010, 'arm')
    return cx, cy, z + 1000
def empty(d, iid, x0, y0, s):
    d.box(iid, 480, 950, 10, x0 + 10, y0 + 25, 0, 'empty')
    yc = y0 + LANE / 2
    d.cyl(iid + '_pin_a', 12, 60, x0 + 100, yc, 10, 'pin'); d.cyl(iid + '_pin_b', 12, 60, x0 + 400, yc, 10, 'pin')
    ys = y0 - 0 if s > 0 else y0 + LANE - 80     # coupler on the spine edge
    d.box(iid + '_coupler', 100, 80, 120, x0 + 200, ys, 10, 'bus')
    return x0 + 250, yc, 10
def tender(d, iid, x, y, lift=400, along_y=False, bench_on=None):
    # plinth on the rails, lift column, two angled arms (300 upper + 250 fore)
    d.box(iid, 520 if not along_y else 320, 320 if not along_y else 520, 10, x - (260 if not along_y else 160), y - (160 if not along_y else 260), 60, 'tender')
    d.box(iid + '_lift', 200, 200, lift, x - 100, y - 100, 70, 'tender')
    top = 70 + lift
    if bench_on:
        return bench(d, bench_on, x - 500, y - 750, 0, z=top, along_y=True) if along_y else bench(d, bench_on, x - 750, y - 500, 0, z=top)
    d.cyl(iid + '_arm_l', 35, 300, x - 60, y, top, 'arm', ry=-35); d.cyl(iid + '_arm_r', 35, 300, x + 60, y, top, 'arm', ry=35)
    d.cyl(iid + '_fore_l', 30, 250, x - 60 - 172, y, top + 246, 'arm', ry=40); d.cyl(iid + '_fore_r', 30, 250, x + 60 + 172, y, top + 246, 'arm', ry=-40)
    return x, y, top

LT = {'saw': 4, 'router': 15, 'mill': 26, 'reflow': 31, 'rack': 40, 'winder': 47, 'stock': 54}
TENDERS = [('A', 4200), ('A', 14000), ('A', 26500), ('B', 9000), ('B', 18500), ('B', 30500), ('C', 6000), ('C', 16500), ('C', 24000), ('C', 32000), ('B', 2600)]

def build(empties=None, fill=None, row_d=0.0, carry=None, show_static_carry=True, labels=True, cut=(), stake=None):
    """empties: set of slot keys still empty (default all 15). fill: dict slot->kind for installed singles; triples filled -> bench.
    row_d: 0..1 fraction of row D spine laid (0 = none). carry: (x, y, along_y, payload kind) moving tender."""
    d = Doc(); d.cut = cut
    ymax = HY if row_d <= 0 else ROW_D + 2350
    if stake: d.box('stake', 2, 2, 2, HX - 2, stake - 2, 0, 'tag')
    # membrane
    d.box('wall_s', HX, WT, HH, 0, 0, 0, 'wall')
    d.box('wall_n', HX, WT, HH, 0, ymax - WT, 0, 'wall')
    for side, x in (('w', 0), ('e', HX - WT)):
        d.box('wall_%s_1' % side, WT, 6000 - WT, HH, x, WT, 0, 'wall')          # Y 100..6000
        d.box('wall_%s_2' % side, WT, ymax - WT - 8000, HH, x, 8000, 0, 'wall')  # Y 8000..ymax-100 (door 6000..8000)
        d.box('curtain_%s' % side, 40, 2000, 2400, x + (WT if x == 0 else -40), 6000, 0, 'curtain')
        d.box('door_head_%s' % side, WT, 2000, 1500, x, 6000, 4500, 'wall')
    if labels:
        d.tag('dock door (W)', 50, 7000, 4500); d.tag('dock door (E)', HX - 50, 7000, 4500)
    # spines + loop
    for r, y in ROWS.items():
        spine_x(d, 'spine_' + r, y, SP_X0 - WALK / 2, SP_X1 + WALK / 2)
        if labels: d.tag('row %s' % r, 2400, y, 300)
    for nm, x in (('xspine_w', SP_X0), ('xspine_e', SP_X1)):
        spine_y(d, nm, x, ROWS['A'] + WALK / 2, ROWS['C'] - WALK / 2)
    spine_x(d, 'dock_spur_w', 7000, WT, SP_X0 - WALK / 2, bus=False)
    spine_x(d, 'dock_spur_e', 7000, SP_X1 + WALK / 2, HX - WT, bus=False)
    if row_d > 0:
        L = (SP_X1 - SP_X0 + WALK) * row_d
        spine_x(d, 'spine_D', ROW_D, SP_X0 - WALK / 2, SP_X0 - WALK / 2 + max(L, 50))
        spine_y(d, 'xspine_w_D', SP_X0, ROWS['C'] + WALK / 2, ROW_D - WALK / 2)
        if row_d >= 1: spine_y(d, 'xspine_e_D', SP_X1, ROWS['C'] + WALK / 2, ROW_D - WALK / 2)
        if labels: d.tag('row D (being laid)', 2400, ROW_D, 300)
    if empties is None: empties = set(TRIPLES) | set(SINGLES)
    fill = fill or {}
    seen = set(); labelled = set(); ne = 0
    for r in ROWS:
        for s in (-1, 1):
            for i in range(NB):
                key = (r, s, i); x0, y0 = bay_origin(r, s, i)
                tr = next((t for t in TRIPLES if t[:2] == key[:2] and t[2] <= i < t[2] + 3), None)
                iid = 'bay_%s%s%02d' % (r, 'n' if s > 0 else 's', i)
                if tr is not None:
                    if tr in empties:
                        p = empty(d, iid, x0, y0, s); ne += 1
                        if labels and i == tr[2] + 1: d.tag('empty bays (bench slot)', *p)
                    elif i == tr[2]:
                        bench(d, iid, x0, y0 + (30 if s > 0 else -30), s)
                    continue
                if key in SINGLES:
                    if key in empties:
                        p = empty(d, iid, x0, y0, s); ne += 1
                        if labels and key in (('A', -1, 22), ('B', 1, 20), ('C', 1, 45)): d.tag('empty bay', *p)
                    elif key in fill:
                        machine(d, iid, fill[key], x0, y0, s)
                    continue
                k = B0[key]
                if k == 'benchX': continue
                if k == 'bench':
                    p = bench(d, iid, x0, y0 + (30 if s > 0 else -30), s)
                    if labels and key == ('B', -1, 12): d.tag('bench (2 arms)', *p)
                    continue
                p = machine(d, iid, k, x0, y0, s)
                if labels and k not in labelled and r == 'B' and s > 0 and i >= LT.get(k, 99):
                    labelled.add(k); d.tag(NAMES[k], *p)
    # tenders
    for n, (r, x) in enumerate(TENDERS, 1):
        tender(d, 'tender_%02d' % n, x, ROWS[r], lift=350 + 60 * (n % 4))
    if show_static_carry:
        p = tender(d, 'tender_12', SP_X1, 4700, lift=600, along_y=True, bench_on='carried_bench')
        if labels: d.tag('carried_bench on tender_12', *p)
    if carry:
        x, y, ay, kind = carry
        if kind == 'bench': tender(d, 'tender_c', x, y, lift=200, along_y=ay, bench_on='new_bench')
        else:
            tender(d, 'tender_c', x, y, lift=150, along_y=ay)
            sx, sy, sz = M[kind]; d.box('new_' + kind, sx, sy, sz, x - sx / 2, y - sy / 2, 230, kind)
    # human at the west dock door (inside, beside the spur)
    hx, hy = 700, 8300
    d.cyl('human_leg_l', 70, 850, hx, hy - 90, 0, 'human'); d.cyl('human_leg_r', 70, 850, hx, hy + 90, 0, 'human')
    d.box('human_torso', 260, 440, 620, hx - 130, hy - 220, 850, 'human')
    d.cyl('human_neck', 50, 60, hx, hy, 1470, 'human'); d.cyl('human', 105, 220, hx, hy, 1530, 'human')
    if labels: d.tag('human 1.75 m', hx, hy, 1750)
    return d, ymax, ne

if __name__ == '__main__':
    import json
    d, ymax, ne = build()
    open(sys.argv[1], 'w').write(d.loon(ymax=ymax)); open(sys.argv[2], 'w').write(d.loon(True, ymax))
    print('instances', len(d.inst), 'parts', len(d.parts), 'empties', ne, file=sys.stderr)
