#!/usr/bin/env python3
"""10-inch toolless mini rack v2: CM5 blades, real-GPU node, humanoid blade bay -> loon.

usage: gen_rack.py [--units 6|9|12] [--depth D] [--t T]
                   [--mode full|labels|cutlist|checks|
                           blade|blade_labels|blade_explode|blade_explode_labels|
                           chassis|chassis_labels|gpu|gpu_labels|gpu_section|
                           bay|bay_labels]

Z-up, mm. X = width (0..254), Y = depth (front y=0 -> back y=D), Z = height.
Frame parts are flat sheet (no bends); see README.md.
"""
import argparse, math

ap = argparse.ArgumentParser()
ap.add_argument('--units', type=int, default=9)
ap.add_argument('--depth', type=float, default=450.0)
ap.add_argument('--t', type=float, default=2.0)
ap.add_argument('--mode', default='full')
ap.add_argument('--bbonly', action='store_true')   # labels doc with the bbox only (to mask its stray leader)
A = ap.parse_args()

# ---------------------------------------------------------------- standard
U = 44.45                    # EIA-310 1U
W = 254.0                    # 10" outer front width
HOLE_PITCH = 236.525         # 10" rail: hole centre to hole centre (9.312")
OPENING = 222.25             # 10" clear opening between rails (8.75")
HOLE_Z = (6.35, 22.225, 38.1)  # EIA hole pattern inside each U
TAP_D = 5.0                  # v2: round M6 tap-drill holes (was 9.5 square cage-nut)
N, D, T = A.units, A.depth, A.t
RT = 3.0                     # v2: rails are 3 mm sheet
hx_l = W / 2 - HOLE_PITCH / 2            # 8.7375
hx_r = W / 2 + HOLE_PITCH / 2            # 245.2625
op_l = W / 2 - OPENING / 2               # 15.875
op_r = W / 2 + OPENING / 2               # 238.125
RAIL_Y = 10.0
z0 = T + 4.0
H = z0 + N * U + 4.0 + T
uz = lambda k: z0 + k * U

# ---------------------------------------------------------------- blade standard (blade-local frame)
# x: across the blade (spine at x 0..3, components on +x), y: front face (0) -> connector mating face (BL)
# z: spine bottom edge (0) -> top edge (BH).  Envelope x -2.3..22.3 incl. panel, body x 0..20.
PITCH = 25.4                 # slot pitch (1.0")
BW = 20.0                    # body envelope (spine + stack)
BL = 132.0                   # panel front face -> connector mating face
BH = 76.0                    # spine height = rail-to-rail
EDGE = 5.0                   # rail edge band (no components), engagement 3.0 nominal
SPINE_T = 3.0
PANEL = (-2.3, 22.3, -2.0, 82.0)    # panel x0, x1, z0, z1 (24.6 x 84)

def r(v): return f"{v:.3f}".rstrip('0').rstrip('.')
BB = [1e9, 1e9, 1e9, -1e9, -1e9, -1e9]
TRACK = [True]
CUR = [1e9, 1e9, 1e9, -1e9, -1e9, -1e9]      # per-part extents (always tracked), read with grab()
def grab():
    v = list(CUR); CUR[:] = [1e9, 1e9, 1e9, -1e9, -1e9, -1e9]; return v
def _bb(x0, y0, z0_, x1, y1, z1):
    for i, v in enumerate((x0, y0, z0_)): CUR[i] = min(CUR[i], v)
    for i, v in enumerate((x1, y1, z1)): CUR[3 + i] = max(CUR[3 + i], v)
    if not TRACK[0]: return
    for i, v in enumerate((x0, y0, z0_)): BB[i] = min(BB[i], v)
    for i, v in enumerate((x1, y1, z1)): BB[3 + i] = max(BB[3 + i], v)
def box(x0, y0, z0_, x1, y1, z1):
    x0, x1 = min(x0, x1), max(x0, x1); y0, y1 = min(y0, y1), max(y0, y1); z0_, z1 = min(z0_, z1), max(z0_, z1)
    _bb(x0, y0, z0_, x1, y1, z1)
    return f"[translate {r(x0)} {r(y0)} {r(z0_)} [cube {r(x1-x0)} {r(y1-y0)} {r(z1-z0_)}]]"
def hole(x0, y0, z0_, x1, y1, z1):           # like box but never grows the bbox (cut-outs)
    x0, x1 = min(x0, x1), max(x0, x1); y0, y1 = min(y0, y1), max(y0, y1); z0_, z1 = min(z0_, z1), max(z0_, z1)
    return f"[translate {r(x0)} {r(y0)} {r(z0_)} [cube {r(x1-x0)} {r(y1-y0)} {r(z1-z0_)}]]"
def cyl(x, y, z, rad, h):
    _bb(x - rad, y - rad, z, x + rad, y + rad, z + h)
    return f"[translate {r(x)} {r(y)} {r(z)} [cylinder {r(rad)} {r(h)}]]"
def ycyl(x, y, z, rad, h):                    # axis along +Y starting at y
    _bb(x - rad, y, z - rad, x + rad, y + h, z + rad)
    return f"[translate {r(x)} {r(y)} {r(z)} [rotate -90 0 0 [cylinder {r(rad)} {r(h)}]]]"
def ycone(x, y, z, r0, h):
    _bb(x - r0, y, z - r0, x + r0, y + h, z + r0)
    return f"[translate {r(x)} {r(y)} {r(z)} [rotate -90 0 0 [cone {r(r0)} 0.01 {r(h)}]]]"
def zcone(x, y, z, r0, h):
    _bb(x - r0, y - r0, z, x + r0, y + r0, z + h)
    return f"[translate {r(x)} {r(y)} {r(z)} [cone {r(r0)} 0.01 {r(h)}]]"
def un(xs):
    xs = list(xs)
    if len(xs) == 1: return xs[0]
    m = len(xs) // 2
    return f"[union {un(xs[:m])} {un(xs[m:])}]"
def cut(plate, holes):          # plate - holes   (loon: [difference A B] = B - A)
    return f"[difference {un(holes)} {plate}]" if holes else plate

