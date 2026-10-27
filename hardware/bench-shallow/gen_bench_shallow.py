"""Generate bench_shallow.loon: the real 1500x500x1000 4040 box as the SPINE module (deck, two SBR16 rails, two rose
fork-v9 arms on a sliding plinth, shallow back-wall racks) with three 500x1000 bays docked to its open front.
Arm geometry = rose_fork.py build() in its native frame (zero pose along +X, the spine). Params: bsp.py.
Run: python3 gen_bench_shallow.py [--no-panels] > bench_shallow.loon"""
import sys, math
FORK = __import__('os').path.join(__import__('os').environ.get('ROSE_DIR', __import__('os').path.expanduser('~/Developer/rose')), 'arm', 'fork')
sys.path.insert(0, FORK)
import rose_fork as F
from rose_arm_geom import prim_loon, sub, mul, add, unit
from bsp import *
PANELS = '--no-panels' not in sys.argv
LIM = {'J1_yaw': (-170, 170), 'J2_sho_pitch': (-150, 120), 'J4_elbow': (-170, 150),
       'J5_fore_roll': (-150, 150), 'J6_wrist_pitch': (-150, 150), 'J7_tool_roll': (-180, 180)}
cube = lambda x, y, z, sx, sy, sz: '[translate %g %g %g [cube %g %g %g]]' % (x, y, z, sx, sy, sz)
cyl = lambda x, y, z, r, h: '[translate %g %g %g [cylinder %g %g]]' % (x, y, z, r, h)
cylY = lambda x, y, z, r, h: '[translate %g %g %g [rotate -90 0 0 [cylinder %g %g]]]' % (x, y, z, r, h)   # axis +Y from (x,y,z)
def U(items):                                   # loon union is binary
    s = items[-1]
    for it in reversed(items[:-1]): s = '[union %s %s]' % (it, s)
    return s
def link_prims(link):
    O = F.ORIGIN[link]; out = []
    for i, p in enumerate([p for p in F.build() if p[0] == link]):
        if link == 'tool' and i > 0: continue      # drop rose's EDU jaws: gripper v2 replaces them
        _, mat, o, u, v, ol, vec = p[:7]
        holes = p[7] if len(p) > 7 else []
        o2 = sub(o, O)
        body = prim_loon((link, mat, o2, u, v, ol, vec))
        for h in holes:
            n = unit(vec)
            cut = prim_loon((link, mat, sub(o2, mul(n, 0.001)), u, v, h, add(vec, mul(n, 0.002))))
            body = '[difference %s %s]' % (cut, body)
        out.append((mat, body))
    # gripper v2 (485 g): 70x56 body x 135 along the tool axis from the flange face + two 20x80 fingers (40 inside)
    if link == 'tool':
        f0 = FLANGE - 520.0
        out.append(('gripper', cube(f0, -35, -28, GRIP_L, 70, 56)))
        fx = f0 + GRIP_L - FING_OVL
        out.append(('finger', U([cube(fx, 12, -10, FING_L, 4, 20), cube(fx, -16, -10, FING_L, 4, 20)])))
    return out
W, D, H = box_w, box_d, box_h
def frame_solid():
    b = []
    for y in (0, D - ext):
        for z in (0, H - ext): b.append(cube(0, y, z, W, ext, ext))
    for x in (0, W - ext):
        for z in (0, H - ext): b.append(cube(x, ext, z, ext, D - 2 * ext, ext))
        for y in (0, D - ext): b.append(cube(x, y, ext, ext, ext, H - 2 * ext))
    b.append(cube(ext, D - ext, deck_z - deck_t - ext, W - 2 * ext, ext, ext))      # deck carrier, back
    b.append(cube(ext, 0, deck_z - deck_t - ext, W - 2 * ext, ext, ext))            # deck carrier, front
    b.append(cube(W / 2 - 20, ext, H - ext, ext, D - 2 * ext, ext))                 # top cross-bar (camera)
    return U(b)
