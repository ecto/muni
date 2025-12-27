//! Rover LED controller firmware - Using stock WS2812 driver
//!
//! WLED works with this strip, so let's use the proven driver.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{Grb, PioWs2812, PioWs2812Program};
use embassy_rp::usb::Driver as UsbDriver;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder as UsbBuilder, Config as UsbConfig};
use smart_leds::RGB8;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Rover LEDs"),
    embassy_rp::binary_info::rp_program_description!(c"WS2811 LED controller for Muni rover"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// LED count - try small number first
const NUM_LEDS: usize = 24;

static EP_MEMORY: StaticCell<[u8; 1024]> = StaticCell::new();
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
static CDC_STATE: StaticCell<State> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== LED test (stock driver, GRB) - {} LEDs ===", NUM_LEDS);

    // USB CDC setup for basic debugging over USB
    let usb_driver = UsbDriver::new(p.USB, Irqs);

    let mut usb_cfg = UsbConfig::new(0xCafe, 0x4004);
    usb_cfg.manufacturer = Some("Muni");
    usb_cfg.product = Some("Rover LEDs");
    usb_cfg.serial_number = Some("0001");

    let ep_mem = EP_MEMORY.init([0u8; 1024]);
    let mut usb_builder = UsbBuilder::new(
        usb_driver,
        usb_cfg,
        ep_mem,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        CONTROL_BUF.init([0; 128]),
    );

    let cdc = CdcAcmClass::new(&mut usb_builder, CDC_STATE.init(State::new()), 64);
    let usb = usb_builder.build();

    spawner.spawn(usb_task(usb)).ok();
    spawner.spawn(cdc_log_task(cdc)).ok();

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let prg = PioWs2812Program::new(&mut common);

    // Use the stock WS2812 driver with GRB color order
    let mut ws2812: PioWs2812<'_, PIO0, 0, NUM_LEDS, Grb> =
        PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_0, &prg);

    let mut buffer: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    // Clear on startup
    info!("Clearing...");
    ws2812.write(&buffer).await;
    Timer::after(Duration::from_millis(100)).await;
    ws2812.write(&buffer).await;
    Timer::after(Duration::from_millis(100)).await;

    info!("Starting multi-color mapping test");

    loop {
        // Test with single and mixed channels to deduce mapping.
        info!("Send R=255 G=0 B=0 (expect red)");
        buffer.fill(RGB8::new(255, 0, 0));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send G=255 R=0 B=0 (expect green)");
        buffer.fill(RGB8::new(0, 255, 0));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send B=255 R=0 G=0 (expect blue)");
        buffer.fill(RGB8::new(0, 0, 255));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send YELLOW R=255 G=255 B=0 (expect yellow)");
        buffer.fill(RGB8::new(255, 255, 0));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send CYAN R=0 G=255 B=255 (expect cyan)");
        buffer.fill(RGB8::new(0, 255, 255));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send MAGENTA R=255 G=0 B=255 (expect magenta)");
        buffer.fill(RGB8::new(255, 0, 255));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;

        info!("Send WHITE R=255 G=255 B=255 (expect white)");
        buffer.fill(RGB8::new(255, 255, 255));
        ws2812.write(&buffer).await;
        Timer::after(Duration::from_secs(4)).await;
    }
}
