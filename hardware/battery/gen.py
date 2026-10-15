#!/usr/bin/env python3
"""rose A/B self-swap battery pack (13S2P, 21700) -> loon.   usage: gen.py --mode MODE [--bbonly]

MODE: pack|pack_labels|explode|explode_labels|section|receiver|receiver_labels|swap1..swap4|dock|dock_labels
Z-up, mm. Pack-local frame: x 0 (inner face, connector) -> PW (outer face, handle); y 0 (front) -> PD; z 0 -> PH.
Robot frame = torso-local frame of rose/mini/cad/k1.loon (torso instance at world z 508); see README.md.
"""
import argparse, math
from reach import solve, fk

ap = argparse.ArgumentParser()
ap.add_argument('--mode', default='pack')
ap.add_argument('--bbonly', action='store_true')
A = ap.parse_args()

# ---------------------------------------------------------------- pack constants
CD, CH = 21.7, 70.0           # 21700 cell, modelled at nominal size (datasheet max 21.55 x 70.15)
PITCH = 22.5                  # 0.8 mm air/holder web between cells
WALL = 1.2
NX, NY, NT = 2, 5, 3          # 2 across x, 5 along y, 3 tiers: 30 slots, 26 cells + BMS bay (4 slots)
TIER = 74.0                   # 70 cell + 2 x 1.5 holder + 1 busbar
PW = 2 * WALL + NX * PITCH    # 47.4
PD = 2 * WALL + NY * PITCH + 1.1   # 116.0
PH = 2.0 + NT * TIER + 2.0    # 226
SLOT_Y_BMS = (3, 4)           # tier 3 rows 3-4 = BMS bay
TORSO_Z0 = 508.0

def r(v): return f"{v:.3f}".rstrip('0').rstrip('.')
BB = [1e9] * 3 + [-1e9] * 3
TRACK = [True]
def _bb(x0, y0, z0, x1, y1, z1):
    if not TRACK[0]: return
    for i, v in enumerate((x0, y0, z0)): BB[i] = min(BB[i], v)
    for i, v in enumerate((x1, y1, z1)): BB[3 + i] = max(BB[3 + i], v)
def box(x0, y0, z0, x1, y1, z1):
    x0, x1 = sorted((x0, x1)); y0, y1 = sorted((y0, y1)); z0, z1 = sorted((z0, z1))
    _bb(x0, y0, z0, x1, y1, z1)
    return f"[translate {r(x0)} {r(y0)} {r(z0)} [cube {r(x1-x0)} {r(y1-y0)} {r(z1-z0)}]]"
def hole(*a):
    s = TRACK[0]; TRACK[0] = False; g = box(*a); TRACK[0] = s; return g
def zcyl(x, y, z, rad, h):
    _bb(x - rad, y - rad, z, x + rad, y + rad, z + h)
    return f"[translate {r(x)} {r(y)} {r(z)} [cylinder {r(rad)} {r(h)}]]"
def xcyl(x, y, z, rad, h):    # axis +X from x
    _bb(x, y - rad, z - rad, x + h, y + rad, z + rad)
    return f"[translate {r(x)} {r(y)} {r(z)} [rotate 0 90 0 [cylinder {r(rad)} {r(h)}]]]"
def sphere(x, y, z, rad):
    _bb(x - rad, y - rad, z - rad, x + rad, y + rad, z + rad)
    return f"[translate {r(x)} {r(y)} {r(z)} [sphere {r(rad)}]]"
def seg(p, q, rad):           # cylinder from p to q
    d = [q[i] - p[i] for i in range(3)]; L = math.sqrt(sum(c * c for c in d))
    th = math.degrees(math.acos(max(-1, min(1, d[2] / L)))); ph = math.degrees(math.atan2(d[1], d[0]))
    for pt in (p, q): _bb(pt[0] - rad, pt[1] - rad, pt[2] - rad, pt[0] + rad, pt[1] + rad, pt[2] + rad)
    return f"[translate {r(p[0])} {r(p[1])} {r(p[2])} [rotate 0 0 {r(ph)} [rotate 0 {r(th)} 0 [cylinder {r(rad)} {r(L)}]]]]"
def un(xs):
    xs = [x for x in xs if x]
    if len(xs) == 1: return xs[0]
    m = len(xs) // 2
    return f"[union {un(xs[:m])} {un(xs[m:])}]"
