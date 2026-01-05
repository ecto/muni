# CAN Bus Protocol Reference

This document provides a complete reference for CAN bus protocols used in the BVR rover, including VESC motor controllers, LED peripherals, and future attachments.

## CAN Bus Configuration

**Interface**: can0 (SocketCAN on Linux)
**Bitrate**: 500 kbps
**Frame Format**: Extended (29-bit IDs for peripherals), Standard (11-bit for VESCs)
**Timeout**: 10ms read/write

## VESC Motor Controller Protocol

### Overview

The BVR uses 4 VESC motor controllers for a four-wheel differential drive system. Communication uses the VESC CAN protocol with standard command and status frames.

**VESC Devices:**
- ID 0: Front Left (FL)
- ID 1: Front Right (FR)
- ID 2: Rear Left (RL)
- ID 3: Rear Right (RR)

### Command Frame Format

**CAN ID Structure (Extended Frame):**
```
Bits [8:15]: Command ID
Bits [0:7]:  VESC ID (0-3)
```

**Example:** SetDuty command to VESC 0
```
Frame ID = (0 << 8) | 0 = 0x0000 (but actually sent as Extended ID)
```

### Command Types

#### CMD_SET_DUTY (0x00)

Sets duty cycle (PWM) for motor control.

**Payload (4 bytes):**
```rust
duty_value: i32 = (duty * 100_000.0) as i32
// Duty range: -1.0 to 1.0
// Scaled range: -100,000 to 100,000
// Byte order: Big-endian
```

**Example:**
```rust
// 50% forward duty
let duty = 0.5;
let duty_value = (duty * 100_000.0) as i32; // 50,000
let bytes = duty_value.to_be_bytes(); // [0x00, 0x00, 0xC3, 0x50]

let frame_id = (CMD_SET_DUTY << 8) | vesc_id;
Frame::new(Id::Extended(frame_id), &bytes)
```

**When to use:**
- Low-level PWM control
- Simple proportional control
- Testing motor direction

#### CMD_SET_RPM (0x03)

Sets electrical RPM target.

**Payload (4 bytes):**
```rust
erpm: i32  // Electrical RPM (divide by pole pairs for mechanical)
// Byte order: Big-endian
```

**Conversion:**
```
Mechanical RPM = ERPM / (pole_pairs / 2)
For 14-pole motor: Mechanical RPM = ERPM / 7
```

**Example:**
```rust
// 1000 mechanical RPM on 14-pole motor
let mech_rpm = 1000;
let pole_pairs = 14;
let erpm = mech_rpm * (pole_pairs / 2); // 7000
let bytes = erpm.to_be_bytes();
```

**When to use:**
- Velocity control with known motor parameters
- Closed-loop speed regulation
- Coordinated wheel speeds

#### CMD_SET_CURRENT (0x01)

Sets current target in milliamps.

**Payload (4 bytes):**
```rust
current_ma: i32  // Current in milliamps
// Byte order: Big-endian
```

**Example:**
```rust
// 5A current limit
let current_a = 5.0;
let current_ma = (current_a * 1000.0) as i32; // 5000
let bytes = current_ma.to_be_bytes();
```

**When to use:**
- Torque control
- Current limiting for safety
- Precise motor control

### Status Frames

VESCs periodically broadcast status frames. Parse these to monitor motor health.

#### STATUS1 (CAN ID: VESC_ID + 0x09)

**Payload (8 bytes):**
```
[0-3]: ERPM (i32, big-endian)
[4-5]: Current * 10 (i16, big-endian, in 0.1A units)
[6-7]: Duty * 1000 (i16, big-endian, in 0.1% units)
```

**Parsing Example:**
```rust
fn parse_status1(data: &[u8]) -> VescStatus {
    if data.len() < 8 {
        return Err(CanError::InvalidFrame);
    }

    VescStatus {
        erpm: i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
        current: i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0,  // Amps
        duty: i16::from_be_bytes([data[6], data[7]]) as f32 / 1000.0,   // -1.0 to 1.0
    }
}
```

**Broadcast rate:** ~10 Hz (configurable in VESC)

#### STATUS4 (CAN ID: VESC_ID + 0x0C)

**Payload (8 bytes):**
```
[0-1]: Temp FET * 10 (i16, big-endian, °C * 10)
[2-3]: Temp Motor * 10 (i16, big-endian, °C * 10)
[4-5]: Current In * 10 (i16, big-endian, input current in 0.1A)
[6-7]: PID Position * 50 (i16, big-endian, if using position mode)
```

**Parsing Example:**
```rust
fn parse_status4(data: &[u8]) -> VescStatus {
    VescStatus {
        temp_fet: i16::from_be_bytes([data[0], data[1]]) as f32 / 10.0,   // °C
        temp_motor: i16::from_be_bytes([data[2], data[3]]) as f32 / 10.0, // °C
        current_in: i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0, // Amps
        pid_pos: i16::from_be_bytes([data[6], data[7]]) as f32 / 50.0,
    }
}
```

