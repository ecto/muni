//! Wheel odometry and pose estimation for bvr.
//!
//! Provides:
//! - Wheel odometry from VESC tachometer readings
//! - EKF pose estimator fusing odometry, GPS, IMU yaw, and SLAM
//! - IMU attitude estimation with Kalman filtering

mod attitude;
mod estimator;
mod odometry;

pub use attitude::{Attitude, AttitudeConfig, AttitudeFilter};
pub use estimator::{EkfConfig, EkfPoseEstimator};
pub use odometry::WheelOdometry;

// Re-export old name for backward compatibility during transition
pub use estimator::EkfPoseEstimator as PoseEstimator;
