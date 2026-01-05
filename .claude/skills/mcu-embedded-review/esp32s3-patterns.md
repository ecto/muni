# ESP32-S3 (Heltec) Embedded Patterns

This document provides detailed patterns and best practices for ESP32-S3 (Heltec LoRa module) firmware development with esp-hal and esp-alloc.

## Platform Overview

**MCU**: ESP32-S3 (Xtensa dual-core, 240MHz)
**Memory**: 512 KB SRAM, 8 MB Flash
**Runtime**: Blocking (no async)
**Edition**: Rust 2024
**Target**: `xtensa-esp32s3-none-elf`

**Heltec Module Features:**
- WiFi + BLE
- LoRa SX1262 (868/915 MHz)
- OLED display (128x64, SSD1306)
- USB-C (UART bridge)

## Entry Point (Blocking Main)

```rust
#![no_std]
#![no_main]

use esp_hal::prelude::*;
use esp_backtrace as _;
use esp_println::println;

#[main]
fn main() -> ! {
    // Initialize heap allocator (optional but recommended)
    esp_alloc::heap_allocator!();

    // Initialize peripherals
    let peripherals = Peripherals::take();
    let mut delay = Delay::new();

    println!("ESP32-S3 initialized");

    // Main loop (must never return)
    loop {
        println!("Loop iteration");
        delay.delay_millis(1000);
    }
}
```

**Key Points:**
- `#[main]` entry point (not Embassy)
- Function must return `!` (never returns)
- Infinite loop required
- `esp_backtrace` for panic messages

## Heap Allocation

### Enabling Heap

```rust
use esp_alloc as _;

#[main]
fn main() -> ! {
    // CRITICAL: Must be first line in main
    esp_alloc::heap_allocator!();

    // Now can use Vec, String, etc.
    let mut buffer = Vec::with_capacity(256);
    let message = String::from("Hello");

    loop {
        // Heap operations allowed
    }
}
```

**Heap Guidelines:**
- Initialize with `esp_alloc::heap_allocator!()` first
- Use sparingly (still embedded system)
- Pre-allocate with `with_capacity()` when possible
- Avoid unbounded growth (check memory usage)

### Monitoring Heap Usage

```rust
use esp_alloc::HEAP;

fn check_heap() {
    let free = HEAP.free();
    let used = HEAP.used();
    println!("Heap: {} free, {} used", free, used);
}
```

## Polling Loop Patterns

### Non-Blocking UART Read

```rust
use esp_hal::uart::{Uart, UartRx, UartTx};

fn main() -> ! {
    let mut uart = Uart::new(/* ... */);
    let mut buffer = Vec::with_capacity(256);

    loop {
        // Non-blocking check
        if let nb::Result::Ok(byte) = uart.read() {
            buffer.push(byte);

            // Process on newline
            if byte == b'\n' {
                process_command(&buffer);
                buffer.clear();
            }
        }

        // Other work
        update_leds();

        // Small delay (avoid busy-wait)
        delay.delay_millis(1);
    }
}
```

**Pattern**: Use `nb::Result` for non-blocking I/O.

### Timer-Based Polling

```rust
use esp_hal::timer::systimer::SystemTimer;

fn main() -> ! {
    let systimer = SystemTimer::new();
    let mut last_update = 0u64;

    loop {
        let now = systimer.now();

        // Update at 10Hz
        if now - last_update >= 100_000 {  // 100ms in µs
            update_sensors();
            last_update = now;
        }

        // Other work
        handle_uart();

        // Small delay
        delay.delay_millis(1);
    }
}
```

## RMT for WS2812 LEDs

### RMT Channel Configuration

```rust
use esp_hal::rmt::{Rmt, TxChannel, TxChannelConfig, PulseCode};
use esp_hal::gpio::OutputPin;

fn init_rmt(rmt: RMT, pin: GPIO48) -> TxChannel<'static, 0> {
    let rmt = Rmt::new(rmt, 80u32.MHz()).unwrap();

    let tx_config = TxChannelConfig {
        clk_divider: 1,
        idle_output_level: false,
        carrier_modulation: false,
        idle_output: true,
    };

    let channel = rmt.channel0.configure(pin, tx_config).unwrap();
    channel
}
```

