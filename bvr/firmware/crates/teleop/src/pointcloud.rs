//! Point cloud downsampling and serialization for streaming.
//!
//! Provides efficient downsampling using voxel grid filtering and
//! bandwidth-optimized binary serialization with int16 quantization.

use crate::MSG_POINT_CLOUD;
use lidar::{Point3D, PointCloud};
use std::collections::HashMap;

/// Default maximum points to send per frame (after downsampling).
pub const DEFAULT_MAX_POINTS: usize = 500;

/// Default voxel grid cell size in meters for downsampling.
pub const DEFAULT_VOXEL_SIZE: f32 = 0.3;

/// Point cloud streaming configuration.
#[derive(Debug, Clone)]
pub struct PointCloudConfig {
    /// Voxel grid cell size in meters. Larger = fewer, more spread out points.
    pub voxel_size: f32,
    /// Maximum points to send per frame.
    pub max_points: usize,
}

impl Default for PointCloudConfig {
    fn default() -> Self {
        Self {
            voxel_size: DEFAULT_VOXEL_SIZE,
            max_points: DEFAULT_MAX_POINTS,
        }
    }
}

/// Header size in bytes (type + flags + count + frame_id + scale + offset).
const HEADER_SIZE: usize = 32;

/// Bytes per point (x:i16 + y:i16 + z:i16 + reflectivity:u8 + tag:u8).
const BYTES_PER_POINT: usize = 8;

/// Voxel key for spatial hashing.
#[derive(Hash, Eq, PartialEq)]
struct VoxelKey {
    x: i32,
    y: i32,
    z: i32,
}

impl VoxelKey {
    fn from_point(p: &Point3D, inv_voxel_size: f32) -> Self {
        Self {
            x: (p.x * inv_voxel_size).floor() as i32,
            y: (p.y * inv_voxel_size).floor() as i32,
            z: (p.z * inv_voxel_size).floor() as i32,
        }
    }
}

/// Downsample a point cloud using voxel grid filtering.
///
/// Each voxel cell keeps the point with highest reflectivity (most visible).
/// Points are sampled uniformly across distance to avoid near-point bias.
pub fn downsample(cloud: &PointCloud) -> Vec<Point3D> {
    downsample_with_config(cloud, &PointCloudConfig::default())
}

/// Extract confidence level from Livox tag byte (bits 0-1).
/// Returns 0-3 where 0 is noise/invalid, 3 is highest confidence.
#[inline]
fn get_confidence(tag: u8) -> u8 {
    tag & 0b11
}

