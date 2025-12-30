# Firmware

Quick reference for building and flashing firmware.

## Components

| Component | Platform | Location | Protocol |
|-----------|----------|----------|----------|
| bvrd | Jetson (aarch64) | `bvr/firmware/bins/bvrd/` | gRPC, CAN |
| Attachment | ESP32-S3 | `mcu/bins/esp32s3/` | SLCAN over USB |
| VESC | STM32 | External | CAN |

## bvrd (Jetson)

Main rover daemon handling motor control, sensors, and depot communication.

### Prerequisites

```bash
# Install Rust cross-compilation target
rustup target add aarch64-unknown-linux-gnu

# Install cross-linker (Ubuntu/Debian)
sudo apt install gcc-aarch64-linux-gnu
```

### Build

```bash
cd bvr/firmware
cargo build --release --target aarch64-unknown-linux-gnu
```

### Deploy

```bash
# Copy binary to Jetson
scp target/aarch64-unknown-linux-gnu/release/bvrd muni@<jetson-ip>:/opt/muni/bin/

# SSH to Jetson and restart
ssh muni@<jetson-ip>
sudo systemctl restart bvrd
sudo systemctl status bvrd
```

### Logs

```bash
# On Jetson
journalctl -u bvrd -f          # Follow logs
journalctl -u bvrd -n 100      # Last 100 lines
journalctl -u bvrd --since "1 hour ago"
```

---

## Attachment Firmware (ESP32-S3)

Runs on Heltec LoRa 32 V3 and compatible ESP32-S3 boards.

### Prerequisites

```bash
# Install ESP32 Rust toolchain
cargo install espup
espup install

# Add to shell profile
echo 'source ~/export-esp.sh' >> ~/.zshrc

# Install flasher
cargo install espflash
```

### Build

```bash
cd mcu/bins/esp32s3
source ~/export-esp.sh
cargo build --release
```

### Flash

```bash
espflash flash \
    --ignore-app-descriptor \
    --partition-table partitions.csv \
    --bootloader bootloader.bin \
    --min-chip-rev 0.0 \
    target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
```

### Monitor

```bash
# Using espflash
espflash monitor

# Using screen
screen /dev/cu.usbserial-0001 115200
# Exit: Ctrl+A, K, Y
```

### Bootloader Mode

If the board won't flash:

1. Unplug USB
2. Hold BOOT/PRG button
3. Plug in USB (keep holding)
4. Release after 2 seconds
5. Retry flash command

### Features

Enable/disable features in `Cargo.toml`:

```toml
[features]
default = ["status-bar", "addressable-leds"]
status-bar = []        # 5 GPIO status LEDs
addressable-leds = []  # WS2811/WS2812 strip
status-led = []        # Single onboard LED
```

---

## SLCAN Protocol

Attachments use SLCAN (CAN-over-serial) for communication with bvrd.

### Commands

| Command | Description |
|---------|-------------|
| `O\r` | Open CAN channel |
| `C\r` | Close CAN channel |
| `tIIILDD...\r` | Send standard frame |
| `V\r` | Query version |

### CAN Message IDs

| ID | Direction | Purpose |
|----|-----------|---------|
| 0x200 | A→H | Heartbeat (1Hz) |
| 0x201 | H→A | Identify request |
| 0x202 | A→H | Identity response |
| 0x203 | H→A | Command |
| 0x204 | A→H | Acknowledgment |

A = Attachment, H = Host (bvrd)

### Example Session

```bash
# Open channel (starts heartbeat)
echo -ne "O\r" > /dev/ttyUSB0

# Request identity
echo -ne "t2010\r" > /dev/ttyUSB0

# Set LED to red (cmd=0x10, r=255, g=0, b=0)
echo -ne "t20341000FF0000\r" > /dev/ttyUSB0

# Close channel
echo -ne "C\r" > /dev/ttyUSB0
```

---

## Text Commands (Debug)

When connected via serial terminal:

| Command | Description |
|---------|-------------|
| `led r,g,b` | Set LED color |
| `led off` | Turn off LEDs |
| `cycle` | Rainbow cycle mode |
| `state idle\|running\|error` | Set state |
| `rgb\|grb\|bgr` | Set color order |
| `ws2811\|ws2812` | Set timing |
| `help` | Show commands |

---

## Troubleshooting

### "No serial ports detected"

```bash
# Check if another process has the port
lsof /dev/cu.usbserial-*
killall screen
```

### "efuse blk rev" error

Use the included bootloader:
```bash
--bootloader bootloader.bin
```

### Binary won't boot

Check entry point:
```bash
xtensa-esp32s3-elf-readelf -h target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
# Entry should be ~0x40378xxx, not 0x0
```

If 0x0, check `.cargo/config.toml` has:
```toml
rustflags = ["-C", "link-arg=-Wl,-Tlinkall.x"]
```

### LEDs not responding

- WS2811 (12V) requires level shifter (3.3V → 5V)
- Check GND connection between ESP32 and LED strip
- Try different color order: `grb`, `rgb`, `bgr`
