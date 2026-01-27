//! Connected-component clustering for obstacle extraction.
//!
//! Identifies discrete obstacles in the costmap by flood-filling
//! LETHAL cells (8-connected), then computes bounding boxes,
//! centroids, and PCA elongation in world coordinates.

use std::collections::VecDeque;

/// Minimum number of cells for a cluster to be reported as an obstacle.
/// At 0.1 m resolution, 12 cells ≈ 0.12 m² — filters LiDAR noise and
/// small returns off cables/edges/furniture legs while catching real obstacles.
const MIN_CLUSTER_CELLS: usize = 12;

/// Maximum number of obstacles to report (sorted by area, largest first).
const MAX_OBSTACLES: usize = 64;

/// Obstacle classification based on shape/size heuristics.
///
/// Wire format: single u8, matching the console decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObstacleClass {
    Unknown = 0,
    /// Tall, narrow obstacle (e.g. sign post, bollard, tree trunk).
    Pole = 1,
    /// Large, roughly rectangular obstacle (e.g. parked car, dumpster).
    Vehicle = 2,
    /// Small, compact obstacle (e.g. person, animal, fire hydrant).
    Pedestrian = 3,
    /// Long, thin obstacle (e.g. wall, fence, curb).
    Wall = 4,
    /// Anything that doesn't match other heuristics.
    Debris = 5,
}

impl ObstacleClass {
    /// Convert from wire format byte.
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Pole,
            2 => Self::Vehicle,
            3 => Self::Pedestrian,
            4 => Self::Wall,
            5 => Self::Debris,
            _ => Self::Unknown,
        }
    }
}

/// A detected obstacle extracted from the costmap.
#[derive(Debug, Clone, PartialEq)]
pub struct Obstacle {
    /// Unique ID within this frame (0-based).
    pub id: u16,
    /// Heuristic classification.
    pub class: ObstacleClass,
    /// Centroid X in world coordinates (meters).
    pub centroid_x: f32,
    /// Centroid Y in world coordinates (meters).
    pub centroid_y: f32,
    /// Bounding box minimum X in world coordinates.
    pub bbox_min_x: f32,
    /// Bounding box minimum Y in world coordinates.
    pub bbox_min_y: f32,
    /// Bounding box maximum X in world coordinates.
    pub bbox_max_x: f32,
    /// Bounding box maximum Y in world coordinates.
    pub bbox_max_y: f32,
    /// Number of cells in the cluster.
    pub cell_count: u32,
    /// Area in square meters.
    pub area: f32,
    /// PCA elongation ratio √(λ₁/λ₂) — rotation-invariant shape measure.
    /// 1.0 = perfectly round, higher = more elongated.
    pub elongation: f32,
    /// PCA principal axis angle in radians (-π/2 to π/2).
    /// Angle of the longest axis from the world +X direction.
    pub rotation: f32,
    /// Minimum observed Z (height) across cluster cells, in meters.
    /// 0.0 when no height data is available.
    pub min_z: f32,
    /// Maximum observed Z (height) across cluster cells, in meters.
    /// 0.0 when no height data is available.
    pub max_z: f32,
}