def cut(plate, holes): return f"[difference {un(holes)} {plate}]" if holes else plate   # loon: B - A
def mv(g, dx, dy, dz): return g if not (dx or dy or dz) else f"[translate {r(dx)} {r(dy)} {r(dz)} {g}]"

MATS = {
    'case': (0.24, 0.25, 0.27, 0.6, 0.5), 'alu': (0.80, 0.81, 0.83, 0.9, 0.35), 'alu_dark': (0.30, 0.31, 0.33, 0.8, 0.45),
    'cell': (0.15, 0.45, 0.75, 0.2, 0.45), 'cellneg': (0.75, 0.30, 0.25, 0.2, 0.5), 'holder': (0.12, 0.12, 0.13, 0.1, 0.8),
    'nickel': (0.72, 0.74, 0.76, 0.9, 0.3), 'copper': (0.85, 0.50, 0.25, 0.95, 0.3), 'pcb': (0.10, 0.42, 0.25, 0.1, 0.6),
    'chip': (0.12, 0.12, 0.13, 0.3, 0.5), 'conn': (0.90, 0.88, 0.80, 0.1, 0.6), 'orange': (0.90, 0.55, 0.15, 0.3, 0.5),
    'datum': (0.90, 0.20, 0.15, 0.5, 0.4), 'mica': (0.85, 0.80, 0.60, 0.1, 0.7), 'ghost': (0.45, 0.40, 0.55, 0.2, 0.6),
    'rubber': (0.08, 0.08, 0.08, 0.0, 0.9), 'led': (0.20, 0.95, 0.40, 0.2, 0.3), 'robot': (0.55, 0.57, 0.60, 0.6, 0.5),
    'pcb_blue': (0.12, 0.20, 0.45, 0.1, 0.6), 'fresh': (0.30, 0.75, 0.35, 0.2, 0.5),
}
parts = []
def part(name, geo, mat): parts.append((name, geo, mat))

# ---------------------------------------------------------------- pack
def cell_slots():
    out = []
    for t in range(NT):
        for j in range(NY):
            if t == NT - 1 and j in SLOT_Y_BMS: continue
            for i in range(NX):
                out.append((WALL + PITCH * (i + .5), WALL + PITCH * (j + .5), 2.0 + t * TIER + 2.5, t, j, i))
    return out
assert len(cell_slots()) == 26

