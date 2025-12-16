//! Wheel odometry and pose estimation for bvr.
//!
//! Provides:
//! - Wheel odometry from VESC tachometer readings
//! - Pose estimator that fuses odometry with GPS

mod odometry;
mod estimator;

pub use odometry::WheelOdometry;
pub use estimator::PoseEstimator;