/// Classify an obstacle based on size, PCA elongation, and fill ratio.
///
/// Uses PCA elongation (rotation-invariant) instead of bbox aspect ratio:
/// - **Pole**: small area (<0.1 m²), low elongation (<3), tall if height available
/// - **Wall**: high elongation (>2.5)
/// - **Vehicle**: large area (>1.0 m²), fill >0.4, tall if height available
/// - **Pedestrian**: medium area (0.15–1.0 m²), compact, fill >0.4, human height if available
/// - **Debris**: anything else
fn classify_obstacle(obs: &Obstacle) -> ObstacleClass {
    let area = obs.area;
    let elongation = obs.elongation;

    let w = obs.bbox_max_x - obs.bbox_min_x;
    let h = obs.bbox_max_y - obs.bbox_min_y;
    let bbox_area = w * h;
    let fill_ratio = if bbox_area > 0.0001 {
        area / bbox_area
    } else {
        1.0
    };

    let height_z = obs.max_z - obs.min_z;
    let has_height = obs.min_z <= obs.max_z && height_z > 0.0;

    // Pole: small footprint, compact shape, tall when height is available
    if area < 0.1 && elongation < 3.0 {
        if !has_height || height_z > 0.5 {
            return ObstacleClass::Pole;
        }
    }

    // Wall: high PCA elongation (rotation-invariant — works for diagonal walls)
    if elongation > 2.5 {
        return ObstacleClass::Wall;
    }

    // Vehicle: large area with substantial fill; height confirms when available
    if area > 1.0 && fill_ratio > 0.4 {
        if !has_height || height_z > 1.0 {
            return ObstacleClass::Vehicle;
        }
    }

    // Pedestrian: medium area, compact shape, substantial fill
    if area >= 0.15 && area <= 1.0 && elongation < 2.5 && fill_ratio > 0.4 {
        if !has_height || (height_z >= 0.8 && height_z <= 2.2) {
            return ObstacleClass::Pedestrian;
        }
    }

    ObstacleClass::Debris
}