def pack_geo(ex=0.0, case_open=True):
    """list of (name, geo, mat, explode-offset). ex scales the explode offsets."""
    G = []
    def add(n, g, m, e=(0, 0, 0)): G.append((n, g, m, tuple(v * ex for v in e)))
    cs = cell_slots()
    add('cells x26', un([zcyl(x, y, z, CD / 2, CH) for (x, y, z, *_ ) in cs]), 'cell', (0, -170, 0))
    # holders: one plate under and over each tier (holes not cut: mockup)
    hp = []
    for t in range(NT):
        z0 = 2.0 + t * TIER
        hp += [box(WALL, WALL, z0, PW - WALL, PD - WALL, z0 + 1.5), box(WALL, WALL, z0 + 73 - 1.5, PW - WALL, PD - WALL, z0 + 73)]
    add('cell holders', un(hp), 'holder', (0, -110, 0))
    # busbars: a strip per parallel pair on each tier face, alternating so groups chain in series along y
    bb = []
    for t in range(NT):
        z0 = 2.0 + t * TIER
        for j in range(NY):
            if t == NT - 1 and j in SLOT_Y_BMS: continue
            y = WALL + PITCH * (j + .5)
            bb.append(box(WALL + 3, y - 4, z0 + 73, PW - WALL - 3, y + 4, z0 + 74))     # top of tier: pair bus
            bb.append(box(WALL + 3, y - 4, z0 + 0.5, PW - WALL - 3, y + 4, z0 + 1.5))
            if j < NY - 1 and not (t == NT - 1 and j + 1 in SLOT_Y_BMS):
                bb.append(box(WALL + 6, y, z0 + 73 if j % 2 == 0 else z0 + 0.5, WALL + 10, y + PITCH, z0 + 74 if j % 2 == 0 else z0 + 1.5))
        if t < NT - 1:                                                                     # tier-to-tier riser
            bb.append(box(PW - WALL - 2.2, WALL + PITCH * (NY - .5) - 4, z0 + 73, PW - WALL - 0.8, WALL + PITCH * (NY - .5) + 4, z0 + TIER + 1.5))
    add('busbars', un(bb), 'nickel', (0, -60, 0))
    # BMS in the tier-3 bay
    zb = 2.0 + 2 * TIER
    yb0, yb1 = WALL + PITCH * 3 + 1, PD - WALL - 1
    add('BMS, CAN', un([box(PW / 2 - 0.8, yb0, zb + 3, PW / 2 + 0.8, yb1, zb + 70),
                        box(PW / 2 + 0.8, yb0 + 8, zb + 20, PW / 2 + 3, yb0 + 22, zb + 34),
                        box(PW / 2 + 0.8, yb0 + 26, zb + 40, PW / 2 + 6, yb1 - 6, zb + 52),
                        box(PW / 2 - 6, yb0 + 6, zb + 50, PW / 2 - 0.8, yb1 - 8, zb + 62)]), 'pcb', (0, 80, 60))
    add('fuse 100 A', box(6, yb0 + 4, zb + 8, 20, yb1 - 4, zb + 20), 'orange', (0, 80, 30))
    # case: inner wall, outer face, front, back, floor, lid (front wall left off in the iso so the cells show)
    walls = [('case inner wall', box(-1.5, 0, 0, 0, PD, PH), (-60, 0, 0)),
             ('case outer face', cut(box(PW, 0, 0, PW + 2, PD, PH), [hole(PW - 1, PD / 2 - 30, 8, PW + 3, PD / 2 + 30, 42)]), (60, 0, 0)),
             ('case back', box(-1.5, PD, 0, PW + 2, PD + 1.5, PH), (0, 60, 0)),
             ('case floor', box(-1.5, 0, -2, PW + 2, PD + 1.5, 0), (0, 0, -60)),
             ('case lid', box(-1.5, 0, PH, PW + 2, PD + 1.5, PH + 2), (0, 0, 60))]
    if not case_open: walls.append(('case front', box(-1.5, -1.5, 0, PW + 2, 0, PH), (0, -60, 0)))
    for n, g, e in walls: add(n, g, 'case', e)
    add('mica barrier', box(-3, 0, 2, -1.5, PD, PH - 2), 'mica', (-90, 0, 0))
    # connector: SBS75X-class housing on the inner face, low; 2 guide pins
    cy, cz = PD / 2, 26.0
    add('connector SBS75X', un([box(-20, cy - 20, cz - 12, -1.5, cy + 20, cz + 12),
                                box(-22, cy - 14, cz - 7, -20, cy - 4, cz + 7), box(-22, cy + 4, cz - 7, -20, cy + 14, cz + 7)]), 'conn', (-60, 0, 0))
    add('guide pins', un([xcyl(-14, 14, cz, 3, 12.5), xcyl(-14, PD - 14, cz, 3, 12.5)]), 'alu', (-60, 0, 0))
    # charge pads on the floor (dock), recessed
    add('dock charge pads', un([box(10 + i * 8, PD / 2 - 3, -2.6, 15 + i * 8, PD / 2 + 3, -2) for i in range(4)]), 'copper', (0, 0, -90))
    # latch: cam bar inside the outer wall, dogs out of floor and lid
    add('latch cam, dogs', un([box(PW - 4, PD / 2 - 3, 6, PW - 1, PD / 2 + 3, PH - 6),
                                box(PW - 9, PD / 2 - 6, PH + 2, PW - 1, PD / 2 + 6, PH + 6),
                                box(PW - 9, PD / 2 - 6, -6, PW - 1, PD / 2 + 6, -2)]), 'datum', (40, 0, 0))
    # handle = ball cuff (K1 ball-hand): bracket + ring on the outer face, low (reach). Ball seat centre at x PW+29, z 8.
    # Seating the ball presses the release paddle (retracts the dogs); spring fingers hold the ball while it pulls.
    hz, cyy = 8.0, 23.0
    ring = cut(xcyl(PW + 18, cyy, hz, 27, 22), [f"[translate {r(PW + 17)} {r(cyy)} {r(hz)} [rotate 0 90 0 [cylinder 23.5 24]]]"])
    add('ball cuff handle', un([ring, box(PW + 2, cyy - 14, hz - 14, PW + 18, cyy + 14, hz + 14)]), 'orange', (90, 0, 0))
    add('release paddle', xcyl(PW + 18, cyy, hz, 10, 3), 'datum', (110, 0, 0))
    add('vent, burst disc', xcyl(PW + 2, PD / 2, PH - 30, 9, 1.5), 'rubber', (60, 0, 0))
    add('SoC LEDs', un([box(PW + 2, PD / 2 - 18 + i * 9, PH - 60, PW + 3, PD / 2 - 14 + i * 9, PH - 56) for i in range(4)]), 'led', (60, 0, 0))
    return G

