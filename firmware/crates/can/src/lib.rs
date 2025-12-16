//! CAN bus abstraction and VESC protocol for bvr.
//!
//! On Linux, uses SocketCAN. On other platforms, provides a mock for development.

pub mod vesc;

use thiserror::Error;
use tracing::warn;

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
}

// ============================================================================
// Linux implementation (real SocketCAN)
// ============================================================================

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use socketcan::{CanFrame, CanSocket, EmbeddedFrame, ExtendedId, Socket, StandardId};
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
            socket
                .set_read_timeout(Duration::from_millis(10))
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
    use tracing::debug;

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

