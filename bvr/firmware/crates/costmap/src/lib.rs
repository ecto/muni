//! Occupancy grid and costmap for navigation.
//!
//! Provides:
//! - Log-odds occupancy grid with Bayesian updates
//! - Multi-layer costmap (static, obstacle, inflation)
//! - Raytrace for integrating LiDAR scans
//!
//! The costmap is used for path planning and obstacle avoidance.

pub mod clustering;

use lidar::PointCloud;
use nalgebra::{Vector2, Vector3};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use thiserror::Error;
use tracing::debug;
use transforms::Transform2D;

pub use clustering::{Obstacle, ObstacleClass};

#[derive(Error, Debug)]
pub enum CostmapError {
    #[error("Point out of bounds: ({x}, {y})")]
    OutOfBounds { x: f64, y: f64 },
    #[error("Invalid resolution: {0}")]
    InvalidResolution(f64),
}

/// Cost values for costmap cells.
pub mod costs {
    pub const FREE: u8 = 0;
    pub const UNKNOWN: u8 = 128;
    pub const INSCRIBED: u8 = 253;
    pub const LETHAL: u8 = 254;
    pub const NO_INFORMATION: u8 = 255;
}

/// Configuration for ground plane segmentation.
#[derive(Debug, Clone)]
pub struct GroundSegmenterConfig {
    /// Maximum point-to-plane distance to be considered ground (meters)
    pub distance_threshold: f64,
    /// Minimum fraction of candidate points on the ground plane
    pub min_inlier_ratio: f64,
    /// Number of RANSAC iterations
    pub max_iterations: usize,
}

impl Default for GroundSegmenterConfig {
    fn default() -> Self {
        Self {
            distance_threshold: 0.15,
            min_inlier_ratio: 0.3,
            max_iterations: 25,
        }
    }
}

/// RANSAC-based ground plane segmenter.
///
/// Fits a plane to candidate low points and classifies the cloud
/// into ground vs obstacle points. More robust than a hard z-threshold
/// because it handles slopes and uneven terrain.
pub struct GroundSegmenter {
    config: GroundSegmenterConfig,
    rng: SmallRng,
}

impl std::fmt::Debug for GroundSegmenter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroundSegmenter")
            .field("config", &self.config)
            .finish()
    }
}

impl GroundSegmenter {
    /// Create a new ground segmenter.
    pub fn new(config: GroundSegmenterConfig) -> Self {
        Self {
            config,
            rng: SmallRng::from_entropy(),
        }
    }

