# arXiv Literature Review: Applicability to Muni

**Date:** 2026-01-27
**Scope:** 22 papers across 8 research areas, assessed against the Muni autonomous sidewalk snow removal platform.

---

## Executive Summary

We surveyed recent arXiv papers across behavioral cloning, shared autonomy, VLA models, Gaussian splatting SLAM, coverage planning, fleet management, sidewalk navigation, and goal-conditioned learning. Of 22 papers reviewed, **2 are HIGH priority** for near-term adoption, **3 are MEDIUM-HIGH**, and **6 are MEDIUM** (worth tracking). The remaining 11 are LOW priority -- intellectually interesting but not actionable given Muni's current architecture and operational needs.

### Priority Matrix

| Tier | Paper | Area | Codebase Impact | Effort |
|------|-------|------|----------------|--------|
| **HIGH** | MPC Active Safety (2106.14554) | Shared Autonomy | `control/`, `teleop/`, `console/` | 2-4 weeks |
| **HIGH** | Task Allocation Review (2501.08726) | Fleet Management | `depot/dispatch/` | 2-3 weeks |
| **MED-HIGH** | Walkability Analysis (2507.12148) | Sidewalk Nav | `bvr/firmware/`, `depot/map-api/`, `console/` | 1-4 weeks |
| **MED-HIGH** | Resilient Fleet Mgmt (2403.11034) | Fleet Management | `depot/dispatch/` | 6-9 weeks |
| **MED-HIGH** | DGS-SLAM Dynamic Filtering (2411.10722) | GS-SLAM | `depot/splat-worker/` | 2-4 weeks |
| MEDIUM | Robust Route Planning (2507.12067) | Sidewalk Nav | `depot/dispatch/` | 2-3 weeks |
| MEDIUM | SMORe (2311.02013) | Goal-Conditioned | `bvr/training/` | 2-3 weeks |
| MEDIUM | ConCPP (2403.10460) | Coverage Planning | `depot/dispatch/` | varies |
| MEDIUM | Fleet-Merge (2310.01362) | Fleet Management | `bvr/training/` | 1-2 weeks |
| MEDIUM | Diffusion Shared Autonomy (2302.12244) | Shared Autonomy | `control/`, `training/` | 3-5 weeks |
| MEDIUM | Teacher-Student Sidewalk Nav (2109.05603) | Sidewalk Nav | `bvr/training/` | 2-3 weeks |
| MEDIUM | UnderwaterVLA dual-brain (2509.22441) | VLA Models | `depot/dispatch/` | 2-3 weeks |
| MEDIUM | Generalization Guarantees (2008.01913) | BC / Imitation | `bvr/training/` | analytical |
| MEDIUM | BehAV (2409.16484) | BC / Imitation | `control/`, `bvr/firmware/` | 2-3 weeks |
| LOW | BESO (2304.02532) | Goal-Conditioned | -- | -- |
| LOW | RT-Sketch (2403.02709) | Goal-Conditioned | -- | -- |
| LOW | VOILA (2105.09371) | BC / Imitation | -- | -- |
| LOW | VLA Survey (2508.13073) | VLA Models | -- | -- |
| LOW | AC^2-VLA (2601.19634) | VLA Models | -- | -- |
| LOW | Hier-SLAM (2409.12518) | GS-SLAM | -- | -- |
| LOW | VIGS SLAM (2501.13402) | GS-SLAM | -- | -- |
| LOW | Weed Mowing Coverage (2111.10462) | Coverage Planning | -- | -- |
| LOW | Group Surfing (2104.05933) | Sidewalk Nav | -- | -- |
| LOW | Learned Arbitration (1906.12280) | Shared Autonomy | -- | -- |

---

## Tier 1: HIGH Priority (Act Now)

### 1. MPC Active Safety for Teleoperated Vehicles
**Paper:** Saparia, Schimpe & Ferranti (2021). arXiv:2106.14554. IV Workshop.

