# Archive

Muni began with the BVR, a sidewalk rover for snow removal. The plan has since moved to arms, pairs and humanoids (see the [vision](paper/vision/vision.typ)), and the rover-era code left this repository in October 2026.

| What | Where it is now |
|---|---|
| `bvr/`: rover firmware (bvrd, the `muni` CLI), training, CAD, docs | tag [`archive/bvr-2026-10`](https://github.com/muni-works/muni/tree/archive/bvr-2026-10/bvr) |
| `mcu/`: LED and tool-attachment boards on the rover's CAN bus | tag [`archive/bvr-2026-10`](https://github.com/muni-works/muni/tree/archive/bvr-2026-10/mcu) |
| `depot/`: fleet services and the console | [ecto/depo](https://github.com/ecto/depo) (private), with its history |

A later BVR2 design note, `bvr/docs/plans/2026-02-22-bvr2-gantry-power-washer-design.md`, is at commit `3e10804`.

To get the rover code back:

```bash
git checkout archive/bvr-2026-10 -- bvr mcu
```
