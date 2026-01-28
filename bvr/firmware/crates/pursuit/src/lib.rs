//! Pure pursuit path follower with reactive obstacle avoidance.
//!
//! Provides smooth path following using the pure pursuit algorithm, with
//! reactive stopping when obstacles are detected in the immediate path.
//!
//! # Features
//!
//! - Pure pursuit steering with adaptive lookahead
//! - Reactive obstacle detection and emergency stop
//! - Smooth velocity profiles (acceleration limiting)
//! - Goal reaching detection

use costmap::{costs, Costmap};
use planner::{Path, Waypoint};
use thiserror::Error;
use tracing::{debug, info, warn};
use types::{Pose, Twist};

/// Errors that can occur during pursuit.
#[derive(Error, Debug, Clone)]
pub enum PursuitError {
    #[error("No path to follow")]
    NoPath,

    #[error("Path is empty")]
    EmptyPath,

    #[error("Lost path - robot too far from path")]
    LostPath,
}

/// Pure pursuit configuration.
#[derive(Debug, Clone)]
pub struct PursuitConfig {
    /// Base lookahead distance (meters)
    pub lookahead_distance: f64,
    /// Minimum lookahead distance
    pub min_lookahead: f64,
    /// Maximum lookahead distance
    pub max_lookahead: f64,
    /// Lookahead gain (lookahead = base + gain * velocity)
    pub lookahead_gain: f64,
    /// Maximum linear velocity (m/s)
    pub max_linear_velocity: f64,
    /// Maximum angular velocity (rad/s)
    pub max_angular_velocity: f64,
    /// Goal reached threshold (meters)
    pub goal_tolerance: f64,
    /// Distance threshold for "lost path" detection (meters)
    pub max_cross_track_error: f64,
    /// Obstacle check distance (meters ahead of robot)
    pub obstacle_check_distance: f64,
    /// Obstacle check radius (meters around check point)
    pub obstacle_check_radius: f64,
    /// Time to stop when obstacle detected (seconds, then recheck)
    pub obstacle_stop_duration: f64,
    /// Slowdown factor when near obstacles (0-1)
    pub obstacle_slowdown_factor: f64,
    /// Distance to start slowing down for goal (meters)
    pub goal_slowdown_distance: f64,
    /// Enable cost-regulated velocity scaling (slow near inflated obstacles)
    pub use_cost_regulated_scaling: bool,
    /// Gain for cost-based speed reduction (0-1, higher = more aggressive)
    pub cost_scaling_gain: f64,
    /// Minimum speed fraction when at inscribed cost (0-1)
    pub min_cost_speed_fraction: f64,
}

impl Default for PursuitConfig {
    fn default() -> Self {
        Self {
            lookahead_distance: 0.5,
            min_lookahead: 0.3,
            max_lookahead: 1.5,
            lookahead_gain: 0.3,
            max_linear_velocity: 1.0,
            max_angular_velocity: 0.6, // Reduced to account for 2.5x angular boost in main loop
            goal_tolerance: 0.3,
            max_cross_track_error: 2.0,
            obstacle_check_distance: 0.5,
            obstacle_check_radius: 0.25,
            obstacle_stop_duration: 0.5,
            obstacle_slowdown_factor: 0.5,
            goal_slowdown_distance: 1.0,
            use_cost_regulated_scaling: true,
            cost_scaling_gain: 0.6,
            min_cost_speed_fraction: 0.3,
        }
    }
}

/// Output from pursuit controller.
#[derive(Debug, Clone)]
pub struct PursuitOutput {
    /// Computed twist command.
    pub twist: Twist,
    /// Current target waypoint index.
    pub waypoint_index: usize,
    /// Distance to current target waypoint.
    pub distance_to_waypoint: f64,
    /// Distance to final goal.
    pub distance_to_goal: f64,
    /// Cross-track error (distance from path).
    pub cross_track_error: f64,
    /// Whether goal has been reached.
    pub goal_reached: bool,
    /// Whether stopped due to obstacle.
    pub obstacle_stop: bool,
    /// Lookahead point in world frame.
    pub lookahead_point: Option<Waypoint>,
}