**What it does:** Model predictive control that runs alongside teleoperator commands, modifying them to avoid collisions while preserving operator intent. Includes a predictive display using the MPC horizon to compensate for teleop latency.

**Why it matters for Muni:** This is a direct evolution of the existing `CollisionGuard` in `bvr/firmware/crates/control/`. Current CollisionGuard only scales linear velocity down; MPC can also adjust angular velocity to find a collision-free path closest to what the operator intended. The predictive display addresses a known operational challenge (video latency).

**Key differences from current CollisionGuard:**
- CollisionGuard: binary arc projection, scales linear vel only
- MPC Safety: optimizes (v, omega) over full prediction horizon with continuous obstacle cost
- CollisionGuard: no path preview; MPC horizon streams to console via existing `path_stream.rs`

**Integration path:**
- Replace/augment `CollisionGuard` in `bvr/firmware/crates/control/` with MPC-based safety filter
- Use existing `DiffDriveMixer` kinematic model for MPC prediction
- Stream MPC horizon via `path_stream.rs` (MSG_PATH 0x25 already exists)
- Render predicted path overlay in `depot/console/` teleop view
- Sampling-based MPC approximation (sample multiple (v,omega) candidates, pick closest feasible) requires zero new dependencies
- Full QP approach: add `osqp` crate, estimated <5ms on Jetson Orin NX
- **No state machine changes required** -- operates as a transparent filter in Teleop mode

**Effort:** 2-4 weeks

---

### 2. Task Allocation in Mobile Robot Fleets (Survey)
**Paper:** Meseguer Valenzuela & Blanes Noguera (2025). arXiv:2501.08726.

**What it does:** Systematic review of 52 papers on fleet task allocation. Taxonomizes approaches: exact (MILP), auction-based, heuristic (GA, ACO), VRP formulations, and RL-based.

**Why it matters for Muni:** Dispatch currently assigns tasks to the first available rover with no optimization. Snow removal zones map cleanly to the VRP formulation (depot-based fleet, geographic tasks, return-to-depot constraints). Auction-based allocation fits Muni's WebSocket architecture naturally.

**Key finding:** Auction-based methods offer the best balance of decentralization and solution quality for dynamic environments. Energy-aware formulations reduce fleet cost by 15-30% vs. naive nearest-task heuristics.

**Integration path:**
- `depot/dispatch/src/main.rs`: Add cost-function-based assignment (rover reports proximity + battery SOC, dispatch picks minimum cost)
- `bvr/firmware/crates/dispatch/`: Rover computes and reports bid costs
- Database: Add assignment cost tracking for offline analysis
- Start with greedy single-round auction (simplest); graduate to MILP if fleet grows beyond 5+ rovers

**Effort:** 2-3 weeks (greedy auction); 1-2 months (full MILP)

---

## Tier 2: MEDIUM-HIGH Priority (Plan for Near-Term)

### 3. Walkability Analysis via Sidewalk Robots
**Paper:** Tong, Simoni, Arfvidsson & Martensson (2025). arXiv:2507.12148.

**What it does:** Uses a sensor-equipped robot (nearly identical hardware to BVR: ZED-F9P RTK-GPS, LiDAR, IMU, Jetson compute) to characterize sidewalk infrastructure: surface irregularity via IMU vertical acceleration, sidewalk width via LiDAR + segmentation, pedestrian density via YOLO.

**Why it matters for Muni:** Rovers already carry all needed sensors. Computing surface quality indices during routine plowing runs costs nothing. This positions Muni as a **municipal data platform**, not just a plow -- valuable for public works departments year-round.

**Integration path:**
1. New `sidewalk-quality` crate: IMU-based surface indices (few hundred lines of Rust). ~1 week.
2. Log to Rerun recordings alongside existing telemetry.
3. Aggregate per-segment scores in `depot/map-api/` (PostgreSQL).
4. Sidewalk condition heatmaps in `depot/console/` and `depot/grafana/`.
5. (Future) YOLOv8 pedestrian detection on CSI camera. ~2-3 weeks additional.

