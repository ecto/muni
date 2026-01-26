//! CAN bus abstraction and VESC protocol for bvr.
//!
//! On Linux, uses SocketCAN. On other platforms, provides a mock for development.

pub mod leds;
pub mod vesc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanError {
    #[error("Socket error: {0}")]
    Socket(String),
    #[error("Invalid CAN ID: {0}")]
    InvalidId(u32),
    #[error("Timeout waiting for response")]
    Timeout,
    #[error("Invalid frame data")]
    InvalidFrame,
    #[error("Not supported on this platform")]
    NotSupported,
}

/// Raw CAN frame (platform-independent).
#[derive(Debug, Clone)]
pub struct Frame {
    /// CAN ID (standard or extended)
    pub id: u32,
    /// Is this an extended (29-bit) ID?
    pub extended: bool,
    /// Frame data (0-8 bytes)
    pub data: Vec<u8>,
}

impl Frame {
    pub fn new(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            extended: id > 0x7FF,
            data: data.to_vec(),
        }
    }

    pub fn new_extended(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            extended: true,
            data: data.to_vec(),
        }
    }

    /// Serialize to wire format for TCP CAN bridge.
    ///
    /// Format: flags(1) + id(4 LE) + len(1) + data(0-8) = 6-14 bytes
    /// flags bit 0: extended ID
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(6 + self.data.len());
        let flags: u8 = if self.extended { 0x01 } else { 0x00 };
        buf.push(flags);
        buf.extend_from_slice(&self.id.to_le_bytes());
        buf.push(self.data.len() as u8);
        buf.extend_from_slice(&self.data);
        buf
    }

    /// Deserialize from wire format. Returns (frame, bytes_consumed) or None.
    pub fn from_bytes(buf: &[u8]) -> Option<(Self, usize)> {
        if buf.len() < 6 {
            return None;
        }
        let flags = buf[0];
        let id = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let len = buf[5] as usize;
        if len > 8 || buf.len() < 6 + len {
            return None;
        }
        let data = buf[6..6 + len].to_vec();
        let extended = (flags & 0x01) != 0;
        Some((Self { id, extended, data }, 6 + len))
    }
}

// ============================================================================
// Linux implementation (real SocketCAN)
// ============================================================================

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use socketcan::{
        CanFrame, CanSocket, EmbeddedFrame, ExtendedId, Frame as SocketcanFrame, Socket,
        StandardId,
    };
    use std::time::Duration;

    /// CAN bus interface.
    pub struct Bus {
        socket: CanSocket,
    }

    impl Bus {
        /// Open a CAN socket on the specified interface.
        pub fn open(interface: &str) -> Result<Self, CanError> {
            let socket =
                CanSocket::open(interface).map_err(|e| CanError::Socket(e.to_string()))?;
            // Short read timeout to minimize blocking on the control loop.
            // VESCs send ~4000 frames/sec total, so frames arrive every ~0.25ms.
            // 1ms timeout catches most frames while keeping loop latency low.
            socket
                .set_read_timeout(Duration::from_millis(1))
                .map_err(|e| CanError::Socket(e.to_string()))?;
            socket
                .set_write_timeout(Duration::from_millis(10))
                .map_err(|e| CanError::Socket(e.to_string()))?;

            Ok(Self { socket })
        }

        /// Send a CAN frame.
        pub fn send(&self, frame: &Frame) -> Result<(), CanError> {
            let can_frame = if frame.extended {
                let id = ExtendedId::new(frame.id).ok_or(CanError::InvalidId(frame.id))?;
                CanFrame::new(id, &frame.data).ok_or(CanError::InvalidFrame)?
            } else {
                let id = StandardId::new(frame.id as u16).ok_or(CanError::InvalidId(frame.id))?;
                CanFrame::new(id, &frame.data).ok_or(CanError::InvalidFrame)?
            };

            self.socket
                .write_frame(&can_frame)
                .map_err(|e| CanError::Socket(e.to_string()))?;
            Ok(())
        }

        /// Read a single frame (non-blocking with timeout).
        pub fn recv(&self) -> Result<Option<Frame>, CanError> {
            match self.socket.read_frame() {
                Ok(frame) => {
                    let (id, extended) = if frame.is_extended() {
                        (frame.raw_id(), true)
                    } else {
                        (frame.raw_id(), false)
                    };
                    Ok(Some(Frame {
                        id,
                        extended,
                        data: frame.data().to_vec(),
                    }))
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
                Err(e) => Err(CanError::Socket(e.to_string())),
            }
        }
    }
}

// ============================================================================
// Non-Linux mock implementation (for development)
// ============================================================================

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::*;
    use tracing::{debug, warn};

    /// Mock CAN bus for development on non-Linux platforms.
    pub struct Bus {
        #[allow(dead_code)]
        interface: String,
    }

    impl Bus {
        pub fn open(interface: &str) -> Result<Self, CanError> {
            warn!(
                interface,
                "Using mock CAN bus (not on Linux) - no real hardware communication"
            );
            Ok(Self {
                interface: interface.to_string(),
            })
        }

        pub fn send(&self, frame: &Frame) -> Result<(), CanError> {
            debug!(id = frame.id, len = frame.data.len(), "Mock CAN TX");
            Ok(())
        }

        pub fn recv(&self) -> Result<Option<Frame>, CanError> {
            Ok(None)
        }
    }
}

pub use platform::Bus;



