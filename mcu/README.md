# MCU Firmware

Embedded firmware for CAN-connected microcontrollers on the rover.

## Architecture

```
mcu/
├── crates/
│   ├── mcu-core/     # Shared: CAN protocol, watchdog, heartbeat
│   └── mcu-leds/     # Driver: WS2812 LED control via PIO
└── bins/
    └── rover-leds/   # Binary: Base rover LED controller (Pico 2)
```

## Hardware

### LED Controller (rover-leds)

- **MCU**: Raspberry Pi Pico 2 (RP2350)
- **CAN**: MCP2515 via SPI
- **LEDs**: WS2812/addressable 12V COB strip

#### Pinout

| Pico 2 Pin | Function      |
| ---------- | ------------- |
| GP16       | MCP2515 MISO  |
| GP17       | MCP2515 CS    |
| GP18       | MCP2515 SCK   |
| GP19       | MCP2515 MOSI  |
| GP20       | MCP2515 INT   |
| GP0        | LED data (5V) |

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

```bash
# Install target
rustup target add thumbv8m.main-none-eabihf

# Build
cd mcu
cargo build --release -p rover-leds

# Flash (with Pico in BOOTSEL mode)
elf2uf2-rs -d target/thumbv8m.main-none-eabihf/release/rover-leds
```

## Debugging

Uses `defmt` for logging via RTT:

```bash
# With probe-rs
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/rover-leds
```
