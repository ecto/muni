//! IMU simulation from physics state.
//!
//! Produces gyro and accelerometer readings from the rover's physics.

use crate::physics::Physics;

/// IMU data derived from physics.
pub struct ImuData {
    pub gyro: [f32; 3],
    pub accel: [f32; 3],
}

/// Generate IMU readings from physics state.
///
/// In 2D simulation:
/// - gyro_z = angular velocity (yaw rate)
/// - accel_z = gravity (9.81 m/s²)
/// - other axes are zero
pub fn imu_from_physics(physics: &Physics) -> ImuData {
    let (_, angular_vel) = physics.velocity();
    ImuData {
        gyro: [0.0, 0.0, angular_vel as f32],
        accel: [0.0, 0.0, 9.81],
    }
}