### WS2812 Timing (RMT)

**Clock**: 80 MHz (12.5ns per tick)

**WS2812 Protocol:**
- Bit 0: 400ns high (32 ticks), 850ns low (68 ticks)
- Bit 1: 800ns high (64 ticks), 450ns low (36 ticks)
- Reset: >50µs low

```rust
fn ws2812_bit(channel: &mut TxChannel, bit: bool) {
    let (high, low) = if bit {
        (64, 36)  // Bit 1: 800ns high, 450ns low
    } else {
        (32, 68)  // Bit 0: 400ns high, 850ns low
    };

    let pulse = PulseCode {
        level1: true,
        length1: high,
        level2: false,
        length2: low,
    };

    channel.transmit(&[pulse]).unwrap();
}

fn send_pixel(channel: &mut TxChannel, r: u8, g: u8, b: u8) {
    // GRB byte order for WS2812
    let grb = ((g as u32) << 16) | ((r as u32) << 8) | (b as u32);

    // Send 24 bits (MSB first)
    for bit in (0..24).rev() {
        let is_one = (grb >> bit) & 1 == 1;
        ws2812_bit(channel, is_one);
    }
}
```

**RMT Limits:**
- 8 RMT channels total
- 64 pulse codes max per transmission
- Use DMA for large sequences (64+ LEDs)

## OLED Display (SSD1306)

### I2C Setup

```rust
use esp_hal::i2c::I2c;
use esp_hal::gpio::{InputOutputPin, OutputOpenDrain};
use ssd1306::{Ssd1306, mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface};

fn init_display(
    i2c: I2C0,
    sda: GPIO17,
    scl: GPIO18,
) -> Ssd1306<I2CInterface<I2C0>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>> {
    // I2C at 400kHz
    let i2c = I2c::new(i2c, sda, scl, 400u32.kHz()).unwrap();

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.clear_buffer();
    display.flush().unwrap();

    display
}
```

### Drawing Text

```rust
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

fn draw_status(display: &mut Ssd1306<...>, status: &str) {
    display.clear_buffer();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(status, Point::new(0, 10), style)
        .draw(display)
        .unwrap();

    display.flush().unwrap();
}
```

**Display Best Practices:**
- Update infrequently (I2C is slow ~10ms per frame)
- Use buffered mode (update once with `flush()`)
- Avoid blocking in main loop (offload to timer)

## SLCAN Protocol Implementation

### Frame Parsing

```rust
enum SlcanCommand {
    Open,
    Close,
    SetBitrate(u8),
    Version,
    SerialNumber,
    StandardFrame { id: u16, data: Vec<u8> },
    ExtendedFrame { id: u32, data: Vec<u8> },
}

fn parse_slcan(buf: &[u8]) -> Result<SlcanCommand, SlcanError> {
    if buf.is_empty() || buf[buf.len() - 1] != b'\r' {
        return Err(SlcanError::MissingCR);
    }

    match buf[0] {
        b'O' => Ok(SlcanCommand::Open),
        b'C' => Ok(SlcanCommand::Close),
        b'S' => {
            let bitrate = buf[1] - b'0';
            if bitrate > 8 {
                return Err(SlcanError::InvalidBitrate);
            }
            Ok(SlcanCommand::SetBitrate(bitrate))
        }
        b'V' => Ok(SlcanCommand::Version),
        b'N' => Ok(SlcanCommand::SerialNumber),
        b't' => parse_standard_frame(&buf[1..]),
        b'T' => parse_extended_frame(&buf[1..]),
        _ => Err(SlcanError::UnknownCommand),
    }
}
```

### Standard Frame Format

**Format**: `t<ID><LEN><DATA>\r`

**Example**: `t10348AABBCCDD\r`
- ID: 0x103 (3 hex digits)
- Length: 4 bytes
- Data: 0xAA, 0xBB, 0xCC, 0xDD