def rail():   # SBR16: 40 wide support + 16 mm rod, rod centre 25 above the deck
    return '[union %s [translate 0 20 25 [rotate 0 90 0 [cylinder 8 %g]]]]' % (cube(0, 0, 0, rail_l, 40, 17), rail_l)
def plinth():
    hx, hy = block_pitch_x / 2, rail_gap / 2
    b = [cube(-plinth_l / 2, -plinth_w / 2, plinth_h - plinth_t, plinth_l, plinth_w, plinth_t)]
    for sx in (-1, 1):
        for sy in (-1, 1):
            b.append(cube(sx * hx - block_l / 2, sy * hy - block_w / 2, 18, block_l, block_w, plinth_h - plinth_t - 18))
    return U(b)
def foot_marks():
    """per foot: dark Ø120 disc (screw heads / harness clearance) + 4x Ø6.6 on Ø190 at 45/135/225/315, on the plate top."""
    b = []
    for fx in (-arm_pitch / 2, arm_pitch / 2):
        b.append(cyl(fx, 0, 0, 60, 0.5))
        for a in (45, 135, 225, 315):
            b.append(cyl(fx + 95 * math.cos(math.radians(a)), 95 * math.sin(math.radians(a)), 0, 3.3, 0.8))
    return U(b)
def busbar(): return cube(rail_x0, plinth_y - 10, deck_z, rail_l, 20, 12)
def chain():  # cable chain along the front rail, outside it (Y 95..125)
    return U([cube(x, 95, deck_z, 30, 30, 28) for x in range(int(rail_x0), int(rail_x0) + 20 * 34, 34)])
# ---- racks on the back wall (Y 380..460, above the plinth/arm-base envelope z>280)
SHELVES = [300, 450, 600, 750]
def parts_rack():
    b = [cube(0, 0, 0, 1400, 80, 4)]  # back sheet is the panel; rack = shelves + uprights
    b = [cube(0, 0, 0, 20, 80, 600), cube(1380, 0, 0, 20, 80, 600), cube(690, 0, 0, 20, 80, 600)]
    for z in SHELVES: b += [cube(0, 0, z - 280, 1400, 80, 6), cube(0, 0, z - 280, 1400, 6, 40)]   # shelf + front lip
    return U(b)
TIP_PEGS = [(110, 560), (170, 620), (230, 680), (290, 740)]        # (X, z) : 4 tip pairs, pegs out to Y 340
def tip_rack():
    b = [cube(0, 0, 0, 260, 10, 300)]                                  # plate X 70..330, Y 370..380, z 500..800
    for x, z in TIP_PEGS:
        b.append(cylY(x - 70, -30, z - 500, 5, 30))                    # peg Y 340..370
        b.append(cube(x - 70 - 12, -30, z - 500 - 40, 24, 22, 30))     # parked tip pair hanging on the peg
    return U(b)
def tool_strip(): return U([cube(0, 0, 0, 240, 10, 240)] + [cube(20 + 50 * k, -25, 180 - 30 * k, 12, 25, 12) for k in range(4)])
# ---- bays on the table in front (Y -1000..0), each 500 wide, open face toward the box
def bay_common(i):
    x0 = 500 * i; b = []
    b.append(('bay%d_plate' % (i + 1), cube(x0 + 5, -1000, 0, 490, 1000, 10), 'deck'))
    b.append(('bay%d_dock' % (i + 1), cube(x0 + 10, -40, 10, 480, 40, 210), 'alu'))      # dock beam, face at Y 0
    b.append(('bay%d_pins' % (i + 1), U([cylY(x0 + 250 + s * 200, 0, 200, 6, 25) for s in (-1, 1)]), 'pin'))
    b.append(('bay%d_coupler' % (i + 1), cube(x0 + 190, -60, 170, 120, 60, 60), 'coupler'))
    return b
