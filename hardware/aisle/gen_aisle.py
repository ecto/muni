"""Generate aisle.loon: 6 m spine, 24 bays, two tenders (plinth + lift + 2 rose fork-v6 arms), one cell_v2 bench in F3-F5.
Writes aisle.loon and aisle_tags.json (instance -> local label point). Run: python3 gen_aisle.py"""
import sys, json, math, os
P = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'bench')
FORK = __import__('os').path.join(__import__('os').environ.get('ROSE_DIR', __import__('os').path.expanduser('~/Developer/rose')), 'arm', 'fork')
S = os.path.dirname(os.path.abspath(__file__)); sys.path[:0] = [S, FORK, P]
import rose_fork as F
from rose_arm_geom import prim_loon, sub, mul, add, unit
from aislep import *
# reuse cell_v2 helpers (frame_solid, rail, plinth, chain, FIX, MAT) without running its main()
G = {'__name__': 'gcv2'}; sys.argv = [sys.argv[0], '--no-panels']
exec(open(P + '/gen_cell_v2.py').read().replace('\nmain()', '\n'), G)
import cellp as C
LIM = G['LIM']
cube = lambda x, y, z, sx, sy, sz: '[translate %g %g %g [cube %g %g %g]]' % (x, y, z, sx, sy, sz)
U = G['U']
rz = lambda v: (v[0], v[1], v[2])                  # tender arms: rose native, zero pose -> +X (along the spine: both lanes at J1 = +/-90)
rzb = lambda v: (v[1], -v[0], v[2])                # bench arms: cell_v2 rz then 180 deg -> zero pose -Y
parts, inst, joints, tags = [], [], [], {}
def add_part(name, solid, mat, parent='floor', at=(0, 0, 0), tag=None, bbox=None):
    parts.append('[part "%s" %s "%s"]' % (name, solid, mat)); inst.append('[instance "%s" "%s" 0 0 0]' % (name, name))
    joints.append('[fixed-joint "%s_mount" "%s" %g %g %g "%s" 0 0 0]' % (name, parent, at[0], at[1], at[2], name))
    if bbox: tag = (bbox[0] / 2, bbox[1] / 2, bbox[2])
    if tag: tags[name] = tag
def box(name, x, y, z, sx, sy, sz, mat):
    add_part(name, cube(0, 0, 0, sx, sy, sz), mat, at=(x, y, z), bbox=(sx, sy, sz))
# ---------------- link part defs (shared by all 6 arms; one variant per mounting rotation)
def link_prims(link, R):
    O = F.ORIGIN[link]; out = []
    for p in F.build():
        if p[0] != link: continue
        _, mat, o, u, v, ol, vec = p[:7]; holes = p[7] if len(p) > 7 else []
        o2, u2, v2, w2 = R(sub(o, O)), R(u), R(v), R(vec)
        body = prim_loon((link, mat, o2, u2, v2, ol, w2))
        for h in (holes if os.environ.get('HOLES') else []):
            n = unit(w2)
            body = '[difference %s %s]' % (prim_loon((link, mat, sub(o2, mul(n, 0.001)), u2, v2, h, add(w2, mul(n, 0.002)))), body)
        out.append((mat, body))
    return out
NPRIM = {}
for tagR, R in (('t', rz), ('b', rzb)):
    for link, *_ in F.LINKS:
        pr = link_prims(link, R); NPRIM[link] = len(pr)
        for k, (mat, body) in enumerate(pr): parts.append('[part "%s_%s_%d" %s "%s"]' % (tagR, link, k, body, mat))
def arm(pfx, jpfx, tagR, R, mount_parent, mount_at):
    """instances lk_<pfx>_<link>[_k], joints <jpfx>_<Jname>"""
    for link, joint, parent, axis, o in F.LINKS:
        lead = 'lk_%s_%s' % (pfx, link)
        for k in range(NPRIM[link]):
            iid = lead if k == 0 else '%s_%d' % (lead, k)
            inst.append('[instance "%s" "%s_%s_%d" 0 0 0]' % (iid, tagR, link, k))
            if k: joints.append('[fixed-joint "%s_w%d" "%s" 0 0 0 "%s" 0 0 0]' % (lead, k, lead, iid))
        if parent is None:
            joints.append('[fixed-joint "%s_mount" "%s" %g %g %g "%s" 0 0 0]' % (jpfx, mount_parent, *mount_at, lead)); continue
        a = R(axis); pa = R(sub(o, F.ORIGIN[parent])); lo, hi = LIM[joint]
        joints.append('[revolute-joint "%s_%s" %g %g %g %d %d "lk_%s_%s" %.3f %.3f %.3f "%s" 0 0 0]' % (
            jpfx, joint, a[0] + 0.0, a[1] + 0.0, a[2] + 0.0, lo, hi, pfx, parent, pa[0] * 1e3, pa[1] * 1e3, pa[2] * 1e3, lead))