```rust
fn parse_standard_frame(buf: &[u8]) -> Result<SlcanCommand, SlcanError> {
    // Expect: IIILDD...\r
    if buf.len() < 5 {  // Min: ID(3) + LEN(1) + CR(1)
        return Err(SlcanError::FrameTooShort);
    }

    // Parse ID (3 hex digits)
    let id_str = core::str::from_utf8(&buf[0..3])?;
    let id = u16::from_str_radix(id_str, 16)?;

    if id > 0x7FF {
        return Err(SlcanError::InvalidId);
    }

    // Parse length (1 hex digit)
    let len = (buf[3] as char).to_digit(16).ok_or(SlcanError::InvalidLength)?;

    if len > 8 {
        return Err(SlcanError::InvalidLength);
    }

    // Parse data (2 hex digits per byte)
    let mut data = Vec::with_capacity(len as usize);
    for i in 0..len {
        let offset = 4 + (i * 2) as usize;
        let byte_str = core::str::from_utf8(&buf[offset..offset + 2])?;
        let byte = u8::from_str_radix(byte_str, 16)?;
        data.push(byte);
    }

    Ok(SlcanCommand::StandardFrame { id, data })
}
```

### Extended Frame Format

**Format**: `T<ID><LEN><DATA>\r`

**Example**: `T00000B0010A\r`
- ID: 0x00000B00 (8 hex digits)
- Length: 1 byte
- Data: 0x0A

```rust
fn parse_extended_frame(buf: &[u8]) -> Result<SlcanCommand, SlcanError> {
    // Expect: IIIIIIIILDD...\r
    if buf.len() < 10 {  // Min: ID(8) + LEN(1) + CR(1)
        return Err(SlcanError::FrameTooShort);
    }

    // Parse ID (8 hex digits)
    let id_str = core::str::from_utf8(&buf[0..8])?;
    let id = u32::from_str_radix(id_str, 16)?;

    if id > 0x1FFFFFFF {
        return Err(SlcanError::InvalidId);
    }

    // Parse length and data (same as standard)
    // ...
}
```

### SLCAN Responses

```rust
fn send_response(uart: &mut Uart, response: &[u8]) {
    uart.write_bytes(response).unwrap();
    uart.write_bytes(b"\r").unwrap();  // Terminate with CR
}

// OK response
send_response(uart, b"");  // Just CR

// Version response
send_response(uart, b"V1234");

// Error (bell character)
send_response(uart, b"\x07");
```

### Bitrate Mapping

```rust
fn slcan_bitrate_to_kbps(code: u8) -> Option<u32> {
    match code {
        0 => Some(10),     // 10 kbps
        1 => Some(20),     // 20 kbps
        2 => Some(50),     // 50 kbps
        3 => Some(100),    // 100 kbps
        4 => Some(125),    // 125 kbps
        5 => Some(250),    // 250 kbps
        6 => Some(500),    // 500 kbps (common)
        7 => Some(800),    // 800 kbps
        8 => Some(1000),   // 1000 kbps (1 Mbps)
        _ => None,
    }
}
```

## CAN (TWAI) Interface

### TWAI Setup

```rust
use esp_hal::twai::{Twai, TwaiConfig, BaudRate};

fn init_can(twai: TWAI0, tx: GPIO6, rx: GPIO7) -> Twai<'static> {
    let config = TwaiConfig::new()
        .set_baud_rate(BaudRate::B500K)
        .set_mode(TwaiMode::Normal);

    let twai = Twai::new(twai, tx, rx, &config);
    twai
}
```

### CAN Frame Transmission

```rust
use esp_hal::twai::{Frame, Id, StandardId, ExtendedId};

// Standard frame (11-bit ID)
fn send_standard(twai: &mut Twai, id: u16, data: &[u8]) {
    let frame = Frame::new(
        Id::Standard(StandardId::new(id).unwrap()),
        data
    ).unwrap();

    twai.transmit(&frame).unwrap();
}

// Extended frame (29-bit ID)
fn send_extended(twai: &mut Twai, id: u32, data: &[u8]) {
    let frame = Frame::new(
        Id::Extended(ExtendedId::new(id).unwrap()),
        data
    ).unwrap();

    twai.transmit(&frame).unwrap();
}
```

