//! Navigation Controller
//!
//! Integrates path planning (A*) and path following (pure pursuit) with
//! costmap-based obstacle avoidance for autonomous navigation.
//!
//! This replaces the policy-based autonomous controller with a classical
//! navigation stack that supports:
//! - Global path planning with A*
//! - Local path following with pure pursuit
//! - Reactive obstacle detection and stopping
//! - Automatic replanning when path is blocked

use costmap::{Costmap, CostmapSnapshot};
use dispatch::TaskAssignment;
use lidar::PointCloud;
use planner::{PlannerConfig, ReactivePlanner, Waypoint};
use pursuit::{ObstacleState, PursuitConfig, PursuitController, PursuitError, PursuitOutput};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use transforms::Transform2D;
use types::{Pose, Twist};
use uuid::Uuid;

/// Navigation controller configuration.
#[derive(Debug, Clone)]
pub struct NavigationConfig {
    /// Costmap configuration
    pub costmap_width: f64,  // meters
    pub costmap_height: f64, // meters
    pub costmap_resolution: f64, // meters per cell
    pub inflation_radius: f64,   // meters
    pub inscribed_radius: f64,   // robot footprint radius

    /// Planner configuration
    pub planner: PlannerConfig,

    /// Pursuit configuration
    pub pursuit: PursuitConfig,

    /// Replan distance (how close to waypoint before advancing)
    pub replan_distance: f64,

    /// Blocked timeout (seconds) - how long to wait before giving up
    pub blocked_timeout: f64,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            costmap_width: 20.0,
            costmap_height: 20.0,
            costmap_resolution: 0.1,
            inflation_radius: 0.4,
            inscribed_radius: 0.35,
            planner: PlannerConfig::default(),
            pursuit: PursuitConfig::default(),
            replan_distance: 0.5,
            blocked_timeout: 30.0,
        }
    }
}

/// State of the navigation controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationState {
    /// No goal, idle.
    Idle,
    /// Planning path to goal.
    Planning,
    /// Following path.
    Following,
    /// Stopped due to obstacle, waiting for it to clear.
    ObstacleStopped,
    /// Replanning due to blocked path.
    Replanning,
    /// Goal reached.
    GoalReached,
    /// Navigation failed (no path found, stuck, etc.).
    Failed,
}

/// Output from navigation controller update.
#[derive(Debug, Clone)]
pub struct NavigationOutput {
    /// Computed twist command.
    pub twist: Twist,
    /// Current state.
    pub state: NavigationState,
    /// Distance to goal.
    pub distance_to_goal: f64,
    /// Current waypoint index.
    pub waypoint_index: usize,
    /// Total waypoints in path.
    pub total_waypoints: usize,
    /// Whether goal was reached.
    pub goal_reached: bool,
    /// Pursuit output (if following).
    pub pursuit_output: Option<PursuitOutput>,
    /// Error message if failed.
    pub error: Option<String>,
}

impl Default for NavigationOutput {
    fn default() -> Self {
        Self {
            twist: Twist::default(),
            state: NavigationState::Idle,
            distance_to_goal: f64::MAX,
            waypoint_index: 0,
            total_waypoints: 0,
            goal_reached: false,
            pursuit_output: None,
            error: None,
        }
    }
}

/// Active task being executed.
#[derive(Debug, Clone)]
pub struct ActiveTask {
    /// Task ID from dispatch.
    pub task_id: Uuid,
    /// Mission ID from dispatch.
    pub mission_id: Uuid,
    /// Waypoints to visit.
    pub waypoints: Vec<Waypoint>,
    /// Whether to loop back to start.
    pub is_loop: bool,
    /// Current waypoint index (which one we're navigating to).
    pub current_waypoint: usize,
    /// Current lap (for looping tasks).
    pub lap: i32,
}

