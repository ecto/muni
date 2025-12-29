# MCU Firmware

Embedded firmware for CAN-connected microcontrollers on the rover.

## Architecture

```
mcu/
├── crates/
│   ├── mcu-core/          # Shared: CAN protocol, watchdog, heartbeat
│   └── mcu-leds/          # Driver: LED animations and control
└── bins/
    ├── rp2350/            # Binary: Base rover LED controller (Pico 2 W)
    └── esp32s3/           # Binary: Attachment controller (ESP32-S3)
```

## Hardware

### LED Controller (rover-leds)

- **MCU**: Raspberry Pi Pico 2 W (RP2350)
- **LEDs**: WS2811 12V addressable strip (ALITOVE RGB, 4 addressable units)
- **CAN**: MCP2515 via SPI (planned)

#### Pinout

| Pico 2 Pin | Function           |
| ---------- | ------------------ |
| GP0        | LED data out       |
| GP16       | MCP2515 MISO (TBD) |
| GP17       | MCP2515 CS (TBD)   |
| GP18       | MCP2515 SCK (TBD)  |
| GP19       | MCP2515 MOSI (TBD) |
| GP20       | MCP2515 INT (TBD)  |

#### LED Wiring (WS2811 12V Strip)

```
LED Strip          Pico 2 W
─────────          ────────
+12V (red)    -->  External 12V supply
GND (white)   -->  GND (shared with 12V supply)
DIN (green)   -->  GP0 (data output)
```

**Important**: The data wire (green) must connect to the **input** end of the strip (marked with arrow or "DIN"). Data flows in one direction only.

### Attachment Controller (heltec-attachment)

- **MCU**: Heltec WiFi LoRa 32 V3 (ESP32-S3FN8)
- **Display**: Onboard 0.96" 128x64 OLED (SSD1306, I2C)
- **CAN**: ESP32-S3 native TWAI controller (no external chip needed)

This is a generic attachment controller for CAN-connected tool attachments. The OLED display shows status, CAN statistics, and debug info.

#### Pinout (Heltec V3)

| GPIO | Function   | Notes                       |
| ---- | ---------- | --------------------------- |
| 17   | OLED SDA   | I2C data (fixed)            |
| 18   | OLED SCL   | I2C clock (fixed)           |
| 21   | OLED RST   | Display reset (fixed)       |
| 36   | Vext       | OLED/peripheral power ctrl  |
| 35   | LED        | Onboard status LED          |
| 0    | PRG Button | Boot/user button            |
| 4    | TWAI RX    | CAN receive (configurable)  |
| 5    | TWAI TX    | CAN transmit (configurable) |

#### Features

- `oled` (default): Enable OLED display support

#### CAN Wiring

Connect a CAN transceiver (e.g., SN65HVD230 or MCP2551) to GPIO4 (RX) and GPIO5 (TX):

```
ESP32-S3         CAN Transceiver       CAN Bus
────────         ───────────────       ───────
GPIO4 (RX)  <--  RXD
GPIO5 (TX)  -->  TXD
3.3V        -->  VCC
GND         -->  GND
                 CANH            <-->  CAN_H
                 CANL            <-->  CAN_L
```

## CAN Protocol

### ID Ranges

| Range         | Purpose                                   |
| ------------- | ----------------------------------------- |
| 0x0A00-0x0AFF | Tool attachments (uses discovery)         |
| 0x0B00-0x0BFF | Base rover peripherals (fixed assignment) |

### LED Controller (0x0B00-0x0B01)

**Command (0x0B00, Jetson -> MCU):**

```
[0]: Mode (0x00=Off, 0x01=Solid, 0x02=Pulse, 0x03=Chase, 0x04=Flash, 0x10=StateLinked)
[1]: Red (0-255)
[2]: Green (0-255)
[3]: Blue (0-255)
[4]: Brightness (0-255)
[5-6]: Period (ms, little-endian, for animations)
[7]: Reserved
```

**Status (0x0B01, MCU -> Jetson):**

```
[0]: Status (0x00=OK, 0x01=Fault)
[1]: Uptime seconds (wraps at 255)
[2-7]: Reserved
```

## Building

### Prerequisites

```bash
# Install Rust target for RP2350
rustup target add thumbv8m.main-none-eabihf

# Install picotool (for flashing)
brew install picotool  # macOS
# or build from https://github.com/raspberrypi/picotool
```

### Build

```bash
cd mcu
cargo build --release -p rover-leds
```

### Flash

1. **Enter BOOTSEL mode**: Hold BOOTSEL button while plugging in USB
2. **Flash with picotool**:

