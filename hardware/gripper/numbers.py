"""grip force / payload / mass budget for gripper v2 (fork v9 RS00 tool roll). EL05 numbers from its user manual (sec 1.3, 1.4)."""
import math
g, mu, SF = 9.81, 0.5, 2.0
T_RATED, T_PEAK, M_EL05 = 1.8, 6.0, 0.242          # N.m continuous (70x70 heat sink), N.m peak, kg
r = 0.008; t = 0.003175; AL = 2700                  # pinion pitch radius, 1/8" sheet, 6061
print('EL05 (manual): rated 1.8 N.m continuous @100 rpm (70x70 mm heat sink), peak 6 N.m, 9:1, 430 rpm no-load, 242 g, Ø46 x 44')
print(f'pinion pitch r = {1000 * r:g} mm (Ø{2000 * r:g}, module 1, 16 T); sheet t = {1000 * t} mm')
for name, T in (('rated', T_RATED), ('peak', T_PEAK)):
    F_tot = T / r; F = F_tot / 2
    m = 2 * mu * F / (g * SF)
    print(f'{name:5s}: T={T} N.m -> pinion force {F_tot:.0f} N -> squeeze {F:.1f} N per finger -> max lift {m:.1f} kg (mu {mu}, 2 faces, SF {SF})')
print('  (no rack/pinion friction loss; at eta 0.8 scale by 0.8. Hold on rated, not peak: stall heating.)')
print()
print('needed squeeze per finger F = m g SF / (2 mu), EL05 torque T = 2 r F:')
for name, m in (('Ø66 alu can (empty)', 0.015), ('500 ml bottle', 0.55), ('1/8" 5052 plate 300x200', 0.300 * 0.200 * t * 2680)):
    F = m * g * SF / (2 * mu); Tq = 2 * r * F
    print(f'  {name:26s} {m:5.3f} kg: F {F:5.2f} N  T {Tq:.3f} N.m = {100 * Tq / T_RATED:4.1f}% of rated')
print('  NB rated squeeze (112 N) WILL crush an empty can -> run the EL05 current-limited.')
print()
A = lambda w, h: w * h * 1e-6
circ = lambda d: math.pi * (d / 2) ** 2 * 1e-6
# exact-ish: area of Ø61 disc outside the 70 x 56.35 rectangle = 2 circular segments of chord at y = +-28.175
R6 = 30.5; h = R6 - 28.175; seg = R6 ** 2 * math.acos((R6 - h) / R6) - (R6 - h) * math.sqrt(2 * R6 * h - h * h)
top = A(70, 56.35) + 2 * seg * 1e-6 - 6 * circ(3.4) - circ(12)
sheet = {'plate_l + plate_r 70x90': 2 * A(70, 90), 'plate_top (70x56 + Ø61 lobes, holes)': top,
         'plate_palm': 2 * A(70, 17) + A(8, 22), 'fingers 20x80 x2': 2 * A(20, 80)}
WIN = A(40, 40) + A(50, 20)                         # windows: plate_l 40x40 (z 35..75), plate_r 50x20 under the EL05 (z 6..26)
racks = 2 * (A(68, 7) + A(6, 20)) * t * AL           # 6061, 1/8"
pinion = (math.pi * 0.007 ** 2 * t + math.pi * 0.004 ** 2 * 0.028) * 7850   # steel gear + shaft (kept steel: wear)
tips = 2 * 0.020 * 0.032 * 0.005 * AL               # flat tips, 6061
screws = 6 * 0.0006                                  # M3x10 SHCS ~0.6 g
misc = 0.010 + 0.004 + 0.015                         # UVC cam module, magnets + dowels, fasteners/bearings allowance
rows = [(k, v * t * AL) for k, v in sheet.items()] + [('alu racks x2 (6061)', racks), ('steel pinion Ø16 + Ø8 shaft', pinion),
        ('flat alu tips x2 (6061)', tips), ('EL05', M_EL05), ('6x M3x10 SHCS', screws), ('cam + magnets + dowels + fasteners', misc)]
tot = sum(v for _, v in rows)
print('mass at the flange (5052 sheet at 2700 kg/m3, 6061 racks/tips):')
for k, v in rows: print(f'  {k:40s} {1000 * v:6.1f} g')
print(f'  {"TOTAL (flat tips, no windows)":40s} {1000 * tot:6.1f} g')
w = WIN * t * AL
print(f'  {"- windows: plate_l 40x40, plate_r 50x20":40s} {-1000 * w:6.1f} g')
print(f'  {"TOTAL with windows":40s} {1000 * (tot - w):6.1f} g   (target <= 500 g)')
print('  (the v9 flange disc Ø61 x 3.175 belongs to the arm, not counted; ISO adapter if fitted: +%.0f g)' % (1000 * (circ(61) * 0.006 - 6 * circ(3.4) * 0.006) * AL + 4 * 1.5))
