"""FK/IK for the UPRIGHT rose fork-v6 arms on the cell_v2 plinth. World mm, Z-up, q in deg with rose_fork.py signs.
rose_fork: pitch axes are +Y (link frame), rot() is right-handed => POSITIVE pitch swings the distal link DOWN.
The arm is mounted turned +90 deg about Z so its zero pose points +Y (into the back lane); J1 positive = CCW from above.
Planar chain: elevation a1 = -q2, a2 = a1 - q4, a3 = a2 - q6; heading = 90 + q1 (deg from +X). J5 = J7 = 0 for position."""
import math
from cellp import *
L1, L2, T = 260.0, 220.0, 222.0    # rose_fork UPPER, FORE, wrist centre -> flange/TCP (rose summary: flange 702 from shoulder)
LIM = {'J1_yaw': (-170, 170), 'J2_sho_pitch': (-150, 120), 'J4_elbow': (-170, 150), 'J6_wrist_pitch': (-150, 150)}
def shoulder(c, side):
    return (car_x0 + c + (-arm_pitch / 2 if side == 'l' else arm_pitch / 2), aisle_y, sh_z)
def fk(c, side, q):
    sx, sy, sz = shoulder(c, side)
    ph = math.radians(90 + q.get('J1_yaw', 0))
    a1 = -math.radians(q.get('J2_sho_pitch', 0)); a2 = a1 - math.radians(q.get('J4_elbow', 0)); a3 = a2 - math.radians(q.get('J6_wrist_pitch', 0))
    pts, r, h = [], 0.0, 0.0
    for L, a in ((L1, a1), (L2, a2), (T, a3)):
        r += L * math.cos(a); h += L * math.sin(a)
        pts.append((sx + r * math.cos(ph), sy + r * math.sin(ph), sz + h))
    return pts      # elbow, wrist centre, TCP
def wrap(a): return (a + 180) % 360 - 180
def ik(c, side, p, tool_pitch=-90.0, elbow_up=True, all_sols=False):
    """TCP at p, tool pitched tool_pitch deg in J1's vertical plane (-90 straight down, 0 horizontal outward)."""
    sx, sy, sz = shoulder(c, side)
    dx, dy = p[0] - sx, p[1] - sy
    r = math.hypot(dx, dy); ph = math.degrees(math.atan2(dy, dx))
    sols = []
    for flip in (False, True):          # flip: face away and reach back over the top
        head = ph if not flip else ph + 180
        rr = r if not flip else -r
        q1 = wrap(head - 90)
        t = math.radians(tool_pitch) if not flip else math.pi - math.radians(tool_pitch)
        wr, wh = rr - T * math.cos(t), (p[2] - sz) - T * math.sin(t)
        cq = (wr * wr + wh * wh - L1 * L1 - L2 * L2) / (2 * L1 * L2)
        if abs(cq) > 1: continue
        for s in (1, -1):
            b = s * math.acos(cq)        # elbow bend (elevation increment a2-a1)
            a1 = math.atan2(wh, wr) - math.atan2(L2 * math.sin(b), L1 + L2 * math.cos(b))
            a2 = a1 + b
            q = {'J1_yaw': q1, 'J2_sho_pitch': wrap(-math.degrees(a1)), 'J4_elbow': wrap(-math.degrees(b)),
                 'J6_wrist_pitch': wrap(math.degrees(a2 - t))}
            if all(LIM[k][0] <= q[k] <= LIM[k][1] for k in q):
                e = fk(c, side, q)[0]
                sols.append(((-e[2] if elbow_up else e[2]) + (1000 if flip else 0), q))
    if all_sols: return [q for _, q in sols]
    if not sols: return None
    sols.sort(key=lambda x: x[0]); return sols[0][1]
def margin(q): return min(min(q[k] - LIM[k][0], LIM[k][1] - q[k]) for k in q if k in LIM)
if __name__ == '__main__':
    print('shoulder z =', sh_z, 'travel', travel, 'car_x0', car_x0, 'rail', rail_x0, rail_l)
    import random
    for _ in range(2000):   # IK/FK round trip
        p = (random.uniform(0, 1500), random.uniform(0, 1000), random.uniform(200, 900)); tp = random.uniform(-90, 0)
        q = ik(435, 'l', p, tp)
        if q:
            e = fk(435, 'l', q)[2]; assert max(abs(a - b) for a, b in zip(e, p)) < 1e-6, (p, q, e)
    print('IK round trip ok')