def funnel_geo(ox=0.0):
    """receiver mouth: a frame with 8 mm x 45 deg lead-ins around the pack opening (pack-local, at the outer face)."""
    m, L = 8.0, 10.0
    y0, y1, z0, z1 = -1.5 - 0.6, PD + 1.5 + 0.6, -2 - 0.6, PH + 2 + 0.6
    xs = PW + 2 + ox
    seg_ = []
    # four tapered strips approximated by two steps each
    for k, (a, b) in enumerate(((0, L / 2), (L / 2, L))):
        g = m * (1 - k / 2)
        seg_ += [box(xs + a, y0 - g - 6, z0 - g - 6, xs + b, y0 - g, z1 + g + 6), box(xs + a, y1 + g, z0 - g - 6, xs + b, y1 + g + 6, z1 + g + 6),
                 box(xs + a, y0 - g, z0 - g - 6, xs + b, y1 + g, z0 - g), box(xs + a, y0 - g, z1 + g, xs + b, y1 + g, z1 + g + 6)]
    return un(seg_)

def place(G, dx, dy, dz, suffix='', mat=None):
    for (n, g, m, e) in G:
        part(n + suffix, mv(mv(g, *e), dx, dy, dz), mat or m)

# ---------------------------------------------------------------- robot side
# right pack A: pack-local x -> torso +x; outer face flush with the widened shell (x 107)
SHELL_X, SHELL_Y = 107.0, 65.0
PZ = 48.0
PY = -PD / 2
PXA = SHELL_X - PW - 2           # 57.6
BAY = (-54.8, -40.0, 122.0, 54.8, 44.0 + 10, 280.0)

def pack_into(side, slide, name, tagonly=False, mat=None):
    """side +1 = right (A), -1 = left (B). slide = mm pulled out along the side axis."""
    G = pack_geo(0, case_open=False)
    keep = ('cells x26', 'case outer face', 'case lid', 'case front', 'case back', 'connector SBS75X', 'ball cuff handle', 'vent, burst disc', 'SoC LEDs', 'latch cam, dogs', 'release paddle')
    gs = [g for (n, g, m, e) in G if n in keep]
    body = un(gs)
    if side > 0: geo = mv(body, PXA + slide, PY, PZ)
    else: geo = f"[translate {r(-PXA - slide)} {r(PY + PD)} {r(PZ)} [rotate 0 0 180 {body}]]"
    if side < 0: _bb(-PXA - slide - PW - 42, PY, PZ - 6, -PXA - slide + 22, PY + PD, PZ + PH + 6)
    part(name, geo, mat or 'case')

def receiver(side, with_funnel=True):
    s = 1 if side > 0 else -1
    x0, x1 = PXA - 24, SHELL_X
    def X(a, b): return (min(s * a, s * b), max(s * a, s * b))
    rx = X(x0, x1)
    rails = [box(rx[0], PY - 3.5, PZ - 4, rx[1], PY - 1.5, PZ + PH + 4), box(rx[0], PY + PD + 1.5, PZ - 4, rx[1], PY + PD + 3.5, PZ + PH + 4),
             box(rx[0], PY - 3.5, PZ - 6, rx[1], PY + PD + 3.5, PZ - 4)]
    nm = 'A' if s > 0 else 'B'
    part(f'receiver {nm}', un(rails), 'alu_dark')
    bp = X(PXA - 26, PXA - 24)
    part(f'backplane, ORing {nm}', box(bp[0], PY, PZ - 4, bp[1], PY + PD, PZ + 60), 'pcb_blue')
    sk = X(PXA - 24, PXA - 18)
    part(f'socket {nm}', box(sk[0], -22, PZ + 26 - 14, sk[1], 22, PZ + 26 + 14), 'conn')
    if with_funnel:
        fx = X(SHELL_X, SHELL_X + 10)
        m = 8
        part(f'funnel {nm}', un([box(fx[0], PY - 3.5 - m - 4, PZ - 6 - m, fx[1], PY - 3.5, PZ + PH + 4 + m),
                                 box(fx[0], PY + PD + 3.5, PZ - 6 - m, fx[1], PY + PD + 3.5 + m + 4, PZ + PH + 4 + m),
                                 box(fx[0], PY - 3.5, PZ - 6 - m, fx[1], PY + PD + 3.5, PZ - 6),
                                 box(fx[0], PY - 3.5, PZ + PH + 4, fx[1], PY + PD + 3.5, PZ + PH + 4 + m)]), 'orange')

