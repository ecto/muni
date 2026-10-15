"""Reach study for cell_v2: which arm reaches each station's work point at carriage q (plinth centre X = car_x0 + q).
A point counts as reached if kin_v2.ik finds a solution, inside rose v6 limits, for ANY tool pitch in the station's
allowed band (doors: horizontal to 30 deg down, into the door along +Y; tops: straight down to 45 deg off).
Also scans a virtual unlimited rail to find the minimum stroke that still reaches everything."""
from kin_v2 import ik, margin
from cellp import car_x0, travel
DOOR = [0, -10, -20, -30]; TOP = [-90, -80, -70, -60, -45]
ST = [('mill door (60 in)', (280, 760, 330), DOOR), ('rack face (60 in)', (750, 760, 400), DOOR),
      ('reflow door (60 in)', (1170, 760, 300), DOOR), ('stock top', (230, 200, 340), TOP),
      ('bench centre', (750, 170, 260), TOP), ('bench left end', (570, 170, 260), TOP), ('bench right end', (930, 170, 260), TOP),
      ('winder top', (1120, 200, 370), TOP), ('finished top', (1360, 190, 340), TOP)]
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
    print('carriage q 0..%d, plinth centre X = %d + q; virtual scan q -400..1300 (step 5); joint margin >= 5 deg\n' % (travel, car_x0))
    print('| station | point (X,Y,Z) | q=0 | q=435 | q=870 | L reach q | R reach q | both-arm q |')
    print('|---|---|---|---|---|---|---|---|')
    both_iv, one_iv = [], []
    for name, p, band in ST:
        def who(q):
            s = ''.join(k for k in 'lr' if ok(q, k, p, band) is not None).upper().replace('LR', 'L+R')
            return s or '-'
        Q = range(-400, 1305, 5)
        L = [q for q in Q if ok(q, 'l', p, band) is not None]; R = [q for q in Q if ok(q, 'r', p, band) is not None]
        B = sorted(set(L) & set(R)); E = sorted(set(L) | set(R))
        print('| %s | %s | %s | %s | %s | %s | %s | %s |' % (name, p, who(0), who(435), who(870), fmt(runs(L)), fmt(runs(R)), fmt(runs(B))))
        both_iv.append((name, B)); one_iv.append((name, E))
    for lab, IV in (('one arm', one_iv), ('both arms', both_iv)):
        # minimal window [a, b] that intersects every station's set
        best = None
        for a in range(-400, 1305, 5):
            b = max(min([q for q in S if q >= a], default=10**9) for _, S in IV)
            if b < 10**9 and (best is None or b - a < best[1] - best[0]): best = (a, b)
        print('\nmin stroke, %s at every station: %d mm (window q %d..%d = plinth centre X %d..%d)' % (lab, best[1] - best[0], *best, best[0] + car_x0, best[1] + car_x0))
    print('\nkey carriage positions (q): mill and oven poses used in the stills/video:')
    for q in (0, 435, 870):
        print(' q=%d: L shoulder X %d, R shoulder X %d' % (q, car_x0 + q - 200, car_x0 + q + 200))