**Broadcast rate:** ~1 Hz

#### STATUS5 (CAN ID: VESC_ID + 0x0D)

**Payload (8 bytes):**
```
[0-3]: Tachometer (i32, big-endian, electrical revolutions * 6)
[4-5]: Voltage In * 10 (i16, big-endian, in 0.1V units)
```

**Parsing Example:**
```rust
fn parse_status5(data: &[u8]) -> VescStatus {
    VescStatus {
        tachometer: i32::from_be_bytes([data[0], data[1], data[2], data[3]]) / 6,
        voltage_in: i16::from_be_bytes([data[4], data[5]]) as f32 / 10.0,  // Volts
    }
}
```

**Use for:** Odometry, battery monitoring

### VESC Error Handling

**Common Errors:**
- **No response:** VESC powered off or disconnected
- **Timeout:** CAN bus congestion or VESC firmware issue
- **Invalid ID:** Frame from unknown VESC

**Recovery Strategy:**
```rust
// Track last valid status per VESC
let mut last_status: [Option<Instant>; 4] = [None; 4];

// In main loop
for vesc_id in 0..4 {
    if let Some(last) = last_status[vesc_id] {
        if last.elapsed() > Duration::from_millis(100) {
            warn!(vesc_id, "VESC heartbeat timeout");
            // Transition to safe state
            state_machine.handle(Event::Fault);
        }
    }
}
```

## LED Peripheral Protocol

### Overview

MCU-based LED controller (RP2350 or ESP32-S3) receives mode commands and controls WS2812 addressable LED strips for rover status indication.

**CAN ID Range:** 0x0B00-0x0BFF (extended IDs reserved for peripherals)

### Command Frames

#### LED_CMD (0x0B00)

Sent from Jetson to MCU to set LED mode.

**Payload (variable length):**
```
[0]: Mode ID
[1-4]: Mode-specific parameters (RGBA, period, etc.)
```

**Mode IDs:**
- `0x00`: Off
- `0x01`: Solid color (R, G, B, brightness)
- `0x02`: Pulse (R, G, B, period_ms as u16)
- `0x03`: Flash (R, G, B, period_ms as u16)
- `0x04`: Chase (R, G, B, speed)
- `0x10`: StateLinked (MCU auto-sets color based on rover mode)

**Example - State-Linked Mode:**
```rust
let frame_id = 0x0B00;
let data = [0x10]; // StateLinked mode ID
Frame::new(Id::Extended(frame_id), &data)
```

**Example - Solid Red:**
```rust
let frame_id = 0x0B00;
let data = [
    0x01,  // Solid mode
    255,   // Red
    0,     // Green
    0,     // Blue
    200,   // Brightness (0-255)
];
Frame::new(Id::Extended(frame_id), &data)
```

#### LED_STATUS (0x0B01)

Sent from MCU to Jetson with LED controller status.

**Payload (4 bytes):**
```
[0]: Current mode ID
[1]: Brightness level
[2]: Error flags
[3]: Strip count
```

**Broadcast rate:** ~1 Hz

### Rover State Color Mapping

When using StateLinked mode (0x10), MCU automatically sets colors:

| Rover Mode   | Color  | Pattern | Period | Notes                    |
|--------------|--------|---------|--------|--------------------------|
| Disabled     | Red    | Solid   | -      | Not initialized          |
| Idle         | Blue   | Solid   | -      | Ready, awaiting commands |
| Teleop       | Green  | Pulse   | 2s     | Human control active     |
| Autonomous   | Cyan   | Pulse   | 1.5s   | Autonomous navigation    |
| EStop        | Red    | Flash   | 200ms  | Emergency stop active    |
| Fault        | Orange | Flash   | 500ms  | Error state              |

**Implementation in firmware:**
```rust
pub fn update_leds(&mut self, mode: Mode) {
    let led_mode = match mode {
        Mode::Disabled => LedMode::solid(255, 0, 0, 200),    // Red
        Mode::Idle => LedMode::solid(0, 0, 255, 200),        // Blue
        Mode::Teleop => LedMode::pulse(0, 255, 0, 2000),     // Green pulse 2s
        Mode::Autonomous => LedMode::pulse(0, 255, 255, 1500), // Cyan pulse 1.5s
        Mode::EStop => LedMode::flash(255, 0, 0, 200),       // Red flash 200ms
        Mode::Fault => LedMode::flash(255, 165, 0, 500),     // Orange flash 500ms
    };
    self.send_led_command(led_mode)?;
}
```

## CAN Attachment Protocol

### Overview

Extensible protocol for tool attachments (brush, snowblower, etc.) connected via CAN bus.

**ID Range:** 0x200-0x2FF (16 attachment slots, 0x10 offset each)

### ID Scheme

Each attachment has 8 message types in a 0x10 range:

