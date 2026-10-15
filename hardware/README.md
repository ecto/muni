# Hardware mockups

Concept mockups for the master plan (`../paper/master-plan/`). Everything here is generated from
Python/loon sources and rendered with vcad; the numbers are modelled, not measured. Nothing has been built.

| folder | what |
|---|---|
| [`gripper/`](gripper/) | gripper v2: EL05 rack-and-pinion parallel gripper for the rose fork-v9 tool roll, 1/8" sheet, swappable tips |
| [`bench/`](bench/) | bench v2 cell: 1500 × 1000 × 1000 4040 cell, two arms upright on a plinth on SBR16 rails, fixtures and panels |
| [`bench-shallow/`](bench-shallow/) | the 1500 × 500 × 1000 box as the spine module, three bays docked in front (the paper's first bench) |
| [`aisle/`](aisle/) | 6 m spine, 24 bays, two tenders (plinth + lift + 2 arms), a bench docked across three bays |
| [`hall/`](hall/) | 36 × 14 × 6 m hall, three aisle rows on a spine loop, 360 bays: layout and growth study, boxes only |
| [`seed/`](seed/) | the bay interface (datum pins, 48 V/CAN/air coupler, busbar, spine rails) and the seed on regolith |
| [`compute/`](compute/) | 10" toolless mini rack v2: CM5 blades, a real-GPU node, and the humanoid blade bay |
| [`battery/`](battery/) | rose A/B 13S2P 21700 battery pack, self-swapped by the robot onto a passive dock |

## Rebuilding

Each folder has a build script (`build*.sh`) that cds to its own directory. They need:

- `VCAD_RENDER` — `vcad-render` with `--theme dark` (ecto/vcad#926); defaults to `vcad-render` on PATH
- `LOON2VCAD` — vcad's `loon2vcad` example; defaults to `loon2vcad` on PATH
- `ROSE_DIR` — the rose repo, for arm geometry (bench, bench-shallow, aisle, seed); defaults to `~/Developer/rose`
- `python3`, plus `magick` and `ffmpeg` for labels and video (PIL for the hall)

`aisle/` imports `../bench`. `.vcad` files, frame folders and per-frame PNGs are not committed; the build
scripts regenerate them from the `.loon` sources.
