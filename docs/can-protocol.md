# CAN Bus Protocol

BVR uses CAN bus for communication with motor controllers (VESCs) and tools.

## Physical Layer

| Parameter   | Value                            |
| ----------- | -------------------------------- |
| Bus speed   | 500 kbps                         |
| Termination | 120Ω at each end                 |
| Interface   | Jetson via USB-CAN or native CAN |

## VESC Protocol

VESCs use **extended CAN IDs** (29-bit) with the format:

```
CAN ID = (command_id << 8) | vesc_id
```

### Commands (Jetson → VESC)

| Command           | ID  | Payload             | Description                 |
| ----------------- | --- | ------------------- | --------------------------- |
| Set Duty          | 0   | i32 BE (×100,000)   | Duty cycle (-100% to +100%) |
| Set Current       | 1   | i32 BE (mA)         | Motor current               |
| Set Current Brake | 2   | i32 BE (mA)         | Braking current             |
| Set RPM           | 3   | i32 BE (ERPM)       | Electrical RPM              |
| Set Position      | 4   | i32 BE (×1,000,000) | Position in degrees         |

**Note**: ERPM = mechanical RPM × pole pairs

### Status Messages (VESC → Jetson)

VESCs broadcast status at ~50Hz.

#### STATUS1 (ID = 9)

| Byte | Field   | Format             |
| ---- | ------- | ------------------ |
| 0-3  | ERPM    | i32 BE             |
| 4-5  | Current | i16 BE (×10, amps) |
| 6-7  | Duty    | i16 BE (×1000)     |

#### STATUS4 (ID = 16)

| Byte | Field        | Format                |
| ---- | ------------ | --------------------- |
| 0-1  | Temp FET     | i16 BE (×10, °C)      |
| 2-3  | Temp Motor   | i16 BE (×10, °C)      |
| 4-5  | Current In   | i16 BE (×10, amps)    |
| 6-7  | PID Position | i16 BE (×50, degrees) |

#### STATUS5 (ID = 27)

| Byte | Field      | Format               |
| ---- | ---------- | -------------------- |
| 0-3  | Tachometer | i32 BE (ERPM counts) |
| 4-5  | Voltage In | i16 BE (×10, volts)  |

### Example

Set VESC ID 1 to 1000 mechanical RPM (15 pole pairs = 15000 ERPM):

```
CAN ID: 0x00000301 (extended)
Data:   00 00 3A 98 (15000 as i32 BE)
```

## Tool Protocol

Tools use a separate CAN ID range with **extended IDs**:

```
CAN ID = 0x0A00 | (slot << 4) | message_type
```

### Message Types

| Type      | ID  | Direction     | Description                 |
| --------- | --- | ------------- | --------------------------- |
| Discovery | 0x0 | Tool → Jetson | Periodic announcement (1Hz) |
| Command   | 0x1 | Jetson → Tool | Control commands            |
| Status    | 0x2 | Tool → Jetson | Status report (20Hz)        |

### Discovery Frame

Sent by tool every 1 second to announce presence.

| Byte | Field            | Description                          |
| ---- | ---------------- | ------------------------------------ |
| 0    | Tool Type        | 1=Auger, 2=Spreader, 3=Mower, 4=Plow |
| 1    | Protocol Version | Currently 1                          |
| 2-3  | Capabilities     | Bitfield (LE)                        |
| 4-7  | Serial Number    | Unique ID (LE)                       |

**Capabilities bitfield:**

| Bit | Capability                 |
| --- | -------------------------- |
| 0   | Axis control (raise/lower) |
| 1   | Motor control (spin)       |
| 2   | Position feedback          |
| 3   | Current feedback           |
| 4   | Temperature feedback       |

### Command Frame

Sent by Jetson to control tool.

| Byte | Field        | Format                   |
| ---- | ------------ | ------------------------ |
| 0    | Command Type | 0 = set values           |
| 1-2  | Axis Value   | i16 LE (-32768 to 32767) |
| 3-4  | Motor Value  | i16 LE (-32768 to 32767) |
| 5-7  | Reserved     |                          |

Values are normalized: -32768 = -1.0, 0 = 0.0, 32767 = +1.0

### Status Frame

Sent by tool at 20Hz.

| Byte | Field     | Format              |
| ---- | --------- | ------------------- |
| 0    | Position  | u8 (0-255 = 0-100%) |
| 1-2  | Motor RPM | u16 LE              |
| 3-4  | Current   | u16 LE (mA)         |
| 5    | Faults    | Bitfield            |
| 6-7  | Reserved  |                     |

**Fault bitfield:**

| Bit | Fault                 |
| --- | --------------------- |
| 0   | Over current          |
| 1   | Over temperature      |
| 2   | Position error        |
| 3   | Communication timeout |

## CAN ID Summary

| Range           | Usage                               |
| --------------- | ----------------------------------- |
| 0x001 - 0x0FF   | Reserved                            |
| 0x100 - 0x1FF   | VESC commands (0x100 + cmd<<8 + id) |
| 0x200 - 0x2FF   | VESC status                         |
| 0x0A00 - 0x0AFF | Tool messages                       |

## Wiring

```
                    CAN Bus
    ────────┬───────┬───────┬───────┬───────┬────────
            │       │       │       │       │
         ┌──┴──┐ ┌──┴──┐ ┌──┴──┐ ┌──┴──┐ ┌──┴──┐
         │VESC │ │VESC │ │VESC │ │VESC │ │Tool │
         │ 1   │ │ 2   │ │ 3   │ │ 4   │ │ MCU │
         └─────┘ └─────┘ └─────┘ └─────┘ └─────┘
    120Ω                                      120Ω
    term                                      term
```

Each VESC and tool connects CANH to CANH, CANL to CANL.
120Ω termination resistors at each physical end of the bus.