def torso_edges(dz=0.0, w=SHELL_X):
    e = 1.5; out = []
    for (x0, x1, y0, y1, z0, z1) in ((-w, w, -SHELL_Y, SHELL_Y, 44, 282), (-92, 92, -60, 60, -46, 44)):
        z0 += dz; z1 += dz
        for (x, y) in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)): out.append(box(x - e, y - e, z0, x + e, y + e, z1))
        for z in (z0, z1):
            out += [box(x0, y0 - e, z - e, x1, y0 + e, z + e), box(x0, y1 - e, z - e, x1, y1 + e, z + e),
                    box(x0 - e, y0, z - e, x0 + e, y1, z + e), box(x1 - e, y0, z - e, x1 + e, y1, z + e)]
    return un(out)

def blade_bay(dz=0.0):
    x0, y0, z0, x1, y1, z1 = BAY
    part('blade bay 4 blades', un([box(x0, y0, z0 + dz, x1, y1 - 10, z1 + dz),
                                    un([box(x0 + 6 + i * 8, y1 - 10, z0 + 20 + dz, x0 + 9 + i * 8, y1, z1 - 20 + dz) for i in range(13)])]), 'alu')

# ---------------------------------------------------------------- swap scene (world frame, torso at z 508)
DOCK_GAP = 6.0
SLOT_PITCH = 150.0
def dock_robot(fresh_in_slot2=True, old_in_slot1=None):
    """passive dock beside the robot's right flank: 2 rail channels along X at pack-floor height, charge pads, stops."""
    zt = TORSO_Z0 + PZ - 6            # rail floor = receiver floor
    x0 = SHELL_X + 10 + DOCK_GAP      # past the torso funnel
    L = PW + 40
    ch = []
    for k in range(2):
        yc = SLOT_PITCH * k
        ch += [box(x0, yc + PY - 8, zt - 4, x0 + L, yc + PY + PD + 8, zt),
               box(x0, yc + PY - 8, zt, x0 + L, yc + PY - 3.5, zt + 40), box(x0, yc + PY + PD + 3.5, zt, x0 + L, yc + PY + PD + 8, zt + 40),
               box(x0 + L - 6, yc + PY - 8, zt, x0 + L, yc + PY + PD + 8, zt + 60)]
    part('dock rails, 2 slots', un(ch), 'alu_dark')
    part('dock lead-in', un([box(x0 - 4, SLOT_PITCH * k + PY - 18, zt - 4, x0, SLOT_PITCH * k + PY + PD + 18, zt + 40) for k in range(2)]), 'orange')
    part('dock stand', un([box(x0 + 10, PY, 0, x0 + 30, PY + 20, zt - 60), box(x0 + 10, PY + PD + SLOT_PITCH - 20, 0, x0 + 30, PY + PD + SLOT_PITCH, zt - 60),
                           box(x0, PY - 40, 0, x0 + L, PY + PD + SLOT_PITCH + 40, 8)]), 'alu')
    part('dock charger', box(x0 + 10, PY + 10, zt - 60, x0 + L - 8, PY + PD + SLOT_PITCH - 10, zt - 4), 'pcb_blue')
    return x0

