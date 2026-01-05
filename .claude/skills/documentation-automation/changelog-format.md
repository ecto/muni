# CHANGELOG.md Format Guide

Complete reference for maintaining CHANGELOG.md using Keep a Changelog format with Conventional Commits.

## Format Specification

### Version Structure

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New features for the next release

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes

## [1.2.0] - 2024-01-15

### Added
- Released features from previous Unreleased section

### Changed
- Released changes

## [1.1.0] - 2024-01-01

...

[Unreleased]: https://github.com/user/repo/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/user/repo/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/user/repo/releases/tag/v1.1.0
```

## Categories

### Added
**Purpose**: New features, capabilities, or functionality added to the project.

**Use for**:
- New modules, crates, or components
- New API endpoints or functions
- New configuration options
- New tools or scripts
- New documentation sections

**Examples**:
```markdown
### Added
- Safety watchdog with 500ms timeout in `bvr/firmware/crates/control/src/lib.rs` to automatically transition to Idle mode when commands stop arriving
- LED feedback system in `mcu/crates/mcu-leds/src/lib.rs` with mode-specific colors: green pulse (teleop), cyan pulse (autonomous), red flash (e-stop)
- Binary WebSocket protocol for 100Hz command rate in `depot/console/src/lib/protocol.ts` with little-endian encoding
- Docker Compose profiles for GPU (`--profile gpu`) and RTK base station (`--profile rtk`) support in `depot/docker-compose.yml`
```

### Changed
**Purpose**: Modifications to existing functionality, including breaking changes.

**Use for**:
- API changes (signature, behavior)
- Configuration format changes
- Default value changes
- Refactored interfaces
- Renamed functions/modules

**IMPORTANT**: Breaking changes must be marked with **BREAKING** and include migration notes.

**Examples**:
```markdown
### Changed
- **BREAKING**: Renamed `State::set_mode()` to `State::handle()` in `bvr/firmware/crates/state/src/lib.rs` for clarity. Update calls from `sm.set_mode(Mode::Teleop)` to `sm.handle(Event::Enable)`
- Updated VESC CAN protocol to use big-endian byte order in `bvr/firmware/crates/can/src/vesc.rs:45-67` per VESC spec
- Increased default watchdog timeout from 250ms to 500ms in `bvr/firmware/config/bvr.toml` to reduce false positives on WiFi networks
- Improved React Three Fiber rendering performance by implementing lerp interpolation with delta time in `depot/console/src/components/scene/RoverModel.tsx:30-45`
```

**Migration Example**:
```markdown
### Changed
- **BREAKING**: Telemetry message format now uses 92 bytes minimum (was 80 bytes) in binary protocol

  Migration: Update decoder to read additional temperature fields:
  ```typescript
  // Before
  const telemetry = decodeTelemetry(data.slice(0, 80));

  // After
  const telemetry = decodeTelemetry(data);  // Reads full 92+ bytes
  ```
  Backward compatibility: Old clients will fail with "frame too short" error
