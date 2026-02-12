//! VESC UART serial gateway — talks to VESCs over USB serial using the VESC
//! binary protocol. The USB-connected VESC acts as a CAN gateway, forwarding
//! commands to other VESCs via `COMM_FORWARD_CAN`.
//!
//! Only the gateway VESC returns telemetry (COMM_FORWARD_CAN doesn't relay
//! responses back over UART). This is acceptable — battery voltage from one
//! VESC is representative.

use can::Frame;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, trace, warn};

/// VESC UART command bytes.
const COMM_GET_VALUES: u8 = 4;
const COMM_SET_DUTY: u8 = 5;
const COMM_SET_CURRENT: u8 = 6;
const COMM_SET_CURRENT_BRAKE: u8 = 7;
const COMM_SET_RPM: u8 = 8;
const COMM_FORWARD_CAN: u8 = 34;

/// VESC UART packet start/end bytes.
const PACKET_START_SHORT: u8 = 0x02;
const PACKET_END: u8 = 0x03;

/// CAN command IDs (from can::vesc::CommandId).
const CAN_CMD_SET_DUTY: u8 = 0;
const CAN_CMD_SET_CURRENT: u8 = 1;
const CAN_CMD_SET_CURRENT_BRAKE: u8 = 2;
const CAN_CMD_SET_RPM: u8 = 3;

/// CAN status command IDs.
const CAN_CMD_STATUS1: u8 = 9;
const CAN_CMD_STATUS4: u8 = 16;
const CAN_CMD_STATUS5: u8 = 27;

/// VESC serial gateway handle.
pub(crate) struct VescSerial {
    /// Serial port writer (shared with background tasks).
    writer: Arc<tokio::sync::Mutex<tokio::io::WriteHalf<tokio_serial::SerialStream>>>,
    /// Incoming CAN frames synthesized from UART responses.
    rx_queue: Arc<Mutex<VecDeque<Frame>>>,
    /// Gateway VESC ID (the one connected via USB).
    gateway_id: u8,
    /// Tokio runtime handle for spawning writes from the control loop thread.
    rt: tokio::runtime::Handle,
}

impl VescSerial {
    /// Open the serial port and spawn background reader + poller tasks.
    pub(crate) async fn open(
        port_path: &str,
        gateway_id: u8,
    ) -> anyhow::Result<Self> {
        use tokio_serial::SerialPortBuilderExt;

        let serial = tokio_serial::new(port_path, 115200)
            .open_native_async()
            .map_err(|e| anyhow::anyhow!("Failed to open serial port {}: {}", port_path, e))?;

        let (reader, writer) = tokio::io::split(serial);
        let writer = Arc::new(tokio::sync::Mutex::new(writer));
        let rx_queue = Arc::new(Mutex::new(VecDeque::<Frame>::new()));

        // Spawn background reader task
        let rx_q = rx_queue.clone();
        let gw_id = gateway_id;
        tokio::spawn(async move {
            reader_task(reader, rx_q, gw_id).await;
        });

        // Spawn background poller task (~20 Hz COMM_GET_VALUES to gateway)
        let poll_writer = writer.clone();
        let poll_gw_id = gateway_id;
        tokio::spawn(async move {
            poller_task(poll_writer, poll_gw_id).await;
        });

        Ok(Self {
            writer,
            rx_queue,
            gateway_id,
            rt: tokio::runtime::Handle::current(),
        })
    }

    /// Translate a CAN frame into a VESC UART packet and send it.
    ///
    /// LED/MCU frames (0x0B00+ range) are silently dropped.
    pub(crate) fn send_frame(&self, frame: &Frame) {
        // Drop non-VESC frames (LED controller, MCU, etc.)
        if frame.id >= 0x0B00 {
            trace!(id = frame.id, "Dropping non-VESC frame");
            return;
        }

        let target_id = (frame.id & 0xFF) as u8;
        let can_cmd = ((frame.id >> 8) & 0xFF) as u8;

        let uart_cmd = match can_cmd {
            CAN_CMD_SET_DUTY => COMM_SET_DUTY,
            CAN_CMD_SET_CURRENT => COMM_SET_CURRENT,
            CAN_CMD_SET_CURRENT_BRAKE => COMM_SET_CURRENT_BRAKE,
            CAN_CMD_SET_RPM => COMM_SET_RPM,
            _ => {
                trace!(can_cmd, target_id, "Unsupported CAN command for serial");
                return;
            }
        };

        // Build payload: gateway gets direct command, others get COMM_FORWARD_CAN
        let payload = if target_id == self.gateway_id {
            let mut p = Vec::with_capacity(1 + frame.data.len());
            p.push(uart_cmd);
            p.extend_from_slice(&frame.data);
            p
        } else {
            let mut p = Vec::with_capacity(3 + frame.data.len());
            p.push(COMM_FORWARD_CAN);
            p.push(target_id);
            p.push(uart_cmd);
            p.extend_from_slice(&frame.data);
            p
        };

        let packet = build_packet(&payload);
        let writer = self.writer.clone();
        self.rt.spawn(async move {
            let mut w = writer.lock().await;
            if let Err(e) = w.write_all(&packet).await {
                warn!(?e, "VESC serial write error");
            }
        });
    }