def robot(dy=0.0, arm_target=None, arm_side='R'):
    T = TORSO_Z0
    part('torso, widened shell', mv(torso_edges(T), 0, dy, 0), 'ghost')
    s = TRACK[0]
    blade_bay_geo = un([box(BAY[0], BAY[1] + dy, BAY[2] + T, BAY[3], BAY[4] - 10 + dy, BAY[5] + T)])
    part('blade bay', blade_bay_geo, 'alu')
    part('head', box(-65, -62 + dy, T + 278 + 50, 65, 62 + dy, T + 278 + 170), 'robot')
    legs = []
    for x in (-60, 60):
        legs += [box(x - 34, -28 + dy, 260, x + 34, 42 + dy, 460), box(x - 29, -26 + dy, 48, x + 29, 40 + dy, 252),
                 box(x - 35, -70 + dy, 0, x + 35, 85 + dy, 25)]
    part('legs', un(legs), 'robot')
    # arms: left hangs; right posed by reach.solve to the target (torso-local)
    arms = []
    for side in ('R', 'L'):
        sx = 132 if side == 'R' else -132
        if side == arm_side and arm_target is not None:
            d, p, rr, e, m, el = solve(side, arm_target, step=2)
            h, el = fk(side, p, rr, e)
        else:
            h, el = fk(side, 0, 0, 0)
        S = (sx, dy, 254 + T); E = (el[0], el[1] + dy, el[2] + T); H = (h[0], h[1] + dy, h[2] + T)
        arms += [sphere(*S, 27), seg(S, E, 22), sphere(*E, 24), seg(E, H, 19), sphere(*H, 22)]
    part('arms, K1 4-DoF', un(arms), 'robot')

def scene(step):
    # robot frame: dy shifts the robot along Y (it side-steps between dock slots)
    T = TORSO_Z0
    x0 = dock_robot()
    hz = PZ + 8                                  # handle grip: pack-local x PW+29, z 8
    if step == 1:      # hook pack A's handle; B carries the load
        dy = 0; slideA = 0
        robot(dy, arm_target=(PXA + PW + 29, -35, hz))
        place_pack_world(+1, slideA, dy, 'pack A, old')
        place_pack_world(-1, 0, dy, 'pack B, live')
        part('fresh pack on dock', mv(dock_pack(), x0 + 6, SLOT_PITCH, 0), 'fresh')
    elif step == 2:    # pulled across the bridge onto dock slot 1 (to its stop)
        dy = 0; slideA = x0 + 6 - PXA
        robot(dy, arm_target=(PXA + slideA + PW + 29, -35, hz))
        place_pack_world(+1, slideA, dy, 'pack A, old')
        place_pack_world(-1, 0, dy, 'pack B, live')
        part('fresh pack on dock', mv(dock_pack(), x0 + 6, SLOT_PITCH, 0), 'fresh')
    elif step == 3:    # side-step to slot 2, hook the fresh pack
        dy = SLOT_PITCH
        robot(dy, arm_target=(x0 + 6 + PW + 29, -35, hz))
        part('pack A, old, charging', mv(dock_pack(), x0 + 6, 0, 0), 'case')
        place_pack_world(-1, 0, dy, 'pack B, live')
        part('fresh pack on dock', mv(dock_pack(), x0 + 6, SLOT_PITCH, 0), 'fresh')
    else:              # pushed home: handle returns, cam draws the last 8 mm
        dy = SLOT_PITCH
        robot(dy, arm_target=(PXA + PW + 29, -35, hz))
        part('pack A, old, charging', mv(dock_pack(), x0 + 6, 0, 0), 'case')
        place_pack_world(-1, 0, dy, 'pack B, live')
        place_pack_world(+1, 0, dy, 'fresh pack, seated', mat='fresh')
    for side in (1, -1):
        s = 1 if side > 0 else -1
        part('receiver ' + ('A' if s > 0 else 'B'), mv(rec_geo(s), 0, dy, T), 'alu_dark')

def rec_geo(s):
    x0, x1 = (PXA - 24, SHELL_X + 10)
    xa, xb = (x0, x1) if s > 0 else (-x1, -x0)
    return un([box(xa, PY - 3.5, PZ - 6, xb, PY + PD + 3.5, PZ - 4), box(xa, PY - 3.5, PZ - 4, xb, PY - 1.5, PZ + PH + 4),
               box(xa, PY + PD + 1.5, PZ - 4, xb, PY + PD + 3.5, PZ + PH + 4)])

