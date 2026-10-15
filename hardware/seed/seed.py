"""seed.loon: a bench + everything it needs to grow, on regolith. World = bench frame frame (bench X 0..1500, Y 0..1000,
doors on the -Y face, swung open). python3 seed.py [NBARS] > seed.loon   (NBARS = bench_2_frame bars present, default 6)"""
import sys, math, benchgen as B
cube, U = B.cube, B.U
B.DOORS_OPEN = True
NBARS = int(sys.argv[1]) if len(sys.argv) > 1 else 6
MATS = ['[material "regolith" 0.62 0.56 0.47 0.0 0.95]', '[material "rock" 0.45 0.42 0.38 0.0 0.9]',
        '[material "genome" 0.80 0.62 0.12 0.8 0.3]', '[material "label" 0.95 0.95 0.92 0.0 0.6]',
        '[material "crate" 0.60 0.45 0.28 0.0 0.8]', '[material "rs" 0.15 0.15 0.17 0.7 0.4]',
        '[material "pcb" 0.10 0.45 0.20 0.1 0.5]', '[material "solar" 0.08 0.12 0.30 0.6 0.15]',
        '[material "battery" 0.85 0.75 0.15 0.2 0.5]', '[material "cable" 0.05 0.05 0.05 0.0 0.8]',
        '[material "hopper" 0.55 0.58 0.60 0.7 0.4]', '[material "sinter" 0.30 0.30 0.32 0.6 0.5]',
        '[material "lens" 0.70 0.85 0.95 0.2 0.05]', '[material "brick" 0.55 0.35 0.25 0.0 0.9]',
        '[material "black" 0.05 0.05 0.05 0.0 0.8]', '[material "white" 0.95 0.95 0.95 0.0 0.6]']
cyl = lambda x, y, z, r, h: '[translate %g %g %g [cylinder %g %g]]' % (x, y, z, r, h)
def bar(p, q, s=40):   # 4040 bar between two points (axis-aligned or not): box of length |q-p| rotated into place
    dx, dy, dz = q[0] - p[0], q[1] - p[1], q[2] - p[2]; L = math.sqrt(dx * dx + dy * dy + dz * dz)
    pitch = math.degrees(math.atan2(dz, math.hypot(dx, dy))); yaw = math.degrees(math.atan2(dy, dx))
    b = '[translate 0 %g %g [cube %g %g %g]]' % (-s / 2, -s / 2, L, s, s)
    return '[translate %g %g %g [rotate 0 0 %g [rotate 0 %g 0 %s]]]' % (p[0], p[1], p[2], yaw, -pitch, b)
# bench_2 frame: 12 bars of a 1500x1000x1000 box, 2 m in front of the doors (Y -3000..-2000); assembly order
B2X, B2Y = 0, -3000
def b2_bars():
    x0, y0, X, Y, H = B2X, B2Y, 1500, 1000, 1000
    return [cube(x0, y0, 0, X, 40, 40), cube(x0, y0 + Y - 40, 0, X, 40, 40), cube(x0, y0 + 40, 0, 40, Y - 80, 40),
            cube(x0 + X - 40, y0 + 40, 0, 40, Y - 80, 40),                                         # 4 bottom
            cube(x0, y0, 40, 40, 40, H - 80), cube(x0 + X - 40, y0, 40, 40, 40, H - 80),            # 2 front posts
            cube(x0, y0 + Y - 40, 40, 40, 40, H - 80), cube(x0 + X - 40, y0 + Y - 40, 40, 40, 40, H - 80),
            cube(x0, y0, H - 40, X, 40, 40), cube(x0, y0 + Y - 40, H - 40, X, 40, 40),
            cube(x0, y0 + 40, H - 40, 40, Y - 80, 40), cube(x0 + X - 40, y0 + 40, H - 40, 40, Y - 80, 40)]
