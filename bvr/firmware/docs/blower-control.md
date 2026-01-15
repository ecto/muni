# Blower Control System Integration

This document describes the integration of blower control into the bvrd firmware for the Base Vectoring Rover (BVR). The blower is a VESC-controlled motor integrated with the autonomy state machine for automated snow clearing operations.

## System Overview

### Architecture

The blower control system consists of:

- **Hardware**: VESC motor controller with CAN interface (dedicated CAN ID)
- **Software**: Integration into bvrd's existing state machine and control loop
- **Control Modes**:
  - **Autonomous**: Blower power proportional to rover velocity (0-50% max)
  - **Teleop**: Operator manual control (0-100%)
  - **E-stop**: Immediate shutdown with safety interlock

### Control Flow

```
┌─────────────────────────────────────────────────────────────┐
│ State Machine (state crate)                                 │
│   Mode: Disabled | Idle | Teleop | Autonomous | EStop       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Blower Controller (new crate: blower)                       │
│   - Velocity-based autonomous control                       │
│   - Operator override in teleop                             │
│   - Power ramping (rate limiting)                           │
│   - Safety checks (state validation)                        │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ VESC Motor Controller (CAN bus)                             │
│   Command: SetDuty (duty cycle control)                     │
│   CAN ID: 0x0C (configurable)                               │
└─────────────────────────────────────────────────────────────┘
```

## CAN Protocol

### VESC CAN Commands

The blower uses VESC's standard CAN protocol over extended (29-bit) CAN frames. The existing `can::vesc` module provides all necessary primitives.

#### Extended CAN ID Format

```
Bits 28-8:  Command ID (8 bits)
Bits 7-0:   VESC ID (8 bits)

Extended ID = (CommandID << 8) | VESC_ID
```

#### Command: SetDuty (0x00)

Sets motor duty cycle from -1.0 (full reverse) to 1.0 (full forward).

**Frame Structure:**
```rust
ID:   (0x00 << 8) | blower_can_id  // Extended frame
Data: [i32 big-endian]             // duty * 100,000
```

**Example:** 50% forward duty on VESC ID 0x0C
```
Extended ID: 0x000C
Data: [0x00, 0x01, 0x86, 0xA0]  // 50000 = 0.5 * 100,000
```

### CAN Address Configuration

The blower VESC must be assigned a unique CAN ID that doesn't conflict with drivetrain VESCs (0-3) or MCU devices (0x0B00+).

**Recommended CAN IDs:**
- `0x0C` (12) - Primary blower
- `0x0D` (13) - Secondary blower (if dual-blower configuration)

Configure via VESC Tool:
1. Connect VESC via USB
2. App Settings → General → CAN Settings
3. Set "VESC ID on CAN-bus" to `12` (0x0C)
4. Enable "Send Status over CAN"
5. Write configuration and reboot VESC

### Example Code

```rust
use can::vesc::Vesc;
use can::Bus;

// Create VESC instance for blower
let blower = Vesc::new(0x0C);

// Build duty cycle command (50% power)
let duty = 0.5;
let frame = blower.build_duty_frame(duty);

// Send via CAN bus
bus.send(&frame)?;
```

**Using CommandId enum:**
```rust
use can::vesc::CommandId;
use can::Frame;

fn build_blower_command(blower_id: u8, duty: f32) -> Frame {
    let can_id = ((CommandId::SetDuty as u32) << 8) | (blower_id as u32);
    let duty_scaled = (duty.clamp(-1.0, 1.0) * 100_000.0) as i32;
    Frame::new_extended(can_id, &duty_scaled.to_be_bytes())
}
```

## Control Modes

### Autonomous Mode

In autonomous operation, blower power scales with rover linear velocity:

```
blower_power = velocity_linear / max_velocity * max_power_autonomous
```

**Behavior:**
- Rover stopped (v < 0.1 m/s): Blower off (0%)
- Rover moving: Blower power proportional to velocity
- Maximum: 50% duty (configurable via `max_power_autonomous`)
- Ramps smoothly with `power_ramp_rate` to avoid motor shock

**Rationale:** Lower blower speed at low velocities conserves power and reduces snow throw when rover is maneuvering. Full blower power only needed at nominal clearing speed.

### Teleop Mode

Operator has direct control via `ToolCommand` from depot console:

```rust
// From types crate
pub struct ToolCommand {
    pub axis: f32,    // Not used for blower
    pub motor: f32,   // Blower power: 0.0 to 1.0
}
```

