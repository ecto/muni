# gripper v2 — EL05 rack-and-pinion parallel gripper (rose fork-v9 RS00 tool roll)

End effector for the fork-v9 tool roll (J7 = RS00). All structure is 1/8" (3.175 mm) sheet: 5052 plates, 6061 racks and
fingers. Two 70×90 side plates + top + palm, a RobStride EL05 bolted to plate_r with its output to the centre, a Ø16
pinion (r 8, module 1, 16 T) between two flat 68 mm racks (50 mm travel/side + engagement). Each rack carries one
finger (20×80) with a tip socket (2× Ø4 dowels + Ø8 magnet). Four swappable tip sets (flat serrated alu, V, 12 mm TPU,
scoop). Palm UVC camera `cam` (12×12×8). Optional TPU `boot` for the rover. Z-up, mm; fingers -Z, opening along X.

## v2 changes (rose arm team)

- **Flange**: the ISO 9409 pattern is gone from the main part. `plate_top` = 70×56 rectangle + Ø61 round outline
  matching the v9 tool-flange disc, 6× Ø3.4 on Ø27 PCD, first hole at 0° then every 60° (= `rose_fork.py`
  `disc('tool', ..., J7M['d']/2 + 0.002, T, pattern(0, 0, RS00 out=(6, 0.027, 0.0034), 0.0))`). Through-bolted with
  6× M3×10 SHCS (`screws`, heads under the plate) through plate + `flange_disc` (the arm's part, shown for fit) into
  the RS00 output. ISO 9409-1-31.5-4-M5 survives only as optional `adapter_iso` (`--adapter`, Ø61×6, 4× PEM M5 nut
  bosses on PCD 31.5, Ø6 dowel; +52 g), shown exploded 40 mm up in gripper_adapter.png only.
- Sheet 3 → 3.175 mm everywhere (plates, racks, fingers).
- Pinion r 10 → 8 (Ø16); racks 72 → 68.
- Racks and tips steel → 6061; lightening windows: plate_l 40×40 (z 35–75), plate_r 50×20 under the EL05 (z 6–26).
- Camera: UVC USB 2.0 module 12×12×8 at the palm. **Wiring**: its USB D+/D− pair runs in the arm harness straight to
  the base computer, bypassing the controller — twisted, shielded, kept short (USB 2.0 HS is fine to ~1.5–2 m of
  decent cable through slip-free cable runs; no clock-spring contacts in the pair). 5 V comes from the arm's aux port.

## Numbers (numbers.py → numbers.txt)

EL05 (manual): rated 1.8 N·m continuous (70×70 heat sink), peak 6 N·m, 242 g. Squeeze per finger = T/(2r), r = 8 mm:

| | torque | squeeze / finger | max lift (μ 0.5, 2 faces, 2× SF) |
|---|---|---|---|
| rated | 1.8 N·m | 112.5 N | 5.7 kg |
| peak | 6 N·m | 375 N | 19.1 kg (don't hold at peak) |

| object | mass | F needed | EL05 torque |
|---|---|---|---|
| Ø66 alu can (empty) | 15 g | 0.3 N | 0.005 N·m (0.3 %) |
| 500 ml bottle | 0.55 kg | 10.8 N | 0.17 N·m (9.6 %) |
| 1/8" 5052 plate 300×200 | 0.51 kg | 10.0 N | 0.16 N·m (8.9 %) |

Mass at the flange (flat tips): side plates 108, top 33, palm 22, fingers 27, alu racks 10, steel pinion+shaft 15,
alu tips 17, EL05 242, M3 screws 4, cam/magnets/dowels/fasteners 29 = **507 g**; windows remove 22 g → **485 g** (≤ 500).
Flange disc is the arm's; ISO adapter +52 g if fitted. No rack friction losses (×0.8 real).

## Rebuild

    ./build.sh            # everything (~100 s: stills + 192-frame video in parallel)
    NOVIDEO=1 ./build.sh  # stills + numbers only (~10 s)

`gen_gripper.py` → `*.loon` → `loon2vcad` → `*_raw.vcad` (joint states 0 = closed) → `pose.py IN OUT opening [lift]`
→ `vcad-render`. Params on the generator: `--open`, `--t` (plate, 3.175), `--r` (pinion radius, 8), `--flen` (finger length),
plus `--tips flat|v|soft|scoop`, `--can`, `--boot`, `--display`, `--adapter`. Each loon evaluates in ~10 ms.

Files: gripper_iso_open.png, gripper_iso_closed.png (closed on a Ø66 can), gripper_tips.png, gripper_section.png
(section y=0 through pinion + racks), gripper_front.png (--dims, 100 mm open), gripper_rover.png
(soft tips + boot), gripper_flange.png (top view, --dims, Ø27 6-hole pattern), gripper_adapter.png (ISO adapter, exploded), gripper_grasp.mp4 (1280×720, 24 fps, 8 s: open → close on can → lift 60 → lower → open, orbit),
gripper_grasp.gif (4 s, 0.4 MB), numbers.txt.

## Gotchas / what didn't fit

- **The brief's 60 mm rack is too short**: 50 mm travel per side + pinion engagement needs ≥ travel + 2r → racks are 68 (r 8).
  At 100 mm open the racks overhang the 70 mm body by ~26 mm per side (hence the boot).
- Rated squeeze (90 N) will crush an empty can — run the EL05 in current/torque mode, ~1–5 N for light objects. That
  is 0.3 % of rated torque for the can: torque resolution at the bottom end matters more than headroom.
- The STEP import (`[import-step ".../el05.stp"]`) loads but comes in as a few open bands (46×46×12.5), not the body;
  the EL05 is modelled as a Ø46×44 cylinder + Ø24×3 boss + connector block from the manual's outline drawing.
- Palm camera is offset to y=-18 on a palm strip, not dead centre: the fingers pass through a ±11 mm palm slot and
  would crush a centred camera at full close.
- Soft TPU pads are 12 mm vs the 6 mm reference tip, so opening with soft tips is 88 mm max (contact plane shifts 6/side).
- The V tip's groove is shallow (4 mm deep, ~136°) — a deep 90° V for Ø90 cylinders needs a thicker tip block.
- Kinematic coupling is faked: finger_l/finger_r prismatic (0..50) and `pinion` revolute are set together by
  `pose.py` (pinion = −q/r rad; rack_l rides on top of the pinion). `lift`/`can_lift` move gripper + can together.
- vcad-render auto-frames each video frame on the doc bbox, so the camera zooms slightly during the lift.
- The renderer's style maps low-saturation materials to its blue line-art palette; only the TPU (orange) and cam keep
  distinct colours — the "black EL05 / dark steel" materials are in the doc but render blue.
- Labelled stills drop dowel_*/magnet_* instances (`STRIP=1 pose.py`) so the labels stay readable; they are in the docs.
- `--dims` only dimensions the bbox (W 121.3 × D 102.7 in gripper_flange.png); the Ø27/Ø3.4/Ø61 values are
  in the generator, not drawn as callouts — vcad-render has no hole/PCD dimensioning.
- M3×10 leaves ~3.6 mm of thread into the RS00 output past plate + disc (2× 3.175); check the RS00's tapped depth.
- Pinion stays steel (16 T module-1 on alu racks: wear item; alu racks are the cheap replaceable side).
- Windows are true cut-outs; plate_r's sits below the EL05 so it doesn't touch the motor bolt circle.

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