```

### Deprecated
**Purpose**: Features scheduled for removal in future versions.

**Use for**:
- Functions/APIs marked for removal
- Configuration options being phased out
- Modules planned for replacement

**MUST include**:
- Version when deprecated
- Version when will be removed
- Replacement/migration path

**Examples**:
```markdown
### Deprecated
- `Watchdog::reset()` deprecated since v1.5.0, will be removed in v2.0.0. Use `Watchdog::feed()` instead for clearer intent
- `Mode::Manual` enum variant deprecated in favor of `Mode::Teleop` for consistency. Will be removed in v2.0.0
- Configuration option `use_legacy_protocol` in `bvr.toml` will be removed in next major version. Legacy protocol support ends March 2025
```

### Removed
**Purpose**: Features that have been removed (already deprecated).

**Use for**:
- Deleted functions/modules
- Removed configuration options
- Discontinued support for platforms/versions

**MUST include**:
- What was removed
- Migration path (if any)
- Reason for removal (if not obvious)

**Examples**:
```markdown
### Removed
- Removed `Mode::Manual` enum variant (deprecated in v1.5.0). All code should use `Mode::Teleop` instead
- Removed support for JSON telemetry protocol (deprecated in v1.3.0). All clients must use binary protocol with `MSG_TELEMETRY (0x10)`
- Removed `legacy_watchdog` feature flag. All builds now use unified watchdog implementation
```

### Fixed
**Purpose**: Bug fixes that don't change intended behavior.

**Use for**:
- Crash fixes
- Logic errors
- Memory leaks
- Race conditions
- Off-by-one errors
- Incorrect calculations

**Should include**:
- What was broken
- What now works correctly
- File/function reference
- Issue number (if applicable)

**Examples**:
```markdown
### Fixed
- Fixed panic in CAN frame parsing when receiving malformed VESC status frames shorter than 8 bytes in `bvr/firmware/crates/can/src/vesc.rs:167-191`. Now returns `CanError::InvalidFrame` (#42)
- Fixed WebSocket reconnection hanging indefinitely when rover is unreachable in `depot/console/src/hooks/useRoverConnection.ts:45-67`. Now uses exponential backoff with 30s maximum delay
- Fixed React Three Fiber angle interpolation wrapping incorrectly at 0°/360° boundary in `depot/console/src/components/scene/RoverModel.tsx:35-40`, causing rover to rotate the long way
- Fixed memory leak from blob URLs not being revoked when video frames update in `depot/console/src/hooks/useVideoStream.ts:78`. Added cleanup in effect return (#51)
```

### Security
**Purpose**: Security vulnerability fixes.

**CRITICAL RULES**:
- ⚠️ **NEVER include exploit details or proof-of-concept**
- ⚠️ **Wait for coordinated disclosure period**
- ⚠️ **Reference CVE if assigned**
- ⚠️ **Use vague language until patched widely**

**Use for**:
- Input validation issues
- Authentication/authorization bypasses
- Injection vulnerabilities
- Cryptographic issues
- Dependency updates for CVEs

**Examples**:
```markdown
### Security
- Fixed input validation issue in teleop command handler that could cause unexpected behavior
- Updated `tokio` dependency to 1.35.1 to address CVE-2024-XXXXX
- Improved WebSocket origin validation to prevent unauthorized connections
```

**After coordinated disclosure** (detailed advisory in separate document):
```markdown
### Security
- Fixed command injection vulnerability in SLCAN parser (CVE-2024-XXXXX). See SECURITY.md for details and coordinated disclosure timeline
```

## Entry Writing Guidelines

### Present Tense
Use present tense imperatives (like git commit messages):

✅ **Good**:
- "Add safety watchdog"
- "Fix CAN frame parsing"
- "Update telemetry protocol"

❌ **Bad**:
- "Added safety watchdog"
- "Fixed CAN frame parsing"
- "Updated telemetry protocol"

### Be Specific
Include file paths, function names, or line ranges for easy navigation:

✅ **Good**:
- "Add watchdog in `bvr/firmware/crates/control/src/lib.rs:162-193`"
- "Fix parsing in `parse_status1()` method"

❌ **Bad**:
- "Add watchdog" (where?)
- "Fix bug in parser" (which parser? which bug?)

### User-Focused Language
Describe **what changed for users**, not how it was implemented:

✅ **Good**:
- "Rover now automatically stops after 500ms of no commands"
- "E-stop requires explicit release before resuming operation"

❌ **Bad**:
- "Refactored watchdog to use Option<Instant>" (implementation detail)
- "Added EStopRelease event to state machine" (internal change)

### Include Context
Help readers understand **why** the change matters:

✅ **Good**:
- "Increase watchdog timeout to 500ms (was 250ms) to reduce false positives on high-latency WiFi networks"
- "Add rate limiting to prevent wheel slip during sudden acceleration changes"

❌ **Bad**:
- "Increase watchdog timeout"
- "Add rate limiting"

### Group Related Changes
Combine related changes into one entry:

✅ **Good**:
```markdown
### Added
- Complete e-stop safety system:
  - E-stop state in state machine requiring explicit release
  - Red flashing LED feedback (200ms period)
  - One-way transition from any mode to EStop
  - Explicit `EStopRelease` event required to exit
```

❌ **Bad** (separate entries for same feature):
```markdown
### Added
- E-stop state
- E-stop LED
- E-stop transition
- E-stop release
```

### Reference Issues
Link to GitHub issues/PRs when applicable:

```markdown
### Fixed
- Fix memory leak in video frame handling (#51)
- Resolve race condition in telemetry updates (#53, #54)
```

## Version Release Process

### 1. Prepare Release

Move entries from `[Unreleased]` to new version section:

**Before**:
```markdown
## [Unreleased]

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix Z
```

**After**:
```markdown
## [Unreleased]

## [1.3.0] - 2024-01-20

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix Z
```

### 2. Add Version Links

Update comparison links at bottom:

```markdown
[Unreleased]: https://github.com/user/muni/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/user/muni/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/user/muni/compare/v1.1.0...v1.2.0
```

### 3. Semantic Versioning

Choose version number based on changes:
- **Major (X.0.0)**: Breaking changes
- **Minor (1.X.0)**: New features (backward compatible)
- **Patch (1.2.X)**: Bug fixes only

## Common Mistakes

### ❌ Missing File Paths
```markdown
### Added
- Add watchdog  # Where? Which file?
```

### ✅ Include Paths
```markdown
### Added
- Add safety watchdog with 500ms timeout in `bvr/firmware/crates/control/src/lib.rs`
```

---

### ❌ Past Tense
```markdown
### Fixed
- Fixed the bug  # Use present tense
```

### ✅ Present Tense
```markdown
### Fixed
- Fix CAN frame parsing to validate buffer length
```

---

### ❌ Vague Descriptions
```markdown
### Changed
- Update protocol  # What changed? How does it affect users?
```

### ✅ Specific Descriptions
```markdown
### Changed
- **BREAKING**: Update telemetry protocol from 80 to 92 bytes to include motor temperatures. Clients must update decoders
```

---

### ❌ Implementation Details
```markdown
### Changed
- Refactor watchdog to use Option<Instant> instead of custom TimeTracker  # Internal detail
```

### ✅ User-Facing Impact
```markdown
### Added
- Add command watchdog to automatically stop rover after 500ms of no commands, preventing runaway if connection is lost
```

---

### ❌ Missing Breaking Change Marker
```markdown
### Changed
- Rename set_mode() to handle()  # Breaking! Should be marked
```

### ✅ Mark Breaking Changes
```markdown
### Changed
- **BREAKING**: Rename `State::set_mode()` to `State::handle()` for clarity. Update calls from `sm.set_mode(Mode::Teleop)` to `sm.handle(Event::Enable)`
```

## Full Example

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GPU-accelerated Gaussian splatting for 3D reconstruction in `depot/splat-worker` using PyTorch with CUDA support

### Changed
- Update depot console to React 19 with improved concurrent rendering performance

## [1.2.0] - 2024-01-15

### Added
- Safety watchdog with 500ms timeout in `bvr/firmware/crates/control/src/lib.rs` to automatically transition to Idle mode when commands stop arriving
- E-stop state machine transitions in `bvr/firmware/crates/state/src/lib.rs` requiring explicit release before resuming operation
- LED feedback system with mode-specific colors in `mcu/crates/mcu-leds/src/lib.rs`:
  - Green pulse (2s) for teleop mode
  - Cyan pulse (1.5s) for autonomous mode
  - Red flash (200ms) for e-stop

### Changed
- **BREAKING**: Rename `State::set_mode()` to `State::handle()` in `bvr/firmware/crates/state/src/lib.rs` for clarity. Update calls from `sm.set_mode(Mode::Teleop)` to `sm.handle(Event::Enable)`
- Increase default watchdog timeout from 250ms to 500ms in `bvr/firmware/config/bvr.toml` to reduce false positives on WiFi networks
- Update VESC CAN protocol to use big-endian byte order in `bvr/firmware/crates/can/src/vesc.rs` per official specification

### Fixed
- Fix panic in CAN frame parsing when receiving malformed VESC status frames in `bvr/firmware/crates/can/src/vesc.rs:167-191`. Now validates buffer length before indexing (#42)
- Fix WebSocket reconnection hanging indefinitely in `depot/console/src/hooks/useRoverConnection.ts:45-67`. Now uses exponential backoff with 30s maximum delay
- Fix memory leak from blob URLs not being revoked in video stream handler at `depot/console/src/hooks/useVideoStream.ts:78` (#51)

### Security
- Fix input validation issue in teleop command handler

## [1.1.0] - 2024-01-01

### Added
- Binary WebSocket protocol for 100Hz command rate with little-endian encoding in `depot/console/src/lib/protocol.ts`
- React Three Fiber 3D visualization with lerp interpolation for smooth rover motion in `depot/console/src/components/scene/`

### Fixed
- Fix angle wraparound at 0°/360° boundary causing incorrect rover rotation in 3D view

[Unreleased]: https://github.com/user/muni/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/user/muni/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/user/muni/releases/tag/v1.1.0
```

## References

- Keep a Changelog: https://keepachangelog.com/
- Semantic Versioning: https://semver.org/
- Conventional Commits: https://www.conventionalcommits.org/
