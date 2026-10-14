"""FK/IK for the hanging rose fork-v5 arms in cell_real. World mm, Z-up. q in deg, rose_fork signs.
Hanging => pitch angles add in the vertical plane with POSITIVE = toward the ceiling."""
import math
L1, L2, T = 260.0, 220.0, 222.0     # shoulder->elbow, elbow->wrist centre, wrist centre->TCP (flange 702 from shoulder)
SH_Z, ARM_DX, RAIL_X0, ARM_Y = 640.0, 200.0, 250.0, 500.0
LIM = {'J1_yaw': (-170, 170), 'J2_sho_pitch': (-150, 65), 'J4_elbow': (-170, 125), 'J6_wrist_pitch': (-120, 120)}
def shoulder(c, side):
    return (RAIL_X0 + c + (-ARM_DX if side == 'l' else ARM_DX), ARM_Y, SH_Z)
def fk(c, side, q):
    """returns (elbow, wrist, tcp) world points; J5=J7=0 assumed for position."""
    sx, sy, sz = shoulder(c, side)
    phi = -math.radians(q.get('J1_yaw', 0))           # axis is -Z
    a1 = math.radians(q.get('J2_sho_pitch', 0)); a2 = a1 + math.radians(q.get('J4_elbow', 0)); a3 = a2 + math.radians(q.get('J6_wrist_pitch', 0))
    pts, r, h = [], 0.0, 0.0
    for L, a in ((L1, a1), (L2, a2), (T, a3)):
        r += L * math.cos(a); h += L * math.sin(a)
        pts.append((sx + r * math.cos(phi), sy + r * math.sin(phi), sz + h))
    return pts
def ik(c, side, p, tool_pitch=-90.0, elbow_up=True):
    """TCP at p with tool direction pitched tool_pitch deg in the vertical plane through J1 (-90 = straight down,
    0 = horizontal outward). Returns q dict or None (unreachable / out of limits)."""
    sx, sy, sz = shoulder(c, side)
    dx, dy = p[0] - sx, p[1] - sy
    phi = math.atan2(dy, dx); r = math.hypot(dx, dy)
    sols = []
    for flip in (False, True):
        ph, rr = (phi, r) if not flip else (phi + math.pi, -r)
        q1 = -math.degrees(ph); q1 = (q1 + 180) % 360 - 180
        t = math.radians(tool_pitch) if not flip else math.pi - math.radians(tool_pitch)
        # in the (rr,h) plane; when flipped, outward direction is reversed
        wr, wh = rr - T * math.cos(t), (p[2] - sz) - T * math.sin(t)
        d2 = wr * wr + wh * wh
        cq = (d2 - L1 * L1 - L2 * L2) / (2 * L1 * L2)
        if abs(cq) > 1: continue
        for s in (1, -1):
            q4 = s * math.acos(cq)
            q2 = math.atan2(wh, wr) - math.atan2(L2 * math.sin(q4), L1 + L2 * math.cos(q4))
            q6 = t - q2 - q4
            q = {'J1_yaw': q1, 'J2_sho_pitch': math.degrees(q2), 'J4_elbow': math.degrees(q4), 'J6_wrist_pitch': math.degrees(q6)}
            for k in ('J2_sho_pitch', 'J6_wrist_pitch'):
                q[k] = (q[k] + 180) % 360 - 180
            if all(LIM[k][0] <= q[k] <= LIM[k][1] for k in q):
                # elbow_up: elbow higher than the shoulder-wrist line
                e = fk(c, side, q)[0]
                sols.append((-(e[2]) if elbow_up else e[2], q))
    if not sols: return None
    sols.sort(key=lambda x: x[0]); return sols[0][1]
def margin(q):
    return min(min(q[k] - LIM[k][0], LIM[k][1] - q[k]) for k in q if k in LIM)