The `motor` field maps directly to blower duty cycle (0-100%).

**Control Flow:**
1. Operator adjusts blower slider in depot console (0-100%)
2. Console sends `Command::Tool(ToolCommand)` via WebSocket
3. bvrd receives command and updates `tool_command.motor`
4. Blower controller applies ramping and sends CAN duty command

### E-Stop Behavior

E-stop immediately shuts down the blower:

1. State machine transitions to `Mode::EStop`
2. Blower controller detects state change
3. Sends 0% duty command to blower VESC
4. Blower coasts to stop (no regenerative braking)

**Safety Interlock:** Blower cannot restart until:
- E-stop released (`Event::EStopRelease`)
- State transitions to `Idle` or valid operating mode
- New valid command received

## Configuration

Add the following section to `bvr/firmware/config/bvr.toml`:

```toml
[blower]
# Enable blower control subsystem
enabled = true

# VESC CAN ID for blower motor controller
can_id = 0x0C

# Maximum power in autonomous mode (percent, 0-100)
# Lower limit prevents excessive power draw and snow throw
max_power_autonomous = 50

# Power ramp rate (percent per second)
# Prevents sudden torque spikes that could damage gearbox
power_ramp_rate = 10

# Minimum velocity to enable blower in autonomous (m/s)
# Below this threshold, blower stays off
min_velocity_threshold = 0.1

# Maximum velocity for power scaling (m/s)
# Above this, blower runs at max_power_autonomous
max_velocity_for_scaling = 3.0
```

**Configuration Loading:**

The config structure mirrors the existing pattern:

```rust
#[derive(Debug, Deserialize)]
#[serde(default)]
struct BlowerFileConfig {
    enabled: bool,
    can_id: u8,
    max_power_autonomous: u8,
    power_ramp_rate: u8,
    min_velocity_threshold: f64,
    max_velocity_for_scaling: f64,
}

impl Default for BlowerFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            can_id: 0x0C,
            max_power_autonomous: 50,
            power_ramp_rate: 10,
            min_velocity_threshold: 0.1,
            max_velocity_for_scaling: 3.0,
        }
    }
}
```

## Implementation Guide

### Step 1: Create Blower Crate

Create `bvr/firmware/crates/blower/` with the following structure:

```
blower/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Cargo.toml:**
```toml
[package]
name = "blower"
version.workspace = true
edition.workspace = true

[dependencies]
can = { path = "../can" }
types = { path = "../types" }
tracing = "0.1"
```

**src/lib.rs:**
```rust
//! Blower control for snow clearing operations.

use can::vesc::Vesc;
use can::Bus;
use std::time::Instant;
use tracing::{debug, warn};
use types::Mode;

/// Blower controller configuration.
#[derive(Debug, Clone)]
pub struct BlowerConfig {
    /// VESC CAN ID for blower motor
    pub can_id: u8,
    /// Maximum power in autonomous mode (0.0 to 1.0)
    pub max_power_autonomous: f32,
    /// Power ramp rate (duty cycle units per second, 0.0 to 1.0)
    pub power_ramp_rate: f32,
    /// Minimum velocity to enable blower (m/s)
    pub min_velocity_threshold: f64,
    /// Maximum velocity for power scaling (m/s)
    pub max_velocity_for_scaling: f64,
}

impl Default for BlowerConfig {
    fn default() -> Self {
        Self {
            can_id: 0x0C,
            max_power_autonomous: 0.5,      // 50%
            power_ramp_rate: 0.1,            // 10% per second
            min_velocity_threshold: 0.1,     // 0.1 m/s
            max_velocity_for_scaling: 3.0,   // 3.0 m/s
        }
    }
}

/// Blower controller state machine.
pub struct BlowerController {
    config: BlowerConfig,
    vesc: Vesc,
    current_duty: f32,
    target_duty: f32,
    last_update: Option<Instant>,
}

impl BlowerController {
    /// Create a new blower controller.
    pub fn new(config: BlowerConfig) -> Self {
        Self {
            vesc: Vesc::new(config.can_id),
            config,
            current_duty: 0.0,
            target_duty: 0.0,
            last_update: None,
        }
    }

