"""bay.loon: bay interface mockup. World frame: bay 1 = X 0..500, Y 0..1000 (spine edge Y=0, walkway on -Y).
Bays 2-4 (X 500..2000) carry a docked bench. Everything hangs off the bench frame (assembly ground), offset BX,BY,BZ.
python3 bay.py [--nobench] > bay.loon"""
import sys, benchgen as B
cube = B.cube
BX, BY, BZ = 500, 0, 210          # bench frame origin in world: sits on its 10 mm floor plate (z 200..210)
NB = '--nobench' in sys.argv
ONE = '--one' in sys.argv   # bay 1 + 900 mm of spine only (for the detail iso)
SL = 900 if ONE else 2400
MATS = ['[material "steel" 0.45 0.46 0.48 0.7 0.5]', '[material "slot" 0.06 0.06 0.07 0.0 0.9]',
        '[material "pin" 0.85 0.85 0.88 1.0 0.15]', '[material "coupler" 0.15 0.17 0.20 0.2 0.6]',
        '[material "p48" 0.85 0.12 0.10 0.3 0.4]', '[material "pcan" 0.95 0.75 0.10 0.2 0.5]',
        '[material "pair" 0.15 0.45 0.90 0.2 0.4]', '[material "copper" 0.80 0.45 0.25 1.0 0.3]',
        '[material "spine" 0.35 0.36 0.38 0.3 0.8]', '[material "hgr" 0.55 0.57 0.60 0.9 0.3]',
        '[material "tender" 0.20 0.40 0.70 0.5 0.5]', '[material "bplate" 0.50 0.52 0.56 0.6 0.5]']
def cyl(x, y, z, r, h, axis='z'):
    rot = {'z': '', 'y': '[rotate -90 0 0 %s]', 'ny': '[rotate 90 0 0 %s]', 'x': '[rotate 0 90 0 %s]'}[axis]
    c = '[cylinder %g %g]' % (r, h)
    return '[translate %g %g %g %s]' % (x, y, z, rot % c if rot else c)
def coupler_ports(x0):   # ports on the spine face (Y=0) of a coupler spanning X x0..x0+120, z 150..210
    return {'48V': B.U([cyl(x0 + 18, 0, 185, 4, 10, 'ny'), cyl(x0 + 36, 0, 185, 4, 10, 'ny')]),
            'CAN': cube(x0 + 52, -2, 174, 16, 2, 12), 'air': cyl(x0 + 95, 0, 180, 3, 8, 'ny')}
def bay(add, bx, tag, full):
    """one bay at world X bx..bx+500. full=True -> named per the brief, else prefixed with tag."""
    n = (lambda s: s) if full else (lambda s: '%s_%s' % (tag, s))
    add(n('bay_floor'), cube(bx, 0, 0, 500, 1000, 6), 'steel')
    for k, (sx, sy) in enumerate([(60, 150), (410, 150), (60, 820), (410, 820)]):   # 4 M8 anchor slots, 12x30
        add(n('slots') if k == 0 else n('slot%d' % k), cube(bx + sx, sy, 6, 12, 30, 0.6), 'slot')
    for side, px in (('l', 50), ('r', 450)):                                         # datum pins 400 apart at Y=20
        add(n('pinblk_' + side), cube(bx + px - 20, -10, 6, 40, 60, 194), 'steel')
        add(n('datum_' + side), cyl(bx + px, 20, 200, 6, 25), 'pin')
    add(n('busbar'), cube(bx, 0, 30, 500, 30, 20), 'copper')                       # along the spine edge, z 30..50
    add(n('tap'), cube(bx + 240, 5, 50, 20, 20, 100), 'copper')                    # busbar -> coupler underside (z 150)
    add(n('coupler'), cube(bx + 190, 0, 150, 120, 60, 60), 'coupler')
    for k, s in coupler_ports(bx + 190).items(): add(n(k), s, {'48V': 'p48', 'CAN': 'pcan', 'air': 'pair'}[k])
def scene(add):
    W = lambda f: (lambda name, solid, mat: (NAMES.append(name), add(name, '[translate %g %g %g %s]' % (-BX, -BY, -BZ, solid), mat)))
    a = W(None)
    bay(a, 0, 'b1', True)
    for i in (() if ONE else (1, 2, 3)): bay(a, 500 * i, 'b%d' % (i + 1), False)
    # spine end: walkway plate, rail pair, busbar, tender plinth with plug
    a('spine', cube(-200, -620, -12, SL, 600, 12), 'spine')
    a('spine_rail_a', cube(-200, -160, 0, SL, 20, 17), 'hgr')
    a('spine_rail_b', cube(-200, -460, 0, SL, 20, 17), 'hgr')
    a('spine_bus', cube(-200, -60, 30, SL, 30, 20), 'copper')
    a('spine_bus_post', cube(-200, -60, 0, SL, 30, 30), 'spine')
    for k, (x, y) in enumerate([(140, -170), (320, -170), (140, -470), (320, -470)]):   # carriage blocks on the rails
        a('blk%d' % k, cube(x, y, 17, 40, 40, 28), 'hgr')
    a('tender_plinth', cube(100, -520, 45, 320, 420, 15), 'tender')
    a('plug_post', cube(230, -110, 60, 40, 30, 120), 'tender')
    a('plug', cube(190, -70, 150, 120, 30, 60), 'coupler')                           # 40 mm short of the coupler face: approaching
    a('plug_48V', B.U([cyl(208, -40, 185, 5, 6, 'ny'), cyl(226, -40, 185, 5, 6, 'ny')]), 'p48')
    a('plug_CAN', cube(240, -42, 172, 20, 2, 16), 'pcan')
    a('plug_air', cyl(285, -40, 180, 5, 6, 'ny'), 'pair')
    if not (NB or ONE):
        a('bench_plate', cube(500, 0, 200, 1500, 1000, 10), 'bplate')                  # pins 200..225 poke 15 through it
        a('bench_plug', cube(1190, -40, 150, 120, 40, 60), 'coupler')                  # mated: hard against bay-3 coupler
        a('bench_plug_hanger', cube(1190, -40, 210, 120, 40, 10), 'coupler')
NAMES = []
if __name__ == '__main__':
    if NB:   # bench-free: a ground plate becomes the root so we keep a single assembly
        B.cube  # noqa
    B.main(MATS, scene); open('bay_names.txt', 'w').write('\n'.join(NAMES))
