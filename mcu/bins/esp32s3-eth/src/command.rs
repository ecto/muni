//! UDP command listener — receives commands from bvrd.

use std::net::UdpSocket;

/// Command packet size.
const COMMAND_SIZE: usize = 16;

/// Message types.
pub const MSG_SET_STATE: u8 = 0x10;
pub const MSG_TOOL_COMMAND: u8 = 0x11;

/// Parsed command from bvrd.
#[derive(Debug)]
pub enum Command {
    /// Rover state changed (mode byte).
    SetState { rover_state: u8 },
    /// Tool command (axis, motor, actions).
    ToolCommand {
        axis: i16,
        motor: i16,
        action_a: bool,
        action_b: bool,
    },
}

/// Command listener on a UDP port.
pub struct CommandListener {
    socket: UdpSocket,
    buf: [u8; 64],
}

impl CommandListener {
    pub fn new(port: u16) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buf: [0u8; 64],
        })
    }

    /// Try to receive a command (non-blocking). Returns None if no data.
    pub fn try_recv(&mut self) -> Option<Command> {
        match self.socket.recv_from(&mut self.buf) {
            Ok((len, _addr)) => {
                if len < COMMAND_SIZE {
                    return None;
                }
                self.parse(&self.buf[..len])
            }
            Err(_) => None,
        }
    }

    fn parse(&self, data: &[u8]) -> Option<Command> {
        let msg_type = data[0];
        let payload = &data[8..];

        match msg_type {
            MSG_SET_STATE => Some(Command::SetState {
                rover_state: payload[0],
            }),
            MSG_TOOL_COMMAND => {
                let axis = i16::from_le_bytes([payload[0], payload[1]]);
                let motor = i16::from_le_bytes([payload[2], payload[3]]);
                let action_a = payload[4] != 0;
                let action_b = payload[5] != 0;
                Some(Command::ToolCommand {
                    axis,
                    motor,
                    action_a,
                    action_b,
                })
            }
            _ => None,
        }
    }
}