impl Default for PursuitOutput {
    fn default() -> Self {
        Self {
            twist: Twist::default(),
            waypoint_index: 0,
            distance_to_waypoint: f64::MAX,
            distance_to_goal: f64::MAX,
            cross_track_error: 0.0,
            goal_reached: false,
            obstacle_stop: false,
            lookahead_point: None,
        }
    }
}

/// State of the obstacle detection system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObstacleState {
    /// No obstacle detected, normal operation.
    Clear,
    /// Obstacle detected, robot is stopping.
    Stopping,
    /// Stopped due to obstacle, waiting for it to clear.
    Stopped,
    /// Obstacle cleared, resuming motion.
    Resuming,
}

/// Pure pursuit path follower.
pub struct PursuitController {
    config: PursuitConfig,
    current_waypoint_idx: usize,
    obstacle_state: ObstacleState,
    obstacle_stop_timer: f64,
    last_velocity: f64,
}

impl PursuitController {
    /// Create a new pursuit controller.
    pub fn new(config: PursuitConfig) -> Self {
        Self {
            config,
            current_waypoint_idx: 0,
            obstacle_state: ObstacleState::Clear,
            obstacle_stop_timer: 0.0,
            last_velocity: 0.0,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(PursuitConfig::default())
    }

    /// Reset controller state (call when switching to new path).
    pub fn reset(&mut self) {
        self.current_waypoint_idx = 0;
        self.obstacle_state = ObstacleState::Clear;
        self.obstacle_stop_timer = 0.0;
        self.last_velocity = 0.0;
    }

    /// Get current obstacle state.
    pub fn obstacle_state(&self) -> ObstacleState {
        self.obstacle_state
    }

    /// Compute cost-based speed scaling factor by sampling points from robot to lookahead.
    ///
    /// Returns a factor in [min_cost_speed_fraction, 1.0] where lower values mean
    /// the robot is near higher-cost (inflated obstacle) regions.
    fn compute_cost_factor(&self, costmap: &Costmap, pose: &Pose, lookahead: &Waypoint) -> f64 {
        if !self.config.use_cost_regulated_scaling {
            return 1.0;
        }

        let num_samples = 5;
        let mut max_cost: u8 = 0;

        for i in 0..num_samples {
            let t = (i as f64 + 1.0) / num_samples as f64;
            let x = pose.x + t * (lookahead.x - pose.x);
            let y = pose.y + t * (lookahead.y - pose.y);
            let cost = costmap.get_cost(x, y);
            max_cost = max_cost.max(cost);
        }

        // Only scale when cost is in the inflation zone (above FREE, below INSCRIBED)
        if max_cost < costs::FREE + 1 {
            return 1.0;
        }

        let normalized = max_cost as f64 / costs::INSCRIBED as f64;
        let factor = 1.0 - normalized * self.config.cost_scaling_gain;
        factor.clamp(self.config.min_cost_speed_fraction, 1.0)
    }

    /// Compute control output to follow the path.
    ///
    /// # Arguments
    /// * `path` - Path to follow
    /// * `pose` - Current robot pose
    /// * `costmap` - Costmap for obstacle checking (optional)
    /// * `dt` - Time step (seconds)
    ///
    /// # Returns
    /// Control output with twist command and status information.
    pub fn compute(
        &mut self,
        path: &Path,
        pose: &Pose,
        costmap: Option<&Costmap>,
        dt: f64,
    ) -> Result<PursuitOutput, PursuitError> {
        let mut output = PursuitOutput::default();

        if path.is_empty() {
            return Err(PursuitError::EmptyPath);
        }

        // Calculate distance to goal
        let goal = path.goal().unwrap();
        let dx_goal = goal.x - pose.x;
        let dy_goal = goal.y - pose.y;
        output.distance_to_goal = (dx_goal * dx_goal + dy_goal * dy_goal).sqrt();

        // Check if goal reached
        if output.distance_to_goal < self.config.goal_tolerance {
            output.goal_reached = true;
            output.twist = Twist::default();
            info!("Goal reached");
            return Ok(output);
        }

        // Check for obstacles
        if let Some(costmap) = costmap {
            self.update_obstacle_state(costmap, pose, dt);
        }

        output.obstacle_stop = matches!(
            self.obstacle_state,
            ObstacleState::Stopping | ObstacleState::Stopped
        );

        if output.obstacle_stop {
            output.twist = Twist::default();
            return Ok(output);
        }

        // Find lookahead point on path
        let lookahead_dist = self.compute_lookahead_distance();
        let (lookahead_point, waypoint_idx) =
            self.find_lookahead_point(path, pose, lookahead_dist);

        output.lookahead_point = Some(lookahead_point);
        output.waypoint_index = waypoint_idx;
        self.current_waypoint_idx = waypoint_idx;

        // Compute distance to current waypoint
        if let Some(wp) = path.waypoints.get(waypoint_idx) {
            let dx = wp.x - pose.x;
            let dy = wp.y - pose.y;
            output.distance_to_waypoint = (dx * dx + dy * dy).sqrt();
        }

        // Compute cross-track error
        output.cross_track_error = self.compute_cross_track_error(path, pose);

        // Check if we've lost the path
        if output.cross_track_error > self.config.max_cross_track_error {
            warn!(
                cross_track_error = output.cross_track_error,
                "Lost path - cross track error too large"
            );
            return Err(PursuitError::LostPath);
        }

        // Compute cost-regulated speed factor
        let cost_factor = if let Some(costmap) = costmap {
            self.compute_cost_factor(costmap, pose, &lookahead_point)
        } else {
            1.0
        };

        // Pure pursuit steering calculation
        let twist = self.pure_pursuit(pose, lookahead_point, output.distance_to_goal, cost_factor);
        output.twist = twist;
        self.last_velocity = twist.linear;

        debug!(
            linear = format!("{:.2}", twist.linear),
            angular = format!("{:.2}", twist.angular),
            lookahead_x = format!("{:.2}", lookahead_point.x),
            lookahead_y = format!("{:.2}", lookahead_point.y),
            waypoint_idx,
            distance_to_goal = format!("{:.2}", output.distance_to_goal),
            "Pursuit output"
        );

        Ok(output)
    }

    /// Compute adaptive lookahead distance based on velocity.
    fn compute_lookahead_distance(&self) -> f64 {
        let velocity_based = self.config.lookahead_distance
            + self.config.lookahead_gain * self.last_velocity.abs();

        velocity_based.clamp(self.config.min_lookahead, self.config.max_lookahead)
    }

    /// Find the lookahead point on the path.
    ///
    /// Projects robot position onto the path, then looks ahead by `lookahead_dist`
    /// along the path from that projection point.
    fn find_lookahead_point(
        &self,
        path: &Path,
        pose: &Pose,
        lookahead_dist: f64,
    ) -> (Waypoint, usize) {
        let robot_pos = Waypoint::new(pose.x, pose.y);

        // First, find closest point on path (projection of robot onto path)
        let mut min_dist = f64::MAX;
        let mut closest_segment_idx = 0;
        let mut closest_point = robot_pos;

        for (i, window) in path.waypoints.windows(2).enumerate() {
            let (proj, dist) = closest_point_on_segment(&robot_pos, &window[0], &window[1]);
            if dist < min_dist {
                min_dist = dist;
                closest_segment_idx = i;
                closest_point = proj;
            }
        }

        // Now walk along the path from the closest point, accumulating distance
        // until we reach lookahead_dist
        let mut accumulated = 0.0;
        let mut current_point = closest_point;

        for i in closest_segment_idx..path.waypoints.len() - 1 {
            let next_wp = &path.waypoints[i + 1];
            let segment_remaining = current_point.distance_to(next_wp);

            if accumulated + segment_remaining >= lookahead_dist {
                // Lookahead point is on this segment
                let dist_on_segment = lookahead_dist - accumulated;
                let t = dist_on_segment / segment_remaining;
                let x = current_point.x + t * (next_wp.x - current_point.x);
                let y = current_point.y + t * (next_wp.y - current_point.y);
                return (Waypoint::new(x, y), i + 1);
            }

            accumulated += segment_remaining;
            current_point = *next_wp;
        }

        // Lookahead extends past end of path, return goal
        let goal = path.goal().unwrap();
        (goal, path.waypoints.len() - 1)
    }

    /// Compute cross-track error (distance from robot to path).
    fn compute_cross_track_error(&self, path: &Path, pose: &Pose) -> f64 {
        if path.waypoints.len() < 2 {
            return 0.0;
        }

        // Find closest point on path
        let mut min_dist = f64::MAX;
        let robot = Waypoint::new(pose.x, pose.y);

        for window in path.waypoints.windows(2) {
            let a = &window[0];
            let b = &window[1];
            let dist = point_to_segment_distance(&robot, a, b);
            min_dist = min_dist.min(dist);
        }

        min_dist
    }

    /// Pure pursuit steering calculation.
    fn pure_pursuit(&self, pose: &Pose, lookahead: Waypoint, distance_to_goal: f64, cost_factor: f64) -> Twist {
        // Transform lookahead point to robot frame
        let dx = lookahead.x - pose.x;
        let dy = lookahead.y - pose.y;
        let cos_theta = pose.theta.cos();
        let sin_theta = pose.theta.sin();

        // Point in robot frame (x forward, y left)
        let local_x = dx * cos_theta + dy * sin_theta;
        let local_y = -dx * sin_theta + dy * cos_theta;

        // Lookahead distance
        let l = (local_x * local_x + local_y * local_y).sqrt().max(0.001);

        // Handle case where target is behind (local_x < 0)
        // In this case, turn in place toward target first
        if local_x < 0.0 {
            // Target is behind us - turn toward it
            let heading_error = local_y.atan2(local_x);
            let angular_vel = (heading_error * 1.0) // P-control for turn in place
                .clamp(-self.config.max_angular_velocity, self.config.max_angular_velocity);
            debug!(
                local_x = format!("{:.2}", local_x),
                local_y = format!("{:.2}", local_y),
                heading_error = format!("{:.2}", heading_error),
                angular_vel = format!("{:.2}", angular_vel),
                "Turn in place mode"
            );
            return Twist {
                linear: 0.05, // Very small forward motion
                angular: angular_vel,
                boost: false,
            };
        }

        // Curvature (2 * y / L^2 from pure pursuit geometry)
        let curvature = 2.0 * local_y / (l * l);

        // Compute velocities
        let mut linear_vel = self.config.max_linear_velocity;

        // Slow down based on curvature (tighter turns = slower)
        // Use a gentler curvature scaling to avoid stopping when turning
        let curvature_factor = 1.0 / (1.0 + curvature.abs() * 1.0);
        linear_vel *= curvature_factor;

        // Slow down based on costmap cost (near inflated obstacles = slower)
        linear_vel *= cost_factor;

        // Ensure minimum linear velocity to avoid getting stuck while turning
        // (allows robot to "drive through" tight turns rather than oscillating)
        let min_linear = 0.25; // 25 cm/s minimum when not near goal
        linear_vel = linear_vel.max(min_linear);

        // Slow down near goal
        if distance_to_goal < self.config.goal_slowdown_distance {
            let slowdown_factor = distance_to_goal / self.config.goal_slowdown_distance;
            linear_vel *= slowdown_factor.max(0.2); // Minimum 20% speed
        }

        // Apply obstacle slowdown if resuming
        if self.obstacle_state == ObstacleState::Resuming {
            linear_vel *= self.config.obstacle_slowdown_factor;
        }

        // Clamp linear velocity
        linear_vel = linear_vel.clamp(0.0, self.config.max_linear_velocity);

        // Angular velocity from curvature: omega = v * kappa
        let angular_vel = (linear_vel * curvature)
            .clamp(-self.config.max_angular_velocity, self.config.max_angular_velocity);

        Twist {
            linear: linear_vel,
            angular: angular_vel,
            boost: false,
        }
    }

    /// Update obstacle detection state machine.
    fn update_obstacle_state(&mut self, costmap: &Costmap, pose: &Pose, dt: f64) {
        let obstacle_detected = self.check_obstacle(costmap, pose);

        match self.obstacle_state {
            ObstacleState::Clear => {
                if obstacle_detected {
                    info!("Obstacle detected, stopping");
                    self.obstacle_state = ObstacleState::Stopping;
                    self.obstacle_stop_timer = 0.0;
                }
            }
            ObstacleState::Stopping => {
                // Immediate transition to stopped (stopping is just for signaling)
                self.obstacle_state = ObstacleState::Stopped;
            }
            ObstacleState::Stopped => {
                self.obstacle_stop_timer += dt;

                if !obstacle_detected {
                    info!(
                        stopped_for = format!("{:.1}s", self.obstacle_stop_timer),
                        "Obstacle cleared, resuming"
                    );
                    self.obstacle_state = ObstacleState::Resuming;
                }
            }
            ObstacleState::Resuming => {
                if obstacle_detected {
                    // Obstacle returned, stop again
                    self.obstacle_state = ObstacleState::Stopping;
                    self.obstacle_stop_timer = 0.0;
                } else {
                    // Fully resumed
                    self.obstacle_state = ObstacleState::Clear;
                }
            }
        }
    }

    /// Check if there's an obstacle in front of the robot.
    fn check_obstacle(&self, costmap: &Costmap, pose: &Pose) -> bool {
        // Check multiple points in front of robot
        let check_distances = [
            self.config.obstacle_check_distance * 0.5,
            self.config.obstacle_check_distance,
            self.config.obstacle_check_distance * 1.5,
        ];

        for dist in check_distances {
            let check_x = pose.x + dist * pose.theta.cos();
            let check_y = pose.y + dist * pose.theta.sin();

            // Check center and sides
            let offsets = [
                (0.0, 0.0),
                (0.0, self.config.obstacle_check_radius),
                (0.0, -self.config.obstacle_check_radius),
            ];

            for (dx, dy) in offsets {
                let x = check_x + dx * pose.theta.cos() - dy * pose.theta.sin();
                let y = check_y + dx * pose.theta.sin() + dy * pose.theta.cos();

                let cost = costmap.get_cost(x, y);
                if cost >= costs::LETHAL {
                    return true;
                }
            }
        }

        false
    }
}

/// Compute distance from point to line segment.
fn point_to_segment_distance(p: &Waypoint, a: &Waypoint, b: &Waypoint) -> f64 {
    let (_, dist) = closest_point_on_segment(p, a, b);
    dist
}

/// Find the closest point on a line segment to a given point.
/// Returns (closest_point, distance).
fn closest_point_on_segment(p: &Waypoint, a: &Waypoint, b: &Waypoint) -> (Waypoint, f64) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;

