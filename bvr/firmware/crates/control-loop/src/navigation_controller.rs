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

use costmap::{Costmap, CostmapSnapshot, Obstacle, ScanAccumulator};
use dispatch::TaskAssignment;
#[cfg(feature = "lidar")]
use lidar::PointCloud;
use mppi::{MppiConfig, MppiController};
use nalgebra::{Vector2, Vector3};
use planner::{PlannerConfig, ReactivePlanner, SmootherConfig, Waypoint};
use pursuit::{PursuitConfig, PursuitController, PursuitError, PursuitOutput};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use transforms::Transform2D;
use types::{Pose, Twist};
use uuid::Uuid;

/// Which local controller to use for path following.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerType {
    /// Classical pure pursuit controller (default).
    PurePursuit,
    /// MPPI sampling-based trajectory optimizer.
    Mppi,
}

impl Default for ControllerType {
    fn default() -> Self {
        Self::PurePursuit
    }
}

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

    /// MPPI configuration (used when controller_type == Mppi)
    pub mppi: MppiConfig,

    /// Which local controller to use
    pub controller_type: ControllerType,

    /// Replan distance (how close to waypoint before advancing)
    pub replan_distance: f64,

    /// Blocked timeout (seconds) - how long to wait before starting recovery
    pub blocked_timeout: f64,

    /// Path smoother configuration (None to disable smoothing)
    pub smoother: Option<SmootherConfig>,

    /// Recovery behavior configuration
    pub recovery: RecoveryConfig,
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
            mppi: MppiConfig::default(),
            controller_type: ControllerType::default(),
            replan_distance: 0.5,
            blocked_timeout: 30.0,
            smoother: Some(SmootherConfig::default()),
            recovery: RecoveryConfig::default(),
        }
    }
}

/// Recovery behavior configuration.
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Distance to back up (meters).
    pub backup_distance: f64,
    /// Speed to back up (m/s, positive value).
    pub backup_speed: f64,
    /// Angular velocity for 360° spin (rad/s).
    pub spin_speed: f64,
    /// Duration to wait before retrying (seconds).
    pub wait_duration: f64,
    /// Maximum recovery cycles before declaring failure.
    pub max_recovery_cycles: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            backup_distance: 0.3,
            backup_speed: 0.2,
            spin_speed: 0.5,
            wait_duration: 10.0,
            max_recovery_cycles: 3,
        }
    }
}

/// Recovery behavior types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryBehavior {
    /// Back up a fixed distance.
    BackUp,
    /// Clear costmap dynamic layer.
    ClearCostmap,
    /// Spin 360° to refresh LiDAR data.
    Spin,
    /// Wait before retrying.
    Wait,
}

/// Progress tracking for the active recovery behavior.
#[derive(Debug, Clone)]
struct RecoveryProgress {
    /// Distance backed up so far (meters).
    distance_traveled: f64,
    /// Angle rotated so far (radians).
    angle_rotated: f64,
    /// Time spent waiting (seconds).
    wait_elapsed: f64,
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
    /// Executing recovery behavior (back up, spin, wait, etc.).
    Recovering(RecoveryBehavior),
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
    /// MPPI best trajectory as (x, y) world positions. Empty when MPPI is not active.
    pub mppi_trajectory: Vec<[f64; 2]>,
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
            mppi_trajectory: vec![],
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
    mppi: Option<MppiController>,
    state: NavigationState,
    current_task: Option<ActiveTask>,
    blocked_start: Option<Instant>,
    last_pose: Pose,
    /// Multi-frame scan accumulator for reinforcing obstacle evidence.
    accumulator: ScanAccumulator,
    /// Recovery cycle count (resets on successful plan).
    recovery_cycle_count: u32,
    /// Progress of current recovery behavior.
    recovery_progress: Option<RecoveryProgress>,
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

