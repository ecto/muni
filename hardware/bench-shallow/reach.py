"""Reach study: which arm reaches each work point at carriage q (plinth centre X = car_x0 + q, q 0..travel).
Reached = kin.ik finds a solution inside rose v9 limits (margin >= 5 deg) for ANY tool pitch in the point's band
(doors/racks: horizontal to 30 deg down, along the plan line from the shoulder; tops: straight down to 45 deg off).
Also scans a virtual rail q -400..1300 for the minimum stroke."""
from kin import ik, margin
from bsp import car_x0, travel, sh_z, plinth_y
DOOR = [0, -10, -20, -30]; TOP = [-90, -80, -70, -60, -45]
ST = [('mill door centre (60 in)', (250, -110, 390), DOOR),
      ('bench free-area centre', (750, -190, 215), TOP), ('bench corner near-L', (540, -60, 215), TOP), ('bench corner near-R', (960, -60, 215), TOP),
      ('bench corner far-L (under oven: in front of it)', (540, -620, 215), TOP), ('bench centre (oven front edge)', (750, -330, 215), TOP),
      ('oven door centre (face)', (750, -340, 300), DOOR),
      ('winder top', (1360, -160, 365), TOP), ('stock drawer top', (1130, -210, 335), TOP), ('finished drawer top', (1130, -520, 335), TOP),
      ('tip peg 1 (z560)', (110, 345, 560), DOOR), ('tip peg 4 (z740)', (290, 345, 740), DOOR), ('tip rack z500', (200, 345, 500), DOOR), ('tip rack z800', (200, 345, 800), DOOR),
      ('parts shelf 1 mid (z300)', (750, 360, 340), DOOR), ('parts shelf 4 mid (z750)', (750, 360, 790), DOOR),
      ('parts shelf 1 left end', (100, 360, 340), DOOR), ('parts shelf 4 right end', (1400, 360, 790), DOOR),
      ('tool strip', (1320, 345, 720), DOOR)]
def ok(q, side, p, band):
    for tp in band:
        s = ik(q, side, p, tp)
        if s and margin(s) >= 5: return tp
    return None
def runs(qs):
    out = []
    for q in qs:
        if out and q - out[-1][1] <= 5: out[-1][1] = q
        else: out.append([q, q])
    return out
fmt = lambda r: ', '.join('%d..%d' % tuple(x) for x in r) if r else '-'
if __name__ == '__main__':
    mid = travel // 2
    print('carriage q 0..%d (plinth centre X = %d + q); shoulders z=%d, Y=%d; margin >= 5 deg; virtual scan q -400..1300\n' % (travel, car_x0, sh_z, plinth_y))
    print('| point | (X,Y,Z) | q=0 | q=%d | q=%d | L reach q | R reach q | both-arm q |' % (mid, travel))
    print('|---|---|---|---|---|---|---|---|')
    both_iv, one_iv, bad = [], [], []
    Q = range(-400, 1305, 5)
    for name, p, band in ST:
        who = lambda q: ''.join(k for k in 'lr' if ok(q, k, p, band) is not None).upper().replace('LR', 'L+R') or '-'
        L = [q for q in Q if ok(q, 'l', p, band) is not None]; R = [q for q in Q if ok(q, 'r', p, band) is not None]
        B = sorted(set(L) & set(R)); E = sorted(set(L) | set(R))
        inr = lambda S: [q for q in S if 0 <= q <= travel]
        flag = '' if inr(E) else ' **UNREACHABLE**' if not E else ' **only off-rail**'
        if flag: bad.append(name)
        print('| %s%s | %s | %s | %s | %s | %s | %s | %s |' % (name, flag, p, who(0), who(mid), who(travel), fmt(runs(L)), fmt(runs(R)), fmt(runs(B))))
        if E: one_iv.append((name, E))
        if B: both_iv.append((name, B))
    for lab, IV in (('one arm', one_iv), ('both arms', both_iv)):
        best = None
        for a in Q:
            b = max(min([q for q in S if q >= a], default=10**9) for _, S in IV)
            if b < 10**9 and (best is None or b - a < best[1] - best[0]): best = (a, b)
        if best: print('\nmin stroke, %s at every reachable point listed (%d pts): %d mm (q %d..%d = plinth centre X %d..%d)' % (lab, len(IV), best[1] - best[0], *best, best[0] + car_x0, best[1] + car_x0))
    print('\nunreachable / off-rail:', ', '.join(bad) or 'none')