**Effort:** 1 week (IMU only) to 4 weeks (full pipeline with visualization)

---

### 4. Resilient Fleet Management with MCTS
**Paper:** Goutham & Stockar (2024). arXiv:2403.11034. ACC 2024.

**What it does:** Two-phase fleet management: offline MCTS + Branch-and-Bound for optimal task assignment, online tree-reuse replanning when rovers fail or battery degrades. Solutions within 5% of global optimum; replanning in <1 second.

**Why it matters for Muni:** Battery-powered rovers in winter = cold-degraded battery capacity. The 5-minute orphaned task timeout is naive -- tree-reuse replanning would optimally reassign remaining tasks across available rovers instead of just handing orphaned tasks to the next idle rover.

**Integration path:**
- New planner module in `depot/dispatch/` (MCTS + B&B solver in Rust)
- Pre-compute assignments for cron-scheduled recurring missions
- Replace orphaned task recovery with tree-reuse replanning
- Rover reports battery SOC to dispatch for energy-aware decisions
- Store search tree metadata in PostgreSQL for reuse across recurring missions

**Effort:** 6-9 weeks. Best adopted after Task Allocation (Paper 2) provides the assignment foundation.

---

### 5. DGS-SLAM Dynamic Object Filtering
**Paper:** Kong, Lee, Lee & Kim (2024). arXiv:2411.10722.

**What it does:** Removes dynamic objects (pedestrians, vehicles) from Gaussian splatting reconstructions using dual filtering: semantic segmentation masks + photometric residual checking. Produces clean static scene models.

**Why it matters for Muni:** `depot/splat-worker/` currently does not filter dynamic objects. Pedestrians captured during missions appear as ghostly artifacts in the Gaussian splat. Clean static maps improve mission review and any future costmap-from-splat pipeline.

**Integration path:**
- Pre-processing stage in `depot/splat-worker/`: video segmentation (YOLO-World + SAM or Track Anything) produces per-frame dynamic masks
- Modify Gaussian optimization to mask out dynamic regions
- Add histogram percentile filtering on render residuals as secondary filter
- No firmware changes -- entirely offline in depot GPU worker

**Effort:** 2-4 weeks of Python/CUDA work. Incremental to existing GPU infrastructure.

---

## Tier 3: MEDIUM Priority (Track / Adopt When Ready)

### 6. Robust Route Planning under Uncertainty
**Paper:** Tong & Simoni (2025). arXiv:2507.12067.

Robust shortest-path optimization accounting for sidewalk travel time uncertainty (pedestrian density, weather, obstacles). The DRSP and ellipsoidal formulations yielded best routes. Directly applicable to dispatch segment ordering -- which sidewalk blocks to service in what order, accounting for weather and pedestrian conditions. **Impact:** `depot/dispatch/`. **Effort:** 2-3 weeks.

### 7. SMORe: Score Models for Offline Goal-Conditioned RL
**Paper:** Sikchi et al. (2023). arXiv:2311.02013.

Advantage-weighted regression extracts better policies from noisy offline data than raw BC. When expert data dropped from 5% to 1%, SMORe degraded only 16% vs. GoFAR's 36%. Directly addresses Muni's pain point: early teleop demos will be messy. **Critical advantage:** final policy is a standard MLP, so `bvr/firmware/crates/policy/` needs zero changes -- only the training pipeline changes. **Adopt when:** BC performance plateaus with sufficient data. **Impact:** `bvr/training/`. **Effort:** 2-3 weeks.

### 8. ConCPP: Concurrent Multi-Robot Coverage
**Paper:** Mitra & Saha (2024). arXiv:2403.10460.

