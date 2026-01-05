# BVR State Machine Reference

This document provides a complete reference for the BVR rover's operational state machine, including valid transitions, LED feedback, and safety mechanisms.

## State Machine Overview

**Location**: `bvr/firmware/crates/state/src/lib.rs`

The BVR state machine enforces safe operational modes and prevents unsafe transitions. It consists of 6 modes and 9 event types.

## Operational Modes

### Mode::Disabled (Initial State)
**Purpose**: Power-on state, motors not initialized

**Characteristics:**
- Motors completely off (no power)
- CAN bus may not be initialized
- Configuration loading in progress
- LED: Red solid

**Valid Exit Events:**
- `Event::Enable` → Idle

**Cannot:**
- Drive motors
- Enter Teleop/Autonomous directly

### Mode::Idle (Ready State)
**Purpose**: Ready to receive commands, motors enabled but stationary

**Characteristics:**
- Motors powered but receiving zero velocity
- Systems initialized and healthy
- Waiting for operational mode selection
- LED: Blue solid

**Valid Exit Events:**
- `Event::Enable` → Teleop
- `Event::Autonomous` → Autonomous
- `Event::Disable` → Disabled
- `Event::EStop` → EStop
- `Event::Fault` → Fault

**Cannot:**
- Drive (not in driving mode)

### Mode::Teleop (Human Control)
**Purpose**: Direct control via gamepad or keyboard

**Characteristics:**
- Human operator controls velocity
- Commands received via UDP/WebSocket
- Watchdog active (500ms timeout)
- Rate limiting applied
- LED: Green pulse (2s period)

**Valid Exit Events:**
- `Event::Disable` → Idle
- `Event::Autonomous` → Autonomous
- `Event::CommandTimeout` → Idle (watchdog)
- `Event::EStop` → EStop
- `Event::Fault` → Fault

**Safety:**
- Watchdog monitors command reception
- Rate limiter prevents sudden acceleration
- Page visibility tracking (web console)

### Mode::Autonomous (Self-Driving)
**Purpose**: Autonomous navigation without human input

**Characteristics:**
- Policy/planner controls velocity
- Vision and localization active
- Velocity limits enforced (1.0 m/s linear, 1.5 rad/s angular)
- Rate limiting applied
- LED: Cyan pulse (1.5s period)

**Valid Exit Events:**
- `Event::Disable` → Idle
- `Event::Enable` → Teleop
- `Event::EStop` → EStop
- `Event::Fault` → Fault

**Safety:**
- Lower velocity limits than teleop
- Emergency stop available
- Sensor monitoring active

### Mode::EStop (Emergency Stop)
**Purpose**: Immediate safety override, requires explicit release

**Characteristics:**
- Motors immediately stopped
- Cannot resume without release
- All driving blocked
- LED: Red flash (200ms, maximum visibility)

**Valid Exit Events:**
- `Event::EStopRelease` → Idle (ONLY valid exit)

**Blocked Events:**
- `Event::Enable` → (no change, stays in EStop)
- `Event::Autonomous` → (no change)
- `Event::Disable` → (no change)

**Safety:**
- One-way entry from ANY mode
- Explicit release required
- No direct path to driving modes
- Force estop method available: `force_estop()`

### Mode::Fault (Error State)
**Purpose**: System error detected, requires diagnosis

**Characteristics:**
- Motors stopped
- Error condition active (VESC timeout, sensor failure, etc.)
- Requires fault acknowledgment to clear
- LED: Orange flash (500ms)

**Valid Exit Events:**
- `Event::ClearFault` → Idle
- `Event::EStop` → EStop (safety override still works)

**Common Fault Triggers:**
- VESC heartbeat timeout (100ms)
- CAN bus error
- Battery critical voltage
- Sensor malfunction

## State Transition Diagram

```
                     ┌──────────────┐
                     │   Disabled   │
                     │  (Red solid) │
                     └──────┬───────┘
                            │ Enable
                            ↓
       ┌────────────────────────────────────┐
       │            Idle                    │
       │        (Blue solid)                │
       │    [Ready, motors enabled]         │
       └─┬────────┬───────────────┬─────────┘
         │        │               │
     Enable    Autonomous      Disable
         │        │               │
         ↓        ↓               ↓
    ┌────────┐ ┌──────────┐ ┌──────────┐
    │Teleop  │ │Autonomous│ │ Disabled │
    │(Green  │ │ (Cyan    │ └──────────┘
    │ pulse) │ │  pulse)  │
    └────┬───┘ └────┬─────┘
         │          │
         └────┬─────┘
              │
      Enable / Autonomous
      (mode switching)
              │
         ┌────┴─────────────────────┐
         │                           │
     Disable                      EStop
         │                      (any mode)
         ↓                           │
    ┌────────┐                       ↓
    │  Idle  │                ┌─────────────┐
    └────────┘                │    EStop    │
         ↑                    │ (Red flash) │
         │                    └─────┬───────┘
         │                          │
         │                   EStopRelease
         │                          │
         └──────────────────────────┘

                Fault (from any mode)
                        │
                        ↓
                 ┌──────────────┐
                 │     Fault    │
                 │(Orange flash)│
                 └──────┬───────┘
                        │ ClearFault
                        ↓
                    Idle
```

