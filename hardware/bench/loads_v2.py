"""Block loads for the cell_v2 plinth: 4 SBR blocks at (+-block_pitch_x/2, +-rail_gap/2) around the plinth centre.
Masses: rose_fork.py v6 prim_mass per link (structure) + vendor actuator masses, COM per link estimated along the link.
Sign: +F = compression (block pushed onto the rail), -F = lift-off (pull-off on an open SBR block)."""
import sys
sys.path.insert(0, __import__('os').path.join(__import__('os').environ.get('ROSE_DIR', __import__('os').path.expanduser('~/Developer/rose')), 'arm', 'fork'))
import rose_fork as F
from cellp import *
g = 9.81
st = {}
for p in F.build():
    m = F.prim_mass(p)
    if m: st[p[0]] = st.get(p[0], 0) + m
RS04, RS00, EDU = F.RS04['kg'], F.RS00['kg'], F.EDU['kg']
# moving mass about the shoulder axis, arm straight out: (kg, x mm from the shoulder)
MOV = [('upper struct', st['upper'], 130), ('J2 RS04 (on the upper arm, on the axis)', RS04, 0), ('J4 RS04', RS04, 260),
       ('fore struct', st['fore'], 330), ('J5 RS00', RS00, 377), ('yoke struct', st['yoke'], 445), ('J6 RS00', RS00, 480),
       ('wrist struct', st['wrist'], 500), ('J7 RS00', RS00, 553), ('tool struct', st['tool'], 600), ('gripper', EDU, 620)]
FIX = [('base struct', st['base']), ('J1 RS04', RS04), ('turret struct', st['turret'])]
harness = 0.25
m_mov = sum(m for _, m, _ in MOV); m_fix = sum(m for _, m in FIX) + harness; arm = m_mov + m_fix
xc_mov = sum(m * x for _, m, x in MOV) / m_mov
PAY, R_TCP = 2.5, 702
plate = plinth_l * plinth_w * plinth_t * 1e-9 * 2700
blocks = 4 * 0.20                   # SBR16UU ~0.2 kg each (assumed)
chain_share = 0.5
W = (plate + blocks + chain_share + 2 * arm + 2 * PAY) * g
hx, hy = block_pitch_x / 2 / 1e3, rail_gap / 2 / 1e3
def blocks_for(Fz, Mx, My):
    """Fz total down (N), Mx about +X (N.m, +ve loads the -Y rail... sign handled by abs), My about +Y.
    returns dict of 4 block loads (compression +)."""
    out = {}
    for sx in (-1, 1):
        for sy in (-1, 1):
            # a moment Mx (from a load at -Y) presses the -Y rail: F = Fz/4 - sy*Mx/(4*hy) ; My (load at +X) presses +X: + sx*My/(4*hx)
            out[(sx, sy)] = Fz / 4 - sy * Mx / (4 * hy) + sx * My / (4 * hx)
    return out
print('arm (rose v6): structure %.2f kg (%s), actuators %.2f, harness %.2f -> %.2f kg (README 8.9); moving %.2f kg, COM %.0f mm from shoulder'
      % (sum(st.values()), ', '.join('%s %.2f' % kv for kv in st.items()), 3 * RS04 + 3 * RS00 + EDU, harness, arm, m_mov, xc_mov))
print('plinth: plate %.0fx%.0fx%.0f Al %.2f kg + 4 blocks %.1f + chain share %.1f + 2 arms %.1f + 2 x %.1f kg payload = %.1f kg -> %.0f N, %.0f N per block static'
      % (plinth_l, plinth_w, plinth_t, plate, blocks, chain_share, 2 * arm, PAY, W / g, W, W / 4))