def dock_pack():
    G = pack_geo(0, case_open=False)
    keep = ('cells x26', 'case outer face', 'case lid', 'case front', 'case back', 'connector SBS75X', 'ball cuff handle', 'SoC LEDs')
    return mv(un([g for (n, g, m, e) in G if n in keep]), 0, PY, TORSO_Z0 + PZ)

def place_pack_world(side, slide, dy, name, mat=None):
    s0 = len(parts)
    pack_into(side, slide, name, mat=mat)
    n, g, m = parts[-1]
    parts[-1] = (n, mv(g, 0, dy, TORSO_Z0), m)

# ---------------------------------------------------------------- 10" rack UPS / dock shelf (3U, 4 packs on edge)
U = 44.45
def ups_shelf():
    W, OPEN, D = 254.0, 222.25, 300.0
    H = 3 * U
    part('rack ears', un([box(0, 0, 0, 16, 3, H), box(W - 16, 0, 0, W, 3, H)]), 'alu')
    part('shelf tray', box(16, 0, 0, W - 16, D, 2), 'alu')
    part('side cheeks', un([box(16, 0, 2, 18, D, H - 2), box(W - 18, 0, 2, W - 16, D, H - 2)]), 'alu')
    # pack lying on its side, floor (charge/UPS pads) to the rear: pack z -> rack -y, pack y -> rack z, pack x -> rack x.
    # The side-mounted SBS75X and the handle stick out of the 47.4 mm body, so 3 packs per row at 72 mm pitch.
    pitch = 72.0
    xs = [W / 2 + pitch * (k - 1) for k in range(3)]
    guides, pogo = [], []
    for k, xc in enumerate(xs):
        x0 = xc - PW / 2
        guides += [box(x0 - 4, 4, 2, x0 - 2, D - 34, 8), box(x0 + PW + 4, 4, 2, x0 + PW + 6, D - 34, 8)]
        pogo.append(box(x0 + 8, D - 34, 6 + PD / 2 - 8, x0 + 42, D - 30, 6 + PD / 2 + 8))
    part('pack guides', un(guides), 'alu_dark')
    part('backplane, chargers, ORing', box(20, D - 30, 2, W - 20, D - 28, H - 4), 'pcb_blue')
    part('pogo pad blocks x3', un(pogo), 'conn')
    part('48 V busbar to rack', box(W / 2 - 30, D - 28, 10, W / 2 + 30, D + 10, 16), 'copper')
    G = pack_geo(0, case_open=False)
    keep = ('cells x26', 'case outer face', 'case lid', 'case front', 'case back', 'case floor', 'ball cuff handle', 'SoC LEDs',
            'vent, burst disc', 'connector SBS75X', 'dock charge pads', 'latch cam, dogs')
    body = un([g for (n, g, m, e) in G if n in keep])
    s = TRACK[0]; TRACK[0] = False
    pk = []
    for k, xc in enumerate(xs):
        out = 110 if k == 2 else 0
        pk.append(f"[translate {r(xc - PW / 2)} {r(D - 32 - out)} 6 [rotate 90 0 0 {body}]]")
    TRACK[0] = s
    _bb(16, D - 32 - 110 - 232, 0, W - 16, D + 10, H)
    part('packs x3, one pulled', un(pk), 'case')
    return dict(W=W, D=D, H=H)

# ---------------------------------------------------------------- emit + labels
def emit(pp, ground):
    out = [f";; GENERATED by gen.py (mode {A.mode}). rose A/B 13S2P 21700 pack. Z-up, mm."]
    out += [f'[material "{m}" {" ".join(map(str, v))}]' for m, v in MATS.items()]
    out.append("[assembly")
    out.append("  #[" + "\n    ".join(f'[part "{n}" {g} "{m}"]' for n, g, m in pp) + "]")
    out.append("  #[" + "\n    ".join(f'[instance "{n}" "{n}" 0 0 0]' for n, *_ in pp) + "]")
    out.append("  #[]")
    out.append(f'  "{ground}"]')
    return "\n".join(out) + "\n"

def tag(name, x, y, z):
    x, y, z = (min(max(v, BB[i] + 1), BB[3 + i] - 1) for i, v in enumerate((x, y, z)))
    s = TRACK[0]; TRACK[0] = False; g = box(x - 1, y - 1, z - 1, x + 1, y + 1, z + 1); TRACK[0] = s
    return (name, g, 'alu')
