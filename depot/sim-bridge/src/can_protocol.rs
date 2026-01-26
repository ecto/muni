//! TCP CAN frame protocol for sim-bridge ↔ bvrd communication.
//!
//! Wire format:
//!   CAN Frame:  type(0x01) + flags(1) + id(4 LE) + len(1) + data(0-8)
//!   Pose:       type(0x02) + x(f64 LE) + y(f64 LE) + theta(f64 LE) = 25 bytes

/// A CAN frame (local type, not depending on firmware workspace).
#[derive(Debug, Clone)]
pub struct CanFrame {
    pub id: u32,
    pub extended: bool,
    pub data: Vec<u8>,
}

impl CanFrame {
    pub fn new_extended(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            extended: true,
            data: data.to_vec(),
        }
    }
}

/// Messages exchanged over TCP.
#[derive(Debug)]
pub enum Message {
    Frame(CanFrame),
    Pose { x: f64, y: f64, theta: f64 },
}

pub const MSG_CAN_FRAME: u8 = 0x01;
pub const MSG_POSE: u8 = 0x02;

/// Serialize a CAN frame for TCP wire.
/// Returns: type(0x01) + flags(1) + id(4 LE) + len(1) + data
pub fn encode_can_frame(frame: &CanFrame) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8 + frame.data.len());
    buf.push(MSG_CAN_FRAME);
    let flags: u8 = if frame.extended { 0x01 } else { 0x00 };
    buf.push(flags);
    buf.extend_from_slice(&frame.id.to_le_bytes());
    buf.push(frame.data.len() as u8);
    buf.extend_from_slice(&frame.data);
    buf
}

/// Serialize a pose update for TCP wire.
/// Returns: type(0x02) + x(f64 LE) + y(f64 LE) + theta(f64 LE)
pub fn encode_pose(x: f64, y: f64, theta: f64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(25);
    buf.push(MSG_POSE);
    buf.extend_from_slice(&x.to_le_bytes());
    buf.extend_from_slice(&y.to_le_bytes());
    buf.extend_from_slice(&theta.to_le_bytes());
    buf
}

/// Parse the next message from a byte buffer.
/// Returns (message, bytes_consumed) or None if incomplete.
pub fn decode_message(buf: &[u8]) -> Option<(Message, usize)> {
    if buf.is_empty() {
        return None;
    }
    match buf[0] {
        MSG_CAN_FRAME => {
            // type(1) + flags(1) + id(4) + len(1) = 7 minimum
            if buf.len() < 7 {
                return None;
            }
            let flags = buf[1];
            let id = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
            let len = buf[6] as usize;
            if len > 8 || buf.len() < 7 + len {
                return None;
            }
            let data = buf[7..7 + len].to_vec();
            let extended = (flags & 0x01) != 0;
            Some((
                Message::Frame(CanFrame {
                    id,
                    extended,
                    data,
                }),
                7 + len,
            ))
        }
        MSG_POSE => {
            if buf.len() < 25 {
                return None;
            }
            let x = f64::from_le_bytes(buf[1..9].try_into().unwrap());
            let y = f64::from_le_bytes(buf[9..17].try_into().unwrap());
            let theta = f64::from_le_bytes(buf[17..25].try_into().unwrap());
            Some((Message::Pose { x, y, theta }, 25))
        }
        _ => {
            // Skip unknown byte
            Some((Message::Frame(CanFrame { id: 0, extended: false, data: vec![] }), 1))
        }
    }
}
