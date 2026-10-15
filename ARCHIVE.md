# Archive

Muni began with the BVR, a sidewalk rover for snow removal. The plan has since moved to arms, pairs and humanoids (see the [master plan](paper/master-plan/master-plan.typ)), and the rover-era code left this repository in October 2026.

| What | Where it is now |
|---|---|
| `bvr/`: rover firmware (bvrd, the `muni` CLI), training, CAD, docs | tag [`archive/bvr-2026-10`](https://github.com/ecto/muni/tree/archive/bvr-2026-10/bvr) |
| `mcu/`: LED and tool-attachment boards on the rover's CAN bus | tag [`archive/bvr-2026-10`](https://github.com/ecto/muni/tree/archive/bvr-2026-10/mcu) |
| `depot/`: fleet services and the console | [ecto/depo](https://github.com/ecto/depo) (private), with its history |

To get the rover code back:

```bash
git checkout archive/bvr-2026-10 -- bvr mcu
```
