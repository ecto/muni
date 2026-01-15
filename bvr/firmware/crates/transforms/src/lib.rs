//! Coordinate frame management for SLAM and navigation.
//!
//! Provides transform handling following standard robotics conventions:
//! - `world` (map): Global fixed frame, SLAM-corrected
//! - `odom`: Continuous odometry frame, drifts over time
//! - `base_link`: Robot body frame, center of rear axle
//! - `lidar`: LiDAR sensor frame, fixed offset from base_link
//!
//! Transform chain: world <- odom <- base_link <- lidar
//!                         ^
//!                         └── SLAM corrects this discontinuously

use nalgebra::{Isometry2, Matrix3, Vector2};
use std::f64::consts::PI;
use thiserror::Error;
use types::Pose;

#[derive(Error, Debug)]
pub enum TransformError {
    #[error("No transform from {from:?} to {to:?}")]
    NotFound { from: FrameId, to: FrameId },
    #[error("Transform chain broken at {frame:?}")]
    ChainBroken { frame: FrameId },
}

/// Coordinate frame identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameId {
    /// Global/map frame (fixed, SLAM-corrected)
    World,
    /// Odometry frame (continuous, drifts over time)
    Odom,
    /// Robot body frame (center of rear axle, X forward, Y left)
    BaseLink,
    /// LiDAR sensor frame (fixed transform from base_link)
    Lidar,
}

/// A 2D rigid body transform (translation + rotation).
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    inner: Isometry2<f64>,
}

impl Transform2D {
    /// Create identity transform.
    pub fn identity() -> Self {
        Self {
            inner: Isometry2::identity(),
        }
    }

    /// Create transform from translation and rotation angle.
    pub fn new(x: f64, y: f64, theta: f64) -> Self {
        Self {
            inner: Isometry2::new(Vector2::new(x, y), theta),
        }
    }

    /// Create from a Pose struct.
    pub fn from_pose(pose: &Pose) -> Self {
        Self::new(pose.x, pose.y, pose.theta)
    }

    /// Convert to a Pose struct.
    pub fn to_pose(&self) -> Pose {
        Pose {
            x: self.inner.translation.x,
            y: self.inner.translation.y,
            theta: self.inner.rotation.angle(),
        }
    }

    /// Get the underlying nalgebra Isometry2.
    pub fn as_isometry(&self) -> &Isometry2<f64> {
        &self.inner
    }

    /// Create from nalgebra Isometry2.
    pub fn from_isometry(iso: Isometry2<f64>) -> Self {
        Self { inner: iso }
    }

    /// Get translation component.
    pub fn translation(&self) -> Vector2<f64> {
        self.inner.translation.vector
    }

    /// Get rotation angle in radians.
    pub fn rotation(&self) -> f64 {
        self.inner.rotation.angle()
    }

    /// Compute inverse transform.
    pub fn inverse(&self) -> Self {
        Self {
            inner: self.inner.inverse(),
        }
    }

    /// Compose transforms: self * other.
    /// If self is A->B and other is B->C, result is A->C.
    pub fn compose(&self, other: &Transform2D) -> Transform2D {
        Transform2D {
            inner: self.inner * other.inner,
        }
    }

    /// Transform a point from child frame to parent frame.
    pub fn transform_point(&self, point: Vector2<f64>) -> Vector2<f64> {
        self.inner.transform_point(&nalgebra::Point2::from(point)).coords
    }

    /// Transform a pose from child frame to parent frame.
    pub fn transform_pose(&self, pose: &Pose) -> Pose {
        let child_iso = Isometry2::new(Vector2::new(pose.x, pose.y), pose.theta);
        let result = self.inner * child_iso;
        Pose {
            x: result.translation.x,
            y: result.translation.y,
            theta: normalize_angle(result.rotation.angle()),
        }
    }