```bash
picotool load target/thumbv8m.main-none-eabihf/release/rover-leds -t elf -f
picotool reboot
```

**Note**: Do NOT use `elf2uf2-rs` for RP2350 - it generates the wrong UF2 family ID (RP2040 instead of RP2350). Use `picotool` which correctly identifies the target.

## Debugging

Uses `defmt` for logging. With a debug probe (Picoprobe, etc.):

```bash
probe-rs run --chip RP235x target/thumbv8m.main-none-eabihf/release/rover-leds
```

Without a debug probe, add USB serial logging (see embassy-usb-logger).

## RP2350 / Pico 2 W Notes

### Key Differences from RP2040

1. **Different UF2 family ID**: RP2350 uses `rp2350-arm-s` (0xe48bff59), not RP2040's 0xe48bff56
2. **Boot2 required**: Must enable `boot2-w25q080` feature in embassy-rp for the flash bootloader
3. **Memory layout**: Requires special linker sections for boot blocks (`.start_block`, `.end_block`)
4. **Onboard LED**: On Pico 2 W, the LED is controlled via CYW43 WiFi chip (not GPIO25)

### Required Embassy Features

```toml
embassy-rp = { version = "0.9", features = [
    "rp235xa",              # RP2350 variant
    "time-driver",          # Async timers
    "boot2-w25q080",        # Flash bootloader (REQUIRED)
    "critical-section-impl", # Critical section implementation
    "binary-info",          # Picotool metadata
] }
```

### Required Linker Script Sections

The `memory.x` must include boot block sections for RP2350:

```ld
SECTIONS {
    .start_block : ALIGN(4) {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH
} INSERT AFTER .vector_table;

_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    .bi_entries : ALIGN(4) {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    .end_block : ALIGN(4) {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH
} INSERT AFTER .uninit;
```

### Build.rs Requirements

```rust
// Required linker args for RP2350
println!("cargo:rustc-link-arg-bins=--nmagic");
println!("cargo:rustc-link-arg-bins=-Tlink.x");
println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
```

### Troubleshooting

**Firmware doesn't run after flashing:**

- Verify UF2 family ID is `rp2350-arm-s` (check with `xxd file.uf2 | head`)
- Use `picotool` instead of `elf2uf2-rs`
- Check `picotool info -a` for boot diagnostics

**Picotool shows "Program Information: none":**

- Device is in BOOTSEL mode, not running application
- Check that boot2 feature is enabled
- Verify memory.x has correct boot block sections

**LEDs not working:**

- Verify data wire goes to DIN (input) end of strip
- Check WS2811 vs WS2812 timing (WS2811 uses 400kHz, WS2812 uses 800kHz)
- Try different color order (GRB vs RGB) - use `Grb` type in PioWs2812
- Confirm LED count matches addressable units (WS2811 12V has 3 LEDs per IC)

## LED Modes

| Mode  | Color  | Effect       | Use Case      |
| ----- | ------ | ------------ | ------------- |
| Off   | -      | -            | Disabled      |
| Solid | Green  | Constant     | Idle          |
| Pulse | Blue   | Breathing    | Teleop active |
| Pulse | Cyan   | Breathing    | Autonomous    |
| Flash | Red    | Strobe 200ms | E-Stop        |
| Flash | Orange | Strobe 500ms | Fault         |

## ESP32-S3 / Heltec LoRa 32 V3 Notes

### Prerequisites

```bash
# Install espup (ESP32 Rust toolchain manager)
cargo install espup
espup install

# Source the environment (add to shell profile for persistence)
source ~/export-esp.sh

# Install espflash for flashing
cargo install espflash
```

### Build

```bash
cd mcu/bins/heltec-attachment
cargo build --release
```

### Flash

```bash
# Flash and open serial monitor
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/heltec-attachment

# Or just flash
espflash flash target/xtensa-esp32s3-none-elf/release/heltec-attachment
```

The board will automatically reset and start running after flashing.

### Debugging

Serial output uses `defmt` via `esp-println`. The `--monitor` flag with espflash shows logs:

```bash
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/heltec-attachment
```

### Troubleshooting

**"Permission denied" on /dev/ttyUSB0 or /dev/ttyACM0:**

```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

**OLED not displaying:**

- Check Vext is enabled (GPIO36 low)
- Verify OLED reset sequence (GPIO21 low then high)
- Confirm I2C address is 0x3C

**CAN not working:**

- Verify CAN transceiver is connected and powered
- Check termination resistors (120Ω at each end of bus)
- Confirm baud rate matches other nodes (500kbps default)
