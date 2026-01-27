//! Map persistence: save and load pose graphs to/from binary files.
//!
//! Serializes keyframe poses, downsampled scans, pose graph edges, and
//! scan context descriptors to a compact binary format using bincode.
//! Supports atomic writes (temp file + rename) for crash safety.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use nalgebra::Matrix3;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use lidar::{Point3D, PointCloud};
use transforms::Transform2D;

use crate::{
    Keyframe, PoseGraphEdge, ScanContextConfig, ScanContextDatabase, ScanContextDescriptor,
    SlamConfig, SlamProcessor,
};

/// Magic bytes identifying a Muni SLAM map file.
const MAP_MAGIC: &[u8; 4] = b"MSLM";

/// Current file format version.
const MAP_VERSION: u32 = 1;

/// Error type for map persistence operations.
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialize(#[from] bincode::Error),
    #[error("Invalid map file: {0}")]
    InvalidFile(String),
    #[error("Version mismatch: file={file}, expected={expected}")]
    VersionMismatch { file: u32, expected: u32 },
}

/// Configuration for map persistence.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Path to save/load the map file
    pub map_path: String,
    /// Auto-save every N keyframes (0 = disabled)
    pub auto_save_keyframes: usize,
    /// Load prior map on startup
    pub load_prior_map: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            map_path: "/var/lib/bvr/slam_map.bin".into(),
            auto_save_keyframes: 50,
            load_prior_map: true,
        }
    }
}

// --- Serializable types (plain data, no nalgebra/Arc) ---

#[derive(Serialize, Deserialize)]
struct MapFileHeader {
    magic: [u8; 4],
    version: u32,
    rover_id: String,
    keyframe_count: u32,
    edge_count: u32,
}

#[derive(Serialize, Deserialize)]
struct SerializedKeyframe {
    id: usize,
    /// Pose as [x, y, theta]
    pose: [f64; 3],
    /// Downsampled scan points as flat [x, y, z, x, y, z, ...]
    scan_points: Vec<f32>,
    /// Optional scan context descriptor matrix
    descriptor_matrix: Option<Vec<f32>>,
    /// Optional ring key
    descriptor_ring_key: Option<Vec<f32>>,
    /// Optional sector key
    descriptor_sector_key: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize)]
struct SerializedEdge {
    from_id: usize,
    to_id: usize,
    /// Measurement as [x, y, theta]
    measurement: [f64; 3],
    /// Information matrix (row-major 3x3)
    information: [f64; 9],
    is_loop_closure: bool,
}

#[derive(Serialize, Deserialize)]
struct SerializedMap {
    header: MapFileHeader,
    keyframes: Vec<SerializedKeyframe>,
    edges: Vec<SerializedEdge>,
    /// Scan context config used when building descriptors
    sc_num_rings: usize,
    sc_num_sectors: usize,
}

// --- Conversion helpers ---

fn serialize_keyframe(kf: &Keyframe, voxel_size: f64) -> SerializedKeyframe {
    // Downsample scan to GICP voxel resolution for compact storage
    let points = downsample_for_storage(&kf.scan, voxel_size);

    let (desc_matrix, desc_ring_key, desc_sector_key) = match &kf.descriptor {
        Some(d) => (
            Some(d.matrix.clone()),
            Some(d.ring_key.clone()),
            Some(d.sector_key.clone()),
        ),
        None => (None, None, None),
    };

    SerializedKeyframe {
        id: kf.id,
        pose: [
            kf.pose.translation().x,
            kf.pose.translation().y,
            kf.pose.rotation(),
        ],
        scan_points: points,
        descriptor_matrix: desc_matrix,
        descriptor_ring_key: desc_ring_key,
        descriptor_sector_key: desc_sector_key,
    }
}