    /// Compute relative transform: from self to other.
    /// If self is A and other is B (both in same frame), returns A->B transform.
    pub fn relative_to(&self, other: &Transform2D) -> Transform2D {
        Transform2D {
            inner: self.inner.inverse() * other.inner,
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Transform2D;

    fn mul(self, rhs: Transform2D) -> Transform2D {
        self.compose(&rhs)
    }
}

impl std::ops::Mul<&Transform2D> for Transform2D {
    type Output = Transform2D;

    fn mul(self, rhs: &Transform2D) -> Transform2D {
        self.compose(rhs)
    }
}

impl std::ops::Mul<Transform2D> for &Transform2D {
    type Output = Transform2D;

    fn mul(self, rhs: Transform2D) -> Transform2D {
        self.compose(&rhs)
    }
}

impl std::ops::Mul<&Transform2D> for &Transform2D {
    type Output = Transform2D;

    fn mul(self, rhs: &Transform2D) -> Transform2D {
        self.compose(rhs)
    }
}

/// Transform tree for managing coordinate frames.
///
/// Stores transforms between frames and provides lookup functionality.
#[derive(Debug, Clone)]
pub struct TransformTree {
    /// odom -> world correction (updated by SLAM)
    odom_to_world: Transform2D,
    /// base_link -> odom (updated by odometry)
    base_to_odom: Transform2D,
    /// lidar -> base_link (static, from calibration)
    lidar_to_base: Transform2D,
}

impl TransformTree {
    /// Create a new transform tree with identity transforms.
    pub fn new() -> Self {
        Self {
            odom_to_world: Transform2D::identity(),
            base_to_odom: Transform2D::identity(),
            lidar_to_base: Self::default_lidar_to_base(),
        }
    }

    /// Default LiDAR to base_link transform.
    /// LiDAR is mounted at front of rover, facing forward.
    /// Adjust these values based on actual mounting position.
    fn default_lidar_to_base() -> Transform2D {
        // LiDAR mounted 0.3m forward of base_link center, at same height
        // Facing forward (0 rotation)
        Transform2D::new(0.3, 0.0, 0.0)
    }

    /// Set the LiDAR mounting transform (from calibration).
    pub fn set_lidar_mount(&mut self, lidar_to_base: Transform2D) {
        self.lidar_to_base = lidar_to_base;
    }

    /// Update base_link pose in odom frame (from wheel odometry).
    pub fn update_odom(&mut self, base_in_odom: Transform2D) {
        self.base_to_odom = base_in_odom;
    }

    /// Update odom to world correction (from SLAM).
    pub fn update_slam_correction(&mut self, odom_to_world: Transform2D) {
        self.odom_to_world = odom_to_world;
    }

    /// Get current base_link pose in odom frame.
    pub fn base_in_odom(&self) -> Transform2D {
        self.base_to_odom
    }

    /// Get current base_link pose in world frame (SLAM-corrected).
    pub fn base_in_world(&self) -> Transform2D {
        &self.odom_to_world * &self.base_to_odom
    }

    /// Get current LiDAR pose in odom frame.
    pub fn lidar_in_odom(&self) -> Transform2D {
        &self.base_to_odom * &self.lidar_to_base
    }

    /// Get current LiDAR pose in world frame.
    pub fn lidar_in_world(&self) -> Transform2D {
        &self.odom_to_world * &self.base_to_odom * &self.lidar_to_base
    }

    /// Get the odom->world correction transform.
    pub fn odom_correction(&self) -> Transform2D {
        self.odom_to_world
    }

    /// Transform a point from one frame to another.
    pub fn transform_point(
        &self,
        point: Vector2<f64>,
        from: FrameId,
        to: FrameId,
    ) -> Result<Vector2<f64>, TransformError> {
        let tf = self.lookup(from, to)?;
        Ok(tf.transform_point(point))
    }

    /// Transform a pose from one frame to another.
    pub fn transform_pose(
        &self,
        pose: &Pose,
        from: FrameId,
        to: FrameId,
    ) -> Result<Pose, TransformError> {
        let tf = self.lookup(from, to)?;
        Ok(tf.transform_pose(pose))
    }

    /// Lookup transform from child frame to parent frame.
    pub fn lookup(&self, from: FrameId, to: FrameId) -> Result<Transform2D, TransformError> {
        use FrameId::*;

        if from == to {
            return Ok(Transform2D::identity());
        }

        // Build chain from 'from' up to world, then down to 'to'
        match (from, to) {
            // Direct transforms up the chain
            (Lidar, BaseLink) => Ok(self.lidar_to_base),
            (BaseLink, Odom) => Ok(self.base_to_odom),
            (Odom, World) => Ok(self.odom_to_world),

            // Direct transforms down the chain (inverses)
            (BaseLink, Lidar) => Ok(self.lidar_to_base.inverse()),
            (Odom, BaseLink) => Ok(self.base_to_odom.inverse()),
            (World, Odom) => Ok(self.odom_to_world.inverse()),

            // Multi-step transforms
            (Lidar, Odom) => Ok(&self.base_to_odom * &self.lidar_to_base),
            (Lidar, World) => Ok(&self.odom_to_world * &self.base_to_odom * &self.lidar_to_base),
            (BaseLink, World) => Ok(&self.odom_to_world * &self.base_to_odom),

            (Odom, Lidar) => Ok((&self.base_to_odom * &self.lidar_to_base).inverse()),
            (World, Lidar) => {
                Ok((&self.odom_to_world * &self.base_to_odom * &self.lidar_to_base).inverse())
            }
            (World, BaseLink) => Ok((&self.odom_to_world * &self.base_to_odom).inverse()),

            // Same frame (identity) - handled above but needed for exhaustiveness
            (World, World) | (Odom, Odom) | (BaseLink, BaseLink) | (Lidar, Lidar) => {
                unreachable!("Same-frame case handled above")
            }
        }
    }
}

impl Default for TransformTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize angle to [-PI, PI).
pub fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle % (2.0 * PI);
    if a >= PI {
        a -= 2.0 * PI;
    } else if a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Compute shortest angular difference from a to b.
pub fn angle_diff(a: f64, b: f64) -> f64 {
    normalize_angle(b - a)
}

/// Compute 2D rotation matrix for given angle.
pub fn rotation_matrix(theta: f64) -> Matrix3<f64> {
    let c = theta.cos();
    let s = theta.sin();
    Matrix3::new(c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_transform_identity() {
        let tf = Transform2D::identity();
        assert_relative_eq!(tf.translation().x, 0.0);
        assert_relative_eq!(tf.translation().y, 0.0);
        assert_relative_eq!(tf.rotation(), 0.0);
    }

    #[test]
    fn test_transform_from_pose() {
        let pose = Pose {
            x: 1.0,
            y: 2.0,
            theta: PI / 4.0,
        };
        let tf = Transform2D::from_pose(&pose);
        let back = tf.to_pose();
        assert_relative_eq!(back.x, pose.x, epsilon = 1e-10);
        assert_relative_eq!(back.y, pose.y, epsilon = 1e-10);
        assert_relative_eq!(back.theta, pose.theta, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_inverse() {
        let tf = Transform2D::new(1.0, 2.0, PI / 2.0);
        let inv = tf.inverse();
        let composed = tf.compose(&inv);
        assert_relative_eq!(composed.translation().x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(composed.translation().y, 0.0, epsilon = 1e-10);
        assert_relative_eq!(composed.rotation(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_compose() {
        // A->B is translate by (1, 0), B->C is rotate 90 degrees CCW
        // Composing: A->C = A->B then B->C
        let a_to_b = Transform2D::new(1.0, 0.0, 0.0);
        let b_to_c = Transform2D::new(0.0, 0.0, PI / 2.0);
        let a_to_c = b_to_c.compose(&a_to_b);

        // Point at origin in frame A should:
        // 1. After a_to_b: be at (1, 0) in B
        // 2. After b_to_c (rotate 90 CCW): be at (0, 1) in C
        let point = Vector2::new(0.0, 0.0);
        let result = a_to_c.transform_point(point);
        assert_relative_eq!(result.x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(result.y, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_point() {
        // Translate by (1, 2), rotate 90 degrees CCW
        let tf = Transform2D::new(1.0, 2.0, PI / 2.0);
        let point = Vector2::new(1.0, 0.0);
        let result = tf.transform_point(point);
        // Rotate (1, 0) by 90 CCW = (0, 1), then translate by (1, 2) = (1, 3)
        assert_relative_eq!(result.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(result.y, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_tree_identity() {
        let tree = TransformTree::new();
        let pose = Pose {
            x: 1.0,
            y: 2.0,
            theta: 0.5,
        };

        // With identity transforms, odom and world should be same
        let in_world = tree.transform_pose(&pose, FrameId::Odom, FrameId::World).unwrap();
        assert_relative_eq!(in_world.x, pose.x, epsilon = 1e-10);
        assert_relative_eq!(in_world.y, pose.y, epsilon = 1e-10);
        assert_relative_eq!(in_world.theta, pose.theta, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_tree_with_odom() {
        let mut tree = TransformTree::new();

        // Robot has moved to (5, 3) with heading PI/4 in odom frame
        tree.update_odom(Transform2D::new(5.0, 3.0, PI / 4.0));

        // Point at robot origin should be at (5, 3) in odom
        let result = tree.transform_point(Vector2::zeros(), FrameId::BaseLink, FrameId::Odom).unwrap();
        assert_relative_eq!(result.x, 5.0, epsilon = 1e-10);
        assert_relative_eq!(result.y, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_transform_tree_with_slam_correction() {
        let mut tree = TransformTree::new();

        // Robot at (5, 3) in odom
        tree.update_odom(Transform2D::new(5.0, 3.0, 0.0));

        // SLAM says odom has drifted: actual position is (5.1, 3.2)
        // So odom->world correction is (0.1, 0.2)
        tree.update_slam_correction(Transform2D::new(0.1, 0.2, 0.0));

        let world_pose = tree.base_in_world().to_pose();
        assert_relative_eq!(world_pose.x, 5.1, epsilon = 1e-10);
        assert_relative_eq!(world_pose.y, 3.2, epsilon = 1e-10);
    }

    #[test]
    fn test_normalize_angle() {
        assert_relative_eq!(normalize_angle(0.0), 0.0, epsilon = 1e-10);
        // PI normalizes to -PI (range is [-PI, PI))
        assert_relative_eq!(normalize_angle(PI).abs(), PI, epsilon = 1e-10);
        assert_relative_eq!(normalize_angle(-PI), -PI, epsilon = 1e-10);
        assert_relative_eq!(normalize_angle(2.0 * PI), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normalize_angle(3.0 * PI).abs(), PI, epsilon = 1e-10);
        assert_relative_eq!(normalize_angle(-3.0 * PI), -PI, epsilon = 1e-10);
    }

    #[test]
    fn test_angle_diff() {
        assert_relative_eq!(angle_diff(0.0, PI / 2.0), PI / 2.0, epsilon = 1e-10);
        assert_relative_eq!(angle_diff(PI / 2.0, 0.0), -PI / 2.0, epsilon = 1e-10);
        // Crossing the -PI/PI boundary
        assert_relative_eq!(angle_diff(-0.9 * PI, 0.9 * PI), -0.2 * PI, epsilon = 1e-10);
    }

    #[test]
    fn test_lookup_roundtrip() {
        let mut tree = TransformTree::new();
        tree.update_odom(Transform2D::new(1.0, 2.0, 0.5));
        tree.update_slam_correction(Transform2D::new(0.1, 0.1, 0.01));

        // Going from lidar to world and back should be identity
        let tf_up = tree.lookup(FrameId::Lidar, FrameId::World).unwrap();
        let tf_down = tree.lookup(FrameId::World, FrameId::Lidar).unwrap();
        let composed = tf_up.compose(&tf_down);

        assert_relative_eq!(composed.translation().x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(composed.translation().y, 0.0, epsilon = 1e-10);
        assert_relative_eq!(composed.rotation(), 0.0, epsilon = 1e-10);
    }
}
