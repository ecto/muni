"""gripper v2 for the rose fork-v9 tool roll (RS00 J7): EL05 + r8 pinion + two flat alu racks, 1/8" (3.175) sheet.
Z-up, mm. Top plate bolts to the v9 flange disc (Ø61, 6x Ø3.4 on PCD 27, phase 0) on top (z>90), fingers point -Z, opening along X, EL05 axis along +Y (bolted to plate_r).
usage: python3 gen_gripper.py [--tips flat|v|soft|scoop] [--can] [--boot] [--display] [--open MM]
                              [--t 3] [--r 10] [--flen 80] > out.loon
  --display  : the four tip sets laid out side by side on a plate (gripper_tips.png)
  --open MM  : nominal tip opening baked into the joint states (0..100); pose later by editing joint `state`."""
import argparse, math
ap = argparse.ArgumentParser()
ap.add_argument('--tips', default='flat'); ap.add_argument('--can', action='store_true')
ap.add_argument('--boot', action='store_true'); ap.add_argument('--display', action='store_true')
ap.add_argument('--adapter', action='store_true', help='add the optional ISO 9409 adapter disc')
ap.add_argument('--open', type=float, default=100.0)
ap.add_argument('--t', type=float, default=3.175, help='plate thickness (1/8")')
ap.add_argument('--r', type=float, default=8.0, help='pinion pitch radius')
ap.add_argument('--flen', type=float, default=80.0, help='finger length')
A = ap.parse_args()
T, R, FL = A.t, A.r, A.flen
BW, BH = 70.0, 90.0          # side plate width (X) x height (Z)
GY = 25.0                    # inner half-gap between side plates (plates at |y| = GY..GY+T)
ZP = 55.0                    # pinion axis height
TT = 6.0                     # reference tip thickness: finger inner face sits TT outboard of the contact plane
TRAVEL = 50.0
RL = TRAVEL + 2 * R + 2      # rack length: must cover pinion over full travel -> 68 at r 8
ZF0 = -25.0                  # finger bottom
ZF1 = ZF0 + FL               # finger top (= 55 for 80 mm)

f = lambda v: ('%.3f' % v).rstrip('0').rstrip('.')
def box(x0, y0, z0, x1, y1, z1):
    return '[translate %s %s %s [cube %s %s %s]]' % (f(min(x0, x1)), f(min(y0, y1)), f(min(z0, z1)), f(abs(x1 - x0)), f(abs(y1 - y0)), f(abs(z1 - z0)))
def cyl(x, y, z, r, h, ax='z'):   # base centre (x,y,z), extends +h along ax
    rot = {'z': '', 'x': '[rotate 0 90 0 %s]', 'y': '[rotate -90 0 0 %s]'}[ax]
    c = '[cylinder %s %s]' % (f(r), f(h))
    return '[translate %s %s %s %s]' % (f(x), f(y), f(z), rot % c if rot else c)
def DIFF(body, cutters):   # NB loon [difference A B] evaluates as B - A
    return '[difference %s %s]' % (U(cutters), body)
def U(items):
    s = items[-1]
    for it in reversed(items[:-1]): s = '[union %s %s]' % (it, s)
    return s

parts, inst, joints = [], [], []
def part(name, solid, mat):
    parts.append('[part "%s" %s "%s"]' % (name, solid, mat)); inst.append('[instance "%s" "%s" 0 0 0]' % (name, name))
def fixed(name, parent, at=(0, 0, 0)):
    joints.append('[fixed-joint "%s_mount" "%s" %s %s %s "%s" 0 0 0]' % (name, parent, f(at[0]), f(at[1]), f(at[2]), name))

