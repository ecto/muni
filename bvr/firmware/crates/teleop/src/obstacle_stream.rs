//! Obstacle list serialization for WebRTC streaming.
//!
//! Serializes extracted obstacles into a compact binary format.
//!
//! # Binary format
//!
//! ```text
//! Header (4 bytes):
//!   [0]      u8    MSG_OBSTACLES (0x23)
//!   [1]      u8    obstacle_count
//!   [2-3]    u16   reserved (0)
//!
//! Per obstacle (36 bytes):
//!   [0-1]    u16   id
//!   [2]      u8    class (0=Unknown, see ObstacleClass)
//!   [3]      u8    reserved
//!   [4-7]    f32   centroid_x (world, meters)
//!   [8-11]   f32   centroid_y (world, meters)
//!   [12-15]  f32   bbox_min_x
//!   [16-19]  f32   bbox_min_y
//!   [20-23]  f32   bbox_max_x
//!   [24-27]  f32   bbox_max_y
//!   [28-31]  u32   cell_count
//!   [32-35]  f32   area (m²)
//! ```

use crate::MSG_OBSTACLES;
use costmap::clustering::Obstacle;
#[cfg(test)]
use costmap::clustering::ObstacleClass;

/// Header size in bytes.
const HEADER_SIZE: usize = 4;

/// Per-obstacle record size in bytes.
const OBSTACLE_RECORD_SIZE: usize = 36;

/// Serialize an obstacle list to binary.
pub fn serialize(obstacles: &[Obstacle]) -> Vec<u8> {
    let count = obstacles.len().min(255) as u8;
    let mut buf = Vec::with_capacity(HEADER_SIZE + count as usize * OBSTACLE_RECORD_SIZE);

    // Header
    buf.push(MSG_OBSTACLES);
    buf.push(count);
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved

    for obs in obstacles.iter().take(count as usize) {
        buf.extend_from_slice(&obs.id.to_le_bytes());
        buf.push(obs.class as u8);
        buf.push(0); // reserved
        buf.extend_from_slice(&obs.centroid_x.to_le_bytes());
        buf.extend_from_slice(&obs.centroid_y.to_le_bytes());
        buf.extend_from_slice(&obs.bbox_min_x.to_le_bytes());
        buf.extend_from_slice(&obs.bbox_min_y.to_le_bytes());
        buf.extend_from_slice(&obs.bbox_max_x.to_le_bytes());
        buf.extend_from_slice(&obs.bbox_max_y.to_le_bytes());
        buf.extend_from_slice(&obs.cell_count.to_le_bytes());
        buf.extend_from_slice(&obs.area.to_le_bytes());
    }

    debug_assert_eq!(buf.len(), HEADER_SIZE + count as usize * OBSTACLE_RECORD_SIZE);
    buf
}

/// Deserialize obstacles from binary (for testing roundtrip).
#[cfg(test)]
fn deserialize(data: &[u8]) -> Option<Vec<Obstacle>> {
    if data.len() < HEADER_SIZE {
        return None;
    }
    if data[0] != MSG_OBSTACLES {
        return None;
    }
    let count = data[1] as usize;
    let expected_len = HEADER_SIZE + count * OBSTACLE_RECORD_SIZE;
    if data.len() < expected_len {
        return None;
    }

    let mut obstacles = Vec::with_capacity(count);
    for i in 0..count {
        let offset = HEADER_SIZE + i * OBSTACLE_RECORD_SIZE;
        let id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let class = ObstacleClass::from_u8(data[offset + 2]);
        let centroid_x = f32::from_le_bytes(data[offset + 4..offset + 8].try_into().ok()?);
        let centroid_y = f32::from_le_bytes(data[offset + 8..offset + 12].try_into().ok()?);
        let bbox_min_x = f32::from_le_bytes(data[offset + 12..offset + 16].try_into().ok()?);
        let bbox_min_y = f32::from_le_bytes(data[offset + 16..offset + 20].try_into().ok()?);
        let bbox_max_x = f32::from_le_bytes(data[offset + 20..offset + 24].try_into().ok()?);
        let bbox_max_y = f32::from_le_bytes(data[offset + 24..offset + 28].try_into().ok()?);
        let cell_count = u32::from_le_bytes(data[offset + 28..offset + 32].try_into().ok()?);
        let area = f32::from_le_bytes(data[offset + 32..offset + 36].try_into().ok()?);

        obstacles.push(Obstacle {
            id,
            class,
            centroid_x,
            centroid_y,
            bbox_min_x,
            bbox_min_y,
            bbox_max_x,
            bbox_max_y,
            cell_count,
            area,
        });
    }

    Some(obstacles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obstacle(id: u16, cx: f32, cy: f32) -> Obstacle {
        Obstacle {
            id,
            class: ObstacleClass::Pedestrian,
            centroid_x: cx,
            centroid_y: cy,
            bbox_min_x: cx - 0.5,
            bbox_min_y: cy - 0.5,
            bbox_max_x: cx + 0.5,
            bbox_max_y: cy + 0.5,
            cell_count: 25,
            area: 0.25,
        }
    }

    #[test]
    fn test_serialize_empty() {
        let data = serialize(&[]);
        assert_eq!(data.len(), HEADER_SIZE);
        assert_eq!(data[0], MSG_OBSTACLES);
        assert_eq!(data[1], 0);
    }

    #[test]
    fn test_roundtrip_single() {
        let obstacles = vec![make_obstacle(0, 1.5, -2.3)];
        let data = serialize(&obstacles);
        assert_eq!(data.len(), HEADER_SIZE + OBSTACLE_RECORD_SIZE);

        let decoded = deserialize(&data).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, 0);
        assert!((decoded[0].centroid_x - 1.5).abs() < 0.001);
        assert!((decoded[0].centroid_y - (-2.3)).abs() < 0.001);
        assert_eq!(decoded[0].cell_count, 25);
        assert!((decoded[0].area - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_multiple() {
        let obstacles = vec![
            make_obstacle(0, 1.0, 2.0),
            make_obstacle(1, -3.0, 4.5),
            make_obstacle(2, 0.0, 0.0),
        ];
        let data = serialize(&obstacles);
        assert_eq!(data.len(), HEADER_SIZE + 3 * OBSTACLE_RECORD_SIZE);

        let decoded = deserialize(&data).unwrap();
        assert_eq!(decoded.len(), 3);
        for (i, (orig, dec)) in obstacles.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(dec.id, i as u16);
            assert!((dec.centroid_x - orig.centroid_x).abs() < 0.001);
            assert!((dec.centroid_y - orig.centroid_y).abs() < 0.001);
            assert!((dec.bbox_min_x - orig.bbox_min_x).abs() < 0.001);
            assert!((dec.bbox_max_y - orig.bbox_max_y).abs() < 0.001);
        }
    }

    #[test]
    fn test_header_fields() {
        let obstacles = vec![make_obstacle(5, 1.0, 2.0)];
        let data = serialize(&obstacles);
        assert_eq!(data[0], MSG_OBSTACLES);
        assert_eq!(data[1], 1); // count
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 0); // reserved
    }

    #[test]
    fn test_deserialize_truncated() {
        let data = vec![MSG_OBSTACLES, 1, 0, 0]; // Header says 1 obstacle but no data
        assert!(deserialize(&data).is_none());
    }
}