Hungarian-algorithm-based task allocation for multi-robot coverage. 1.6x speedup over horizon-based methods. The grid model and sync assumptions don't directly transfer, but two ideas are immediately adoptable: (a) zone-splitting for multi-rover coverage (~200 lines in `coverage.rs`), (b) Hungarian assignment for mission-to-rover matching (~100 lines in dispatch). Full ConCPP is over-engineered until fleet exceeds ~10 rovers. **Impact:** `depot/dispatch/`. **Effort:** Low for zone-splitting; high for full implementation.

### 9. Fleet-Merge: Policy Consolidation
**Paper:** Wang et al. (2024). arXiv:2310.01362. ICLR 2024.

Merges per-rover policies via permutation-aligned weight averaging (soft Birkhoff polytope). Reduces communication from TB/day to MB/day. For Muni's tiny MLP (5k params) and small fleet, centralizing data is trivial, so the bandwidth argument doesn't apply. **Adopt when:** fleet exceeds 3-5 rovers or policies grow larger. **Impact:** `bvr/training/`. **Effort:** 1-2 weeks for MLP merge script.

### 10. Diffusion-Based Shared Autonomy (Copilot)
**Paper:** Yoneda et al. (2023). arXiv:2302.12244. ICML 2023.

Denoising diffusion model as teleop copilot: partial forward-then-reverse diffusion nudges operator commands toward expert distribution. 68% success vs. 20.7% unassisted (Lunar Lander). **Barrier:** 50-step diffusion at 20ms control rate is infeasible without distillation. With consistency distillation to ~5 steps, each MLP inference is ~15us on Jetson, making 5-step diffusion ~75us -- feasible. **Adopt when:** MPC safety filter (Paper 1) is in place and more teleop data is collected. **Impact:** `bvr/training/`, `control/`. **Effort:** 3-5 weeks.

### 11. Teacher-Student Sidewalk Navigation
**Paper:** Sorokin et al. (2022). arXiv:2109.05603. IEEE RA-L.

Teacher trains in abstract 2D world (from OpenStreetMap), student learns via DAGGER in simulation with semantic features. 3.2 km navigated on real sidewalks. **Adoptable ideas:** abstract world from Muni's costmaps for teacher training (addresses data bottleneck), DAGGER over pure BC (reduces compounding errors). **Adopt when:** training pipeline matures beyond initial teleop data. **Impact:** `bvr/training/`, `depot/map-api/`. **Effort:** 2-3 weeks.

### 12. UnderwaterVLA Dual-Brain Architecture
**Paper:** Wang et al. (2025). arXiv:2509.22441.

Cloud VLM decomposes missions into sub-tasks via chain-of-thought; local model executes. The hierarchical split validates Muni's depot/rover architecture. **Actionable idea:** VLM-based mission decomposition at the dispatch layer (e.g., "clear snow from all sidewalks on Oak Street, prioritizing intersections"). This would be a depot-side enhancement calling a cloud LLM API. **Impact:** `depot/dispatch/`. **Effort:** 2-3 weeks.

### 13. Generalization Guarantees for Imitation Learning
**Paper:** Gao et al. (2020). arXiv:2008.01913.

PAC-Bayes theoretical bounds on policy generalization across novel environments. Practical value: answers "how many teleop demonstrations from diverse sidewalks does Muni need before the policy generalizes safely?" The sample complexity formulas can be applied analytically. The CVAE-based approach could improve generalization when demos span varied environments, with the deployed policy still extracted as a simple MLP from the CVAE decoder. **Impact:** `bvr/training/`. **Effort:** Analytical (no code changes for bounds); 2-3 weeks for CVAE training pipeline.

### 14. BehAV: Behavioral Cost Maps for Autonomous Navigation
**Paper:** arXiv:2409.16484.

Encodes semantic navigation rules (sidewalk following, pedestrian yielding) as behavioral cost maps. 82% success rate with human-like trajectories. The cloud API dependency (GPT-4o at 3.6s latency) is a dealbreaker for real-time use, but the behavioral cost map concept is cherry-pickable: layer semantic behavioral costs onto the existing A* costmap using a lightweight on-device segmentation model on the Jetson. **Impact:** `bvr/firmware/crates/control/`. **Effort:** 2-3 weeks (with lightweight segmentation model).