# ---------------------------------------------------------------- fingertips (left side; right = mirror in x)
# every tip sits on the finger inner face (x = -TT at q=0) over the bottom 30 mm, contact plane x = 0 at q=0
def strip(p0, p1, z0, z1, t=1.5):   # vertical plate from p0 to p1 in plan (xy), z0..z1
    L = math.hypot(p1[0] - p0[0], p1[1] - p0[1]); a = math.degrees(math.atan2(p1[1] - p0[1], p1[0] - p0[0]))
    return '[translate %s %s 0 [rotate 0 0 %s %s]]' % (f(p0[0]), f(p0[1]), f(a), box(0, -t / 2, z0, L, t / 2, z1))
def seg_xz(p0, p1, hw, t=2.0):      # plate from p0 to p1 in elevation (xz), y -hw..hw
    L = math.hypot(p1[0] - p0[0], p1[1] - p0[1]); a = math.degrees(math.atan2(-(p1[1] - p0[1]), p1[0] - p0[0]))
    return '[translate %s 0 %s [rotate 0 %s 0 %s]]' % (f(p0[0]), f(p0[1]), f(a), box(0, -hw, -t / 2, L, hw, t / 2))
def tip_geom(kind, s):          # s = -1 left, +1 right; returns list of (suffix, solid, mat)
    X = lambda a: s * a        # a measured from contact plane outward
    zb, zt = ZF0 - 6, ZF0 + 26
    if kind == 'flat':          # 5 mm steel jaw + 1 mm serration ridges
        g = [box(X(TT), -10, zb, X(1), 10, zt)] + [box(X(1), -10, z, X(0), 10, z + 1) for z in range(int(zb) + 2, int(zt) - 1, 3)]
        return [('', U(g), 'alu')]
    if kind == 'v':             # V notch (groove along Z): jaw edges on the contact plane at y=+-10, root 4 mm back
        g = [box(X(TT), -10, zb, X(4), 10, zt)]
        for sy in (-1, 1):
            g.append(strip((X(0), sy * 10), (X(4), 0), zb, zt))
        return [('', U(g), 'alu')]
    if kind == 'soft':          # 12 mm TPU pad, rounded: box core + 2 half-round edges
        g = [box(X(TT), -10, zb + 4, X(TT - 12 + 4), 10, zt - 4), box(X(TT), -10, zb, X(TT - 8), 10, zt),
             cyl(X(TT - 8), -10, zb + 4, 4, 20, 'y') if False else box(X(TT), -6, zb + 4, X(TT - 12), 6, zt - 4)]
        g.append('[translate %s 0 %s [rotate -90 0 0 [translate 0 0 -10 [cylinder 4 20]]]]' % (f(X(TT - 8)), f(zb + 4)))
        g.append('[translate %s 0 %s [rotate -90 0 0 [translate 0 0 -10 [cylinder 4 20]]]]' % (f(X(TT - 8)), f(zt - 4)))
        return [('', U(g), 'tpu')]
    if kind == 'scoop':         # shallow spoon 40 wide: back plate + 3-segment lip curling inward under the tips
        g = [box(X(TT), -20, zb, X(TT - 2), 20, zt)]
        pts = [(TT - 1, zb), (TT - 3, zb - 8), (TT - 9, zb - 13), (TT - 17, zb - 15)]
        for (a0, z0), (a1, z1) in zip(pts, pts[1:]):
            g.append(seg_xz((X(a0), z0), (X(a1), z1), 20))
        return [('', U(g), 'steel')]
    raise SystemExit('unknown tip ' + kind)

def finger_geom(s):
    X = lambda a: s * a
    return box(X(TT), -10, ZF0, X(TT + T), 10, ZF1)

def socket(s):                  # 2x dowel 4 (12 apart, z) + magnet 8, poking 1 mm out of the finger's outer face
    X = lambda a: s * a
    zc = ZF0 + 10
    side = 'l' if s < 0 else 'r'
    out = []
    for k, dz in ((1, -6), (2, 6)):
        out.append(('dowel_%s%d' % (side, k), '[translate %s 0 %s [rotate 0 90 0 [cylinder 2 %s]]]' % (f(min(X(TT - 3), X(TT + T + 1))), f(zc + dz), f(T + 4)), 'steel'))
    out.append(('magnet_%s' % side, '[translate %s 0 %s [rotate 0 90 0 [cylinder 4 %s]]]' % (f(min(X(TT + T), X(TT + T + 1.5))), f(zc + 15), '1.5'), 'magnet'))
    return out

