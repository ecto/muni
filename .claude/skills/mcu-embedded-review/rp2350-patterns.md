# RP2350 (Pico 2 W) Embedded Patterns

This document provides detailed patterns and best practices for RP2350 (Raspberry Pi Pico 2 W) firmware development with Embassy async runtime.

## Platform Overview

**MCU**: RP2350 (Cortex-M33 dual-core, 150MHz)
**Memory**: 520 KB SRAM, 4 MB Flash
**Runtime**: Embassy async executor
**Edition**: Rust 2021
**Target**: `thumbv8m.main-none-eabihf`

## Embassy Async Patterns

### Entry Point

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1. Initialize HAL
    let p = embassy_rp::init(Default::default());

    // 2. Spawn tasks
    spawner.spawn(blink_task(p.PIN_25)).unwrap();
    spawner.spawn(usb_task(p.USB)).unwrap();

    // 3. Main loop (or just wait)
    loop {
        Timer::after_secs(1).await;
    }
}
```

**Key Points:**
- `#![no_std]` required (no standard library)
- `#![no_main]` required (custom entry point)
- `#[embassy_executor::main]` sets up async runtime
- `spawner: Spawner` argument for spawning tasks
- Peripherals initialized once with `embassy_rp::init()`

### Task Definitions

```rust
#[embassy_executor::task]
async fn blink_task(pin: PIN_25) {
    let mut led = Output::new(pin, Level::Low);

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn usb_task(usb: USB) {
    // USB device initialization
    let driver = Driver::new(usb, Irqs);

    // ... USB operations with .await
}
```

**Task Guidelines:**
- Mark with `#[embassy_executor::task]`
- Must be `async fn`
- Infinite loop or explicit return
- Take ownership of peripherals (moved from main)
- Use `.await` for async operations

### Interrupt Binding

```rust
use embassy_rp::{bind_interrupts, usb::InterruptHandler};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    UART0_IRQ => InterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Use Irqs type with peripherals
    let driver = Driver::new(p.USB, Irqs);
    let uart = Uart::new(p.UART0, Irqs, /* ... */);
}
```

**Binding Rules:**
- One `bind_interrupts!` per binary
- List all interrupt sources used
- Pass `Irqs` type to peripheral constructors

### Embassy Timers

```rust
use embassy_time::{Timer, Duration, Instant};

// Delay for milliseconds
async fn delay_example() {
    Timer::after_millis(100).await;
    Timer::after_secs(1).await;
}

// Duration type
async fn duration_example() {
    let duration = Duration::from_millis(500);
    Timer::after(duration).await;
}

// Timestamp
fn timestamp_example() {
    let now = Instant::now();
    // ... do work ...
    let elapsed = now.elapsed();
    defmt::info!("Elapsed: {}ms", elapsed.as_millis());
}
```

**Timer Features:**
- Non-blocking (async)
- High precision (1µs resolution)
- Low power (uses hardware timer)

## Static Memory Allocation

### StaticCell Pattern

```rust
use static_cell::StaticCell;

// Global static with StaticCell
static USB_BUS: StaticCell<UsbBus> = StaticCell::new();
static USB_DEVICE: StaticCell<UsbDevice> = StaticCell::new();

fn init_usb() {
    // Initialize once
    let bus = USB_BUS.init(UsbBus::new(/* ... */));
    let device = USB_DEVICE.init(UsbDevice::new(bus));
}
```

**Rules:**
- Use for `static mut` data (safe alternative)
- Call `.init()` exactly once
- Returns `&'static mut T`

### Fixed-Size Buffers

```rust
// Good: Fixed-size array
static BUFFER: [u8; 256] = [0; 256];

// Good: StaticCell for mutable buffer
static mut BUFFER: StaticCell<[u8; 256]> = StaticCell::new();

// Bad: Vec (requires heap)
// let mut buffer = Vec::new();  // ❌ Won't compile
```

### Embassy Channels

```rust
use embassy_sync::channel::{Channel, Sender, Receiver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

// Define channel type
type LedCommand = (u8, u8, u8);  // RGB color

// Global channel (8 message capacity)
static LED_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, LedCommand, 8>> = StaticCell::new();

// Initialize in main
let channel = LED_CHANNEL.init(Channel::new());
let sender = channel.sender();
let receiver = channel.receiver();

// Send from one task
async fn send_task(sender: Sender<'static, CriticalSectionRawMutex, LedCommand, 8>) {
    sender.send((255, 0, 0)).await;  // Red
}

// Receive in another task
async fn receive_task(receiver: Receiver<'static, CriticalSectionRawMutex, LedCommand, 8>) {
    let color = receiver.receive().await;
    defmt::info!("Received color: {:?}", color);
}
```

