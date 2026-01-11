//! Dispatch Client
//!
//! Connects to the depot dispatch service to receive task assignments
//! and report progress. This crate provides the rover-side implementation
//! of the dispatch protocol.
//!
//! # Usage
//!
//! ```ignore
//! use dispatch::DispatchClient;
//!
//! let client = DispatchClient::new("ws://depot:4890/ws", "bvr0-01");
//! let mut events = client.subscribe();
//! client.start();
//!
//! // Receive tasks
//! while let Ok(event) = events.recv().await {
//!     match event {
//!         DispatchEvent::TaskAssigned(task) => {
//!             // Execute task...
//!             client.report_progress(task.task_id, 50, 5, 0).await?;
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Errors that can occur in the dispatch client
#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Send failed: channel closed")]
    SendFailed,

    #[error("Not connected")]
    NotConnected,
}

/// Waypoint in a zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theta: Option<f64>,
}

/// Zone data received from dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneData {
    pub waypoints: Vec<Waypoint>,
    #[serde(rename = "loop")]
    pub is_loop: bool,
}

/// Task assignment from dispatch
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub task_id: Uuid,
    pub mission_id: Uuid,
    pub zone: ZoneData,
}

/// Message from dispatch to rover
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum DispatchToRover {
    Task {
        task_id: Uuid,
        mission_id: Uuid,
        zone: ZoneData,
    },
    Cancel {
        task_id: Uuid,
    },
}

/// Message from rover to dispatch
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RoverToDispatch {
    Register { rover_id: String },
    Progress {
        task_id: Uuid,
        progress: i32,
        waypoint: i32,
        lap: i32,
    },
    Complete { task_id: Uuid, laps: i32 },
    Failed { task_id: Uuid, error: String },
}

/// Event emitted by the dispatch client
#[derive(Debug, Clone)]
pub enum DispatchEvent {
    /// New task assigned
    TaskAssigned(TaskAssignment),
    /// Current task cancelled
    TaskCancelled(Uuid),
    /// Connection state changed
    Connected(bool),
}

/// Internal command for the client
enum ClientCommand {
    ReportProgress {
        task_id: Uuid,
        progress: i32,
        waypoint: i32,
        lap: i32,
    },
    ReportComplete {
        task_id: Uuid,
        laps: i32,
    },
    ReportFailed {
        task_id: Uuid,
        error: String,
    },
}

/// Dispatch client for connecting to the depot
#[derive(Clone)]
pub struct DispatchClient {
    /// Dispatch service URL
    url: String,
    /// Rover ID
    rover_id: String,
    /// Channel to send commands to the connection task
    cmd_tx: mpsc::Sender<ClientCommand>,
    /// Broadcast channel for events
    event_tx: broadcast::Sender<DispatchEvent>,
    /// Current task (if any)
    current_task: Arc<Mutex<Option<Uuid>>>,
}

impl DispatchClient {
    /// Create a new dispatch client
    ///
    /// # Arguments
    /// * `url` - WebSocket URL of the dispatch service (e.g., "ws://depot:4890/ws")
    /// * `rover_id` - Unique identifier for this rover
    pub fn new(url: impl Into<String>, rover_id: impl Into<String>) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (event_tx, _) = broadcast::channel(32);

        let client = Self {
            url: url.into(),
            rover_id: rover_id.into(),
            cmd_tx,
            event_tx,
            current_task: Arc::new(Mutex::new(None)),
        };

        // Start the connection task
        client.spawn_connection_task(cmd_rx);

