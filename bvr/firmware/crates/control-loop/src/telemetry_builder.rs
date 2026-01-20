//! Telemetry Builder
//!
//! Collects telemetry data from all subsystems and builds the Telemetry
//! struct for transmission to operators.

use std::time::{SystemTime, UNIX_EPOCH};
use teleop::{Telemetry, ToolStatus};
use types::{Mode, Pose, PowerStatus, SlamStatus, SubsystemHealth, Twist};

/// Builds telemetry packets from subsystem data
pub struct TelemetryBuilder {
    /// Last built telemetry (for caching/comparison)
    last_telemetry: Option<Telemetry>,
}

impl TelemetryBuilder {
    pub fn new() -> Self {
        Self {
            last_telemetry: None,
        }
    }

    /// Build telemetry from current subsystem states
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        mode: Mode,
        pose: Pose,
        battery_voltage: f64,
        motor_temps: [f32; 4],
        motor_currents: [f32; 4],
        velocity: Twist,
        active_tool: Option<String>,
        tool_status: Option<ToolStatus>,
        slam_status: Option<SlamStatus>,
        health: SubsystemHealth,
    ) -> Telemetry {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let system_current = motor_currents.iter().sum::<f32>() as f64;

        let telemetry = Telemetry {
            timestamp_ms,
            mode,
            pose,
            power: PowerStatus {
                battery_voltage,
                system_current,
            },
            velocity,
            motor_temps,
            motor_currents,
            active_tool,
            tool_status,
            slam_status,
            health,
        };

        self.last_telemetry = Some(telemetry.clone());
        telemetry
    }

    /// Get the last built telemetry (if any)
    pub fn last(&self) -> Option<&Telemetry> {
        self.last_telemetry.as_ref()
    }
}

impl Default for TelemetryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to extract motor data from VESC states
pub struct MotorData {
    pub temps: [f32; 4],
    pub currents: [f32; 4],
    pub tachometers: [i32; 4],
}

impl MotorData {
    /// Extract motor data from an array of VESC states
    pub fn from_vesc_states<S: VescState>(states: &[S; 4]) -> Self {
        Self {
            temps: [
                states[0].motor_temp(),
                states[1].motor_temp(),
                states[2].motor_temp(),
                states[3].motor_temp(),
            ],
            currents: [
                states[0].current(),
                states[1].current(),
                states[2].current(),
                states[3].current(),
            ],
            tachometers: [
                states[0].tachometer(),
                states[1].tachometer(),
                states[2].tachometer(),
                states[3].tachometer(),
            ],
        }
    }

    /// Total system current
    pub fn total_current(&self) -> f64 {
        self.currents.iter().sum::<f32>() as f64
    }
}

/// Trait for extracting data from VESC state structs
pub trait VescState {
    fn motor_temp(&self) -> f32;
    fn current(&self) -> f32;
    fn tachometer(&self) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_builder() {
        let mut builder = TelemetryBuilder::new();

        let telemetry = builder.build(
            Mode::Teleop,
            Pose {
                x: 1.0,
                y: 2.0,
                theta: 0.5,
            },
            48.0,
            [30.0, 31.0, 32.0, 33.0],
            [5.0, 5.5, 6.0, 6.5],
            Twist {
                linear: 1.0,
                angular: 0.5,
                boost: false,
            },
            None,
            None,
            None,
            SubsystemHealth::default(),
        );

        assert_eq!(telemetry.mode, Mode::Teleop);
        assert!((telemetry.pose.x - 1.0).abs() < 0.001);
        assert!((telemetry.power.battery_voltage - 48.0).abs() < 0.001);
        assert!((telemetry.power.system_current - 23.0).abs() < 0.001); // Sum of currents
        assert!(builder.last().is_some());
    }

    #[test]
    fn test_telemetry_with_slam() {
        let mut builder = TelemetryBuilder::new();

        let slam_status = SlamStatus {
            pose: Pose {
                x: 1.0,
                y: 2.0,
                theta: 0.5,
            },
            confidence: 0.9,
            keyframe_count: 100,
            loop_closure_count: 5,
            mapping_active: true,
        };

        let telemetry = builder.build(
            Mode::Autonomous,
            Pose {
                x: 1.0,
                y: 2.0,
                theta: 0.5,
            },
            48.0,
            [30.0; 4],
            [5.0; 4],
            Twist::default(),
            None,
            None,
            Some(slam_status),
            SubsystemHealth::default(),
        );

        assert!(telemetry.slam_status.is_some());
        let slam = telemetry.slam_status.unwrap();
        assert_eq!(slam.keyframe_count, 100);
    }

    #[test]
    fn test_motor_data() {
        // Simple test struct implementing VescState
        struct TestVesc {
            temp: f32,
            current: f32,
            tach: i32,
        }

        impl VescState for TestVesc {
            fn motor_temp(&self) -> f32 {
                self.temp
            }
            fn current(&self) -> f32 {
                self.current
            }
            fn tachometer(&self) -> i32 {
                self.tach
            }
        }

        let states = [
            TestVesc {
                temp: 30.0,
                current: 5.0,
                tach: 100,
            },
            TestVesc {
                temp: 31.0,
                current: 5.5,
                tach: 101,
            },
            TestVesc {
                temp: 32.0,
                current: 6.0,
                tach: 102,
            },
            TestVesc {
                temp: 33.0,
                current: 6.5,
                tach: 103,
            },
        ];

        let data = MotorData::from_vesc_states(&states);

        assert!((data.temps[0] - 30.0).abs() < 0.001);
        assert!((data.total_current() - 23.0).abs() < 0.001);
        assert_eq!(data.tachometers[2], 102);
    }
}