fn deserialize_keyframe(skf: &SerializedKeyframe, sc_config: &ScanContextConfig) -> Keyframe {
    let pose = Transform2D::new(skf.pose[0], skf.pose[1], skf.pose[2]);

    // Reconstruct point cloud from flat f32 array
    let mut points = Vec::with_capacity(skf.scan_points.len() / 3);
    for chunk in skf.scan_points.chunks_exact(3) {
        points.push(Point3D {
            x: chunk[0],
            y: chunk[1],
            z: chunk[2],
            reflectivity: 128,
            tag: 0,
        });
    }
    let scan = Arc::new(PointCloud {
        points,
        ..Default::default()
    });

    // Reconstruct descriptor if available
    let descriptor = match (&skf.descriptor_matrix, &skf.descriptor_ring_key, &skf.descriptor_sector_key) {
        (Some(matrix), Some(ring_key), Some(sector_key)) => Some(ScanContextDescriptor {
            matrix: matrix.clone(),
            ring_key: ring_key.clone(),
            sector_key: sector_key.clone(),
            num_rings: sc_config.num_rings,
            num_sectors: sc_config.num_sectors,
        }),
        _ => None,
    };

    Keyframe {
        id: skf.id,
        pose,
        scan,
        timestamp: Instant::now(), // Not preserved across saves
        descriptor,
    }
}

fn serialize_edge(edge: &PoseGraphEdge) -> SerializedEdge {
    let info = edge.information;
    SerializedEdge {
        from_id: edge.from_id,
        to_id: edge.to_id,
        measurement: [
            edge.measurement.translation().x,
            edge.measurement.translation().y,
            edge.measurement.rotation(),
        ],
        information: [
            info[(0, 0)], info[(0, 1)], info[(0, 2)],
            info[(1, 0)], info[(1, 1)], info[(1, 2)],
            info[(2, 0)], info[(2, 1)], info[(2, 2)],
        ],
        is_loop_closure: edge.is_loop_closure,
    }
}

fn deserialize_edge(se: &SerializedEdge) -> PoseGraphEdge {
    let measurement = Transform2D::new(se.measurement[0], se.measurement[1], se.measurement[2]);
    let i = &se.information;
    let information = Matrix3::new(
        i[0], i[1], i[2],
        i[3], i[4], i[5],
        i[6], i[7], i[8],
    );

    PoseGraphEdge {
        from_id: se.from_id,
        to_id: se.to_id,
        measurement,
        information,
        is_loop_closure: se.is_loop_closure,
    }
}

/// Downsample a point cloud for storage (flat f32 array).
fn downsample_for_storage(scan: &PointCloud, voxel_size: f64) -> Vec<f32> {
    use std::collections::HashMap;

    type VoxelAccum = (f64, f64, f64, usize);
    let inv = 1.0 / voxel_size;
    let mut grid: HashMap<(i32, i32, i32), VoxelAccum> = HashMap::new();

    for p in &scan.points {
        let range_sq = p.x * p.x + p.y * p.y + p.z * p.z;
        if !range_sq.is_finite() || !(0.01..=2500.0).contains(&range_sq) {
            continue;
        }
        let ix = (p.x as f64 * inv).floor() as i32;
        let iy = (p.y as f64 * inv).floor() as i32;
        let iz = (p.z as f64 * inv).floor() as i32;

        let e = grid.entry((ix, iy, iz)).or_insert((0.0, 0.0, 0.0, 0));
        e.0 += p.x as f64;
        e.1 += p.y as f64;
        e.2 += p.z as f64;
        e.3 += 1;
    }

    let mut result = Vec::with_capacity(grid.len() * 3);
    for (sx, sy, sz, c) in grid.into_values() {
        let n = c as f64;
        result.push((sx / n) as f32);
        result.push((sy / n) as f32);
        result.push((sz / n) as f32);
    }
    result
}

// --- Public API on SlamProcessor ---