ROCKS = [(-2000, -1500, 180), (3200, -2200, 250), (4600, 600, 120), (-1800, 2600, 150), (2400, -500, 100), (4200, 2700, 200)]
def scene(a):
    a('regolith', cube(-2500, -3200, -20, 8000, 6000, 20), 'regolith')
    for k, (x, y, r) in enumerate(ROCKS): a('rock%d' % k, '[translate %g %g %g [sphere %g]]' % (x, y, -0.55 * r, r), 'rock')
    # genome: sealed 200x150x100 box bolted to the outside of the left (X=0) side panel, label plate on its face
    a('genome', cube(-106, 420, 420, 100, 200, 150), 'genome')
    a('genome_tag', cube(-108, 450, 470, 2, 140, 50), 'label')
    # bought-parts crate 600x400x400, actuators + boards on the lid
    cx, cy = -1300, 150
    a('bought_parts', cube(cx, cy, 0, 600, 400, 400), 'crate')
    for k, (x, y, r, h) in enumerate([(80, 100, 50, 60), (200, 100, 50, 60), (80, 260, 50, 60), (330, 110, 30, 40), (330, 220, 30, 40)]):
        a('rs%d' % k, cyl(cx + x, cy + y, 400, r, h), 'rs')       # 3x RS04-class (d100x60) + 2x RS00-class (d60x40)
    a('board0', cube(cx + 400, cy + 60, 400, 150, 100, 2), 'pcb')
    a('board1', cube(cx + 410, cy + 200, 400, 120, 90, 2), 'pcb')
    a('board2', cube(cx + 400, cy + 200, 402, 100, 70, 2), 'pcb')
    # collector: 2000x1000 panel on a 4040 A-frame, tilted 30 deg toward -Y; battery + cable to the bench
    px, py = -2300, 1700
    a('collector', '[translate %g %g 600 [rotate 30 0 0 [cube 2000 1000 12]]]' % (px, py), 'solar')
    hy = math.cos(math.radians(30)) * 1000; hz = 600 + math.sin(math.radians(30)) * 1000
    legs = []
    for x in (px + 100, px + 1860):
        legs += [bar((x + 20, py, 0), (x + 20, py, 600)), bar((x + 20, py + hy, 0), (x + 20, py + hy, hz)),
                 bar((x + 20, py + 200, 0), (x + 20, py + hy - 50, hz - 30))]
    legs.append(cube(px + 100, py - 20, 560, 1800, 40, 40))
    a('collector_frame', U(legs), 'alu')
    a('battery', cube(-700, 1450, 0, 300, 200, 150), 'battery')
    a('cable', U([cube(-1200, 1540, 0, 500, 20, 20), cube(-400, 1540, 0, 330, 20, 20), cube(-90, 700, 0, 20, 860, 20),
                  cube(-90, 700, 0, 84, 20, 20)]), 'cable')
    # regolith hopper -> screw-feed tube -> sinter bay (lens on top) -> two bricks
    a('hopper', cube(2200, 1500, 0, 500, 500, 600), 'hopper')
    a('feed', '[translate 2700 1750 420 [rotate 0 90 0 [cylinder 40 400]]]', 'hopper')
    a('sinter', cube(3100, 1550, 0, 400, 400, 400), 'sinter')
    a('lens', cyl(3300, 1750, 400, 50, 20), 'lens')
    a('brick0', cube(3560, 1580, 0, 200, 100, 50), 'brick')
    a('brick1', cube(3560, 1700, 0, 200, 100, 50), 'brick')
    # bench_2 being assembled
    bars = b2_bars()[:NBARS]
    if bars: a('bench_2_frame', U(bars), 'alu')
    # 1 m scale bar, alternating 100 mm black/white
    a('scale_1m', U([cube(1200 + 200 * i, -1300, 0, 100, 60, 8) for i in range(5)]), 'black')
    a('scale_w', U([cube(1300 + 200 * i, -1300, 0, 100, 60, 8) for i in range(5)]), 'white')
if __name__ == '__main__':
    B.main(MATS, lambda add_part: scene(lambda n, s, m: add_part(n, s, m)))
