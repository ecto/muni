# Power System

BVR uses a 48V battery system with a 12V accessory rail.

## Power Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            48V Battery Pack                                  │
│                         (13S LiPo, 39-54.6V)                                │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │
                                    │ Main fuse (100A)
                                    │
         ┌──────────────────────────┼──────────────────────────┐
         │                          │                          │
         │                          │                          │
    ┌────┴────┐              ┌─────┴─────┐              ┌─────┴─────┐
    │  VESC   │──┐           │  VESC     │──┐          │   DCDC    │
    │  FL     │  │           │  FR       │  │          │  48V→12V  │
    └─────────┘  │           └───────────┘  │          │  (20A)    │
                 │                          │          └─────┬─────┘
    ┌─────────┐  │           ┌───────────┐  │                │
    │  VESC   │──┼───────────│  VESC     │──┘                │
    │  RL     │  │           │  RR       │                   │
    └─────────┘  │           └───────────┘           ┌───────┴───────┐
                 │                                    │               │
                 └──► Battery voltage               │               │
                      reported via CAN          ┌────┴────┐   ┌─────┴─────┐
                                                │ Jetson  │   │   Tool    │
                                                │ Orin NX │   │   MCU     │
                                                │ (12V)   │   │  (12V)    │
                                                └─────────┘   └───────────┘
```

## Voltage Monitoring

### From VESCs (Primary)

VESCs report battery voltage in STATUS5 messages over CAN:

```rust
// In can/src/vesc.rs
pub struct VescStatus5 {
    pub voltage_in: f32,  // Battery voltage
    // ...
}
```

This is the primary voltage measurement — no additional hardware needed.

### 12V Rail (Secondary)

Optional monitoring of the 12V accessory rail via Jetson's ADC or external INA226.

## Voltage Thresholds

For a 13S LiPo (48V nominal):

| State    | Voltage | Action                          |
| -------- | ------- | ------------------------------- |
| Full     | 54.6V   | 100% capacity                   |
| Nominal  | 48.0V   | Normal operation                |
| Low      | 42.0V   | Reduce max speed, warn operator |
| Critical | 39.0V   | Safe stop, shutdown sequence    |
| Cutoff   | 36.4V   | Hard cutoff (BMS)               |

```rust
// In hal/src/lib.rs
pub struct PowerMonitor {
    low_voltage_threshold: f64,      // 42.0V
    critical_voltage_threshold: f64, // 39.0V
}

impl PowerMonitor {
    pub fn is_low(&self, voltage: f64) -> bool {
        voltage < self.low_voltage_threshold
    }

    pub fn is_critical(&self, voltage: f64) -> bool {
        voltage < self.critical_voltage_threshold
    }
}
```

## Current Monitoring

### Per-Motor Current

VESCs report motor current in STATUS1:

```rust
pub struct VescStatus {
    pub current: f32,  // Motor current in amps
    // ...
}
```

### Total System Current (Optional)

For accurate State of Charge (SoC) estimation, add an INA226 current sensor on the main battery bus:

```
Battery+ ──────┬────────────────► To VESCs
               │
            ┌──┴──┐
            │INA226│ ◄── Shunt resistor
            └──┬──┘
               │ I2C
               ▼
            Jetson
```

**INA226 specs:**

- 16-bit ADC
- Programmable gain
- I2C interface
- Measures voltage and current simultaneously

## Temperature Monitoring

VESCs report temperatures in STATUS4:

| Reading    | Source                     | Warning | Critical |
| ---------- | -------------------------- | ------- | -------- |
| FET temp   | VESC                       | 80°C    | 100°C    |
| Motor temp | VESC (if sensor installed) | 80°C    | 120°C    |

```rust
pub struct VescStatus4 {
    pub temp_fet: f32,    // FET temperature °C
    pub temp_motor: f32,  // Motor temperature °C
    // ...
}
```

## Power Budget

| Component      | Typical  | Peak      |
| -------------- | -------- | --------- |
| Jetson Orin NX | 15W      | 25W       |
| 4× Hub Motors  | 100W     | 800W      |
| Tool (auger)   | 50W      | 200W      |
| Electronics    | 10W      | 10W       |
| **Total**      | **175W** | **1035W** |

At 48V nominal:

- Typical: 3.6A
- Peak: 21.6A

## Battery Selection

Recommended: **13S4P LiPo/Li-ion**

| Parameter       | Value                                              |
| --------------- | -------------------------------------------------- |
| Nominal voltage | 48.1V                                              |
| Capacity        | 10-20 Ah                                           |
| Discharge rate  | 30A continuous, 60A peak                           |
| BMS             | Required (over-discharge, over-current protection) |

## Charging

Use a 54.6V CC/CV charger with appropriate current rating.

**Safety:**

- Charge in fire-safe location
- Monitor during charging
- Don't charge immediately after heavy use (let cells cool)

## Low Power Behavior

### Low Voltage (< 42V)

1. Warning sent to operator
2. Max speed reduced to 50%
3. Tool power limited

### Critical Voltage (< 39V)

1. Critical warning to operator
2. Safe stop (coast to stop)
3. Motors disabled
4. Telemetry continues (for recovery)

### Shutdown Sequence

If voltage continues to drop:

1. Save state to persistent storage
2. Send final telemetry
3. Clean shutdown of Jetson

## Telemetry

Power status included in every telemetry message:

```rust
pub struct PowerStatus {
    pub battery_voltage: f64,  // From VESC
    pub system_current: f64,   // Sum of motor currents
}
```

Operator station displays:

- Voltage gauge
- Estimated range/time remaining
- Current draw
- Low battery warnings


