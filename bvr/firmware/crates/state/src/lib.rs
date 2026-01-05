//! State machine and mode management for bvr.

use can::leds::LedCommand;
use tracing::{info, warn};
use types::Mode;

/// Events that trigger state transitions.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Enable command received
    Enable,
    /// Disable command received
    Disable,
    /// Teleop command received
    TeleopCommand,
    /// Autonomous mode requested
    AutonomousRequest,
    /// Autonomous mode ended
    AutonomousEnd,
    /// E-stop triggered
    EStop,
    /// E-stop released
    EStopRelease,
    /// Fault detected
    Fault,
    /// Fault cleared
    FaultClear,
    /// Command timeout (watchdog)
    CommandTimeout,
}

/// State machine for rover operating modes.
pub struct StateMachine {
    mode: Mode,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            mode: Mode::Disabled,
        }
    }

    /// Get current mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Process an event and return the new mode.
    pub fn transition(&mut self, event: Event) -> Mode {
        let old_mode = self.mode;

        self.mode = match (self.mode, event) {
            // From Disabled
            (Mode::Disabled, Event::Enable) => Mode::Idle,

            // From Idle
            (Mode::Idle, Event::Disable) => Mode::Disabled,
            (Mode::Idle, Event::TeleopCommand) => Mode::Teleop,
            (Mode::Idle, Event::AutonomousRequest) => Mode::Autonomous,

            // From Teleop
            (Mode::Teleop, Event::Disable) => Mode::Disabled,
            (Mode::Teleop, Event::CommandTimeout) => {
                warn!("Command timeout in teleop, returning to idle");
                Mode::Idle
            }
            (Mode::Teleop, Event::AutonomousRequest) => Mode::Autonomous,

            // From Autonomous
            (Mode::Autonomous, Event::Disable) => Mode::Disabled,
            (Mode::Autonomous, Event::TeleopCommand) => Mode::Teleop,
            (Mode::Autonomous, Event::AutonomousEnd) => Mode::Idle,
            (Mode::Autonomous, Event::CommandTimeout) => {
                warn!("Command timeout in autonomous, returning to idle");
                Mode::Idle
            }

            // E-Stop from any active mode
            (Mode::Idle | Mode::Teleop | Mode::Autonomous, Event::EStop) => Mode::EStop,

            // E-Stop release
            (Mode::EStop, Event::EStopRelease) => Mode::Idle,

            // Faults
            (_, Event::Fault) => Mode::Fault,
            (Mode::Fault, Event::FaultClear) => Mode::Disabled,

            // No transition
            (mode, _) => mode,
        };

        if self.mode != old_mode {
            info!(?old_mode, new_mode = ?self.mode, ?event, "Mode transition");
        }

        self.mode
    }

    /// Check if the rover should be driving (motors enabled).
    pub fn is_driving(&self) -> bool {
        matches!(self.mode, Mode::Teleop | Mode::Autonomous)
    }

    /// Check if the rover is in a safe state (motors disabled).
    pub fn is_safe(&self) -> bool {
        matches!(self.mode, Mode::Disabled | Mode::Idle | Mode::EStop)
    }

    /// Force into e-stop (for critical safety).
    pub fn force_estop(&mut self) {
        if self.mode != Mode::EStop {
            warn!(old_mode = ?self.mode, "Forcing e-stop");
            self.mode = Mode::EStop;
        }
    }

    /// Get the LED command for the current mode.
    pub fn led_command(&self) -> LedCommand {
        match self.mode {
            Mode::Disabled => LedCommand::state_disabled(),
            Mode::Idle => LedCommand::state_idle(),
            Mode::Teleop => LedCommand::state_teleop(),
            Mode::Autonomous => LedCommand::state_autonomous(),
            Mode::EStop => LedCommand::state_estop(),
            Mode::Fault => LedCommand::state_fault(),
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_transitions() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.mode(), Mode::Disabled);

        sm.transition(Event::Enable);
        assert_eq!(sm.mode(), Mode::Idle);

        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_estop_requires_release() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::EStop);

        // Can't enable from e-stop
        sm.transition(Event::Enable);
        assert_eq!(sm.mode(), Mode::EStop);

        // Must release first
        sm.transition(Event::EStopRelease);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_teleop_to_autonomous() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::AutonomousRequest);
        assert_eq!(sm.mode(), Mode::Autonomous);

        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);
    }

    #[test]
    fn test_command_timeout_in_teleop() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::CommandTimeout);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_command_timeout_in_autonomous() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::AutonomousRequest);
        assert_eq!(sm.mode(), Mode::Autonomous);

        sm.transition(Event::CommandTimeout);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_autonomous_end() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::AutonomousRequest);
        assert_eq!(sm.mode(), Mode::Autonomous);

        sm.transition(Event::AutonomousEnd);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_disable_from_any_active_mode() {
        // From Idle
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Disabled);

        // From Teleop
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::TeleopCommand);
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Disabled);

        // From Autonomous
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::AutonomousRequest);
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Disabled);
    }

    #[test]
    fn test_estop_from_all_active_modes() {
        // From Idle
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // From Teleop
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::TeleopCommand);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // From Autonomous
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::AutonomousRequest);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_fault_from_any_mode() {
        let modes_to_test = [
            Mode::Disabled,
            Mode::Idle,
            Mode::Teleop,
            Mode::Autonomous,
            Mode::EStop,
        ];

        for start_mode in modes_to_test {
            let mut sm = StateMachine::new();
            // Manually reach the target mode
            match start_mode {
                Mode::Disabled => {}
                Mode::Idle => {
                    sm.transition(Event::Enable);
                }
                Mode::Teleop => {
                    sm.transition(Event::Enable);
                    sm.transition(Event::TeleopCommand);
                }
                Mode::Autonomous => {
                    sm.transition(Event::Enable);
                    sm.transition(Event::AutonomousRequest);
                }
                Mode::EStop => {
                    sm.transition(Event::Enable);
                    sm.transition(Event::EStop);
                }
                Mode::Fault => {}
            }

            sm.transition(Event::Fault);
            assert_eq!(sm.mode(), Mode::Fault, "Fault should be reachable from {:?}", start_mode);
        }
    }

    #[test]
    fn test_fault_clear_goes_to_disabled() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::Fault);
        assert_eq!(sm.mode(), Mode::Fault);

        sm.transition(Event::FaultClear);
        assert_eq!(sm.mode(), Mode::Disabled);
    }

    #[test]
    fn test_force_estop() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.force_estop();
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_force_estop_idempotent() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // Calling force_estop when already in EStop should be fine
        sm.force_estop();
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_is_driving() {
        let mut sm = StateMachine::new();
        assert!(!sm.is_driving());

        sm.transition(Event::Enable);
        assert!(!sm.is_driving()); // Idle

        sm.transition(Event::TeleopCommand);
        assert!(sm.is_driving()); // Teleop

        sm.transition(Event::AutonomousRequest);
        assert!(sm.is_driving()); // Autonomous

        sm.transition(Event::EStop);
        assert!(!sm.is_driving()); // EStop
    }

    #[test]
    fn test_is_safe() {
        let mut sm = StateMachine::new();
        assert!(sm.is_safe()); // Disabled

        sm.transition(Event::Enable);
        assert!(sm.is_safe()); // Idle

        sm.transition(Event::TeleopCommand);
        assert!(!sm.is_safe()); // Teleop

        sm.transition(Event::EStop);
        assert!(sm.is_safe()); // EStop
    }

    #[test]
    fn test_no_transition_invalid_events() {
        // Can't teleop from disabled
        let mut sm = StateMachine::new();
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Disabled);

        // Can't release estop from idle
        let mut sm = StateMachine::new();
        sm.transition(Event::Enable);
        sm.transition(Event::EStopRelease);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_default_impl() {
        let sm = StateMachine::default();
        assert_eq!(sm.mode(), Mode::Disabled);
    }
}



