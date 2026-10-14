#!/usr/bin/env python3
"""K1 arm reach check (collision-checked against the torso and pelvis boxes) for the pack handle (torso-local frame of rose/mini/cad/k1.loon, Z-up, mm).
Chain from k1.loon: sh-pitch (Y) at (+-132,0,254) -> sh-roll (X) -> elbow (Y) 180 below -> wrist yaw (Z) 170 below
-> hand-ball centre 19 below the wrist (369 shoulder->ball, per the K1 ball-cuff work).
Prints, for each pull-path point, the best joint solution and its margin to the joint limits."""
import math, itertools, sys
L1, L2 = 180.0, 189.0
LIM = {'R': dict(p=(-180, 60), r=(-100, 15), e=(-120, 5)), 'L': dict(p=(-180, 60), r=(-15, 100), e=(-120, 5))}
SH = {'R': (132.0, 0.0, 254.0), 'L': (-132.0, 0.0, 254.0)}
def rx(a, v): c, s = math.cos(a), math.sin(a); x, y, z = v; return (x, c*y - s*z, s*y + c*z)
def ry(a, v): c, s = math.cos(a), math.sin(a); x, y, z = v; return (c*x + s*z, y, -s*x + c*z)
def fk(side, p, r, e):
    """Physical K1 convention: shoulder pitch about the LATERAL axis (X here; negative = arm raised forward),
    shoulder roll about Y (negative = right arm abducts outward, +X; mirrored for the left), elbow flex about X
    (negative = forearm forward). k1.loon names these joints the same but puts pitch on its Y axis, which with the
    shoulders on +-X would swing the arm sideways; see README."""
    p, r, e = map(math.radians, (p, r, e))
    fa = rx(e, (0, 0, -L2)); v = (fa[0], fa[1], fa[2] - L1)
    v = rx(p, ry(r, v)); el = rx(p, ry(r, (0, 0, -L1)))
    s = SH[side]; return tuple(s[i] + v[i] for i in range(3)), tuple(s[i] + el[i] for i in range(3))
BOXES = [(-107, -65, 44, 107, 65, 282), (-92, -60, -46, 92, 60, 44)]   # torso shell + pelvis (k1.loon, widened 214)
def hits(a, b, rad=20.0):
    for k in range(1, 11):
        t = k / 10; q = [a[i] + t * (b[i] - a[i]) for i in range(3)]
        for (x0, y0, z0, x1, y1, z1) in BOXES:
            if x0 - rad < q[0] < x1 + rad and y0 - rad < q[1] < y1 + rad and z0 - rad < q[2] < z1 + rad:
                return True
    return False
def solve(side, tgt, step=3):
    lim = LIM[side]; best = None
    for p in range(lim['p'][0], lim['p'][1] + 1, step):
        for r in range(lim['r'][0], lim['r'][1] + 1, step):
            for e in range(lim['e'][0], lim['e'][1] + 1, step):
                h, el = fk(side, p, r, e)
                d = math.dist(h, tgt)
                if best is not None and d >= best[0]: continue
                if hits(SH[side], el, 22) or hits(el, h, 12): continue
                if True:
                    m = min(p - lim['p'][0], lim['p'][1] - p, r - lim['r'][0], lim['r'][1] - r, e - lim['e'][0], lim['e'][1] - e)
                    best = (d, p, r, e, m, el)
    return best
if __name__ == '__main__':
    print("# REJECTED: handle on the torso FRONT (pack drawn out along -Y). target x 81, z 92, pull y -83 -> -208")
    for side in 'RL':
        for y in (-83, -208):
            d, p, r, e, m, el = solve(side, (81.0, y, 92.0))
            print(f"  {side} hand y {y:5.0f}: miss {d:5.1f} mm  (pitch {p} roll {r} elbow {e}, limit margin {m} deg)")
    print("# CHOSEN: flank handle, right pack A drawn out along +X. grip z 56 (pack floor + 8), y -35 (front third of the flank)")
    print("#   seated grip x 134; pulled onto the dock stop x 205 (pack slides 71 mm)")
    for x in (134, 150, 170, 190, 205):
        d, p, r, e, m, el = solve('R', (float(x), -35.0, 56.0), step=2)
        print(f"  R hand x {x:4d}: miss {d:4.1f} mm  pitch {p:4d} roll {r:4d} elbow {e:5d}  limit margin {m:3d} deg")
    print("#   capture box at the seated grip (+-y, +-z 10 mm):")
    for dy, dz in ((-10, 0), (10, 0), (0, -10), (0, 10)):
        d, p, r, e, m, el = solve('R', (134.0, -35.0 + dy, 56.0 + dz), step=2)
        print(f"  dy {dy:4d} dz {dz:4d}: miss {d:4.1f} mm  limit margin {m:3d} deg")
    print("# ALTERNATIVE: grip 36 mm lower (z 20, y -40), i.e. packs dropped into the pelvis transition")
    for x in (134, 170, 205):
        d, p, r, e, m, el = solve('R', (float(x), -40.0, 20.0), step=2)
        print(f"  R hand x {x:4d}: miss {d:4.1f} mm  limit margin {m:3d} deg")