impl SlamProcessor {
    /// Save the current map (keyframes + edges) to a binary file.
    ///
    /// Uses atomic write (temp file + rename) for crash safety.
    pub fn save_map(&self, path: &Path, rover_id: &str) -> Result<(), MapError> {
        let sc_config = self.scan_context_db().config();

        let serialized = SerializedMap {
            header: MapFileHeader {
                magic: *MAP_MAGIC,
                version: MAP_VERSION,
                rover_id: rover_id.to_string(),
                keyframe_count: self.keyframes().len() as u32,
                edge_count: self.edges().len() as u32,
            },
            keyframes: self
                .keyframes()
                .iter()
                .map(|kf| serialize_keyframe(kf, self.config().voxel_size))
                .collect(),
            edges: self.edges().iter().map(serialize_edge).collect(),
            sc_num_rings: sc_config.num_rings,
            sc_num_sectors: sc_config.num_sectors,
        };

        let bytes = bincode::serialize(&serialized)?;

        // Atomic write: temp file → rename
        let tmp_path = path.with_extension("bin.tmp");
        fs::write(&tmp_path, &bytes)?;
        fs::rename(&tmp_path, path)?;

        info!(
            path = %path.display(),
            keyframes = serialized.header.keyframe_count,
            edges = serialized.header.edge_count,
            bytes = bytes.len(),
            "Saved SLAM map"
        );

        Ok(())
    }

    /// Load a map from a binary file, reconstructing keyframes, edges,
    /// and the scan context database.
    ///
    /// Returns `(keyframes, edges, scan_context_db)` for the caller to
    /// populate into a SlamProcessor.
    pub fn load_map(
        path: &Path,
        config: &SlamConfig,
    ) -> Result<(Vec<Keyframe>, Vec<PoseGraphEdge>, ScanContextDatabase), MapError> {
        let bytes = fs::read(path)?;
        let map: SerializedMap = bincode::deserialize(&bytes)?;

        // Validate header
        if map.header.magic != *MAP_MAGIC {
            return Err(MapError::InvalidFile("Bad magic bytes".into()));
        }
        if map.header.version != MAP_VERSION {
            return Err(MapError::VersionMismatch {
                file: map.header.version,
                expected: MAP_VERSION,
            });
        }

        let sc_config = ScanContextConfig {
            num_rings: map.sc_num_rings,
            num_sectors: map.sc_num_sectors,
            ..config.scan_context.clone()
        };

        let keyframes: Vec<Keyframe> = map
            .keyframes
            .iter()
            .map(|skf| deserialize_keyframe(skf, &sc_config))
            .collect();

        let edges: Vec<PoseGraphEdge> = map.edges.iter().map(deserialize_edge).collect();

        // Rebuild scan context database
        let mut sc_db = ScanContextDatabase::new(sc_config);
        for kf in &keyframes {
            if let Some(ref desc) = kf.descriptor {
                sc_db.insert(kf.id, desc.clone());
            }
        }

        info!(
            path = %path.display(),
            keyframes = keyframes.len(),
            edges = edges.len(),
            rover_id = %map.header.rover_id,
            "Loaded SLAM map"
        );

        Ok((keyframes, edges, sc_db))
    }