MATS = {
    'alu': (0.80, 0.81, 0.83, 0.9, 0.35), 'alu_dark': (0.30, 0.31, 0.33, 0.8, 0.45),
    'rubber': (0.08, 0.08, 0.08, 0.0, 0.9), 'pcb': (0.10, 0.42, 0.25, 0.1, 0.6),
    'pcb_blue': (0.12, 0.20, 0.45, 0.1, 0.6), 'chip': (0.12, 0.12, 0.13, 0.3, 0.5),
    'nvme': (0.15, 0.15, 0.16, 0.3, 0.5), 'port': (0.72, 0.74, 0.76, 0.9, 0.3),
    'heatsink': (0.55, 0.57, 0.60, 0.9, 0.4), 'copper': (0.85, 0.50, 0.25, 0.95, 0.3),
    'standoff': (0.85, 0.70, 0.30, 0.9, 0.3), 'dcdc': (0.20, 0.30, 0.60, 0.2, 0.5),
    'gpu': (0.18, 0.62, 0.30, 0.3, 0.5), 'psu': (0.45, 0.46, 0.48, 0.8, 0.5),
    'orange': (0.90, 0.55, 0.15, 0.3, 0.5), 'datum': (0.90, 0.20, 0.15, 0.5, 0.4),
    'pad': (0.85, 0.45, 0.55, 0.0, 0.8), 'conn': (0.90, 0.88, 0.80, 0.1, 0.6),
    'led': (0.20, 0.95, 0.40, 0.2, 0.3), 'fan': (0.16, 0.16, 0.18, 0.2, 0.6),
    'air': (0.30, 0.80, 0.95, 0.1, 0.4), 'hot': (0.95, 0.30, 0.20, 0.1, 0.4),
    'ghost': (0.45, 0.40, 0.55, 0.2, 0.6),
}
parts = []          # (name, geometry, material, explode (dx,dy,dz))
cutlist = []        # (part, qty, L, W, T, material, note)
def part(name, geo, mat, ex=(0, 0, 0)): parts.append((name, geo, mat, ex))
def cl(name, q, l, w, t, note, m='5052'): cutlist.append((name, q, l, w, t, m, note))

# ---------------------------------------------------------------- frame
SIDE_TABS_Y = [(40, 80), (D / 2 - 20, D / 2 + 20), (D - 80, D - 40)]
SLIDE = 10.0
TRAY_TAB_Y = [(32, 48), (D - 48, D - 32)]
RAIL_TAB_K = list(range(0, N, 2))
def rail_tab_z(k): return (uz(k) + U - 7, uz(k) + U + 3)
DROP = 4.0
CH = 30.0
VB_X = (190, 206)

def frame():
    for xo, name, sgn in ((0, 'side_l', -1), (W - T, 'side_r', 1)):
        plate = box(xo, 0, T, xo + T, D, H - T)
        tabs = []
        for (a, b) in SIDE_TABS_Y:
            tabs.append(box(xo, a, H - T, xo + T, b, H))
            tabs.append(box(xo, a, 0, xo + T, b, T))
        holes = []
        for k in RAIL_TAB_K:
            za, zb = rail_tab_z(k)
            for y in (RAIL_Y, D - RAIL_Y - RT):
                holes.append(hole(xo - 1, y - 0.1, za - 0.1, xo + T + 1, y + RT + 0.1, zb + DROP))
        for k in range(N):
            for (a, b) in TRAY_TAB_Y:
                holes.append(hole(xo - 1, a - 0.2, uz(k) + 1 - 0.1, xo + T + 1, b + 0.2, uz(k) + 1 + T + DROP))
        for i in range(4):
            yc = D / 2 + (i - 1.5) * 75
            holes.append(hole(xo - 1, yc - 25, H / 2 - 12, xo + T + 1, yc + 25, H / 2 + 12))
        part(name, cut(un([plate] + tabs), holes), 'alu', (sgn * 110, 0, 0))
        cl(name, 1, D, H, T, f"{N} x 2 hook slots, {len(RAIL_TAB_K)*2} rail-tab slots (3 mm), 6 lid tabs, 4 grips")
    for name, z, top in (('lid_top', H - T, True), ('lid_bottom', 0, False)):
        plate = box(0, 0, z, W, D, z + T)
        holes = []
        for xo in (0, W - T):
            for (a, b) in SIDE_TABS_Y:
                holes.append(hole(xo - 0.2, a - 0.2, z - 1, xo + T + 0.2, b + SLIDE + 0.2, z + T + 1))
        if top:
            for i in range(9):
                x = 40 + i * 21.75
                holes.append(hole(x, 60, z - 1, x + 6, D - 60, z + T + 1))
        part(name, cut(plate, holes), 'alu', (0, 0, 90 if top else -70))
        cl(name, 1, W, D, T, "6 slide-lock notches" + (", 9 vent slots" if top else ", 4 stick-on feet"))
    part('feet', un([cyl(x, y, -8, 11, 8) for x in (22, W - 22) for y in (22, D - 22)]), 'rubber')
    for nm, y in (('rail_front', RAIL_Y), ('rail_rear', D - RAIL_Y - RT)):
        for side, (x0, x1, hx, td) in (('l', (T, op_l, hx_l, -1)), ('r', (op_r, W - T, hx_r, 1))):
            body = box(x0, y, z0 - 2, x1, y + RT, z0 + N * U + 2)
            tabs = []
            for k in RAIL_TAB_K:
                za, zb = rail_tab_z(k)
                tabs.append(box(-0.6, y, za, x0, y + RT, zb) if td < 0 else box(x1, y, za, W + 0.6, y + RT, zb))
            TRACK[0] = False
            holes = [ycyl(hx, y - 1, uz(k) + hz, TAP_D / 2, RT + 2) for k in range(N) for hz in HOLE_Z]
            TRACK[0] = True
            part(f'{nm}_{side}', cut(un([body] + tabs), holes), 'alu_dark')
        cl(nm, 2, round(N * U + 4, 1), round(op_l - T + T + 0.6, 2), RT,
           f"{3*N} x M6 tapped (5.0 drill), {len(RAIL_TAB_K)} hook tabs; web 4.2 mm outside hole")
    ch_z0, ch_z1 = z0, z0 + N * U
    fins = [box(T, D - 8, ch_z0, T + T, D + CH, ch_z1), box(W - 2 * T, D - 8, ch_z0, W - T, D + CH, ch_z1)]
    comb = box(T, D + CH, ch_z0, W - T, D + CH + T, ch_z1)
    comb_holes = [hole(30 + i * 50, D + CH - 1, uz(k) + 10, 30 + i * 50 + 30, D + CH + T + 1, uz(k) + 34)
                  for k in range(N) for i in range(4)]
    part('cable_channel', un(fins + [cut(comb, comb_holes)]), 'alu_dark')
    cl('cable_fin', 2, CH + 8, round(N * U, 1), T, "flat, L-hook into side-panel rear slot")
    cl('cable_comb', 1, W - 2 * T, round(N * U, 1), T, f"{4*N} cable windows")
    part('bus_48v', un([box(x, D + 6, ch_z0 + 2, x + 6, D + 18, ch_z1 - 2) for x in VB_X]), 'copper')