# ---------------- floor (root) + spine
parts.append('[part "floor" %s "deck"]' % cube(0, 0, 0, AISLE_L, AISLE_D, FLOOR_T)); inst.append('[instance "floor" "floor" 0 0 0]')
tags['floor'] = (150, 150, FLOOR_T)
for n, y in (('spine_rail_f', RAIL_Y[0]), ('spine_rail_b', RAIL_Y[1])):
    add_part(n, cube(0, 0, 0, AISLE_L, 20, 17), 'rail', at=(0, y - 10, FLOOR_T), tag=(5600 if n.endswith('b') else 400, 10, 17))
add_part('spine_busbar', cube(0, 0, 0, 5800, 20, 20), 'busbar', at=(100, SPINE_Y - 10, 30), tag=(5000, 10, 20))  # 30 wide x 20 tall
# ---------------- bays
def bay(name, x0, front):
    y0 = 0 if front else BACK_EDGE; edge = FRONT_EDGE if front else BACK_EDGE
    add_part(name, cube(0, 0, 0, BAY_W - 4, BAY_D, BAY_T), 'bayplate', at=(x0 + 2, y0, FLOOR_T), tag=(BAY_W / 2, 60 if front else BAY_D - 60, BAY_T))
    sgn = 1 if front else -1          # +Y toward the spine for the front lane
    for s, dx in (('l', -200), ('r', 200)):
        py = edge - 40 if front else edge
        pin = '[translate 20 %g 200 [rotate -90 0 0 [cylinder 6 25]]]' % (40 if front else 0) if front else '[translate 20 0 200 [rotate 90 0 0 [cylinder 6 25]]]'
        add_part('%s_datum_%s' % (name, s), '[union %s %s]' % (cube(0, 0, 0, 40, 40, 215 - BAY_T), pin), 'datum',
                 at=(x0 + 250 + dx - 20, py, BZ))
    add_part(name + '_coupler', cube(0, 0, 0, 120, 60, 60), 'coupler', at=(x0 + 190, edge - 60 if front else edge, 120), bbox=(120, 60, 60))
for i in range(12):
    bay('bay_B%d' % (i + 1), i * BAY_W, False); bay('bay_F%d' % (i + 1), i * BAY_W, True)
# ---------------- machines (back lane: face Y = 1760; front lane: face Y = 940)
BF, FF = BACK_EDGE + 60, FRONT_EDGE - 60
box('saw', 150, BF, BZ, 700, 600, 900, 'fixture')                  # B1-B2, table z 926
box('sheet_router', 950, BF, BZ, 1100, 900, 500, 'router')       # B3-B4: 1100 wide in a 1000 slot -> overhangs 50 each side
box('mill_stand', 2025, BF, BZ, 450, 500, 700, 'stand'); box('mill', 2050, BF, BZ + 700, 400, 350, 380, 'fixture')       # B5, door z 726..1106
box('reflow_stand', 2525, BF, BZ, 450, 500, 700, 'stand'); box('reflow', 2600, BF, BZ + 700, 300, 300, 200, 'fixture')   # B6
def rack(name, x, y, front):   # 900 x 500 x 1800, 8 shelves (z 200 .. 1700 above the floor plate datum), open toward the spine
    b = [cube(px, py, 0, 30, 30, 1800) for px in (0, 870) for py in (0, 470)]
    for k in range(8):
        z = 200 + k * 1500 / 7 - BZ
        b.append(cube(0, 0, z - 15, 900, 500, 15))
    add_part(name, U(b), 'rack', at=(x, y, BZ), tag=(450, 250, 1800))
rack('tray_rack', 3050, BF, False)                                 # B7-B8, shelf tops z 200..1700 (absolute)
box('winder_stand', 4025, BF, BZ, 450, 500, 700, 'stand'); box('winder', 4100, BF, BZ + 700, 300, 300, 300, 'fixture')   # B9
box('stock_drawers', 4525, BF, BZ, 450, 600, 900, 'fixture')       # B10
# B11-B12 empty
def roller(name, x):           # 450 x 900 x 800 roller table
    b = [cube(px, py, 0, 30, 30, 770) for px in (0, 420) for py in (0, 870)] + [cube(0, 0, 770, 450, 900, 10)]
    b += [cube(0, 30 + k * 70, 780, 450, 20, 20) for k in range(12)]
    add_part(name, U(b), 'roller', at=(x, 20, BZ), tag=(225, 450, 800))