    /// Pop next received CAN frame (synthesized from UART telemetry).
    pub(crate) fn recv(&self) -> Option<Frame> {
        self.rx_queue
            .lock()
            .expect("vesc serial rx_queue mutex poisoned")
            .pop_front()
    }
}

// ---------------------------------------------------------------------------
// Background tasks
// ---------------------------------------------------------------------------

/// Reads UART packets from serial, parses COMM_GET_VALUES responses, and
/// pushes synthesized CAN STATUS frames into the rx_queue.
async fn reader_task(
    mut reader: tokio::io::ReadHalf<tokio_serial::SerialStream>,
    rx_queue: Arc<Mutex<VecDeque<Frame>>>,
    gateway_id: u8,
) {
    let mut buf = [0u8; 256];
    let mut pending = Vec::new();

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                warn!("VESC serial port closed");
                break;
            }
            Ok(n) => {
                pending.extend_from_slice(&buf[..n]);

                // Try to parse complete packets from pending buffer
                while let Some((payload, consumed)) = parse_packet(&pending) {
                    if !payload.is_empty() {
                        process_uart_response(&payload, gateway_id, &rx_queue);
                    }
                    pending.drain(..consumed);
                }

                // Prevent unbounded growth from garbage data.
                // GET_VALUES responses are ~70 bytes; at 20 Hz polling we
                // can legitimately accumulate a few hundred bytes between
                // reads, so use a generous limit.
                if pending.len() > 4096 {
                    warn!("VESC serial buffer overflow, clearing");
                    pending.clear();
                }
            }
            Err(e) => {
                error!(?e, "VESC serial read error");
                break;
            }
        }
    }
}