# ---------------------------------------------------------------- module helpers
def tray(n, k):
    z = uz(k) + 1
    plate = box(T + 0.2, 14, z, W - T - 0.2, D - 14, z + T)
    tabs = [box(-0.6, a, z, T + 0.2, b, z + T) for (a, b) in TRAY_TAB_Y] + \
           [box(W - T - 0.2, a, z, W + 0.6, b, z + T) for (a, b) in TRAY_TAB_Y]
    part(f'{n}_tray', un([plate] + tabs), 'alu')
    cl(f'{n}_tray', 1, round(W - 2 * T - 0.4 + 2 * (T + 0.8), 1), D - 28, T, "4 hook tabs (L hooks, drop 4 mm)")
    return z + T

def face(n, k, u, holes=(), x0=T + 0.5, x1=W - T - 0.5, nm=None, note="faceplate"):
    plate = box(x0, RAIL_Y - T, uz(k) + 0.4, x1, RAIL_Y, uz(k) + u * U - 0.4)
    part(nm or f'{n}_face', cut(plate, list(holes)), 'alu_dark')
    cl(nm or f'{n}_face', 1, round(x1 - x0, 1), round(u * U - 0.8, 2), T, note)

def fcut(x0, x1, z0_, z1): return hole(x0, RAIL_Y - T - 1, z0_, x1, RAIL_Y + 1, z1)

def fan(cx, y, cz, s, d):        # square fan frame s x s, depth d (along Y), with a hub
    ring = f"[translate {r(cx)} {r(y-1)} {r(cz)} [rotate -90 0 0 [cylinder {r(s*0.46)} {r(d+2)}]]]"
    return un([cut(box(cx - s / 2, y, cz - s / 2, cx + s / 2, y + d, cz + s / 2), [ring]), ycyl(cx, y + 2, cz, s * 0.17, d - 4)])

def arrow_y(x, y0, y1, z, w=3.0):     # airflow arrow along +Y
    return un([box(x - w / 2, y0, z - w / 2, x + w / 2, y1 - 3 * w, z + w / 2), ycone(x, y1 - 3 * w, z, 1.6 * w, 3 * w)])
def arrow_z(x, y, z0_, z1, w=3.0):    # along +Z
    return un([box(x - w / 2, y - w / 2, z0_, x + w / 2, y + w / 2, z1 - 3 * w), zcone(x, y, z1 - 3 * w, 1.6 * w, 3 * w)])

# ---------------------------------------------------------------- the blade (standard module)
def blade_geo(explode=False):
    """returns [(name, geo, mat, ex)] in blade-local coords."""
    e = (lambda v: v) if explode else (lambda v: (0, 0, 0))
    g = []; grab()
    spine = box(0, 2, 0, SPINE_T, BL - 2, BH)
    spine_h = [hole(-1, 100, 14, 4, 124, 30), hole(-1, 100, 46, 4, 124, 62)]       # lightening over the DC-DC end
    g.append(('spine, cold rail', cut(spine, spine_h), 'alu', e((-120, 0, 0)), grab()))
    g.append(('gap pad', box(3, 32, 18, 4, 87, 58), 'pad', e((-85, 0, 0)), grab()))
    g.append(('CM5', box(4, 32, 18, 8.7, 87, 58), 'chip', e((-50, 0, 0)), grab()))
    g.append(('standoffs', un([box(3, y - 2, z - 2, 10.2, y + 2, z + 2) for y in (14, 112) for z in (12, 64)]), 'standoff', e((-20, 0, 0)), grab()))
    g.append(('carrier', box(10.2, 4, EDGE + 1, 11.8, 124, BH - EDGE - 1), 'pcb', e((8, 0, 0)), grab()))
    g.append(('NVMe 2242', box(11.8, 40, 46, 14.8, 82, 68), 'nvme', e((40, 0, 0)), grab()))
    g.append(('DC-DC 48-5V', box(11.8, 92, 10, 19.8, 118, 36), 'dcdc', e((40, 0, 0)), grab()))
    g.append(('eFuse', box(11.8, 112, 40, 13.8, 120, 46), 'chip', e((40, 0, 0)), grab()))
    g.append(('rear connector', un([box(11.8, 118, 20, 19.0, BL, 56), box(14.4, BL, 58, 16.4, BL + 6, 60),
                                    box(14.4, BL, 16, 16.4, BL + 6, 18)]), 'conn', e((40, 30, 0)), grab()))
    g.append(('USB-C', box(11.8, 1, 30, 15.0, 10, 39), 'port', e((40, 0, 0)), grab()))
    g.append(('LED', box(15.5, 0, 70, 18.5, 8, 73), 'led', e((40, -14, 0)), grab()))
    pan = box(PANEL[0], 0, PANEL[2], PANEL[1], 2, PANEL[3])
    ph = [hole(11.3, -1, 29.5, 15.5, 3, 39.5), hole(15, -1, 69.5, 19, 3, 73.5)] + \
         [hole(1, -1, zz, 9, 3, zz + 2.5) for zz in (46, 51, 56, 61)] + [hole(11, -1, zz, 20, 3, zz + 2.5) for zz in (46, 51, 56, 61)]
    g.append(('front panel', cut(pan, ph), 'alu_dark', e((0, -40, 0)), grab()))
    g.append(('pull handle', un([box(4, -12, 6, 12, -9, 26), box(4, -12, 6, 12, 0, 9), box(4, -12, 23, 12, 0, 26)]), 'orange', e((0, -70, 0)), grab()))
    return g

def place_blade(prefix, ox, oy, oz, rot=None, only=None):
    """add blade parts at an offset (rot='bay': local y -> -Z, local z -> +Y)."""
    for (n, geo, m, ex, _) in blade_geo():
        if only and n not in only: continue
        if rot == 'bay':
            geo = f"[translate {r(ox)} {r(oy)} {r(oz)} [rotate -90 0 0 {geo}]]"
        else:
            geo = f"[translate {r(ox)} {r(oy)} {r(oz)} {geo}]"
        yield (n, geo, m)

def blades_into(prefix, origins, rot=None):
    """group the same component of every blade into one part (keeps the part count sane)."""
    groups = {}
    s = TRACK[0]; TRACK[0] = False
    for (ox, oy, oz) in origins:
        for (n, geo, m) in place_blade(prefix, ox, oy, oz, rot):
            groups.setdefault((n, m), []).append(geo)
    TRACK[0] = s
    for (n, m), gs in groups.items():
        part(f'{prefix}{n}', un(gs), m)

