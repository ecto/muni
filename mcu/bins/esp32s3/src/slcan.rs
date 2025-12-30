//! SLCAN (Serial Line CAN) Protocol Implementation
//!
//! Implements the SLCAN protocol for CAN-over-serial communication.
//! Compatible with Linux slcand and can-utils.
//!
//! ## Frame Format
//!
//! ```text
//! tIIILDD...\r     - Transmit standard frame (11-bit ID)
//! TIIIIIIIILDD...\r - Transmit extended frame (29-bit ID)
//! rIIIL\r          - Transmit standard RTR frame
//! RIIIIIIIIL\r     - Transmit extended RTR frame
//! ```
//!
//! Where:
//! - III = 3 hex chars for 11-bit ID (000-7FF)
//! - IIIIIIII = 8 hex chars for 29-bit ID
//! - L = data length (0-8)
//! - DD = data bytes as hex pairs
//!
//! ## Commands
//!
//! - `O\r` - Open CAN channel
//! - `C\r` - Close CAN channel
//! - `S0-S8\r` - Set bitrate (S6 = 500kbps)
//! - `V\r` - Version query
//! - `N\r` - Serial number query

use heapless::Vec;

/// Maximum CAN data length
pub const MAX_DATA_LEN: usize = 8;

/// Parsed CAN frame
#[derive(Clone, Debug)]
pub struct CanFrame {
    /// CAN ID (11-bit or 29-bit)
    pub id: u32,
    /// Extended ID flag
    pub extended: bool,
    /// Remote transmission request
    pub rtr: bool,
    /// Data bytes
    pub data: Vec<u8, MAX_DATA_LEN>,
}

impl CanFrame {
    /// Create a new standard frame
    pub fn new(id: u16, data: &[u8]) -> Self {
        let mut frame_data = Vec::new();
        for &b in data.iter().take(MAX_DATA_LEN) {
            let _ = frame_data.push(b);
        }
        Self {
            id: id as u32,
            extended: false,
            rtr: false,
            data: frame_data,
        }
    }

    /// Format frame as SLCAN string (without trailing \r)
    pub fn to_slcan(&self, buf: &mut heapless::String<32>) {
        buf.clear();

        if self.extended {
            if self.rtr {
                let _ = buf.push('R');
            } else {
                let _ = buf.push('T');
            }
            // 8 hex chars for 29-bit ID
            let _ = core::fmt::write(buf, format_args!("{:08X}", self.id));
        } else {
            if self.rtr {
                let _ = buf.push('r');
            } else {
                let _ = buf.push('t');
            }
            // 3 hex chars for 11-bit ID
            let _ = core::fmt::write(buf, format_args!("{:03X}", self.id));
        }

        // Data length
        let _ = core::fmt::write(buf, format_args!("{}", self.data.len()));

        // Data bytes (if not RTR)
        if !self.rtr {
            for &b in self.data.iter() {
                let _ = core::fmt::write(buf, format_args!("{:02X}", b));
            }
        }
    }
}

/// SLCAN command result
#[derive(Debug)]
pub enum SlcanResult {
    /// Parsed CAN frame to process
    Frame(CanFrame),
    /// Open channel command
    Open,
    /// Close channel command
    Close,
    /// Set bitrate (index 0-8)
    SetBitrate(u8),
    /// Version query
    Version,
    /// Serial number query
    SerialNumber,
    /// Unknown/unsupported command
    Unknown,
    /// Parse error
    Error,
    /// Empty/incomplete input
    Empty,
}

/// Parse an SLCAN command string (without trailing \r)
pub fn parse(input: &str) -> SlcanResult {
    let input = input.trim();
    if input.is_empty() {
        return SlcanResult::Empty;
    }

    let first = input.chars().next().unwrap();
    let rest = &input[1..];

    match first {
        // Standard frame
        't' => parse_standard_frame(rest, false),
        // Extended frame
        'T' => parse_extended_frame(rest, false),
        // Standard RTR
        'r' => parse_standard_frame(rest, true),
        // Extended RTR
        'R' => parse_extended_frame(rest, true),
        // Open channel
        'O' => SlcanResult::Open,
        // Close channel
        'C' => SlcanResult::Close,
        // Set bitrate
        'S' => {
            if let Some(c) = rest.chars().next() {
                if let Some(n) = c.to_digit(10) {
                    return SlcanResult::SetBitrate(n as u8);
                }
            }
            SlcanResult::Error
        }
        // Version
        'V' | 'v' => SlcanResult::Version,
        // Serial number
        'N' | 'n' => SlcanResult::SerialNumber,
        // Timestamp toggle (ignored but acknowledged)
        'Z' => SlcanResult::Unknown,
        _ => SlcanResult::Unknown,
    }
}

