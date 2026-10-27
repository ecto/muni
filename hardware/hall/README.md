# hall — 36 × 14 × 6 m, three aisle rows on a spine loop, 360 bays (concept mockup)

Layout and growth study only: boxes, no arms, no kinematics. It fixes one requirement early — tenders must be
able to carry bays between rows, which is what the spine loop is for.

Rebuild: `./build_hall.sh` (~10 s static, then growth stills + video). Needs PIL.

| file | what |
|---|---|
| `gen_hall.py` → `hall.loon`, `hall_cut_*.loon` | geometry and section cuts |
| `run_static.py`, `lab.py`, `rl.sh` | labelled stills `hall_top/iso/dock/carry.png` |
| `growth.py` → `hall_grow_{1,2,3}.loon/png`, `hall_growth.mp4` | growth story |

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