# ---------------------------------------------------------------- blade chassis (2U, 8 blades)
NSLOT = 8
SLOT_X0 = W / 2 - NSLOT * PITCH / 2 + 0.4 - PANEL[0]       # spine x of slot 0 (28.1)
def slot_x(i): return SLOT_X0 + i * PITCH
BLADE_Y0 = RAIL_Y - T                                        # panel front face flush with the faceplates (y 8)
MATE_Y = BLADE_Y0 + BL                                       # connector mating plane (y 140)

def blade_chassis(n, k, standalone=False):
    b = tray(n, k)
    bot, top = b, b + BH                                     # spine sits on the tray, 3 mm in the comb slots
    # filler faceplates either side of the blade panels
    face(n, k, 2, nm=f'{n}_face_l', x0=T + 0.5, x1=slot_x(0) + PANEL[0] - 0.4, note="filler faceplate")
    face(n, k, 2, nm=f'{n}_face_r', x0=slot_x(NSLOT - 1) + PANEL[1] + 0.4, x1=W - T - 0.5, note="filler faceplate")
    # comb guides: flat 3 mm plates with open-ended slots, one on the tray, one under the ceiling
    for nm, zc in (('comb_lower', bot), ('comb_upper', top - 3)):
        plate = box(20, 14, zc, 234, MATE_Y - 6, zc + 3)
        sl = [hole(slot_x(i) - 0.2, 13, zc - 1, slot_x(i) + SPINE_T + 0.2, MATE_Y - 10, zc + 4) for i in range(NSLOT)]
        part(f'{n}_{nm}', cut(plate, sl), 'alu_dark')
        cl(f'{n}_{nm}', 1, 214, MATE_Y - 20, 3.0, f"{NSLOT} open-ended 3.4 mm guide slots")
    if not standalone:                                       # standalone view leaves the ceiling off
        part(f'{n}_ceiling', box(18, 14, top, 236, 330, top + T), 'alu')
    cl(f'{n}_ceiling', 1, 218, 316, T, "rests on cheek tabs")
    if not standalone:
        part(f'{n}_cheeks', un([box(16, 16, bot, 18, 330, top), box(236, 16, bot, 238, 330, top)]), 'alu')
    cl(f'{n}_cheek', 2, 314, BH, T, "tabs into tray + ceiling slots")
    origins = [(slot_x(i), BLADE_Y0 - (70 if standalone and i == 5 else 0), bot) for i in range(NSLOT)]
    pre = '' if standalone else f'{n}_'
    blades_into(pre + 'blade ' if standalone else pre, origins)
    cl(f'{n}_blade_spine', NSLOT, BL - 4, BH, SPINE_T, "blade spine / cold rail, 6061-T6", '6061')
    cl(f'{n}_blade_panel', NSLOT, 24.6, 84, T, "blade front panel")
    # backplane: receptacles + PCB with airflow windows between connectors
    rec = [box(slot_x(i) + 11.6, MATE_Y, bot + 18, slot_x(i) + 19.2, MATE_Y + 10, bot + 58) for i in range(NSLOT)]
    part(f'{n}_receptacles', un(rec), 'chip')
    bp = box(22, MATE_Y + 10, bot + 4, 232, MATE_Y + 11.6, bot + 72)
    win = [hole(slot_x(i) + 1, MATE_Y + 9, bot + 10, slot_x(i) + 10.5, MATE_Y + 13, bot + 66) for i in range(NSLOT)]
    part(f'{n}_backplane', cut(bp, win), 'pcb_blue')
    part(f'{n}_switch', un([box(40, 170, bot, 214, 250, bot + 1.6), box(110, 195, bot + 1.6, 140, 225, bot + 8)]), 'pcb_blue')
    part(f'{n}_uplinks', un([box(150 + i * 20, D - 40, bot, 166 + i * 20, D - 14, bot + 13.5) for i in range(2)]), 'port')
    fans = [fan(55 + i * 72, 280, bot + 38, 60, 25) for i in range(3)]
    part(f'{n}_fans', un(fans), 'fan')
    part(f'{n}_fan_wall', cut(box(20, 306, bot, 234, 308, top),
                              [f"[translate {r(55+i*72)} 305 {r(bot+38)} [rotate -90 0 0 [cylinder 28 4]]]" for i in range(3)]), 'alu_dark')
    cl(f'{n}_fan_wall', 1, 214, BH, T, "3 x 60 mm fan holes")
    busbar(n, bot, 240)

def busbar(n, bz, y_from):
    """module 48 V pair: from the vertical bus fingers forward to y_from along the right side."""
    bars = [box(VB_X[0], D - 24, bz, VB_X[0] + 6, D + 6, bz + 12), box(VB_X[1], D - 30, bz, VB_X[1] + 6, D + 6, bz + 12),
            box(VB_X[0], D - 27, bz, 232, D - 24, bz + 12), box(VB_X[1], D - 33, bz, 226, D - 30, bz + 12),
            box(229, y_from, bz, 232, D - 24, bz + 12), box(223, y_from, bz, 226, D - 30, bz + 12)]
    part(f'{n}_busbar', un(bars), 'copper')
    part(f'{n}_datum', un([cyl(x, D - 20, bz, 3, 8) for x in (12 + T, W - 12 - T)]), 'datum')

# ---------------------------------------------------------------- GPU node (3U)
G = dict(card_L=340, card_H=150, card_T=61, intake=15, bend=35, slot_blk=12, card_x0=30, card_y0=50,
         mezz=2, board=1.6, board_so=6, cooler=37, fan_d=25)