/// Extract obstacles from a costmap using connected-component labeling.
///
/// Scans the combined costmap for cells with cost >= `threshold` (typically
/// LETHAL = 254), groups contiguous cells via BFS, and returns obstacle
/// descriptors sorted by area (largest first).
pub fn extract_obstacles(
    cells: &[u8],
    width: usize,
    height: usize,
    resolution: f64,
    origin_x: f64,
    origin_y: f64,
    threshold: u8,
    height_min: &[f32],
    height_max: &[f32],
) -> Vec<Obstacle> {
    let total = width * height;
    if cells.len() != total || total == 0 {
        return Vec::new();
    }

    let has_height = height_min.len() == total && height_max.len() == total;

    let mut visited = vec![false; total];
    let mut obstacles = Vec::new();
    let mut queue = VecDeque::new();

    for start in 0..total {
        if visited[start] || cells[start] < threshold {
            continue;
        }

        // BFS flood fill through occupied cells (8-connected)
        queue.clear();
        queue.push_back(start);
        visited[start] = true;

        let mut sum_gx: u64 = 0;
        let mut sum_gy: u64 = 0;
        let mut sum_gx2: u64 = 0;
        let mut sum_gy2: u64 = 0;
        let mut sum_gxgy: u64 = 0;
        let mut min_gx = usize::MAX;
        let mut min_gy = usize::MAX;
        let mut max_gx: usize = 0;
        let mut max_gy: usize = 0;
        let mut count: u32 = 0;
        let mut cluster_min_z = f32::INFINITY;
        let mut cluster_max_z = f32::NEG_INFINITY;

        while let Some(idx) = queue.pop_front() {
            let gx = idx % width;
            let gy = idx / width;

            sum_gx += gx as u64;
            sum_gy += gy as u64;
            sum_gx2 += (gx as u64) * (gx as u64);
            sum_gy2 += (gy as u64) * (gy as u64);
            sum_gxgy += (gx as u64) * (gy as u64);
            min_gx = min_gx.min(gx);
            min_gy = min_gy.min(gy);
            max_gx = max_gx.max(gx);
            max_gy = max_gy.max(gy);
            count += 1;

            if has_height {
                let lo = height_min[idx];
                let hi = height_max[idx];
                if lo <= hi {
                    cluster_min_z = cluster_min_z.min(lo);
                    cluster_max_z = cluster_max_z.max(hi);
                }
            }

            // 8-connected neighbors through occupied cells
            for (dx, dy) in [(-1i32, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let nidx = ny as usize * width + nx as usize;
                if !visited[nidx] && cells[nidx] >= threshold {
                    visited[nidx] = true;
                    queue.push_back(nidx);
                }
            }
        }

        if (count as usize) < MIN_CLUSTER_CELLS {
            continue;
        }

        // PCA elongation from second-order moments
        let n = count as f64;
        let mean_x = sum_gx as f64 / n;
        let mean_y = sum_gy as f64 / n;
        let cov_xx = sum_gx2 as f64 / n - mean_x * mean_x;
        let cov_yy = sum_gy2 as f64 / n - mean_y * mean_y;
        let cov_xy = sum_gxgy as f64 / n - mean_x * mean_y;

        // Eigenvalues of 2×2 covariance matrix:
        // λ = (cov_xx + cov_yy)/2 ± sqrt(((cov_xx - cov_yy)/2)² + cov_xy²)
        let half_trace = (cov_xx + cov_yy) * 0.5;
        let diff_half = (cov_xx - cov_yy) * 0.5;
        let disc = (diff_half * diff_half + cov_xy * cov_xy).sqrt();
        let lambda1 = half_trace + disc; // larger eigenvalue
        let lambda2 = half_trace - disc; // smaller eigenvalue

        let elongation = if lambda2 > 1e-9 {
            (lambda1 / lambda2).sqrt() as f32
        } else {
            // Degenerate (collinear cells) → very elongated
            10.0_f32
        };

        // Principal axis angle: atan2(2·cov_xy, cov_xx - cov_yy) / 2
        let rotation = (0.5 * (2.0 * cov_xy).atan2(cov_xx - cov_yy)) as f32;

        // Convert grid coordinates to world coordinates
        let centroid_x = origin_x + (mean_x + 0.5) * resolution;
        let centroid_y = origin_y + (mean_y + 0.5) * resolution;

        // Bounding box: use cell edges (not centers)
        let bbox_min_x = origin_x + min_gx as f64 * resolution;
        let bbox_min_y = origin_y + min_gy as f64 * resolution;
        let bbox_max_x = origin_x + (max_gx + 1) as f64 * resolution;
        let bbox_max_y = origin_y + (max_gy + 1) as f64 * resolution;

        let area = count as f64 * resolution * resolution;

        obstacles.push(Obstacle {
            id: 0,
            class: ObstacleClass::Unknown,
            centroid_x: centroid_x as f32,
            centroid_y: centroid_y as f32,
            bbox_min_x: bbox_min_x as f32,
            bbox_min_y: bbox_min_y as f32,
            bbox_max_x: bbox_max_x as f32,
            bbox_max_y: bbox_max_y as f32,
            cell_count: count,
            area: area as f32,
            elongation,
            rotation,
            min_z: if cluster_min_z <= cluster_max_z { cluster_min_z } else { 0.0 },
            max_z: if cluster_min_z <= cluster_max_z { cluster_max_z } else { 0.0 },
        });
    }

    // Sort by area descending, truncate to max
    obstacles.sort_by(|a, b| b.area.partial_cmp(&a.area).unwrap_or(std::cmp::Ordering::Equal));
    obstacles.truncate(MAX_OBSTACLES);

    // Assign sequential IDs and classify
    for (i, obs) in obstacles.iter_mut().enumerate() {
        obs.id = i as u16;
        obs.class = classify_obstacle(obs);
    }

    obstacles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::costs;

    fn make_grid(width: usize, height: usize) -> Vec<u8> {
        vec![costs::FREE; width * height]
    }

    fn set_cell(cells: &mut [u8], width: usize, gx: usize, gy: usize, value: u8) {
        cells[gy * width + gx] = value;
    }

    #[test]
    fn test_empty_grid_no_obstacles() {
        let cells = make_grid(20, 20);
        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL, &[], &[]);
        assert!(obs.is_empty());
    }

    #[test]
    fn test_single_cell_filtered() {
        let mut cells = make_grid(20, 20);
        set_cell(&mut cells, 20, 10, 10, costs::LETHAL);
        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL, &[], &[]);
        // Single cell < MIN_CLUSTER_CELLS (3), should be filtered
        assert!(obs.is_empty());
    }

    #[test]
    fn test_small_cluster_filtered() {
        let mut cells = make_grid(20, 20);
        // 6-cell cluster — below MIN_CLUSTER_CELLS (12), should be filtered
        for dx in 0..2 {
            for dy in 0..3 {
                set_cell(&mut cells, 20, 5 + dx, 5 + dy, costs::LETHAL);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert!(obs.is_empty());
    }

    #[test]
    fn test_cluster_at_threshold() {
        let mut cells = make_grid(20, 20);
        // 12-cell cluster (3x4 block) — exactly MIN_CLUSTER_CELLS
        for dx in 0..3 {
            for dy in 0..4 {
                set_cell(&mut cells, 20, 5 + dx, 5 + dy, costs::LETHAL);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].id, 0);
        assert_eq!(obs[0].cell_count, 12);

        // Area = 12 cells * 0.1 * 0.1 = 0.12 m²
        assert!((obs[0].area - 0.12).abs() < 0.001);

        // Bounding box: cells (5,5) to (7,8) -> world (0.5, 0.5) to (0.8, 0.9)
        assert!((obs[0].bbox_min_x - 0.5).abs() < 0.001);
        assert!((obs[0].bbox_min_y - 0.5).abs() < 0.001);
        assert!((obs[0].bbox_max_x - 0.8).abs() < 0.001);
        assert!((obs[0].bbox_max_y - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_two_separate_clusters() {
        let mut cells = make_grid(20, 20);
        // Cluster A: 16 cells (4x4 block)
        for gx in 2..=5 {
            for gy in 2..=5 {
                set_cell(&mut cells, 20, gx, gy, costs::LETHAL);
            }
        }

        // Cluster B: 12 cells (3x4 block)
        for gx in 14..=16 {
            for gy in 14..=17 {
                set_cell(&mut cells, 20, gx, gy, costs::LETHAL);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 2);

        // Sorted by area descending: cluster A (16 cells) first
        assert_eq!(obs[0].cell_count, 16);
        assert_eq!(obs[0].id, 0);
        assert_eq!(obs[1].cell_count, 12);
        assert_eq!(obs[1].id, 1);
    }

    #[test]
    fn test_centroid_world_coords() {
        let mut cells = make_grid(20, 20);
        // Horizontal bar: (3,10)..(14,10) — 12 cells
        for gx in 3..=14 {
            set_cell(&mut cells, 20, gx, 10, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 1);

        // Grid centroid: gx = (3+4+...+14)/12 = 8.5, gy = 10
        // World: x = -1.0 + (8.5 + 0.5) * 0.1 = -0.1, y = -1.0 + (10.0 + 0.5) * 0.1 = 0.05
        assert!((obs[0].centroid_x - (-0.1)).abs() < 0.001);
        assert!((obs[0].centroid_y - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_threshold_filters_lower_costs() {
        let mut cells = make_grid(20, 20);
        // INSCRIBED cells (253) should NOT be picked up with LETHAL threshold
        // 3x4 block = 12 cells
        for dx in 0..3 {
            for dy in 0..4 {
                set_cell(&mut cells, 20, 5 + dx, 5 + dy, costs::INSCRIBED);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert!(obs.is_empty());

        // But with INSCRIBED threshold, they should be found
        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::INSCRIBED, &[], &[]);
        assert_eq!(obs.len(), 1);
    }

    #[test]
    fn test_diagonal_cluster_too_small() {
        let mut cells = make_grid(20, 20);
        // 3 diagonal cells: connected via 8-connectivity but still below
        // MIN_CLUSTER_CELLS (12), so filtered out.
        set_cell(&mut cells, 20, 5, 5, costs::LETHAL);
        set_cell(&mut cells, 20, 6, 6, costs::LETHAL);
        set_cell(&mut cells, 20, 7, 7, costs::LETHAL);

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert!(obs.is_empty());
    }

    #[test]
    fn test_max_obstacles_truncation() {
        let mut cells = make_grid(200, 200);
        // Create 70 separate 12-cell clusters (exceeds MAX_OBSTACLES = 64)
        for i in 0..70 {
            let base_x = (i % 14) * 10;
            let base_y = (i / 14) * 10;
            // 3x4 block = 12 cells
            for dx in 0..3 {
                for dy in 0..4 {
                    set_cell(&mut cells, 200, base_x + dx, base_y + dy, costs::LETHAL);
                }
            }
        }

        let obs = extract_obstacles(&cells, 200, 200, 0.1, -10.0, -10.0, costs::LETHAL, &[], &[]);
        assert!(obs.len() <= MAX_OBSTACLES);
    }

    // ====================================================================
    // Classification tests
    // ====================================================================

    fn make_obstacle(area: f32, elongation: f32) -> Obstacle {
        // bbox is only used for fill_ratio now; make it consistent with area
        let side = area.sqrt();
        Obstacle {
            id: 0,
            class: ObstacleClass::Unknown,
            centroid_x: 0.0,
            centroid_y: 0.0,
            bbox_min_x: -side / 2.0,
            bbox_min_y: -side / 2.0,
            bbox_max_x: side / 2.0,
            bbox_max_y: side / 2.0,
            cell_count: (area / 0.01) as u32,
            area,
            elongation,
            rotation: 0.0,
            min_z: 0.0,
            max_z: 0.0,
        }
    }

    fn make_obstacle_with_bbox(area: f32, elongation: f32, w: f32, h: f32) -> Obstacle {
        Obstacle {
            id: 0,
            class: ObstacleClass::Unknown,
            centroid_x: 0.0,
            centroid_y: 0.0,
            bbox_min_x: -w / 2.0,
            bbox_min_y: -h / 2.0,
            bbox_max_x: w / 2.0,
            bbox_max_y: h / 2.0,
            cell_count: (area / 0.01) as u32,
            area,
            elongation,
            rotation: 0.0,
            min_z: 0.0,
            max_z: 0.0,
        }
    }

    fn make_obstacle_with_height(area: f32, elongation: f32, min_z: f32, max_z: f32) -> Obstacle {
        let side = area.sqrt();
        Obstacle {
            id: 0,
            class: ObstacleClass::Unknown,
            centroid_x: 0.0,
            centroid_y: 0.0,
            bbox_min_x: -side / 2.0,
            bbox_min_y: -side / 2.0,
            bbox_max_x: side / 2.0,
            bbox_max_y: side / 2.0,
            cell_count: (area / 0.01) as u32,
            area,
            elongation,
            rotation: 0.0,
            min_z,
            max_z,
        }
    }

    #[test]
    fn test_classify_pole() {
        // Small, compact: area < 0.1, elongation < 3
        let obs = make_obstacle(0.05, 1.2);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pole);
    }

    #[test]
    fn test_classify_wall() {
        // High elongation > 2.5
        let obs = make_obstacle(0.5, 5.0);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Wall);
    }

    #[test]
    fn test_classify_vehicle() {
        // Large area (> 1.0 m²), low elongation, good fill
        let obs = make_obstacle(2.0, 1.3);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Vehicle);
    }

    #[test]
    fn test_classify_pedestrian() {
        // Medium area (0.15 - 1.0), compact shape, good fill
        let obs = make_obstacle(0.3, 1.2);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pedestrian);
    }

    #[test]
    fn test_classify_debris() {
        // Medium area but low fill ratio → Debris
        let obs = make_obstacle_with_bbox(0.4, 1.5, 2.0, 1.0);
        // fill = 0.4 / 2.0 = 0.2 → pedestrian fill check fails → Debris
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Debris);
    }

    #[test]
    fn test_classification_integrated() {
        // Create a grid with obstacles of different sizes and verify classification
        let mut cells = make_grid(100, 100);
        // Small 12-cell cluster (pole-sized at 0.1m resolution: 0.12 m² < 0.15)
        // But wait: area=0.12 is < 0.15, so not Pedestrian. aspect ~1.3, area < 0.15 → Pole? No, area >= 0.1 but < 0.15 → Pole (< 0.1 check fails, wall fails, vehicle fails, pedestrian area >= 0.15 fails → Debris)
        // Use a 3x4 block → area=0.12, aspect = 0.4/0.3 = 1.33 → Pole (area < 0.1 check)... actually area = 0.12 > 0.1, so Pole check fails.
        // Let's make a compact 12-cell cluster that's definitely Pole: need area < 0.1. That means < 10 cells at 0.01 m²/cell. But MIN_CLUSTER_CELLS=12. So a 12-cell cluster can't be Pole anymore.
        // Instead test Debris (small non-pole) vs Wall:
        // 12 cells in a 3x4 block: area=0.12, aspect = 0.4/0.3 = 1.33 → not Pole (area >= 0.1), not Wall (aspect < 4), not Vehicle (< 1.0), Pedestrian? area >= 0.15 fails → Debris
        for dx in 0..3 {
            for dy in 0..4 {
                set_cell(&mut cells, 100, 10 + dx, 10 + dy, costs::LETHAL);
            }
        }

        // Long wall: 20 cells in a line (0.2 m² area, 2.0m x 0.1m)
        for gx in 50..70 {
            set_cell(&mut cells, 100, gx, 50, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 100, 100, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 2);

        // Sorted by area: wall (0.2 m²) first, then debris (0.12 m²)
        assert_eq!(obs[0].class, ObstacleClass::Wall);
        assert_eq!(obs[1].class, ObstacleClass::Debris);
    }

    #[test]
    fn test_obstacle_class_from_u8() {
        assert_eq!(ObstacleClass::from_u8(0), ObstacleClass::Unknown);
        assert_eq!(ObstacleClass::from_u8(1), ObstacleClass::Pole);
        assert_eq!(ObstacleClass::from_u8(2), ObstacleClass::Vehicle);
        assert_eq!(ObstacleClass::from_u8(3), ObstacleClass::Pedestrian);
        assert_eq!(ObstacleClass::from_u8(4), ObstacleClass::Wall);
        assert_eq!(ObstacleClass::from_u8(5), ObstacleClass::Debris);
        assert_eq!(ObstacleClass::from_u8(99), ObstacleClass::Unknown);
    }

    // ====================================================================
    // Height-aware tests
    // ====================================================================

    #[test]
    fn test_extract_obstacles_with_height_data() {
        let width = 20;
        let height = 20;
        let total = width * height;
        let mut cells = make_grid(width, height);

        // 3x4 block = 12 cells at (5,5)
        for dx in 0..3 {
            for dy in 0..4 {
                set_cell(&mut cells, width, 5 + dx, 5 + dy, costs::LETHAL);
            }
        }

        // Create height data: sentinel for most cells, real values for the obstacle
        let mut h_min = vec![f32::INFINITY; total];
        let mut h_max = vec![f32::NEG_INFINITY; total];

        for dx in 0..3 {
            for dy in 0..4 {
                let idx = (5 + dy) * width + (5 + dx);
                h_min[idx] = 0.1;
                h_max[idx] = 1.2;
            }
        }

        let obs = extract_obstacles(
            &cells, width, height, 0.1, 0.0, 0.0, costs::LETHAL,
            &h_min, &h_max,
        );
        assert_eq!(obs.len(), 1);
        assert!((obs[0].min_z - 0.1).abs() < 0.001);
        assert!((obs[0].max_z - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_extract_obstacles_mixed_height_cells() {
        let width = 20;
        let height = 20;
        let total = width * height;
        let mut cells = make_grid(width, height);

        // 4x3 block = 12 cells at (2,2)
        for dx in 0..4 {
            for dy in 0..3 {
                set_cell(&mut cells, width, 2 + dx, 2 + dy, costs::LETHAL);
            }
        }

        let mut h_min = vec![f32::INFINITY; total];
        let mut h_max = vec![f32::NEG_INFINITY; total];

        // Only some cells have height observations
        let idx1 = 2 * width + 2;
        h_min[idx1] = 0.3;
        h_max[idx1] = 0.8;

        let idx2 = 3 * width + 3;
        h_min[idx2] = 0.1;
        h_max[idx2] = 1.5;

        let obs = extract_obstacles(
            &cells, width, height, 0.1, 0.0, 0.0, costs::LETHAL,
            &h_min, &h_max,
        );
        assert_eq!(obs.len(), 1);
        // min should be minimum of 0.1 and 0.3
        assert!((obs[0].min_z - 0.1).abs() < 0.001);
        // max should be maximum of 0.8 and 1.5
        assert!((obs[0].max_z - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_extract_obstacles_no_height_fallback() {
        let width = 20;
        let height = 20;
        let mut cells = make_grid(width, height);

        // 3x4 block
        for dx in 0..3 {
            for dy in 0..4 {
                set_cell(&mut cells, width, 5 + dx, 5 + dy, costs::LETHAL);
            }
        }

        // No height data (empty slices)
        let obs = extract_obstacles(
            &cells, width, height, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[],
        );
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].min_z, 0.0);
        assert_eq!(obs[0].max_z, 0.0);
    }

    #[test]
    fn test_classify_pole_with_height() {
        // Small footprint + confirmed tall → Pole
        let obs = make_obstacle_with_height(0.05, 1.2, 0.0, 1.0);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pole);
    }

    #[test]
    fn test_classify_pedestrian_with_height() {
        // Medium area, compact, human height range → Pedestrian
        let obs = make_obstacle_with_height(0.3, 1.2, 0.0, 1.7);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pedestrian);
    }

    #[test]
    fn test_classify_pedestrian_too_short_with_height() {
        // Medium area, compact, but very short → Debris (not a person)
        let obs = make_obstacle_with_height(0.3, 1.2, 0.0, 0.3);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Debris);
    }

    // ====================================================================
    // PCA elongation classification tests
    // ====================================================================

    #[test]
    fn test_classify_diagonal_wall() {
        // Diagonal wall: high PCA elongation despite roughly square bbox
        let obs = make_obstacle(0.5, 4.0);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Wall);
    }

    #[test]
    fn test_classify_vehicle_not_wall() {
        // Large solid obstacle: low elongation → Vehicle, not Wall
        let obs = make_obstacle(1.5, 1.3);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Vehicle);
    }

    #[test]
    fn test_classify_elongated_large_cluster_as_wall() {
        // Thin L-shaped cluster: high elongation → Wall even with large area
        let obs = make_obstacle(1.2, 3.5);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Wall);
    }

    // ====================================================================
    // PCA elongation integration tests
    // ====================================================================

    #[test]
    fn test_horizontal_bar_elongation() {
        let mut cells = make_grid(30, 10);
        // Horizontal bar: 20 cells in a line at y=5
        for gx in 5..25 {
            set_cell(&mut cells, 30, gx, 5, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 30, 10, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 1);
        // 20 cells in a row → very elongated
        assert!(obs[0].elongation > 3.0, "expected high elongation, got {}", obs[0].elongation);
        assert_eq!(obs[0].class, ObstacleClass::Wall);
    }

    #[test]
    fn test_square_block_elongation() {
        let mut cells = make_grid(20, 20);
        // 4×4 square block = 16 cells
        for gx in 5..9 {
            for gy in 5..9 {
                set_cell(&mut cells, 20, gx, gy, costs::LETHAL);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 1);
        // Square → elongation ≈ 1.0
        assert!(obs[0].elongation < 1.5, "expected low elongation, got {}", obs[0].elongation);
    }

    #[test]
    fn test_diagonal_line_elongation() {
        let mut cells = make_grid(30, 30);
        // Diagonal line: 15 cells along y=x
        for i in 5..20 {
            set_cell(&mut cells, 30, i, i, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 30, 30, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 1);
        // Diagonal line → high elongation despite square-ish bbox
        assert!(obs[0].elongation > 3.0, "expected high elongation for diagonal, got {}", obs[0].elongation);
        assert_eq!(obs[0].class, ObstacleClass::Wall);
    }

    #[test]
    fn test_no_gap_bridging() {
        let mut cells = make_grid(20, 20);
        // Two 12-cell blocks separated by a 1-cell gap.
        // Without dilation, they stay separate.
        // Block A: 3×4 = 12 cells
        for gx in 2..5 {
            for gy in 2..6 {
                set_cell(&mut cells, 20, gx, gy, costs::LETHAL);
            }
        }
        // Block B: 3×4 = 12 cells (gap at gx=5)
        for gx in 6..9 {
            for gy in 2..6 {
                set_cell(&mut cells, 20, gx, gy, costs::LETHAL);
            }
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL, &[], &[]);
        assert_eq!(obs.len(), 2);
    }
}
