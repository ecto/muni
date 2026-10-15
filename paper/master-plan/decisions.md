# Master plan decisions

Decisions taken while reviewing the master plan. Each one is cited by the paper
or the hardware mockups. Newest first within a date.

## 2026-10-15 · Power, data and compute

| # | decision | why | rejected |
|---|---|---|---|
| D1 | **Product scope: humanoids (K1, rose) and maybe a drone (moth). BVR is dead.** | Focus. | Keeping BVR as the first customer for the standards. |
| D2 | **Bus: 36–60 V DC, unregulated.** Every load accepts the whole window; nothing assumes a regulated 48 V. | 13S NMC spans 39–54.6 V; RobStride parts and the blade standard already take it; under 60 V stays extra-low voltage. | 14S (58.8 V full leaves too little margin under 60 V). |
| D3 | **One pack everywhere: 13S2P Molicel P50B, 468 Wh, NMC.** Robot (two, A/B), dock, rack UPS, seed. | Keep it simple: one envelope, one BMS, one connector. | LFP for stationary uses (better cycle life, lower fire risk); revisit only if a city objects to NMC in a sidewalk dock. |
| D4 | **No hardwired e-stop loop.** The pack connector's mate-last auxiliary pin is the power enable. | Keep it simple; other humanoids don't carry one. The safety table's electronics layer stays "planned". | A dual-channel safety loop through every coupler and pack. |
| D5 | **Bay coupler unchanged: 48 V + CAN + air.** | Keep it simple. Bulk data (cameras) goes over the cell's own network, not the coupler. | Adding GbE or single-pair Ethernet to the coupler. |
| D6 | **Compute: identical GPU blades in slots.** One Jetson-class carrier on the blade standard, 30 W power class (in the blade EEPROM; CM5 blades stay 20 W). | One part and one interface on every body; how much compute a body has is a slot count, not an architecture. | Thin body + rack (dumb when the link drops); a bespoke GPU board per body. |
| D7 | **Slot 1 always runs S0, S1, the abstract state bus and perception, isolated at real-time priority. Slots 2–3 run S2 when fitted; otherwise S2 runs on the rack over the same seam.** | Maps ipse's existing split (`ipse-split`: Locomotion and Reflexes `OnBoard`, HardReasoning on- or off-board) onto hardware. A full body is option B (offline-capable); a one-blade body is option C. On S2 loss, S1 finishes the skill and stands. | A new on/off-board interface. |
| D8 | **A fan in the torso is fine.** The humanoid bay is cooled by a fan over its rear fins, rated for 3 blades (~90 W). | Removes the sealed-bay thermal risk (60 W at ~1 K/W vs a bare shell's 5–8 K/W). | Power-capping blades to 8 W; a passive 2-blade bay. |
| D9 | **rose vs K1 is measured, not asserted.** Same ipse build on both; report S0 tick jitter, inference latency and where S2 runs, per body. rose is first run with one blade, then with more. | rose should be better than the K1, and the difference should be a number. | — |
| D10 | **Board library, own vs buy.** Own first: BMS (safety and EUM instrument) and the bay coupler / spine hub (the standard). Next: blade carriers and backplanes. Step 2: torc. All designs open; PCB fab and assembly stay bought through chapter 2; a pick-and-place bay is a chapter-3 milestone. | Own the boards that define a standard or a measurement first. | — |
| D11 | **moth uses the same bus, CAN and blade standard, with its own lighter pack.** | A 2.65 kg, 468 Wh pack only suits a heavy-lift drone. | moth carrying the humanoid pack. |

| D12 | **moth has two markets: street perception and kino's directed camera. No military use, no defense work, never armed.** The safety commitment "no sales for weapons or military use" stands unchanged. No moth imagery leaves a street drone, only facts. | One body, two markets; the cheapest test that the standards are body-agnostic. | Drone defense (interception), considered and dropped 2026-10-15. |

## Corrections found in review

- The K1's onboard computer is **not a Jetson**. The perception machine is a
  Qualcomm QCS8550 (6 cores, 10 GB, Adreno GPU, Hexagon DSP, no CUDA), measured
  2026-08-05 on firmware v1.6.1.1; a separate motion machine runs the vendor
  control (`ipse/docs/deploy.md` §1). The vendor-summary "Jetson Orin NX 8 GB"
  was wrong.

## Open

- **S2 size** (parameters, memory). Decides whether a 16 GB Orin NX blade holds
  it or a Thor-class module is needed, which would not fit the 20 mm blade.
- **moth's mass budget**, and whether one ~150 g blade fits it.
