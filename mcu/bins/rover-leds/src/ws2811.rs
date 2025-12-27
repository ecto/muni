//! WS2811 LED driver with 400kHz timing.
//!
//! Based on embassy-rp's PioWs2812 but with halved clock frequency.

use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use embassy_rp::Peri;
use embassy_time::Timer;
use fixed::types::U24F8;
use smart_leds::RGB8;

const T1: u8 = 2; // start bit
const T2: u8 = 5; // data bit
const T3: u8 = 3; // stop bit
const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

/// WS2811 program loaded into PIO instruction memory.
pub struct Ws2811Program<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> Ws2811Program<'a, PIO> {
    /// Load the WS2811 program into PIO.
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let prg = common.load_program(&prg);

        Self { prg }
    }
}

/// WS2811 driver using PIO with 400kHz timing.
pub struct Ws2811<'d, P: Instance, const S: usize, const N: usize> {
    dma: Peri<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize, const N: usize> Ws2811<'d, P, S, N> {
    /// Create a new WS2811 driver with 400kHz timing.
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        pin: Peri<'d, impl PioPin>,
        program: &Ws2811Program<'d, P>,
    ) -> Self {
        let mut cfg = Config::default();

        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&program.prg, &[&out_pin]);

        // Clock config for 400kHz (WS2811) instead of 800kHz (WS2812)
        let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
        let ws2811_freq = U24F8::from_num(400); // 400kHz for WS2811
        let bit_freq = ws2811_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24, // Only 24 bits per LED, not 32
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.into(),
            sm,
        }
    }

    /// Write RGB data to the LED strip.
    pub async fn write(&mut self, colors: &[RGB8; N]) {
        // Pack RGB into upper 24 bits (for 24-bit threshold)
        // WS2811 uses RGB order, MSB first
        let mut words = [0u32; N];
        for i in 0..N {
            let c = colors[i];
            // Data in bits 31-8 (upper 24 bits), bits 7-0 unused
            words[i] = (u32::from(c.r) << 24) | (u32::from(c.g) << 16) | (u32::from(c.b) << 8);
        }

        self.sm.tx().dma_push(self.dma.reborrow(), &words, false).await;

        // WS2811 reset time (needs >50µs)
        Timer::after_micros(80).await;
    }
}

