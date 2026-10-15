"""Carriage mass and rail moments for cell_real. Masses: rose_fork.py prim_mass (structure) + vendor actuator masses."""
g = 9.81
# moving arm (outboard of the shoulder), (kg, x mm from shoulder axis, arm straight out)
MOV = [('upper struct', .812, 130), ('J4 RS04', 1.42, 260), ('fore struct', .431, 333), ('J5 RS00', .31, 377),
       ('yoke struct', .276, 445), ('J6 RS00', .31, 480), ('wrist struct', .256, 500), ('J7 RS00', .31, 560),
       ('tool struct', .150, 600), ('gripper EduLite', .242, 620)]
FIXED = [('base struct', .754), ('J1 RS04', 1.42), ('turret struct', .579), ('J2 RS04', 1.42)]
harness = 0.25
m_mov = sum(m for _, m, _ in MOV); m_fix = sum(m for _, m in FIXED)
arm = m_mov + m_fix + harness
M_arm = sum(m * x for _, m, x in MOV) / 1e3 * g          # N.m about the shoulder, arm horizontal
PAY, R_TCP = 2.5, 0.702
M_pay = PAY * R_TCP * g
plate = 0.520 * 0.160 * 0.010 * 2700                     # 10 mm 6061 carriage plate
block, chain = 1.0, 1.0                                  # rail carriage block(s) + cable-chain share (estimates)
car = 2 * arm + plate + block + chain
print('arm: moving %.2f kg, base+turret %.2f, harness %.2f -> %.2f kg each (rose fork v5 README: 8.9)' % (m_mov, m_fix, harness, arm))
print('carriage: plate %.2f + block %.1f + chain %.1f + 2 arms %.2f = %.1f kg; with 2 x 2.5 kg payload %.1f kg -> %.0f N hanging (lift-off on an inverted rail)' % (plate, block, chain, 2 * arm, car, car + 5, (car + 5) * g))
print('one arm straight out, horizontal, 2.5 kg at the 702 mm TCP: arm self %.1f N.m + payload %.1f N.m = %.1f N.m about its shoulder' % (M_arm, M_pay, M_arm + M_pay))
Mx = M_arm + M_pay
print('  arm out along Y (across the rail): roll moment Mx = %.1f N.m' % Mx)
# arm out along +X from the +200 shoulder; the other arm tucked over its own shoulder (-200)
My = Mx + (arm + PAY) * g * 0.2 - arm * g * 0.2
print('  arm out along X (along the rail), other arm tucked: pitch moment My = %.1f N.m about the carriage centre' % My)
print('  same with both arms out the same way + both payloads: My = %.1f N.m' % (2 * Mx + 0 ))
# dynamic: peak motor reaction
print('dynamic: J1/J2 RS04 peak 120 N.m reacts straight into the carriage -> design Mx/My/Mz for ~120 N.m peak per arm')
a = 2.0
print('carriage accel %.0f m/s^2 with COM ~0.15 m below the rail block: %.0f N axial, %.1f N.m pitch' % (a, (car + 5) * a, (car + 5) * a * 0.15))