def rack_geom(s):               # s=-1: rack_l above pinion, runs from finger (x=-(TT+T)) toward +x
    X = lambda a: s * a
    x0 = X(TT + T)
    x1 = x0 + (-s) * -RL if False else x0 + (RL if s < 0 else -RL)
    if s < 0: zt0, zt1, zb0, zb1 = ZP + R - 1, ZP + R + 1, ZP + R + 1, ZP + R + 8
    else:     zt0, zt1, zb0, zb1 = ZP - R - 1, ZP - R + 1, ZP - R - 8, ZP - R - 1
    h = T / 2
    g = [box(x0, -h, zb0, x1, h, zb1)]
    lo, hi = min(x0, x1), max(x0, x1)
    xx = lo + 2
    while xx < hi - 2:          # teeth: module 1 -> pitch pi
        g.append(box(xx, -h, zt0, xx + 1.4, h, zt1)); xx += math.pi
    # carrier tab from rack down/up to the finger top (z = ZF1), welded to the finger's top edge
    g.append(box(x0, -10, ZF1 - 3, x0 + 6, 10, zb1) if s < 0 else box(x0, -10, zb0, x0 - 6, 10, ZF1))
    return U(g)

def pinion_geom():             # local origin = pinion centre; axis +Y; 3 mm sheet gear on an 8 mm shaft to the EL05
    g = ['[translate 0 1.5 0 [rotate 90 0 0 [cylinder %s 3]]]' % f(R - 1)]
    n = int(round(2 * R))       # module 1
    for k in range(n):
        a = 360.0 * k / n
        g.append('[rotate 0 %s 0 %s]' % (f(a), box(-0.7, -1.5, R - 1.2, 0.7, 1.5, R + 1)))
    g.append('[translate 0 %s 0 [rotate 90 0 0 [cylinder 4 %s]]]' % (f(GY + T), f(GY + T)))   # shaft y 0..28
    return U(g)

def el05_geom():               # manual sec 1.1: Ø46 x 44 body + Ø24 output boss (3 mm); STEP import came in incomplete
    y0 = GY + T
    return U(['[translate 0 %s %s [rotate -90 0 0 [cylinder 23 41]]]' % (f(y0 + 3), f(ZP)),
              '[translate 0 %s %s [rotate -90 0 0 [cylinder 12 3]]]' % (f(y0), f(ZP)),
              box(-8, y0 + 3 + 10, ZP + 22, 8, y0 + 3 + 30, ZP + 27)])     # CN1/CN2 connector block

PCD, NB, DH, RDISC = 27.0, 6, 3.4, 30.5      # rose_fork v9: disc('tool', ..., J7M['d']/2 + 2 = 30.5, pattern(RS00 out=(6, 27, 3.4), phase 0))
def bolt_xy(k): a = 2 * math.pi * k / NB; return PCD / 2 * math.cos(a), PCD / 2 * math.sin(a)
def holes(z0, h): return [cyl(*bolt_xy(k), z0 - 1, DH / 2, h + 2) for k in range(NB)]
def top_geom():                # top plate: 70 x 56 body rectangle + Ø61 round outline matching the v9 flange disc, 6x Ø3.4 PCD 27
    zb = BH
    body = U([box(-BW / 2, -GY - T, zb, BW / 2, GY + T, zb + T), cyl(0, 0, zb, RDISC, T)])
    return DIFF(body, holes(zb, T) + [cyl(0, 0, zb - 1, 6, T + 2)])
def disc_geom():               # the arm's v9 tool-flange disc (rides the RS00 output), same pattern, sits on the top plate
    zb = BH + T
    return DIFF(cyl(0, 0, zb, RDISC, T), holes(zb, T))
