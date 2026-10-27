"""Generate cell_v2.loon: 4040 cell, two rose fork-v6 arms STANDING on a plinth that slides on two SBR rails on
the deck. Arm geometry = rose_fork.py build() (v6), turned +90 deg about Z so the zero pose points +Y.
Parameters live in cellp.py; loon `let` geometry needs literal numbers, so they are baked here.
Run: python3 gen_cell_v2.py [--no-panels] > cell_v2.loon"""
import sys, math
FORK = __import__('os').path.join(__import__('os').environ.get('ROSE_DIR', __import__('os').path.expanduser('~/Developer/rose')), 'arm', 'fork')
sys.path.insert(0, FORK)
import rose_fork as F
from rose_arm_geom import prim_loon, sub, mul, add, unit
from cellp import *
PANELS = '--no-panels' not in sys.argv
LIM = {'J1_yaw': (-170, 170), 'J2_sho_pitch': (-150, 120), 'J4_elbow': (-170, 150),
       'J5_fore_roll': (-170, 170), 'J6_wrist_pitch': (-150, 150), 'J7_tool_roll': (-180, 180)}
rz = lambda v: (-v[1], v[0], v[2])             # +90 deg about Z: arm zero pose points +Y
cube = lambda x, y, z, sx, sy, sz: '[translate %g %g %g [cube %g %g %g]]' % (x, y, z, sx, sy, sz)
def U(items):                                   # loon union is binary
    s = items[-1]
    for it in reversed(items[:-1]): s = '[union %s %s]' % (it, s)
    return s
def link_prims(link):
    O = F.ORIGIN[link]; out = []
    for p in F.build():
        if p[0] != link: continue
        _, mat, o, u, v, ol, vec = p[:7]
        holes = p[7] if len(p) > 7 else []
        o2, u2, v2, w2 = rz(sub(o, O)), rz(u), rz(v), rz(vec)
        body = prim_loon((link, mat, o2, u2, v2, ol, w2))
        for h in holes:
            n = unit(w2)
            cut = prim_loon((link, mat, sub(o2, mul(n, 0.001)), u2, v2, h, add(w2, mul(n, 0.002))))
            body = '[difference %s %s]' % (cut, body)
        out.append((mat, body))
    return out