**Channel Features:**
- Bounded capacity (prevents unbounded growth)
- Async send/receive
- Multiple producers/consumers
- CriticalSectionRawMutex for Cortex-M

## PIO (Programmable I/O)

### WS2812 LED Driver

```rust
use embassy_rp::pio::{Pio, Common, StateMachine, Config};
use embassy_rp::gpio::Pin;

async fn ws2812_task(pio: PIO0, pin: PIN_25) {
    let Pio { mut common, sm0, .. } = Pio::new(pio);

    // WS2812 PIO program
    let prg = pio_proc::pio_asm!(
        ".side_set 1 opt",
        ".wrap_target",
        "bitloop:",
        "  out x, 1       side 0 [1]",
        "  jmp !x do_zero side 1 [2]",
        "do_one:",
        "  jmp bitloop    side 1 [4]",
        "do_zero:",
        "  nop            side 0 [4]",
        ".wrap"
    );

    let program = common.load_program(&prg.program);

    // Configure state machine
    let mut cfg = Config::default();
    cfg.use_program(&program, &[]);
    cfg.set_out_pins(&[&pin]);
    cfg.set_set_pins(&[&pin]);
    cfg.clock_divider = 125.into();  // 1MHz for WS2812

    // Start state machine
    sm0.set_config(&cfg);
    sm0.set_enable(true);

    // Send color data
    loop {
        // GRB format for WS2812
        let g = 255u8;
        let r = 0u8;
        let b = 0u8;

        let grb = ((g as u32) << 16) | ((r as u32) << 8) | (b as u32);

        // Write 24 bits
        for bit in (0..24).rev() {
            let value = (grb >> bit) & 1;
            sm0.tx().write(value as u32);
        }

        // Reset (>50µs low)
        Timer::after_micros(60).await;
    }
}
```

**PIO Guidelines:**
- 8 PIO state machines total (use judiciously)
- Clock divider sets timing
- Side-set for precise pin control
- DMA for bulk transfers (advanced)

### PIO Assembly Reference

**Instructions:**
- `out x, 1`: Shift 1 bit from OSR to x register
- `in x, 1`: Shift 1 bit from x to ISR
- `jmp`: Conditional/unconditional jump
- `side`: Set side-set pins (precise timing)
- `[N]`: Delay N cycles
- `.wrap_target` / `.wrap`: Loop boundaries

**Registers:**
- `x`, `y`: Scratch registers (32-bit)
- `OSR`: Output Shift Register
- `ISR`: Input Shift Register

## USB CDC Serial

### Complete USB Setup

```rust
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::{Builder, Config};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

static EP_MEM: StaticCell<[u8; 1024]> = StaticCell::new();
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static MSOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

#[embassy_executor::task]
async fn usb_task(driver: Driver<'static, USB>) {
    // USB configuration
    let mut config = Config::new(0x16c0, 0x27dd);  // VID:PID
    config.manufacturer = Some("Muni Robotics");
    config.product = Some("BVR LED Controller");
    config.serial_number = Some("12345678");
    config.max_power = 100;  // 100mA

    // Build USB device
    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        MSOS_DESC.init([0; 256]),
        CONTROL_BUF.init([0; 64]),
    );

    // Add CDC-ACM class (USB serial)
    let mut state = State::new();
    let mut cdc = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build and run USB device
    let mut usb = builder.build();

    loop {
        usb.run_until_suspend().await;
    }
}

// Separate task for CDC communication
#[embassy_executor::task]
async fn cdc_task(mut cdc: CdcAcmClass<'static, Driver<'static, USB>>) {
    loop {
        cdc.wait_connection().await;
        defmt::info!("USB connected");

        loop {
            let mut buf = [0u8; 64];
            match cdc.read_packet(&mut buf).await {
                Ok(n) => {
                    // Echo back
                    let _ = cdc.write_packet(&buf[..n]).await;
                }
                Err(_) => break,  // Disconnected
            }
        }

        defmt::info!("USB disconnected");
    }
}
```

**USB Best Practices:**
- Unique VID:PID (0x16c0:0x27dd is test ID)
- Descriptive manufacturer/product strings
- Separate task for USB device and CDC communication
- Handle connection/disconnection gracefully

## GPIO Operations

### Output Pin

```rust
use embassy_rp::gpio::{Output, Level};

let mut led = Output::new(pin, Level::Low);

led.set_high();
led.set_low();
led.toggle();

if led.is_set_high() {
    // ...
}
```

### Input Pin (with Pull-up)

