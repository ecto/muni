# bench_shallow — the real 1500×500×1000 4040 box as the spine, three bays docked in front

Everything here is generated; rebuild all of it (loon → vcad → stills → tables → video/gif) with:

```
./build.sh            # STILLS_ONLY=1 ./build.sh skips the video; NP=12 sets render parallelism
```

Files: `bsp.py` (all parameters), `gen_bench_shallow.py` → `bench_shallow.loon` → `bench_shallow_raw.vcad`
(`_np` = panels off), `kin.py` (FK/IK), `reach.py` → `reach.txt`, `loads.py` → `loads.txt`, `frames.py`
(poses, keyframes, frames; `frames.py table` → `joints.txt`), `label_doc.py` + `render_labeled.sh` (readable labels),
`subset.py` (view docs). Loon eval ~1 s.

## Assumptions

- Box sits on a work table; the table top is z = 0 at the box's bottom and extends 1000 mm in front of the box
  (Y −1000..0) at the same level. Table height 750 is assumed only to stand the human on the floor (feet z = −750).
- Z-up, X along the box (the spine), Y = depth, box front face (Y = 0) open, back at Y = 500.
- Arm = rose fork v9 read live from `rose_fork.py` (`build()`, `LINKS`, `ORIGIN`), mounted in its native frame
  (zero pose along +X), one vcad part per primitive welded with fixed joints. Rose's own EDU jaws are dropped and
  replaced by gripper v2 as a 70×56×135 body on the tool flange + two 20×80 fingers (40 mm inside the body), 485 g.
  TCP = fingertip = flange 610.6 + 135 + 80 − 40 → **305.6 mm from the wrist centre** (785.6 from the shoulder axis).
  Note: that is longer than the "~222 wrist→TCP" in the brief (rose's jaws end 731 from the shoulder); the gripper
  v2 body length dominates. All reach numbers use 305.6.
- Bay standard: 500 wide × 1000 deep, floor plate, dock beam with face at Y = 0, 2× Ø12 datum pins at z = 200,
  400 apart, pointing +Y into the box's deck edge, 120×60×60 coupler centred.

## Stack-up (shoulder height)

| item | z |
|---|---|
| table / box bottom | 0 |
| deck plate (20 mm, on two 4040 carriers) | 180..200 |
| SBR16 support 17 + block + 10 mm plinth plate | 200..255 |
| rose foot underside → shoulder axis (`Z_SH`) | +200 |
| **shoulder axes** | **455** (Y = 260, X = plinth centre ∓ 200) |
| electronics bay under the deck: psu 200×100×50, canhub 100×60×30, controller 150×100×40 | 40..90 |

## Rails — moved from Y 180/400 to Y 150/370 (still 220 apart)

The brief's Y = 400 back rail puts the plinth (320 wide → Y 130..450 + 45 mm blocks) into the back 4040 (Y 460..500)
and through the back-wall racks. At Y 150/370 the plinth is Y 100..420, blocks Y 127.5..392.5, and the racks sit at
Y 370..460 above z 280 (plinth top 255, so the arm bases pass underneath the rack bottoms). Busbar 20×12 runs
between the rails at Y 260 under the plinth; cable chain outside the front rail at Y 95..125.

## Carriage travel that fits

Feet Ø230 at 400 pitch → 630 mm span; posts leave X 40..1460 = 1420 → **travel 790 mm**
(`carriage_x` 0..790, plinth centre X = 355 + q). Rails 1300 long at X 100..1400: block extents 152.5..1347.5 over the
full stroke, 52 mm spare each end.

## Reach (reach.txt)

TCP must hit the point inside rose limits with ≥ 5° margin, tool pitch in the point's band (doors/racks: 0…−30°,
tops: −90…−45°). Columns: who reaches at q = 0 / 395 / 790, and the carriage ranges on a virtual rail −400..1300.