/// Downsample a point cloud with custom configuration.
pub fn downsample_with_config(cloud: &PointCloud, config: &PointCloudConfig) -> Vec<Point3D> {
    if cloud.points.is_empty() {
        return Vec::new();
    }

    let inv_voxel_size = 1.0 / config.voxel_size;

    // First pass: accumulate points per voxel, keeping highest reflectivity
    // Filter out noise points (confidence=0) before downsampling
    let mut voxels: HashMap<VoxelKey, Point3D> = HashMap::new();

    for point in &cloud.points {
        // Skip noise points (confidence level 0 in tag bits 0-1)
        if get_confidence(point.tag) == 0 {
            continue;
        }

        let key = VoxelKey::from_point(point, inv_voxel_size);

        voxels
            .entry(key)
            .and_modify(|best| {
                // Keep point with highest reflectivity within voxel
                if point.reflectivity > best.reflectivity {
                    *best = *point;
                }
            })
            .or_insert(*point);
    }

    // Collect all voxel representatives
    let mut points: Vec<Point3D> = voxels.into_values().collect();

    // If we have more points than max, sample uniformly by distance
    // This avoids the near-point bias from reflectivity sorting
    if points.len() > config.max_points {
        // Sort by distance from origin (sensor position)
        points.sort_unstable_by(|a, b| {
            let dist_a = a.x * a.x + a.y * a.y + a.z * a.z;
            let dist_b = b.x * b.x + b.y * b.y + b.z * b.z;
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Uniform stride sampling to get points across all distances
        let stride = points.len() as f32 / config.max_points as f32;
        let sampled: Vec<Point3D> = (0..config.max_points)
            .map(|i| {
                let idx = ((i as f32 * stride) as usize).min(points.len() - 1);
                points[idx]
            })
            .collect();
        points = sampled;
    }

    points
}

/// Serialize downsampled points to binary format for streaming.
///
/// # Binary format (32-byte header + 8 bytes/point):
/// ```text
/// [0]      u8    MSG_POINT_CLOUD (0x21)
/// [1]      u8    flags (reserved)
/// [2-3]    u16   point_count (LE)
/// [4-7]    u32   frame_id (LE)
/// [8-11]   f32   scale_x (LE)
/// [12-15]  f32   scale_y (LE)
/// [16-19]  f32   scale_z (LE)
/// [20-23]  f32   offset_x (LE)
/// [24-27]  f32   offset_y (LE)
/// [28-31]  f32   offset_z (LE)
/// [32...]  per-point: [x:i16 LE][y:i16 LE][z:i16 LE][reflectivity:u8][tag:u8]
/// ```
///
/// Points are quantized to int16 using: `value = (raw - offset) / scale`
/// To decode: `raw = value * scale + offset`
pub fn serialize(points: &[Point3D], frame_id: u32) -> Vec<u8> {
    if points.is_empty() {
        // Return minimal packet with zero points
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        buf.push(MSG_POINT_CLOUD);
        buf.push(0); // flags
        buf.extend_from_slice(&0u16.to_le_bytes()); // point_count
        buf.extend_from_slice(&frame_id.to_le_bytes());
        // Scale and offset (zeros)
        buf.extend_from_slice(&[0u8; 24]);
        return buf;
    }

    // Compute bounding box for quantization
    let (min_x, max_x, min_y, max_y, min_z, max_z) = points.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(min_x, max_x, min_y, max_y, min_z, max_z), p| {
            (
                min_x.min(p.x),
                max_x.max(p.x),
                min_y.min(p.y),
                max_y.max(p.y),
                min_z.min(p.z),
                max_z.max(p.z),
            )
        },
    );

    // Compute scale and offset for int16 quantization
    // Range: -32768 to 32767, we use 0-centered range for safety
    const QUANT_RANGE: f32 = 32000.0; // Leave some headroom

    let range_x = (max_x - min_x).max(0.001);
    let range_y = (max_y - min_y).max(0.001);
    let range_z = (max_z - min_z).max(0.001);

    let scale_x = range_x / (2.0 * QUANT_RANGE);
    let scale_y = range_y / (2.0 * QUANT_RANGE);
    let scale_z = range_z / (2.0 * QUANT_RANGE);

    let offset_x = (min_x + max_x) / 2.0;
    let offset_y = (min_y + max_y) / 2.0;
    let offset_z = (min_z + max_z) / 2.0;

    // Allocate buffer
    let point_count = points.len().min(u16::MAX as usize);
    let buf_size = HEADER_SIZE + point_count * BYTES_PER_POINT;
    let mut buf = Vec::with_capacity(buf_size);

    // Write header
    buf.push(MSG_POINT_CLOUD);
    buf.push(0); // flags (reserved)
    buf.extend_from_slice(&(point_count as u16).to_le_bytes());
    buf.extend_from_slice(&frame_id.to_le_bytes());
    buf.extend_from_slice(&scale_x.to_le_bytes());
    buf.extend_from_slice(&scale_y.to_le_bytes());
    buf.extend_from_slice(&scale_z.to_le_bytes());
    buf.extend_from_slice(&offset_x.to_le_bytes());
    buf.extend_from_slice(&offset_y.to_le_bytes());
    buf.extend_from_slice(&offset_z.to_le_bytes());

    // Write points
    for point in points.iter().take(point_count) {
        let qx = ((point.x - offset_x) / scale_x).round() as i16;
        let qy = ((point.y - offset_y) / scale_y).round() as i16;
        let qz = ((point.z - offset_z) / scale_z).round() as i16;

        buf.extend_from_slice(&qx.to_le_bytes());
        buf.extend_from_slice(&qy.to_le_bytes());
        buf.extend_from_slice(&qz.to_le_bytes());
        buf.push(point.reflectivity);
        buf.push(point.tag);
    }

    buf
}

/// Downsample and serialize a point cloud in one step.
pub fn prepare_for_streaming(cloud: &PointCloud) -> Vec<u8> {
    prepare_for_streaming_with_config(cloud, &PointCloudConfig::default())
}

/// Downsample and serialize with custom configuration.
pub fn prepare_for_streaming_with_config(cloud: &PointCloud, config: &PointCloudConfig) -> Vec<u8> {
    let points = downsample_with_config(cloud, config);
    serialize(&points, cloud.frame_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_point(x: f32, y: f32, z: f32, reflectivity: u8) -> Point3D {
        Point3D {
            x,
            y,
            z,
            reflectivity,
            tag: 3, // High confidence so points aren't filtered
        }
    }

    #[test]
    fn test_downsample_empty() {
        let cloud = PointCloud::default();
        let result = downsample(&cloud);
        assert!(result.is_empty());
    }

    #[test]
    fn test_downsample_preserves_points_in_different_voxels() {
        let mut cloud = PointCloud::default();
        // Points far apart (different voxels)
        cloud.points = vec![
            make_point(0.0, 0.0, 0.0, 100),
            make_point(1.0, 1.0, 1.0, 100),
            make_point(2.0, 2.0, 2.0, 100),
        ];
        let result = downsample(&cloud);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_downsample_merges_points_in_same_voxel() {
        let mut cloud = PointCloud::default();
        // Points very close together (same voxel)
        cloud.points = vec![
            make_point(0.0, 0.0, 0.0, 50),
            make_point(0.1, 0.1, 0.1, 100), // Higher reflectivity
            make_point(0.2, 0.2, 0.2, 75),
        ];
        let result = downsample(&cloud);
        assert_eq!(result.len(), 1);
        // Should keep the point with highest reflectivity
        assert_eq!(result[0].reflectivity, 100);
    }

    #[test]
    fn test_downsample_respects_max_points() {
        let config = PointCloudConfig::default();
        let mut cloud = PointCloud::default();
        // Create many points in different voxels
        for i in 0..1000 {
            cloud.points.push(make_point(
                i as f32 * config.voxel_size * 2.0,
                0.0,
                0.0,
                (i % 256) as u8,
            ));
        }
        let result = downsample_with_config(&cloud, &config);
        assert!(result.len() <= config.max_points);
    }

    #[test]
    fn test_downsample_uniform_distance_sampling() {
        let config = PointCloudConfig {
            voxel_size: 0.1,
            max_points: 10,
        };
        let mut cloud = PointCloud::default();
        // Create points at various distances
        for i in 0..100 {
            cloud.points.push(make_point(
                i as f32 * 0.5, // 0 to 50 meters
                0.0,
                0.0,
                100,
            ));
        }
        let result = downsample_with_config(&cloud, &config);
        assert_eq!(result.len(), 10);

        // Verify we got points from different distances (not just near)
        let distances: Vec<f32> = result.iter().map(|p| p.x).collect();
        assert!(distances.iter().any(|&d| d > 20.0), "Should include far points");
    }

    #[test]
    fn test_serialize_empty() {
        let points: Vec<Point3D> = vec![];
        let buf = serialize(&points, 42);
        assert_eq!(buf.len(), HEADER_SIZE);
        assert_eq!(buf[0], MSG_POINT_CLOUD);
        // Point count should be 0
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 0);
        // Frame ID should be 42
        assert_eq!(
            u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            42
        );
    }

    #[test]
    fn test_serialize_single_point() {
        let points = vec![make_point(1.0, 2.0, 3.0, 128)];
        let buf = serialize(&points, 1);

        assert_eq!(buf[0], MSG_POINT_CLOUD);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 1);
        assert_eq!(buf.len(), HEADER_SIZE + BYTES_PER_POINT);

        // Last two bytes of point data are reflectivity and tag
        assert_eq!(buf[buf.len() - 2], 128); // reflectivity
        assert_eq!(buf[buf.len() - 1], 3); // tag (high confidence)
    }

    #[test]
    fn test_serialize_roundtrip() {
        let points = vec![
            make_point(-5.0, -5.0, -1.0, 50),
            make_point(5.0, 5.0, 2.0, 200),
            make_point(0.0, 0.0, 0.5, 100),
        ];
        let buf = serialize(&points, 99);

        // Parse header
        assert_eq!(buf[0], MSG_POINT_CLOUD);
        let point_count = u16::from_le_bytes([buf[2], buf[3]]) as usize;
        assert_eq!(point_count, 3);

        let scale_x = f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let scale_y = f32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let scale_z = f32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let offset_x = f32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        let offset_y = f32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);
        let offset_z = f32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);

        // Decode first point
        let qx = i16::from_le_bytes([buf[32], buf[33]]);
        let qy = i16::from_le_bytes([buf[34], buf[35]]);
        let qz = i16::from_le_bytes([buf[36], buf[37]]);
        let reflectivity = buf[38];

        let x = qx as f32 * scale_x + offset_x;
        let y = qy as f32 * scale_y + offset_y;
        let z = qz as f32 * scale_z + offset_z;

        // Check reconstruction accuracy (should be within quantization error)
        assert!((x - points[0].x).abs() < 0.01);
        assert!((y - points[0].y).abs() < 0.01);
        assert!((z - points[0].z).abs() < 0.01);
        assert_eq!(reflectivity, points[0].reflectivity);
    }
}
