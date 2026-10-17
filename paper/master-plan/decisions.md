# Master plan decisions

Decisions taken while reviewing the master plan. Each one is cited by the paper
or the hardware mockups. Newest first within a date.

## 2026-10-15 · Power, data and compute

| # | decision | why | rejected |
|---|---|---|---|
| D1 | **Product scope: humanoids (K1, rose) and maybe a drone (moth). BVR is dead.** | Focus. | Keeping BVR as the first customer for the standards. |
| D2 | **Bus: 36–60 V DC, unregulated.** Every load accepts the whole window; nothing assumes a regulated 48 V. | 13S NMC spans 39–54.6 V; RobStride parts and the kore standard already take it; under 60 V stays extra-low voltage. | 14S (58.8 V full leaves too little margin under 60 V). |
| D3 | **One pack everywhere: 13S2P Molicel P50B, 468 Wh, NMC.** Robot (two, A/B), dock, rack UPS, seed. | Keep it simple: one envelope, one BMS, one connector. | LFP for stationary uses (better cycle life, lower fire risk); revisit only if a city objects to NMC in a sidewalk dock. |
| D4 | **No hardwired e-stop loop.** The pack connector's mate-last auxiliary pin is the power enable. | Keep it simple; other humanoids don't carry one. The safety table's electronics layer stays "planned". | A dual-channel safety loop through every coupler and pack. |
| D5 | **Bay coupler unchanged: 48 V + CAN + air.** | Keep it simple. Bulk data (cameras) goes over the cell's own network, not the coupler. | Adding GbE or single-pair Ethernet to the coupler. |
| D6 | **Compute: identical GPU kores in slots.** One Jetson-class carrier on the kore standard, 30 W power class (in the kore EEPROM; CM5 kores stay 20 W). | One part and one interface on every body; how much compute a body has is a slot count, not an architecture. | Thin body + rack (dumb when the link drops); a bespoke GPU board per body. |
| D7 | **Slot 1 always runs S0, S1, the abstract state bus and perception, isolated at real-time priority. Slots 2–3 run S2 when fitted; otherwise S2 runs on the rack over the same seam.** | Maps ipse's existing split (`ipse-split`: Locomotion and Reflexes `OnBoard`, HardReasoning on- or off-board) onto hardware. A full body is option B (offline-capable); a one-core body is option C. On S2 loss, S1 finishes the skill and stands. | A new on/off-board interface. |
| D8 | **A fan in the torso is fine.** The humanoid bay is cooled by a fan over its rear fins, rated for 3 kores (~90 W). | Removes the sealed-bay thermal risk (60 W at ~1 K/W vs a bare shell's 5–8 K/W). | Power-capping kores to 8 W; a passive 2-core bay. |
| D9 | **rose vs K1 is measured, not asserted.** Same ipse build on both; report S0 tick jitter, inference latency and where S2 runs, per body. rose is first run with one kore, then with more. | rose should be better than the K1, and the difference should be a number. | — |
| D10 | **Board library, own vs buy.** Own first: BMS (safety and EUM instrument) and the bay coupler / spine hub (the standard). Next: kore carriers and backplanes. Step 2: torc. All designs open; PCB fab and assembly stay bought through chapter 2; a pick-and-place bay is a chapter-3 milestone. | Own the boards that define a standard or a measurement first. | — |
| D11 | **moth uses the same bus, CAN and kore standard, with its own lighter pack.** | A 2.65 kg, 468 Wh pack only suits a heavy-lift drone. | moth carrying the humanoid pack. |

| D12 | **moth is for street perception (kino left out of the plan 2026-10-15). No military use, no defense work, never armed.** The safety commitment "no sales for weapons or military use" stands unchanged. moth follows the ground robots' street rules: imagery leaves the drone for mapping, with people and cars removed on board and delayed release; its filter's miss rate is measured and published. | One body, two markets; the cheapest test that the standards are body-agnostic. | Drone defense (interception), considered and dropped 2026-10-15. |

| D13 | **Stages climb in scale: Arm, Body, Cell, City, Planet, Moon, Mars, Sun.** Not "chapters". | Each stage is a step up in scale and is built by the one before. | Earth-first framing; numbered chapters. |
| D14 | **Doubling time is a headline number beside EUM.** | It sets how long the arc takes: ~75 doublings from 10 kW to the sun. | — |
| D15 | **Replication is licensed per copy, capped in rate, checked against a germ line, drilled for shutdown every generation; no foraging on Earth.** | A self-copying lineage must stay stoppable by people. | Relying on dependence on bought parts alone. |
| D16 | **Sun is a swarm; power is used where it is collected** (more collectors, cells and computation). No power beamed to Earth. | Locus of energy stated plainly. | Beamed power home. |

| D17 | **The ladder is Lymb, Duet, Body, City, Planet, Moon, Mars, Sun. A duet (two lymbs on any table) is the cell: it works, and it builds lymbs, so it builds duets.** The bench, bay standard, aisle, hall and lexicon are cut; the seed survives as the Moon concept. | One part everywhere: a lymb is a product, half of a duet and an arm of a body. Homogeneity simplifies hardware, software, standards and replication. | Bench/aisle/hall cells with spines, bays and tenders. |
| D18 | **The paper runs in four parts: Why, Foundations, The stages, Honesty.** Laconic section titles. | Each stage reads as a whole; standards and rules come before the stages that use them. | Topic-ordered sections. |

| D19 | **Sizes are mini (K1 size), full (human size), mega (industrial), for both lymbs and rose.** rose's arms are lymbs, full stop; each rose takes the lymb of its size. | Homogeneity per size class; today's fork v9 is lymb full. | 4-DoF purpose-built humanoid arms; pro/max naming. |
| D20 | **The duet is a product first.** First buyers: labs and builders. First task: folding laundry, as demo, benchmark and market at once. | Replication is a capability the product grows into. | — |
| D21 | **Duets learn sim-first in phyz.** Cloth exists in phyz (XPBD, VBD on a branch); a gradient through cloth contact is not yet shown, and folding is gated on it. | The loop applied to a task. | Teleop-first learning. |
| D22 | **rose mini starts now, in parallel, on lymb mini.** Body overlaps Duet. | The K1 shows the limits of a bought body. | Waiting for duets to sell. |
| D23 | **City sells clean streets and the map, bundled; pricing stays open.** | Set with the first city. | Per-mile pricing in the paper. |
| D24 | **The reader should finish with: this is real, I want in, awe, trust, hope.** Hope lives in War, City and the ending; far stages stay poetic. | — | Engineering detail for Planet through Sun. |
| D25 | **Stages are four-letter places, one scale: Desk → Shop → Road → City → Home → Moon → Mars → Star** (were Lymb, Duet, Body, City, Planet, Moon, Mars, Sun). | Product names no longer double as stage names; each stage is a place, farther from home than the last. | Curb, Walk (street stage); Gaia, Land (planet stage); Sun, Halo (last stage). |
| D26 | **48 V everywhere, tools included.** Tools are bus devices: 48 V and CAN-FD through a wrist coupler when held by a robot; a *brick* (half a pack, 13S1P, 234 Wh) when not. First tools: drill/driver (torc clutch) and leaf blower. | One voltage simplifies every board; a pack splits into two bricks with nothing new. | A 15S pack of three 18 V-class bricks (cordless-tool compatible), dropped to keep 48 V. |
| D27 | **Tools mount on a tool changer at the wrist, in the gripper's place:** two datum pins, a quarter-turn ring, 48 V + CAN-FD contacts and a mate-last enable pin. First tools, in order: the litter grabber (testbed: the K1, through an adapter plate), the drill/driver, the leaf blower. | Rigid and precise, no trigger to pull; the same enable rule as the pack. The K1 tests it before Rose exists. | A smart handle the gripper closes on (more human, harder to make rigid). |
| D28 | **Software first: Desk's first step is revenue from kyte.** frog, the coding agent for local models, is free; kyte (hosted inference on tang, named 2026-10-16; spoken "kite") is its default non-local model and the thing we sell, priced per finished task. Its revenue funds the lymb. Gate: ≥ $10,000 a month for 3 months running, no outside money (proposed). | Software earns now, while hardware is slow and capital-hungry; it funds the arm without diluting the plan. | Charging for frog itself (frog Pro); licensing tang to robotics teams. |
| D29 | **The two-lymb product is named duet** (was pair). | Pronounceable, common, poetic, and it sounds like "do it"; ownable in robotics (Duet3D makes printer boards). "pair" stays only as a plain word (a pair of arms, a camera pair). | twyn (collides with the digital twin), yoke, dyad (coolest, too unfamiliar). |
| D30 | **kyte rents compute in three steps, each opened by demand, and runs on tang alone (no SGLang or vLLM):** open-model APIs first while tang learns to serve (batched decode, session cache slots, full-speed backend on rented NVIDIA GPUs; tang runs through wgpu today); one rented H200 (~\$2,500/month) running tang once it serves frog's tasks and demand nears ~23,000 tasks/month; another H200 each time the last is ~40% busy. Price per finished task (~\$0.25 proposed) with a token cap. | Owning the whole stack is the margin: no one else's serving layer, kernels tuned to frog's workload, every tang gain lands in cost per task. Open-model tokens sell near cost (~\$0.03 a task on DeepSeek V4 Flash), so the margin is per task against a frontier bill of \$1–3. | Renting GPUs up front; serving on SGLang first (decided against 2026-10-16: owning the engine is the advantage); reselling a speed provider; charging per token. |
| D31 | **The plan's organizing idea is an autonomous company, amplified by intelligence:** a company that grows on its own, its work increasingly done by its own machines and software (frog writes its code, kyte earns its revenue, duets build its machines), with people in the loop where it matters: they set what it may do, license each division, read the receipts. "Lineage" gives way to "the company" and "branch" (a site's duets and the machines they built, under one license). | Legible to investors and cities; honest about control (a company has owners and a board, a lineage sounds unsupervised); the replication rules read as governance. | Lineages of machines (sounds biological and unsupervised); a DAO framing (reads as crypto). |
| D32 | **The autonomous company gets its own number and section.** A fourth headline number, the labor fraction (people's share of the work, by function); a Company section in What: functions × stages, the bought intelligence fraction, controls on agents (caps, approvals, signed receipts, no self-raises, quarantine), governance (proposed: a board of ≥ 3, no single key can license a division, a named successor, a public license record). Capital, liability and competition are named as unknowns. | Without its own metric and controls, "grows on its own" is a slogan; agents with authority need the same limits as duets; governance by one person is the largest real risk. | Leaving the company implicit in the stages. Board members and timing are not decided. |

## Corrections found in review

- The K1's onboard computer is **not a Jetson**. The perception machine is a
  Qualcomm QCS8550 (6 kores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), measured
  2026-08-05 on firmware v1.6.1.1; a separate motion machine runs the vendor
  control (`ipse/docs/deploy.md` §1). The vendor-summary "Jetson Orin NX 8 GB"
  was wrong.

## Open

- **S2 size** (parameters, memory). Decides whether a 16 GB Orin NX kore holds
  it or a Thor-class module is needed, which would not fit the 20 mm kore.
- **moth's mass budget**, and whether one ~150 g kore fits it.
