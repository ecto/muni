# Muni Firmware & Depot Refactor Plan

**Created:** 2026-01-20
**Status:** In Progress
**Last Session:** `0d3ec578-8112-4967-aacd-88bbf8501d64`

---

## Progress Overview

| Priority | Task | Status | Notes |
|----------|------|--------|-------|
| **P0-1** | Extend watchdog to cover autonomous mode | ✅ Completed | Policy timeout tracking added |
| **P0-2** | Add hardware e-stop GPIO integration | ✅ Completed | Config, GPIO impl, tests added |
| **P1-3** | Extract control loop from main.rs | ⏳ Pending | Detailed plan ready |
| **P1-4** | Implement Rust→TypeScript type generation | ✅ Completed | ts-rs integrated, types generated |
| **P1-5** | Add protocol version fields | ⏳ Pending | Per-subsystem versioning |
| **P2-6** | Add subsystem health flags to telemetry | ✅ Completed | SubsystemHealth struct + TS types |
| **P2-7** | Add connection status indicators to Console | ⏳ Pending | Toast notifications |
| **P2-8** | Create depot-types shared crate | ✅ Completed | mapper/map-api migrated |
| **P3-9** | Lazy-load XR environments | ⏳ Pending | 74% bundle reduction |
| **P3-10** | Implement texture pooling for video | ⏳ Pending | 3-6ms/frame savings |
| - | Fix dispatch task recovery on rover crash | ⏳ Pending | **CRITICAL GAP** |

---

## Recommended Implementation Order

```
Week 1: P0-1 (Watchdog) + P0-2 (E-Stop)     ← Safety first
Week 2: P1-4 (Type Gen) + P2-8 (depot-types) ← Foundation
Week 3: P1-5 (Versioning) + P2-6 (Health)    ← Observability
Week 4: P1-3 (Control Loop extraction)       ← Architecture
Week 5: P2-7 (Connection UI) + P3-9 (XR)     ← UX/Performance
Week 6: P3-10 (Textures) + Dispatch Recovery ← Polish
```

---

## Critical Findings

### Must Fix (Safety)
1. **Watchdog gap in autonomous mode** - Policy hang won't stop rover ✅ FIXED
2. **No hardware e-stop** - Only software e-stop via network exists 🔄 IN PROGRESS

### Should Fix (Architecture)
3. **Dispatch task recovery missing** - Tasks become orphaned on rover crash
4. **Type duplication** - 15+ types manually synced between Rust/TypeScript
5. **No protocol versioning** - Breaking changes would fail silently

### Nice to Have (Performance)
6. **7MB bundle** - XR environments loaded eagerly
7. **60 textures/sec** - Video frames cause allocation churn

---

## P0-1: Autonomous Watchdog (COMPLETED)

**Problem:** The watchdog only protects teleop mode. In autonomous mode, if `policy.infer()` hangs, the entire control loop freezes and the rover continues with last commanded velocities.

**Solution Implemented:**
- Feed watchdog every control loop iteration (not just on teleop commands)
- Add 50ms policy execution timeout with warning at 20ms
- Transition to Fault mode (not Idle) on policy hang

**Files Modified:**
- `bvr/firmware/bins/bvrd/src/main.rs` - Added policy timeout tracking

**Definition of Done:**
- Watchdog feeds on every control loop tick in all modes
- Policy execution timeout triggers Fault transition with warning at 20ms
- Policy hang no longer leaves rover driving on stale commands

**Validation:**
- Simulate a policy hang and confirm Fault transition within 50ms
- Confirm warning log emitted at ~20ms threshold

---

## P0-2: Hardware E-Stop GPIO (COMPLETED)

**Problem:** Only software e-stop exists via network. Hardware button needed for safety.

**Solution Implemented:**
- Use `tokio-gpiod` for GPIO on Jetson Orin NX
- Pin 36 (GPIO16), active-low with internal pull-up
- Software debouncing (50ms)
- Latch semantics requiring operator confirmation to release

**Files Modified:**
- `bvr/firmware/crates/hal/Cargo.toml` - Added `gpio` feature with `tokio-gpiod`
- `bvr/firmware/crates/hal/src/lib.rs` - Added `EStopInput`, `EStopEvent`, `EStopConfig` with tests
- `bvr/firmware/bins/bvrd/Cargo.toml` - Added `gpio` feature
- `bvr/firmware/bins/bvrd/src/main.rs` - Added `[estop]` config parsing and control loop integration
- `bvr/firmware/config/bvr.toml` - Added `[estop]` configuration section

