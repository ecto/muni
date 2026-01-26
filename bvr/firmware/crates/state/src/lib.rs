//! State machine and mode management for bvr.

use can::leds::LedCommand;
use tracing::{info, warn};
use types::Mode;

/// Events that trigger state transitions.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Disable command received (returns to Idle from active modes)
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
    /// Enter sleep mode (sensors off but rover reachable)
    Sleep,
    /// Wake from sleep mode
    Wake,
}

/// State machine for rover operating modes.
pub struct StateMachine {
    mode: Mode,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            mode: Mode::Idle,
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
            // From Idle
            (Mode::Idle, Event::Disable) => Mode::Idle, // no-op, already idle
            (Mode::Idle, Event::TeleopCommand) => Mode::Teleop,
            (Mode::Idle, Event::AutonomousRequest) => Mode::Autonomous,
            (Mode::Idle, Event::Sleep) => Mode::Sleep,

            // From Teleop
            (Mode::Teleop, Event::Disable) => Mode::Idle,
            (Mode::Teleop, Event::CommandTimeout) => {
                warn!("Command timeout in teleop, returning to idle");
                Mode::Idle
            }
            (Mode::Teleop, Event::AutonomousRequest) => Mode::Autonomous,

            // From Autonomous
            (Mode::Autonomous, Event::Disable) => Mode::Idle,
            (Mode::Autonomous, Event::TeleopCommand) => Mode::Teleop,
            (Mode::Autonomous, Event::AutonomousEnd) => Mode::Idle,
            (Mode::Autonomous, Event::CommandTimeout) => {
                warn!("Command timeout in autonomous, returning to idle");
                Mode::Idle
            }

            // From Sleep
            (Mode::Sleep, Event::Wake) => Mode::Idle,
            (Mode::Sleep, Event::Disable) => Mode::Idle,

            // E-Stop from any active mode (including Sleep)
            (Mode::Idle | Mode::Teleop | Mode::Autonomous | Mode::Sleep, Event::EStop) => Mode::EStop,

            // E-Stop release
            (Mode::EStop, Event::EStopRelease) => Mode::Idle,

