"""SBR16 block loads for the bench_shallow plinth: 4 blocks at (+-block_pitch_x/2, +-rail_gap/2) about the plinth centre.
Masses: rose_fork.py v9 prim_mass per link + vendor actuators (J1/J2/J4 RS04, J5/J6 RS02, J7 RS00) + gripper v2 485 g.
+F = compression, -F = lift-off. Compared at rail_gap 220 (this box) vs 300 (cell_v2)."""
import sys
sys.path.insert(0, __import__('os').path.join(__import__('os').environ.get('ROSE_DIR', __import__('os').path.expanduser('~/Developer/rose')), 'arm', 'fork'))
import rose_fork as F
from bsp import *
g = 9.81; st = {}
for p in F.build():
    m = F.prim_mass(p)
    if m: st[p[0]] = st.get(p[0], 0) + m
st['tool'] = st['tool'] * 0.4          # jaws removed, flange disc only (approx)
RS04, RS02, RS00, GRIP = F.RS04['kg'], F.RS02['kg'], F.RS00['kg'], 0.485
xg = FLANGE + GRIP_L / 2               # gripper COM from the shoulder axis (mm)
R_TCP = 480 + T                        # 786
MOV = [('upper', st['upper'], 130), ('J2 RS04 (on axis)', RS04, 0), ('J4 RS04', RS04, 260), ('fore', st['fore'], 330),
       ('J5 RS02', RS02, 400), ('yoke', st['yoke'], 445), ('J6 RS02', RS02, 480), ('wrist', st['wrist'], 500),
       ('J7 RS00', RS00, 560), ('tool flange', st['tool'], 607), ('gripper v2', GRIP, xg)]
FIX = [('base', st['base']), ('J1 RS04', RS04), ('turret', st['turret'])]
harness = 0.25
m_mov = sum(m for _, m, _ in MOV); m_fix = sum(m for _, m in FIX) + harness; arm = m_mov + m_fix
xc = sum(m * x for _, m, x in MOV) / m_mov
PAY = 2.5
plate = plinth_l * plinth_w * plinth_t * 1e-9 * 2700; blocks = 4 * 0.20; chain_share = 0.5
W0 = (plate + blocks + chain_share + 2 * arm) * g        # no payload
M_self = m_mov * g * xc / 1e3; M_pay = PAY * g * R_TCP / 1e3
def blk(Fz, Mx, My, gap):
    hx, hy = block_pitch_x / 2e3, gap / 2e3
    return {(sx, sy): Fz / 4 - sy * Mx / (4 * hy) + sx * My / (4 * hx) for sx in (-1, 1) for sy in (-1, 1)}
print('arm (rose v9 + gripper v2): structure %.2f kg, actuators %.2f, gripper %.3f, harness %.2f -> %.2f kg; moving %.2f kg, COM %.0f mm from shoulder; TCP %d mm'
      % (sum(st.values()), 3 * RS04 + 2 * RS02 + RS00, GRIP, harness, arm, m_mov, xc, R_TCP))
print('plinth: plate %.2f kg + 4 blocks %.1f + chain share %.1f + 2 arms %.2f = %.1f kg -> %.0f N (%.0f N/block static)' % (plate, blocks, chain_share, 2 * arm, W0 / g, W0, W0 / 4))
print('one arm straight out horizontal + %.1f kg at %d mm: self %.1f + payload %.1f = %.1f N.m about its shoulder' % (PAY, R_TCP, M_self, M_pay, M_self + M_pay))
Wa = W0 + PAY * g
# A: R arm (X +200) straight out to -Y over the front bay, 2.5 kg; L tucked over the plinth
MxA = M_self + M_pay; MyA = PAY * g * arm_pitch / 2e3 - (m_mov * g * 0) / 1e3
# B: as A plus RS04 J2 peak 120 N.m reacting (sudden stop / impact)
MxB = MxA + 120
# C: both arms straight out to -Y, 2.5 kg each
Wc = W0 + 2 * PAY * g; MxC = 2 * (M_self + M_pay)
# E: R arm straight out along +X (spine) past the plinth end, 2.5 kg
MyE = (m_mov * g * xc + PAY * g * R_TCP) / 1e3 + (arm + PAY) * g * arm_pitch / 2e3 - arm * g * arm_pitch / 2e3
cases = [('static, both tucked, no payload', W0, 0, 0), ('A: R arm straight out to -Y (front bay), 2.5 kg', Wa, MxA, MyA),
         ('B: A + RS04 peak 120 N.m reacting', Wa, MxB, MyA), ('C: both arms straight out to -Y, 2.5 kg each', Wc, MxC, 0),
         ('E: R arm straight out along +X, 2.5 kg', Wa, 0, MyE)]
for gap in (rail_gap, 300):
    print('\nrail gap %d mm (blocks at X -/+%d, Y -/+%d); order (-X-Y) (-X+Y) (+X-Y) (+X+Y):' % (gap, block_pitch_x / 2, gap / 2))
    for n, Fz, Mx, My in cases:
        b = blk(Fz, Mx, My, gap); v = sorted(b.values())
        print('  %-50s Mx %6.1f My %5.1f N.m -> %s N; max comp %4.0f, max lift-off %4.0f' % (n, Mx, My, ' '.join('%+5.0f' % b[k] for k in sorted(b)), v[-1], -min(0, v[0])))
print('''
ASSUMED SBR16UU ratings (typical catalogue, verify): C ~1.2 kN dynamic, C0 ~1.4 kN static, open-block pull-off ~0.7 kN (50 % C0).
Tip-over of the plinth itself is not possible (bolted to blocks); lift-off = tension on the open block's retention.''')