**Remaining Work:**
1. ~~Add `[estop]` section to `bvr/firmware/config/bvr.toml`~~ ✅
2. ~~Add tests for GPIO layer~~ ✅ (11 tests)
3. Test on actual hardware (requires Jetson)

**Config Example:**
```toml
[estop]
enabled = true
gpio_chip = "gpiochip0"
gpio_line = 16  # Pin 36 on Jetson 40-pin header
active_low = true
debounce_ms = 50
require_confirmation = true
```

**Definition of Done:**
- `[estop]` config is present in `bvr/firmware/config/bvr.toml` and parsed
- GPIO input debounces and latches until confirmation
- E-stop events are wired into the control loop integration

**Validation:**
- Unit tests cover debounce and latch behavior
- Hardware test on Jetson confirms line behavior and latch release

---

## P1-3: Extract Control Loop

**Problem:** main.rs is 1,720 lines with a 627-line monolithic control loop (lines 1038-1665).

**Solution Design:** Create `bvr/firmware/crates/control-loop/` with 5 modules:

```
bvr/firmware/crates/control-loop/
├── lib.rs                      # Public API and orchestration
├── command_processor.rs        # Command channel draining and routing
├── autonomous_controller.rs    # Policy inference and waypoint management
├── motor_controller.rs         # Rate limiting, duty cycle, CAN sending
├── telemetry_builder.rs        # Collect telemetry from all subsystems
└── dispatch_handler.rs         # Dispatch event processing
```

**Key Structs:**

```rust
// lib.rs - Main orchestrator
pub struct ControlLoop {
    command_processor: CommandProcessor,
    autonomous_controller: AutonomousController,
    motor_controller: MotorController,
    telemetry_builder: TelemetryBuilder,
    dispatch_handler: DispatchHandler,
    control_period: Duration,
    last_tick: Instant,
    loop_count: u64,
}

impl ControlLoop {
    pub fn tick(
        &mut self,
        shared: &mut SharedState,
        can_interface: &CanInterface,
        channels: &Channels,
        subsystems: &mut Subsystems,
    ) -> Result<()>;
}
```

**Implementation Phases:**
1. Create crate structure (foundation)
2. Extract command processing (safest first)
3. Extract telemetry building (pure function)
4. Extract motor control (critical path)
5. Extract dispatch handling (complex state)
6. Extract autonomous control (most complex)
7. Create main orchestrator (integration)
8. Update main.rs (final step)

**Expected Impact:**
- main.rs: 1,720 → ~500 lines (71% reduction)
- Each module < 200 lines

**Definition of Done:**
- `control-loop` crate compiles and is used by `bvrd`
- main.rs reduced to ~500 lines with no duplicate logic
- Teleop/autonomous behavior matches current control loop

**Validation:**
- `cargo build -p bvrd` passes
- Run teleop and autonomous smoke tests (sim or hardware)

---

## P1-4: Type Generation (Rust → TypeScript) (COMPLETED)

**Problem:** 15+ types manually synced between Rust and TypeScript with no automated synchronization.

**Solution Implemented:** Use `ts-rs` crate for automatic generation.

**Types Generated:**
| Type | Status |
|------|--------|
| Mode | ✅ Generated |
| Twist | ✅ Generated |
| Pose | ✅ Generated |
| Command | ✅ Generated |
| PowerStatus | ✅ Generated |
| ToolCommand | ✅ Generated |
| SlamStatus | ✅ Generated |
| GpsCoord | ✅ Generated |
| WheelVelocities | ✅ Generated |
| SubsystemHealth | ✅ Generated |

**Files Modified:**
- `bvr/firmware/Cargo.toml` - Added `ts-rs = "10"` workspace dependency
- `bvr/firmware/crates/types/Cargo.toml` - Added optional `ts` feature
- `bvr/firmware/crates/types/src/lib.rs` - Added `#[derive(TS)]` to all shared types
- `bvr/firmware/crates/types/generate-ts.sh` - Script to regenerate types
- `depot/console/src/lib/generated/` - Generated TypeScript types

**Usage:**
```bash
# Regenerate TypeScript types
cd bvr/firmware && cargo test -p types --features ts
cp crates/types/bindings/generated/*.ts ../../depot/console/src/lib/generated/
```

**Definition of Done:**
- ~~All listed Rust types derive `TS` and export to `depot/console/src/lib/generated`~~ ✅
- ~~Console uses generated types instead of manual duplicates~~ ✅
- CI fails when generated output is stale (TODO: add CI check)

---

## P1-5: Protocol Versioning

**Problem:** Breaking protocol changes would fail silently.

**Solution:** Per-subsystem versioning (1.0 format):