        client
    }

    /// Subscribe to dispatch events
    pub fn subscribe(&self) -> broadcast::Receiver<DispatchEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current task ID (if any)
    pub async fn current_task(&self) -> Option<Uuid> {
        *self.current_task.lock().await
    }

    /// Spawn the connection task
    fn spawn_connection_task(&self, mut cmd_rx: mpsc::Receiver<ClientCommand>) {
        let url = self.url.clone();
        let rover_id = self.rover_id.clone();
        let event_tx = self.event_tx.clone();
        let current_task = self.current_task.clone();

        tokio::spawn(async move {
            loop {
                info!(url = %url, rover_id = %rover_id, "Connecting to dispatch...");

                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        info!("Connected to dispatch");
                        let _ = event_tx.send(DispatchEvent::Connected(true));

                        let (mut write, mut read) = ws_stream.split();

                        // Register with dispatch
                        let register = RoverToDispatch::Register {
                            rover_id: rover_id.clone(),
                        };
                        if let Ok(json) = serde_json::to_string(&register) {
                            if write.send(Message::Text(json.into())).await.is_err() {
                                continue;
                            }
                        }

                        loop {
                            tokio::select! {
                                // Handle outgoing commands
                                Some(cmd) = cmd_rx.recv() => {
                                    let msg = match cmd {
                                        ClientCommand::ReportProgress { task_id, progress, waypoint, lap } => {
                                            RoverToDispatch::Progress { task_id, progress, waypoint, lap }
                                        }
                                        ClientCommand::ReportComplete { task_id, laps } => {
                                            *current_task.lock().await = None;
                                            RoverToDispatch::Complete { task_id, laps }
                                        }
                                        ClientCommand::ReportFailed { task_id, error } => {
                                            *current_task.lock().await = None;
                                            RoverToDispatch::Failed { task_id, error }
                                        }
                                    };

                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        if write.send(Message::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                }

                                // Handle incoming messages
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            if let Ok(dispatch_msg) = serde_json::from_str::<DispatchToRover>(&text) {
                                                match dispatch_msg {
                                                    DispatchToRover::Task { task_id, mission_id, zone } => {
                                                        info!(task_id = %task_id, "Task assigned");
                                                        *current_task.lock().await = Some(task_id);

                                                        let assignment = TaskAssignment {
                                                            task_id,
                                                            mission_id,
                                                            zone,
                                                        };
                                                        let _ = event_tx.send(DispatchEvent::TaskAssigned(assignment));
                                                    }
                                                    DispatchToRover::Cancel { task_id } => {
                                                        info!(task_id = %task_id, "Task cancelled");
                                                        *current_task.lock().await = None;
                                                        let _ = event_tx.send(DispatchEvent::TaskCancelled(task_id));
                                                    }
                                                }
                                            }
                                        }
                                        Some(Ok(Message::Ping(data))) => {
                                            let _ = write.send(Message::Pong(data)).await;
                                        }
                                        Some(Ok(Message::Close(_))) | None => {
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            warn!(error = %e, "WebSocket error");
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to connect to dispatch");
                    }
                }

                // Disconnected
                let _ = event_tx.send(DispatchEvent::Connected(false));

                // Wait before reconnecting
                info!("Reconnecting to dispatch in 5 seconds...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    /// Report task progress
    pub async fn report_progress(
        &self,
        task_id: Uuid,
        progress: i32,
        waypoint: i32,
        lap: i32,
    ) -> Result<(), DispatchError> {
        self.cmd_tx
            .send(ClientCommand::ReportProgress {
                task_id,
                progress,
                waypoint,
                lap,
            })
            .await
            .map_err(|_| DispatchError::SendFailed)
    }

    /// Report task completion
    pub async fn report_complete(&self, task_id: Uuid, laps: i32) -> Result<(), DispatchError> {
        self.cmd_tx
            .send(ClientCommand::ReportComplete { task_id, laps })
            .await
            .map_err(|_| DispatchError::SendFailed)
    }

    /// Report task failure
    pub async fn report_failed(
        &self,
        task_id: Uuid,
        error: impl Into<String>,
    ) -> Result<(), DispatchError> {
        self.cmd_tx
            .send(ClientCommand::ReportFailed {
                task_id,
                error: error.into(),
            })
            .await
            .map_err(|_| DispatchError::SendFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_deserialize() {
        let json = r#"{"waypoints": [{"x": 1.0, "y": 2.0}, {"x": 3.0, "y": 4.0, "theta": 1.57}], "loop": true}"#;
        let zone: ZoneData = serde_json::from_str(json).unwrap();

        assert_eq!(zone.waypoints.len(), 2);
        assert_eq!(zone.waypoints[0].x, 1.0);
        assert_eq!(zone.waypoints[0].y, 2.0);
        assert!(zone.waypoints[0].theta.is_none());
        assert_eq!(zone.waypoints[1].theta, Some(1.57));
        assert!(zone.is_loop);
    }

    #[test]
    fn test_register_serialize() {
        let msg = RoverToDispatch::Register {
            rover_id: "bvr0-01".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"register\""));
        assert!(json.contains("\"rover_id\":\"bvr0-01\""));
    }
}
