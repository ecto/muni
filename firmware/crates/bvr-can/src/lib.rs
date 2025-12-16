//! CAN bus abstraction and ESC communication for bvr.
//!
//! On Linux, uses SocketCAN. On other platforms, provides a mock for development.

use bvr_types::{EscFault, EscStatus, WheelPosition, WheelVelocities};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Error, Debug)]
pub enum CanError {
    #[error("Socket error: {0}")]
    Socket(String),
    #[error("Invalid ESC ID: {0}")]
    InvalidId(u8),
    #[error("Timeout waiting for response")]
    Timeout,
    #[error("Invalid frame data")]
    InvalidFrame,
    #[error("Not supported on this platform")]
    NotSupported,
}

/// CAN message IDs for ESC communication.
/// Using a simple protocol:
/// - 0x100 + id: velocity command to ESC
/// - 0x200 + id: status report from ESC
mod msg_id {
    #[cfg(target_os = "linux")]
    pub const CMD_BASE: u16 = 0x100;
    pub const STATUS_BASE: u16 = 0x200;
}

// ============================================================================
// Linux implementation (real SocketCAN)
// ============================================================================

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

    /// CAN bus interface for ESC communication.
    pub struct CanBus {
        socket: CanSocket,
        esc_ids: [u8; 4],
    }

    impl CanBus {
        /// Open a CAN socket on the specified interface.
        pub fn open(interface: &str, esc_ids: [u8; 4]) -> Result<Self, CanError> {
            let socket = CanSocket::open(interface).map_err(|e| CanError::Socket(e.to_string()))?;
            socket
                .set_read_timeout(Duration::from_millis(10))
                .map_err(|e| CanError::Socket(e.to_string()))?;
            socket
                .set_write_timeout(Duration::from_millis(10))
                .map_err(|e| CanError::Socket(e.to_string()))?;

            Ok(Self { socket, esc_ids })
        }

        /// Send velocity commands to all ESCs.
        pub fn send_velocities(&self, velocities: &WheelVelocities) -> Result<(), CanError> {
            for (i, vel) in velocities.as_array().iter().enumerate() {
                let id = self.esc_ids[i];
                self.send_velocity_cmd(id, *vel)?;
            }
            Ok(())
        }

        /// Send velocity command to a single ESC.
        fn send_velocity_cmd(&self, esc_id: u8, velocity_rps: f64) -> Result<(), CanError> {
            let can_id = msg_id::CMD_BASE + esc_id as u16;
            let std_id = StandardId::new(can_id).ok_or(CanError::InvalidId(esc_id))?;

            // Encode velocity as signed 32-bit fixed point (16.16)
            let vel_fixed = (velocity_rps * 65536.0) as i32;
            let data = vel_fixed.to_le_bytes();

            let frame = CanFrame::new(std_id, &data).ok_or(CanError::InvalidFrame)?;
            self.socket
                .write_frame(&frame)
                .map_err(|e| CanError::Socket(e.to_string()))?;

            debug!(esc_id, velocity_rps, "Sent velocity command");
            Ok(())
        }

        /// Read a single frame from the bus (non-blocking with timeout).
        pub fn read_frame(&self) -> Result<Option<RawCanFrame>, CanError> {
            match self.socket.read_frame() {
                Ok(frame) => Ok(Some(RawCanFrame {
                    id: frame.raw_id() as u16,
                    data: frame.data().to_vec(),
                })),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
                Err(e) => Err(CanError::Socket(e.to_string())),
            }
        }

        /// Get the ESC ID for a wheel position.
        pub fn esc_id(&self, position: WheelPosition) -> u8 {
            self.esc_ids[position.index()]
        }
    }
}

// ============================================================================
// Non-Linux mock implementation (for development)
// ============================================================================

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::*;

    /// Mock CAN bus for development on non-Linux platforms.
    pub struct CanBus {
        esc_ids: [u8; 4],
    }

    impl CanBus {
        /// Create a mock CAN interface.
        pub fn open(interface: &str, esc_ids: [u8; 4]) -> Result<Self, CanError> {
            warn!(
                interface,
                "Using mock CAN bus (not on Linux) - no real hardware communication"
            );
            Ok(Self { esc_ids })
        }

        /// Mock send - just logs.
        pub fn send_velocities(&self, velocities: &WheelVelocities) -> Result<(), CanError> {
            debug!(?velocities, "Mock: sending velocities");
            Ok(())
        }

        /// Mock read - always returns None.
        pub fn read_frame(&self) -> Result<Option<RawCanFrame>, CanError> {
            Ok(None)
        }

        /// Get the ESC ID for a wheel position.
        pub fn esc_id(&self, position: WheelPosition) -> u8 {
            self.esc_ids[position.index()]
        }
    }
}

// ============================================================================
// Common types and re-exports
// ============================================================================

pub use platform::CanBus;

/// Raw CAN frame (platform-independent).
#[derive(Debug, Clone)]
pub struct RawCanFrame {
    pub id: u16,
    pub data: Vec<u8>,
}

impl RawCanFrame {
    /// Parse as an ESC status frame.
    pub fn parse_esc_status(&self, esc_ids: &[u8; 4]) -> Option<EscStatus> {
        // Check if this is a status message
        if self.id < msg_id::STATUS_BASE || self.id >= msg_id::STATUS_BASE + 256 {
            return None;
        }

        let esc_id = (self.id - msg_id::STATUS_BASE) as u8;

        // Verify this is one of our ESCs
        if !esc_ids.contains(&esc_id) {
            warn!(esc_id, "Received status from unknown ESC");
            return None;
        }

        if self.data.len() < 8 {
            warn!(esc_id, len = self.data.len(), "Status frame too short");
            return None;
        }

        // Decode status frame:
        // [0:3] velocity (fixed 16.16)
        // [4:5] current (mA, u16)
        // [6]   temperature (°C, i8)
        // [7]   fault flags
        let vel_fixed =
            i32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
        let current_ma = u16::from_le_bytes([self.data[4], self.data[5]]);
        let temp = self.data[6] as i8;
        let fault_byte = self.data[7];

        Some(EscStatus {
            id: esc_id,
            velocity: vel_fixed as f64 / 65536.0,
            current: current_ma as f64 / 1000.0,
            temperature: temp as f64,
            fault: EscFault {
                over_current: fault_byte & 0x01 != 0,
                over_temperature: fault_byte & 0x02 != 0,
                under_voltage: fault_byte & 0x04 != 0,
                encoder_error: fault_byte & 0x08 != 0,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_velocity_encoding() {
        // Test that our fixed-point encoding is reasonable
        let vel = 1.5_f64;
        let fixed = (vel * 65536.0) as i32;
        let decoded = fixed as f64 / 65536.0;
        assert!((vel - decoded).abs() < 0.0001);
    }
}

