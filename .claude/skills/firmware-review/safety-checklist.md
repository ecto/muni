# Firmware Safety Verification Checklist

This document provides a detailed, line-by-line safety verification checklist for BVR firmware code reviews.

## Pre-Review Setup

Before starting the review:
1. Read the PR description and understand the intended changes
2. Check out the branch and build locally: `cargo build`
3. Run all tests: `cargo test`
4. Review git diff to identify modified files
5. Identify any changes to safety-critical crates (state, control, can)

## Critical File Checklist

### State Machine (`crates/state/src/lib.rs`)

#### Mode Enum Definition
- [ ] All modes are explicitly defined with clear documentation
- [ ] No modes added that bypass safety (e.g., "UnsafeTest", "BypassEStop")
- [ ] Mode transitions are documented

#### Event Handling
```rust
// Verify each match arm in handle()
pub fn handle(&mut self, event: Event) {
    match (self.mode, event) {
        // Check these patterns:
        - [ ] (_, Event::EStop) => Mode::EStop  // From ANY mode
        - [ ] (Mode::EStop, Event::EStopRelease) => Mode::Idle  // Only valid exit
        - [ ] (Mode::EStop, Event::Enable) => /* NO CHANGE */  // Blocked
        - [ ] (Mode::Disabled, Event::Enable) => Mode::Idle  // Not Teleop
        - [ ] All transitions log old and new mode
    }
}
```

#### Safety Predicates
```rust
// Verify these methods exist and are used correctly
- [ ] is_driving() returns true ONLY for Teleop and Autonomous
- [ ] is_safe() returns true ONLY for Disabled, Idle, EStop
- [ ] Motor commands gated by is_driving() check
- [ ] Configuration changes gated by is_safe() check
```

#### Unit Tests
- [ ] Test: E-stop from each operational mode
- [ ] Test: Cannot enable directly from e-stop
- [ ] Test: E-stop release goes to Idle (not Teleop)
- [ ] Test: Invalid transitions are rejected
- [ ] Test: Mode switching (Teleop ↔ Autonomous)

### Control Logic (`crates/control/src/lib.rs`)

#### Watchdog Implementation
```rust
// Verify Watchdog struct
- [ ] timeout: Duration field exists
- [ ] last_command: Option<Instant> field tracks last feed
- [ ] new(Duration) constructor sets timeout
- [ ] feed() updates last_command to Instant::now()
- [ ] is_timed_out() returns true if:
      - last_command is None (never fed)
      - elapsed time > timeout duration
```

#### Watchdog Integration (in bvrd main.rs)
```rust
// In main control loop
- [ ] Watchdog created with reasonable timeout (250-500ms)
- [ ] feed() called on EVERY valid command reception
- [ ] is_timed_out() checked in loop (not just once)
- [ ] Timeout triggers Event::CommandTimeout
- [ ] CommandTimeout transitions to Idle mode
- [ ] Reset on manual e-stop or disable
```

#### Rate Limiter
```rust
// Verify RateLimiter struct
- [ ] max_accel: f32 (typically 50.0 m/s²)
- [ ] max_decel: f32 (typically 15.0 m/s²)
- [ ] max_linear: f32 (from chassis config, ~5.0 m/s)
- [ ] max_angular: f32 (from chassis config, ~2.5 rad/s)
- [ ] prev: Twist (tracks previous command)

// In limit() method
- [ ] Direction change detection: prev.linear * target.linear < 0.0
- [ ] Uses max_decel for braking, max_accel otherwise
- [ ] Applies rate = accel * dt (delta time)
- [ ] Clamps to absolute velocity limits
- [ ] Updates prev for next iteration
- [ ] Handles dt = 0 gracefully (no divide-by-zero)
- [ ] Similar logic for angular velocity
```

#### Rate Limiter Integration
```rust
// In main control loop (bvrd)
- [ ] Rate limiter created with chassis parameters
- [ ] Applied to ALL velocity commands (teleop + autonomous)
- [ ] Delta time (dt) calculated per iteration
- [ ] Reset on mode transition or e-stop
- [ ] No bypass path exists (check for raw velocity sends)
```

