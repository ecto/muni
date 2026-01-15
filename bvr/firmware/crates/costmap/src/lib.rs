//! Occupancy grid and costmap for navigation.
//!
//! Provides:
//! - Log-odds occupancy grid with Bayesian updates
//! - Multi-layer costmap (static, obstacle, inflation)
//! - Raytrace for integrating LiDAR scans
//!
//! The costmap is used for path planning and obstacle avoidance.

use lidar::LaserScan;
use nalgebra::Vector2;
use thiserror::Error;
use transforms::Transform2D;

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
        Self {
            data: vec![0; width * height],
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

    /// Integrate a LiDAR scan into the grid using raycasting.
    pub fn integrate_scan(&mut self, scan: &LaserScan, robot_pose: &Transform2D) {
        let sensor_pos = robot_pose.translation();

        for (i, &range) in scan.ranges.iter().enumerate() {
            // Skip invalid ranges
            if !range.is_finite() || range < scan.range_min || range > scan.range_max {
                continue;
            }

            // Compute endpoint in world frame
            let angle = i as f32 * scan.angle_increment;
            let local_x = range * angle.cos();
            let local_y = range * angle.sin();
            let endpoint = robot_pose.transform_point(Vector2::new(local_x as f64, local_y as f64));

            // Determine if we hit an obstacle (not max range)
            let hit_obstacle = range < scan.range_max - 0.1;

            // Raytrace from sensor to endpoint
            self.raytrace(sensor_pos, endpoint, hit_obstacle);
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

    /// Update obstacle layer from a LiDAR scan.
    pub fn update_obstacles(&mut self, scan: &LaserScan, robot_pose: &Transform2D) {
        self.obstacle_layer.integrate_scan(scan, robot_pose);
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

    /// Inflate obstacles by the inflation radius.
    fn inflate(&mut self) {
        let inflation_cells = (self.inflation_radius / self.resolution).ceil() as i32;
        let inscribed_cells = (self.inscribed_radius / self.resolution).ceil() as i32;

        // Find lethal cells and inflate
        let lethal_cells: Vec<(usize, usize)> = self
            .combined
            .iter()
            .enumerate()
            .filter(|(_, &cost)| cost >= costs::LETHAL)
            .map(|(idx, _)| (idx % self.width, idx / self.width))
            .collect();

        let mut inflated = self.combined.clone();

        for (cx, cy) in lethal_cells {
            for dy in -inflation_cells..=inflation_cells {
                for dx in -inflation_cells..=inflation_cells {
                    let x = cx as i32 + dx;
                    let y = cy as i32 + dy;

                    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                        continue;
                    }

                    let dist_sq = dx * dx + dy * dy;
                    let dist = (dist_sq as f64).sqrt();

                    let cost = if dist_sq == 0 {
                        costs::LETHAL
                    } else if dist <= inscribed_cells as f64 {
                        costs::INSCRIBED
                    } else if dist <= inflation_cells as f64 {
                        // Exponential decay
                        let scale = 1.0 - (dist - inscribed_cells as f64) / (inflation_cells - inscribed_cells) as f64;
                        (costs::INSCRIBED as f64 * scale * scale) as u8
                    } else {
                        continue;
                    };

                    let idx = y as usize * self.width + x as usize;
                    inflated[idx] = inflated[idx].max(cost);
                }
            }
        }

        self.combined = inflated;
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
}