roller('dock_in', 25)                                              # F1
# F3-F5 bench below
legs = [cube(px, py, 0, 40, 40, 800 - BZ) for px in (0, 1460) for py in (0, 560)]
add_part('assembly_table', U(legs + [cube(0, 0, 800 - BZ, 1500, 600, 40)]), 'bench', at=(2500, FF - 600, BZ), tag=(750, 300, 840 - BZ))   # F6-F8, top z 800..840
rack('finished_rack', 4550, FF - 500, True)                         # F10-F11
roller('dock_out', 5525)                                           # F12
# ---------------- bench (cell_v2) in F3-F5, rotated 180 deg so its own front lane faces the spine
def bwrap(solid): return '[translate %g 1000 %g [rotate 0 0 180 %s]]' % (BENCH_X1, BZ, solid)
add_part('bench', bwrap(G['frame_solid']()), 'alu', tag=bench2aisle((750, 500, 1000)))
add_part('bench_deck', bwrap(cube(40, 40, C.deck_z - C.deck_t, C.cell_l - 80, C.cell_d - 80, C.deck_t)), 'deck', tag=None)
add_part('bench_bay_floor', bwrap(cube(40, 40, 40, C.cell_l - 80, C.cell_d - 80, 6)), 'deck', tag=None)
ry = (C.aisle_y - C.rail_gap // 2, C.aisle_y + C.rail_gap // 2)
for n, yy in (('bench_rail_f', ry[0]), ('bench_rail_b', ry[1])):
    add_part(n, bwrap('[translate %g %g %g %s]' % (C.rail_x0, yy - 20, C.deck_z, G['rail']())), 'rail')
add_part('bench_chain', bwrap(G['chain']()), 'chain')
for n, (x, y, z, sx, sy, sz) in G['FIX'].items():
    if n in ('cam', 'light'): continue
    ax, ay, az = bench2aisle((x + sx, y + sy, z))
    add_part('bench_' + n, cube(0, 0, 0, sx, sy, sz), G['MAT'][n], at=(ax, ay, az), bbox=(sx, sy, sz) if n in ('mill', 'reflow', 'bench', 'stock', 'finished', 'winder') else None)
parts.append('[part "bench_plinth" %s "plinth"]' % G['plinth']()); inst.append('[instance "bench_plinth" "bench_plinth" 0 0 0]')
bx = bench2aisle((C.car_x0, C.aisle_y, C.deck_z))
parts.append('[part "bench_feet" %s "hole"]' % G['foot_marks']()); inst.append('[instance "bench_feet" "bench_feet" 0 0 0]')
joints.append('[fixed-joint "bench_feet_mount" "bench_plinth" 0 0 %g "bench_feet" 0 0 0]' % C.plinth_h)
joints.append('[prismatic-joint "bench_carriage_x" -1 0 0 0 %d "floor" %g %g %g "bench_plinth" 0 0 0]' % (C.travel, *bx))
for side, dx in (('l', -C.arm_pitch / 2), ('r', C.arm_pitch / 2)):
    arm('b' + side, 'bench_arm_' + side, 'b', rzb, 'bench_plinth', (-dx, 0, C.plinth_h))
# ---------------- tenders
def tender(n, park):
    pl = 'tender_%d' % n
    parts.append('[part "%s" %s "plinth"]' % (pl, G['plinth']())); inst.append('[instance "%s" "%s" 0 0 0]' % (pl, pl))
    joints.append('[prismatic-joint "%s_x" 1 0 0 0 %d "floor" %g %g %g "%s" 0 0 0]' % (pl, T_TRAVEL, T_X0, SPINE_Y, FLOOR_T, pl))
    tags[pl] = (0, -160, PL_H)
    add_part(pl + '_column', cube(-100, -100, 0, 200, 200, COL_H), 'column', parent=pl, at=(0, 0, PL_H))
    st = pl + '_stage'
    parts.append('[part "%s" [union %s %s] "plinth"]' % (st, cube(-260, -160, 0, 520, 320, PLATE_T), cube(-85, -85, -640, 170, 170, 640)))
    inst.append('[instance "%s" "%s" 0 0 0]' % (st, st))
    joints.append('[prismatic-joint "%s_lift" 0 0 1 0 %d "%s" 0 0 %g "%s" 0 0 0]' % (pl, LIFT_MAX, pl, PL_H + COL_H, st))
    parts.append('[part "%s_feet" %s "hole"]' % (st, G['foot_marks'](ARM_PITCH))); inst.append('[instance "%s_feet" "%s_feet" 0 0 0]' % (st, st))
    joints.append('[fixed-joint "%s_feet_mount" "%s" 0 0 %g "%s_feet" 0 0 0]' % (st, st, PLATE_T, st))
    for side, dx in (('l', -ARM_PITCH / 2), ('r', ARM_PITCH / 2)):
        arm('%d%s' % (n, side), 'arm_%d%s' % (n, side), 't', rz, st, (dx, 0, PLATE_T))
tender(1, 1200); tender(2, 4200)
# ---------------- membrane + overhead + human
for n, x in (('curtain_in', -20), ('curtain_out', AISLE_L)):     # light curtain = emitter + receiver columns 700 apart
    add_part(n, U([cube(0, 0, 0, 20, 40, 2000), cube(0, 660, 0, 20, 40, 2000), cube(0, 0, 1960, 20, 700, 40)]), 'curtain',
             at=(x, FRONT_EDGE, FLOOR_T), tag=(10, 350, 2000))
def fence(n, y):               # 2000 mesh fence: posts every 1000 + 5 rails (renderer ignores alpha -> drawn as open mesh)
    b = [cube(px, 0, 0, 30, 30, 2000) for px in range(0, AISLE_L + 1, 1000)][:-1] + [cube(AISLE_L - 30, 0, 0, 30, 30, 2000)]
    b += [cube(0, 5, z, AISLE_L, 20, 15) for z in (100, 600, 1100, 1550, 1985)]
    add_part(n, U(b), 'fence', at=(0, y, FLOOR_T), tag=(3000, 15, 2000))
fence('fence_front', -30); fence('fence_back', AISLE_D)
gant = [cube(-40, 0, 0, 40, 40, 2440 - FLOOR_T), cube(AISLE_L, 0, 0, 40, 40, 2440 - FLOOR_T)] + [cube(-40, 0, 2400 - FLOOR_T, AISLE_L + 80, 40, 40)]
add_part('light_rail', U(gant), 'alu', at=(0, SPINE_Y - 20, FLOOR_T), tag=(3000, 20, 2420 - FLOOR_T))
for k in range(4):
    add_part('cam_%d' % (k + 1), cube(0, 0, 0, 60, 60, 50), 'elec', at=(750 + 1500 * k - 30, SPINE_Y - 30, 2350), tag=(30, 30, 0))
add_part('human', '[union [cylinder 200 1200] [union [translate 0 0 1200 [cylinder 60 400]] [translate 0 0 1640 [sphere 110]]]]',
         'human', at=(3000, SPINE_Y, FLOOR_T), tag=(0, 0, 1750))
# ---------------- emit
L = [';; aisle.loon - GENERATED by gen_aisle.py (params in aislep.py; bench = cell_v2 via gen_cell_v2.py helpers). Z-up, mm.',
     ';; spine X 0..6000 (walkway Y 1000..1700), 12 bays per lane at 500 pitch, two tenders, bench in F3-F5 rotated 180 deg.']
L += ['[material "alu" 0.78 0.78 0.80 0.9 0.4]', '[material "deck" 0.55 0.52 0.48 0.0 0.8]',
      '[material "rail" 0.25 0.25 0.27 0.8 0.3]', '[material "plinth" 0.70 0.72 0.75 0.9 0.4]', '[material "hole" 0.06 0.06 0.07 0.0 0.9]',
      '[material "fixture" 0.85 0.45 0.2 0.2 0.6]', '[material "bench" 0.62 0.48 0.32 0.0 0.8]',
      '[material "elec" 0.20 0.45 0.30 0.2 0.6]', '[material "light" 0.98 0.96 0.80 0.0 0.3]',
      '[material "chain" 0.12 0.12 0.12 0.0 0.7]', '[material "busbar" 0.80 0.50 0.25 0.9 0.3]',
      '[material "bayplate" 0.42 0.44 0.47 0.6 0.5]', '[material "datum" 0.85 0.15 0.15 0.5 0.4]',
      '[material "coupler" 0.95 0.75 0.10 0.2 0.5]', '[material "router" 0.30 0.45 0.70 0.3 0.5]',
      '[material "stand" 0.35 0.35 0.38 0.3 0.6]', '[material "rack" 0.55 0.60 0.66 0.6 0.4]',
      '[material "roller" 0.50 0.52 0.55 0.7 0.4]', '[material "column" 0.30 0.32 0.36 0.6 0.4]',
      '[material "curtain" 0.95 0.80 0.10 0.2 0.5]', '[material "fence" 0.70 0.75 0.30 0.3 0.6]',
      '[material "human" 0.30 0.55 0.85 0.0 0.7]', F.MATERIALS.strip()]
L.append('[assembly\n  #[' + '\n    '.join(parts) + ']\n  #[' + '\n    '.join(inst) + ']\n  #[' + '\n    '.join(joints) + ']\n  "floor"]')
open(S + '/aisle.loon', 'w').write('\n'.join(L) + '\n')
json.dump(tags, open(S + '/aisle_tags.json', 'w'), indent=0)
print(len(parts), 'parts', len(inst), 'instances', len(joints), 'joints', file=sys.stderr)