def screws_geom():             # 6x M3x10 SHCS from below the top plate, up through plate + disc into the RS00 output
    zb = BH
    g = []
    for k in range(NB):
        x, y = bolt_xy(k)
        g += [cyl(x, y, zb - 3, 2.75, 3), cyl(x, y, zb, 1.5, 10)]
    return U(g)
def adapter_geom():            # optional ISO 9409-1-31.5-4-M5 adapter: Ø61 x 6 alu disc above the RS00 face; 6x Ø3.4 c'bore
    zb = BH + 2 * T + 40       # (bolts onto the arm side), 4x PEM M5 nut bosses on PCD 31.5 at 45 deg + Ø6 dowel hole
    h = 6.0
    g = [cyl(0, 0, zb, RDISC, h)]
    for k in range(4):
        a = math.radians(45 + 90 * k)
        g.append(cyl(15.75 * math.cos(a), 15.75 * math.sin(a), zb + h, 4.5, 3))
    cut = holes(zb, h) + [cyl(15.75 * math.cos(math.radians(45 + 90 * k)), 15.75 * math.sin(math.radians(45 + 90 * k)), zb - 1, 2.5, h + 5) for k in range(4)]
    cut.append(cyl(0, 0, zb - 1, 3, h + 2))
    return DIFF(U(g), cut)

MATS = ['[material "alu" 0.80 0.81 0.83 0.9 0.35]', '[material "steel" 0.30 0.30 0.34 0.9 0.35]',
        '[material "el05" 0.06 0.06 0.07 0.3 0.6]', '[material "tpu" 0.86 0.52 0.30 0.0 0.8]',
        '[material "magnet" 0.12 0.12 0.14 0.6 0.4]', '[material "cam" 0.15 0.35 0.55 0.2 0.5]',
        '[material "lens" 0.05 0.05 0.08 0.9 0.1]', '[material "can" 0.62 0.63 0.66 0.9 0.3]',
        '[material "table" 0.55 0.52 0.48 0.0 0.8]', '[material "boot" 0.80 0.48 0.28 0.0 0.9]']

KIND = {'flat': 'flat', 'v': 'v', 'soft': 'soft', 'scoop': 'scoop'}
def tips_display():
    """four tip pairs side by side on a plate, each pair facing each other 30 mm apart."""
    part('plate', box(-20, -40, -3, 300, 40, 0), 'table');
    for k, kind in enumerate(['flat', 'v', 'soft', 'scoop']):
        for s, side in ((-1, 'l'), (1, 'r')):
            (_, solid, mat), = tip_geom(kind, s)
            # stand the tip on the plate: lift so zb sits on z=0, spread pairs along X
            nm = 'tip_%s_%s' % (kind, side)
            part(nm, '[translate %s 0 %s %s]' % (f(70 * k + 30 + s * 15), f(-(ZF0 - 6) + (16 if kind == 'scoop' else 0)), solid), mat)
            fixed(nm, 'plate')