    /// Segment a point cloud into ground and obstacle points.
    ///
    /// Returns `(ground_3d, obstacle_3d)` — world-frame XY with
    /// original Z preserved for height-annotated obstacle detection.
    pub fn segment(
        &mut self,
        cloud: &PointCloud,
        robot_pose: &Transform2D,
    ) -> (Vec<Vector3<f64>>, Vec<Vector3<f64>>) {
        const MAX_RANGE: f32 = 50.0;
        const GROUND_CANDIDATE_MAX_Z: f32 = 0.5;
        const OBSTACLE_MIN_ABOVE_GROUND: f64 = 0.1;
        const OBSTACLE_MAX_ABOVE_GROUND: f64 = 2.0;
        // Maximum angle from vertical for a valid ground normal (30 degrees)
        const MAX_NORMAL_ANGLE_COS: f64 = 0.866; // cos(30°)

        // Filter valid points and separate ground candidates
        let mut candidates_3d: Vec<Vector3<f64>> = Vec::new();
        let mut all_valid: Vec<(Vector3<f64>, Vector2<f64>)> = Vec::new();

        for point in &cloud.points {
            let range_sq = point.x * point.x + point.y * point.y;
            if !range_sq.is_finite() || range_sq < 0.01 || range_sq > MAX_RANGE * MAX_RANGE {
                continue;
            }

            let world_2d =
                robot_pose.transform_point(Vector2::new(point.x as f64, point.y as f64));
            let p3 = Vector3::new(world_2d.x, world_2d.y, point.z as f64);

            all_valid.push((p3, world_2d));

            if point.z < GROUND_CANDIDATE_MAX_Z {
                candidates_3d.push(p3);
            }
        }

        if candidates_3d.len() < 3 || all_valid.is_empty() {
            // Not enough data for RANSAC — fall back: everything is obstacle
            let obstacles = all_valid
                .iter()
                .filter(|(p3, _)| p3.z > -0.3 && p3.z < 1.5)
                .map(|(p3, _)| *p3)
                .collect();
            return (Vec::new(), obstacles);
        }

        // RANSAC plane fitting
        let mut best_plane: Option<(Vector3<f64>, f64)> = None; // (normal, d) where normal·p + d = 0
        let mut best_inlier_count = 0;

        for _ in 0..self.config.max_iterations {
            // Sample 3 random points
            let sample: Vec<&Vector3<f64>> = candidates_3d
                .choose_multiple(&mut self.rng, 3)
                .collect();

            if sample.len() < 3 {
                break;
            }

            // Fit plane through 3 points
            let v1 = sample[1] - sample[0];
            let v2 = sample[2] - sample[0];
            let normal = v1.cross(&v2);
            let norm = normal.norm();
            if norm < 1e-10 {
                continue; // Degenerate (collinear points)
            }
            let normal = normal / norm;

            // Check normal is roughly vertical (z component dominant)
            if normal.z.abs() < MAX_NORMAL_ANGLE_COS {
                continue;
            }

            // Ensure normal points upward
            let normal = if normal.z < 0.0 { -normal } else { normal };
            let d = -normal.dot(sample[0]);

            // Count inliers
            let inlier_count = candidates_3d
                .iter()
                .filter(|p| (normal.dot(p) + d).abs() < self.config.distance_threshold)
                .count();

            if inlier_count > best_inlier_count {
                best_inlier_count = inlier_count;
                best_plane = Some((normal, d));
            }
        }

        let inlier_ratio = best_inlier_count as f64 / candidates_3d.len() as f64;

        // If RANSAC found a good plane, use it; otherwise fall back to z-threshold
        let (ground, obstacles) = if let Some((normal, d)) = best_plane {
            if inlier_ratio >= self.config.min_inlier_ratio {
                debug!(
                    inliers = best_inlier_count,
                    total = candidates_3d.len(),
                    ratio = inlier_ratio,
                    normal_z = normal.z,
                    "Ground plane fitted"
                );

                let mut ground = Vec::new();
                let mut obstacles = Vec::new();

                for (p3, _p2) in &all_valid {
                    let dist_to_plane = normal.dot(p3) + d;

                    if dist_to_plane.abs() < self.config.distance_threshold {
                        // On the ground plane
                        ground.push(*p3);
                    } else if dist_to_plane > OBSTACLE_MIN_ABOVE_GROUND
                        && dist_to_plane < OBSTACLE_MAX_ABOVE_GROUND
                    {
                        // Above ground plane — obstacle
                        obstacles.push(*p3);
                    }
                    // Below ground or too high — ignore
                }

                (ground, obstacles)
            } else {
                // Poor plane fit — fall back to z-threshold
                self.fallback_segment(&all_valid)
            }
        } else {
            self.fallback_segment(&all_valid)
        };

        (ground, obstacles)
    }

    /// Fallback segmentation using simple z-threshold when RANSAC fails.
    fn fallback_segment(
        &self,
        points: &[(Vector3<f64>, Vector2<f64>)],
    ) -> (Vec<Vector3<f64>>, Vec<Vector3<f64>>) {
        let mut ground = Vec::new();
        let mut obstacles = Vec::new();

        for (p3, _p2) in points {
            if p3.z < 0.1 && p3.z > -0.3 {
                ground.push(*p3);
            } else if p3.z >= 0.1 && p3.z < 1.5 {
                obstacles.push(*p3);
            }
        }

        (ground, obstacles)
    }
}

/// Occupancy grid with log-odds representation.
///
/// Uses log-odds for efficient Bayesian updates:
/// - Positive values = occupied
/// - Negative values = free
/// - Zero = unknown
#[derive(Debug, Clone)]
pub struct OccupancyGrid {
    /// Grid data in log-odds (-128 to 127)
    data: Vec<i8>,
    /// Per-cell minimum observed Z (height), meters. INFINITY = no observation.
    height_min: Vec<f32>,
    /// Per-cell maximum observed Z (height), meters. NEG_INFINITY = no observation.
    height_max: Vec<f32>,
    /// Grid width in cells
    pub width: usize,
    /// Grid height in cells
    pub height: usize,
    /// Cell resolution in meters
    pub resolution: f64,
    /// Origin in world frame (bottom-left corner)
    pub origin: Vector2<f64>,
    /// Log-odds update for hit (occupied)
    log_odds_hit: i8,
    /// Log-odds update for miss (free)
    log_odds_miss: i8,
    /// Maximum log-odds (clamp)
    log_odds_max: i8,
    /// Minimum log-odds (clamp)
    log_odds_min: i8,
}

impl OccupancyGrid {
    /// Create a new occupancy grid.
    ///
    /// # Arguments
    /// * `width` - Grid width in cells
    /// * `height` - Grid height in cells
    /// * `resolution` - Cell size in meters
    /// * `origin` - World coordinates of bottom-left corner
    pub fn new(width: usize, height: usize, resolution: f64, origin: Vector2<f64>) -> Self {
        let total = width * height;
        Self {
            data: vec![0; total],
            height_min: vec![f32::INFINITY; total],
            height_max: vec![f32::NEG_INFINITY; total],
            width,
            height,
            resolution,
            origin,
            log_odds_hit: 20,   // ~0.75 probability
            log_odds_miss: -10, // ~0.45 probability
            log_odds_max: 100,  // ~0.9999 probability
            log_odds_min: -100, // ~0.0001 probability
        }
    }

