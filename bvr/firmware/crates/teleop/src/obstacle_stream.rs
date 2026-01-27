//! Obstacle list serialization for WebRTC streaming.
//!
//! Serializes tracked obstacles into a compact binary format.
//!
//! # Binary format (version 3)
//!
//! ```text
//! Header (4 bytes):
//!   [0]      u8    MSG_OBSTACLES (0x23)
//!   [1]      u8    obstacle_count
//!   [2-3]    u16   version (3)
//!
//! Per obstacle (60 bytes):
//!   [0-3]    u32   track_id
//!   [4]      u8    class
//!   [5]      u8    confidence
//!   [6-7]    u16   age
//!   [8-11]   f32   centroid_x (world, meters)
//!   [12-15]  f32   centroid_y (world, meters)
//!   [16-19]  f32   bbox_min_x
//!   [20-23]  f32   bbox_min_y
//!   [24-27]  f32   bbox_max_x
//!   [28-31]  f32   bbox_max_y
//!   [32-35]  u32   cell_count
//!   [36-39]  f32   area (m²)
//!   [40-43]  f32   min_z (meters)
//!   [44-47]  f32   max_z (meters)
//!   [48-51]  f32   rotation (radians, PCA principal axis)
//!   [52-55]  f32   velocity_x (m/s)
//!   [56-59]  f32   velocity_y (m/s)
//! ```

use crate::MSG_OBSTACLES;
#[cfg(test)]
use costmap::clustering::ObstacleClass;
use costmap::TrackedObstacle;

/// Header size in bytes.
const HEADER_SIZE: usize = 4;

/// Per-obstacle record size: 60 bytes.
const RECORD_SIZE: usize = 60;

/// Wire format version.
const WIRE_VERSION: u16 = 3;

/// Serialize tracked obstacles to binary.
pub fn serialize_tracked(obstacles: &[TrackedObstacle]) -> Vec<u8> {
    let count = obstacles.len().min(255) as u8;
    let mut buf = Vec::with_capacity(HEADER_SIZE + count as usize * RECORD_SIZE);

    // Header
    buf.push(MSG_OBSTACLES);
    buf.push(count);
    buf.extend_from_slice(&WIRE_VERSION.to_le_bytes());

    for obs in obstacles.iter().take(count as usize) {
        buf.extend_from_slice(&obs.track_id.to_le_bytes());   // [0-3]
        buf.push(obs.class as u8);                             // [4]
        buf.push(obs.confidence);                              // [5]
        buf.extend_from_slice(&obs.age.to_le_bytes());         // [6-7]
        buf.extend_from_slice(&obs.centroid_x.to_le_bytes());  // [8-11]
        buf.extend_from_slice(&obs.centroid_y.to_le_bytes());  // [12-15]
        buf.extend_from_slice(&obs.bbox_min_x.to_le_bytes());  // [16-19]
        buf.extend_from_slice(&obs.bbox_min_y.to_le_bytes());  // [20-23]
        buf.extend_from_slice(&obs.bbox_max_x.to_le_bytes());  // [24-27]
        buf.extend_from_slice(&obs.bbox_max_y.to_le_bytes());  // [28-31]
        buf.extend_from_slice(&obs.cell_count.to_le_bytes());  // [32-35]
        buf.extend_from_slice(&obs.area.to_le_bytes());        // [36-39]
        buf.extend_from_slice(&obs.min_z.to_le_bytes());       // [40-43]
        buf.extend_from_slice(&obs.max_z.to_le_bytes());       // [44-47]
        buf.extend_from_slice(&obs.rotation.to_le_bytes());    // [48-51]
        buf.extend_from_slice(&obs.velocity_x.to_le_bytes());  // [52-55]
        buf.extend_from_slice(&obs.velocity_y.to_le_bytes());  // [56-59]
    }

    debug_assert_eq!(buf.len(), HEADER_SIZE + count as usize * RECORD_SIZE);
    buf
}

