//! Shared types for bvrd inter-thread state.

use can::vesc::Drivetrain;
use dispatch::{TaskAssignment, Waypoint as DispatchWaypoint};
use state::StateMachine;
use tools::Registry as ToolRegistry;
use tracing::info;
use types::{SubsystemHealth, Twist};

/// State for a dispatched task being executed
#[derive(Debug, Clone)]
pub(crate) struct DispatchedTask {
    pub task_id: uuid::Uuid,
    pub waypoints: Vec<DispatchWaypoint>,
    pub current_waypoint: usize,
    pub lap: i32,
    pub is_loop: bool,
}

impl DispatchedTask {
    pub(crate) fn from_assignment(assignment: TaskAssignment) -> Self {
        Self {
            task_id: assignment.task_id,
            waypoints: assignment.zone.waypoints,
            current_waypoint: 0,
            lap: 0,
            is_loop: assignment.zone.is_loop,
        }
    }

    pub(crate) fn current_goal(&self) -> Option<[f64; 2]> {
        self.waypoints.get(self.current_waypoint).map(|wp| [wp.x, wp.y])
    }

    /// Advance to next waypoint. Returns true if task is complete.
    pub(crate) fn advance(&mut self) -> bool {
        self.current_waypoint += 1;
        if self.current_waypoint >= self.waypoints.len() {
            if self.is_loop {
                self.current_waypoint = 0;
                self.lap += 1;
                info!(lap = self.lap, "Starting new lap");
                false
            } else {
                true // Task complete
            }
        } else {
            false
        }
    }

    pub(crate) fn progress_percent(&self) -> i32 {
        if self.waypoints.is_empty() {
            return 100;
        }
        ((self.current_waypoint as f64 / self.waypoints.len() as f64) * 100.0) as i32
    }
}

/// Fault code indicating why the rover entered Fault mode.
/// Sent in telemetry so the console can display a reason.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FaultCode {
    #[default]
    None = 0,
    /// Watchdog timeout during autonomous mode (control loop or policy hung)
    WatchdogTimeout = 1,
}

/// Shared state between threads.
pub(crate) struct SharedState {
    pub state_machine: StateMachine,
    pub commanded_twist: Twist,
    pub drivetrain: Drivetrain,
    pub tool_registry: ToolRegistry,
    /// Subsystem health status (updated by various components)
    pub health: SubsystemHealth,
    /// Fault code — set when entering Fault, cleared on FaultClear
    pub fault_code: FaultCode,
}
