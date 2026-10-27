# bench — cell_v2: the first bench cell (concept mockup)

1500 × 1000 × 1000 mm 4040 cell. Two rose fork-v6 arms stand upright on a plinth that slides on two SBR16
rails; fixtures for a mill, rack, reflow oven, winder and stock; panels; electronics bay. Superseded for the
paper by `../bench-shallow/` (the 1500 × 500 × 1000 box), but still imported by `../aisle/` and `../seed/`.

Rebuild everything: `./build_v2.sh` (stills ~2 min, then 337 video frames — the long pole).

| file | what |
|---|---|
| `cellp.py` | all parameters |
| `gen_cell_v2.py` → `cell_v2.loon` | geometry (`cell_v2_np.loon` = panels off) |
| `kin_v2.py` | FK/IK for the upright arms |
| `reach_v2.py` → `reach_v2.txt` | every station reached by both arms, 425 mm carriage window (q 270..695) |
| `loads_v2.py` → `loads_v2.txt` | SBR16UU block loads: worst case 317 N compression / 197 N lift-off, C0/F 4.4 |
| `make_frames_v2.py` → `joints_v2.txt`, `cell_v2_working.mp4` | poses, keyframes, video |
| `label_doc_v2.py`, `render_labeled_v2.sh`, `stills_v2.sh` | labelled stills (`cell_v2_*.png`) |
| `real/` | the earlier hanging-arm cell_real scripts, kept for reference only |

Key findings: 870 mm carriage travel fits the 1500 frame; arms mount with zero pose along the spine; the per-arm
controller lives in the foot ring, so the plinth needs no controller box. Catalogue ratings are assumed — verify.

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `ROSE_DIR` — checkout of the rose repo (default `~/Developer/rose`); arm geometry is read from `$ROSE_DIR/arm/fork/` (`rose_fork.py`, `rose_arm_geom.py`). The original renders used rose's fork v6/v9 geometry.
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