    /// Update blower power based on current mode and inputs.
    ///
    /// Returns the CAN frame to send, or None if no change needed.
    pub fn update(
        &mut self,
        mode: Mode,
        velocity_linear: f64,
        operator_power: f32,
    ) -> Option<can::Frame> {
        // Determine target duty based on mode
        self.target_duty = match mode {
            Mode::Autonomous => {
                self.compute_autonomous_duty(velocity_linear)
            }
            Mode::Teleop => {
                // Operator has full control (0-100%)
                operator_power.clamp(0.0, 1.0)
            }
            Mode::EStop | Mode::Disabled | Mode::Idle | Mode::Fault => {
                // Safety: blower off
                0.0
            }
        };

        // Apply power ramping
        let now = Instant::now();
        if let Some(last) = self.last_update {
            let dt = now.duration_since(last).as_secs_f32();
            let max_delta = self.config.power_ramp_rate * dt;

            let delta = self.target_duty - self.current_duty;
            if delta.abs() > max_delta {
                self.current_duty += delta.signum() * max_delta;
            } else {
                self.current_duty = self.target_duty;
            }
        } else {
            // First update: jump to target
            self.current_duty = self.target_duty;
        }
        self.last_update = Some(now);

        // Log power changes
        if (self.current_duty - self.target_duty).abs() > 0.01 {
            debug!(
                current = format!("{:.1}%", self.current_duty * 100.0),
                target = format!("{:.1}%", self.target_duty * 100.0),
                "Blower ramping"
            );
        }

        // Build and return CAN frame
        Some(self.vesc.build_duty_frame(self.current_duty))
    }

    /// Compute blower duty for autonomous mode based on velocity.
    fn compute_autonomous_duty(&self, velocity_linear: f64) -> f32 {
        if velocity_linear.abs() < self.config.min_velocity_threshold {
            return 0.0;
        }

        let velocity_ratio = (velocity_linear.abs() / self.config.max_velocity_for_scaling)
            .clamp(0.0, 1.0);

        (velocity_ratio as f32) * self.config.max_power_autonomous
    }

    /// Process status frame from blower VESC (if needed for telemetry).
    pub fn process_frame(&mut self, frame: &can::Frame) {
        self.vesc.process_frame(frame);
    }

    /// Get blower status for telemetry.
    pub fn status(&self) -> BlowerStatus {
        let vesc_state = self.vesc.state();
        BlowerStatus {
            duty: self.current_duty,
            target_duty: self.target_duty,
            erpm: vesc_state.status.erpm,
            current: vesc_state.status.current,
            temp_motor: vesc_state.status4.temp_motor,
            temp_fet: vesc_state.status4.temp_fet,
        }
    }

    /// Force blower off (for safety/e-stop).
    pub fn force_stop(&mut self) {
        self.target_duty = 0.0;
        self.current_duty = 0.0;
        self.last_update = None;
    }
}

/// Blower status for telemetry and debugging.
#[derive(Debug, Clone, Default)]
pub struct BlowerStatus {
    pub duty: f32,
    pub target_duty: f32,
    pub erpm: i32,
    pub current: f32,
    pub temp_motor: f32,
    pub temp_fet: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonomous_duty_stopped() {
        let config = BlowerConfig::default();
        let mut blower = BlowerController::new(config);

        // Velocity below threshold: blower off
        let frame = blower.update(Mode::Autonomous, 0.05, 0.0);
        assert!(frame.is_some());
        assert_eq!(blower.current_duty, 0.0);
    }

    #[test]
    fn test_autonomous_duty_scaling() {
        let config = BlowerConfig {
            max_power_autonomous: 0.5,
            max_velocity_for_scaling: 3.0,
            min_velocity_threshold: 0.1,
            ..Default::default()
        };
        let mut blower = BlowerController::new(config);

        // 1.5 m/s = 50% of max velocity = 25% duty (50% of max_power)
        blower.update(Mode::Autonomous, 1.5, 0.0);
        assert!((blower.current_duty - 0.25).abs() < 0.01);

        // 3.0 m/s = 100% of max velocity = 50% duty (max_power)
        blower.update(Mode::Autonomous, 3.0, 0.0);
        // After ramping
        std::thread::sleep(std::time::Duration::from_millis(100));
        blower.update(Mode::Autonomous, 3.0, 0.0);
    }

    #[test]
    fn test_teleop_mode() {
        let config = BlowerConfig::default();
        let mut blower = BlowerController::new(config);

        // Teleop: operator sets 75% power
        blower.update(Mode::Teleop, 0.0, 0.75);
        assert_eq!(blower.target_duty, 0.75);
    }

    #[test]
    fn test_estop_forces_off() {
        let config = BlowerConfig::default();
        let mut blower = BlowerController::new(config);

        // Start with blower running
        blower.current_duty = 0.5;
        blower.target_duty = 0.5;

        // E-stop
        blower.update(Mode::EStop, 0.0, 0.0);
        assert_eq!(blower.target_duty, 0.0);
    }

