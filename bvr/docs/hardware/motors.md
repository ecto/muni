# Motor Configuration & VESC Tuning

This document covers the drivetrain motor configuration, VESC settings, and troubleshooting.

## Hardware Specifications

| Parameter | Value |
|-----------|-------|
| Motor type | Hoverboard hub motors (BLDC) |
| Wheel diameter | 160mm |
| Poles | 32 (16 pole pairs) |
| Hall sensors | Yes (3-wire) |
| VESC model | VESC 75/300 R2 |
| Nominal voltage | 48V |

## VESC Tool Configuration

### Motor Detection

1. Connect to each VESC via USB
2. **Motor Settings → FOC → General**
   - Run motor detection wizard
   - Select **"Hall Sensors"** as sensor mode (not Sensorless)
3. **Motor Settings → FOC → Hall Sensors**
   - Click "Detect Hall Sensors" and run detection
   - Verify hall table is populated (values 1-6 should be non-255)
   - Set **Sensorless ERPM**: 4000
   - Set **Hall Interpolation ERPM**: 500-1000
4. Write motor configuration to each VESC

### CAN Status Messages (Required for Telemetry)

Battery voltage and other telemetry requires VESCs to broadcast status messages:

1. **App Settings → General**
2. Enable **"Send CAN Status"**
3. Set **"CAN Status Rate"**: 50 Hz
4. Enable status message types: STATUS1, STATUS4, STATUS5
5. Write app configuration

Without this, battery voltage will show 0V in the operator UI.

### CAN Bus Setup

| VESC | CAN ID | Position |
|------|--------|----------|
| Local | 0 | Front Left |
| CAN | 1 | Front Right |
| CAN | 2 | Rear Left |
| CAN | 3 | Rear Right |

## Control Modes

The firmware supports multiple motor control modes:

### Duty Cycle Control (Default)

Used for smooth low-speed operation with hall sensors. The firmware sends duty cycle commands (-1.0 to 1.0) directly to the motor.

**Advantages:**
- Smooth at all speeds, including very low RPM
- No PID hunting/oscillation
- Works well with hall sensors

**Trade-off:**
- Open-loop speed control (speed varies with load)

### RPM Control (Alternative)

Closed-loop speed control using VESC's internal PID. Can cause cogging/oscillation at low speeds with sensorless or poorly-tuned hall sensors.

## Troubleshooting

### Motor Cogging at Low Speed

**Symptoms:** Audible grinding/stuttering when starting or stopping slowly.

**Causes & Solutions:**

1. **Using RPM control mode** → Switch to duty cycle control (default in firmware)
2. **Hall sensors not configured** → Enable hall sensor mode in VESC Tool
3. **Hall interpolation too low** → Increase Hall Interpolation ERPM to 1000+
4. **Observer gain too high** → Reduce Observer Gain in FOC → Advanced

### Battery Voltage Shows 0V

**Cause:** VESCs not sending STATUS5 messages.

**Solution:** Enable CAN status messages in App Settings → General (see above).

### Motors Don't Spin

1. Check CAN bus connections and termination
2. Verify VESC IDs match firmware configuration
3. Check motor phase and hall sensor wiring
4. Verify VESCs are in FOC mode with hall sensors

## Speed & Power Limits

### Default Limits (firmware)

| Parameter | Normal Mode | Boost Mode |
|-----------|-------------|------------|
| Max duty | 50% | 95% |
| Approx. speed | ~3 m/s | ~6 m/s |

**Boost mode:** Hold L3 (left stick click) on gamepad or Shift on keyboard.

### Physical Limits

| Parameter | Value |
|-----------|-------|
| Max wheel RPM | ~650 RPM |
| Max linear speed | ~5.5 m/s |
| Max angular velocity | 2.5 rad/s |

## Firmware Parameters

Located in `bvr/firmware/bins/bvrd/src/main.rs`:

```rust
// Motor pole pairs (32 poles = 16 pairs)
#[arg(long, default_value = "16")]
pole_pairs: u8,

// Chassis: wheel diameter 160mm, track 550mm, wheelbase 550mm
let chassis = ChassisParams::new(0.160, 0.55, 0.55);
```

Located in `bvr/firmware/crates/control/src/lib.rs`:

```rust
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_linear: 5.0,      // m/s
            max_angular: 2.5,     // rad/s  
            max_accel: 3.0,       // m/s²
            max_decel: 8.0,       // m/s²
            max_wheel_rpm: 650.0, // RPM
        }
    }
}
```