/// Periodically sends COMM_GET_VALUES to the gateway VESC at ~20 Hz.
async fn poller_task(
    writer: Arc<tokio::sync::Mutex<tokio::io::WriteHalf<tokio_serial::SerialStream>>>,
    _gateway_id: u8,
) {
    let packet = build_packet(&[COMM_GET_VALUES]);
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));

    loop {
        interval.tick().await;
        let mut w = writer.lock().await;
        if let Err(e) = w.write_all(&packet).await {
            warn!(?e, "VESC serial poll write error");
            // Brief backoff on error
            drop(w);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

// ---------------------------------------------------------------------------
// VESC UART protocol helpers
// ---------------------------------------------------------------------------

/// Parse a COMM_GET_VALUES response and push synthesized CAN STATUS frames.
///
/// The COMM_GET_VALUES response is ~70 bytes. We extract key fields at known
/// offsets to synthesize STATUS1, STATUS4, and STATUS5 frames that
/// `Drivetrain::process_frame()` already understands.
///
/// COMM_GET_VALUES response layout (VESC firmware 6.x):
///   [0]     = COMM_GET_VALUES (4)
///   [1..3]  = temp_fet (i16, ×10)
///   [3..5]  = temp_motor (i16, ×10)
///   [5..9]  = avg_motor_current (i32, ×100)
///   [9..13] = avg_input_current (i32, ×100)
///   [13..17] = avg_id (i32, ×100)
///   [17..21] = avg_iq (i32, ×100)
///   [21..23] = duty_cycle_now (i16, ×1000)
///   [23..27] = rpm (i32)
///   [27..29] = v_in (i16, ×10)
///   [29..33] = amp_hours (i32, ×10000)
///   [33..37] = amp_hours_charged (i32, ×10000)
///   [37..41] = watt_hours (i32, ×10000)
///   [41..45] = watt_hours_charged (i32, ×10000)
///   [45..49] = tachometer (i32)
///   [49..53] = tachometer_abs (i32)
///   ... (fault code, pid pos, etc.)
fn process_uart_response(
    payload: &[u8],
    gateway_id: u8,
    rx_queue: &Arc<Mutex<VecDeque<Frame>>>,
) {
    if payload.is_empty() {
        return;
    }

    let cmd = payload[0];
    if cmd != COMM_GET_VALUES {
        trace!(cmd, "Ignoring non-GET_VALUES response");
        return;
    }

    // Need at least 53 bytes for all the fields we parse
    if payload.len() < 53 {
        debug!(len = payload.len(), "COMM_GET_VALUES response too short");
        return;
    }

    let data = &payload[1..]; // Skip command byte

    // Extract fields
    let temp_fet_raw = i16::from_be_bytes([data[0], data[1]]);
    let temp_motor_raw = i16::from_be_bytes([data[2], data[3]]);
    let avg_motor_current_raw = i32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let avg_input_current_raw = i32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let duty_raw = i16::from_be_bytes([data[20], data[21]]);
    let rpm = i32::from_be_bytes([data[22], data[23], data[24], data[25]]);
    let v_in_raw = i16::from_be_bytes([data[26], data[27]]);
    let tachometer = i32::from_be_bytes([data[44], data[45], data[46], data[47]]);

    // Synthesize STATUS1: ERPM(i32 BE) + current(i16 BE, ×10) + duty(i16 BE, ×1000)
    let current_10 = (avg_motor_current_raw / 100) as i16; // ×100 → ×10
    let status1_data = [
        rpm.to_be_bytes()[0],
        rpm.to_be_bytes()[1],
        rpm.to_be_bytes()[2],
        rpm.to_be_bytes()[3],
        current_10.to_be_bytes()[0],
        current_10.to_be_bytes()[1],
        duty_raw.to_be_bytes()[0],
        duty_raw.to_be_bytes()[1],
    ];
    let status1 = Frame::new_extended(
        (CAN_CMD_STATUS1 as u32) << 8 | gateway_id as u32,
        &status1_data,
    );

    // Synthesize STATUS4: temp_fet(i16 BE, ×10) + temp_motor(i16 BE, ×10) + current_in(i16 BE, ×10)
    let current_in_10 = (avg_input_current_raw / 100) as i16; // ×100 → ×10
    let status4_data = [
        temp_fet_raw.to_be_bytes()[0],
        temp_fet_raw.to_be_bytes()[1],
        temp_motor_raw.to_be_bytes()[0],
        temp_motor_raw.to_be_bytes()[1],
        current_in_10.to_be_bytes()[0],
        current_in_10.to_be_bytes()[1],
        0,
        0,
    ];
    let status4 = Frame::new_extended(
        (CAN_CMD_STATUS4 as u32) << 8 | gateway_id as u32,
        &status4_data,
    );

    // Synthesize STATUS5: tachometer(i32 BE) + voltage_in(i16 BE, ×10)
    let status5_data = [
        tachometer.to_be_bytes()[0],
        tachometer.to_be_bytes()[1],
        tachometer.to_be_bytes()[2],
        tachometer.to_be_bytes()[3],
        v_in_raw.to_be_bytes()[0],
        v_in_raw.to_be_bytes()[1],
        0,
        0,
    ];
    let status5 = Frame::new_extended(
        (CAN_CMD_STATUS5 as u32) << 8 | gateway_id as u32,
        &status5_data,
    );

    // Push all three status frames
    let mut queue = rx_queue.lock().expect("vesc serial rx_queue mutex poisoned");
    queue.push_back(status1);
    queue.push_back(status4);
    queue.push_back(status5);
}

/// Build a VESC UART packet: `0x02 | len | payload | crc16_hi | crc16_lo | 0x03`
///
/// Uses short format (1-byte length) for payloads ≤ 255 bytes.
fn build_packet(payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() <= 255, "VESC short packet max payload is 255");
    let crc = crc16(payload);
    let mut pkt = Vec::with_capacity(4 + payload.len());
    pkt.push(PACKET_START_SHORT);
    pkt.push(payload.len() as u8);
    pkt.extend_from_slice(payload);
    pkt.push((crc >> 8) as u8);
    pkt.push((crc & 0xFF) as u8);
    pkt.push(PACKET_END);
    pkt
}

/// Try to parse a complete VESC UART packet from the buffer.
///
/// Returns `Some((payload, bytes_consumed))` if a complete packet is found,
/// or `None` if more data is needed.
fn parse_packet(buf: &[u8]) -> Option<(Vec<u8>, usize)> {
    if buf.is_empty() {
        return None;
    }

    // Scan for 0x02 (short packet start). We intentionally ignore 0x03
    // (long packet start) because it collides with the end-of-packet byte,
    // and all VESC responses we care about fit in short packets (≤255 bytes).
    let start_pos = buf.iter().position(|&b| b == PACKET_START_SHORT)?;

    // Skip garbage bytes before start
    if start_pos > 0 {
        return Some((Vec::new(), start_pos));
    }

    // Short packet: 0x02 | len(1) | payload(len) | crc(2) | 0x03
    if buf.len() < 2 {
        return None;
    }
    let len = buf[1] as usize;
    if len == 0 || len > 255 {
        // Invalid length — skip this 0x02
        return Some((Vec::new(), 1));
    }
    let total = 2 + len + 3; // start + len_byte + payload + crc(2) + end
    if buf.len() < total {
        return None;
    }
    let payload = &buf[2..2 + len];
    let crc_recv = ((buf[2 + len] as u16) << 8) | buf[2 + len + 1] as u16;
    let crc_calc = crc16(payload);
    if crc_recv != crc_calc {
        debug!(
            expected = crc_calc,
            received = crc_recv,
            "VESC packet CRC mismatch"
        );
        // Skip this start byte and try again
        return Some((Vec::new(), 1));
    }
    if buf[total - 1] != PACKET_END {
        return Some((Vec::new(), 1));
    }
    Some((payload.to_vec(), total))
}

/// CRC16-CCITT (poly 0x1021, init 0x0000) — standard VESC CRC.
fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_empty() {
        assert_eq!(crc16(&[]), 0x0000);
    }

    #[test]
    fn test_crc16_known_vector() {
        // "123456789" → CRC16-CCITT (XModem, init=0) = 0x31C3
        assert_eq!(crc16(b"123456789"), 0x31C3);
    }

    #[test]
    fn test_build_packet_short() {
        let payload = [COMM_GET_VALUES]; // cmd=4
        let packet = build_packet(&payload);
        assert_eq!(packet[0], PACKET_START_SHORT);
        assert_eq!(packet[1], 1); // length
        assert_eq!(packet[2], COMM_GET_VALUES);
        // CRC of [4]
        let crc = crc16(&payload);
        assert_eq!(packet[3], (crc >> 8) as u8);
        assert_eq!(packet[4], (crc & 0xFF) as u8);
        assert_eq!(packet[5], PACKET_END);
    }

    #[test]
    fn test_parse_packet_roundtrip() {
        let payload = vec![COMM_GET_VALUES, 0x01, 0x02, 0x03];
        let packet = build_packet(&payload);
        let (parsed, consumed) = parse_packet(&packet).unwrap();
        assert_eq!(parsed, payload);
        assert_eq!(consumed, packet.len());
    }

    #[test]
    fn test_parse_packet_with_leading_garbage() {
        let payload = vec![COMM_SET_RPM, 0x00, 0x00, 0x10, 0x00];
        let mut data = vec![0xFF, 0xFE]; // garbage
        data.extend_from_slice(&build_packet(&payload));

        // First call skips garbage
        let (p1, c1) = parse_packet(&data).unwrap();
        assert!(p1.is_empty());
        assert_eq!(c1, 2);

        // Second call parses the packet
        let (p2, _c2) = parse_packet(&data[c1..]).unwrap();
        assert_eq!(p2, payload);
    }

    #[test]
    fn test_parse_packet_incomplete() {
        let payload = vec![COMM_GET_VALUES];
        let packet = build_packet(&payload);
        // Truncate
        assert!(parse_packet(&packet[..3]).is_none());
    }

    #[test]
    fn test_can_to_uart_cmd_mapping() {
        // Verify our CAN command → UART command mapping
        assert_eq!(CAN_CMD_SET_DUTY, 0);
        assert_eq!(CAN_CMD_SET_CURRENT, 1);
        assert_eq!(CAN_CMD_SET_CURRENT_BRAKE, 2);
        assert_eq!(CAN_CMD_SET_RPM, 3);

        // UART commands
        assert_eq!(COMM_SET_DUTY, 5);
        assert_eq!(COMM_SET_CURRENT, 6);
        assert_eq!(COMM_SET_CURRENT_BRAKE, 7);
        assert_eq!(COMM_SET_RPM, 8);
    }

    #[test]
    fn test_process_uart_response_get_values() {
        let gateway_id: u8 = 1;
        let rx_queue = Arc::new(Mutex::new(VecDeque::<Frame>::new()));

        // Build a minimal COMM_GET_VALUES response (53 bytes: cmd + 52 data)
        let mut payload = vec![0u8; 53];
        payload[0] = COMM_GET_VALUES;

        // temp_fet = 350 (35.0°C) at bytes [1..3]
        let temp_fet: i16 = 350;
        payload[1] = temp_fet.to_be_bytes()[0];
        payload[2] = temp_fet.to_be_bytes()[1];

        // temp_motor = 400 (40.0°C) at bytes [3..5]
        let temp_motor: i16 = 400;
        payload[3] = temp_motor.to_be_bytes()[0];
        payload[4] = temp_motor.to_be_bytes()[1];

        // avg_motor_current = 50000 (500.00A... well, ×100 so 500A) at bytes [5..9]
        let motor_current: i32 = 1500; // 15.00A
        payload[5] = motor_current.to_be_bytes()[0];
        payload[6] = motor_current.to_be_bytes()[1];
        payload[7] = motor_current.to_be_bytes()[2];
        payload[8] = motor_current.to_be_bytes()[3];

        // avg_input_current = 1200 (12.00A) at bytes [9..13]
        let input_current: i32 = 1200;
        payload[9] = input_current.to_be_bytes()[0];
        payload[10] = input_current.to_be_bytes()[1];
        payload[11] = input_current.to_be_bytes()[2];
        payload[12] = input_current.to_be_bytes()[3];

        // duty = 500 (0.500) at bytes [21..23]
        let duty: i16 = 500;
        payload[21] = duty.to_be_bytes()[0];
        payload[22] = duty.to_be_bytes()[1];

        // rpm = 3000 at bytes [23..27]
        let rpm: i32 = 3000;
        payload[23] = rpm.to_be_bytes()[0];
        payload[24] = rpm.to_be_bytes()[1];
        payload[25] = rpm.to_be_bytes()[2];
        payload[26] = rpm.to_be_bytes()[3];

        // v_in = 480 (48.0V) at bytes [27..29]
        let v_in: i16 = 480;
        payload[27] = v_in.to_be_bytes()[0];
        payload[28] = v_in.to_be_bytes()[1];

        // tachometer = 12345 at bytes [45..49]
        let tach: i32 = 12345;
        payload[45] = tach.to_be_bytes()[0];
        payload[46] = tach.to_be_bytes()[1];
        payload[47] = tach.to_be_bytes()[2];
        payload[48] = tach.to_be_bytes()[3];

        process_uart_response(&payload, gateway_id, &rx_queue);

        let queue = rx_queue.lock().unwrap();
        assert_eq!(queue.len(), 3, "Should have STATUS1 + STATUS4 + STATUS5");

        // STATUS1
        let s1 = &queue[0];
        assert_eq!(s1.id, (CAN_CMD_STATUS1 as u32) << 8 | 1);
        assert!(s1.extended);
        let erpm = i32::from_be_bytes([s1.data[0], s1.data[1], s1.data[2], s1.data[3]]);
        assert_eq!(erpm, 3000);
        let current_10 = i16::from_be_bytes([s1.data[4], s1.data[5]]);
        assert_eq!(current_10, 15); // 1500/100 = 15 (representing 1.5A in ×10 format)
        let duty_1000 = i16::from_be_bytes([s1.data[6], s1.data[7]]);
        assert_eq!(duty_1000, 500);

        // STATUS4
        let s4 = &queue[1];
        assert_eq!(s4.id, (CAN_CMD_STATUS4 as u32) << 8 | 1);
        let fet_temp = i16::from_be_bytes([s4.data[0], s4.data[1]]);
        assert_eq!(fet_temp, 350);
        let motor_temp = i16::from_be_bytes([s4.data[2], s4.data[3]]);
        assert_eq!(motor_temp, 400);
        let cur_in = i16::from_be_bytes([s4.data[4], s4.data[5]]);
        assert_eq!(cur_in, 12); // 1200/100 = 12

        // STATUS5
        let s5 = &queue[2];
        assert_eq!(s5.id, (CAN_CMD_STATUS5 as u32) << 8 | 1);
        let tach_out = i32::from_be_bytes([s5.data[0], s5.data[1], s5.data[2], s5.data[3]]);
        assert_eq!(tach_out, 12345);
        let v_in_out = i16::from_be_bytes([s5.data[4], s5.data[5]]);
        assert_eq!(v_in_out, 480);
    }

    #[test]
    fn test_process_uart_response_too_short() {
        let rx_queue = Arc::new(Mutex::new(VecDeque::<Frame>::new()));
        // Payload too short
        process_uart_response(&[COMM_GET_VALUES, 0, 0], 1, &rx_queue);
        assert!(rx_queue.lock().unwrap().is_empty());
    }

    #[test]
    fn test_process_uart_response_wrong_cmd() {
        let rx_queue = Arc::new(Mutex::new(VecDeque::<Frame>::new()));
        let payload = vec![0xFF; 53]; // Wrong command byte
        process_uart_response(&payload, 1, &rx_queue);
        assert!(rx_queue.lock().unwrap().is_empty());
    }
}