```rust
use embassy_rp::gpio::{Input, Pull};

let button = Input::new(pin, Pull::Up);

loop {
    button.wait_for_falling_edge().await;  // Button pressed
    defmt::info!("Button pressed!");

    button.wait_for_rising_edge().await;   // Button released
    defmt::info!("Button released!");
}
```

### Async Wait for Edge

```rust
// Wait for any edge
button.wait_for_any_edge().await;

// Wait for specific edge
button.wait_for_rising_edge().await;
button.wait_for_falling_edge().await;
```

## UART Communication

### UART Setup

```rust
use embassy_rp::uart::{Uart, Config};

let mut uart = Uart::new(
    p.UART0,
    p.PIN_0,   // TX
    p.PIN_1,   // RX
    Irqs,
    p.DMA_CH0,
    p.DMA_CH1,
    Config::default(),
);

// Write
uart.write(b"Hello\r\n").await.unwrap();

// Read
let mut buf = [0u8; 64];
let n = uart.read(&mut buf).await.unwrap();
```

### Non-Blocking Read with Timeout

```rust
use embassy_time::{with_timeout, Duration};

let result = with_timeout(
    Duration::from_millis(100),
    uart.read(&mut buf)
).await;

match result {
    Ok(Ok(n)) => {
        defmt::info!("Read {} bytes", n);
    }
    Ok(Err(e)) => {
        defmt::error!("UART error: {:?}", e);
    }
    Err(_) => {
        defmt::warn!("UART read timeout");
    }
}
```

## Debugging with defmt

### Logging

```rust
use defmt::*;

info!("System initialized");
warn!("Battery low: {}V", voltage);
error!("Motor fault: {:?}", error);
debug!("Loop iteration: {}", count);
```

### Formatting

```rust
// Basic types
info!("Value: {}", 42);

// Debug format
info!("Struct: {:?}", my_struct);

// Hex
info!("Byte: {:02x}", 0xAB);

// Binary
info!("Bits: {:08b}", 0b10101010);
```

### Panic Handler

```rust
use defmt_rtt as _;
use panic_probe as _;

// Panics will be logged via defmt and probe-rs
```

## Flash and Debug

### Build

```bash
# Build for RP2350
cargo build --release --target thumbv8m.main-none-eabihf

# Size information
cargo size --release --target thumbv8m.main-none-eabihf
```

### Flash with picotool

```bash
# Put Pico in BOOTSEL mode (hold BOOTSEL, press RESET)

# Flash ELF file
picotool load target/thumbv8m.main-none-eabihf/release/rover-leds -t elf -f

# Reboot
picotool reboot
```

### Debug with probe-rs

```bash
# Run with logging
probe-rs run --chip RP2350 target/thumbv8m.main-none-eabihf/release/rover-leds

# Attach debugger
probe-rs attach --chip RP2350
```

## Memory Layout

### Typical Binary

```
.text    (code):        ~80 KB
.rodata  (constants):   ~10 KB
.data    (initialized): ~5 KB
.bss     (zero-init):   ~50 KB
---
Total Flash: ~95 KB
Total RAM: ~55 KB + stack
```

### Stack Size

Default stack: 16 KB

**Configure in `memory.x`:**
```
MEMORY
{
  FLASH : ORIGIN = 0x10000000, LENGTH = 4096K
  RAM   : ORIGIN = 0x20000000, LENGTH = 520K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

## Performance Tips

1. **Use DMA for bulk transfers** (UART, SPI, I2C)
2. **Avoid polling in tight loops** (use async `.await`)
3. **Minimize allocations** (use fixed-size buffers)
4. **Profile with `defmt::timestamp`**
5. **Optimize critical sections** (minimize interrupt-disabled time)

## Common Pitfalls

### ❌ Blocking in Async Context

```rust
// WRONG
async fn delay_bad() {
    cortex_m::asm::delay(1_000_000);  // ❌ Blocks executor
}
```

### ✅ Correct Async Delay

```rust
// CORRECT
async fn delay_good() {
    Timer::after_millis(100).await;  // ✅ Non-blocking
}
```

### ❌ Forgetting .await

```rust
// WRONG
async fn send_bad() {
    uart.write(b"Hello");  // ❌ Missing .await
}
```

### ✅ Correct Await

```rust
// CORRECT
async fn send_good() {
    uart.write(b"Hello").await.unwrap();  // ✅ With .await
}
```

## References

- Embassy Documentation: https://embassy.dev/
- RP2350 Datasheet: https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf
- Embassy RP HAL: https://docs.embassy.dev/embassy-rp/
- PIO Guide: https://github.com/raspberrypi/pico-sdk/tree/master/docs/pio