def gpu_node(n, k, u=3, airflow=False):
    b = tray(n, k)
    cx0, cy0 = G['card_x0'], G['card_y0']
    cx1, cy1 = cx0 + G['card_H'], cy0 + G['card_L']
    cz0, cz1 = b + G['intake'], b + G['intake'] + G['card_T']
    mz = cz1 + 3                                             # mezzanine bottom
    a = mz + G['mezz']                                       # upper deck
    # face: 3 x 60 fan grilles (lower deck) + 2 x 40 (upper deck)
    vents = [fcut(36 + i * 64 + 4, 36 + i * 64 + 56, b + 8, b + 64) for i in range(3)] + \
            [fcut(70 + i * 70, 110 + i * 70, a + 6, a + 40) for i in range(2)]
    face(n, k, u, vents)
    part(f'{n}_fans', un([fan(66 + i * 64, 16, b + 36, 60, 25) for i in range(3)] +
                         [fan(90 + i * 70, 16, a + 23, 40, 20) for i in range(2)]), 'fan')
    # GPU: lying flat, fans down, IO bracket at the rear, PCIe fingers toward -X into the riser slot block
    gpu = [box(cx0, cy0, cz0 + 2, cx1, cy1, cz1 - 4),                 # cooler + shroud
           box(cx0, cy0, cz1 - 4, cx1 - 10, cy1, cz1),                # PCB + backplate
           box(cx0 - 2, cy1, cz0 + 4, cx0 + 120, cy1 + 2, cz1)]        # IO bracket (rear)
    gfans = [f"[translate {r(cx0 + 75)} {r(cy0 + 60 + i * 110)} {r(cz0)} [cylinder 46 2]]" for i in range(3)]
    _bb(cx0, cy0, cz0, cx1, cy1, cz1)
    part(f'{n}_GPU', un(gpu + gfans), 'gpu')
    part(f'{n}_riser slot', box(cx0 - G['slot_blk'], cy0 + 60, cz1 - 14, cx0, cy0 + 150, cz1), 'orange')
    # 12V-2x6 on the top edge (+X), connector + 35 mm straight run before the bend
    px = cx1
    part(f'{n}_12V-2x6', un([box(px, cy0 + 230, cz1 - 16, px + 10, cy0 + 252, cz1 - 4),
                             box(px + 10, cy0 + 232, cz1 - 15, px + G['bend'], cy0 + 250, cz1 - 5),
                             box(px + G['bend'] - 6, cy0 + 232, cz1 - 5, px + G['bend'], cy0 + 250, a + 8)]), 'chip')
    # mezzanine (2 mm) on 4 posts, cut-out for the flex riser and the GPU power cable
    mezz = box(T + 0.4, 14, mz, W - T - 0.4, D - 24, mz + G['mezz'])
    mh = [hole(6, cy0 + 60, mz - 1, cx0 - 4, cy0 + 150, mz + 4), hole(px + G['bend'] - 8, cy0 + 228, mz - 1, px + G['bend'] + 2, cy0 + 254, mz + 4)] + \
         [hole(40 + i * 30, 160, mz - 1, 52 + i * 30, 230, mz + 4) for i in range(5)]
    part(f'{n}_mezzanine', cut(mezz, mh), 'alu', (0, 0, 70))
    cl(f'{n}_mezzanine', 1, W - 2 * T - 0.8, D - 38, T, "on 4 x M4 posts, riser + cable cut-outs, 5 vent slots")
    part(f'{n}_posts', un([cyl(x, y, b, 4, mz - b) for x in (12, 242) for y in (24, D - 34)]), 'standoff')
    # upper deck: 48->12 V DC-DC at the front, mini-ITX at the rear with IO out the back
    part(f'{n}_DC-DC 48-12V', un([box(150, 50, a, 230, 130, a + 12), box(154, 54, a + 12, 226, 126, a + 30)]), 'dcdc')
    bx0, by0 = 20, D - 24 - 170 - 6
    part(f'{n}_standoffs', un([cyl(x, y, a, 3, G['board_so']) for x in (bx0 + 6, bx0 + 164) for y in (by0 + 6, by0 + 164)]), 'standoff')
    bz = a + G['board_so']
    part(f'{n}_mITX', box(bx0, by0, bz, bx0 + 170, by0 + 170, bz + G['board']), 'pcb')
    bt = bz + G['board']
    part(f'{n}_rear IO', box(bx0 + 5, by0 + 158, bt, bx0 + 165, D - 14, bt + 30), 'port')
    part(f'{n}_CPU cooler', box(bx0 + 40, by0 + 50, bt, bx0 + 135, by0 + 145, bt + G['cooler']), 'heatsink')
    part(f'{n}_DIMMs', un([box(bx0 + 145 + i * 9, by0 + 18, bt, bx0 + 151 + i * 9, by0 + 151, bt + 31) for i in range(2)]), 'chip')
    # flex riser: x16 slot on the board edge -> loop down through the mezzanine -> riser slot block
    rz = cz1 - 8
    part(f'{n}_flex riser', un([box(bx0 + 4, by0 + 20, bt, bx0 + 14, by0 + 110, bt + 11),       # slot on board
                                box(6, by0 + 20, rz, 9, by0 + 110, bt + 6),                   # vertical ribbon
                                box(6, cy0 + 60, rz, cx0 - G['slot_blk'], cy0 + 150, rz + 3),
                                box(6, cy0 + 110, rz, 9, by0 + 110, rz + 3),
                                box(6, by0 + 20, bt + 6, bx0 + 4, by0 + 110, bt + 9)]), 'orange')
    part(f'{n}_12V cable', un([box(px + G['bend'] - 6, cy0 + 232, a + 2, px + G['bend'], 130, a + 8)]), 'chip')
    busbar(n, a, 132)
    if airflow:
        X = 105                                              # all arrows straddle the --section x=105 plane
        arr = [arrow_y(X, 44, cy0 + 50, b + 7), arrow_y(X, cy1 + 4, D + 26, cz0 + 25),
               arrow_y(X, 40, by0 - 4, a + 40), arrow_y(X, by0 + 150, D + 26, bt + 40)] + \
              [arrow_z(X, cy0 + 60 + i * 110, b + 1, cz0 + 1, 2.5) for i in range(3)]
        part('airflow', un(arr), 'air')
    return dict(b=b, cz0=cz0, cz1=cz1, mz=mz, a=a, top=bt + G['cooler'], px=px)