/// Deserialize tracked obstacles from binary (for testing roundtrip).
#[cfg(test)]
fn deserialize_tracked(data: &[u8]) -> Option<Vec<TrackedObstacle>> {
    if data.len() < HEADER_SIZE || data[0] != MSG_OBSTACLES {
        return None;
    }
    let count = data[1] as usize;
    let expected_len = HEADER_SIZE + count * RECORD_SIZE;
    if data.len() < expected_len {
        return None;
    }

    let mut obstacles = Vec::with_capacity(count);
    for i in 0..count {
        let offset = HEADER_SIZE + i * RECORD_SIZE;
        obstacles.push(TrackedObstacle {
            track_id: u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?),
            class: ObstacleClass::from_u8(data[offset + 4]),
            confidence: data[offset + 5],
            age: u16::from_le_bytes(data[offset + 6..offset + 8].try_into().ok()?),
            centroid_x: f32::from_le_bytes(data[offset + 8..offset + 12].try_into().ok()?),
            centroid_y: f32::from_le_bytes(data[offset + 12..offset + 16].try_into().ok()?),
            bbox_min_x: f32::from_le_bytes(data[offset + 16..offset + 20].try_into().ok()?),
            bbox_min_y: f32::from_le_bytes(data[offset + 20..offset + 24].try_into().ok()?),
            bbox_max_x: f32::from_le_bytes(data[offset + 24..offset + 28].try_into().ok()?),
            bbox_max_y: f32::from_le_bytes(data[offset + 28..offset + 32].try_into().ok()?),
            cell_count: u32::from_le_bytes(data[offset + 32..offset + 36].try_into().ok()?),
            area: f32::from_le_bytes(data[offset + 36..offset + 40].try_into().ok()?),
            min_z: f32::from_le_bytes(data[offset + 40..offset + 44].try_into().ok()?),
            max_z: f32::from_le_bytes(data[offset + 44..offset + 48].try_into().ok()?),
            rotation: f32::from_le_bytes(data[offset + 48..offset + 52].try_into().ok()?),
            velocity_x: f32::from_le_bytes(data[offset + 52..offset + 56].try_into().ok()?),
            velocity_y: f32::from_le_bytes(data[offset + 56..offset + 60].try_into().ok()?),
        });
    }

    Some(obstacles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tracked(track_id: u32, cx: f32, cy: f32) -> TrackedObstacle {
        TrackedObstacle {
            track_id,
            class: ObstacleClass::Pedestrian,
            confidence: 200,
            age: 10,
            centroid_x: cx,
            centroid_y: cy,
            bbox_min_x: cx - 0.5,
            bbox_min_y: cy - 0.5,
            bbox_max_x: cx + 0.5,
            bbox_max_y: cy + 0.5,
            cell_count: 25,
            area: 0.25,
            min_z: 0.1,
            max_z: 1.7,
            rotation: 0.42,
            velocity_x: 0.5,
            velocity_y: -0.3,
        }
    }

    #[test]
    fn test_serialize_empty() {
        let data = serialize_tracked(&[]);
        assert_eq!(data.len(), HEADER_SIZE);
        assert_eq!(data[0], MSG_OBSTACLES);
        assert_eq!(data[1], 0);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 3);
    }

    #[test]
    fn test_roundtrip_single() {
        let obstacles = vec![make_tracked(42, 1.5, -2.3)];
        let data = serialize_tracked(&obstacles);
        assert_eq!(data.len(), HEADER_SIZE + RECORD_SIZE);

        let decoded = deserialize_tracked(&data).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].track_id, 42);
        assert_eq!(decoded[0].confidence, 200);
        assert_eq!(decoded[0].age, 10);
        assert!((decoded[0].centroid_x - 1.5).abs() < 0.001);
        assert!((decoded[0].centroid_y - (-2.3)).abs() < 0.001);
        assert!((decoded[0].rotation - 0.42).abs() < 0.001);
        assert!((decoded[0].velocity_x - 0.5).abs() < 0.001);
        assert!((decoded[0].velocity_y - (-0.3)).abs() < 0.001);
        assert!((decoded[0].min_z - 0.1).abs() < 0.001);
        assert!((decoded[0].max_z - 1.7).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_multiple() {
        let obstacles = vec![
            make_tracked(0, 1.0, 2.0),
            make_tracked(1, -3.0, 4.5),
            make_tracked(2, 0.0, 0.0),
        ];
        let data = serialize_tracked(&obstacles);
        assert_eq!(data.len(), HEADER_SIZE + 3 * RECORD_SIZE);

        let decoded = deserialize_tracked(&data).unwrap();
        assert_eq!(decoded.len(), 3);
        for (orig, dec) in obstacles.iter().zip(decoded.iter()) {
            assert_eq!(dec.track_id, orig.track_id);
            assert!((dec.centroid_x - orig.centroid_x).abs() < 0.001);
            assert!((dec.centroid_y - orig.centroid_y).abs() < 0.001);
            assert!((dec.rotation - orig.rotation).abs() < 0.001);
            assert!((dec.velocity_x - orig.velocity_x).abs() < 0.001);
            assert!((dec.velocity_y - orig.velocity_y).abs() < 0.001);
        }
    }

    #[test]
    fn test_message_size() {
        // Max message: 4 + 64 * 60 = 3844 bytes (fits WebRTC MTU)
        let obstacles: Vec<TrackedObstacle> = (0..64).map(|i| make_tracked(i, 0.0, 0.0)).collect();
        let data = serialize_tracked(&obstacles);
        assert_eq!(data.len(), 4 + 64 * 60);
        assert!(data.len() <= 4096); // well under typical SCTP MTU
    }
}