    #[test]
    fn test_power_clamping() {
        let config = BlowerConfig::default();
        let mut blower = BlowerController::new(config);

        // Operator sends > 100%
        blower.update(Mode::Teleop, 0.0, 1.5);
        assert_eq!(blower.target_duty, 1.0);

        // Operator sends negative
        blower.update(Mode::Teleop, 0.0, -0.5);
        assert_eq!(blower.target_duty, 0.0);
    }
}
```

### Step 2: Integrate into bvrd

**Modify `bvr/firmware/Cargo.toml`:**

```toml
[workspace.dependencies]
# ... existing dependencies ...
blower = { path = "crates/blower" }
```

**Modify `bvr/firmware/bins/bvrd/Cargo.toml`:**

```toml
[dependencies]
# ... existing dependencies ...
blower.workspace = true
```

**Modify `bvr/firmware/bins/bvrd/src/main.rs`:**

Add imports:
```rust
use blower::{BlowerConfig, BlowerController, BlowerStatus};
```

Add to `FileConfig`:
```rust
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FileConfig {
    // ... existing fields ...
    blower: BlowerFileConfig,
}
```

Initialize in `main()` after CAN interface:
```rust
// Initialize blower controller (if enabled)
let blower_enabled = file_config.blower.enabled;
let mut blower_controller = if blower_enabled {
    let blower_config = BlowerConfig {
        can_id: file_config.blower.can_id,
        max_power_autonomous: file_config.blower.max_power_autonomous as f32 / 100.0,
        power_ramp_rate: file_config.blower.power_ramp_rate as f32 / 100.0,
        min_velocity_threshold: file_config.blower.min_velocity_threshold,
        max_velocity_for_scaling: file_config.blower.max_velocity_for_scaling,
    };
    info!(can_id = blower_config.can_id, "Blower control enabled");
    Some(BlowerController::new(blower_config))
} else {
    info!("Blower control disabled");
    None
};
```

In control loop, after processing CAN frames:
```rust
// Update blower controller
if let Some(ref mut blower) = blower_controller {
    // Process incoming CAN status frames
    while let Ok(Some(frame)) = can_interface.recv() {
        // ... existing drivetrain processing ...
        blower.process_frame(&frame);
    }
}
```

In control loop, after computing motor outputs:
```rust
// Update and send blower commands
if let Some(ref mut blower) = blower_controller {
    let current_velocity = twist.linear;
    let operator_blower_power = tool_command.motor;

    if let Some(frame) = blower.update(current_mode, current_velocity, operator_blower_power) {
        if let Err(e) = can_interface.send(&frame) {
            warn!(?e, "Failed to send blower command");
        }
    }
}
```

Add blower status to telemetry:
```rust
// Extend telemetry structure
let blower_status = blower_controller.as_ref().map(|b| b.status());

// Log to recorder if active
if let Some(ref status) = blower_status {
    let _ = recorder.log_blower(status.duty, status.current, status.temp_motor);
}
```

### Step 3: Safety Checks

Add safety validation in the state machine transition handler:

```rust
// In main control loop, before sending blower commands
if let Some(ref mut blower) = blower_controller {
    // Safety check: force stop if not in valid driving mode
    if !state.state_machine.is_driving() && blower_status.duty > 0.0 {
        warn!("Blower running in non-driving mode, forcing stop");
        blower.force_stop();
    }

    // Safety check: force stop on e-stop
    if current_mode == Mode::EStop {
        blower.force_stop();
    }
}
```

## Testing

### Unit Tests

The `blower` crate includes comprehensive unit tests:

```bash
cd bvr/firmware
cargo test -p blower
```

**Test Coverage:**
- Autonomous mode duty scaling
- Teleop mode operator control
- E-stop safety shutdown
- Power clamping (0-100%)
- Velocity threshold behavior
- Power ramping (rate limiting)

### Integration Tests

Create `bvr/firmware/crates/integration-tests/tests/blower_integration.rs`:

```rust
//! Integration test for blower control with state machine.

use blower::{BlowerConfig, BlowerController};
use state::{Event, StateMachine};
use types::Mode;

