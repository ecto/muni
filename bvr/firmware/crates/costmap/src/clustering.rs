//! Connected-component clustering for obstacle extraction.
//!
//! Identifies discrete obstacles in the costmap by flood-filling
//! contiguous LETHAL cells, then computes bounding boxes and centroids
//! in world coordinates.

use std::collections::VecDeque;

/// Minimum number of cells for a cluster to be reported as an obstacle.
/// Filters out single-cell noise from LiDAR returns.
const MIN_CLUSTER_CELLS: usize = 3;

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
}

/// Classify an obstacle based on size and shape heuristics.
///
/// Uses bounding box dimensions and area to estimate type:
/// - **Pole**: small area, low aspect ratio (compact & tiny)
/// - **Wall**: high aspect ratio (one dimension much larger than the other)
/// - **Vehicle**: large area, moderate aspect ratio
/// - **Pedestrian**: medium area, compact shape
/// - **Debris**: anything else
fn classify_obstacle(obs: &Obstacle) -> ObstacleClass {
    let w = obs.bbox_max_x - obs.bbox_min_x;
    let h = obs.bbox_max_y - obs.bbox_min_y;
    let extent_max = w.max(h);
    let extent_min = w.min(h);
    let aspect = if extent_min > 0.01 {
        extent_max / extent_min
    } else {
        10.0 // degenerate → treat as wall-like
    };

    let area = obs.area;

    // Pole: very small area (< 0.1 m²) and compact shape
    if area < 0.1 && aspect < 3.0 {
        return ObstacleClass::Pole;
    }

    // Wall: high aspect ratio (> 4:1) regardless of area
    if aspect > 4.0 {
        return ObstacleClass::Wall;
    }

    // Vehicle: large area (> 1.0 m²)
    if area > 1.0 {
        return ObstacleClass::Vehicle;
    }

    // Pedestrian: medium area (0.1 - 1.0 m²), compact shape
    if area >= 0.1 && aspect < 2.5 {
        return ObstacleClass::Pedestrian;
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
) -> Vec<Obstacle> {
    let total = width * height;
    if cells.len() != total || total == 0 {
        return Vec::new();
    }

    let mut visited = vec![false; total];
    let mut obstacles = Vec::new();
    let mut queue = VecDeque::new();

    for start in 0..total {
        if visited[start] || cells[start] < threshold {
            continue;
        }

        // BFS flood fill from this cell
        queue.clear();
        queue.push_back(start);
        visited[start] = true;

        let mut sum_gx: u64 = 0;
        let mut sum_gy: u64 = 0;
        let mut min_gx = usize::MAX;
        let mut min_gy = usize::MAX;
        let mut max_gx: usize = 0;
        let mut max_gy: usize = 0;
        let mut count: u32 = 0;

        while let Some(idx) = queue.pop_front() {
            let gx = idx % width;
            let gy = idx / width;

            sum_gx += gx as u64;
            sum_gy += gy as u64;
            min_gx = min_gx.min(gx);
            min_gy = min_gy.min(gy);
            max_gx = max_gx.max(gx);
            max_gy = max_gy.max(gy);
            count += 1;

            // 4-connected neighbors
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
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

        // Convert grid coordinates to world coordinates
        let centroid_gx = sum_gx as f64 / count as f64;
        let centroid_gy = sum_gy as f64 / count as f64;

        let centroid_x = origin_x + (centroid_gx + 0.5) * resolution;
        let centroid_y = origin_y + (centroid_gy + 0.5) * resolution;

        // Bounding box: use cell edges (not centers)
        let bbox_min_x = origin_x + min_gx as f64 * resolution;
        let bbox_min_y = origin_y + min_gy as f64 * resolution;
        let bbox_max_x = origin_x + (max_gx + 1) as f64 * resolution;
        let bbox_max_y = origin_y + (max_gy + 1) as f64 * resolution;

        let area = count as f64 * resolution * resolution;

        obstacles.push(Obstacle {
            id: 0, // assigned after sorting
            class: ObstacleClass::Unknown, // classified after construction
            centroid_x: centroid_x as f32,
            centroid_y: centroid_y as f32,
            bbox_min_x: bbox_min_x as f32,
            bbox_min_y: bbox_min_y as f32,
            bbox_max_x: bbox_max_x as f32,
            bbox_max_y: bbox_max_y as f32,
            cell_count: count,
            area: area as f32,
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
        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL);
        assert!(obs.is_empty());
    }

    #[test]
    fn test_single_cell_filtered() {
        let mut cells = make_grid(20, 20);
        set_cell(&mut cells, 20, 10, 10, costs::LETHAL);
        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL);
        // Single cell < MIN_CLUSTER_CELLS (3), should be filtered
        assert!(obs.is_empty());
    }

    #[test]
    fn test_small_cluster() {
        let mut cells = make_grid(20, 20);
        // 3-cell L-shaped cluster at (5,5), (6,5), (5,6)
        set_cell(&mut cells, 20, 5, 5, costs::LETHAL);
        set_cell(&mut cells, 20, 6, 5, costs::LETHAL);
        set_cell(&mut cells, 20, 5, 6, costs::LETHAL);

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].id, 0);
        assert_eq!(obs[0].cell_count, 3);

        // Area = 3 cells * 0.1 * 0.1 = 0.03 m^2
        assert!((obs[0].area - 0.03).abs() < 0.001);

        // Bounding box: cells (5,5) to (6,6) -> world (0.5, 0.5) to (0.7, 0.7)
        assert!((obs[0].bbox_min_x - 0.5).abs() < 0.001);
        assert!((obs[0].bbox_min_y - 0.5).abs() < 0.001);
        assert!((obs[0].bbox_max_x - 0.7).abs() < 0.001);
        assert!((obs[0].bbox_max_y - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_two_separate_clusters() {
        let mut cells = make_grid(20, 20);
        // Cluster A: 4 cells at (2,2)-(3,3)
        set_cell(&mut cells, 20, 2, 2, costs::LETHAL);
        set_cell(&mut cells, 20, 3, 2, costs::LETHAL);
        set_cell(&mut cells, 20, 2, 3, costs::LETHAL);
        set_cell(&mut cells, 20, 3, 3, costs::LETHAL);

        // Cluster B: 3 cells at (15,15)-(16,15)-(15,16)
        set_cell(&mut cells, 20, 15, 15, costs::LETHAL);
        set_cell(&mut cells, 20, 16, 15, costs::LETHAL);
        set_cell(&mut cells, 20, 15, 16, costs::LETHAL);

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL);
        assert_eq!(obs.len(), 2);

        // Sorted by area descending: cluster A (4 cells) first
        assert_eq!(obs[0].cell_count, 4);
        assert_eq!(obs[0].id, 0);
        assert_eq!(obs[1].cell_count, 3);
        assert_eq!(obs[1].id, 1);
    }

    #[test]
    fn test_centroid_world_coords() {
        let mut cells = make_grid(20, 20);
        // Horizontal bar: (5,10), (6,10), (7,10), (8,10)
        for gx in 5..=8 {
            set_cell(&mut cells, 20, gx, 10, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 20, 20, 0.1, -1.0, -1.0, costs::LETHAL);
        assert_eq!(obs.len(), 1);

        // Grid centroid: gx = (5+6+7+8)/4 = 6.5, gy = 10
        // World: x = -1.0 + (6.5 + 0.5) * 0.1 = -0.3, y = -1.0 + (10.0 + 0.5) * 0.1 = 0.05
        assert!((obs[0].centroid_x - (-0.3)).abs() < 0.001);
        assert!((obs[0].centroid_y - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_threshold_filters_lower_costs() {
        let mut cells = make_grid(20, 20);
        // INSCRIBED cells (253) should NOT be picked up with LETHAL threshold
        set_cell(&mut cells, 20, 5, 5, costs::INSCRIBED);
        set_cell(&mut cells, 20, 6, 5, costs::INSCRIBED);
        set_cell(&mut cells, 20, 5, 6, costs::INSCRIBED);

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL);
        assert!(obs.is_empty());

        // But with INSCRIBED threshold, they should be found
        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::INSCRIBED);
        assert_eq!(obs.len(), 1);
    }

    #[test]
    fn test_diagonal_cells_not_connected() {
        let mut cells = make_grid(20, 20);
        // Diagonal cells (4-connected means these are separate)
        set_cell(&mut cells, 20, 5, 5, costs::LETHAL);
        set_cell(&mut cells, 20, 6, 6, costs::LETHAL);
        set_cell(&mut cells, 20, 7, 7, costs::LETHAL);

        let obs = extract_obstacles(&cells, 20, 20, 0.1, 0.0, 0.0, costs::LETHAL);
        // Each is a single cell < MIN_CLUSTER_CELLS, all filtered
        assert!(obs.is_empty());
    }

    #[test]
    fn test_max_obstacles_truncation() {
        let mut cells = make_grid(200, 200);
        // Create 70 separate 3-cell clusters (exceeds MAX_OBSTACLES = 64)
        for i in 0..70 {
            let base_x = (i % 20) * 5;
            let base_y = (i / 20) * 5;
            set_cell(&mut cells, 200, base_x, base_y, costs::LETHAL);
            set_cell(&mut cells, 200, base_x + 1, base_y, costs::LETHAL);
            set_cell(&mut cells, 200, base_x, base_y + 1, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 200, 200, 0.1, -10.0, -10.0, costs::LETHAL);
        assert!(obs.len() <= MAX_OBSTACLES);
    }

    // ====================================================================
    // Classification tests
    // ====================================================================

    fn make_obstacle(area: f32, w: f32, h: f32) -> Obstacle {
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
        }
    }

    #[test]
    fn test_classify_pole() {
        // Small, compact: area < 0.1, aspect < 3
        let obs = make_obstacle(0.05, 0.2, 0.2);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pole);
    }

    #[test]
    fn test_classify_wall() {
        // Long and thin: aspect > 4
        let obs = make_obstacle(0.5, 3.0, 0.3);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Wall);
    }

    #[test]
    fn test_classify_vehicle() {
        // Large area (> 1.0 m²)
        let obs = make_obstacle(2.0, 2.0, 1.5);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Vehicle);
    }

    #[test]
    fn test_classify_pedestrian() {
        // Medium area (0.1 - 1.0), compact shape
        let obs = make_obstacle(0.3, 0.5, 0.6);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Pedestrian);
    }

    #[test]
    fn test_classify_debris() {
        // Medium area but somewhat elongated (not wall, not compact enough for pedestrian)
        let obs = make_obstacle(0.4, 1.2, 0.4);
        assert_eq!(classify_obstacle(&obs), ObstacleClass::Debris);
    }

    #[test]
    fn test_classification_integrated() {
        // Create a grid with obstacles of different sizes and verify classification
        let mut cells = make_grid(100, 100);
        // Small 3-cell cluster (pole-sized at 0.1m resolution: 0.03 m²)
        set_cell(&mut cells, 100, 10, 10, costs::LETHAL);
        set_cell(&mut cells, 100, 11, 10, costs::LETHAL);
        set_cell(&mut cells, 100, 10, 11, costs::LETHAL);

        // Long wall: 20 cells in a line (0.2 m² area, 2.0m x 0.1m)
        for gx in 50..70 {
            set_cell(&mut cells, 100, gx, 50, costs::LETHAL);
        }

        let obs = extract_obstacles(&cells, 100, 100, 0.1, 0.0, 0.0, costs::LETHAL);
        assert_eq!(obs.len(), 2);

        // Sorted by area: wall (0.2 m²) first, then pole (0.03 m²)
        assert_eq!(obs[0].class, ObstacleClass::Wall);
        assert_eq!(obs[1].class, ObstacleClass::Pole);
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
}
