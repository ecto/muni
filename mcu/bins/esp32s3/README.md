# ESP32-S3 MCU Firmware

Generic attachment controller firmware for ESP32-S3 based boards (Heltec LoRa 32 V3, etc.).

## Features

- **OLED Display**: 128x64 SSD1306 with animated UI
  - Splash screen with Muni branding
  - Status indicator with pulsing animation
  - State display (IDLE, RUNNING, ERROR)
  - Cycling info bar (activity, CAN health, device info)
- **Dual Protocol Serial Interface**:
  - Text commands for debugging
  - SLCAN (CAN-over-serial) for bvrd integration
- **Status LEDs** (feature: `status-bar`):
  - 5 GPIO LEDs: Red (error), Yellow (warn), Green (idle), Blue (running), White (heartbeat)
  - Configurable patterns per state
- **Addressable LED Driver** (feature: `addressable-leds`):
  - RMT-based for precise timing
  - WS2811 (400kHz) and WS2812/SK6812 (800kHz) support
  - RGB/GRB/BGR color order options
  - RGB cycle mode for testing

## Supported Hardware

- **Heltec WiFi LoRa 32 V3 / V3.2** (ESP32-S3FN8)
- Other ESP32-S3 boards with minor pin adjustments

### Heltec V3 Pinout

| GPIO | Function       | Notes                                              |
| ---- | -------------- | -------------------------------------------------- |
| 17   | OLED SDA       | I2C data                                           |
| 18   | OLED SCL       | I2C clock                                          |
| 21   | OLED RST       | Display reset                                      |
| 36   | Vext           | OLED power control (LOW = on)                      |
| 35   | LED            | Onboard status LED                                 |
| 0    | PRG Button     | Boot/user button                                   |
| 4    | LED Strip Data | WS2811/WS2812 (needs level shifter for 12V strips) |
| 43   | UART TX        | Serial output (CP2102)                             |
| 44   | UART RX        | Serial input (CP2102)                              |

### Status Bar LEDs (feature: `status-bar`)

| GPIO | Color  | Meaning           |
| ---- | ------ | ----------------- |
| 19   | Red    | Error/fault       |
| 20   | Yellow | Warning/attention |
| 26   | Green  | Idle/ok           |
| 48   | Blue   | Running/active    |
| 47   | White  | Heartbeat         |

Wire each LED: `GPIO → 330Ω resistor → LED anode (+) → LED cathode (-) → GND`

## Communication Protocols

The firmware supports two protocols over USB serial:

### 1. Text Commands (Human/Debug)

Connect via screen or any serial terminal at 115200 baud:

```bash
screen /dev/cu.usbserial-0001 115200
```

| Command         | Description                    |
| --------------- | ------------------------------ |
| `led r,g,b`     | Set LED color (0-255 each)     |
| `led off`       | Turn off LEDs                  |
| `cycle`         | Toggle RGB rainbow cycle mode  |
| `state idle`    | Set display state to IDLE      |
| `state running` | Set display state to RUNNING   |
| `state error`   | Set display state to ERROR     |
| `rgb/grb/bgr`   | Set LED color order            |
| `ws2811/ws2812` | Set LED timing (400kHz/800kHz) |
| `help`          | Show available commands        |

Exit screen with `Ctrl+A` then `K` then `Y`.

### 2. SLCAN (CAN-over-Serial for bvrd)

The firmware implements SLCAN protocol for integration with bvrd. This allows unified CAN messaging whether using USB serial or hardware CAN.

**SLCAN Commands:**

| Command           | Description                         |
| ----------------- | ----------------------------------- |
| `O`               | Open CAN channel (starts heartbeat) |
| `C`               | Close CAN channel                   |
| `tIIILDD...`      | Send standard frame (11-bit ID)     |
| `TIIIIIIIILDD...` | Send extended frame (29-bit ID)     |
| `V`               | Query firmware version              |
| `N`               | Query serial number                 |

**Example:**

```bash
# Open channel
echo -ne "O\r" > /dev/ttyUSB0

# Query identity (ID 0x201, 0 data bytes)
echo -ne "t2010\r" > /dev/ttyUSB0

# Set LED to red (ID 0x203, cmd=0x10, r=255, g=0, b=0)
echo -ne "t20341000FF00\r" > /dev/ttyUSB0
```

## CAN Protocol (Message IDs)

Attachments use CAN IDs in the `0x200-0x2FF` range. Each attachment slot gets 16 IDs.

### Attachment Slot 0 (Base ID 0x200)

| ID    | Dir | Name      | Description                             |
| ----- | --- | --------- | --------------------------------------- |
| 0x200 | A→H | Heartbeat | Periodic status (1Hz when channel open) |
| 0x201 | H→A | Identify  | Request attachment info                 |
| 0x202 | A→H | Identity  | Response with type/version/caps         |
| 0x203 | H→A | Command   | Control command                         |
| 0x204 | A→H | Ack       | Command acknowledgment                  |
| 0x205 | A→H | Sensor    | Sensor data broadcast                   |
| 0x206 | H→A | Config    | Configuration update                    |
| 0x207 | A→H | Error     | Error/fault report                      |

