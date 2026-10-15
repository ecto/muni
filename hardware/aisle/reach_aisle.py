"""For each station: scan lift 0..1500 (step 10) x tool pitch -90..+30 (step 5), tender_x = shoulder over the target X.
Reports the lift window (margin >= 5 deg, elbow+wrist above the floor) and whether 0..600 covers it."""
from kin_aisle import *
ST = [  # name, TCP target (X,Y,Z), note
 ('rack top shelf (z1700, 100 in)', (3500, 1860, 1730), 'back lane B7-B8, shelf front Y1760'),
 ('rack bottom shelf (z200, 100 in)', (3500, 1860, 230), ''),
 ('saw table (z926, 140 in)', (500, 1900, 950), 'B1-B2'),
 ('mill door (80 in, mid-door)', (2250, 1840, 880), 'Carvera on 700 stand, door z726..1106'),
 ('assembly bench top (z840, 200 in)', (3250, 740, 860), 'F6-F8 slab z800..840, Y340..940'),
 ('bench front-lane work pt z286', (1750, 830, 286), 'bench (750,170,260) local'),
 ('bench front-lane work pt z476', (1750, 800, 476), 'bench HANDOFF height (750,200,450) local'),
 ('sheet router bed (z526)', (1500, 1950, 560), 'B3-B4'),
]
import math
def scan(p, side='l'):
    x = x_for(p[0], side); ok = []
    for lift in range(0, 1501, 10):
        best = None
        for tp in range(-90, 31, 5):
            for q in ik(x, lift, side, p, tp, all_sols=True) or []:
                if margin(q) < 5: continue
                e, w, t = fk(x, lift, side, q)
                if min(e[2], w[2]) < 60: continue
                best = (tp, q); break
            if best: break
        if best: ok.append((lift, best[0]))
    return x, ok
def runs(v):
    out = []; 
    for l in v:
        if out and l - out[-1][1] <= 10: out[-1][1] = l
        else: out.append([l, l])
    return out
if __name__ == '__main__':
    print('| station | TCP target | tender_1_x | lift window (mm) | needs | within 0..600? |')
    print('|---|---|---|---|---|---|')
    for n, p, note in ST:
        x, ok = scan(p)
        w = runs([l for l, _ in ok]); lifts = [l for l, _ in ok]
        need = ('%d (at tool %d deg)' % (lifts[0], ok[0][1])) if ok else 'unreachable <=1500'
        fit = [l for l in lifts if l <= 600]
        print('| %s | (%d,%d,%d) | %d | %s | %s | %s |' % (n, *p, x, ', '.join('%d..%d' % tuple(r) for r in w) or '-', need,
              ('yes (%d..%d)' % (fit[0], fit[-1])) if fit else 'NO'))
