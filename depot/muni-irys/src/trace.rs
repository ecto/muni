//! Read ipse's K1 bus traces (`ipse-bus-trace`, v1–v3) into poses.
//!
//! The layout is ipse's (`crates/ipse-deploy/src/trace.rs`): one JSON header
//! line, then `u8 kind | u64 recv_ns | u32 len | payload` records, all LE.
//! Only `low_state` (IMU rpy + serial joint q) and `odometer` (x, y, θ) are
//! read; everything else is skipped by length. Re-implemented here rather
//! than depended on so muni does not pull ipse's robot SDK bindings.

use crate::Error;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// The K1's serial-bus joint order (ipse `K1_SERIAL_JOINTS`), URDF names.
pub const K1_SERIAL_JOINTS: [&str; 22] = [
    "AAHead_yaw",
    "Head_pitch",
    "ALeft_Shoulder_Pitch",
    "Left_Shoulder_Roll",
    "Left_Elbow_Pitch",
    "Left_Elbow_Yaw",
    "ARight_Shoulder_Pitch",
    "Right_Shoulder_Roll",
    "Right_Elbow_Pitch",
    "Right_Elbow_Yaw",
    "Left_Hip_Pitch",
    "Left_Hip_Roll",
    "Left_Hip_Yaw",
    "Left_Knee_Pitch",
    "Left_Ankle_Pitch",
    "Left_Ankle_Roll",
    "Right_Hip_Pitch",
    "Right_Hip_Roll",
    "Right_Hip_Yaw",
    "Right_Knee_Pitch",
    "Right_Ankle_Pitch",
    "Right_Ankle_Roll",
];

const MOTOR_BYTES: usize = 22;

/// One pose sample: everything a render needs.
#[derive(Debug, Clone, Default)]
pub struct PoseSample {
    /// Seconds since the trace started.
    pub t: f64,
    /// Trunk roll, pitch, yaw from the IMU (rad).
    pub rpy: [f64; 3],
    /// Odometer x, y, θ (m, m, rad), the latest seen at this sample.
    pub odom: [f64; 3],
    /// Joint angles by URDF name.
    pub q: HashMap<String, f64>,
}

/// Read every `low_state` record of a trace, newest odometer attached.
pub fn read(path: &Path) -> Result<Vec<PoseSample>, Error> {
    let f = std::fs::File::open(path).map_err(|e| Error::Io(path.into(), e))?;
    let mut r = BufReader::new(f);
    let mut header = String::new();
    r.read_line(&mut header).map_err(|e| Error::Io(path.into(), e))?;
    let h: serde_json::Value =
        serde_json::from_str(&header).map_err(|e| Error::Trace(format!("header: {e}")))?;
    if h["format"] != "ipse-bus-trace" {
        return Err(Error::Trace("not an ipse-bus-trace".into()));
    }
    let mut out = Vec::new();
    let mut odom = [0.0; 3];
    let mut head = [0u8; 13];
    let mut payload = Vec::new();
    loop {
        match r.read_exact(&mut head) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::Io(path.into(), e)),
        }
        let kind = head[0];
        let ns = u64::from_le_bytes(head[1..9].try_into().unwrap());
        let len = u32::from_le_bytes(head[9..13].try_into().unwrap()) as usize;
        payload.resize(len, 0);
        if r.read_exact(&mut payload).is_err() {
            break; // a capture cut mid-record
        }
        let f = |o: usize| f32::from_le_bytes(payload[o..o + 4].try_into().unwrap()) as f64;
        match kind {
            2 if len >= 12 => odom = [f(0), f(4), f(8)],
            1 if len >= 44 => {
                let u = |o: usize| u32::from_le_bytes(payload[o..o + 4].try_into().unwrap()) as usize;
                let (np, ns_) = (u(36), u(40));
                if len < 44 + (np + ns_) * MOTOR_BYTES {
                    continue;
                }
                let base = 44 + np * MOTOR_BYTES;
                let q = K1_SERIAL_JOINTS
                    .iter()
                    .take(ns_)
                    .enumerate()
                    .map(|(i, n)| (n.to_string(), f(base + i * MOTOR_BYTES)))
                    .collect();
                out.push(PoseSample { t: ns as f64 * 1e-9, rpy: [f(0), f(4), f(8)], odom, q });
            }
            _ => {}
        }
    }
    Ok(out)
}