## Event Types

### Event::Enable
**Transitions:**
- Disabled → Idle
- Idle → Teleop
- Autonomous → Teleop

**Purpose:** Enable motors or enter teleop mode

**Blocked in:** EStop (must release first)

### Event::Disable
**Transitions:**
- Teleop → Idle
- Autonomous → Idle
- Idle → Disabled

**Purpose:** Stop driving or disable motors

### Event::Autonomous
**Transitions:**
- Idle → Autonomous
- Teleop → Autonomous

**Purpose:** Enter autonomous navigation mode

### Event::EStop
**Transitions:**
- ANY mode → EStop

**Purpose:** Emergency safety override

**Special:** Can be triggered from any mode, no exceptions

### Event::EStopRelease
**Transitions:**
- EStop → Idle

**Purpose:** Release emergency stop after condition cleared

**Note:** ONLY way to exit EStop

### Event::Fault
**Transitions:**
- ANY mode → Fault

**Purpose:** System error detected

**Triggers:**
- VESC timeout
- CAN bus error
- Battery critical
- Sensor malfunction

### Event::ClearFault
**Transitions:**
- Fault → Idle

**Purpose:** Acknowledge fault and return to ready state

**Requires:** Fault condition resolved

### Event::CommandTimeout
**Transitions:**
- Teleop → Idle

**Purpose:** Watchdog timeout, stop receiving commands

**Automatic:** Triggered by watchdog, not user input

### Event::ModeSwitch
**Transitions:**
- Teleop ↔ Autonomous

**Purpose:** Switch between operational modes

## Predicate Methods

### is_driving()
```rust
pub fn is_driving(&self) -> bool {
    matches!(self.mode, Mode::Teleop | Mode::Autonomous)
}
```

**Purpose:** Check if rover should accept motor commands

**Usage:**
```rust
if state_machine.is_driving() {
    send_velocity_command(twist);
} else {
    send_zero_velocity();
}
```

### is_safe()
```rust
pub fn is_safe(&self) -> bool {
    matches!(self.mode, Mode::Disabled | Mode::Idle | Mode::EStop)
}
```

**Purpose:** Check if configuration changes are allowed

**Usage:**
```rust
if state_machine.is_safe() {
    update_chassis_params(new_params);
} else {
    warn!("Cannot update config while driving");
}
```

### force_estop()
```rust
pub fn force_estop(&mut self) {
    warn!(mode = ?self.mode, "Force e-stop triggered");
    self.mode = Mode::EStop;
}
```

**Purpose:** Immediate e-stop without event handling

**When to use:**
- Critical safety override
- Panic handler
- Unrecoverable error

## LED Feedback Integration

LED color and pattern are automatically synchronized with state transitions.

### LED State Mapping

| Mode       | Color  | Pattern | Period | Brightness | RGB          |
|------------|--------|---------|--------|------------|--------------|
| Disabled   | Red    | Solid   | -      | 200/255    | (255, 0, 0)  |
| Idle       | Blue   | Solid   | -      | 200/255    | (0, 0, 255)  |
| Teleop     | Green  | Pulse   | 2000ms | 150-255    | (0, 255, 0)  |
| Autonomous | Cyan   | Pulse   | 1500ms | 150-255    | (0, 255, 255)|
| EStop      | Red    | Flash   | 200ms  | 255/255    | (255, 0, 0)  |
| Fault      | Orange | Flash   | 500ms  | 200/255    | (255, 165, 0)|

### LED Update Code

```rust
impl StateMachine {
    pub fn handle(&mut self, event: Event) {
        let old_mode = self.mode;

        // Handle state transition
        self.mode = match (self.mode, event) {
            // ... transition logic ...
        };

        // Update LEDs on mode change
        if self.mode != old_mode {
            self.update_leds();
        }
    }

    fn update_leds(&mut self) {
        let led_mode = match self.mode {
            Mode::Disabled => LedMode::solid(255, 0, 0, 200),
            Mode::Idle => LedMode::solid(0, 0, 255, 200),
            Mode::Teleop => LedMode::pulse(0, 255, 0, 2000),
            Mode::Autonomous => LedMode::pulse(0, 255, 255, 1500),
            Mode::EStop => LedMode::flash(255, 0, 0, 200),
            Mode::Fault => LedMode::flash(255, 165, 0, 500),
        };

        if let Err(e) = self.led_controller.set_mode(led_mode) {
            warn!(?e, "Failed to update LEDs");
        }
    }
}
```

## Logging Best Practices

All state transitions should be logged with context:

```rust
match (self.mode, event) {
    (Mode::Idle, Event::Enable) => {
        info!(old = ?Mode::Idle, new = ?Mode::Teleop, "Entering teleop mode");
        self.mode = Mode::Teleop;
    }
    (_, Event::EStop) => {
        warn!(mode = ?self.mode, "E-stop triggered");
        self.mode = Mode::EStop;
    }
    (Mode::EStop, Event::Enable) => {
        warn!("Cannot enable from e-stop, release required");
        // No state change
    }
    _ => {
        debug!(?self.mode, ?event, "Unhandled event");
    }
}
```