| point | (X,Y,Z) | q=0 | q=395 | q=790 | L q | R q | both q |
|---|---|---|---|---|---|---|---|
| mill door centre (60 in) | (250,−110,390) | L+R | L | – | ≤780 | ≤380 | ≤380 |
| bench free-area centre | (750,−190,215) | R | L+R | L | 65..1125 | ≤725 | 65..725 |
| bench near-L corner | (540,−60,215) | L+R | L+R | L | ≤1000 | ≤600 | ≤600 |
| bench near-R corner | (960,−60,215) | R | L+R | L+R | 190.. | ..1020 | 190..1020 |
| **bench far corners** | (540/960,−620,215) | – | – | – | **unreachable** | | |
| bench at the oven front edge | (750,−330,215) | R | L+R | L | 230..960 | ≤560 | 230..560 |
| oven door centre | (750,−340,300) | R | L+R | L | 115..1075 | ≤675 | 115..675 |
| winder top | (1360,−160,365) | – | R | L+R | 675.. | 275.. | 675.. |
| stock drawer top | (1130,−210,335) | – | R | L+R | 475.. | 75..1075 | 475..1075 |
| **finished drawer top** | (1130,−520,335) | – | – | – | **unreachable** | | |
| tip peg 1 / 4 (z 560 / 740) | (110/290,345,…) | L / L+R | – / L+R | – / L | ≤320 / ≤450 | ≤280 / 220..420 | |
| tip rack z 500 / 800 | (200,345,…) | L+R / L | L / – | – | ≤420 / ≤360 | ≤365 / ≤275 | |
| parts shelf 1 / 4 mid | (750,360,340/790) | L+R | L+R | L+R | (gappy) | | |
| parts shelf 1 left end / 4 right end | | L / – | – / R | – / L+R | ≤315 / 605.. | ≤200 / 205..1215 | |
| tool strip | (1320,345,720) | – | R | L+R | 465.. | 65..1080 | 465..1080 |

- One arm, everything reachable: in theory a single carriage position (q ≈ 275) touches every reachable point with
  one arm or the other, but it sits on the edge of the winder and peg-1 ranges; in practice use q 0..790 freely.
- Both arms at every reachable point: **minimum stroke 490 mm** (q 185..675). The 790 available covers it.
- Front reach limit (shoulder Y 260, z 455): top-down at z 215 → **Y ≈ −435**; tilted to horizontal → −485; door
  height z 390 → −520. Only the first ~430 mm of each 1000-deep bay is workable.
- Rack points lie only 85–100 mm behind the shoulder in Y; IK reaches them by folding back over the shoulder. Limits
  are respected but self-collision is NOT checked (no collision model) — the rack still needs a real check.

**Flags + fixes**
1. Finished drawer (Y −520) unreachable → stack it under the stock drawer as a two-high drawer unit
   (stock top 440, both within Y −360..−60), or rotate both drawers 90° side by side at Y −240..−40 (200 deep).
2. Bench far corners / anything beyond Y ≈ −430 unreachable → treat bays as ~450 deep work zones: put the oven at the
   near half or turn it so its door faces ±X near the dock edge; bays can stay 1000 deep for storage/clearance.
3. Moving the rails forward is the only change that buys depth: Y 60/280 (plinth Y 10..330) gains 90 mm (front reach
   ≈ −525) but loses rack clearance at the back, and the front 4040 (Y 0..40) then limits the front block. Raising the
   shoulder does not help the bench (it only lengthens the vertical drop).

## Loads (loads.txt) — four SBR16 blocks, X ±180, Y ±110

Arm 8.44 kg (rose v9 structure 2.37, RS04×3 + RS02×2 + RS00 5.33, gripper 0.485, harness 0.25); moving part 5.55 kg,
COM 278 mm from the shoulder. Plinth + 2 arms + blocks + chain share = 22.7 kg → 56 N/block static.
One arm straight out horizontal + 2.5 kg at 786 mm: 34.4 N·m about its shoulder.

| case | rail gap 220 (this box): max comp / max lift-off | rail gap 300 (cell_v2) |
|---|---|---|
| static, both tucked | 56 / 0 N | 56 / 0 N |
| A: R arm straight out over the front bay, 2.5 kg | **147 / 23 N** | 126 / 2 N |
| B: A + RS04 peak 120 N·m reacting (stop/impact) | **419 / 296 N** | 326 / 202 N |
| C: both arms straight out front, 2.5 kg each | 224 / 89 N | 183 / 47 N |
| E: arm out along +X, 2.5 kg | 116 / 0 N | 116 / 0 N |

