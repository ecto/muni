//! Wheel odometry and pose estimation for bvr.
//!
//! Provides:
//! - Wheel odometry from VESC tachometer readings
//! - Pose estimator that fuses odometry with GPS
//! - IMU attitude estimation with Kalman filtering

mod attitude;
mod estimator;
mod odometry;

pub use attitude::{Attitude, AttitudeConfig, AttitudeFilter};
pub use estimator::PoseEstimator;
pub use odometry::WheelOdometry;