---

## Tier 4: LOW Priority (Not Recommended)

| Paper | Reason for LOW Rating |
|-------|----------------------|
| BESO (2304.02532) | Muni's action space is unimodal; diffusion adds latency/complexity for no nav benefit |
| RT-Sketch (2403.02709) | Vision-based manipulation goals; wrong modality entirely for sidewalk nav |
| VOILA (2105.09371) | Keypoint-based reward from video demos; requires camera-centric policy Muni doesn't have |
| VLA Survey (2508.13073) | VLAs are 600,000x larger than Muni's MLP; wrong paradigm for geometric nav |
| AC^2-VLA (2601.19634) | Optimizes VLA inference speed; no VLA to optimize |
| Hier-SLAM (2409.12518) | Semantic GS-SLAM for indoor; RTK-GPS makes SLAM tracking redundant |
| VIGS SLAM (2501.13402) | IMU-fused SLAM achieves 16-145cm ATE; RTK-GPS is 10-100x better |
| Weed Mowing (2111.10462) | Sparse target coverage; snow removal requires dense full-area sweeps |
| Group Surfing (2104.05933) | Pedestrian group following; winter sidewalks are pedestrian-sparse |
| Learned Arbitration (1906.12280) | Immature workshop paper; conflicts with discrete state machine safety model |

---

## Recommended Roadmap

### Phase 1: Immediate (current sprint)
1. **MPC Active Safety** -- Natural evolution of CollisionGuard. No state machine changes. Predictive display addresses known teleop latency issue. Path streaming infra already exists.
2. **Auction-Based Task Allocation** -- Simple greedy auction replaces first-available dispatch. Immediate improvement to multi-rover efficiency.

### Phase 2: Near-term (next 1-2 months)
3. **Walkability Sensing** -- IMU surface quality is ~1 week of work. Adds municipal data product value. Year-round deployment justification.
4. **DGS-SLAM Dynamic Filtering** -- Clean up splat-worker output. Incremental to existing GPU pipeline.

### Phase 3: Growth (fleet scaling)
5. **MCTS Resilient Fleet Management** -- Adopt when fleet exceeds 2-3 rovers and recurring missions are common. Builds on auction allocation from Phase 1.
6. **SMORe Training** -- Adopt when BC performance plateaus with sufficient demo data. Zero inference-side changes.

### Phase 4: Future exploration
7. **Robust Route Planning** -- When dispatch has segment-level historical data
8. **Teacher-Student Training** -- When training pipeline needs more data diversity
9. **Diffusion Copilot** -- After MPC safety filter is proven and more teleop data exists
10. **VLM Mission Decomposition** -- When natural-language mission specification is desired

---

## Cross-Cutting Observations

1. **Muni's MLP policy is appropriately sized.** None of the papers make a compelling case for larger on-rover models. The 7-dim state -> 5k-param MLP -> 10us inference is well-matched to the geometric navigation task. Improvements should come from better training methods (SMORe, DAGGER), not bigger models.

2. **The depot/rover split is validated.** UnderwaterVLA's dual-brain, the fleet management papers, and the dispatch-centric ideas all reinforce running heavy computation at the depot and keeping the rover lean.

3. **Safety filters > mode changes.** MPC active safety and diffusion copilot both operate as transparent command filters, preserving the state machine's safety model. Papers that proposed replacing the discrete mode system (learned arbitration) were the least applicable.

4. **Data products add strategic value.** Walkability analysis positions Muni beyond snow removal -- year-round sidewalk condition monitoring for municipal public works. This is a business case argument, not just a technical one.

5. **The training pipeline has room to grow.** SMORe, DAGGER, and Fleet-Merge all improve policy quality without changing the inference architecture. The training side (`bvr/training/`) is where research adoption will have the most leverage.