#[test]
fn test_blower_follows_state_machine() {
    let mut sm = StateMachine::new();
    let mut blower = BlowerController::new(BlowerConfig::default());

    // Disabled: blower off
    blower.update(sm.mode(), 1.0, 0.5);
    assert_eq!(blower.status().target_duty, 0.0);

    // Enable to Idle: still off
    sm.transition(Event::Enable);
    blower.update(sm.mode(), 1.0, 0.5);
    assert_eq!(blower.status().target_duty, 0.0);

    // Teleop: operator control active
    sm.transition(Event::TeleopCommand);
    blower.update(sm.mode(), 1.0, 0.5);
    assert_eq!(blower.status().target_duty, 0.5);

    // E-stop: immediate shutdown
    sm.transition(Event::EStop);
    blower.update(sm.mode(), 1.0, 0.5);
    assert_eq!(blower.status().target_duty, 0.0);
}

#[test]
fn test_autonomous_velocity_coupling() {
    let mut sm = StateMachine::new();
    let config = BlowerConfig {
        max_power_autonomous: 0.5,
        max_velocity_for_scaling: 2.0,
        min_velocity_threshold: 0.1,
        ..Default::default()
    };
    let mut blower = BlowerController::new(config);

    sm.transition(Event::Enable);
    sm.transition(Event::AutonomousRequest);

    // Stopped: blower off
    blower.update(sm.mode(), 0.0, 0.0);
    assert_eq!(blower.status().target_duty, 0.0);

    // Half speed: 25% duty (0.5 * max_power_autonomous)
    blower.update(sm.mode(), 1.0, 0.0);
    assert!((blower.status().target_duty - 0.25).abs() < 0.01);

    // Full speed: 50% duty (max_power_autonomous)
    blower.update(sm.mode(), 2.0, 0.0);
    // Wait for ramping
    std::thread::sleep(std::time::Duration::from_millis(100));
    blower.update(sm.mode(), 2.0, 0.0);
    assert!(blower.status().target_duty > 0.4);
}
```

Run integration tests:
```bash
cargo test -p integration-tests --test blower_integration
```

### Manual Testing Procedures

#### 1. CAN Bus Verification

**Verify blower VESC is visible on CAN bus:**

```bash
# On rover (requires root)
sudo ip link set can0 type can bitrate 500000
sudo ip link set can0 up

# Monitor CAN traffic
candump can0
```

Expected output should show STATUS frames from blower VESC (ID 0x090C, 0x100C, 0x1B0C).

**Send test command:**

```bash
# Send 50% duty to VESC ID 0x0C
cansend can0 00000000C#0001869A

# Monitor response
candump can0 | grep 0C
```

#### 2. Teleop Mode Testing

1. Start bvrd with blower enabled:
   ```bash
   cargo run --bin bvrd -- --sim --log-dir ./logs --no-recording
   ```

2. Open depot console or use CLI tool

3. Enable teleop mode and adjust blower slider

4. Verify:
   - Blower power increases/decreases smoothly
   - Power ramps at configured rate (not instant jumps)
   - Zero power when slider at 0%
   - Maximum power when slider at 100%

5. Trigger e-stop and verify blower stops immediately

**Expected log output:**
```
INFO blower: Blower control enabled, can_id=12
DEBUG blower: Blower ramping, current=0.0%, target=50.0%
DEBUG blower: Blower ramping, current=10.0%, target=50.0%
INFO bvrd: E-Stop command received
DEBUG blower: Blower ramping, current=50.0%, target=0.0%
```

#### 3. Autonomous Mode Testing

1. Configure test waypoint in `bvr.toml`:
   ```toml
   [autonomous]
   goal = [5.0, 0.0]
   ```

2. Start bvrd in simulation mode

3. Transition to autonomous mode

4. Verify:
   - Blower remains off when rover is stationary
   - Blower power increases as rover accelerates
   - Blower power decreases as rover decelerates
   - Blower stops when rover reaches goal

**Expected behavior:**
- Velocity 0.0 m/s → Blower 0%
- Velocity 1.5 m/s → Blower ~25% (half of max speed)
- Velocity 3.0 m/s → Blower 50% (max_power_autonomous)

#### 4. Safety Interlock Testing

Test matrix:

| Scenario | Expected Blower State |
|----------|----------------------|
| Disabled mode | Off (0%) |
| Idle mode | Off (0%) |
| Teleop mode, operator 0% | Off (0%) |
| Teleop mode, operator 75% | On (75%) |
| Autonomous, stopped | Off (0%) |
| Autonomous, moving 1 m/s | On (~17%) |
| E-stop from any mode | Off (0%), immediate |
| E-stop release → Idle | Off (0%), requires new command |

## Troubleshooting

### Common Issues

#### Blower Not Responding

**Symptoms:** No blower activation in any mode, no CAN traffic from blower VESC.

**Diagnosis:**
```bash
# Check CAN interface is up
ip link show can0

