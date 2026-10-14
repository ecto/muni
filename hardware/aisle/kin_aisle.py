"""Tender FK/IK: kin_v2's planar rose-v6 chain with the shoulder moved by tender_N_x and tender_N_lift.
shoulder(x, lift, side) = (T_X0 + x -/+ 200, SPINE_Y, SH0 + lift). J1 = 0 points +X; +90 -> back lane (+Y), -90 -> front lane (-Y)."""
import math, sys
sys.path.insert(0, __import__('os').path.join(__import__('os').path.dirname(__import__('os').path.abspath(__file__)), '..', 'bench'))
import kin_v2 as K
from aislep import *
def shoulder(x, lift, side): return (T_X0 + x + (-ARM_PITCH / 2 if side == 'l' else ARM_PITCH / 2), SPINE_Y, SH0 + lift)
# tender arms are mounted rose-native (zero pose +X, along the spine); kin_v2 assumes zero pose +Y. Work in a frame
# turned +90 deg about the shoulder: world offset (dx,dy) -> kin_v2 (-dy,dx); back: (a,b) -> (b,-a).
def _with(sh, f, *a, **k):
    old = K.shoulder; K.shoulder = lambda c, s: sh
    try: return f(0, 'l', *a, **k)
    finally: K.shoulder = old
def _to(sh, p): return (sh[0] - (p[1] - sh[1]), sh[1] + (p[0] - sh[0]), p[2])
def _back(sh, p): return (sh[0] + (p[1] - sh[1]), sh[1] - (p[0] - sh[0]), p[2])
def fk(x, lift, side, q):
    sh = shoulder(x, lift, side); return [_back(sh, p) for p in _with(sh, K.fk, q)]
def ik(x, lift, side, p, tp=-90.0, all_sols=False):
    sh = shoulder(x, lift, side); return _with(sh, K.ik, _to(sh, p), tp, all_sols=all_sols)
margin = K.margin
def x_for(px, side): return px - T_X0 - (-ARM_PITCH / 2 if side == 'l' else ARM_PITCH / 2)   # tender_x putting that shoulder at X=px