def bays():
    P = []
    for i in range(3): P += bay_common(i)
    # bay_1: Carvera Air on a stand, door (front) facing +Y toward the box
    P += [('mill_stand', cube(50, -420, 10, 400, 370, 190), 'alu'), ('mill', cube(50, -400, 200, 400, 350, 380), 'fixture')]
    # bay_2: bench surface 460x600x40, top at z 200; reflow at its far end, door facing +Y
    P += [('bench', cube(520, -640, 160, 460, 600, 40), 'bench'), ('bench_legs', U([cube(x, y, 10, 40, 40, 150) for x in (530, 930) for y in (-630, -90)]), 'alu'),
          ('reflow', cube(600, -640, 200, 300, 300, 200), 'fixture')]
    # bay_3: shelf at z 200 with coil winder + stock drawer + finished drawer
    P += [('bay3_shelf', cube(1020, -680, 160, 460, 640, 40), 'bench'), ('bay3_legs', U([cube(x, y, 10, 40, 40, 150) for x in (1030, 1430) for y in (-670, -90)]), 'alu'),
          ('winder', cube(1260, -260, 200, 200, 200, 150), 'fixture'),
          ('stock', cube(1030, -360, 200, 200, 300, 120), 'fixture'), ('finished', cube(1030, -670, 200, 200, 300, 120), 'fixture')]
    return P
def human():   # 1750 tall, feet on the floor at z = -table_h, facing +Y
    z0 = -table_h
    return U([cube(-90, -60, z0, 70, 120, 820), cube(20, -60, z0, 70, 120, 820), cube(-110, -75, z0 + 820, 220, 150, 600),
              cube(-160, -50, z0 + 870, 45, 100, 530), cube(115, -50, z0 + 870, 45, 100, 530), cyl(0, 0, z0 + 1420, 45, 80),
              '[translate 0 0 %g [sphere 115]]' % (z0 + 1635)])
def table():
    z0 = -table_h
    return U([cube(-100, -1000, -40, 1700, 1500, 40)] + [cube(x, y, z0, 60, 60, table_h - 40) for x in (-80, 1520) for y in (-980, 420)])
def panels():
    t = 6
    return {'panel_l': cube(-t, 0, 0, t, D, H), 'panel_r': cube(W, 0, 0, t, D, H),
            'panel_top': cube(-t, 0, H, W + 2 * t, D + t, t), 'panel_back': cube(0, D, 0, W, t, H)}
