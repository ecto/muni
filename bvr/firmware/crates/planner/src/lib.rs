//! A* path planner for autonomous navigation.
//!
//! Provides grid-based A* search on the costmap for collision-free path planning.
//! Outputs a sequence of waypoints that the local planner (pure pursuit) can follow.
//!
//! # Features
//!
//! - 8-connected grid search (diagonal movement allowed)
//! - Respects costmap inflation layer
//! - Efficient priority queue implementation
//! - Replanning on path obstruction

use costmap::{costs, Costmap};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use thiserror::Error;
use tracing::{debug, warn};
use types::Pose;

/// Errors that can occur during path planning.
#[derive(Error, Debug, Clone)]
pub enum PlannerError {
    #[error("Start position ({x}, {y}) is in collision")]
    StartInCollision { x: f64, y: f64 },

    #[error("Goal position ({x}, {y}) is in collision")]
    GoalInCollision { x: f64, y: f64 },

    #[error("Start position ({x}, {y}) is outside map bounds")]
    StartOutOfBounds { x: f64, y: f64 },

    #[error("Goal position ({x}, {y}) is outside map bounds")]
    GoalOutOfBounds { x: f64, y: f64 },

    #[error("No path found from start to goal")]
    NoPathFound,

    #[error("Costmap not available")]
    NoCostmap,
}

/// Planner configuration.
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    /// Maximum cost to consider traversable (0-253, 254=lethal, 255=unknown)
    pub max_traversable_cost: u8,
    /// Cost penalty for cells near obstacles (multiplier)
    pub proximity_cost_weight: f64,
    /// Whether to allow diagonal movement
    pub allow_diagonal: bool,
    /// Path simplification tolerance (meters) - removes collinear waypoints
    pub simplification_tolerance: f64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_traversable_cost: costs::INSCRIBED - 1, // 252 - just under inscribed
            proximity_cost_weight: 0.5,
            allow_diagonal: true,
            simplification_tolerance: 0.1,
        }
    }
}

/// A waypoint in the planned path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Waypoint {
    /// X position in world frame (meters)
    pub x: f64,
    /// Y position in world frame (meters)
    pub y: f64,
}

impl Waypoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Distance to another waypoint.
    pub fn distance_to(&self, other: &Waypoint) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<Waypoint> for [f64; 2] {
    fn from(wp: Waypoint) -> [f64; 2] {
        [wp.x, wp.y]
    }
}

impl From<[f64; 2]> for Waypoint {
    fn from(arr: [f64; 2]) -> Self {
        Self { x: arr[0], y: arr[1] }
    }
}

/// A planned path as a sequence of waypoints.
#[derive(Debug, Clone)]
pub struct Path {
    /// Waypoints from start to goal.
    pub waypoints: Vec<Waypoint>,
    /// Total path length in meters.
    pub length: f64,
}

impl Path {
    /// Create an empty path.
    pub fn empty() -> Self {
        Self {
            waypoints: Vec::new(),
            length: 0.0,
        }
    }

    /// Check if path is empty.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// Number of waypoints.
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Get the goal (last waypoint).
    pub fn goal(&self) -> Option<Waypoint> {
        self.waypoints.last().copied()
    }
}

/// Grid cell for A* search.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Cell {
    x: i32,
    y: i32,
}

impl Cell {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Priority queue node for A* search.
#[derive(Debug, Clone)]
struct Node {
    cell: Cell,
    f_score: f64, // g + h
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (lower f_score = higher priority)
        other.f_score.partial_cmp(&self.f_score).unwrap_or(Ordering::Equal)
    }
}

/// A* path planner.
pub struct Planner {
    config: PlannerConfig,
}