    /// Attempt to relocalize against a loaded map using scan context + GICP.
    ///
    /// Returns the index of the best matching keyframe and the estimated pose,
    /// or `None` if no good match is found.
    pub fn relocalize(
        &self,
        scan: &PointCloud,
        min_score: f64,
    ) -> Option<(usize, Transform2D)> {
        // Compute descriptor for current scan
        let query_desc = self.compute_descriptor(scan);

        // Find candidates in the loaded map's scan context DB
        let candidates = self.scan_context_db.find_candidates(&query_desc);
        if candidates.is_empty() {
            debug!("Relocalization: no scan context candidates");
            return None;
        }

        debug!(
            num_candidates = candidates.len(),
            "Relocalization: testing candidates"
        );

        let scan_arc = Arc::new(scan.clone());

        let mut best_match: Option<(usize, Transform2D, f64)> = None;

        for candidate in &candidates {
            let kf = &self.keyframes()[candidate.keyframe_id];
            let initial_guess = Transform2D::identity(); // No prior pose knowledge

            match self.scan_matcher().match_scans(&kf.scan, &scan_arc, initial_guess) {
                Ok(result) => {
                    if result.score >= min_score {
                        let is_better = best_match.as_ref().is_none_or(
                            |(_, _, best_score)| result.score > *best_score,
                        );
                        if is_better {
                            // Absolute pose = keyframe pose * scan match transform
                            let pose = kf.pose * result.transform;
                            best_match = Some((candidate.keyframe_id, pose, result.score));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        if let Some((kf_id, pose, score)) = best_match {
            info!(
                keyframe_id = kf_id,
                score,
                x = pose.translation().x,
                y = pose.translation().y,
                "Relocalized against loaded map"
            );
            Some((kf_id, pose))
        } else {
            warn!("Relocalization failed: no candidates above score threshold");
            None
        }
    }

    /// Restore state from a loaded map (keyframes, edges, scan context DB).
    /// Typically called after `load_map()` succeeds.
    pub fn restore_map(
        &mut self,
        keyframes: Vec<Keyframe>,
        edges: Vec<PoseGraphEdge>,
        scan_context_db: ScanContextDatabase,
    ) {
        let kf_count = keyframes.len();
        let edge_count = edges.len();
        let lc_count = edges.iter().filter(|e| e.is_loop_closure).count();

        if let Some(last) = keyframes.last() {
            self.last_keyframe_pose = last.pose;
            self.last_odom_pose = last.pose;
            self.last_scan_odom_pose = last.pose;
            self.ekf.reset_pose(&last.pose);
        }

        self.keyframes = keyframes;
        self.edges = edges;
        self.scan_context_db = scan_context_db;
        self.loop_closure_count = lc_count;

        info!(
            keyframes = kf_count,
            edges = edge_count,
            loop_closures = lc_count,
            "Restored SLAM map state"
        );
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &SlamConfig {
        &self.config
    }

    /// Get a reference to the scan matcher (for relocalization).
    fn scan_matcher(&self) -> &crate::CorrelativeScanMatcher {
        &self.scan_matcher
    }

    /// Get the number of keyframes since last save (for auto-save logic).
    pub fn keyframes_since(&self, last_save_count: usize) -> usize {
        self.keyframes.len().saturating_sub(last_save_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SlamConfig;
    use lidar::Point3D;
    use std::path::PathBuf;

    fn make_test_scan(offset_x: f32) -> PointCloud {
        let mut points = Vec::new();
        for i in 0..360 {
            let angle = (i as f32) * std::f32::consts::TAU / 360.0;
            let range = 5.0 + offset_x;
            points.push(Point3D {
                x: range * angle.cos(),
                y: range * angle.sin(),
                z: 0.5,
                reflectivity: 128,
                tag: 0,
            });
        }
        PointCloud {
            points,
            ..Default::default()
        }
    }

    fn temp_map_path() -> PathBuf {
        let dir = std::env::temp_dir().join("slam_test");
        fs::create_dir_all(&dir).ok();
        dir.join(format!("test_map_{}.bin", std::process::id()))
    }

    #[test]
    fn test_round_trip_save_load() {
        let mut config = SlamConfig::default();
        config.keyframe_distance = 0.5;
        let mut processor = SlamProcessor::new(config.clone());

        // Add some keyframes by driving forward
        let scan1 = make_test_scan(0.0);
        processor.process_scan(&scan1);

        processor.update_odometry(&types::Pose {
            x: 2.0,
            y: 0.0,
            theta: 0.0,
        });
        let scan2 = make_test_scan(0.1);
        processor.process_scan(&scan2);

        processor.update_odometry(&types::Pose {
            x: 4.0,
            y: 0.0,
            theta: 0.0,
        });
        let scan3 = make_test_scan(0.2);
        processor.process_scan(&scan3);

        assert!(processor.keyframes().len() >= 2);

        // Save
        let path = temp_map_path();
        processor.save_map(&path, "test-rover").unwrap();

        // Verify file exists
        assert!(path.exists());
        let file_size = fs::metadata(&path).unwrap().len();
        assert!(file_size > 0);

        // Load
        let (keyframes, edges, sc_db) = SlamProcessor::load_map(&path, &config).unwrap();

        // Verify loaded data matches
        assert_eq!(keyframes.len(), processor.keyframes().len());
        assert_eq!(edges.len(), processor.edges().len());

        // Verify pose values match
        for (loaded, original) in keyframes.iter().zip(processor.keyframes().iter()) {
            assert_eq!(loaded.id, original.id);
            let lt = loaded.pose.translation();
            let ot = original.pose.translation();
            assert!((lt.x - ot.x).abs() < 1e-6, "x mismatch");
            assert!((lt.y - ot.y).abs() < 1e-6, "y mismatch");
            assert!(
                (loaded.pose.rotation() - original.pose.rotation()).abs() < 1e-6,
                "theta mismatch"
            );
        }

        // Verify scan context DB was rebuilt
        assert_eq!(sc_db.len(), processor.scan_context_db().len());

        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_atomic_write_no_temp_left() {
        let config = SlamConfig::default();
        let mut processor = SlamProcessor::new(config.clone());
        let scan = make_test_scan(0.0);
        processor.process_scan(&scan);

        let path = temp_map_path();
        processor.save_map(&path, "test").unwrap();

        // Temp file should not exist after successful save
        let tmp_path = path.with_extension("bin.tmp");
        assert!(!tmp_path.exists());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_invalid_magic() {
        let path = temp_map_path();
        fs::write(&path, b"NOT_A_MAP_FILE").unwrap();

        let config = SlamConfig::default();
        let result = SlamProcessor::load_map(&path, &config);
        assert!(result.is_err());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_restore_map() {
        let config = SlamConfig::default();
        let mut processor = SlamProcessor::new(config.clone());

        // Create some keyframes
        let scan = make_test_scan(0.0);
        processor.process_scan(&scan);
        processor.update_odometry(&types::Pose {
            x: 2.0,
            y: 0.0,
            theta: 0.0,
        });
        let scan2 = make_test_scan(0.1);
        processor.process_scan(&scan2);

        // Save and load
        let path = temp_map_path();
        processor.save_map(&path, "test").unwrap();
        let (keyframes, edges, sc_db) = SlamProcessor::load_map(&path, &config).unwrap();

        // Restore into a fresh processor
        let mut new_processor = SlamProcessor::new(config);
        new_processor.restore_map(keyframes, edges, sc_db);

        assert_eq!(
            new_processor.keyframes().len(),
            processor.keyframes().len()
        );
        assert_eq!(new_processor.edges().len(), processor.edges().len());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_edge_round_trip() {
        let edge = PoseGraphEdge {
            from_id: 0,
            to_id: 5,
            measurement: Transform2D::new(1.0, 2.0, 0.3),
            information: Matrix3::new(
                100.0, 1.0, 2.0,
                1.0, 100.0, 3.0,
                2.0, 3.0, 50.0,
            ),
            is_loop_closure: true,
        };

        let serialized = serialize_edge(&edge);
        let deserialized = deserialize_edge(&serialized);

        assert_eq!(deserialized.from_id, edge.from_id);
        assert_eq!(deserialized.to_id, edge.to_id);
        assert!(deserialized.is_loop_closure);
        assert!((deserialized.measurement.translation().x - 1.0).abs() < 1e-10);
        assert!((deserialized.information[(0, 0)] - 100.0).abs() < 1e-10);
        assert!((deserialized.information[(0, 1)] - 1.0).abs() < 1e-10);
    }
}