impl ActiveTask {
    /// Create from dispatch task assignment.
    pub fn from_assignment(assignment: TaskAssignment) -> Self {
        let waypoints: Vec<Waypoint> = assignment
            .zone
            .waypoints
            .iter()
            .map(|w| Waypoint::new(w.x, w.y))
            .collect();

        Self {
            task_id: assignment.task_id,
            mission_id: assignment.mission_id,
            waypoints,
            is_loop: assignment.zone.is_loop,
            current_waypoint: 0,
            lap: 0,
        }
    }

    /// Get current goal waypoint.
    pub fn current_goal(&self) -> Option<Waypoint> {
        self.waypoints.get(self.current_waypoint).copied()
    }

    /// Advance to next waypoint. Returns true if task is complete.
    pub fn advance(&mut self) -> bool {
        self.current_waypoint += 1;

        if self.current_waypoint >= self.waypoints.len() {
            if self.is_loop {
                // Loop back to start
                self.current_waypoint = 0;
                self.lap += 1;
                info!(lap = self.lap, "Starting new lap");
                false
            } else {
                // Task complete
                true
            }
        } else {
            false
        }
    }

    /// Progress percentage (0-100).
    pub fn progress_percent(&self) -> i32 {
        if self.waypoints.is_empty() {
            return 100;
        }
        ((self.current_waypoint as f64 / self.waypoints.len() as f64) * 100.0) as i32
    }
}

/// Navigation controller that combines planning and following.
pub struct NavigationController {
    config: NavigationConfig,
    costmap: Costmap,
    planner: ReactivePlanner,
    pursuit: PursuitController,
    state: NavigationState,
    current_task: Option<ActiveTask>,
    blocked_start: Option<Instant>,
    last_pose: Pose,
}

