//! Dispatch Handler
//!
//! Processes dispatch events (task assignments, cancellations) from the
//! dispatch service WebSocket connection.

use dispatch::{DispatchClient, DispatchEvent, TaskAssignment};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use types::Mode;
use uuid::Uuid;

/// Result of processing dispatch events
#[derive(Debug, Default)]
pub struct DispatchResult {
    /// A new task was assigned (includes the assignment)
    pub task_assigned: Option<TaskAssignment>,
    /// Task ID that was cancelled
    pub task_cancelled: Option<Uuid>,
    /// Connection status changed
    pub connection_changed: Option<bool>,
    /// Request to transition to autonomous mode
    pub request_autonomous: bool,
    /// Forwarded commands from voice/API
    pub commands: Vec<types::Command>,
}

/// Handles dispatch service events
pub struct DispatchHandler {
    /// Dispatch client for reporting progress
    client: Option<Arc<DispatchClient>>,
}

impl DispatchHandler {
    /// Create a new dispatch handler
    pub fn new(client: Option<Arc<DispatchClient>>) -> Self {
        Self { client }
    }

    /// Process pending dispatch events from the receiver
    pub fn process_events(
        &self,
        dispatch_rx: &mut Option<mpsc::Receiver<DispatchEvent>>,
        current_mode: Mode,
    ) -> DispatchResult {
        let mut result = DispatchResult::default();

        let rx = match dispatch_rx {
            Some(rx) => rx,
            None => return result,
        };

        while let Ok(event) = rx.try_recv() {
            match event {
                DispatchEvent::TaskAssigned(assignment) => {
                    info!(
                        task_id = %assignment.task_id,
                        mission_id = %assignment.mission_id,
                        waypoints = assignment.zone.waypoints.len(),
                        is_loop = assignment.zone.is_loop,
                        "Task assigned from dispatch"
                    );

                    result.task_assigned = Some(assignment);

                    // Request autonomous mode if currently idle
                    match current_mode {
                        Mode::Idle => {
                            result.request_autonomous = true;
                            info!("Requesting transition to Autonomous mode for dispatch task");
                        }
                        Mode::Teleop => {
                            info!("Task queued - currently in Teleop mode");
                        }
                        Mode::Autonomous => {
                            info!("New task received while autonomous");
                        }
                        _ => {}
                    }
                }
                DispatchEvent::TaskCancelled(task_id) => {
                    info!(task_id = %task_id, "Task cancelled by dispatch");
                    result.task_cancelled = Some(task_id);
                }
                DispatchEvent::CommandReceived(cmd) => {
                    info!(?cmd, "Voice command received via dispatch");
                    result.commands.push(cmd);
                }
                DispatchEvent::Connected(connected) => {
                    if connected {
                        info!("Connected to dispatch service");
                    } else {
                        warn!("Disconnected from dispatch service");
                    }
                    result.connection_changed = Some(connected);
                }
            }
        }

        result
    }

    /// Report task progress to dispatch service
    pub fn report_progress(&self, task_id: Uuid, progress: i32, waypoint: i32, lap: i32) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            tokio::spawn(async move {
                let _ = client.report_progress(task_id, progress, waypoint, lap).await;
            });
        }
    }

    /// Report task completion to dispatch service
    pub fn report_complete(&self, task_id: Uuid, laps: i32) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            tokio::spawn(async move {
                let _ = client.report_complete(task_id, laps).await;
            });
        }
    }

    /// Report task failure to dispatch service
    pub fn report_failed(&self, task_id: Uuid, error: &str) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            let error = error.to_string();
            tokio::spawn(async move {
                let _ = client.report_failed(task_id, &error).await;
            });
        }
    }

    /// Check if we have a dispatch client
    pub fn has_client(&self) -> bool {
        self.client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_dispatch_rx() {
        let handler = DispatchHandler::new(None);
        let result = handler.process_events(&mut None, Mode::Idle);

        assert!(result.task_assigned.is_none());
        assert!(result.task_cancelled.is_none());
        assert!(!result.request_autonomous);
    }

    #[test]
    fn test_no_client() {
        let handler = DispatchHandler::new(None);
        assert!(!handler.has_client());
    }
}
