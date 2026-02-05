//! Telemetry Builder
//!
//! Collects telemetry data from all subsystems and builds the Telemetry
//! struct for transmission to operators.

use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU16, Ordering};
use teleop::{Telemetry, ToolStatus};
use types::{Mode, Pose, PowerStatus, SlamStatus, SubsystemHealth, Twist};

/// Global telemetry sequence counter
static TELEMETRY_SEQUENCE: AtomicU16 = AtomicU16::new(0);

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
        cmd_velocity: Twist,
        meas_velocity: (f32, f32),
        acceleration: (f32, f32),
        active_tool: Option<String>,
        tool_status: Option<ToolStatus>,
        slam_status: Option<SlamStatus>,
        health: SubsystemHealth,
        odometry_quality: u16,
        dt_ms: f32,
        last_cmd_seq: u16,
        ack_bits: u16,
    ) -> Telemetry {
        let timestamp_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let sequence = TELEMETRY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let system_current = motor_currents.iter().sum::<f32>() as f64;

        let telemetry = Telemetry {
            sequence,
            timestamp_us,
            mode,
            pose,
            power: PowerStatus {
                battery_voltage,
                system_current,
            },
            cmd_velocity,
            meas_velocity,
            acceleration,
            motor_temps,
            motor_currents,
            active_tool,
            tool_status,
            slam_status,
            health,
            odometry_quality,
            dt_ms,
            last_cmd_seq,
            ack_bits,
            // IMU orientation (set elsewhere, main loop reads from IMU channel)
            roll: 0.0,
            pitch: 0.0,
            // System metrics are set elsewhere (main loop)
            cpu_percent: 0,
            mem_percent: 0,
            disk_percent: 0,
            lidar_core_temp_c: None,
            lidar_work_state: 0,
            mode_changed_at: 0,
            policy_intention: None,
            fault_code: 0,
            active_tool_type: 0,
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
            (1.0, 0.5), // meas_velocity
            (0.0, 0.0), // acceleration
            None,
            None,
            None,
            SubsystemHealth::default(),
            100, // odometry_quality
            20.0, // dt_ms
            0, // last_cmd_seq
            0, // ack_bits
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
            (0.0, 0.0), // meas_velocity
            (0.0, 0.0), // acceleration
            None,
            None,
            Some(slam_status),
            SubsystemHealth::default(),
            100, // odometry_quality
            20.0, // dt_ms
            0, // last_cmd_seq
            0, // ack_bits
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