impl Planner {
    /// Create a new planner with the given configuration.
    pub fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    /// Create a planner with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(PlannerConfig::default())
    }

    /// Plan a path from start pose to goal position.
    ///
    /// Returns a path of waypoints from start to goal, avoiding obstacles.
    pub fn plan(
        &self,
        costmap: &Costmap,
        start: &Pose,
        goal: Waypoint,
    ) -> Result<Path, PlannerError> {
        let start_wp = Waypoint::new(start.x, start.y);

        // Convert to grid coordinates
        let start_cell = self.world_to_grid(costmap, start_wp)?;
        let goal_cell = self.world_to_grid(costmap, goal)?;

        // Check start and goal validity
        let start_cost = self.get_cost(costmap, start_cell);
        if start_cost >= costs::LETHAL {
            return Err(PlannerError::StartInCollision { x: start.x, y: start.y });
        }

        let goal_cost = self.get_cost(costmap, goal_cell);
        if goal_cost >= costs::LETHAL {
            return Err(PlannerError::GoalInCollision { x: goal.x, y: goal.y });
        }

        // A* search
        let grid_path = self.astar(costmap, start_cell, goal_cell)?;

        // Convert grid path to world coordinates
        let mut waypoints: Vec<Waypoint> = grid_path
            .iter()
            .map(|cell| self.grid_to_world(costmap, *cell))
            .collect();

        // Simplify path (remove collinear points)
        waypoints = self.simplify_path(waypoints);

        // Calculate total length
        let length = self.calculate_path_length(&waypoints);

        debug!(
            start_x = start.x,
            start_y = start.y,
            goal_x = goal.x,
            goal_y = goal.y,
            waypoints = waypoints.len(),
            length = format!("{:.2}", length),
            "Path planned"
        );

        Ok(Path { waypoints, length })
    }

    /// Check if the current path is blocked by obstacles.
    ///
    /// Returns the index of the first blocked waypoint, or None if path is clear.
    pub fn check_path_blocked(&self, costmap: &Costmap, path: &Path) -> Option<usize> {
        for (i, wp) in path.waypoints.iter().enumerate() {
            let cost = costmap.get_cost(wp.x, wp.y);
            if cost >= costs::LETHAL {
                return Some(i);
            }
        }
        None
    }

    /// Check if a specific position is in collision.
    pub fn is_collision(&self, costmap: &Costmap, x: f64, y: f64) -> bool {
        costmap.get_cost(x, y) >= costs::LETHAL
    }

    /// Check if there's an obstacle within the given radius of a position.
    pub fn obstacle_within_radius(
        &self,
        costmap: &Costmap,
        x: f64,
        y: f64,
        radius: f64,
    ) -> bool {
        // Sample points in a circle
        let samples = 8;
        for i in 0..samples {
            let angle = (i as f64 / samples as f64) * std::f64::consts::TAU;
            let check_x = x + radius * angle.cos();
            let check_y = y + radius * angle.sin();
            if costmap.get_cost(check_x, check_y) >= costs::LETHAL {
                return true;
            }
        }
        // Also check center
        costmap.get_cost(x, y) >= costs::LETHAL
    }

    /// A* search algorithm.
    fn astar(&self, costmap: &Costmap, start: Cell, goal: Cell) -> Result<Vec<Cell>, PlannerError> {
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<Cell, Cell> = HashMap::new();
        let mut g_score: HashMap<Cell, f64> = HashMap::new();

        g_score.insert(start, 0.0);
        open_set.push(Node {
            cell: start,
            f_score: self.heuristic(start, goal),
        });

        // 8-connected neighbors (or 4-connected if diagonal disabled)
        let neighbors: &[(i32, i32, f64)] = if self.config.allow_diagonal {
            &[
                (-1, 0, 1.0),
                (1, 0, 1.0),
                (0, -1, 1.0),
                (0, 1, 1.0),
                (-1, -1, std::f64::consts::SQRT_2),
                (-1, 1, std::f64::consts::SQRT_2),
                (1, -1, std::f64::consts::SQRT_2),
                (1, 1, std::f64::consts::SQRT_2),
            ]
        } else {
            &[(-1, 0, 1.0), (1, 0, 1.0), (0, -1, 1.0), (0, 1, 1.0)]
        };

        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100_000;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                warn!("A* exceeded maximum iterations");
                return Err(PlannerError::NoPathFound);
            }

            if current.cell == goal {
                // Reconstruct path
                return Ok(self.reconstruct_path(&came_from, current.cell));
            }

            let current_g = *g_score.get(&current.cell).unwrap_or(&f64::INFINITY);

            for (dx, dy, base_cost) in neighbors {
                let neighbor = Cell::new(current.cell.x + dx, current.cell.y + dy);

                // Check bounds
                if neighbor.x < 0
                    || neighbor.y < 0
                    || neighbor.x >= costmap.width as i32
                    || neighbor.y >= costmap.height as i32
                {
                    continue;
                }

                // Check traversability
                let cell_cost = self.get_cost(costmap, neighbor);
                if cell_cost >= costs::LETHAL {
                    continue;
                }

                // Calculate movement cost with proximity penalty
                let proximity_penalty = (cell_cost as f64 / 255.0) * self.config.proximity_cost_weight;
                let move_cost = base_cost * (1.0 + proximity_penalty);

                let tentative_g = current_g + move_cost;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f64::INFINITY) {
                    came_from.insert(neighbor, current.cell);
                    g_score.insert(neighbor, tentative_g);

                    let f = tentative_g + self.heuristic(neighbor, goal);
                    open_set.push(Node {
                        cell: neighbor,
                        f_score: f,
                    });
                }
            }
        }

        Err(PlannerError::NoPathFound)
    }

    /// Heuristic function (Euclidean distance).
    fn heuristic(&self, a: Cell, b: Cell) -> f64 {
        let dx = (b.x - a.x) as f64;
        let dy = (b.y - a.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    /// Reconstruct path from came_from map.
    fn reconstruct_path(&self, came_from: &HashMap<Cell, Cell>, goal: Cell) -> Vec<Cell> {
        let mut path = vec![goal];
        let mut current = goal;

        while let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    /// Convert world coordinates to grid cell.
    fn world_to_grid(&self, costmap: &Costmap, wp: Waypoint) -> Result<Cell, PlannerError> {
        let local_x = wp.x - costmap.origin.x;
        let local_y = wp.y - costmap.origin.y;

        if local_x < 0.0 || local_y < 0.0 {
            return Err(PlannerError::StartOutOfBounds { x: wp.x, y: wp.y });
        }

        let gx = (local_x / costmap.resolution) as i32;
        let gy = (local_y / costmap.resolution) as i32;

        if gx >= costmap.width as i32 || gy >= costmap.height as i32 {
            return Err(PlannerError::GoalOutOfBounds { x: wp.x, y: wp.y });
        }

        Ok(Cell::new(gx, gy))
    }

    /// Convert grid cell to world coordinates (cell center).
    fn grid_to_world(&self, costmap: &Costmap, cell: Cell) -> Waypoint {
        let x = costmap.origin.x + (cell.x as f64 + 0.5) * costmap.resolution;
        let y = costmap.origin.y + (cell.y as f64 + 0.5) * costmap.resolution;
        Waypoint::new(x, y)
    }

    /// Get cost at grid cell.
    fn get_cost(&self, costmap: &Costmap, cell: Cell) -> u8 {
        if cell.x < 0 || cell.y < 0 {
            return costs::NO_INFORMATION;
        }
        let (x, y) = (
            costmap.origin.x + (cell.x as f64 + 0.5) * costmap.resolution,
            costmap.origin.y + (cell.y as f64 + 0.5) * costmap.resolution,
        );
        costmap.get_cost(x, y)
    }

    /// Simplify path by removing collinear points.
    fn simplify_path(&self, path: Vec<Waypoint>) -> Vec<Waypoint> {
        if path.len() <= 2 {
            return path;
        }

        let mut simplified = vec![path[0]];
        let tol = self.config.simplification_tolerance;

        for i in 1..path.len() - 1 {
            let prev = simplified.last().unwrap();
            let curr = &path[i];
            let next = &path[i + 1];

            // Check if curr is collinear with prev and next using cross product
            let dx1 = curr.x - prev.x;
            let dy1 = curr.y - prev.y;
            let dx2 = next.x - curr.x;
            let dy2 = next.y - curr.y;

            let cross = (dx1 * dy2 - dy1 * dx2).abs();
            let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();
            let len2 = (dx2 * dx2 + dy2 * dy2).sqrt();

            // Distance from point to line
            let dist = if len1 > 0.0 && len2 > 0.0 {
                cross / len1.max(len2)
            } else {
                0.0
            };

            if dist > tol {
                simplified.push(*curr);
            }
        }

        simplified.push(*path.last().unwrap());
        simplified
    }

    /// Calculate total path length.
    fn calculate_path_length(&self, path: &[Waypoint]) -> f64 {
        if path.len() < 2 {
            return 0.0;
        }

        path.windows(2)
            .map(|w| w[0].distance_to(&w[1]))
            .sum()
    }
}

/// State for the reactive planner (manages replanning).
pub struct ReactivePlanner {
    planner: Planner,
    current_path: Option<Path>,
    current_waypoint_idx: usize,
    replan_distance: f64,
    goal: Option<Waypoint>,
}

impl ReactivePlanner {
    /// Create a new reactive planner.
    pub fn new(config: PlannerConfig, replan_distance: f64) -> Self {
        Self {
            planner: Planner::new(config),
            current_path: None,
            current_waypoint_idx: 0,
            replan_distance,
            goal: None,
        }
    }

    /// Set a new goal and plan path.
    pub fn set_goal(
        &mut self,
        costmap: &Costmap,
        start: &Pose,
        goal: Waypoint,
    ) -> Result<&Path, PlannerError> {
        self.goal = Some(goal);
        self.current_path = Some(self.planner.plan(costmap, start, goal)?);
        self.current_waypoint_idx = 0;
        Ok(self.current_path.as_ref().unwrap())
    }

    /// Clear the current goal and path.
    pub fn clear_goal(&mut self) {
        self.goal = None;
        self.current_path = None;
        self.current_waypoint_idx = 0;
    }

    /// Get current path.
    pub fn path(&self) -> Option<&Path> {
        self.current_path.as_ref()
    }

    /// Get current waypoint index.
    pub fn waypoint_index(&self) -> usize {
        self.current_waypoint_idx
    }

    /// Get current target waypoint.
    pub fn current_waypoint(&self) -> Option<Waypoint> {
        self.current_path
            .as_ref()
            .and_then(|p| p.waypoints.get(self.current_waypoint_idx).copied())
    }

    /// Get the goal.
    pub fn goal(&self) -> Option<Waypoint> {
        self.goal
    }

    /// Update planner state. Returns the current target waypoint.
    ///
    /// This should be called every control loop iteration. It:
    /// 1. Advances waypoint if we're close enough
    /// 2. Checks if path is blocked and triggers replan
    /// 3. Returns current target waypoint (or None if goal reached/no path)
    pub fn update(
        &mut self,
        costmap: &Costmap,
        current_pose: &Pose,
    ) -> Result<Option<Waypoint>, PlannerError> {
        let path = match &self.current_path {
            Some(p) => p,
            None => return Ok(None),
        };

        if path.is_empty() {
            return Ok(None);
        }

        // Check if we've reached current waypoint
        if let Some(wp) = path.waypoints.get(self.current_waypoint_idx) {
            let dx = wp.x - current_pose.x;
            let dy = wp.y - current_pose.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < self.replan_distance {
                self.current_waypoint_idx += 1;

                // Check if we've reached the goal
                if self.current_waypoint_idx >= path.waypoints.len() {
                    debug!("Goal reached");
                    return Ok(None);
                }
            }
        }

        // Check if path ahead is blocked
        if let Some(blocked_idx) = self.planner.check_path_blocked(costmap, path) {
            // Only replan if the blocked waypoint is ahead of us
            if blocked_idx >= self.current_waypoint_idx {
                debug!(
                    blocked_idx,
                    current_idx = self.current_waypoint_idx,
                    "Path blocked, replanning"
                );

                // Replan from current position to goal
                if let Some(goal) = self.goal {
                    self.current_path = Some(self.planner.plan(costmap, current_pose, goal)?);
                    self.current_waypoint_idx = 0;
                }
            }
        }

        Ok(self.current_waypoint())
    }

    /// Check if there's an immediate obstacle (for reactive stopping).
    pub fn check_immediate_obstacle(
        &self,
        costmap: &Costmap,
        pose: &Pose,
        lookahead: f64,
    ) -> bool {
        // Check in front of robot
        let check_x = pose.x + lookahead * pose.theta.cos();
        let check_y = pose.y + lookahead * pose.theta.sin();
        self.planner.is_collision(costmap, check_x, check_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_costmap() -> Costmap {
        // 10m x 10m costmap at 0.1m resolution, centered at origin
        Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2)
    }

    #[test]
    fn test_planner_creation() {
        let planner = Planner::with_defaults();
        assert!(planner.config.allow_diagonal);
    }

    #[test]
    fn test_plan_empty_costmap() {
        let planner = Planner::with_defaults();
        let costmap = make_costmap();

        let start = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let goal = Waypoint::new(2.0, 2.0);

        let path = planner.plan(&costmap, &start, goal).unwrap();

        assert!(!path.is_empty());
        assert!(path.length > 0.0);
        // Should be relatively direct in empty costmap
        assert!(path.waypoints.len() < 10);
    }

    #[test]
    fn test_waypoint_distance() {
        let a = Waypoint::new(0.0, 0.0);
        let b = Waypoint::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_path_simplification() {
        let planner = Planner::with_defaults();

        // Three collinear points
        let path = vec![
            Waypoint::new(0.0, 0.0),
            Waypoint::new(1.0, 1.0),
            Waypoint::new(2.0, 2.0),
        ];

        let simplified = planner.simplify_path(path);

        // Middle point should be removed (collinear)
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn test_reactive_planner() {
        let config = PlannerConfig::default();
        let mut reactive = ReactivePlanner::new(config, 0.5);
        let costmap = make_costmap();

        let start = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let goal = Waypoint::new(2.0, 0.0);

        reactive.set_goal(&costmap, &start, goal).unwrap();

        assert!(reactive.path().is_some());
        assert!(reactive.current_waypoint().is_some());
    }
}