| Offset | Direction | Purpose                  |
|--------|-----------|--------------------------|
| +0x00  | A → H     | Heartbeat (periodic)     |
| +0x01  | H → A     | Identify request         |
| +0x02  | A → H     | Identity response        |
| +0x03  | H → A     | Command                  |
| +0x04  | A → H     | Acknowledgment           |
| +0x05  | A → H     | Sensor data              |
| +0x06  | H → A     | Configuration            |
| +0x07  | A → H     | Error report             |

**Legend:** A = Attachment, H = Host (Jetson)

**Example - Attachment 0 (base 0x200):**
- Heartbeat: 0x200
- Identify request: 0x201
- Identity response: 0x202
- Command: 0x203
- Ack: 0x204
- Sensor: 0x205
- Config: 0x206
- Error: 0x207

### Heartbeat Frame (Offset +0x00)

Periodic beacon from attachment to indicate presence.

**Payload (8 bytes):**
```
[0-1]: Firmware version (u16, e.g., 0x0100 = v1.0)
[2]: Device type (0x01 = brush, 0x02 = snowblower, etc.)
[3]: Status flags (bit 0 = ready, bit 1 = fault, bit 2 = calibrating)
[4-7]: Uptime in seconds (u32)
```

**Broadcast rate:** 1 Hz

**Parsing Example:**
```rust
fn parse_heartbeat(data: &[u8]) -> Attachment {
    Attachment {
        fw_version: u16::from_be_bytes([data[0], data[1]]),
        device_type: data[2],
        status: data[3],
        uptime: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
    }
}
```

### Command Frame (Offset +0x03)

Sent from host to attachment with commands.

**Payload (variable, max 8 bytes):**
```
[0]: Command ID
[1-7]: Command-specific parameters
```

**Common Command IDs:**
- `0x01`: Enable
- `0x02`: Disable
- `0x03`: Set speed (for motor-driven attachments)
- `0x04`: Calibrate
- `0x10-0xFF`: Device-specific commands

**Example - Set Brush Speed:**
```rust
let attachment_base = 0x200; // Attachment 0
let frame_id = attachment_base + 0x03; // Command offset

let speed_percent = 75; // 75% speed
let data = [
    0x03,          // Command: Set speed
    speed_percent, // Speed parameter
];
Frame::new(Id::Extended(frame_id), &data)
```

### Sensor Data Frame (Offset +0x05)

Periodic or event-driven sensor readings from attachment.

**Payload (variable, max 8 bytes):**
```
[0]: Sensor bitmask (which sensors are present)
[1-7]: Sensor values (device-specific encoding)
```

**Broadcast rate:** Device-specific (1-10 Hz typical)

## CAN Bus Debugging

### SocketCAN Tools

**View raw CAN traffic:**
```bash
candump can0
```

**Send test frame:**
```bash
cansend can0 003#0000C350  # SetDuty 50% to VESC 0
```

**Monitor specific ID range:**
```bash
candump can0,0B00:0BFF  # LED peripheral range
```

### SLCAN Bridge (ESP32-S3)

For debugging via USB serial, the ESP32-S3 MCU implements SLCAN protocol.

**Frame format:**
```
t<ID><LEN><DATA>\r       (standard frame)
T<ID><LEN><DATA>\r       (extended frame)
```

**Example:**
```
T00000B0010A      # Extended ID 0x0B00, length 1, data 0x0A
```

**Commands:**
- `O\r` - Open CAN bus
- `C\r` - Close CAN bus
- `S4\r` - Set bitrate to 500kbps
- `V\r` - Get version
- `N\r` - Get serial number

**Connect to SLCAN:**
```bash
slcand -o -c -s6 /dev/ttyUSB0 can0
ip link set up can0
```

## Error Recovery Best Practices

### Timeout Detection
```rust
const VESC_TIMEOUT: Duration = Duration::from_millis(100);
const LED_TIMEOUT: Duration = Duration::from_secs(5);

// Track last message per device
if last_msg.elapsed() > VESC_TIMEOUT {
    warn!("VESC timeout, transitioning to safe state");
    state_machine.handle(Event::Fault);
}
```

### Invalid Frame Handling
```rust
match Frame::try_from(raw_frame) {
    Ok(frame) => process_frame(frame),
    Err(e) => {
        error!(?e, ?raw_frame, "Invalid CAN frame");
        // Log but continue (don't crash)
    }
}
```

### Bounds Checking
```rust
// Always check data length before indexing
if data.len() < expected_len {
    warn!("Frame too short, expected {} got {}", expected_len, data.len());
    return Err(CanError::InvalidFrame);
}
```

### Big-Endian Conversion
```rust
// CORRECT: VESC uses big-endian
let value = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);

// INCORRECT: Little-endian (will produce wrong values)
let value = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
```

## References

- VESC CAN Protocol: https://github.com/vedderb/bldc/blob/master/documentation/CAN_protocol.md
- SocketCAN: https://www.kernel.org/doc/html/latest/networking/can.html
- SLCAN: https://www.can232.com/docs/canusb_manual.pdf
