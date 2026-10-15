# Master plan decisions

Decisions taken while reviewing the master plan. Each one is cited by the paper
or the hardware mockups. Newest first within a date.

## 2026-10-15 · Power, data and compute

| # | decision | why | rejected |
|---|---|---|---|
| D1 | **Product scope: humanoids (K1, rose) and maybe a drone (moth). BVR is dead.** | Focus. | Keeping BVR as the first customer for the standards. |
| D2 | **Bus: 36–60 V DC, unregulated.** Every load accepts the whole window; nothing assumes a regulated 48 V. | 13S NMC spans 39–54.6 V; RobStride parts and the core standard already take it; under 60 V stays extra-low voltage. | 14S (58.8 V full leaves too little margin under 60 V). |
| D3 | **One pack everywhere: 13S2P Molicel P50B, 468 Wh, NMC.** Robot (two, A/B), dock, rack UPS, seed. | Keep it simple: one envelope, one BMS, one connector. | LFP for stationary uses (better cycle life, lower fire risk); revisit only if a city objects to NMC in a sidewalk dock. |
| D4 | **No hardwired e-stop loop.** The pack connector's mate-last auxiliary pin is the power enable. | Keep it simple; other humanoids don't carry one. The safety table's electronics layer stays "planned". | A dual-channel safety loop through every coupler and pack. |
| D5 | **Bay coupler unchanged: 48 V + CAN + air.** | Keep it simple. Bulk data (cameras) goes over the cell's own network, not the coupler. | Adding GbE or single-pair Ethernet to the coupler. |
| D6 | **Compute: identical GPU cores in slots.** One Jetson-class carrier on the core standard, 30 W power class (in the core EEPROM; CM5 cores stay 20 W). | One part and one interface on every body; how much compute a body has is a slot count, not an architecture. | Thin body + rack (dumb when the link drops); a bespoke GPU board per body. |
| D7 | **Slot 1 always runs S0, S1, the abstract state bus and perception, isolated at real-time priority. Slots 2–3 run S2 when fitted; otherwise S2 runs on the rack over the same seam.** | Maps ipse's existing split (`ipse-split`: Locomotion and Reflexes `OnBoard`, HardReasoning on- or off-board) onto hardware. A full body is option B (offline-capable); a one-core body is option C. On S2 loss, S1 finishes the skill and stands. | A new on/off-board interface. |
| D8 | **A fan in the torso is fine.** The humanoid bay is cooled by a fan over its rear fins, rated for 3 cores (~90 W). | Removes the sealed-bay thermal risk (60 W at ~1 K/W vs a bare shell's 5–8 K/W). | Power-capping cores to 8 W; a passive 2-core bay. |
| D9 | **rose vs K1 is measured, not asserted.** Same ipse build on both; report S0 tick jitter, inference latency and where S2 runs, per body. rose is first run with one core, then with more. | rose should be better than the K1, and the difference should be a number. | — |
| D10 | **Board library, own vs buy.** Own first: BMS (safety and EUM instrument) and the bay coupler / spine hub (the standard). Next: core carriers and backplanes. Step 2: torc. All designs open; PCB fab and assembly stay bought through chapter 2; a pick-and-place bay is a chapter-3 milestone. | Own the boards that define a standard or a measurement first. | — |
| D11 | **moth uses the same bus, CAN and core standard, with its own lighter pack.** | A 2.65 kg, 468 Wh pack only suits a heavy-lift drone. | moth carrying the humanoid pack. |

| D12 | **moth is for street perception (kino left out of the plan 2026-10-15). No military use, no defense work, never armed.** The safety commitment "no sales for weapons or military use" stands unchanged. moth follows the ground robots' street rules: imagery leaves the drone for mapping, with people and cars removed on board and delayed release; its filter's miss rate is measured and published. | One body, two markets; the cheapest test that the standards are body-agnostic. | Drone defense (interception), considered and dropped 2026-10-15. |

| D13 | **Stages climb in scale: Arm, Body, Cell, City, Planet, Moon, Mars, Sun.** Not "chapters". | Each stage is a step up in scale and is built by the one before. | Earth-first framing; numbered chapters. |
| D14 | **Doubling time is a headline number beside EUM.** | It sets how long the arc takes: ~75 doublings from 10 kW to the sun. | — |
| D15 | **Replication is licensed per copy, capped in rate, checked against a germ line, drilled for shutdown every generation; no foraging on Earth.** | A self-copying lineage must stay stoppable by people. | Relying on dependence on bought parts alone. |
| D16 | **Sun is a swarm; power is used where it is collected** (more collectors, cells and computation). No power beamed to Earth. | Locus of energy stated plainly. | Beamed power home. |

| D17 | **The ladder is Limb, Pair, Body, City, Planet, Moon, Mars, Sun. A pair (two limbs on any table) is the cell: it works, and it builds limbs, so it builds pairs.** The bench, bay standard, aisle, hall and lexicon are cut; the seed survives as the Moon concept. | One part everywhere: a limb is a product, half of a pair and an arm of a body. Homogeneity simplifies hardware, software, standards and replication. | Bench/aisle/hall cells with spines, bays and tenders. |
| D18 | **The paper runs in four parts: Why, Foundations, The stages, Honesty.** Laconic section titles. | Each stage reads as a whole; standards and rules come before the stages that use them. | Topic-ordered sections. |

| D19 | **Sizes are mini (K1 size), full (human size), mega (industrial), for both limbs and rose.** rose's arms are limbs, full stop; each rose takes the limb of its size. | Homogeneity per size class; today's fork v9 is limb full. | 4-DoF purpose-built humanoid arms; pro/max naming. |
| D20 | **The pair is a product first.** First buyers: labs and builders. First task: folding laundry, as demo, benchmark and market at once. | Replication is a capability the product grows into. | — |
| D21 | **Pairs learn sim-first in phyz.** Cloth exists in phyz (XPBD, VBD on a branch); a gradient through cloth contact is not yet shown, and folding is gated on it. | The loop applied to a task. | Teleop-first learning. |
| D22 | **rose mini starts now, in parallel, on limb mini.** Body overlaps Pair. | The K1 shows the limits of a bought body. | Waiting for pairs to sell. |
| D23 | **City sells clean streets and the map, bundled; pricing stays open.** | Set with the first city. | Per-mile pricing in the paper. |
| D24 | **The reader should finish with: this is real, I want in, awe, trust, hope.** Hope lives in War, City and the ending; far stages stay poetic. | — | Engineering detail for Planet through Sun. |
| D25 | **Stages are four-letter places, one scale: Desk → Shop → Road → City → Home → Moon → Mars → Star** (were Limb, Pair, Body, City, Planet, Moon, Mars, Sun). | Product names no longer double as stage names; each stage is a place, farther from home than the last. | Curb, Walk (street stage); Gaia, Land (planet stage); Sun, Halo (last stage). |
| D26 | **48 V everywhere, tools included.** Tools are bus devices: 48 V and CAN-FD through a wrist coupler when held by a robot; a *brick* (half a pack, 13S1P, 234 Wh) when not. First tools: drill/driver (torc clutch) and leaf blower. | One voltage simplifies every board; a pack splits into two bricks with nothing new. | A 15S pack of three 18 V-class bricks (cordless-tool compatible), dropped to keep 48 V. |

## Corrections found in review

- The K1's onboard computer is **not a Jetson**. The perception machine is a
  Qualcomm QCS8550 (6 cores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), measured
  2026-08-05 on firmware v1.6.1.1; a separate motion machine runs the vendor
  control (`ipse/docs/deploy.md` §1). The vendor-summary "Jetson Orin NX 8 GB"
  was wrong.

## Open

- **S2 size** (parameters, memory). Decides whether a 16 GB Orin NX core holds
  it or a Thor-class module is needed, which would not fit the 20 mm core.
- **moth's mass budget**, and whether one ~150 g core fits it.