def gripper():
    q = A.open / 2
    ground = 'plate_l'
    if A.can:
        part('table', box(-90, -60, -123, 90, 60, -120), 'table')
        ground = 'table'
        part('can', cyl(0, 0, -120, 33, 115), 'can')
        joints.append('[prismatic-joint "can_lift" 0 0 1 0 100 "table" 0 0 0 "can" 0 0 0]')
    # lightening windows (v2, mass <= 500 g): plate_l 40x40 round the pinion line, plate_r 50x20 below the EL05
    part('plate_l', DIFF(box(-BW / 2, -GY - T, 0, BW / 2, -GY, BH), [box(-20, -GY - T - 1, 35, 20, -GY + 1, 75)]), 'alu')
    if A.can: joints.append('[prismatic-joint "lift" 0 0 1 0 100 "table" 0 0 0 "plate_l" 0 0 0]')
    part('plate_r', DIFF(box(-BW / 2, GY, 0, BW / 2, GY + T, BH), [box(-25, GY - 1, 6, 25, GY + T + 1, 26)]), 'alu'); fixed('plate_r', 'plate_l')
    part('plate_top', top_geom(), 'alu'); fixed('plate_top', 'plate_l')
    part('plate_palm', U([box(-BW / 2, -GY - T, -T, BW / 2, -11, 0), box(-BW / 2, 11, -T, BW / 2, GY + T, 0),
                          box(-BW / 2, -GY - T, -T, -BW / 2 + 8, GY + T, 0)]), 'alu'); fixed('plate_palm', 'plate_l')
    part('flange_disc', disc_geom(), 'alu'); fixed('flange_disc', 'plate_top')
    part('screws', screws_geom(), 'steel'); fixed('screws', 'plate_top')
    if A.adapter: part('adapter_iso', adapter_geom(), 'alu'); fixed('adapter_iso', 'plate_top')
    part('el05', el05_geom(), 'el05'); fixed('el05', 'plate_r')
    part('pinion', pinion_geom(), 'steel')
    joints.append('[revolute-joint "pinion" 0 1 0 -180 180 "plate_l" 0 0 %s "pinion" 0 0 0]' % f(ZP))
    # palm camera: 12x12x8 + Ø6 lens looking -Z (along the fingers); offset to y=-18 on the palm strip so a fully
    # closed pair of fingers (which pass through the y=+-11 palm slot) cannot crush it
    part('cam', U([box(-6, -24, -T - 8, 6, -12, -T), cyl(0, -18, -T - 10, 3, 2)]), 'cam'); fixed('cam', 'plate_palm')
    for s, side in ((-1, 'l'), (1, 'r')):
        rn, fn_ = 'rack_' + side, 'finger_' + side
        part(rn, rack_geom(s), 'alu')
        joints.append('[prismatic-joint "%s" %d 0 0 0 %s "plate_l" 0 0 0 "%s" 0 0 0]' % (fn_, s, f(TRAVEL), rn))
        part(fn_, finger_geom(s), 'alu'); fixed(fn_, rn)
        for nm, solid, mat in socket(s):
            part(nm, solid, mat); fixed(nm, fn_)
        (_, solid, mat), = tip_geom(A.tips, s)
        nm = 'tip_%s_%s' % (A.tips, side)
        part(nm, solid, mat); fixed(nm, fn_)
    if A.boot:   # rover TPU boot: open-bottom 1.5 mm shell over the body + rack overhang, rounded vertical corners
        bx, by, z0, z1, w = 72.0, GY + T + 3, 30.0, BH + T + 1.5, 1.5
        g = [box(-bx, -by, z1 - w, bx, by, z1), box(-bx, -by, z0, bx, -by + w, z1), box(-bx, by - w, z0, bx, by, z1),
             box(-bx - 6, -by + 6, z0, -bx - 6 + w, by - 6, z1 - 6), box(bx + 6 - w, -by + 6, z0, bx + 6, by - 6, z1 - 6)]
        for sx in (-1, 1):
            for sy in (-1, 1):
                g.append(cyl(sx * bx, sy * (by - 6), z0, 6, z1 - z0 - 6))
            g.append('[translate %s %s %s [rotate 90 0 0 [cylinder 6 %s]]]' % (f(sx * bx), f(by - 6), f(z1 - 6), f(2 * by - 12)))
        part('boot', U(g), 'boot'); fixed('boot', 'plate_top')
    return ground

ground = 'plate' if A.display else None
if A.display: tips_display()
else: ground = gripper()
print(';; gripper.loon - GENERATED by gen_gripper.py. rack length %g, pinion r %g, plate %g, finger %g' % (RL, R, T, FL))
print('\n'.join(MATS))
print('[assembly #[%s] #[%s] #[%s] "%s"]' % ('\n '.join(parts), '\n '.join(inst), '\n '.join(joints), ground))