#### Differential Drive Mixer
```rust
// Verify DiffDriveMixer
- [ ] Takes Twist (linear, angular, boost) as input
- [ ] Returns WheelVelocities (fl, fr, rl, rr)
- [ ] Formulas:
      left = linear - angular * track_width / 2
      right = linear + angular * track_width / 2
- [ ] Wheel radius and track width from chassis config
- [ ] Boost multiplier applied if enabled
- [ ] No divide-by-zero if track_width = 0
```

#### Unit Tests
- [ ] Test: Watchdog times out after duration
- [ ] Test: Watchdog reset on feed()
- [ ] Test: Rate limiter respects acceleration limits
- [ ] Test: Rate limiter detects direction change
- [ ] Test: Diff drive forward (left = right)
- [ ] Test: Diff drive rotation (left = -right)
- [ ] Test: Diff drive arc (left ≠ right, both positive)

### CAN Bus (`crates/can/src/vesc.rs` and `src/leds.rs`)

#### VESC Motor Controller
```rust
// Command ID constants
- [ ] CMD_SET_DUTY = 0
- [ ] CMD_SET_RPM = 3
- [ ] CMD_SET_CURRENT = 1
- [ ] IDs are correct per VESC protocol spec

// VESC ID configuration
- [ ] IDs loaded from bvr.toml (not hardcoded)
- [ ] 4 VESCs: FL=0, FR=1, RL=2, RR=3
- [ ] IDs are unique (no duplicates)

// Command encoding
- [ ] Frame ID = (CMD_ID << 8) | VESC_ID
- [ ] Extended CAN frame format used
- [ ] Big-endian byte order (to_be_bytes())
- [ ] Duty cycle clamped to [-1.0, 1.0]
- [ ] Duty value scaled: duty * 100_000 as i32

// Status parsing
- [ ] Bounds check: data.len() >= 8 before indexing
- [ ] Big-endian decode: i32::from_be_bytes()
- [ ] Conversion factors applied:
      current /= 10.0
      duty /= 1000.0
- [ ] Invalid values logged, not panic
```

#### LED Peripheral
```rust
// LED CAN IDs
- [ ] LED_CMD = 0x0B00 (Jetson → MCU)
- [ ] LED_STATUS = 0x0B01 (MCU → Jetson)
- [ ] Range 0x0B00-0x0BFF reserved for peripherals

// LED mode encoding
- [ ] StateLinked = 0x10 (MCU controls color)
- [ ] Mode updated on state transitions
- [ ] E-stop = red flash 200ms (highly visible)
- [ ] Idle = blue solid (calm, ready)
- [ ] Teleop = green pulse 2s
- [ ] Autonomous = cyan pulse 1.5s
```

#### CAN Error Handling
```rust
- [ ] CanError enum covers all error cases
- [ ] Timeouts handled gracefully (no panic)
- [ ] Invalid frame IDs rejected
- [ ] Socket errors logged with context
- [ ] Mock CAN bus for non-Linux builds
```

### Main Daemon (`bins/bvrd/src/main.rs`)

#### Initialization
- [ ] Config loaded from bvr.toml
- [ ] CAN bus opened with correct interface name
- [ ] State machine initialized to Disabled
- [ ] Watchdog created with config timeout
- [ ] Rate limiter created with chassis params
- [ ] All subsystems initialized before loop

#### Main Control Loop
```rust
loop {
    // Command reception
    - [ ] Commands received from teleop/autonomous
    - [ ] Watchdog fed on valid command
    - [ ] Invalid commands rejected (not fed to watchdog)

    // Safety checks
    - [ ] Watchdog timeout checked each iteration
    - [ ] is_driving() gates motor commands
    - [ ] E-stop input monitored

    // Control pipeline
    - [ ] Rate limiter applied to velocity commands
    - [ ] Diff drive mixer converts to wheel velocities
    - [ ] Velocities sent to VESCs via CAN
    - [ ] LED state updated on mode transitions

    // Telemetry
    - [ ] State published to metrics
    - [ ] VESC status monitored
    - [ ] Battery voltage tracked
}
```