L_, D_, H_ = cell_l, cell_d, cell_h
dz0 = deck_z - deck_t
ry = (aisle_y - rail_gap // 2, aisle_y + rail_gap // 2)
# ---- fixtures: name -> (x, y, z, sx, sy, sz); all on the deck top unless noted
FIX = {
    'mill':     (80, 700, deck_z, 400, 350, 380),     # door face Y=700; NOTE overhangs the back frame (Y 1000) by 50
    'rack':     (600, 700, deck_z, 300, 150, 300),
    'reflow':   (1020, 700, deck_z, 300, 300, 200),   # door face Y=700, back flush with the frame's outer face
    'stock':    (80, 100, deck_z, 300, 200, 120),     # front of drawer at Y=300
    'bench':    (550, 40, deck_z, 400, 260, 40),
    'winder':   (1020, 100, deck_z, 200, 200, 150),
    'finished': (1260, 40, deck_z, 200, 300, 120),    # 300 deep: Y 40..340, stops 10 short of the aisle
    'psu':      (200, 200, 46, 200, 100, 50),          # electronics bay on the floor plate under the deck
    'canhub':   (500, 200, 46, 100, 60, 30),
    'controller': (700, 200, 46, 150, 100, 40),
    'cam':      (725, 530, 920, 50, 50, 40),           # under the top cross-bar, looking down
    'light':    (50, 470, 940, 1400, 30, 20),
}
MAT = {'mill': 'fixture', 'rack': 'fixture', 'reflow': 'fixture', 'stock': 'fixture', 'bench': 'bench', 'winder': 'fixture',
       'finished': 'fixture', 'psu': 'elec', 'canhub': 'elec', 'controller': 'elec', 'cam': 'elec', 'light': 'light'}
def frame_solid():
    b = []
    for y in (0, D_ - 40):
        for z in (0, H_ - 40): b.append(cube(0, y, z, L_, 40, 40))
        b.append(cube(40, y + (40 if y == 0 else -40), dz0 - 40, L_ - 80, 40, 40))   # deck support rails, inside the posts
    for x in (0, L_ - 40):
        for z in (0, dz0 - 40, H_ - 40): b.append(cube(x, 40, z, 40, D_ - 80, 40))
        for y in (0, D_ - 40): b.append(cube(x, y, 40, 40, 40, H_ - 80))
    b.append(cube(L_ / 2 - 20, 40, H_ - 40, 40, D_ - 80, 40))                         # top cross-bar (camera, light)
    return U(b)
def chain():
    segs = [cube(x, aisle_y - 20, deck_z, 30, 40, 28) for x in range(60, 60 + 22 * 34, 34)]
    return U(segs)
def plinth():
    hx = block_pitch_x / 2; hy = rail_gap / 2
    b = [cube(-plinth_l / 2, -plinth_w / 2, plinth_h - plinth_t, plinth_l, plinth_w, plinth_t)]
    for sx in (-1, 1):
        for sy in (-1, 1):
            b.append(cube(sx * hx - block_l / 2, sy * hy - block_w / 2, 18, block_l, block_w, plinth_h - plinth_t - 18))
    return U(b)
def rail():   # SBR16: 40 wide aluminium support + 16 mm rod, rod centre 25 above the deck
    return '[union %s [translate 0 20 25 [rotate 0 90 0 [cylinder 8 %g]]]]' % (cube(0, 0, 0, rail_l, 40, 17), rail_l)
def panels():
    t = 6; P = {}
    P['panel_l'] = cube(-t, 0, 0, t, D_, H_); P['panel_r'] = cube(L_, 0, 0, t, D_, H_)
    P['panel_top'] = cube(-t, -t, H_, L_ + 2 * t, D_ + 2 * t, t)
    P['panel_door_l'] = cube(0, -t, 0, L_ / 2 - 1, t, H_); P['panel_door_r'] = cube(L_ / 2 + 1, -t, 0, L_ / 2 - 1, t, H_)
    mx0, mx1, mz1 = 80, 480, deck_z + 380                                   # cut-out where the mill pokes through
    P['panel_back'] = U([cube(0, D_, 0, mx0, t, H_), cube(mx1, D_, 0, L_ - mx1, t, H_),
                         cube(mx0, D_, 0, mx1 - mx0, t, deck_z), cube(mx0, D_, mz1, mx1 - mx0, t, H_ - mz1)])
    return P
def main():
    L = [';; cell_v2.loon - GENERATED by gen_cell_v2.py (params in cellp.py). Z-up, mm.',
         ';; %dx%dx%d 4040 cell. Deck top z=%d (%d mm plate), electronics bay under it.' % (L_, D_, H_, deck_z, deck_t),
         ';; Two rose fork-v6 arms stand on a plinth riding two SBR16 rails at Y=%d/%d (rail %d mm, X %d..%d).' % (ry[0], ry[1], rail_l, rail_x0, rail_x0 + rail_l),
         ';; carriage_x q = 0..%d, plinth centre X = %d + q. Feet %d apart, shoulder axes z=%d.' % (travel, car_x0, arm_pitch, sh_z),
         ';; Arm joints use rose_fork.py signs: +pitch swings the distal link DOWN; zero pose = arm horizontal along +Y.']
    for k, v in [('cell_l', L_), ('cell_d', D_), ('cell_h', H_), ('deck_z', deck_z), ('aisle_y', aisle_y), ('rail_gap', rail_gap),
                 ('arm_pitch', arm_pitch), ('plinth_h', plinth_h)]:
        L.append('[defparam %s %g]' % (k, v))
    L += ['[material "alu" 0.78 0.78 0.80 0.9 0.4]', '[material "deck" 0.55 0.52 0.48 0.0 0.8]',
          '[material "rail" 0.25 0.25 0.27 0.8 0.3]', '[material "plinth" 0.70 0.72 0.75 0.9 0.4]',
          '[material "fixture" 0.85 0.45 0.2 0.2 0.6]', '[material "bench" 0.62 0.48 0.32 0.0 0.8]',
          '[material "elec" 0.20 0.45 0.30 0.2 0.6]', '[material "light" 0.98 0.96 0.80 0.0 0.3]',
          '[material "chain" 0.12 0.12 0.12 0.0 0.7]', '[material "poly" 0.86 0.92 0.97 0.0 0.1]']
    L.append(F.MATERIALS.strip())
    parts, inst, joints = [], [], []
    def add_part(name, solid, mat, parent='frame', at=(0, 0, 0)):
        parts.append('[part "%s" %s "%s"]' % (name, solid, mat)); inst.append('[instance "%s" "%s" 0 0 0]' % (name, name))
        joints.append('[fixed-joint "%s_mount" "%s" %g %g %g "%s" 0 0 0]' % (name, parent, at[0], at[1], at[2], name))
    parts.append('[part "frame" %s "alu"]' % frame_solid()); inst.append('[instance "frame" "frame" 0 0 0]')
    add_part('deck', cube(0, 0, 0, L_ - 80, D_ - 80, deck_t), 'deck', at=(40, 40, dz0))
    add_part('floor', cube(0, 0, 0, L_ - 80, D_ - 80, 6), 'deck', at=(40, 40, 40))
    add_part('rail_f', rail(), 'rail', at=(rail_x0, ry[0] - 20, deck_z))
    add_part('rail_b', rail(), 'rail', at=(rail_x0, ry[1] - 20, deck_z))
    add_part('chain', chain(), 'chain')
    for n, (x, y, z, sx, sy, sz) in FIX.items():
        add_part(n, cube(0, 0, 0, sx, sy, sz), MAT[n], at=(x, y, z))
    if PANELS:
        for n, s in panels().items(): add_part(n, s, 'poly')
    parts.append('[part "plinth" %s "plinth"]' % plinth()); inst.append('[instance "plinth" "plinth" 0 0 0]')
    joints.append('[prismatic-joint "carriage_x" 1 0 0 0 %d "frame" %g %g %g "plinth" 0 0 0]' % (travel, car_x0, aisle_y, deck_z))
    defs = {}
    for link, *_ in F.LINKS:
        defs[link] = link_prims(link)
        for k, (mat, body) in enumerate(defs[link]):
            parts.append('[part "%s_%d" %s "%s"]' % (link, k, body, mat))
    for side, dx in (('l', -arm_pitch / 2), ('r', arm_pitch / 2)):
        for link, joint, parent, axis, o in F.LINKS:
            lead = '%s_%s' % (side, link)
            for k in range(len(defs[link])):
                iid = lead if k == 0 else '%s_%d' % (lead, k)
                inst.append('[instance "%s" "%s_%d" 0 0 0]' % (iid, link, k))
                if k: joints.append('[fixed-joint "%s_w%d" "%s" 0 0 0 "%s" 0 0 0]' % (lead, k, lead, iid))
            if parent is None:
                joints.append('[fixed-joint "arm_%s_mount" "plinth" %g 0 %g "%s" 0 0 0]' % (side, dx, plinth_h, lead)); continue
            a = rz(axis); pa = rz(sub(o, F.ORIGIN[parent])); lo, hi = LIM[joint]
            joints.append('[revolute-joint "arm_%s_%s" %g %g %g %d %d "%s_%s" %.3f %.3f %.3f "%s" 0 0 0]' % (
                side, joint, a[0] + 0.0, a[1] + 0.0, a[2] + 0.0, lo, hi, side, parent, pa[0] * 1e3, pa[1] * 1e3, pa[2] * 1e3, lead))
    L.append('[assembly\n  #[' + '\n    '.join(parts) + ']\n  #[' + '\n    '.join(inst) + ']\n  #[' + '\n    '.join(joints) + ']\n  "frame"]')
    print('\n'.join(L))
main()