impl NavigationController {
    /// Create a new navigation controller.
    pub fn new(config: NavigationConfig) -> Self {
        let costmap = Costmap::new(
            config.costmap_width,
            config.costmap_height,
            config.costmap_resolution,
            config.inflation_radius,
            config.inscribed_radius,
        );

        let planner = ReactivePlanner::new(config.planner.clone(), config.replan_distance);
        let pursuit = PursuitController::new(config.pursuit.clone());

        Self {
            config,
            costmap,
            planner,
            pursuit,
            state: NavigationState::Idle,
            current_task: None,
            blocked_start: None,
            last_pose: Pose::default(),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(NavigationConfig::default())
    }

    /// Get current state.
    pub fn state(&self) -> NavigationState {
        self.state
    }

    /// Get current task.
    pub fn current_task(&self) -> Option<&ActiveTask> {
        self.current_task.as_ref()
    }

    /// Get mutable reference to current task.
    pub fn current_task_mut(&mut self) -> Option<&mut ActiveTask> {
        self.current_task.as_mut()
    }

    /// Get reference to costmap (for visualization).
    pub fn costmap(&self) -> &Costmap {
        &self.costmap
    }

    /// Create an owned snapshot of the current costmap for streaming.
    pub fn costmap_snapshot(&self) -> CostmapSnapshot {
        self.costmap.snapshot()
    }

    /// Set a new dispatch task.
    pub fn set_task(&mut self, assignment: TaskAssignment) {
        let task = ActiveTask::from_assignment(assignment);
        info!(
            task_id = %task.task_id,
            waypoints = task.waypoints.len(),
            is_loop = task.is_loop,
            "New navigation task"
        );
        self.current_task = Some(task);
        self.state = NavigationState::Planning;
        self.planner.clear_goal();
        self.pursuit.reset();
        self.blocked_start = None;
    }

    /// Set a simple goal (for testing without dispatch).
    pub fn set_goal(&mut self, goal: Waypoint) {
        self.current_task = Some(ActiveTask {
            task_id: Uuid::new_v4(),
            mission_id: Uuid::new_v4(),
            waypoints: vec![goal],
            is_loop: false,
            current_waypoint: 0,
            lap: 0,
        });
        self.state = NavigationState::Planning;
        self.planner.clear_goal();
        self.pursuit.reset();
        self.blocked_start = None;
    }

    /// Cancel current task.
    pub fn cancel(&mut self) {
        self.current_task = None;
        self.state = NavigationState::Idle;
        self.planner.clear_goal();
        self.pursuit.reset();
        self.blocked_start = None;
    }

    /// Update costmap with new LiDAR scan.
    pub fn update_costmap(&mut self, scan: &PointCloud, robot_pose: &Transform2D) {
        self.costmap.update_obstacles(scan, robot_pose);
    }

    /// Clear costmap obstacles (use when starting fresh).
    pub fn clear_costmap(&mut self) {
        self.costmap.clear_obstacles();
    }

    /// Main update function. Call every control loop iteration.
    ///
    /// # Arguments
    /// * `pose` - Current robot pose
    /// * `dt` - Time step (seconds)
    ///
    /// # Returns
    /// Navigation output with twist command and status.
    pub fn update(&mut self, pose: &Pose, dt: f64) -> NavigationOutput {
        self.last_pose = *pose;
        let mut output = NavigationOutput::default();

        // Get current goal from task
        let goal = match &self.current_task {
            Some(task) => match task.current_goal() {
                Some(g) => g,
                None => {
                    output.state = NavigationState::GoalReached;
                    output.goal_reached = true;
                    return output;
                }
            },
            None => {
                output.state = NavigationState::Idle;
                return output;
            }
        };

        // Calculate distance to goal
        let dx = goal.x - pose.x;
        let dy = goal.y - pose.y;
        output.distance_to_goal = (dx * dx + dy * dy).sqrt();

        // State machine
        match self.state {
            NavigationState::Idle => {
                // Waiting for task
            }

            NavigationState::Planning | NavigationState::Replanning => {
                // Plan path to current goal
                match self.planner.set_goal(&self.costmap, pose, goal) {
                    Ok(path) => {
                        info!(
                            goal_x = format!("{:.2}", goal.x),
                            goal_y = format!("{:.2}", goal.y),
                            waypoints = path.waypoints.len(),
                            length = format!("{:.2}", path.length),
                            "Path planned"
                        );
                        self.pursuit.reset();
                        self.state = NavigationState::Following;
                        output.total_waypoints = path.waypoints.len();
                    }
                    Err(e) => {
                        error!(?e, "Path planning failed");
                        self.state = NavigationState::Failed;
                        output.error = Some(format!("Planning failed: {}", e));
                    }
                }
            }

            NavigationState::Following => {
                // Check if we need to replan
                if let Ok(Some(_)) = self.planner.update(&self.costmap, pose) {
                    // Path was replanned, pursuit will follow new path
                    debug!("Path updated");
                }

                // Get current path
                let path = match self.planner.path() {
                    Some(p) => p.clone(),
                    None => {
                        // No path, need to replan
                        self.state = NavigationState::Replanning;
                        return output;
                    }
                };

                output.total_waypoints = path.waypoints.len();

                // Follow path with pursuit controller
                match self.pursuit.compute(&path, pose, Some(&self.costmap), dt) {
                    Ok(pursuit_out) => {
                        output.twist = pursuit_out.twist;
                        output.waypoint_index = pursuit_out.waypoint_index;
                        output.pursuit_output = Some(pursuit_out.clone());

                        if pursuit_out.goal_reached {
                            // Reached current waypoint in task
                            output.goal_reached = true;
                            self.state = NavigationState::GoalReached;
                        } else if pursuit_out.obstacle_stop {
                            self.state = NavigationState::ObstacleStopped;
                            if self.blocked_start.is_none() {
                                self.blocked_start = Some(Instant::now());
                            }
                        }
                    }
                    Err(PursuitError::LostPath) => {
                        warn!("Lost path, replanning");
                        self.state = NavigationState::Replanning;
                    }
                    Err(e) => {
                        error!(?e, "Pursuit error");
                        self.state = NavigationState::Failed;
                        output.error = Some(format!("Pursuit failed: {}", e));
                    }
                }
            }

            NavigationState::ObstacleStopped => {
                // Check if obstacle cleared
                if self.pursuit.obstacle_state() == ObstacleState::Clear {
                    info!("Obstacle cleared, resuming");
                    self.state = NavigationState::Following;
                    self.blocked_start = None;
                } else {
                    // Check timeout
                    if let Some(start) = self.blocked_start {
                        let blocked_time = start.elapsed().as_secs_f64();
                        if blocked_time > self.config.blocked_timeout {
                            warn!(
                                blocked_time = format!("{:.1}s", blocked_time),
                                "Blocked timeout exceeded, replanning"
                            );
                            self.state = NavigationState::Replanning;
                            self.blocked_start = None;
                        }
                    }
                }

                // Still stopped
                output.twist = Twist::default();
            }

            NavigationState::GoalReached => {
                output.goal_reached = true;
                // Will be handled by task progression in main loop
            }

            NavigationState::Failed => {
                // Waiting for new command or retry
            }
        }

        output.state = self.state;
        output
    }

    /// Advance to next waypoint in task. Returns true if task is complete.
    pub fn advance_waypoint(&mut self) -> bool {
        if let Some(ref mut task) = self.current_task {
            let complete = task.advance();
            if complete {
                info!(
                    task_id = %task.task_id,
                    laps = task.lap,
                    "Task complete"
                );
                self.state = NavigationState::Idle;
                self.current_task = None;
            } else {
                // Plan to next waypoint
                self.state = NavigationState::Planning;
                self.planner.clear_goal();
            }
            complete
        } else {
            true
        }
    }

    /// Get task progress for reporting.
    pub fn task_progress(&self) -> Option<(Uuid, i32, i32, i32)> {
        self.current_task.as_ref().map(|task| {
            (
                task.task_id,
                task.progress_percent(),
                task.current_waypoint as i32,
                task.lap,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_controller_creation() {
        let controller = NavigationController::with_defaults();
        assert_eq!(controller.state(), NavigationState::Idle);
        assert!(controller.current_task().is_none());
    }

    #[test]
    fn test_set_goal() {
        let mut controller = NavigationController::with_defaults();
        controller.set_goal(Waypoint::new(5.0, 5.0));

        assert_eq!(controller.state(), NavigationState::Planning);
        assert!(controller.current_task().is_some());
    }

    #[test]
    fn test_cancel() {
        let mut controller = NavigationController::with_defaults();
        controller.set_goal(Waypoint::new(5.0, 5.0));
        controller.cancel();

        assert_eq!(controller.state(), NavigationState::Idle);
        assert!(controller.current_task().is_none());
    }

    #[test]
    fn test_active_task_progress() {
        let mut task = ActiveTask {
            task_id: Uuid::new_v4(),
            mission_id: Uuid::new_v4(),
            waypoints: vec![
                Waypoint::new(1.0, 0.0),
                Waypoint::new(2.0, 0.0),
                Waypoint::new(3.0, 0.0),
                Waypoint::new(4.0, 0.0),
            ],
            is_loop: false,
            current_waypoint: 0,
            lap: 0,
        };

        assert_eq!(task.progress_percent(), 0);
        assert!(!task.advance());
        assert_eq!(task.progress_percent(), 25);
        assert!(!task.advance());
        assert_eq!(task.progress_percent(), 50);
        assert!(!task.advance());
        assert_eq!(task.progress_percent(), 75);
        assert!(task.advance()); // Task complete
    }

    #[test]
    fn test_looping_task() {
        let mut task = ActiveTask {
            task_id: Uuid::new_v4(),
            mission_id: Uuid::new_v4(),
            waypoints: vec![Waypoint::new(1.0, 0.0), Waypoint::new(2.0, 0.0)],
            is_loop: true,
            current_waypoint: 0,
            lap: 0,
        };

        assert!(!task.advance());
        assert_eq!(task.current_waypoint, 1);
        assert!(!task.advance()); // Loops back
        assert_eq!(task.current_waypoint, 0);
        assert_eq!(task.lap, 1);
    }
}