def gpu_checks():
    k = 1; b = uz(k) + 1 + T
    cz1 = b + G['intake'] + G['card_T']; a = cz1 + 3 + G['mezz']; top = a + G['board_so'] + G['board'] + G['cooler']
    lim = uz(k) + 3 * U + 1                                  # underside of the next tray
    out = ["# GPU fit checks (3U node, depth %.0f)" % D]
    rail_clear_y = (D - RAIL_Y - RT) - 14                    # the rear rails sit at the sides only; card is in the opening
    out.append(f"card length 340 + rear bracket 2: y {G['card_y0']}..{G['card_y0']+342}; behind it to the tray end y {D-14:.0f}: "
               f"{D-14-G['card_y0']-342:.0f} mm for exhaust; front fans y 16..41, plenum {G['card_y0']-41} mm")
    out.append(f"longest card that fits: {D-14-6-G['card_y0']:.0f} mm (keeps 6 mm at the rear grille); 12V-2x6 is on the top edge, so no length penalty")
    xw = G['card_x0'] - G['slot_blk']
    out.append(f"across: riser block x {xw}..{G['card_x0']}, card height {G['card_H']} -> x {G['card_x0']+G['card_H']}, "
               f"+ {G['bend']} mm straight run before the 12V-2x6 bend -> x {G['card_x0']+G['card_H']+G['bend']}; "
               f"opening edge x {op_r:.1f} -> {op_r-(G['card_x0']+G['card_H']+G['bend']):.1f} mm spare")
    out.append(f"tallest card with the bend: {op_r - G['card_x0'] - G['bend'] - 3:.0f} mm (3 mm clear of the rail line)")
    out.append(f"vertical: tray top {b-uz(k):.1f}, intake gap {G['intake']}, card {G['card_T']} (3-slot incl. backplate), "
               f"3 clear, mezz {G['mezz']}, posts {G['board_so']}, board {G['board']}, cooler {G['cooler']} -> top {top-uz(k):.1f} "
               f"vs next tray underside {lim-uz(k):.1f}: {lim-top:.1f} mm spare")
    # 2-GPU in 4U: two cards stacked (fans down), each with its own intake gap
    lim4 = 4 * U + 1
    for slots, t in ((2, 40), (2.5, 51), (3, 61)):
        gap = 12
        need = 3 + 2 * (gap + t) + 3 + G['mezz'] + G['board_so'] + G['board'] + G['cooler']
        out.append(f"2-GPU 4U, {slots}-slot ({t} mm) cards, {gap} mm gaps: needs {need:.1f} of {lim4:.1f} -> "
                   f"{'FITS' if need <= lim4 - 2 else 'DOES NOT FIT'} ({lim4-need:+.1f})")
    out.append("")
    out.append(f"# humanoid blade bay ({BAY_N} blades): outer {BAY_W:.1f} W x {BAY_D:.0f} D (+10 fins) x {BAY_H:.0f} H mm; "
               f"torso interior ~206 x 126 x 234 -> spare {206-BAY_W:.0f} / {126-BAY_D-10:.0f} / {234-BAY_H:.0f} mm")
    rho = 2.70e-6
    bay = [("cold wall (grooved)", 2, BAY_W, BAY_H - 6 - BASE, WALL_Y, "6061 plate, 4 x 3.4 x 4.0 grooves, machined"),
           ("side wall", 2, BAY_D - 2 * WALL_Y, BAY_H - 6 - BASE, WALL_X, "6061, laser + tapped"),
           ("base", 1, BAY_W, BAY_D, BASE, "6061, connector bore + backplane bosses"),
           ("lid", 1, BAY_W + 2, BAY_D + 2, 6, "6061, O-ring groove"),
           ("rear fin block", 1, 13 * 8, BAY_H - 40, 10, "6061, 13 fins (or skip: bolt the cold wall to the torso)")]
    kg = 0
    for n, q, l, w, t, note in bay:
        out.append(f"{n:<22}{q:>3}{l:>8.1f}{w:>8.1f}{t:>6.1f}  {note}")
        kg += q * l * w * t * rho
    out.append(f"# ESTIMATE bay metal ~{kg:.2f} kg (solid outline, before pockets), ~${kg*12:.0f} raw 6061 at $12/kg; machining not priced")
    return "\n".join(out)

# ---------------------------------------------------------------- PSU, patch, blank
def psu(n, k):
    b = tray(n, k)
    face(n, k, 1, [fcut(35, 60, uz(k) + 10, uz(k) + 34), fcut(160, 185, uz(k) + 10, uz(k) + 34)])
    part(f'{n}_CRPS x2', un([box(28 + i * 115, 12, b, 28 + i * 115 + 73.5, 197, b + 39.5) for i in range(2)]), 'psu')
    part(f'{n}_PDB', box(28, 200, b, 226, 260, b + 1.6), 'pcb_blue')
    busbar(n, b + 2, 200)

def patch(n, k):
    b = tray(n, k)
    ks = [fcut(30 + i * 16.5, 30 + i * 16.5 + 14.5, uz(k) + 8, uz(k) + 8 + 16.5) for i in range(12)]
    face(n, k, 1, ks)
    part(f'{n}_keystones', un([box(31 + i * 16.5, RAIL_Y - 1, uz(k) + 9, 31 + i * 16.5 + 12.5, RAIL_Y + 30, uz(k) + 24)
                               for i in range(12)]), 'chip')
    part(f'{n}_agg switch', box(40, 60, b, 214, 200, b + 28), 'chip')

def blank(n, k):
    tray(n, k); face(n, k, 1, [fcut(30 + i * 20, 40 + i * 20, uz(k) + 10, uz(k) + 34) for i in range(10)])

# ---------------------------------------------------------------- humanoid blade bay
BAY_N = 4
WALL_X, WALL_Y = 4.0, 8.0                    # side walls / grooved cold walls
GROOVE = 4.0                                 # groove depth (spine engagement)
BAY_IN_Y = BH - 2 * GROOVE                   # 68 between the cold walls
BAY_W = BAY_N * PITCH + 2 * WALL_X           # 109.6
BAY_D = BAY_IN_Y + 2 * WALL_Y                # 84
BASE, BPZ = 6.0, 12.0                        # base plate, backplane+receptacle height
BAY_H = BASE + BPZ + BL + 2 + 6              # + panel + lid
TORSO = (210, 130, 238, 2)                   # rose mini / K1-class upper torso shell (rose/mini/cad/k1.loon)