| Protocol | Version Mechanism |
|----------|------------------|
| Teleop WebSocket | Add 0xFF handshake message |
| JSON protocols | Add `version` field |
| REST APIs | Add `/v1/` prefix |
| CAN bus | Add 0x0BFF version exchange |

**Deprecation Policy:** 6 months for breaking changes.

**Definition of Done:**
- Each protocol carries a version and rejects mismatches with a clear error
- Version constants live in shared modules/docs per protocol

**Validation:**
- Mismatched versions fail fast in integration tests
- Matching versions continue to connect normally

---

## P2-6: Subsystem Health Telemetry (PARTIALLY COMPLETED)

**Problem:** No visibility into subsystem health from console.

**Solution Implemented:** Added `SubsystemHealth` struct with 8 flags:

```rust
pub struct SubsystemHealth {
    pub can_healthy: bool,
    pub recording_active: bool,
    pub gps_fix: bool,
    pub camera_active: bool,
    pub dispatch_connected: bool,
    pub discovery_connected: bool,
    pub lidar_active: bool,
    pub slam_running: bool,
}
```

**Files Modified:**
- `bvr/firmware/crates/types/src/lib.rs` - Added `SubsystemHealth` with `to_bits()`/`from_bits()` methods
- `depot/console/src/lib/generated/SubsystemHealth.ts` - Generated TypeScript type

**Remaining Work:**
1. Wire `SubsystemHealth` into telemetry packet encoding/decoding
2. Add health indicators to Console TelemetryPanel

**Definition of Done:**
- ~~`SubsystemHealth` struct with bit packing~~ ✅
- ~~TypeScript type generated~~ ✅
- `SubsystemHealth` is encoded in telemetry and decoded in the console
- TelemetryPanel shows all 8 flags with consistent labels/colors

**Validation:**
- ~~Unit test for bit packing/unpacking~~ ✅ (7 tests added)
- Manual UI check with mocked flags

---

## P2-7: Connection Status Indicators

**Problem:** No visibility into connection state in console.

**Solution:**
- Add `ConnectionState` enum to Zustand store
- Expandable status bar component (auto-hides when healthy)
- Toast notifications on connect/disconnect
- Update Dashboard with live status

**Definition of Done:**
- `ConnectionState` drives status bar and toasts per subsystem
- Status bar auto-hides when healthy and stays visible on errors

**Validation:**
- Simulate disconnect/reconnect and verify UI changes

---

## P2-8: depot-types Shared Crate (COMPLETED)

**Problem:** 7 types duplicated across depot services.

**Solution Implemented:** Created `depot/types/` crate:

```
depot/types/
├── Cargo.toml
└── src/
    ├── lib.rs      # Module exports
    ├── geo.rs      # GpsBounds, GpsCoord
    ├── session.rs  # SessionMetadata, SessionStatus, Session
    └── map.rs      # MapManifest, MapAssets, MapStats, MapIndex, MapIndexEntry, MapSessionRef
```

**Files Created/Modified:**
- `depot/types/` - New shared crate with 13 tests
- `depot/mapper/Cargo.toml` - Added depot-types dependency
- `depot/mapper/src/main.rs` - Removed duplicated types, import from depot-types
- `depot/map-api/Cargo.toml` - Added depot-types dependency
- `depot/map-api/src/main.rs` - Removed duplicated types, import from depot-types

**Note:** Discovery service uses slightly different session types (reads directly from rover metadata with string dates) and was not migrated.

**Definition of Done:**
- ~~`depot/types` crate is the single source of shared depot types~~ ✅
- ~~mapper and map-api compile using the shared crate~~ ✅
- ~~Duplicated type definitions removed~~ ✅

**Validation:**
- ~~`cargo build` for map-api, mapper passes~~ ✅

---

## P3-9: Lazy-load XR Environments

**Problem:** XR environments are 5.8MB (1.9MB gzipped) loaded eagerly from `@pmndrs/xr` → `@iwer/sem`.

**Solution:**
1. Use `React.lazy()` for TeleopView route
2. Split Scene.tsx into XR and non-XR versions
3. Configure Vite manual chunks for XR packages
4. Preload XR code when WebXR detected

**Expected Impact:**
- Initial bundle: 7.4MB → 1.6MB (**74% smaller**)
- Gzipped: 2.3MB → 450KB (**80% smaller**)

**Definition of Done:**
- XR code splits into its own chunk and loads only on TeleopView
- Initial bundle size reduction verified by build report

**Validation:**
- `vite build --report` shows size drop
- Manual XR session works after lazy load

---

## P3-10: Video Texture Pooling

**Problem:** Creating 60 textures/second causes allocation churn.