A = Attachment, H = Host (bvrd/Jetson)

### Heartbeat Frame (0x200)

Sent every ~1 second when SLCAN channel is open:

```
Byte 0: State (0=Idle, 1=Running, 2=Error, 3=Warning)
Byte 1: Uptime (seconds, low byte)
Byte 2: Uptime (seconds, high byte)
Byte 3: Flags (reserved)
```

### Identity Frame (0x202)

Response to Identify request:

```
Byte 0: Attachment type (0x02 = LedStrip)
Byte 1: Hardware revision
Byte 2: Software version major
Byte 3: Software version minor
Byte 4: Capabilities (0x01=LED, 0x02=Sensor, 0x04=Actuator)
```

### Command Frame (0x203)

| Cmd  | Name      | Args                     |
| ---- | --------- | ------------------------ |
| 0x00 | Nop       | (none)                   |
| 0x01 | Enable    | (none)                   |
| 0x02 | Disable   | (none)                   |
| 0x03 | SetState  | arg0=state               |
| 0x10 | SetLed    | arg0=R, arg1=G, arg2=B   |
| 0x11 | LedCycle  | arg0=0/1 (off/on)        |
| 0x12 | LedTiming | arg0=0/1 (SK68/WS2811)   |
| 0x13 | LedOrder  | arg0=0/1/2 (RGB/GRB/BGR) |

### Ack Frame (0x204)

```
Byte 0: Command that was acknowledged
Byte 1: Result (0x00=Ok, 0x01=UnknownCmd, 0x02=InvalidArgs)
```

## Full-Stack Integration

```
┌─────────────┐     WebSocket      ┌─────────────┐
│  Operator   │◄──────────────────►│   Depot     │
│    App      │                    │  (Cloud)    │
└─────────────┘                    └─────────────┘
                                          ▲
                                          │ gRPC/HTTP
                                          ▼
                                   ┌─────────────┐
                                   │    bvrd     │
                                   │  (Jetson)   │
                                   └─────────────┘
                                          ▲
                                          │ USB Serial (SLCAN)
                                          ▼
                                   ┌─────────────┐
                                   │ Attachment  │
                                   │   (ESP32)   │
                                   └─────────────┘
```

1. **Discovery**: bvrd detects USB device, sends `O` then `t2010` (Identify)
2. **Registration**: Attachment responds with Identity frame (0x202)
3. **Heartbeat**: Attachment sends periodic heartbeat (0x200)
4. **Control**: Operator sends command through Depot → bvrd → SLCAN → Attachment
5. **Status**: Attachment heartbeat flows back to Operator via Depot

## Prerequisites

```bash
# Install ESP32 Rust toolchain
cargo install espup
espup install

# Add to your shell profile (~/.zshrc or ~/.bashrc)
source ~/export-esp.sh

# Install espflash
cargo install espflash
```

## Building

```bash
cd mcu/bins/esp32s3
source ~/export-esp.sh
cargo build --release
```

## Flashing

**Important**: Use the included `bootloader.bin` to avoid chip revision compatibility issues.

```bash
espflash flash \
    --ignore-app-descriptor \
    --partition-table partitions.csv \
    --bootloader bootloader.bin \
    --min-chip-rev 0.0 \
    target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
```

To monitor serial output:

```bash
espflash monitor
```

Press `CTRL+R` to reset, `CTRL+C` to exit.

## WS2811 LED Strip Setup

The ESP32 outputs 3.3V logic, but 12V WS2811 strips require 5V logic levels.

### Required Hardware

**Level Shifter**: Use a bidirectional logic level converter (3.3V to 5V):

- 4-channel I2C Logic Level Converter module (recommended)
- 74HCT125
- SN74AHCT125N
- TXB0104

### Wiring with Level Shifter

```
ESP32 (3.3V side)          Level Shifter          LED Strip (5V side)
─────────────────          ─────────────          ──────────────────
3.3V  ──────────────────── LV
GND   ──────────────────── GND ─────────────────── GND
GPIO4 ──────────────────── LV1 ──── HV1 ────────── DIN
                           HV ──────────────────── 5V (from LED power)
```

| From            | To                    |
| --------------- | --------------------- |
| ESP32 3.3V      | LV pin                |
| ESP32 GND       | GND pin (either side) |
| ESP32 GPIO4     | LV1                   |
| 5V power supply | HV pin                |
| HV1             | LED strip DIN         |

### Why Level Shifting is Required

- WS2811 data input threshold: ~0.7 \* VDD = 3.5V (for 5V power)
- ESP32 GPIO output: 3.3V
- 3.3V < 3.5V = unreliable detection