def humanoid_bay():
    # bay origin: x 0..BAY_W, y 0..BAY_D, z 0..BAY_H. Blades stand upright, connector down.
    yi0 = WALL_Y
    FW = (175, 0)                                         # front cold wall drawn pulled aside so the blades show
    part('cold wall front', cut(box(0 + FW[0], 0 + FW[1], BASE, BAY_W + FW[0], WALL_Y + FW[1], BAY_H - 6),
         [hole(FW[0] + WALL_X + 2.3 + i * PITCH + 0.4 - 0.2, FW[1] + WALL_Y - GROOVE, BASE + BPZ - 2, FW[0] + WALL_X + 2.3 + i * PITCH + 0.4 + SPINE_T + 0.2, FW[1] + WALL_Y + 1, BAY_H) for i in range(BAY_N)]), 'alu_dark')
    part('cold wall rear', cut(box(0, BAY_D - WALL_Y, BASE, BAY_W, BAY_D, BAY_H - 6),
         [hole(WALL_X + 2.3 + i * PITCH + 0.4 - 0.2, BAY_D - WALL_Y - 1, BASE + BPZ - 2, WALL_X + 2.3 + i * PITCH + 0.4 + SPINE_T + 0.2, BAY_D - WALL_Y + GROOVE, BAY_H) for i in range(BAY_N)]), 'alu_dark')
    fins = [box(6 + i * 8, BAY_D, BASE + 20, 9 + i * 8, BAY_D + 10, BAY_H - 20) for i in range(13)]
    part('rear fins', un(fins), 'heatsink')
    part('side walls', un([box(0, WALL_Y, BASE, WALL_X, BAY_D - WALL_Y, BAY_H - 6), box(BAY_W - WALL_X, WALL_Y, BASE, BAY_W, BAY_D - WALL_Y, BAY_H - 6)]), 'alu')
    part('base', box(0, 0, 0, BAY_W, BAY_D, BASE), 'alu')
    part('bay backplane', box(WALL_X + 1, yi0 + 2, BASE, BAY_W - WALL_X - 1, BAY_D - yi0 - 2, BASE + 1.6), 'pcb_blue')
    # blade origins: local y=0 (panel front face) at the top, local z=0 (spine bottom edge) at y = WALL_Y - GROOVE
    top = BASE + BPZ + BL
    origins = [(WALL_X + 2.3 + 0.4 + i * PITCH, WALL_Y - GROOVE, top + (95 if i == 2 else 0)) for i in range(BAY_N)]
    part('receptacles', un([box(ox + 11.6, yi0 + 18 - GROOVE, BASE + 1.6, ox + 19.2, yi0 + 58 - GROOVE, BASE + BPZ) for (ox, _, _) in origins]), 'chip')
    blades_into('', origins, rot='bay')
    # wedge clamps (spring wedge per edge) in the grooves
    wl = []
    for (ox, oy, oz) in origins:
        if oz > top: continue
        for y0 in (WALL_Y - GROOVE, BAY_D - WALL_Y):
            wl.append(box(ox + SPINE_T + 0.2, y0 + (0 if y0 < 20 else 0), BASE + BPZ + 20, ox + SPINE_T + 2.2, y0 + GROOVE, top - 20))
    part('wedge clamps', un(wl), 'orange')
    LIFT = 125                                               # shown lifted off
    part('lid + gasket', box(-1, -1, BAY_H - 6 + LIFT, BAY_W + 1, BAY_D + 1, BAY_H + LIFT), 'alu')
    part('robot connector', un([cyl(BAY_W / 2, BAY_D / 2, -14, 11, 14), cyl(BAY_W / 2, BAY_D / 2, -18, 8, 4)]), 'chip')
    # reference: torso shell edges, bay centred in X and Y, sitting on the torso floor
    tx, ty, tz, tw = TORSO
    ox, oy = BAY_W / 2 - tx / 2, BAY_D / 2 - ty / 2 + 5
    oz = -20
    e = 1.5
    edges = []
    for (x, y) in ((ox, oy), (ox + tx, oy), (ox, oy + ty), (ox + tx, oy + ty)):
        edges.append(box(x - e, y - e, oz, x + e, y + e, oz + tz))
    for z in (oz, oz + tz):
        edges += [box(ox, oy - e, z - e, ox + tx, oy + e, z + e), box(ox, oy + ty - e, z - e, ox + tx, oy + ty + e, z + e),
                  box(ox - e, oy, z - e, ox + e, oy + ty, z + e), box(ox + tx - e, oy, z - e, ox + tx + e, oy + ty, z + e)]
    part('torso ref', un(edges), 'ghost')
    return dict(W=BAY_W, D=BAY_D + 10, H=BAY_H)

# ---------------------------------------------------------------- populate
PLANS = {
    6:  [('psu', psu, 0, 1), ('gpu', gpu_node, 1, 3), ('blades_a', blade_chassis, 4, 2)],
    9:  [('psu', psu, 0, 1), ('gpu', gpu_node, 1, 3), ('blades_a', blade_chassis, 4, 2), ('blades_b', blade_chassis, 6, 2),
         ('patch', patch, 8, 1)],
    12: [('psu', psu, 0, 1), ('gpu', gpu_node, 1, 3), ('blades_a', blade_chassis, 4, 2), ('blades_b', blade_chassis, 6, 2),
         ('blades_c', blade_chassis, 8, 2), ('blank', blank, 10, 1), ('patch', patch, 11, 1)],
}
LABEL = {'psu': '48 V PSU', 'gpu': 'GPU node 3U', 'patch': 'patch / switch', 'blank': 'blank',
         'blades_a': 'blade chassis 2U', 'blades_b': 'blade chassis 2U', 'blades_c': 'blade chassis 2U'}

def emit(pp, ground):
    out = [f";; GENERATED by gen_rack.py v2 (mode {A.mode}, {N}U, depth {D}, sheet {T} mm, rails {RT} mm). Z-up, mm."]
    out += [f'[material "{m}" {" ".join(map(str, v))}]' for m, v in MATS.items()]
    out.append("[assembly")
    out.append("  #[" + "\n    ".join(f'[part "{n}" {g} "{m}"]' for n, g, m, _ in pp) + "]")
    out.append("  #[" + "\n    ".join(f'[instance "{n}" "{n}" 0 0 0]' for n, *_ in pp) + "]")
    out.append("  #[]")
    out.append(f'  "{ground}"]')
    return "\n".join(out) + "\n"

def tag(name, x, y, z):          # 2 mm tag cube, kept inside the bbox so it never changes the framing
    x, y, z = (min(max(v, BB[i] + 1), BB[3 + i] - 1) for i, v in enumerate((x, y, z)))
    return (name, box(x - 1, y - 1, z - 1, x + 1, y + 1, z + 1), 'alu', 0)
def labels_doc(tags, ghost=False):
    """tags-only doc framed like the full doc. ghost=True carries the real geometry under blank names (needed when the
    silhouette is far from its 3D bbox, e.g. exploded views, since the renderer frames on the projected silhouette)."""
    s = TRACK[0]; TRACK[0] = False
    base = [g for (n, g, m, _) in parts] if ghost else [box(*BB)]
    tg = [tag(n, *p) for n, p in tags]
    if A.bbonly:                      # twin: same geometry, tags merged into the one blank-named part
        pp = [('', un(base + [t[1] for t in tg]), 'alu', 0)]
    else:
        pp = [('', un(base), 'alu', 0)] + tg
    TRACK[0] = s
    return emit(pp, '')