def labels_doc(tags, ghost=False):
    s = TRACK[0]; TRACK[0] = False
    base = [g for (n, g, m) in parts] if ghost else [box(*BB)]
    tg = [tag(n, *p) for n, p in tags]
    pp = [('', un(base + [t[1] for t in tg]), 'alu')] if A.bbonly else [('', un(base), 'alu')] + tg
    TRACK[0] = s
    return emit(pp, '')

M = A.mode
if M in ('pack', 'pack_labels'):
    for (n, g, m, e) in pack_geo(0): part(n, g, m)
    part('receiver funnel', funnel_geo(28), 'ghost')
    if M == 'pack_labels':
        print(labels_doc([('cells 13S2P', (WALL, 30, 150)), ('BMS, CAN', (PW / 2 + 6, PD - 20, 2 + 2 * TIER + 52)),
                          ('connector SBS75X', (-22, PD / 2, 20)), ('latch cam, dogs', (PW - 5, PD / 2, PH + 6)),
                          ('ball cuff handle', (PW + 40, 0, 8)), ('receiver funnel', (PW + 40, -12, PH + 10))], ghost=True), end='')
    else:
        print(emit(parts, 'case floor'), end='')
elif M in ('explode', 'explode_labels'):
    G = pack_geo(1.0)
    for (n, g, m, e) in G: part(n, mv(g, *e), m)
    if M == 'explode_labels':
        o = {n: e for (n, _, _, e) in G}
        def p(n, x, y, z): e = o[n]; return (n, (x + e[0], y + e[1], z + e[2]))
        print(labels_doc([p('cells x26', 0, 30, 150), p('cell holders', 0, 20, 2 * TIER + 75), p('busbars', PW, 60, TIER + 74),
                          p('BMS, CAN', PW / 2, PD - 10, 2 + 2 * TIER + 70), p('case outer face', PW + 2, PD, PH),
                          p('case inner wall', 0, 0, PH), p('connector SBS75X', -22, PD / 2, 20), p('ball cuff handle', PW + 40, 0, 8),
                          p('mica barrier', -3, 0, PH - 10), p('fuse 100 A', 20, PD - 20, 2 * TIER + 20)], ghost=True), end='')
    else:
        print(emit(parts, 'case floor'), end='')
elif M == 'section':
    for (n, g, m, e) in pack_geo(0, case_open=False): part(n, g, m)
    print(emit(parts, 'case floor'), end='')
elif M in ('receiver', 'receiver_labels'):
    part('torso shell, widened 214', torso_edges(0), 'ghost')
    blade_bay()
    for s in (1, -1): receiver(s)
    pack_into(+1, 0, 'pack A, seated')
    pack_into(-1, 70, 'pack B, pulled 70')
    part('receiver mica wall', un([box(PXA - 18, PY, PZ + 60, PXA - 16, PY + PD, PZ + PH), box(-PXA + 16, PY, PZ + 60, -PXA + 18, PY + PD, PZ + PH)]), 'mica')
    if M == 'receiver_labels':
        print(labels_doc([('pack A, seated', (SHELL_X + 2, PY + PD, PZ + 120)), ('pack B, pulled 70', (-SHELL_X - 70 - 16, PY, PZ + 140)),
                          ('blade bay 4 blades', (BAY[3], BAY[1], BAY[5])), ('funnel A', (SHELL_X + 10, PY - 15, PZ - 10)),
                          ('socket B', (-PXA + 18, -22, PZ + 40)), ('torso shell, widened 214', (SHELL_X, SHELL_Y, 282)),
                          ('receiver mica wall', (PXA - 16, PY, PZ + PH - 20))]), end='')
    else:
        print(emit(parts, 'receiver A'), end='')
elif M.startswith('swap'):
    scene(int(M[4:]))
    if M.endswith('_labels'): pass
    print(emit(parts, 'legs'), end='')
elif M in ('dock', 'dock_labels'):
    d = ups_shelf()
    if M == 'dock_labels':
        print(labels_doc([('packs x3, one pulled', (199, -150, 60)), ('backplane, chargers, ORing', (234, 270, 120)),
                          ('rack ears', (3, 1, 125)), ('pack guides', (40, 100, 8))]), end='')
    else:
        print(emit(parts, 'shelf tray'), end='')
else:
    raise SystemExit(f'unknown mode {M}')