#### Async Tasks
- [ ] Teleop server spawned with error logging
- [ ] Metrics publisher spawned
- [ ] Video stream spawned
- [ ] Discovery service spawned
- [ ] GPS receiver spawned
- [ ] All tasks have error handling (no unwrap)
- [ ] Graceful shutdown on Ctrl+C

#### Channels
- [ ] Command channel: mpsc bounded (buffer size reasonable, e.g., 100)
- [ ] Telemetry channel: watch (latest value)
- [ ] GPS channel: watch (latest fix)
- [ ] No unbounded channels (prevent memory growth)

## Configuration Review (`config/bvr.toml`)

### Chassis Parameters
- [ ] wheel_diameter in meters (e.g., 0.165)
- [ ] track_width in meters (e.g., 0.55)
- [ ] wheelbase in meters (e.g., 0.55)
- [ ] max_speed reasonable (e.g., 3.0 m/s)

### CAN Configuration
- [ ] interface = "can0"
- [ ] bitrate = 500000 (500kHz)
- [ ] vesc_ids = [0, 1, 2, 3]
- [ ] heartbeat_timeout_ms = 100

### Control Configuration
- [ ] loop_rate_hz = 100 (10ms period)
- [ ] command_timeout_ms = 250-500 (watchdog)
- [ ] pid_gains present if using PID

### Safety Limits
- [ ] battery_low_voltage < battery_nominal_voltage
- [ ] battery_critical_voltage < battery_low_voltage
- [ ] velocity limits match physical capabilities

## Common Safety Violations

### Critical Issues (Reject PR)
- [ ] ❌ Bypassing e-stop (direct transition to Teleop from EStop)
- [ ] ❌ No watchdog timeout checking
- [ ] ❌ Raw velocity commands sent without rate limiting
- [ ] ❌ CAN parsing without bounds checks (panic risk)
- [ ] ❌ Hardcoded VESC IDs different from config
- [ ] ❌ Little-endian used for VESC commands (wrong)
- [ ] ❌ Panic (unwrap/expect) in main control loop
- [ ] ❌ Blocking calls in async context

### Major Issues (Request fixes)
- [ ] ⚠️ Watchdog timeout too long (>1 second)
- [ ] ⚠️ Rate limiter acceleration too high (>100 m/s²)
- [ ] ⚠️ Missing unit tests for safety functions
- [ ] ⚠️ State transitions not logged
- [ ] ⚠️ No LED feedback on e-stop
- [ ] ⚠️ Unbounded channels

### Minor Issues (Suggest improvements)
- [ ] ℹ️ Error context could be more descriptive
- [ ] ℹ️ Magic numbers (should be constants or config)
- [ ] ℹ️ Missing documentation on safety-critical functions
- [ ] ℹ️ Test coverage could be improved

## Testing Safety Manually

### Watchdog Test
```bash
# Start bvrd
cargo run --bin bvrd

# Send commands via teleop (gamepad)
# Then stop sending commands

# Verify: Rover transitions to Idle after timeout (500ms)
# Verify: Motors stop
# Verify: Logs show "Command timeout"
```

### E-Stop Test
```bash
# Start in Teleop mode, rover moving
# Press e-stop button (or send E-Stop event)

# Verify: Immediate motor stop
# Verify: LEDs flash red
# Verify: Cannot resume without release
# Verify: Logs show "E-stop triggered"
```

### Rate Limiting Test
```bash
# In Teleop, command full speed forward
# Verify: Acceleration is gradual (not instant)
# Reverse direction immediately
# Verify: Deceleration applied first
```

## Post-Review

After reviewing:
1. Document any violations found in PR comments
2. Suggest fixes with code examples
3. Request tests for any new safety features
4. Approve only if all critical issues resolved
5. Consider manual testing for complex safety changes