M = A.mode
if M in ('full', 'labels', 'cutlist', 'checks'):
    assert sum(p[3] for p in PLANS[N]) == N
    frame()
    for (n, fn, k, u) in PLANS[N]: fn(n, k)
    if M == 'labels':
        tags = [('toolless frame', (W - 2, 60, H - 2))]
        for (n, fn, k, u) in PLANS[N]:
            if n in ('blank', 'blades_b', 'blades_c'): continue
            tags.append((LABEL[n], (W - 30, RAIL_Y - T - 1, uz(k) + u * U - 5)))
        tags.append(('48 V bus', (VB_X[1] + 3, D + 18, H - 30)))
        print(labels_doc(tags), end='')
    elif M == 'cutlist':
        rho = {'5052': 2.68e-6, '6061': 2.70e-6}
        price = {'5052': 10.0, '6061': 12.0}
        print(f"# cut list - {N}U, depth {D:.0f} mm; frame/trays {T} mm 5052-H32, rails + combs 3 mm 5052, blade spines 3 mm 6061-T6")
        print(f"# (outline L x W, before slots; ESTIMATES below)")
        print(f"{'part':<26}{'qty':>4}{'L mm':>9}{'W mm':>9}{'t':>5}  mat   note")
        tot = {}
        for (n, q, l, w, t, m, note) in cutlist:
            print(f"{n:<26}{q:>4}{l:>9.1f}{w:>9.1f}{t:>5.1f}  {m:<5} {note}")
            tot.setdefault((m, t), 0.0); tot[(m, t)] += q * l * w
        kgsum = usd = 0
        for (m, t), a in sorted(tot.items()):
            kg = a * t * rho[m]; kgsum += kg; usd += kg * price[m]
            print(f"# ESTIMATE {m} {t} mm: {a/1e6:.3f} m^2 gross, ~{kg:.2f} kg, ~${kg*price[m]:.0f} raw at ${price[m]:.0f}/kg")
        fr = sum(q * l * w * t for (n, q, l, w, t, m, _) in cutlist if n.startswith(('side', 'lid', 'rail', 'cable'))) * rho['5052']
        print(f"# ESTIMATE frame only: ~{fr:.2f} kg, ~${fr*10:.0f} raw")
        print(f"# ESTIMATE total sheet: ~{kgsum:.2f} kg, ~${usd:.0f} raw stock (no slot deduction, no nesting waste, no cutting service)")
    elif M == 'checks':
        print(gpu_checks())
    else:
        print(emit(parts, 'side_l'), end='')
elif M in ('blade', 'blade_labels', 'blade_explode', 'blade_explode_labels'):
    ex = M.startswith('blade_explode')
    TRACK[0] = False
    for (n, geo, m, e, bb) in blade_geo(explode=ex):
        TRACK[0] = True; _bb(bb[0] + e[0], bb[1] + e[1], bb[2] + e[2], bb[3] + e[0], bb[4] + e[1], bb[5] + e[2]); TRACK[0] = False
        if any(e):
            geo = f"[translate {r(e[0])} {r(e[1])} {r(e[2])} {geo}]"
        part(n, geo, m)
    TRACK[0] = True
    if M.endswith('labels'):
        o = {n: e for (n, _, _, e, _) in blade_geo(explode=ex)}
        def p(n, x, y, z): e = o[n] if ex else (0, 0, 0); return (n, (x + e[0], y + e[1], z + e[2]))
        if ex:
            L = [p('spine, cold rail', 0, 20, BH), p('gap pad', 4, 87, 58), p('CM5', 8.7, 32, 18), p('carrier', 11.8, 124, 6),
                 p('DC-DC 48-5V', 19.8, 118, 10), p('rear connector', 19, BL, 20), p('front panel', 22.3, 0, -2), p('pull handle', 12, -12, 26)]
        else:
            L = [p('spine, cold rail', 0, 60, BH), p('carrier', 11.8, 20, 68), p('NVMe 2242', 14.8, 82, 68),
                 p('DC-DC 48-5V', 19.8, 104, 36), p('rear connector', 19, BL, 56), p('front panel', 22.3, 0, 82), p('pull handle', 12, -12, 26)]
        print(labels_doc(L, ghost=ex), end='')
    else:
        print(emit(parts, 'spine, cold rail'), end='')
elif M in ('chassis', 'chassis_labels'):
    N = 2; z0 = 0.0; uz = lambda k: z0 + k * U
    blade_chassis('chassis', 0, standalone=True)
    for x in (0, W): _bb(x, 0, 0, x, D + 6, 2 * U)
    _bb(slot_x(5) - 2.3, BLADE_Y0 - 70 - 12, 0, slot_x(5) + 22.3, BLADE_Y0 - 70, 2 * U)     # the pulled blade's handle
    if M == 'chassis_labels':
        b = 3.0
        print(labels_doc([('blade x8', (slot_x(7) + 22.3, 8, b + 80)), ('comb guide', (234, 80, b + 3)),
                          ('backplane', (232, MATE_Y + 11.6, b + 72)), ('switch', (214, 250, b + 8)), ('fans 3x60', (234, 305, b + 68)),
                          ('48 V busbar', (VB_X[0] + 3, D + 6, b + 12))]), end='')
    else:
        print(emit(parts, 'chassis_tray'), end='')
elif M in ('gpu', 'gpu_labels', 'gpu_section'):
    N = 3; z0 = 0.0; uz = lambda k: z0 + k * U
    info = gpu_node('gpu', 0, airflow=(M == 'gpu_section'))
    for x in (0, W): _bb(x, 0, 0, x, D + 6, 3 * U)
    if M == 'gpu_labels':
        print(labels_doc([('GPU 340 x 150 x 61', (30, 300, info['cz0'] + 30)), ('mini-ITX', (60, D - 24 - 170, info['a'] + 7.6)),
                          ('DC-DC 48-12V', (230, 90, info['a'] + 30)), ('front fans', (230, 16, info['b'] + 66)),
                          ('flex riser', (9, 150, info['a']))]), end='')
    else:
        print(emit(parts, 'gpu_tray'), end='')
elif M in ('bay', 'bay_labels'):
    d = humanoid_bay()
    if M == 'bay_labels':
        top = BASE + BPZ + BL
        print(labels_doc([('blade x4', (WALL_X + 2.7 + 2 * PITCH, 4, top + 97)), ('cold wall, pulled off', (175 + BAY_W, 0, BAY_H - 30)),
                          ('grooved cold wall', (BAY_W, BAY_D - WALL_Y, BAY_H - 40)), ('wedge clamp', (WALL_X + 2.7 + 3 * PITCH + 3, 4, top - 30)),
                          ('robot connector', (BAY_W / 2 + 11, BAY_D / 2, -14)),
                          ('K1-class torso 210x130x238', (BAY_W / 2 - 105, BAY_D / 2 - 60, 100))]), end='')
    else:
        print(emit(parts, 'base'), end='')
