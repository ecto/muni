//! Command Processor
//!
//! Handles draining and routing of commands from the teleop channel.
//! Implements "latest Twist wins" semantics for velocity commands.

use tokio::sync::mpsc;
use tracing::debug;
use types::{Command, Mode, ToolCommand, Twist};

/// Result of processing commands from the channel
#[derive(Debug, Default)]
pub struct ProcessedCommands {
    /// Latest Twist command (if any) - only the most recent matters
    pub latest_twist: Option<Twist>,
    /// E-stop was triggered
    pub estop_triggered: bool,
    /// E-stop release requested
    pub estop_release_requested: bool,
    /// Mode change requested
    pub mode_request: Option<Mode>,
    /// Heartbeat received
    pub heartbeat_received: bool,
    /// Tool command received
    pub tool_command: Option<ToolCommand>,
}

impl ProcessedCommands {
    /// Check if any non-twist command was received
    pub fn has_commands(&self) -> bool {
        self.estop_triggered
            || self.estop_release_requested
            || self.mode_request.is_some()
            || self.heartbeat_received
            || self.tool_command.is_some()
    }
}

/// Processes commands from the teleop channel
pub struct CommandProcessor {
    /// Number of twist commands dropped this tick (for diagnostics)
    dropped_twists: u32,
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self { dropped_twists: 0 }
    }

    /// Drain all pending commands from the channel.
    ///
    /// For Twist commands, only the latest is kept (newer commands overwrite older).
    /// For other commands, they are accumulated and returned in order.
    pub fn drain(&mut self, cmd_rx: &mut mpsc::Receiver<Command>) -> ProcessedCommands {
        let mut result = ProcessedCommands::default();
        self.dropped_twists = 0;

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Twist(twist) => {
                    if result.latest_twist.is_some() {
                        self.dropped_twists += 1;
                    }
                    result.latest_twist = Some(twist);
                }
                Command::EStop => {
                    result.estop_triggered = true;
                }
                Command::EStopRelease => {
                    result.estop_release_requested = true;
                }
                Command::SetMode(mode) => {
                    result.mode_request = Some(mode);
                }
                Command::Heartbeat => {
                    result.heartbeat_received = true;
                }
                Command::Tool(tc) => {
                    result.tool_command = Some(tc);
                }
                Command::LidarToggle(_) => {
                    // LidarToggle is handled per-connection in WebRTC, not forwarded
                }
                Command::SetGoal { .. } => {
                    // SetGoal is handled by dispatch / navigation controller
                }
            }
        }

        if self.dropped_twists > 0 {
            debug!(dropped = self.dropped_twists, "Dropped stale Twist commands");
        }

        result
    }

    /// Get count of dropped twist commands from last drain
    pub fn dropped_twists(&self) -> u32 {
        self.dropped_twists
    }
}

impl Default for CommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_drain_twist_latest_wins() {
        let (tx, mut rx) = mpsc::channel(16);

        // Send multiple twists
        tx.send(Command::Twist(Twist {
            linear: 1.0,
            angular: 0.0,
            boost: false,
        }))
        .await
        .unwrap();
        tx.send(Command::Twist(Twist {
            linear: 2.0,
            angular: 0.5,
            boost: false,
        }))
        .await
        .unwrap();
        tx.send(Command::Twist(Twist {
            linear: 3.0,
            angular: 1.0,
            boost: true,
        }))
        .await
        .unwrap();

        let mut processor = CommandProcessor::new();
        let result = processor.drain(&mut rx);

        // Only the last twist should be kept
        assert!(result.latest_twist.is_some());
        let twist = result.latest_twist.unwrap();
        assert!((twist.linear - 3.0).abs() < 0.001);
        assert!((twist.angular - 1.0).abs() < 0.001);
        assert!(twist.boost);

        // Two twists were dropped
        assert_eq!(processor.dropped_twists(), 2);
    }

    #[tokio::test]
    async fn test_drain_mixed_commands() {
        let (tx, mut rx) = mpsc::channel(16);

        tx.send(Command::Heartbeat).await.unwrap();
        tx.send(Command::Twist(Twist {
            linear: 1.0,
            angular: 0.0,
            boost: false,
        }))
        .await
        .unwrap();
        tx.send(Command::SetMode(Mode::Teleop)).await.unwrap();
        tx.send(Command::Tool(ToolCommand {
            axis: 0.5,
            motor: 0.0,
            action_a: true,
            action_b: false,
        }))
        .await
        .unwrap();

        let mut processor = CommandProcessor::new();
        let result = processor.drain(&mut rx);

        assert!(result.heartbeat_received);
        assert!(result.latest_twist.is_some());
        assert_eq!(result.mode_request, Some(Mode::Teleop));
        assert!(result.tool_command.is_some());
    }

    #[tokio::test]
    async fn test_drain_estop_commands() {
        let (tx, mut rx) = mpsc::channel(16);

        tx.send(Command::EStop).await.unwrap();
        tx.send(Command::EStopRelease).await.unwrap();

        let mut processor = CommandProcessor::new();
        let result = processor.drain(&mut rx);

        assert!(result.estop_triggered);
        assert!(result.estop_release_requested);
    }

    #[tokio::test]
    async fn test_drain_empty() {
        let (_tx, mut rx) = mpsc::channel::<Command>(16);

        let mut processor = CommandProcessor::new();
        let result = processor.drain(&mut rx);

        assert!(result.latest_twist.is_none());
        assert!(!result.has_commands());
    }
}