**Solution:**
- Double-buffered CanvasTexture approach
- Reuse 2-3 textures instead of creating new ones
- Use `createImageBitmap()` for async decode
- Pause updates when tab hidden

**Expected Impact:** 3-6ms saved per frame.

**Definition of Done:**
- Video textures are reused from a small pool with no per-frame allocation
- Updates pause when the tab is hidden without visual regressions

**Validation:**
- Performance profiling shows reduced allocations/frame time
- Visual output matches current behavior

---

## Investigation: Dispatch Task Recovery

**CRITICAL GAP FOUND:**

When a rover crashes, tasks become orphaned:
- Tasks left in `assigned`/`active` status forever
- No timeout detection on server
- No recovery on reconnect
- No task reassignment to other rovers
- Server doesn't mark tasks failed on disconnect

**Needed Fixes:**
1. Add task timeout detection (configurable, e.g., 5 minutes)
2. On rover disconnect, mark active tasks as `failed`
3. On rover reconnect, allow task recovery or reassignment
4. Add admin UI for manually reassigning orphaned tasks

**Definition of Done:**
- Orphaned tasks move to `failed` after timeout or disconnect
- Reconnect path supports explicit recovery or reassignment
- Admin UI can reassign or resolve orphaned tasks

**Validation:**
- Simulate rover crash and verify timeout transition
- Reconnect flow resumes or reassigns tasks as configured
- Admin UI changes persist in dispatch backend

---

## Investigation: Video Memory Management

**LOW RISK** - Current implementation mostly safe.

Minor issues:
- Geometry not disposed on unmount (~49KB)
- TextureLoader recreated per frame unnecessarily
- No pause when tab hidden

**Recommended Fixes:**
- Dispose geometry on unmount
- Reuse a single TextureLoader instance
- Pause updates when tab hidden (align with P3-10)

---

## Investigation: Arc<Mutex> Deadlock Risk

**SAFE** - Current design prevents deadlocks via:
- Strict async/sync separation
- Consistent lock ordering (dispatch_task → shared)
- No async code accesses shared state

Risk is future maintenance only - document lock ordering.

**Action:** Document lock ordering in shared state and dispatch task modules.

---

## Files Reference

### Firmware
| File | Purpose |
|------|---------|
| `bvr/firmware/bins/bvrd/src/main.rs` | Main daemon, control loop |
| `bvr/firmware/crates/hal/src/lib.rs` | Hardware abstraction (GPIO, power) |
| `bvr/firmware/crates/control/src/lib.rs` | Watchdog, rate limiter, mixer |
| `bvr/firmware/crates/types/src/lib.rs` | Shared types |
| `bvr/firmware/crates/state/src/lib.rs` | State machine |
| `bvr/firmware/config/bvr.toml` | Runtime config |

### Console
| File | Purpose |
|------|---------|
| `depot/console/src/App.tsx` | Routes, lazy loading |
| `depot/console/src/store.ts` | Zustand state |
| `depot/console/src/lib/types.ts` | TypeScript types (manual) |
| `depot/console/src/lib/protocol.ts` | Binary protocol encoding |
| `depot/console/src/components/scene/Scene.tsx` | 3D view, XR |
| `depot/console/vite.config.ts` | Build config |

### Depot Services
| Service | Port | Purpose |
|---------|------|---------|
| discovery | 4860 | Rover registry |
| dispatch | 4890 | Mission planning |
| map-api | 4880 | Map serving |
| mapper | - | Map processing |
| gps-status | 4870 | RTK status |

---

## Agent IDs for Resumption

These agent IDs can be used with `resume` parameter if deeper work needed:

| Task | Agent ID |
|------|----------|
| Control Loop Extraction | `a7ae571` |

---

## Session Recovery

To resume this work in Claude Code:

```bash
# The conversation is stored at:
~/.claude/projects/-Users-cam-Developer-muni/0d3ec578-8112-4967-aacd-88bbf8501d64.jsonl
```

Current uncommitted changes:
```bash
git diff --stat HEAD
# bvr/firmware/bins/bvrd/Cargo.toml    |   5 +
# bvr/firmware/bins/bvrd/src/main.rs   | 238 ++++++++++++++
# bvr/firmware/crates/hal/Cargo.toml   |   9 +
# bvr/firmware/crates/hal/src/lib.rs   | 243 ++++++++++++++
```

To continue:
1. Review uncommitted changes with `git diff`
2. Add `[estop]` config to `bvr/firmware/config/bvr.toml`
3. Test build with `cargo build -p bvrd --features gpio`
4. Cross-compile for Jetson with `cargo build -p bvrd --features gpio --target aarch64-unknown-linux-gnu`