        let planner = if let Some(ref smoother_config) = config.smoother {
            ReactivePlanner::with_smoother(
                config.planner.clone(),
                config.replan_distance,
                smoother_config.clone(),
            )
        } else {
            ReactivePlanner::new(config.planner.clone(), config.replan_distance)
        };
        let pursuit = PursuitController::new(config.pursuit.clone());
        let mppi = if config.controller_type == ControllerType::Mppi {
            Some(MppiController::new(config.mppi.clone()))
        } else {
            None
        };

        Self {
            config,
            costmap,
            planner,
            pursuit,
            mppi,
            state: NavigationState::Idle,
            current_task: None,
            blocked_start: None,
            last_pose: Pose::default(),
            accumulator: ScanAccumulator::new(2),
            recovery_cycle_count: 0,
            recovery_progress: None,
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

    /// Extract discrete obstacles from the current costmap.
    pub fn extract_obstacles(&self) -> Vec<Obstacle> {
        self.costmap.extract_obstacles()
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
        if let Some(ref mut mppi) = self.mppi {
            mppi.reset();
        }
        self.blocked_start = None;
        self.recovery_cycle_count = 0;
        self.recovery_progress = None;
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
        if let Some(ref mut mppi) = self.mppi {
            mppi.reset();
        }
        self.blocked_start = None;
        self.recovery_cycle_count = 0;
        self.recovery_progress = None;
    }

    /// Cancel current task.
    pub fn cancel(&mut self) {
        self.current_task = None;
        self.state = NavigationState::Idle;
        self.planner.clear_goal();
        self.pursuit.reset();
        if let Some(ref mut mppi) = self.mppi {
            mppi.reset();
        }
        self.blocked_start = None;
        self.recovery_cycle_count = 0;
        self.recovery_progress = None;
    }

    /// Update costmap from pre-classified ground and obstacle points (world frame).
    ///
    /// Generic entry point for any perception source (depth cameras, lidar, etc.).
    pub fn update_costmap_from_points(
        &mut self,
        sensor_pos: Vector2<f64>,
        ground_pts: &[Vector3<f64>],
        obstacle_pts: &[Vector3<f64>],
    ) {
        self.costmap.update_from_points_accumulated(
            sensor_pos,
            ground_pts,
            obstacle_pts,
            &mut self.accumulator,
        );
    }

    /// Update costmap with new LiDAR scan.
    ///
    /// Uses multi-frame scan accumulation to reinforce obstacle evidence
    /// from recent scans, producing denser and more stable obstacles.
    #[cfg(feature = "lidar")]
    pub fn update_costmap(&mut self, scan: &PointCloud, robot_pose: &Transform2D) {
        self.costmap
            .update_obstacles_accumulated(scan, robot_pose, &mut self.accumulator);
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
                        if let Some(ref mut mppi) = self.mppi {
                            mppi.reset();
                        }
                        self.state = NavigationState::Following;
                        output.total_waypoints = path.waypoints.len();
                        // Reset recovery cycle count on successful plan
                        self.recovery_cycle_count = 0;
                        self.recovery_progress = None;
                    }
                    Err(e) => {
                        error!(?e, "Path planning failed");
                        // If we were replanning during recovery, try spin
                        if self.recovery_progress.is_some() {
                            warn!("Replan failed during recovery, trying spin");
                            self.start_recovery(RecoveryBehavior::Spin);
                        } else {
                            self.state = NavigationState::Failed;
                            output.error = Some(format!("Planning failed: {}", e));
                        }
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

                // Follow path with selected controller
                if let Some(ref mut mppi) = self.mppi {
                    // MPPI controller
                    let vel = Twist {
                        linear: 0.0,
                        angular: 0.0,
                        boost: false,
                    };
                    let mppi_out = mppi.compute(&path, pose, &vel, &self.costmap);
                    output.twist = mppi_out.twist;
                    output.mppi_trajectory = mppi_out.best_trajectory;

                    // Check goal reached (distance-based)
                    if output.distance_to_goal < self.config.replan_distance {
                        output.goal_reached = true;
                        self.state = NavigationState::GoalReached;
                    }
                    // Note: obstacle stopping is handled externally by
                    // CollisionMonitor via notify_collision_status()
                } else {
                    // Pure pursuit controller
                    match self.pursuit.compute(&path, pose, Some(&self.costmap), dt) {
                        Ok(pursuit_out) => {
                            output.twist = pursuit_out.twist;
                            output.waypoint_index = pursuit_out.waypoint_index;
                            output.pursuit_output = Some(pursuit_out.clone());

                            if pursuit_out.goal_reached {
                                // Reached current waypoint in task
                                output.goal_reached = true;
                                self.state = NavigationState::GoalReached;
                            }
                            // Note: obstacle stopping is now handled externally by
                            // CollisionMonitor via notify_collision_status()
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
            }

            NavigationState::ObstacleStopped => {
                // Collision status is updated externally via notify_collision_status().
                // If it transitions us back to Following, we'll resume next tick.
                // Check blocked timeout to start recovery.
                if let Some(start) = self.blocked_start {
                    let blocked_time = start.elapsed().as_secs_f64();
                    if blocked_time > self.config.blocked_timeout {
                        warn!(
                            blocked_time = format!("{:.1}s", blocked_time),
                            cycle = self.recovery_cycle_count,
                            "Blocked timeout exceeded, starting recovery"
                        );
                        self.blocked_start = None;
                        self.start_recovery(RecoveryBehavior::BackUp);
                    }
                }

                // Still stopped
                output.twist = Twist::default();
            }

            NavigationState::Recovering(behavior) => {
                output.twist = self.update_recovery(behavior, pose, dt);
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

    /// Notify the controller that the collision monitor has stopped/released the robot.
    ///
    /// Called from the main loop after the CollisionMonitor filters the twist.
    /// When `is_stopped` transitions from false→true, we enter ObstacleStopped.
    /// When it transitions from true→false, we resume Following.
    pub fn notify_collision_status(&mut self, is_stopped: bool) {
        match self.state {
            NavigationState::Following if is_stopped => {
                info!("Collision monitor stopped robot, entering ObstacleStopped");
                self.state = NavigationState::ObstacleStopped;
                if self.blocked_start.is_none() {
                    self.blocked_start = Some(Instant::now());
                }
            }
            NavigationState::ObstacleStopped if !is_stopped => {
                info!("Collision monitor cleared, resuming Following");
                self.state = NavigationState::Following;
                self.blocked_start = None;
            }
            _ => {}
        }
    }

    /// Start a recovery behavior.
    fn start_recovery(&mut self, behavior: RecoveryBehavior) {
        info!(?behavior, cycle = self.recovery_cycle_count, "Starting recovery behavior");
        self.recovery_progress = Some(RecoveryProgress {
            distance_traveled: 0.0,
            angle_rotated: 0.0,
            wait_elapsed: 0.0,
        });
        self.state = NavigationState::Recovering(behavior);
    }

    /// Update the active recovery behavior. Returns twist command.
    fn update_recovery(&mut self, behavior: RecoveryBehavior, _pose: &Pose, dt: f64) -> Twist {
        let progress = match self.recovery_progress.as_mut() {
            Some(p) => p,
            None => {
                // Shouldn't happen, transition to Replanning
                self.state = NavigationState::Replanning;
                return Twist::default();
            }
        };

        match behavior {
            RecoveryBehavior::BackUp => {
                let target_dist = self.config.recovery.backup_distance;
                progress.distance_traveled += self.config.recovery.backup_speed * dt;

                if progress.distance_traveled >= target_dist {
                    info!(distance = format!("{:.2}m", progress.distance_traveled), "Backup complete");
                    // Next step: clear costmap
                    self.start_recovery(RecoveryBehavior::ClearCostmap);
                    return Twist::default();
                }

                Twist {
                    linear: -self.config.recovery.backup_speed,
                    angular: 0.0,
                    boost: false,
                }
            }

            RecoveryBehavior::ClearCostmap => {
                info!("Clearing costmap dynamic layer");
                self.costmap.clear_obstacles();
                self.accumulator = ScanAccumulator::new(2);
                // Transition to replanning
                self.state = NavigationState::Replanning;
                // Keep recovery_progress alive so failed replan can trigger Spin
                Twist::default()
            }

            RecoveryBehavior::Spin => {
                let target_angle = std::f64::consts::TAU; // 360°
                progress.angle_rotated += self.config.recovery.spin_speed * dt;

                if progress.angle_rotated >= target_angle {
                    info!("Spin complete, attempting replan");
                    // After spin, try wait then retry from step 1
                    self.start_recovery(RecoveryBehavior::Wait);
                    return Twist::default();
                }

                Twist {
                    linear: 0.0,
                    angular: self.config.recovery.spin_speed,
                    boost: false,
                }
            }

            RecoveryBehavior::Wait => {
                progress.wait_elapsed += dt;

                if progress.wait_elapsed >= self.config.recovery.wait_duration {
                    self.recovery_cycle_count += 1;
                    info!(cycle = self.recovery_cycle_count, "Wait complete");

                    if self.recovery_cycle_count >= self.config.recovery.max_recovery_cycles {
                        error!(
                            cycles = self.recovery_cycle_count,
                            "Max recovery cycles exceeded, navigation failed"
                        );
                        self.state = NavigationState::Failed;
                        self.recovery_progress = None;
                    } else {
                        // Start another recovery cycle from BackUp
                        self.start_recovery(RecoveryBehavior::BackUp);
                    }
                    return Twist::default();
                }

                Twist::default()
            }
        }
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

    /// Get the current planned path waypoints (from A* planner).
    ///
    /// Returns the path waypoints if a path is currently planned, along with
    /// the waypoint index being pursued. Used for streaming to the console.
    pub fn planned_path(&self) -> Option<&planner::Path> {
        self.planner.path()
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
    fn test_recovery_config_defaults() {
        let config = RecoveryConfig::default();
        assert!((config.backup_distance - 0.3).abs() < 1e-6);
        assert!((config.backup_speed - 0.2).abs() < 1e-6);
        assert!((config.spin_speed - 0.5).abs() < 1e-6);
        assert!((config.wait_duration - 10.0).abs() < 1e-6);
        assert_eq!(config.max_recovery_cycles, 3);
    }

    #[test]
    fn test_recovery_behavior_enum() {
        // Verify all variants exist and are distinct
        let behaviors = [
            RecoveryBehavior::BackUp,
            RecoveryBehavior::ClearCostmap,
            RecoveryBehavior::Spin,
            RecoveryBehavior::Wait,
        ];
        for (i, a) in behaviors.iter().enumerate() {
            for (j, b) in behaviors.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn test_navigation_state_recovering() {
        let state = NavigationState::Recovering(RecoveryBehavior::BackUp);
        assert!(matches!(state, NavigationState::Recovering(RecoveryBehavior::BackUp)));
        assert_ne!(state, NavigationState::Following);
    }

    #[test]
    fn test_notify_collision_status() {
        let mut controller = NavigationController::with_defaults();
        controller.set_goal(Waypoint::new(5.0, 5.0));

        // Simulate planning -> following
        controller.state = NavigationState::Following;

        // Notify collision stopped
        controller.notify_collision_status(true);
        assert_eq!(controller.state(), NavigationState::ObstacleStopped);

        // Notify collision cleared
        controller.notify_collision_status(false);
        assert_eq!(controller.state(), NavigationState::Following);
    }

    #[test]
    fn test_recovery_resets_on_new_task() {
        let mut controller = NavigationController::with_defaults();
        controller.recovery_cycle_count = 2;
        controller.set_goal(Waypoint::new(5.0, 5.0));
        assert_eq!(controller.recovery_cycle_count, 0);
        assert!(controller.recovery_progress.is_none());
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
