//! Generic attachment controller firmware for RP2350 MCUs.
//!
//! Configurable for different attachment types via compile-time features.
//! Default configuration is LED controller for base rover lights.

#![no_std]
#![no_main]

mod config;

use config::{get_config, DeviceConfig, DeviceType};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{Grb, PioWs2812, PioWs2812Program};
use embassy_rp::usb::Driver as UsbDriver;
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder as UsbBuilder, Config as UsbConfig, UsbDevice};
use mcu_leds::LedController;
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
    embassy_rp::binary_info::rp_program_name!(c"Muni MCU RP2350"),
    embassy_rp::binary_info::rp_program_description!(c"Generic attachment controller for Muni rover"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// LED count (from config, but const generic requires compile-time value)
const NUM_LEDS: usize = 24;

static EP_MEMORY: StaticCell<[u8; 1024]> = StaticCell::new();
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
static CDC_STATE: StaticCell<State> = StaticCell::new();
static DEVICE_CONFIG: StaticCell<DeviceConfig> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = DEVICE_CONFIG.init(get_config());

    info!("=== Muni MCU Firmware (RP2350) ===");
    info!("Device: {}", config.name);
    info!("Type: {:?}", config.device_type);
    info!("CAN ID: 0x{:04X}", config.can.base_id);

    let p = embassy_rp::init(Default::default());

    // USB CDC setup for debugging
    let usb_driver = UsbDriver::new(p.USB, Irqs);

    let mut usb_cfg = UsbConfig::new(0xCafe, 0x4004);
    usb_cfg.manufacturer = Some("Muni");
    usb_cfg.product = Some(config.name);
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
    spawner.spawn(cdc_task(cdc)).ok();

    // Initialize based on device type
    match config.device_type {
        DeviceType::LedController => {
            info!("Starting LED controller");

            let Pio {
                mut common, sm0, ..
            } = Pio::new(p.PIO0, Irqs);

            let prg = PioWs2812Program::new(&mut common);

            let mut ws2812: PioWs2812<'_, PIO0, 0, NUM_LEDS, Grb> =
                PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_0, &prg);

            let mut controller: LedController<NUM_LEDS> = LedController::new();

            // Start with idle mode (solid green)
            controller.set_mode_immediate(mcu_leds::LedMode::idle());

            loop {
                let buffer = controller.update();
                ws2812.write(buffer).await;
                Timer::after(Duration::from_millis(20)).await;
            }
        }
        DeviceType::BrushAttachment => {
            info!("Starting brush attachment controller");
            // TODO: Implement brush motor control
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        }
        _ => {
            info!("Unknown device type, running heartbeat only");
            let start = Instant::now();
            loop {
                let uptime = start.elapsed().as_secs();
                info!("Heartbeat: uptime={}s", uptime);
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}

/// USB device task.
#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, UsbDriver<'static, USB>>) {
    usb.run().await;
}

/// USB CDC task for serial communication.
#[embassy_executor::task]
async fn cdc_task(mut cdc: CdcAcmClass<'static, UsbDriver<'static, USB>>) {
    loop {
        cdc.wait_connection().await;
        info!("USB CDC connected");

        let mut buf = [0u8; 64];
        loop {
            match cdc.read_packet(&mut buf).await {
                Ok(n) => {
                    if n > 0 {
                        // Echo back for testing
                        let _ = cdc.write_packet(&buf[..n]).await;
                    }
                }
                Err(_) => break,
            }
        }

        info!("USB CDC disconnected");
    }
}