### CAN Frame Reception

```rust
loop {
    if let Ok(frame) = twai.receive() {
        match frame.id() {
            Id::Standard(id) => {
                println!("RX Standard ID: 0x{:03X}", id.as_raw());
            }
            Id::Extended(id) => {
                println!("RX Extended ID: 0x{:08X}", id.as_raw());
            }
        }

        let data = frame.data();
        println!("Data: {:02X?}", data);
    }

    // Don't busy-wait
    delay.delay_millis(1);
}
```

## Debugging with esp-println

### Printing

```rust
use esp_println::println;

println!("Hello from ESP32-S3!");
println!("Value: {}", 42);
println!("Hex: 0x{:02X}", 0xAB);
```

### Panic Messages

```rust
use esp_backtrace as _;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {:?}", info);
    loop {}
}
```

## Flash and Debug

### Setup Environment

```bash
# Install espup (Xtensa Rust toolchain)
cargo install espup
espup install

# Source environment (add to .bashrc/.zshrc)
source ~/export-esp.sh
```

### Build and Flash

```bash
cd mcu/bins/esp32s3

# Build
cargo build --release

# Flash and monitor
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/heltec-attachment

# Just flash (no monitor)
espflash flash target/xtensa-esp32s3-none-elf/release/heltec-attachment

# Erase flash
espflash erase-flash
```

### Monitor Serial

```bash
# Monitor only (after flashing)
espflash monitor

# Or use screen/minicom
screen /dev/ttyUSB0 115200
```

## Memory Layout

### Typical Binary

```
.text    (code):        ~400 KB
.rodata  (constants):   ~50 KB
.data    (initialized): ~10 KB
.bss     (zero-init):   ~20 KB
.heap:                  ~400 KB
---
Total Flash: ~460 KB
Total RAM: ~30 KB + heap (400 KB)
```

### Optimizing Size

**In `Cargo.toml`:**
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = "fat"              # Link-time optimization
codegen-units = 1        # Single codegen unit (better optimization)
strip = true             # Strip symbols
```

## Performance Tips

1. **Use DMA for bulk transfers** (SPI, I2C, UART)
2. **Minimize heap allocations** (pre-allocate buffers)
3. **Offload to second core** (ESP32-S3 is dual-core)
4. **Profile with `SystemTimer::now()`**
5. **Avoid busy-wait loops** (use delays)

## Common Pitfalls

### ❌ Missing Heap Allocator

```rust
// WRONG: Vec without heap allocator
#[main]
fn main() -> ! {
    let vec = Vec::new();  // ❌ Panics (no allocator)
}
```

### ✅ Initialize Heap First

```rust
// CORRECT
#[main]
fn main() -> ! {
    esp_alloc::heap_allocator!();  // ✅ First line
    let vec = Vec::new();  // Now works
}
```

### ❌ Busy-Wait Loop

```rust
// WRONG: 100% CPU usage
loop {
    if uart.read_ready() {
        // ...
    }
    // No delay!
}
```

### ✅ Add Delay

```rust
// CORRECT: Yield CPU
loop {
    if uart.read_ready() {
        // ...
    }
    delay.delay_millis(1);  // ✅ Prevent busy-wait
}
```

### ❌ RGB Instead of GRB

```rust
// WRONG: WS2812 expects GRB
let color = (r as u32) << 16 | (g as u32) << 8 | b as u32;  // ❌
```

### ✅ Correct GRB Order

```rust
// CORRECT
let color = (g as u32) << 16 | (r as u32) << 8 | b as u32;  // ✅ GRB
```

## References

- ESP-HAL Documentation: https://docs.esp-rs.org/esp-hal/
- ESP32-S3 Technical Reference: https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf
- esp-rs Book: https://docs.esp-rs.org/book/
- Heltec Automation: https://heltec.org/