fn parse_standard_frame(input: &str, rtr: bool) -> SlcanResult {
    // Need at least 4 chars: 3 for ID + 1 for length
    if input.len() < 4 {
        return SlcanResult::Error;
    }

    // Parse 11-bit ID (3 hex chars)
    let id = match u16::from_str_radix(&input[0..3], 16) {
        Ok(id) if id <= 0x7FF => id as u32,
        _ => return SlcanResult::Error,
    };

    // Parse length
    let len = match input.chars().nth(3).and_then(|c| c.to_digit(10)) {
        Some(l) if l <= 8 => l as usize,
        _ => return SlcanResult::Error,
    };

    // Parse data bytes
    let mut data: Vec<u8, MAX_DATA_LEN> = Vec::new();
    if !rtr {
        let data_str = &input[4..];
        if data_str.len() < len * 2 {
            return SlcanResult::Error;
        }
        for i in 0..len {
            let byte_str = &data_str[i*2..i*2+2];
            match u8::from_str_radix(byte_str, 16) {
                Ok(b) => { let _ = data.push(b); }
                Err(_) => return SlcanResult::Error,
            }
        }
    }

    SlcanResult::Frame(CanFrame {
        id,
        extended: false,
        rtr,
        data,
    })
}

fn parse_extended_frame(input: &str, rtr: bool) -> SlcanResult {
    // Need at least 9 chars: 8 for ID + 1 for length
    if input.len() < 9 {
        return SlcanResult::Error;
    }

    // Parse 29-bit ID (8 hex chars)
    let id = match u32::from_str_radix(&input[0..8], 16) {
        Ok(id) if id <= 0x1FFFFFFF => id,
        _ => return SlcanResult::Error,
    };

    // Parse length
    let len = match input.chars().nth(8).and_then(|c| c.to_digit(10)) {
        Some(l) if l <= 8 => l as usize,
        _ => return SlcanResult::Error,
    };

    // Parse data bytes
    let mut data: Vec<u8, MAX_DATA_LEN> = Vec::new();
    if !rtr {
        let data_str = &input[9..];
        if data_str.len() < len * 2 {
            return SlcanResult::Error;
        }
        for i in 0..len {
            let byte_str = &data_str[i*2..i*2+2];
            match u8::from_str_radix(byte_str, 16) {
                Ok(b) => { let _ = data.push(b); }
                Err(_) => return SlcanResult::Error,
            }
        }
    }

    SlcanResult::Frame(CanFrame {
        id,
        extended: false,
        rtr,
        data,
    })
}

/// Send a CAN frame as SLCAN over serial
pub fn send_frame<W: core::fmt::Write>(writer: &mut W, frame: &CanFrame) {
    let mut buf: heapless::String<32> = heapless::String::new();
    frame.to_slcan(&mut buf);
    let _ = writer.write_str(&buf);
    let _ = writer.write_char('\r');
}

/// Send SLCAN OK response
pub fn send_ok<W: core::fmt::Write>(writer: &mut W) {
    let _ = writer.write_char('\r');
}

/// Send SLCAN error response
pub fn send_error<W: core::fmt::Write>(writer: &mut W) {
    let _ = writer.write_char('\x07'); // BEL
}

/// Send version response
pub fn send_version<W: core::fmt::Write>(writer: &mut W, version: &str) {
    let _ = writer.write_char('V');
    let _ = writer.write_str(version);
    let _ = writer.write_char('\r');
}

/// Send serial number response
pub fn send_serial<W: core::fmt::Write>(writer: &mut W, serial: &str) {
    let _ = writer.write_char('N');
    let _ = writer.write_str(serial);
    let _ = writer.write_char('\r');
}