Without a level shifter, only the first LED may respond, and colors will be incorrect.

## Critical Configuration

### .cargo/config.toml

The linker script is **required** for proper linking:

```toml
[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"
rustflags = [
  "-C", "link-arg=-Wl,-Tlinkall.x",  # REQUIRED: Links esp-hal runtime
  "-C", "link-arg=-nostartfiles",
]

[build]
target = "xtensa-esp32s3-none-elf"

[unstable]
build-std = ["alloc", "core"]
```

Without `-Tlinkall.x`, the binary will have entry point 0x0 and fail to boot.

### Bootloader Compatibility

ESP-IDF v5.5.x bootloaders have a strict "efuse block revision" check that fails on ESP32-S3 v0.2 chips. The error looks like:

```
E (95) boot_comm: Image requires efuse blk rev >= v20.50, but chip is v1.3
E (101) boot: Factory app partition is not bootable
```

**Solution**: Use the included `bootloader.bin` (ESP-IDF v5.1-beta1) which doesn't have this check.

To update the bootloader:

```bash
curl -sL "https://github.com/esp-rs/espflash/raw/v3.0.0/espflash/resources/bootloaders/esp32s3-bootloader.bin" -o bootloader.bin
```

## Entering Bootloader Mode

If the board is stuck or unresponsive:

1. **Unplug USB**
2. **Hold BOOT/PRG button**
3. **Plug in USB** (keep holding)
4. **Release after 2 seconds**

This forces download mode for flashing.

## Troubleshooting

### Binary is empty / entry point 0x0

- Check `.cargo/config.toml` has `-Wl,-Tlinkall.x`
- Verify `build-std = ["alloc", "core"]`
- Clean rebuild: `cargo clean && cargo build --release`

Verify with:

```bash
xtensa-esp32s3-elf-readelf -h target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
# Entry point should be ~0x40378xxx, not 0x0
```

### "abort() was called" on boot

- Usually caused by missing linker script
- Check entry point as above

### "efuse blk rev" error

- Use older bootloader: `--bootloader bootloader.bin`
- ESP-IDF v5.5.x has incompatible checks for v0.2 chips

### OLED not displaying

- Vext must be LOW (GPIO36) to power display
- Reset sequence: GPIO21 LOW -> 50ms -> HIGH -> 50ms
- I2C address: 0x3C (some boards use 0x3D)
- Pins: SDA=GPIO17, SCL=GPIO18

### UART read blocking main loop

- Use `embedded_io::ReadReady` trait to check for data before reading
- `uart.read()` is blocking; always check `read_ready()` first

### Serial port not found

```bash
# Check if another process is using it
lsof /dev/cu.usbserial-*

# Kill conflicting processes
killall screen
```

### Permission denied (Linux)

```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### LEDs not responding (WS2811)

- **Level shifter required**: 3.3V GPIO cannot reliably drive 5V WS2811
- Check wiring: GND must be shared between ESP32 and LED power supply
- Verify data pin is connected to strip INPUT (marked DIN or with arrow)
- Try different GPIO if issues persist

## Files

| File                  | Purpose                                    |
| --------------------- | ------------------------------------------ |
| `bootloader.bin`      | ESP-IDF v5.1-beta1 bootloader (compatible) |
| `partitions.csv`      | Partition table (nvs, phy_init, factory)   |
| `.cargo/config.toml`  | Cargo build configuration with linker args |
| `rust-toolchain.toml` | ESP Rust toolchain specification           |

## Lessons Learned

1. **Linker script is mandatory**: Without `-Wl,-Tlinkall.x`, the esp-hal runtime isn't linked and the binary is empty.

2. **Bootloader version matters**: ESP-IDF v5.5.x introduced strict efuse block revision checks that reject valid binaries on older ESP32-S3 chips (v0.2).

3. **Vext controls OLED power**: The Heltec boards use GPIO36 (active low) to control power to the OLED and other peripherals. Must be set LOW before display works.

4. **OLED reset sequence is required**: GPIO21 must be toggled (low -> high) with delays for the SSD1306 to initialize properly.

5. **Use `core::mem::forget` for GPIO pins**: If GPIO pins go out of scope, they're reconfigured. Use `forget()` to keep them alive.

6. **UART read is blocking**: The `embedded_io::Read` trait's `read()` method blocks. Use `ReadReady::read_ready()` first to avoid blocking the main loop.

7. **WS2811 vs WS2812 timing**: WS2811 (often 12V strips) uses 400kHz timing, WS2812 (5V strips) uses 800kHz. The timing is completely different.

8. **Level shifting for LED strips**: 12V WS2811 strips need 5V logic. ESP32's 3.3V output is below the detection threshold. A bidirectional level shifter is required.

9. **heapless collections can panic**: Using `.collect()` on iterators with `heapless::Vec` will panic if the iterator has more elements than the Vec capacity. Use manual iteration instead.