            // Faults
            (_, Event::Fault) => Mode::Fault,
            (Mode::Fault, Event::FaultClear) => Mode::Idle,

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
        matches!(self.mode, Mode::Disabled | Mode::Idle | Mode::EStop | Mode::Sleep)
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
            Mode::Disabled => LedCommand::state_idle(), // legacy variant, treat as idle
            Mode::Idle => LedCommand::state_idle(),
            Mode::Teleop => LedCommand::state_teleop(),
            Mode::Autonomous => LedCommand::state_autonomous(),
            Mode::EStop => LedCommand::state_estop(),
            Mode::Fault => LedCommand::state_fault(),
            Mode::Sleep => LedCommand::state_sleep(),
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
    fn test_boots_into_idle() {
        let sm = StateMachine::new();
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_basic_transitions() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.mode(), Mode::Idle);

        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_estop_requires_release() {
        let mut sm = StateMachine::new();
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // Must release first
        sm.transition(Event::EStopRelease);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_teleop_to_autonomous() {
        let mut sm = StateMachine::new();
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
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::CommandTimeout);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_command_timeout_in_autonomous() {
        let mut sm = StateMachine::new();
        sm.transition(Event::AutonomousRequest);
        assert_eq!(sm.mode(), Mode::Autonomous);

        sm.transition(Event::CommandTimeout);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_autonomous_end() {
        let mut sm = StateMachine::new();
        sm.transition(Event::AutonomousRequest);
        assert_eq!(sm.mode(), Mode::Autonomous);

        sm.transition(Event::AutonomousEnd);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_disable_returns_to_idle() {
        // From Idle (no-op)
        let mut sm = StateMachine::new();
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Idle);

        // From Teleop
        let mut sm = StateMachine::new();
        sm.transition(Event::TeleopCommand);
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Idle);

        // From Autonomous
        let mut sm = StateMachine::new();
        sm.transition(Event::AutonomousRequest);
        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_estop_from_all_active_modes() {
        // From Idle
        let mut sm = StateMachine::new();
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // From Teleop
        let mut sm = StateMachine::new();
        sm.transition(Event::TeleopCommand);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // From Autonomous
        let mut sm = StateMachine::new();
        sm.transition(Event::AutonomousRequest);
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_fault_from_any_mode() {
        let modes_to_test = [
            Mode::Idle,
            Mode::Teleop,
            Mode::Autonomous,
            Mode::EStop,
            Mode::Sleep,
        ];

        for start_mode in modes_to_test {
            let mut sm = StateMachine::new();
            match start_mode {
                Mode::Idle => {}
                Mode::Teleop => {
                    sm.transition(Event::TeleopCommand);
                }
                Mode::Autonomous => {
                    sm.transition(Event::AutonomousRequest);
                }
                Mode::EStop => {
                    sm.transition(Event::EStop);
                }
                Mode::Sleep => {
                    sm.transition(Event::Sleep);
                }
                _ => {}
            }

            sm.transition(Event::Fault);
            assert_eq!(sm.mode(), Mode::Fault, "Fault should be reachable from {:?}", start_mode);
        }
    }

    #[test]
    fn test_fault_clear_goes_to_idle() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Fault);
        assert_eq!(sm.mode(), Mode::Fault);

        sm.transition(Event::FaultClear);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_force_estop() {
        let mut sm = StateMachine::new();
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.force_estop();
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_force_estop_idempotent() {
        let mut sm = StateMachine::new();
        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        sm.force_estop();
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_is_driving() {
        let mut sm = StateMachine::new();
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
        assert!(sm.is_safe()); // Idle

        sm.transition(Event::TeleopCommand);
        assert!(!sm.is_safe()); // Teleop

        sm.transition(Event::EStop);
        assert!(sm.is_safe()); // EStop
    }

    #[test]
    fn test_no_transition_invalid_events() {
        // Can't release estop from idle
        let mut sm = StateMachine::new();
        sm.transition(Event::EStopRelease);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_default_impl() {
        let sm = StateMachine::default();
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_sleep_transitions() {
        // Idle -> Sleep
        let mut sm = StateMachine::new();
        assert_eq!(sm.mode(), Mode::Idle);
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);

        // Sleep -> Wake (returns to Idle)
        sm.transition(Event::Wake);
        assert_eq!(sm.mode(), Mode::Idle);

        // Idle -> Sleep again
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);
    }

    #[test]
    fn test_sleep_is_safe() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Sleep);
        assert!(sm.is_safe());
        assert!(!sm.is_driving());
    }

    #[test]
    fn test_sleep_to_estop() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);

        sm.transition(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);
    }

    #[test]
    fn test_sleep_disable_goes_to_idle() {
        let mut sm = StateMachine::new();
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);

        sm.transition(Event::Disable);
        assert_eq!(sm.mode(), Mode::Idle);
    }

    #[test]
    fn test_sleep_wake_cycle() {
        // Idle -> Sleep -> Wake -> Idle
        let mut sm = StateMachine::new();
        assert_eq!(sm.mode(), Mode::Idle);
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);
        sm.transition(Event::Wake);
        assert_eq!(sm.mode(), Mode::Idle);

        // After wake, full functionality: Idle -> Teleop -> Idle -> Sleep
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);
        sm.transition(Event::CommandTimeout);
        assert_eq!(sm.mode(), Mode::Idle);
        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Sleep);
    }

    #[test]
    fn test_cannot_sleep_from_teleop() {
        let mut sm = StateMachine::new();
        sm.transition(Event::TeleopCommand);
        assert_eq!(sm.mode(), Mode::Teleop);

        sm.transition(Event::Sleep);
        assert_eq!(sm.mode(), Mode::Teleop); // No transition
    }
}