M_self = m_mov * g * xc_mov / 1e3; M_pay = PAY * g * R_TCP / 1e3
print('one arm straight out, horizontal, %.1f kg at %d mm: self %.1f + payload %.1f = %.1f N.m about its shoulder' % (PAY, R_TCP, M_self, M_pay, M_self + M_pay))
print('  (rose README: v6 HOLDS 1.28 kg straight out continuously -- wrist binds; 2.5 kg straight out is a peak/short-duty case)')
# case A: right arm (+200) straight out toward -Y (front lane), other arm tucked over the plinth, only the reaching arm carries payload
Wa = W - PAY * g                      # tucked arm has no payload
MxA = M_self + M_pay                  # load at -Y: presses the -Y rail (front)
MyA = (arm + PAY) * g * arm_pitch / 2e3 - arm * g * arm_pitch / 2e3     # weights at +-200 cancel except the payload
A = blocks_for(Wa, MxA, MyA)
# case B: right arm straight out along +X (beyond the plinth end), payload at its flange; other arm tucked
MyB = (m_mov * g * (arm_pitch / 2 + xc_mov) + m_fix * g * arm_pitch / 2 + PAY * g * (arm_pitch / 2 + R_TCP) - arm * g * arm_pitch / 2) / 1e3
B = blocks_for(Wa, 0.0, MyB)
# case C: both arms straight out toward -Y with payload (worst roll)
MxC = 2 * (M_self + M_pay); C = blocks_for(W, MxC, 0.0)
# case D: dynamic -- J2 RS04 peak 120 N.m reacting into the plinth from one arm pitching toward -Y, on top of case A
MxD = MxA + 120   # conservative: the full RS04 peak torque added to the static roll moment
D = blocks_for(Wa, MxD, MyA)
def show(name, Mx, My, b):
    v = sorted(b.values())
    print('%-62s Mx %6.1f  My %6.1f N.m -> blocks %s N; max compression %4.0f, max lift-off %4.0f' % (
        name, Mx, My, ' '.join('%+5.0f' % b[k] for k in sorted(b)), max(v), -min(0, min(v))))
print('\nblock loads (blocks at X -/+%.0f, Y -/+%.0f mm about the plinth centre; order (-X-Y) (-X+Y) (+X-Y) (+X+Y)):' % (hx * 1e3, hy * 1e3))
show('static, both arms tucked + payloads', 0, 0, blocks_for(W, 0, 0))
show('A: R arm straight out to -Y (front), 2.5 kg', MxA, MyA, A)
show('B: R arm straight out along +X past the plinth end, 2.5 kg', 0, MyB, B)
show('C: both arms straight out to -Y, 2.5 kg each', MxC, 0, C)
show('D: A + RS04 peak torque 120 N.m reacting (dynamic)', MxD, MyA, D)
# carriage acceleration
a = 2.0; h_com = 0.30   # COM ~300 mm above the rod centre with arms up
print('\ncarriage accel %.0f m/s^2: %.0f N axial drive force; COM ~%.0f mm above the rods -> %.1f N.m pitch, +/-%.0f N on the blocks'
      % (a, W / g * a, h_com * 1e3, W / g * a * h_com, W / g * a * h_com / (4 * hx)))
print('''
ASSUMED ratings (typical catalogue values for open SBR blocks on 16/20 mm rod -- not from a Vevor datasheet, verify before buying):
  SBR16UU  dynamic C ~ 1.2 kN, static C0 ~ 1.4 kN; lift-off (pull-off) on an open block taken as ~50 % of C0 ~ 0.7 kN
  SBR20UU  dynamic C ~ 1.6 kN, static C0 ~ 2.4 kN; lift-off ~ 1.2 kN''')
worst_c = max(max(b.values()) for b in (A, B, C, D)); worst_t = -min(min(b.values()) for b in (A, B, C, D))
for n, C_, C0, T in (('SBR16UU', 1200, 1400, 700), ('SBR20UU', 1600, 2400, 1200)):
    print('  %s: worst compression %.0f N -> C0/F = %.1f (static), C/F = %.1f; worst lift-off %.0f N -> %.1f x margin' % (n, worst_c, C0 / worst_c, C_ / worst_c, worst_t, T / worst_t if worst_t else float('inf')))