Narrower spacing multiplies the roll term by 300/220 = 1.36, so the worst case goes 326 → 419 N compression and
202 → 296 N lift-off, and case A now *lifts* the back blocks statically (23 N). Against assumed SBR16UU ratings
(C0 ≈ 1.4 kN, open-block pull-off ≈ 0.7 kN, verify) that is still 3.3× / 2.4× margin, so 220 is acceptable — but the
blocks must be bolted (never a gravity-seated plinth), and the dynamic case B is the one to respect: limit J2 accel when
reaching over the front.

## Joints

`carriage_x` (prismatic X, 0..790) and per arm `arm_{l,r}_J1_yaw` (±170), `J2_sho_pitch` (−150..120),
`J4_elbow` (−170..150), `J5_fore_roll` (±150), `J6_wrist_pitch` (±150), `J7_tool_roll` (±180). Rose signs: +pitch
swings the distal link down. All other parts are fixed-jointed to `frame` (or `plinth`).

## Keyframes (frames.py, joints.txt)

TUCK (J1 L −30 / R 150, J2 −20, J4 145, J6 140: folded over the plinth, margin 5°) → carriage 0 → MILL_PRE (250,−40,470)
→ MILL (250,−110,390, tool −40°) hold → back → carriage 395 → BENCH (both, (690/810,−200,215), tool −60°) hold →
HANDOFF (725/775,−150,430) hold → carriage 200 → RACK_PRE → RACK (peg 4, (290,345,740), tool −20°) hold → RACK_OFF
(drop 30, back 60) → carriage 790 → WIND_PRE → WIND (1360,−160,365, tool −80°) hold → TUCK → carriage 395 (home).
14.6 s at 24 fps, camera az 250 el 40 rendered 1600×900 and cropped to 1280×720 (human/table/bay floor plates left out
of the video so the bays don't hide the arms). `bench_shallow_working.gif` = the same squeezed to 6 s, 640 wide, 12 fps.
Hold frames checked: the mill reach is partly hidden behind the mill body from any front camera (its door faces the
box) — `bench_shallow_mill.png` shows it in section.

## Stills

`bench_shallow_iso.png` (az 215 el 25), `_top.png` (--dims), `_front.png`, `_mill.png` (section x = 300 through the
mill, arm L in the door), `_work.png` (both arms on the bench), `_rack.png` (L arm at peg 4), `_enclosed.png`
(panels + back panel on), `_bay.png` (bay_2 dock from inside the box: pins, coupler, dock beam, plate).

## Gotchas

- vcad-render `--labels` labels every instance (≈90 arm parts) → labels come from `label_doc.py` (parts shrunk to tags,
  two unlabelled bbox markers keep the framing identical) composited onto the full render. Section renders can't use it.
- `--section` bypasses the mesh cache (needs BRep) and is slow.
- Rendering cost is all in the arms' bolt/lightening holes: every `[difference cut body]` costs ~90 s CPU per arm per
  render even with a warm mesh cache; without them a whole frame renders in well under a second. The video therefore
  uses hole-free arms (`frames.strip_holes`, body = the RIGHT operand: loon `[difference A B]` = B − A); the stills keep
  the holes. Frames are also deduped (holds render once): 351 frames, 259 unique, ~20 s on 10 cores.
- FK ignores instance xyz for jointed children; pose only via joint `state` in the JSON.
- loon `union` takes exactly two args (`U()` folds); label bbox must rotate `Rotate` nodes (rails are rotated cylinders).
- Zero pose is NOT a safe home here (arms straight along ±X, 786 mm, hit the posts at the travel ends): use TUCK.

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `ROSE_DIR` — checkout of the rose repo (default `~/Developer/rose`); arm geometry is read from `$ROSE_DIR/arm/fork/` (`rose_fork.py`, `rose_arm_geom.py`). The original renders used rose's fork v6/v9 geometry.
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
