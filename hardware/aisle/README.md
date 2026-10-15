# aisle — 6 m spine, 24 bays, two tenders (concept mockup)

A 6 m spine with 24 bays, two tenders (plinth + lift + two rose fork-v6 arms each) and one `../bench` cell_v2
docked across bays F3–F5. Imports `../bench` (`gen_cell_v2.py`, `kin_v2.py`, `cellp.py`).

Rebuild: `./build_aisle.sh` (5+ min of frames).

| file | what |
|---|---|
| `aislep.py` | parameters |
| `gen_aisle.py` → `aisle.loon` | geometry; `aisle_tags.json` = part tags |
| `kin_aisle.py`, `reach_aisle.py` | tender FK/IK and reach |
| `make_frames_aisle.py`, `pose_util.py` | frames → `aisle_working.gif` |
| `subset_doc.py`, `label_doc_aisle.py`, `render_labeled_aisle.sh` | close-up and labelled stills (`aisle_*.png`) |

Findings: tenders can't pass each other on one spine; the lift needs 0–800 mm (600 was short); bays need a
60 mm keep-out at the spine edge. Arm bolt holes make the render hang, so they are off unless `HOLES=1`.

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `ROSE_DIR` — checkout of the rose repo (default `~/Developer/rose`); arm geometry is read from `$ROSE_DIR/arm/fork/` (`rose_fork.py`, `rose_arm_geom.py`). The original renders used rose's fork v6/v9 geometry.
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
