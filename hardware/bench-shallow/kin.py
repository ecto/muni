"""FK/IK for the upright rose fork-v9 arms on the bench_shallow plinth. World mm, Z-up, q deg, rose_fork.py signs:
+pitch swings the distal link DOWN. Native mount: zero pose along +X (the spine); heading = J1 (deg from +X, CCW from above).
Planar chain: elevation a1 = -q2, a2 = a1 - q4, a3 = a2 - q6. J5 = J7 = 0 for position. TCP = fingertip (T from wrist centre)."""
import math
from bsp import *
LIM = {'J1_yaw': (-170, 170), 'J2_sho_pitch': (-150, 120), 'J4_elbow': (-120, 115), 'J6_wrist_pitch': (-135, 135)}
def shoulder(c, side): return (car_x0 + c + (-arm_pitch / 2 if side == 'l' else arm_pitch / 2), plinth_y, sh_z)
def fk(c, side, q):
    sx, sy, sz = shoulder(c, side)
    ph = math.radians(q.get('J1_yaw', 0))
    a1 = -math.radians(q.get('J2_sho_pitch', 0)); a2 = a1 - math.radians(q.get('J4_elbow', 0)); a3 = a2 - math.radians(q.get('J6_wrist_pitch', 0))
    pts, r, h = [], 0.0, 0.0
    for Lk, a in ((L1, a1), (L2, a2), (T, a3)):
        r += Lk * math.cos(a); h += Lk * math.sin(a)
        pts.append((sx + r * math.cos(ph), sy + r * math.sin(ph), sz + h))
    return pts      # elbow, wrist centre, TCP
def wrap(a): return (a + 180) % 360 - 180
def ik(c, side, p, tool_pitch=-90.0, all_sols=False):
    """TCP at p, tool pitched tool_pitch deg in J1's vertical plane (-90 down, 0 horizontal pointing away from the shoulder)."""
    sx, sy, sz = shoulder(c, side)
    dx, dy = p[0] - sx, p[1] - sy
    r = math.hypot(dx, dy); ph = math.degrees(math.atan2(dy, dx))
    sols = []
    for flip in (False, True):          # flip: face away and reach back over the top
        q1 = wrap(ph if not flip else ph + 180); rr = r if not flip else -r
        t = math.radians(tool_pitch) if not flip else math.pi - math.radians(tool_pitch)
        wr, wh = rr - T * math.cos(t), (p[2] - sz) - T * math.sin(t)
        cq = (wr * wr + wh * wh - L1 * L1 - L2 * L2) / (2 * L1 * L2)
        if abs(cq) > 1: continue
        for s in (1, -1):
            b = s * math.acos(cq)
            a1 = math.atan2(wh, wr) - math.atan2(L2 * math.sin(b), L1 + L2 * math.cos(b))
            q = {'J1_yaw': q1, 'J2_sho_pitch': wrap(-math.degrees(a1)), 'J4_elbow': wrap(-math.degrees(b)),
                 'J6_wrist_pitch': wrap(math.degrees(a1 + b - t))}
            if all(LIM[k][0] <= q[k] <= LIM[k][1] for k in q):
                e = fk(c, side, q)[0]; sols.append((-e[2] + (1000 if flip else 0), q))
    if all_sols: return [q for _, q in sols]
    if not sols: return None
    sols.sort(key=lambda x: x[0]); return sols[0][1]
def margin(q): return min(min(q[k] - LIM[k][0], LIM[k][1] - q[k]) for k in q if k in LIM)
if __name__ == '__main__':
    import random
    for _ in range(3000):
        p = (random.uniform(0, 1500), random.uniform(-800, 500), random.uniform(100, 900)); tp = random.uniform(-90, 0)
        for q in ik(400, 'l', p, tp, all_sols=True):
            e = fk(400, 'l', q)[2]; assert max(abs(a - b) for a, b in zip(e, p)) < 1e-6, (p, q, e)
    print('IK round trip ok; shoulder z', sh_z, 'T', T, 'travel', travel)