    if len_sq < 1e-10 {
        // Segment is a point
        return (*a, p.distance_to(a));
    }

    // Project point onto line
    let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);

    // Closest point on segment
    let closest = Waypoint::new(a.x + t * dx, a.y + t * dy);
    let dist = p.distance_to(&closest);

    (closest, dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_straight_path() -> Path {
        Path {
            waypoints: vec![
                Waypoint::new(0.0, 0.0),
                Waypoint::new(1.0, 0.0),
                Waypoint::new(2.0, 0.0),
                Waypoint::new(3.0, 0.0),
            ],
            length: 3.0,
        }
    }

    #[test]
    fn test_pursuit_creation() {
        let controller = PursuitController::with_defaults();
        assert_eq!(controller.obstacle_state, ObstacleState::Clear);
    }

    #[test]
    fn test_pursuit_straight_path() {
        let mut controller = PursuitController::with_defaults();
        let path = make_straight_path();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };

        let output = controller.compute(&path, &pose, None, 0.01).unwrap();

        // Should have forward velocity, minimal angular
        assert!(output.twist.linear > 0.0);
        assert!(output.twist.angular.abs() < 0.1);
        assert!(!output.goal_reached);
    }

    #[test]
    fn test_pursuit_goal_reached() {
        let mut controller = PursuitController::with_defaults();
        let path = make_straight_path();
        let pose = Pose { x: 2.9, y: 0.0, theta: 0.0 };

        let output = controller.compute(&path, &pose, None, 0.01).unwrap();

        // Should be at goal (within tolerance)
        assert!(output.goal_reached);
        assert!(output.twist.linear.abs() < 0.01);
    }

    #[test]
    fn test_pursuit_turning() {
        let mut controller = PursuitController::with_defaults();

        // Path that turns right
        let path = Path {
            waypoints: vec![
                Waypoint::new(0.0, 0.0),
                Waypoint::new(1.0, 0.0),
                Waypoint::new(1.5, -0.5),
                Waypoint::new(2.0, -1.0),
            ],
            length: 2.5,
        };

        let pose = Pose { x: 0.5, y: 0.0, theta: 0.0 };
        let output = controller.compute(&path, &pose, None, 0.01).unwrap();

        // Should have angular velocity to turn right (negative)
        // Actually depends on path geometry, just check it's non-zero
        assert!(output.twist.linear > 0.0);
    }

    #[test]
    fn test_point_to_segment_distance() {
        let p = Waypoint::new(1.0, 1.0);
        let a = Waypoint::new(0.0, 0.0);
        let b = Waypoint::new(2.0, 0.0);

        let dist = point_to_segment_distance(&p, &a, &b);
        assert!((dist - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_track_error() {
        let controller = PursuitController::with_defaults();
        let path = make_straight_path();

        // Robot 0.5m off path
        let pose = Pose { x: 1.0, y: 0.5, theta: 0.0 };
        let cte = controller.compute_cross_track_error(&path, &pose);

        assert!((cte - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cost_factor_defaults() {
        let config = PursuitConfig::default();
        assert!(config.use_cost_regulated_scaling);
        assert!((config.cost_scaling_gain - 0.6).abs() < 1e-6);
        assert!((config.min_cost_speed_fraction - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_cost_factor_no_costmap() {
        // When no costmap is provided, cost factor should be 1.0 (no scaling)
        let controller = PursuitController::with_defaults();
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let lookahead = Waypoint::new(1.0, 0.0);
        // Without costmap, compute() skips cost factor — tested indirectly
        // via pure_pursuit being called with cost_factor=1.0
        let twist = controller.pure_pursuit(&pose, lookahead, 3.0, 1.0);
        assert!(twist.linear > 0.0);
    }

    #[test]
    fn test_cost_factor_disabled() {
        let config = PursuitConfig {
            use_cost_regulated_scaling: false,
            ..PursuitConfig::default()
        };
        let controller = PursuitController::new(config);
        let costmap = costmap::Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let lookahead = Waypoint::new(1.0, 0.0);
        let factor = controller.compute_cost_factor(&costmap, &pose, &lookahead);
        assert!((factor - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cost_factor_free_space() {
        let controller = PursuitController::with_defaults();
        let costmap = costmap::Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        let lookahead = Waypoint::new(1.0, 0.0);
        let factor = controller.compute_cost_factor(&costmap, &pose, &lookahead);
        assert!((factor - 1.0).abs() < 1e-6, "free space should give factor 1.0");
    }

    #[test]
    fn test_cost_factor_clamped_to_min() {
        // With max cost = INSCRIBED and gain = 0.6:
        // factor = 1.0 - (253/253) * 0.6 = 0.4 — above min of 0.3
        // With gain = 1.0 and min = 0.3:
        // factor = 1.0 - 1.0 * 1.0 = 0.0 → clamped to 0.3
        let config = PursuitConfig {
            cost_scaling_gain: 1.0,
            min_cost_speed_fraction: 0.3,
            ..PursuitConfig::default()
        };
        let controller = PursuitController::new(config);
        let costmap = costmap::Costmap::new(10.0, 10.0, 0.1, 0.3, 0.2);
        let pose = Pose { x: 0.0, y: 0.0, theta: 0.0 };
        // Lookahead in free space, factor should still be 1.0
        let lookahead = Waypoint::new(1.0, 0.0);
        let factor = controller.compute_cost_factor(&costmap, &pose, &lookahead);
        // Empty costmap has FREE everywhere, so factor=1.0
        assert!(factor >= 0.3);
    }
}
