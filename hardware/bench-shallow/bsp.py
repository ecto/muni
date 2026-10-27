"""Shared bench_shallow parameters (mm, Z-up). Box = the real 4040 frame: X 0..box_w, Y 0..box_d, Z 0..box_h.
Table top z=0 under the box, extending 1000 in front (Y -1000..0). Every other script imports this."""
box_w, box_d, box_h = 1500, 500, 1000
ext = 40                     # 4040
deck_z, deck_t = 200, 20     # deck top / thickness (plate z 180..200)
# rails: brief asked Y=180/400; 400 puts the back rail+plinth (Y..520) through the back wall racks and the
# back frame (Y 460..500). Moved forward to 150/370 (still 220 apart): plinth Y 100..420, racks Y 380..460 above z 280.
rail_y = (150, 370)
rail_gap = rail_y[1] - rail_y[0]          # 220
plinth_y = sum(rail_y) / 2                # 260
rail_l = 1300
rail_x0 = (box_w - rail_l) / 2            # 100
plinth_l, plinth_w, plinth_t = 520, 320, 10
plinth_h = 55                             # deck top -> plinth top (SBR16 support+block 45 + plate 10)
block_pitch_x, block_l, block_w = 360, 45, 45
arm_pitch = 400
foot_r = 115                              # Ø230 foot
Z_SH = 200                                # rose_fork.Z_SH (foot underside -> shoulder axis)
sh_z = deck_z + plinth_h + Z_SH           # 455
# carriage: plinth centre X = car_x0 + q. Feet (Ø230 at 400 pitch -> 630 span) stay between the posts (X 40..1460)
car_x0 = ext + foot_r + arm_pitch // 2    # 355
travel = box_w - 2 * car_x0               # 790
# arm (rose fork v9) planar chain lengths; T = wrist centre -> gripper v2 fingertip
L1, L2 = 260.0, 220.0
FLANGE = 610.6                            # rose x of the tool-flange face (disc 607.4 + 3.175)
GRIP_L, FING_L, FING_OVL = 135, 80, 40    # gripper box along the tool axis, finger length, finger inside the box
T = FLANGE + GRIP_L + FING_L - FING_OVL - 480.0   # 305.6
table_h = 750                             # assumed table height (floor z = -750) -- only used for the human
