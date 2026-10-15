"""Reach study for cell_real (hanging fork-v5 arms). Spherical wrist: J1/J2/J4 place the wrist centre,
J5/J6/J7 orient the tool; feasible if wrist IK is inside limits and the forearm->tool bend <= 120 deg (J6)."""
import math
from kin_real import L1, L2, T, LIM, shoulder
def sub(a, b): return tuple(x - y for x, y in zip(a, b))
def unit(a): n = math.sqrt(sum(x * x for x in a)); return tuple(x / n for x in a)
def wrist_ik(c, side, w):
    sx, sy, sz = shoulder(c, side); dx, dy = w[0] - sx, w[1] - sy; dh = w[2] - sz
    out = []
    for flip in (False, True):
        phi = math.atan2(dy, dx) + (math.pi if flip else 0); r = math.hypot(dx, dy) * (-1 if flip else 1)
        q1 = (-math.degrees(phi) + 180) % 360 - 180
        cq = (r * r + dh * dh - L1 * L1 - L2 * L2) / (2 * L1 * L2)
        if abs(cq) > 1: continue
        for s in (1, -1):
            q4 = s * math.acos(cq); q2 = math.atan2(dh, r) - math.atan2(L2 * math.sin(q4), L1 + L2 * math.cos(q4))
            q = {'J1_yaw': q1, 'J2_sho_pitch': (math.degrees(q2) + 180) % 360 - 180, 'J4_elbow': math.degrees(q4)}
            if all(LIM[k][0] <= q[k] <= LIM[k][1] for k in q):
                e = (sx + L1 * math.cos(q2) * math.cos(phi), sy + L1 * math.cos(q2) * math.sin(phi), sz + L1 * math.sin(q2))
                out.append((q, e))
    return out
def feasible(c, side, p, d):
    d = unit(d); w = tuple(pi - T * di for pi, di in zip(p, d))
    for q, e in wrist_ik(c, side, w):
        f = unit(sub(w, e)); bend = math.degrees(math.acos(max(-1, min(1, sum(a * b for a, b in zip(f, d))))))
        if bend <= 120 and w[2] > 140: return q, bend     # wrist centre must stay above the deck
    return None
DOWN = [(0, 0, -1)]
def front(maxtilt=45): return [(0, math.cos(math.radians(a)), -math.sin(math.radians(a))) for a in range(0, maxtilt + 1, 5)]
TARGETS = [  # name, TCP point, approach set, note
    ('mill', (280, 380, 260), front(45), 'through the -Y door, 60 mm inside, bed height'),
    ('stock', (230, 160, 200), DOWN, 'top-down into drawer centre'),
    ('plate', (750, 500, 150), DOWN, 'top-down, plate centre'),
    ('plate_far', (580, 320, 150), DOWN, 'top-down, plate front-left corner'),
    ('winder', (1120, 200, 280), DOWN, 'top-down on winder top'),
    ('reflow', (1170, 480, 220), front(30), 'through the -Y door, 80 mm inside'),
    ('reflow_top', (1170, 550, 330), DOWN, 'top-down onto oven top (if top-loading)'),
    ('finished', (1340, 210, 200), DOWN, 'top-down into drawer centre'),
]
def ok(c, p, ds):
    return [s for s in 'lr' if any(feasible(c, s, p, d) for d in ds)]
def disc(z, ds, c=500, side='l'):
    sx, sy, _ = shoulder(c, side); rs = [r for r in range(0, 800, 5) if any(feasible(c, side, (sx + r * math.cos(a), sy + r * math.sin(a), z), d) for d in ds for a in (0, 1.0))]
    return (min(rs), max(rs)) if rs else None
ANY = [unit((math.cos(a) * math.cos(b), math.sin(a) * math.cos(b), math.sin(b))) for a in [i * math.pi / 6 for i in range(12)] for b in [math.radians(x) for x in range(-90, 91, 15)]]
if __name__ == '__main__':
    print('reach disc about one shoulder (horizontal TCP radius, mm):')
    for z in (130, 400):
        print('  z=%d  top-down %s   any orientation %s' % (z, disc(z, DOWN), disc(z, ANY)))
    print('\nfixture reachability (arms able) at carriage 0 / 500 / 1000, and the carriage interval that reaches it:')
    need = {}
    for n, p, ds, note in TARGETS:
        row = ['%-5s' % (''.join(ok(c, p, ds)) or '-') for c in (0, 500, 1000)]
        cs = [c for c in range(-300, 1301, 10) if ok(c, p, ds)]
        iv = (min(cs), max(cs)) if cs else None; need[n] = iv
        print('  %-11s %s   c in %s   (%s)' % (n, ' '.join(row), iv, note))
    lo = max(iv[0] for iv in need.values()); hi = min(iv[1] for iv in need.values())
    print('\nminimal stroke must touch every interval: needs c from %d to %d -> %d mm of travel' % (min(iv[1] for iv in need.values()), max(iv[0] for iv in need.values()), max(iv[0] for iv in need.values()) - min(iv[1] for iv in need.values())))