# Monitor CAN bus for blower messages (ID 0x0C in lower byte)
candump can0 | grep 0C
```

**Solutions:**
1. Verify CAN bus is initialized and active
2. Check VESC CAN ID matches config (default 0x0C)
3. Verify VESC "Send Status over CAN" is enabled in VESC Tool
4. Check physical CAN wiring (CANH, CANL, GND, termination resistor)
5. Ensure VESC has power (check battery voltage via VESC Tool)

#### Blower Stuttering/Jerking

**Symptoms:** Blower motor stutters or jerks instead of smooth ramping.

**Causes:**
- Ramp rate too low (takes too long to reach target)
- CAN commands not being sent consistently
- VESC control loop instability

**Solutions:**
1. Increase `power_ramp_rate` in config (try 20% per second)
2. Check control loop is running at 100Hz (log timing in bvrd)
3. Verify VESC motor parameters (pole pairs, sensor type)
4. Check for CAN bus errors: `ip -details -statistics link show can0`

#### Blower Runs in Idle Mode

**Symptoms:** Blower motor runs when rover is in Idle/Disabled state.

**Diagnosis:**
```bash
# Check current mode in telemetry
# Should show Mode::Idle with blower duty = 0.0
```

**Solutions:**
1. This indicates a safety interlock failure
2. Check state machine integration in bvrd control loop
3. Verify `force_stop()` is called on mode transitions
4. Review logs for state machine events leading to the condition
5. Consider adding additional safety assertion before CAN send

#### Incorrect Power Scaling

**Symptoms:** Blower power doesn't match expected autonomous scaling.

**Diagnosis:**
- Check velocity measurement (from odometry/GPS)
- Review `max_velocity_for_scaling` config
- Log `compute_autonomous_duty()` inputs/outputs

**Solutions:**
1. Verify velocity is accurate (compare with wheel RPM)
2. Adjust `max_velocity_for_scaling` to match typical operating speed
3. Tune `max_power_autonomous` for optimal snow clearing

### Debug Logging

Enable detailed blower logging:

```bash
RUST_LOG=blower=debug,bvrd=info cargo run --bin bvrd -- --sim
```

**Key log messages:**
```
DEBUG blower: Blower ramping, current=X%, target=Y%
INFO blower: Blower control enabled, can_id=12
WARN bvrd: Blower running in non-driving mode, forcing stop
```

### CAN Bus Monitoring

Monitor blower CAN traffic in real-time:

```bash
# Filter for blower VESC (ID 0x0C in last byte)
candump can0 | grep --line-buffered 'C#'

# Decode duty cycle commands (Command ID 0x00)
candump can0,00000000C:1FFFFFFF
```

**Expected traffic:**
- Command frames from Jetson: `0000000C#XXXXXXXX` (SetDuty)
- Status frames from VESC: `090C#...` (STATUS1), `100C#...` (STATUS4), `1B0C#...` (STATUS5)

### Performance Metrics

Track blower performance in Rerun recordings:

```rust
// Add to recorder
recorder.log_blower(duty, current, temp_motor);
```

View in Rerun:
```bash
rerun /var/log/bvr/sessions/latest.rrd
```

Check for:
- Duty cycle ramping smoothness
- Current draw correlation with duty
- Motor temperature over time (should stay < 80°C)
- Power consumption vs velocity relationship

## References

### VESC Documentation
- [VESC CAN Protocol](https://github.com/vedderb/bldc/blob/master/comm/comm_can.c)
- [VESC Tool](https://vesc-project.com/vesc_tool)
- VESC ID Configuration: App Settings → General → CAN Settings

### Muni Codebase
- `bvr/firmware/crates/can/src/vesc.rs` - VESC protocol implementation
- `bvr/firmware/crates/state/src/lib.rs` - State machine and events
- `bvr/firmware/crates/control/src/lib.rs` - Rate limiting patterns
- `bvr/firmware/bins/bvrd/src/main.rs` - Main control loop integration

### Configuration
- `bvr/firmware/config/bvr.toml` - Runtime configuration
- `bvr/firmware/Cargo.toml` - Workspace dependencies

### Testing
- Unit tests: `cargo test -p blower`
- Integration tests: `cargo test -p integration-tests --test blower_integration`
- Manual testing: See "Manual Testing Procedures" section above
