//! Costmap serialization for WebRTC streaming.
//!
//! Serializes costmap snapshots into a compact binary format with RLE encoding.
//! The 200×200 grid is mostly FREE (0), so RLE compresses ~40KB → ~2-5KB.
//!
//! # Binary format
//!
//! ```text
//! Header (18 bytes):
//!   [0]      u8    MSG_COSTMAP (0x22)
//!   [1]      u8    encoding (0=raw, 1=RLE)
//!   [2-3]    u16   width (cells, LE)
//!   [4-5]    u16   height (cells, LE)
//!   [6-9]    f32   resolution (m, LE)
//!   [10-13]  f32   origin_x (m, LE)
//!   [14-17]  f32   origin_y (m, LE)
//!   [18...]  cell data (raw or RLE encoded)
//! ```
//!
//! RLE encoding: `[count:u8][value:u8]` pairs. Max run length 255.

use crate::MSG_COSTMAP;
use costmap::CostmapSnapshot;

/// Header size in bytes.
const HEADER_SIZE: usize = 18;

/// Encoding types.
const ENCODING_RAW: u8 = 0;
const ENCODING_RLE: u8 = 1;

/// Serialize a costmap snapshot to binary with RLE encoding.
pub fn serialize(snapshot: &CostmapSnapshot) -> Vec<u8> {
    let rle = rle_encode(&snapshot.cells);
    let raw_size = snapshot.cells.len();

    // Use RLE if it's actually smaller
    let (encoding, data) = if rle.len() < raw_size {
        (ENCODING_RLE, rle)
    } else {
        (ENCODING_RAW, snapshot.cells.clone())
    };

    let mut buf = Vec::with_capacity(HEADER_SIZE + data.len());

    // Header
    buf.push(MSG_COSTMAP);
    buf.push(encoding);
    buf.extend_from_slice(&snapshot.width.to_le_bytes());
    buf.extend_from_slice(&snapshot.height.to_le_bytes());
    buf.extend_from_slice(&snapshot.resolution.to_le_bytes());
    buf.extend_from_slice(&snapshot.origin_x.to_le_bytes());
    buf.extend_from_slice(&snapshot.origin_y.to_le_bytes());

    debug_assert_eq!(buf.len(), HEADER_SIZE);

    // Cell data
    buf.extend_from_slice(&data);

    buf
}

/// RLE encode: produces `[count:u8][value:u8]` pairs.
fn rle_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    // Worst case: every cell changes value → 2x size
    let mut out = Vec::with_capacity(data.len());
    let mut current = data[0];
    let mut count: u8 = 1;

    for &val in &data[1..] {
        if val == current && count < 255 {
            count += 1;
        } else {
            out.push(count);
            out.push(current);
            current = val;
            count = 1;
        }
    }
    // Flush last run
    out.push(count);
    out.push(current);

    out
}

/// RLE decode: expands `[count:u8][value:u8]` pairs back to flat data.
pub fn rle_decode(encoded: &[u8], expected_len: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(expected_len);

    let mut i = 0;
    while i + 1 < encoded.len() {
        let count = encoded[i] as usize;
        let value = encoded[i + 1];
        for _ in 0..count {
            out.push(value);
        }
        i += 2;
    }

    if out.len() == expected_len {
        Some(out)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(width: u16, height: u16, cells: Vec<u8>) -> CostmapSnapshot {
        CostmapSnapshot {
            cells,
            width,
            height,
            resolution: 0.1,
            origin_x: -10.0,
            origin_y: -10.0,
        }
    }

    #[test]
    fn test_rle_roundtrip_uniform() {
        // All zeros (typical free space)
        let data = vec![0u8; 200 * 200];
        let encoded = rle_encode(&data);
        // Should compress significantly (40000 bytes → ~314 bytes for 157 runs of 255 + 1 run of 55)
        assert!(encoded.len() < 400);
        let decoded = rle_decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_roundtrip_mixed() {
        let mut data = vec![0u8; 100];
        // Add some obstacles
        data[50] = 254;
        data[51] = 254;
        data[52] = 253;
        data[80] = 254;
        let encoded = rle_encode(&data);
        let decoded = rle_decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_roundtrip_alternating() {
        // Worst case: alternating values
        let data: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0 } else { 254 }).collect();
        let encoded = rle_encode(&data);
        let decoded = rle_decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_long_run() {
        // Run longer than 255 — should split into multiple runs
        let data = vec![42u8; 300];
        let encoded = rle_encode(&data);
        let decoded = rle_decode(&encoded, data.len()).unwrap();
        assert_eq!(decoded, data);
        // Should be: [255, 42, 45, 42] = 4 bytes
        assert_eq!(encoded.len(), 4);
    }

    #[test]
    fn test_serialize_header() {
        let snapshot = make_snapshot(200, 200, vec![0u8; 200 * 200]);
        let buf = serialize(&snapshot);

        assert_eq!(buf[0], MSG_COSTMAP);
        // Should use RLE (much smaller than raw)
        assert_eq!(buf[1], ENCODING_RLE);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 200);
        assert_eq!(u16::from_le_bytes([buf[4], buf[5]]), 200);
        let resolution = f32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]);
        assert!((resolution - 0.1).abs() < 0.001);
        let origin_x = f32::from_le_bytes([buf[10], buf[11], buf[12], buf[13]]);
        assert!((origin_x - (-10.0)).abs() < 0.001);
        let origin_y = f32::from_le_bytes([buf[14], buf[15], buf[16], buf[17]]);
        assert!((origin_y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_serialize_compression_ratio() {
        // Typical costmap: mostly free with some obstacles
        let mut cells = vec![0u8; 200 * 200];
        // Add a line of obstacles
        for i in 0..200 {
            cells[100 * 200 + i] = 254;
        }
        let snapshot = make_snapshot(200, 200, cells);
        let buf = serialize(&snapshot);

        // Should be much smaller than raw 40KB
        assert!(buf.len() < 2000, "Serialized size {} is too large", buf.len());
    }

    #[test]
    fn test_rle_decode_wrong_length() {
        let encoded = vec![5, 0]; // 5 zeros
        assert!(rle_decode(&encoded, 5).is_some());
        assert!(rle_decode(&encoded, 10).is_none());
    }
}
