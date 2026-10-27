"""Aisle parameters (mm, Z-up). Spine along X, walkway Y 1000..1700, front lane Y 0..1000, back lane Y 1700..2700."""
AISLE_L, AISLE_D = 6000, 2700
FLOOR_T = 20                 # floor plate z 0..20
BAY_W, BAY_D, BAY_T = 500, 1000, 6
BZ = FLOOR_T + BAY_T         # 26: machine datum (top of bay floor plates)
SPINE_Y = 1350
RAIL_Y = (1200, 1500)
FRONT_EDGE, BACK_EDGE = 1000, 1700
# tender: plinth (cell_v2) on the spine rails, fixed outer column, lift stage (top plate + inner mast)
PL_H = 55                    # rail base -> plinth top
COL_H = 320                  # fixed outer column above the plinth plate
PLATE_T = 12
LIFT_MAX = 600
Z_SH = 200
ARM_PITCH = 400
SH0 = FLOOR_T + PL_H + COL_H + PLATE_T + Z_SH     # 607: shoulder z at lift 0
T_X0 = 300                   # plinth centre X at tender_N_x = 0
T_TRAVEL = 5400
# bench (cell_v2) placed rotated 180 deg about Z: bench-local (x,y,z) -> aisle (2500-x, 1000-y, z+BZ)
BENCH_X1 = 2500
def bench2aisle(p): return (BENCH_X1 - p[0], 1000 - p[1], p[2] + BZ)