def main():
    L = [';; bench_shallow.loon - GENERATED by gen_bench_shallow.py (params in bsp.py). Z-up, mm.',
         ';; Box %dx%dx%d 4040 = spine. Deck top z=%d. SBR16 rails Y=%d/%d (%d long). Plinth centre X = %d + carriage_x (0..%d).' % (
             W, D, H, deck_z, rail_y[0], rail_y[1], rail_l, car_x0, travel),
         ';; Arms: rose fork v9, native frame (zero pose along +X), shoulders z=%d. Bays on the table in front, Y -1000..0.' % sh_z]
    for k, v in [('box_w', W), ('box_d', D), ('box_h', H), ('rail_gap', rail_gap), ('arm_pitch', arm_pitch), ('deck_z', deck_z)]:
        L.append('[defparam %s %g]' % (k, v))
    L += ['[material "alu" 0.78 0.78 0.80 0.9 0.4]', '[material "deck" 0.55 0.52 0.48 0.0 0.8]',
          '[material "rail" 0.25 0.25 0.27 0.8 0.3]', '[material "plinth" 0.70 0.72 0.75 0.9 0.4]', '[material "hole" 0.06 0.06 0.07 0.0 0.9]',
          '[material "fixture" 0.85 0.45 0.2 0.2 0.6]', '[material "bench" 0.62 0.48 0.32 0.0 0.8]', '[material "rack" 0.35 0.55 0.75 0.3 0.5]',
          '[material "elec" 0.20 0.45 0.30 0.2 0.6]', '[material "light" 0.98 0.96 0.80 0.0 0.3]', '[material "pin" 0.9 0.9 0.2 0.8 0.3]',
          '[material "coupler" 0.8 0.2 0.2 0.3 0.5]', '[material "copper" 0.80 0.50 0.25 0.9 0.3]', '[material "gripper" 0.15 0.15 0.17 0.2 0.6]',
          '[material "finger" 0.75 0.75 0.78 0.9 0.4]', '[material "skin" 0.80 0.65 0.55 0.0 0.8]', '[material "table" 0.45 0.35 0.25 0.0 0.8]',
          '[material "chain" 0.12 0.12 0.12 0.0 0.7]', '[material "poly" 0.86 0.92 0.97 0.0 0.1]', '[material "curtain" 0.9 0.1 0.1 0.2 0.5]']
    L.append(F.MATERIALS.strip())
    parts, inst, joints = [], [], []
    def add_part(name, solid, mat, parent='frame', at=(0, 0, 0)):
        parts.append('[part "%s" %s "%s"]' % (name, solid, mat)); inst.append('[instance "%s" "%s" 0 0 0]' % (name, name))
        joints.append('[fixed-joint "%s_mount" "%s" %g %g %g "%s" 0 0 0]' % (name, parent, at[0], at[1], at[2], name))
    parts.append('[part "frame" %s "alu"]' % frame_solid()); inst.append('[instance "frame" "frame" 0 0 0]')
    add_part('deck', cube(ext, ext, deck_z - deck_t, W - 2 * ext, D - 2 * ext, deck_t), 'deck')
    add_part('rail_f', rail(), 'rail', at=(rail_x0, rail_y[0] - 20, deck_z))
    add_part('rail_b', rail(), 'rail', at=(rail_x0, rail_y[1] - 20, deck_z))
    add_part('busbar', busbar(), 'copper'); add_part('chain', chain(), 'chain')
    for n, (x, y, z, sx, sy, sz) in {'psu': (200, 200, 40, 200, 100, 50), 'canhub': (500, 220, 40, 100, 60, 30),
                                      'controller': (700, 200, 40, 150, 100, 40)}.items():
        add_part(n, cube(x, y, z, sx, sy, sz), 'elec')
    add_part('parts_rack', parts_rack(), 'rack', at=(50, 380, 280))
    add_part('tip_rack', tip_rack(), 'rack', at=(70, 370, 500))
    add_part('tool_strip', tool_strip(), 'rack', at=(1200, 370, 600))
    add_part('cam', cube(725, 230, 920, 50, 50, 40), 'elec')
    add_part('light', cube(50, 0, 940, 1400, 30, 20), 'light')
    add_part('curtain_l', cube(0, -30, 10, 30, 30, 980), 'curtain'); add_part('curtain_r', cube(W - 30, -30, 10, 30, 30, 980), 'curtain')
    add_part('table', table(), 'table')
    for n, s, m in bays(): add_part(n, s, m)
    add_part('human', human(), 'skin', at=(750, -1100, 0))
    if PANELS:
        for n, s in panels().items(): add_part(n, s, 'poly')
    parts.append('[part "plinth" %s "plinth"]' % plinth()); inst.append('[instance "plinth" "plinth" 0 0 0]')
    joints.append('[prismatic-joint "carriage_x" 1 0 0 0 %d "frame" %g %g %g "plinth" 0 0 0]' % (travel, car_x0, plinth_y, deck_z))
    parts.append('[part "foot_holes" %s "hole"]' % foot_marks()); inst.append('[instance "foot_holes" "foot_holes" 0 0 0]')
    joints.append('[fixed-joint "foot_holes_mount" "plinth" 0 0 %g "foot_holes" 0 0 0]' % plinth_h)
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
            pa = sub(o, F.ORIGIN[parent]); lo, hi = LIM[joint]
            joints.append('[revolute-joint "arm_%s_%s" %g %g %g %d %d "%s_%s" %.3f %.3f %.3f "%s" 0 0 0]' % (
                side, joint, axis[0], axis[1], axis[2], lo, hi, side, parent, pa[0] * 1e3, pa[1] * 1e3, pa[2] * 1e3, lead))
    L.append('[assembly\n  #[' + '\n    '.join(parts) + ']\n  #[' + '\n    '.join(inst) + ']\n  #[' + '\n    '.join(joints) + ']\n  "frame"]')
    print('\n'.join(L))
main()