    /// Create a centered grid.
    pub fn centered(width_m: f64, height_m: f64, resolution: f64) -> Self {
        let width = (width_m / resolution).ceil() as usize;
        let height = (height_m / resolution).ceil() as usize;
        let origin = Vector2::new(-width_m / 2.0, -height_m / 2.0);
        Self::new(width, height, resolution, origin)
    }

    /// Clear the grid to unknown.
    pub fn clear(&mut self) {
        self.data.fill(0);
        self.height_min.fill(f32::INFINITY);
        self.height_max.fill(f32::NEG_INFINITY);
    }

    /// Convert world coordinates to grid cell.
    pub fn world_to_grid(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let local_x = x - self.origin.x;
        let local_y = y - self.origin.y;

        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }

        let gx = (local_x / self.resolution) as usize;
        let gy = (local_y / self.resolution) as usize;

        if gx < self.width && gy < self.height {
            Some((gx, gy))
        } else {
            None
        }
    }

    /// Convert grid cell to world coordinates (cell center).
    pub fn grid_to_world(&self, gx: usize, gy: usize) -> (f64, f64) {
        let x = self.origin.x + (gx as f64 + 0.5) * self.resolution;
        let y = self.origin.y + (gy as f64 + 0.5) * self.resolution;
        (x, y)
    }

    /// Get cell index from grid coordinates.
    fn cell_index(&self, gx: usize, gy: usize) -> Option<usize> {
        if gx < self.width && gy < self.height {
            Some(gy * self.width + gx)
        } else {
            None
        }
    }

    /// Get log-odds value at grid cell.
    pub fn get_log_odds(&self, gx: usize, gy: usize) -> Option<i8> {
        self.cell_index(gx, gy).map(|idx| self.data[idx])
    }

    /// Get occupancy probability at grid cell (0.0 = free, 1.0 = occupied).
    pub fn get_probability(&self, gx: usize, gy: usize) -> Option<f64> {
        self.get_log_odds(gx, gy).map(log_odds_to_probability)
    }

    /// Get occupancy probability at world coordinates.
    pub fn get_probability_at(&self, x: f64, y: f64) -> Option<f64> {
        self.world_to_grid(x, y)
            .and_then(|(gx, gy)| self.get_probability(gx, gy))
    }

    /// Update a cell with a new observation.
    pub fn update_cell(&mut self, gx: usize, gy: usize, occupied: bool) {
        if let Some(idx) = self.cell_index(gx, gy) {
            let update = if occupied {
                self.log_odds_hit
            } else {
                self.log_odds_miss
            };
            let current = self.data[idx] as i16;
            let new_value = (current + update as i16).clamp(self.log_odds_min as i16, self.log_odds_max as i16);
            self.data[idx] = new_value as i8;
        }
    }

    /// Expand the height envelope for a cell.
    fn update_height(&mut self, gx: usize, gy: usize, z: f32) {
        if let Some(idx) = self.cell_index(gx, gy) {
            if z < self.height_min[idx] {
                self.height_min[idx] = z;
            }
            if z > self.height_max[idx] {
                self.height_max[idx] = z;
            }
        }
    }

    /// Expose per-cell height envelope for clustering.
    pub fn height_data(&self) -> (&[f32], &[f32]) {
        (&self.height_min, &self.height_max)
    }

    /// Integrate a LiDAR point cloud into the grid using raycasting.
    ///
    /// Projects 3D points to 2D ground plane and filters by height.
    pub fn integrate_scan(&mut self, cloud: &PointCloud, robot_pose: &Transform2D) {
        let sensor_pos = robot_pose.translation();

        // Height filter: only consider points near ground plane
        // Mid360 is mounted at ~0.5m height, so filter for -0.3m to 1.5m
        const MIN_Z: f32 = -0.3;
        const MAX_Z: f32 = 1.5;
        const MAX_RANGE: f32 = 50.0;

        for point in &cloud.points {
            // Skip points outside height range
            if point.z < MIN_Z || point.z > MAX_Z {
                continue;
            }

            // Compute 2D range (distance in ground plane)
            let range = (point.x * point.x + point.y * point.y).sqrt();

            // Skip invalid ranges
            if !range.is_finite() || range < 0.1 || range > MAX_RANGE {
                continue;
            }

            // Transform point from sensor frame to world frame
            let endpoint = robot_pose.transform_point(Vector2::new(point.x as f64, point.y as f64));

            // Points within max range are considered obstacle hits
            let hit_obstacle = range < MAX_RANGE - 1.0;

            // Raytrace from sensor to endpoint, recording height for obstacle hits
            if hit_obstacle {
                self.raytrace_with_height(sensor_pos, endpoint, true, point.z);
            } else {
                self.raytrace(sensor_pos, endpoint, false);
            }
        }
    }

    /// Integrate a LiDAR point cloud using ground plane segmentation.
    ///
    /// Uses RANSAC-based ground segmentation instead of a hard z-threshold.
    /// Ground points produce free-space rays, obstacle points produce hits
    /// with Z height recorded for the height envelope.
    pub fn integrate_scan_segmented(
        &mut self,
        cloud: &PointCloud,
        robot_pose: &Transform2D,
        segmenter: &mut GroundSegmenter,
    ) {
        let sensor_pos = robot_pose.translation();
        let (ground_pts, obstacle_pts) = segmenter.segment(cloud, robot_pose);

        // Ground points: raytrace as free space (no height needed)
        for pt in &ground_pts {
            self.raytrace(sensor_pos, Vector2::new(pt.x, pt.y), false);
        }

        // Obstacle points: raytrace with hit at endpoint, recording Z
        for pt in &obstacle_pts {
            self.raytrace_with_height(
                sensor_pos,
                Vector2::new(pt.x, pt.y),
                true,
                pt.z as f32,
            );
        }
    }

    /// Raytrace a line, marking cells as free and the endpoint as occupied.
    fn raytrace(&mut self, start: Vector2<f64>, end: Vector2<f64>, hit_obstacle: bool) {
        let start_cell = match self.world_to_grid(start.x, start.y) {
            Some(c) => c,
            None => return,
        };
        let end_cell = match self.world_to_grid(end.x, end.y) {
            Some(c) => c,
            None => return,
        };

        // Bresenham's line algorithm
        let mut x = start_cell.0 as i32;
        let mut y = start_cell.1 as i32;
        let x1 = end_cell.0 as i32;
        let y1 = end_cell.1 as i32;

        let dx = (x1 - x).abs();
        let dy = (y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            // Mark current cell as free (except endpoint)
            if x != x1 || y != y1 {
                if x >= 0 && y >= 0 {
                    self.update_cell(x as usize, y as usize, false);
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        // Mark endpoint as occupied (if we hit something)
        if hit_obstacle {
            if end_cell.0 < self.width && end_cell.1 < self.height {
                self.update_cell(end_cell.0, end_cell.1, true);
            }
        }
    }

    /// Raytrace with height tracking at the endpoint.
    fn raytrace_with_height(
        &mut self,
        start: Vector2<f64>,
        end: Vector2<f64>,
        hit_obstacle: bool,
        z: f32,
    ) {
        let start_cell = match self.world_to_grid(start.x, start.y) {
            Some(c) => c,
            None => return,
        };
        let end_cell = match self.world_to_grid(end.x, end.y) {
            Some(c) => c,
            None => return,
        };

        // Bresenham's line algorithm
        let mut x = start_cell.0 as i32;
        let mut y = start_cell.1 as i32;
        let x1 = end_cell.0 as i32;
        let y1 = end_cell.1 as i32;

        let dx = (x1 - x).abs();
        let dy = (y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            // Mark current cell as free (except endpoint)
            if x != x1 || y != y1 {
                if x >= 0 && y >= 0 {
                    self.update_cell(x as usize, y as usize, false);
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        // Mark endpoint as occupied (if we hit something)
        if hit_obstacle {
            if end_cell.0 < self.width && end_cell.1 < self.height {
                self.update_cell(end_cell.0, end_cell.1, true);
                self.update_height(end_cell.0, end_cell.1, z);
            }
        }
    }

    /// Convert to a cost grid (0-254).
    pub fn to_cost_grid(&self) -> Vec<u8> {
        self.data
            .iter()
            .map(|&log_odds| {
                let prob = log_odds_to_probability(log_odds);
                if prob > 0.65 {
                    costs::LETHAL
                } else if prob > 0.5 {
                    ((prob - 0.5) * 2.0 * 200.0) as u8
                } else {
                    costs::FREE
                }
            })
            .collect()
    }

    /// Get grid width in cells.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get grid height in cells.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get cell resolution in meters.
    pub fn resolution(&self) -> f64 {
        self.resolution
    }

    /// Get world origin (bottom-left corner).
    pub fn origin(&self) -> Vector2<f64> {
        self.origin
    }

    /// Get probability at grid cell (convenience wrapper).
    pub fn probability(&self, gx: usize, gy: usize) -> f64 {
        self.get_probability(gx, gy).unwrap_or(0.5)
    }

    /// Get raw data (for serialization/visualization).
    pub fn raw_data(&self) -> &[i8] {
        &self.data
    }

    /// Decay all cells toward zero (unknown) by `step`.
    /// Call once per scan integration to fade stale observations.
    /// Resets height envelope when a cell's log-odds reaches zero.
    pub fn decay(&mut self, step: i8) {
        let s = step as i16;
        for i in 0..self.data.len() {
            let v = self.data[i] as i16;
            if v > 0 {
                let new_val = (v - s).max(0) as i8;
                self.data[i] = new_val;
                if new_val == 0 {
                    self.height_min[i] = f32::INFINITY;
                    self.height_max[i] = f32::NEG_INFINITY;
                }
            } else if v < 0 {
                self.data[i] = (v + s).min(0) as i8;
            }
        }
    }

    /// Convert to PNG-compatible grayscale (0 = occupied, 255 = free, 128 = unknown).
    pub fn to_grayscale(&self) -> Vec<u8> {
        self.data
            .iter()
            .map(|&log_odds| {
                let prob = log_odds_to_probability(log_odds);
                // Invert: 0 = occupied (black), 255 = free (white)
                ((1.0 - prob) * 255.0) as u8
            })
            .collect()
    }
}

/// Owned snapshot of costmap state for streaming.
#[derive(Debug, Clone)]
pub struct CostmapSnapshot {
    /// Combined cost data (0=free, 253=inscribed, 254=lethal)
    pub cells: Vec<u8>,
    /// Grid width in cells
    pub width: u16,
    /// Grid height in cells
    pub height: u16,
    /// Cell resolution in meters
    pub resolution: f32,
    /// Origin X in world frame (meters)
    pub origin_x: f32,
    /// Origin Y in world frame (meters)
    pub origin_y: f32,
}

/// Multi-layer costmap combining static map, obstacles, and inflation.
#[derive(Debug)]
pub struct Costmap {
    /// Static layer (from pre-built map)
    static_layer: Option<OccupancyGrid>,
    /// Dynamic obstacle layer (from recent LiDAR)
    obstacle_layer: OccupancyGrid,
    /// Combined costmap after inflation
    combined: Vec<u8>,
    /// Grid dimensions and origin (same as obstacle_layer)
    pub width: usize,
    pub height: usize,
    pub resolution: f64,
    pub origin: Vector2<f64>,
    /// Inflation radius in meters
    inflation_radius: f64,
    /// Robot inscribed radius in meters
    inscribed_radius: f64,
    /// RANSAC ground segmenter for LiDAR integration
    ground_segmenter: GroundSegmenter,
    /// Log-odds decay per scan integration (fades stale obstacles)
    decay_step: i8,
}

impl Costmap {
    /// Create a new costmap.
    pub fn new(
        width_m: f64,
        height_m: f64,
        resolution: f64,
        inflation_radius: f64,
        inscribed_radius: f64,
    ) -> Self {
        let obstacle_layer = OccupancyGrid::centered(width_m, height_m, resolution);
        let width = obstacle_layer.width;
        let height = obstacle_layer.height;
        let origin = obstacle_layer.origin;

        Self {
            static_layer: None,
            obstacle_layer,
            combined: vec![costs::FREE; width * height],
            width,
            height,
            resolution,
            origin,
            inflation_radius,
            inscribed_radius,
            ground_segmenter: GroundSegmenter::new(GroundSegmenterConfig::default()),
            decay_step: 5,
        }
    }

    /// Set the static layer from a pre-built map.
    pub fn set_static_layer(&mut self, map: OccupancyGrid) {
        self.static_layer = Some(map);
        self.recompute();
    }

    /// Clear the obstacle layer.
    pub fn clear_obstacles(&mut self) {
        self.obstacle_layer.clear();
    }

    /// Update obstacle layer from a LiDAR point cloud.
    ///
    /// Decays stale observations before integrating the new scan using
    /// RANSAC ground segmentation (rather than a naive z-threshold).
    pub fn update_obstacles(&mut self, scan: &PointCloud, robot_pose: &Transform2D) {
        self.obstacle_layer.decay(self.decay_step);
        self.obstacle_layer
            .integrate_scan_segmented(scan, robot_pose, &mut self.ground_segmenter);
        self.recompute();
    }

    /// Recompute the combined costmap with inflation.
    fn recompute(&mut self) {
        // Start with static layer or free
        if let Some(ref static_map) = self.static_layer {
            self.combined = static_map.to_cost_grid();
        } else {
            self.combined.fill(costs::FREE);
        }

        // Add obstacle layer
        let obstacle_costs = self.obstacle_layer.to_cost_grid();
        for (i, &cost) in obstacle_costs.iter().enumerate() {
            self.combined[i] = self.combined[i].max(cost);
        }

        // Apply inflation
        self.inflate();
    }

    /// Inflate obstacles by the inflation radius using distance transform.
    ///
    /// Uses a two-pass distance transform (O(n) where n = total cells) instead of
    /// the naive O(k*m²) approach where k = lethal cells, m = inflation radius.
    /// This makes inflation fast even with dense LiDAR scans.
    fn inflate(&mut self) {
        let inflation_cells = (self.inflation_radius / self.resolution).ceil() as usize;
        let inscribed_cells = (self.inscribed_radius / self.resolution).ceil() as usize;

        if inflation_cells == 0 {
            return;
        }

        let w = self.width;
        let h = self.height;

        // Distance transform using two-pass algorithm (Rosenfeld & Pfaltz)
        // Initialize distance grid: 0 for lethal cells, infinity for others
        let inf = (w + h) as u32; // Large enough "infinity"
        let mut dist: Vec<u32> = self
            .combined
            .iter()
            .map(|&cost| if cost >= costs::LETHAL { 0 } else { inf })
            .collect();

        // Forward pass: propagate from top-left
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let mut d = dist[idx];

                // Check left neighbor
                if x > 0 {
                    d = d.min(dist[idx - 1].saturating_add(1));
                }
                // Check top neighbor
                if y > 0 {
                    d = d.min(dist[idx - w].saturating_add(1));
                }
                // Check diagonal neighbors (for better Euclidean approximation)
                if x > 0 && y > 0 {
                    // Approximate diagonal distance as 1.4 (~sqrt(2))
                    d = d.min(dist[idx - w - 1].saturating_add(1));
                }
                if x < w - 1 && y > 0 {
                    d = d.min(dist[idx - w + 1].saturating_add(1));
                }

                dist[idx] = d;
            }
        }

        // Backward pass: propagate from bottom-right
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let idx = y * w + x;
                let mut d = dist[idx];

                // Check right neighbor
                if x < w - 1 {
                    d = d.min(dist[idx + 1].saturating_add(1));
                }
                // Check bottom neighbor
                if y < h - 1 {
                    d = d.min(dist[idx + w].saturating_add(1));
                }
                // Check diagonal neighbors
                if x < w - 1 && y < h - 1 {
                    d = d.min(dist[idx + w + 1].saturating_add(1));
                }
                if x > 0 && y < h - 1 {
                    d = d.min(dist[idx + w - 1].saturating_add(1));
                }

                dist[idx] = d;
            }
        }

        // Apply costs based on distance
        let inflation_cells = inflation_cells as u32;
        let inscribed_cells = inscribed_cells as u32;

        for (idx, &d) in dist.iter().enumerate() {
            let cost = if d == 0 {
                costs::LETHAL
            } else if d <= inscribed_cells {
                costs::INSCRIBED
            } else if d <= inflation_cells {
                // Exponential decay from inscribed to inflation radius
                let scale = 1.0 - (d - inscribed_cells) as f64 / (inflation_cells - inscribed_cells) as f64;
                (costs::INSCRIBED as f64 * scale * scale) as u8
            } else {
                continue; // Don't modify cells beyond inflation radius
            };

            self.combined[idx] = self.combined[idx].max(cost);
        }
    }

    /// Get cost at world position.
    pub fn get_cost(&self, x: f64, y: f64) -> u8 {
        let local_x = x - self.origin.x;
        let local_y = y - self.origin.y;

        if local_x < 0.0 || local_y < 0.0 {
            return costs::NO_INFORMATION;
        }

        let gx = (local_x / self.resolution) as usize;
        let gy = (local_y / self.resolution) as usize;

        if gx < self.width && gy < self.height {
            self.combined[gy * self.width + gx]
        } else {
            costs::NO_INFORMATION
        }
    }

    /// Check if a position is collision-free.
    pub fn is_free(&self, x: f64, y: f64) -> bool {
        self.get_cost(x, y) < costs::INSCRIBED
    }

    /// Get raw combined costmap data.
    pub fn raw_data(&self) -> &[u8] {
        &self.combined
    }

    /// Get the obstacle layer's occupancy grid.
    pub fn obstacle_grid(&self) -> &OccupancyGrid {
        &self.obstacle_layer
    }

    /// Extract discrete obstacles from the costmap.
    ///
    /// Uses connected-component labeling on LETHAL cells to identify
    /// obstacle clusters with bounding boxes and centroids.
    pub fn extract_obstacles(&self) -> Vec<Obstacle> {
        let (h_min, h_max) = self.obstacle_layer.height_data();
        clustering::extract_obstacles(
            &self.combined,
            self.width,
            self.height,
            self.resolution,
            self.origin.x,
            self.origin.y,
            costs::LETHAL,
            h_min,
            h_max,
        )
    }

    /// Create an owned snapshot of the current costmap state for streaming.
    pub fn snapshot(&self) -> CostmapSnapshot {
        CostmapSnapshot {
            cells: self.combined.clone(),
            width: self.width as u16,
            height: self.height as u16,
            resolution: self.resolution as f32,
            origin_x: self.origin.x as f32,
            origin_y: self.origin.y as f32,
        }
    }
}

/// Convert log-odds to probability.
fn log_odds_to_probability(log_odds: i8) -> f64 {
    let l = log_odds as f64 / 10.0; // Scale factor
    1.0 / (1.0 + (-l).exp())
}

/// Convert probability to log-odds.
#[allow(dead_code)]
fn probability_to_log_odds(p: f64) -> i8 {
    let p = p.clamp(0.001, 0.999);
    let l = (p / (1.0 - p)).ln() * 10.0;
    l.clamp(-127.0, 127.0) as i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occupancy_grid_creation() {
        let grid = OccupancyGrid::new(100, 100, 0.1, Vector2::new(0.0, 0.0));
        assert_eq!(grid.width, 100);
        assert_eq!(grid.height, 100);
        assert_eq!(grid.resolution, 0.1);
    }

    #[test]
    fn test_centered_grid() {
        let grid = OccupancyGrid::centered(10.0, 10.0, 0.1);
        assert_eq!(grid.width, 100);
        assert_eq!(grid.height, 100);
        assert_eq!(grid.origin, Vector2::new(-5.0, -5.0));
    }

    #[test]
    fn test_world_to_grid() {
        let grid = OccupancyGrid::new(100, 100, 0.1, Vector2::new(0.0, 0.0));

        assert_eq!(grid.world_to_grid(0.0, 0.0), Some((0, 0)));
        assert_eq!(grid.world_to_grid(0.05, 0.05), Some((0, 0)));
        assert_eq!(grid.world_to_grid(0.15, 0.15), Some((1, 1)));
        assert_eq!(grid.world_to_grid(9.95, 9.95), Some((99, 99)));
        assert_eq!(grid.world_to_grid(-0.1, 0.0), None);
        assert_eq!(grid.world_to_grid(10.1, 0.0), None);
    }

    #[test]
    fn test_grid_to_world() {
        let grid = OccupancyGrid::new(100, 100, 0.1, Vector2::new(0.0, 0.0));

        let (x, y) = grid.grid_to_world(0, 0);
        assert!((x - 0.05).abs() < 0.001);
        assert!((y - 0.05).abs() < 0.001);

        let (x, y) = grid.grid_to_world(10, 10);
        assert!((x - 1.05).abs() < 0.001);
        assert!((y - 1.05).abs() < 0.001);
    }

    #[test]
    fn test_cell_update() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        // Initially unknown (0.5 probability)
        let prob = grid.get_probability(5, 5).unwrap();
        assert!((prob - 0.5).abs() < 0.01);

        // Mark as occupied
        grid.update_cell(5, 5, true);
        let prob = grid.get_probability(5, 5).unwrap();
        assert!(prob > 0.5);

        // Mark as free multiple times
        for _ in 0..10 {
            grid.update_cell(5, 5, false);
        }
        let prob = grid.get_probability(5, 5).unwrap();
        assert!(prob < 0.5);
    }

    #[test]
    fn test_log_odds_conversion() {
        assert!((log_odds_to_probability(0) - 0.5).abs() < 0.01);
        assert!(log_odds_to_probability(100) > 0.99);
        assert!(log_odds_to_probability(-100) < 0.01);
    }

    #[test]
    fn test_costmap_creation() {
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.5, 0.3);
        assert_eq!(costmap.width, 100);
        assert_eq!(costmap.height, 100);
    }

    #[test]
    fn test_costmap_free_space() {
        let costmap = Costmap::new(10.0, 10.0, 0.1, 0.5, 0.3);

        // Initially all free (centered grid: -5 to +5)
        assert!(costmap.is_free(0.0, 0.0));
        assert!(costmap.is_free(4.5, 4.5));
        assert!(costmap.is_free(-4.5, -4.5));
        assert_eq!(costmap.get_cost(0.0, 0.0), costs::FREE);
    }

    #[test]
    fn test_ground_segmenter_flat_plane() {
        use lidar::Point3D;

        let mut segmenter = GroundSegmenter::new(GroundSegmenterConfig::default());

        // Create a flat ground plane at z=0 with some obstacles at z=1
        let mut points = Vec::new();

        // Ground points
        for ix in -10..=10 {
            for iy in -10..=10 {
                points.push(Point3D {
                    x: ix as f32 * 0.5,
                    y: iy as f32 * 0.5,
                    z: 0.0,
                    reflectivity: 128,
                    tag: 0,
                });
            }
        }

        // Obstacle points (wall at x=3, z=0.5..1.5)
        for iy in -5..=5 {
            for iz in 2..=6 {
                points.push(Point3D {
                    x: 3.0,
                    y: iy as f32 * 0.5,
                    z: iz as f32 * 0.25,
                    reflectivity: 128,
                    tag: 0,
                });
            }
        }

        let cloud = PointCloud {
            points,
            ..Default::default()
        };

        let (ground, obstacles) = segmenter.segment(&cloud, &Transform2D::identity());

        // Should have ground points
        assert!(!ground.is_empty(), "Expected ground points");
        // Should have obstacle points
        assert!(!obstacles.is_empty(), "Expected obstacle points");
        // Ground should be majority of points
        assert!(
            ground.len() > obstacles.len(),
            "Expected more ground ({}) than obstacles ({})",
            ground.len(),
            obstacles.len()
        );
    }

    #[test]
    fn test_ground_segmenter_empty_cloud() {
        let mut segmenter = GroundSegmenter::new(GroundSegmenterConfig::default());
        let cloud = PointCloud::default();
        let (ground, obstacles) = segmenter.segment(&cloud, &Transform2D::identity());
        assert!(ground.is_empty());
        assert!(obstacles.is_empty());
    }

    #[test]
    fn test_ground_segmenter_all_high_points() {
        use lidar::Point3D;

        let mut segmenter = GroundSegmenter::new(GroundSegmenterConfig::default());

        // All points above ground candidate threshold — should fall back
        let points: Vec<Point3D> = (0..100)
            .map(|i| Point3D {
                x: (i as f32 * 0.1) + 1.0,
                y: 0.0,
                z: 2.0, // Too high for ground candidates
                reflectivity: 128,
                tag: 0,
            })
            .collect();

        let cloud = PointCloud {
            points,
            ..Default::default()
        };

        let (ground, obstacles) = segmenter.segment(&cloud, &Transform2D::identity());
        // All points are above 1.5m in fallback, so nothing classified
        assert!(ground.is_empty());
        assert!(obstacles.is_empty());
    }

    #[test]
    fn test_decay_toward_zero() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        // Set some cells to positive (occupied) and negative (free)
        grid.update_cell(2, 2, true); // +20
        grid.update_cell(5, 5, false); // -10

        let before_occ = grid.get_log_odds(2, 2).unwrap();
        let before_free = grid.get_log_odds(5, 5).unwrap();
        assert!(before_occ > 0);
        assert!(before_free < 0);

        grid.decay(5);

        let after_occ = grid.get_log_odds(2, 2).unwrap();
        let after_free = grid.get_log_odds(5, 5).unwrap();

        // Positive cell decayed toward zero
        assert!(after_occ < before_occ);
        assert!(after_occ > 0); // 20 - 5 = 15, still positive

        // Negative cell decayed toward zero
        assert!(after_free > before_free);
        assert!(after_free < 0); // -10 + 5 = -5, still negative
    }

    #[test]
    fn test_decay_does_not_overshoot_zero() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        // Set a cell to a small positive value
        grid.data[0] = 2;
        grid.decay(5);
        // saturating_sub: 2 - 5 would be negative, but saturating keeps at 0
        assert_eq!(grid.data[0], 0);

        // Set a cell to a small negative value
        grid.data[1] = -3;
        grid.decay(5);
        // saturating_add: -3 + 5 would be positive, but saturating keeps at 0
        assert_eq!(grid.data[1], 0);
    }

    // ====================================================================
    // Height envelope tests
    // ====================================================================

    #[test]
    fn test_update_height_tracks_min_max() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        // Initially sentinel values
        let (h_min, h_max) = grid.height_data();
        assert_eq!(h_min[0], f32::INFINITY);
        assert_eq!(h_max[0], f32::NEG_INFINITY);

        // First observation
        grid.update_height(0, 0, 0.5);
        let (h_min, h_max) = grid.height_data();
        assert!((h_min[0] - 0.5).abs() < 0.001);
        assert!((h_max[0] - 0.5).abs() < 0.001);

        // Expand envelope
        grid.update_height(0, 0, 0.2);
        grid.update_height(0, 0, 1.3);
        let (h_min, h_max) = grid.height_data();
        assert!((h_min[0] - 0.2).abs() < 0.001);
        assert!((h_max[0] - 1.3).abs() < 0.001);
    }

    #[test]
    fn test_decay_resets_height_at_zero() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        // Set cell to small positive log-odds and give it height
        grid.data[0] = 3;
        grid.update_height(0, 0, 0.5);
        grid.update_height(0, 0, 1.2);

        // Decay enough to reach zero
        grid.decay(5);
        assert_eq!(grid.data[0], 0);

        // Height should be reset to sentinel
        let (h_min, h_max) = grid.height_data();
        assert_eq!(h_min[0], f32::INFINITY);
        assert_eq!(h_max[0], f32::NEG_INFINITY);
    }

    #[test]
    fn test_clear_resets_height() {
        let mut grid = OccupancyGrid::new(10, 10, 1.0, Vector2::new(0.0, 0.0));

        grid.update_height(5, 5, 0.8);
        grid.clear();

        let (h_min, h_max) = grid.height_data();
        let idx = 5 * 10 + 5;
        assert_eq!(h_min[idx], f32::INFINITY);
        assert_eq!(h_max[idx], f32::NEG_INFINITY);
    }
}