## Unit Tests

### Basic Transitions
```rust
#[test]
fn test_disabled_to_idle() {
    let mut sm = StateMachine::new();
    assert_eq!(sm.mode(), Mode::Disabled);

    sm.handle(Event::Enable);
    assert_eq!(sm.mode(), Mode::Idle);
}

#[test]
fn test_idle_to_teleop() {
    let mut sm = StateMachine::new();
    sm.handle(Event::Enable); // Disabled → Idle
    sm.handle(Event::Enable); // Idle → Teleop
    assert_eq!(sm.mode(), Mode::Teleop);
}
```

### E-Stop Tests
```rust
#[test]
fn test_estop_from_any_mode() {
    let mut sm = StateMachine::new();
    sm.handle(Event::Enable); // Idle
    sm.handle(Event::Enable); // Teleop

    sm.handle(Event::EStop);
    assert_eq!(sm.mode(), Mode::EStop);
}

#[test]
fn test_estop_requires_release() {
    let mut sm = StateMachine::new();
    sm.force_estop();
    assert_eq!(sm.mode(), Mode::EStop);

    // Try to enable (should be blocked)
    sm.handle(Event::Enable);
    assert_eq!(sm.mode(), Mode::EStop, "Still in e-stop");

    // Release
    sm.handle(Event::EStopRelease);
    assert_eq!(sm.mode(), Mode::Idle, "Released to Idle");
}
```

### Mode Switching Tests
```rust
#[test]
fn test_teleop_autonomous_switching() {
    let mut sm = StateMachine::new();
    sm.handle(Event::Enable); // Idle
    sm.handle(Event::Enable); // Teleop

    sm.handle(Event::Autonomous);
    assert_eq!(sm.mode(), Mode::Autonomous);

    sm.handle(Event::Enable);
    assert_eq!(sm.mode(), Mode::Teleop);
}
```

### Safety Predicates
```rust
#[test]
fn test_is_driving() {
    let mut sm = StateMachine::new();
    assert!(!sm.is_driving(), "Not driving in Disabled");

    sm.handle(Event::Enable); // Idle
    assert!(!sm.is_driving(), "Not driving in Idle");

    sm.handle(Event::Enable); // Teleop
    assert!(sm.is_driving(), "Driving in Teleop");

    sm.handle(Event::Autonomous);
    assert!(sm.is_driving(), "Driving in Autonomous");

    sm.handle(Event::EStop);
    assert!(!sm.is_driving(), "Not driving in EStop");
}
```

## Common Mistakes

### ❌ Bypassing E-Stop
```rust
// WRONG: Direct transition from EStop to Teleop
match (self.mode, event) {
    (Mode::EStop, Event::Enable) => {
        self.mode = Mode::Teleop; // DANGEROUS
    }
}
```

### ✅ Correct E-Stop Handling
```rust
// CORRECT: Require release first
match (self.mode, event) {
    (Mode::EStop, Event::EStopRelease) => {
        self.mode = Mode::Idle; // Go to safe state
    }
    (Mode::EStop, Event::Enable) => {
        warn!("Cannot enable from e-stop");
        // No state change
    }
}
```

### ❌ Missing Logging
```rust
// WRONG: Silent transition
self.mode = Mode::Teleop;
```

### ✅ Logged Transition
```rust
// CORRECT: Log with context
info!(old = ?self.mode, new = ?Mode::Teleop, "Mode transition");
self.mode = Mode::Teleop;
```

### ❌ Unhandled Events
```rust
// WRONG: Ignoring invalid events
match (self.mode, event) {
    (Mode::Idle, Event::Enable) => { /* ... */ }
    _ => { /* silently ignored */ }
}
```

### ✅ Explicit Unhandled Case
```rust
// CORRECT: Log unhandled events
match (self.mode, event) {
    (Mode::Idle, Event::Enable) => { /* ... */ }
    _ => {
        debug!(?self.mode, ?event, "Unhandled event");
    }
}
```

## Integration with Control Loop

```rust
// In bvrd main loop
loop {
    // Receive commands
    if let Some(cmd) = recv_command() {
        watchdog.feed();

        // Apply rate limiting
        let limited = rate_limiter.limit(cmd.twist, dt);

        // Send if in driving mode
        if state_machine.is_driving() {
            let wheels = diff_drive.mix(limited);
            drivetrain.send_commands(wheels)?;
        } else {
            drivetrain.send_zero()?;
        }
    }

    // Check watchdog
    if watchdog.is_timed_out() {
        state_machine.handle(Event::CommandTimeout);
    }

    // Handle external events
    if estop_button_pressed() {
        state_machine.handle(Event::EStop);
    }

    // Update telemetry
    telemetry.mode = state_machine.mode();
}
```

## References

- Implementation: `bvr/firmware/crates/state/src/lib.rs`
- LED integration: `bvr/firmware/crates/can/src/leds.rs`
- Main integration: `bvr/firmware/bins/bvrd/src/main.rs`
