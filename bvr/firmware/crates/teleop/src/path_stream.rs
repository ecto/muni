//! Navigation path serialization for WebRTC streaming.
//!
//! Serializes the planned A* path and navigation state into a compact binary
//! format for visualization in the console teleop UI.
//!
//! # Binary format
//!
//! ```text
//! Header (12 bytes):
//!   [0]      u8    MSG_PATH (0x25)
//!   [1]      u8    navigation_state (0=Idle, 1=Planning, 2=Following, 3=ObstacleStopped, 4=Replanning, 5=Recovering, 6=GoalReached, 7=Failed)
//!   [2-3]    u16   waypoint_count (LE)
//!   [4-5]    u16   current_waypoint_index (LE)
//!   [6-9]    f32   distance_to_goal (meters, LE)
//!   [10-11]  u16   reserved
//!
//! Per waypoint (8 bytes):
//!   [0-3]    f32   x (world meters, LE)
//!   [4-7]    f32   y (world meters, LE)
//! ```

use crate::{MSG_MPPI_TRAJECTORY, MSG_PATH};

/// Header size in bytes.
const HEADER_SIZE: usize = 12;

/// Per-waypoint record size: 8 bytes (x: f32 + y: f32).
const WAYPOINT_SIZE: usize = 8;

/// Navigation state wire values.
pub mod nav_state {
    pub const IDLE: u8 = 0;
    pub const PLANNING: u8 = 1;
    pub const FOLLOWING: u8 = 2;
    pub const OBSTACLE_STOPPED: u8 = 3;
    pub const REPLANNING: u8 = 4;
    pub const RECOVERING: u8 = 5;
    pub const GOAL_REACHED: u8 = 6;
    pub const FAILED: u8 = 7;
}

/// Snapshot of the current navigation path for streaming.
#[derive(Debug, Clone)]
pub struct PathSnapshot {
    /// Navigation state.
    pub state: u8,
    /// Path waypoints (x, y) in world meters.
    pub waypoints: Vec<(f32, f32)>,
    /// Index of the current target waypoint.
    pub current_waypoint: u16,
    /// Distance to current goal in meters.
    pub distance_to_goal: f32,
    /// MPPI best trajectory (x, y) in world meters. Empty when MPPI is not active.
    pub mppi_trajectory: Vec<(f32, f32)>,
}

/// Serialize a path snapshot to binary.
pub fn serialize(snapshot: &PathSnapshot) -> Vec<u8> {
    let count = snapshot.waypoints.len().min(u16::MAX as usize) as u16;
    let mut buf = Vec::with_capacity(HEADER_SIZE + count as usize * WAYPOINT_SIZE);

    // Header
    buf.push(MSG_PATH);
    buf.push(snapshot.state);
    buf.extend_from_slice(&count.to_le_bytes());
    buf.extend_from_slice(&snapshot.current_waypoint.to_le_bytes());
    buf.extend_from_slice(&snapshot.distance_to_goal.to_le_bytes());
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved

    debug_assert_eq!(buf.len(), HEADER_SIZE);

    // Waypoints
    for &(x, y) in snapshot.waypoints.iter().take(count as usize) {
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
    }

    debug_assert_eq!(buf.len(), HEADER_SIZE + count as usize * WAYPOINT_SIZE);
    buf
}

/// MPPI trajectory header size: [msg_type:u8][reserved:u8][num_points:u16][reserved:4B] = 8 bytes.
const MPPI_HEADER_SIZE: usize = 8;

/// Serialize an MPPI trajectory to binary.
///
/// # Binary format
///
/// ```text
/// Header (8 bytes):
///   [0]      u8    MSG_MPPI_TRAJECTORY (0x26)
///   [1]      u8    reserved
///   [2-3]    u16   num_points (LE)
///   [4-7]    u32   reserved
///
/// Per point (8 bytes):
///   [0-3]    f32   x (world meters, LE)
///   [4-7]    f32   y (world meters, LE)
/// ```
pub fn serialize_mppi_trajectory(points: &[(f32, f32)]) -> Vec<u8> {
    let count = points.len().min(u16::MAX as usize) as u16;
    let mut buf = Vec::with_capacity(MPPI_HEADER_SIZE + count as usize * WAYPOINT_SIZE);

    // Header
    buf.push(MSG_MPPI_TRAJECTORY);
    buf.push(0); // reserved
    buf.extend_from_slice(&count.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // reserved

    debug_assert_eq!(buf.len(), MPPI_HEADER_SIZE);

    // Points
    for &(x, y) in points.iter().take(count as usize) {
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
    }

    debug_assert_eq!(buf.len(), MPPI_HEADER_SIZE + count as usize * WAYPOINT_SIZE);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_empty() {
        let snapshot = PathSnapshot {
            state: nav_state::IDLE,
            waypoints: vec![],
            current_waypoint: 0,
            distance_to_goal: 0.0,
            mppi_trajectory: vec![],
        };
        let data = serialize(&snapshot);
        assert_eq!(data.len(), HEADER_SIZE);
        assert_eq!(data[0], MSG_PATH);
        assert_eq!(data[1], nav_state::IDLE);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 0);
    }

    #[test]
    fn test_serialize_with_waypoints() {
        let snapshot = PathSnapshot {
            state: nav_state::FOLLOWING,
            waypoints: vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)],
            current_waypoint: 1,
            distance_to_goal: 3.5,
            mppi_trajectory: vec![],
        };
        let data = serialize(&snapshot);
        assert_eq!(data.len(), HEADER_SIZE + 3 * WAYPOINT_SIZE);
        assert_eq!(data[0], MSG_PATH);
        assert_eq!(data[1], nav_state::FOLLOWING);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 3);
        assert_eq!(u16::from_le_bytes([data[4], data[5]]), 1);

        let dist = f32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        assert!((dist - 3.5).abs() < 0.001);

        // First waypoint
        let x0 = f32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let y0 = f32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        assert!((x0 - 1.0).abs() < 0.001);
        assert!((y0 - 2.0).abs() < 0.001);

        // Second waypoint
        let x1 = f32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let y1 = f32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        assert!((x1 - 3.0).abs() < 0.001);
        assert!((y1 - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_message_size() {
        // 100 waypoints: 12 + 100 * 8 = 812 bytes — well under SCTP MTU
        let waypoints: Vec<(f32, f32)> = (0..100).map(|i| (i as f32, i as f32)).collect();
        let snapshot = PathSnapshot {
            state: nav_state::FOLLOWING,
            waypoints,
            current_waypoint: 50,
            distance_to_goal: 10.0,
            mppi_trajectory: vec![],
        };
        let data = serialize(&snapshot);
        assert_eq!(data.len(), HEADER_SIZE + 100 * WAYPOINT_SIZE);
        assert!(data.len() <= 4096);
    }

    #[test]
    fn test_serialize_mppi_trajectory_empty() {
        let data = serialize_mppi_trajectory(&[]);
        assert_eq!(data.len(), MPPI_HEADER_SIZE);
        assert_eq!(data[0], MSG_MPPI_TRAJECTORY);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 0);
    }

    #[test]
    fn test_serialize_mppi_trajectory_with_points() {
        let points = vec![(1.0, 2.0), (3.0, 4.0)];
        let data = serialize_mppi_trajectory(&points);
        assert_eq!(data.len(), MPPI_HEADER_SIZE + 2 * WAYPOINT_SIZE);
        assert_eq!(data[0], MSG_MPPI_TRAJECTORY);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 2);

        let x0 = f32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let y0 = f32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        assert!((x0 - 1.0).abs() < 0.001);
        assert!((y0 - 2.0).abs() < 0.001);
    }
}
