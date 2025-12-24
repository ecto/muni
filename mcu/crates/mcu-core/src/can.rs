//! CAN bus communication via MCP2515 SPI controller.

use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Config as SpiConfig, Spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_can::{ExtendedId, Frame as _, Id};
use mcp2515::{error::Error as McpError, frame::CanFrame, Config, McpOperationMode, MCP2515};

use crate::protocol::LedCommand;

/// CAN frame received from the bus.
#[derive(Debug, Clone, defmt::Format)]
pub struct RxFrame {
    pub id: u32,
    pub data: heapless::Vec<u8, 8>,
}

/// Channel for received CAN frames.
pub static RX_CHANNEL: Channel<CriticalSectionRawMutex, RxFrame, 8> = Channel::new();

/// CAN bus controller wrapping MCP2515.
pub struct CanBus<'d> {
    mcp: MCP2515<Spi<'d, SPI0, embassy_rp::spi::Async>, Output<'d>, Output<'d>>,
    _int: Input<'d>,
}

impl<'d> CanBus<'d> {
    /// Initialize MCP2515 on SPI0 with the given pins.
    ///
    /// Pins (matching plan):
    /// - GP16: MISO
    /// - GP17: CS
    /// - GP18: SCK
    /// - GP19: MOSI
    /// - GP20: INT
    pub async fn new(
        spi: Spi<'d, SPI0, embassy_rp::spi::Async>,
        cs: Output<'d>,
        int: Input<'d>,
    ) -> Result<Self, McpError> {
        // MCP2515 needs a dummy pin for reset (we don't use hardware reset)
        // Create a dummy output that we won't actually toggle
        let dummy_reset = unsafe {
            // We'll handle reset via SPI command instead
            core::mem::zeroed::<Output<'d>>()
        };

        let mut mcp = MCP2515::new(spi, cs, dummy_reset);

        // Reset and configure MCP2515
        // 500kbps CAN, matching VESC default
        let config = Config::default()
            .mode(McpOperationMode::Normal)
            .bitrate_500_000();

        mcp.init(&config)?;

        Ok(Self { mcp, _int: int })
    }

    /// Set acceptance filter to only receive specific CAN IDs.
    pub fn set_filter(&mut self, id: u32) -> Result<(), McpError> {
        // For now, accept all extended IDs
        // TODO: Implement proper filtering via MCP2515 registers
        let _ = id;
        Ok(())
    }

    /// Send a CAN frame.
    pub fn send(&mut self, id: u32, data: &[u8]) -> Result<(), McpError> {
        let ext_id = ExtendedId::new(id).unwrap_or(ExtendedId::ZERO);
        let frame = CanFrame::new(Id::Extended(ext_id), data).ok_or(McpError::InvalidArgument)?;
        self.mcp.send_message(&frame)?;
        Ok(())
    }

    /// Try to receive a CAN frame (non-blocking).
    pub fn try_receive(&mut self) -> Result<Option<RxFrame>, McpError> {
        match self.mcp.receive_message() {
            Ok(frame) => {
                let id = match frame.id() {
                    Id::Standard(sid) => sid.as_raw() as u32,
                    Id::Extended(eid) => eid.as_raw(),
                };
                let mut data = heapless::Vec::new();
                for &byte in frame.data() {
                    let _ = data.push(byte);
                }
                Ok(Some(RxFrame { id, data }))
            }
            Err(McpError::NoMessage) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Send LED command.
    pub fn send_led_command(&mut self, cmd: &LedCommand) -> Result<(), McpError> {
        self.send(crate::protocol::peripheral::LED_CMD, &cmd.to_bytes())
    }

    /// Send heartbeat status.
    pub fn send_heartbeat(
        &mut self,
        status: &crate::protocol::HeartbeatStatus,
    ) -> Result<(), McpError> {
        self.send(crate::protocol::peripheral::LED_STATUS, &status.to_bytes())
    }
}

/// Configuration for CAN bus initialization.
pub struct CanConfig {
    /// CAN bitrate in bps.
    pub bitrate: u32,
    /// Filter for received IDs (0 = accept all).
    pub filter_id: u32,
}

impl Default for CanConfig {
    fn default() -> Self {
        Self {
            bitrate: 500_000,
            filter_id: 0,
        }
    }
}
