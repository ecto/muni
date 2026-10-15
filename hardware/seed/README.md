# seed — the bay interface and the seed (concept mockup)

**(A) Bay interface** at detail: floor plate, slots, datum pins, coupler (48 V / CAN / air), busbar, spine rails,
tender plug; a bench docked as three bays (`bay.py` → `bay.loon`, `bay_one.loon`; renders `bay_*.png`,
part names in `bay_names.txt`).

**(B) The seed**: bench + genome + bought parts + collector/battery + hopper/sinter + a half-built bench_2 on
regolith (`seed.py` → `seed.loon`; `seed_*.png`, `seed_growth.mp4` via `seed_video.py`).

Rebuild: `./build_all.sh` (bay docs ~4 min because of booleans in the bench; video 144 frames).
`benchgen.py`, `gen_cell_v2.py`, `cellp.py` are local copies of the `../bench` generator.

## Rebuilding: dependencies

Status: concept mockup with modelled numbers; nothing here has been built.

- `VCAD_RENDER` — the `vcad-render` binary (default: `vcad-render` on PATH). Dark renders need `--theme dark` (ecto/vcad#926).
- `LOON2VCAD` — the `loon2vcad` binary (default: `loon2vcad` on PATH; build with `cargo build --release -p vcad-loon --example loon2vcad` in ecto/vcad).
- `ROSE_DIR` — checkout of the rose repo (default `~/Developer/rose`); arm geometry is read from `$ROSE_DIR/arm/fork/` (`rose_fork.py`, `rose_arm_geom.py`). The original renders used rose's fork v6/v9 geometry.
- `python3`; `magick` and `ffmpeg` (Homebrew) for labels and video.

`.vcad` files are not committed: they are regenerated from the `.loon` sources by the build script. Frame folders and per-frame PNGs are also regenerated.
