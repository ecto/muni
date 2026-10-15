//! Visual odometry via 2D ICP on depth-derived point clouds.
//!
//! Projects 3D depth points to the ground plane and aligns consecutive
//! "scans" using Iterative Closest Point to estimate ego-motion (dx, dy, dθ).
//!
//! Returns the same `(dx, dy, dtheta)` tuple as `WheelOdometry`, so it
//! can plug directly into the EKF prediction step.

use crate::ClassifiedPoints;
use kiddo::{KdTree, SquaredEuclidean};
use nalgebra::{Matrix2, Vector2};
use tracing::{debug, trace, warn};

/// ICP configuration.
#[derive(Debug, Clone)]
pub struct IcpConfig {
    /// Maximum iterations per alignment.
    pub max_iterations: usize,
    /// Convergence threshold on transform delta (meters + radians).
    pub convergence_threshold: f64,
    /// Maximum correspondence distance (meters). Points farther are rejected.
    pub max_correspondence_dist: f64,
    /// Minimum inlier fraction to accept alignment (0.0-1.0).
    pub min_inlier_ratio: f64,
}

impl Default for IcpConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            convergence_threshold: 1e-4,
            max_correspondence_dist: 0.5,
            min_inlier_ratio: 0.3,
        }
    }
}

/// Visual odometry configuration.
#[derive(Debug, Clone)]
pub struct VisualOdometryConfig {
    /// ICP parameters.
    pub icp: IcpConfig,
    /// Maximum plausible displacement per frame (meters). Rejects outlier estimates.
    pub max_displacement: f64,
    /// Maximum plausible rotation per frame (radians).
    pub max_rotation: f64,
    /// Minimum points required for alignment.
    pub min_points: usize,
}

impl Default for VisualOdometryConfig {
    fn default() -> Self {
        Self {
            icp: IcpConfig::default(),
            // At 15fps and 3m/s max speed: ~0.2m per frame
            max_displacement: 0.5,
            max_rotation: 0.3,
            min_points: 50,
        }
    }
}

/// 2D rigid transform result from ICP alignment.
#[derive(Debug, Clone, Copy)]
struct Transform2D {
    /// Translation (meters)
    translation: Vector2<f64>,
    /// Rotation angle (radians)
    rotation: f64,
}

/// Visual odometry from depth-derived 2D scans.
///
/// Tracks ego-motion by running 2D ICP between consecutive depth frames
/// projected to the ground plane. Produces (dx, dy, dtheta) updates
/// compatible with the EKF prediction step.
pub struct VisualOdometry {
    /// Previous frame's 2D points (rover body frame, projected to ground plane)
    prev_points: Option<Vec<[f64; 2]>>,
    /// Configuration
    config: VisualOdometryConfig,
    /// Accumulated distance
    total_distance: f64,
    /// Frame counter
    frame_count: u64,
}

impl VisualOdometry {
    /// Create a new visual odometry tracker.
    pub fn new(config: VisualOdometryConfig) -> Self {
        Self {
            prev_points: None,
            config,
            total_distance: 0.0,
            frame_count: 0,
        }
    }

    /// Update with classified 3D points from one or more cameras.
    ///
    /// Projects all points to 2D (ground plane) and runs ICP against
    /// the previous frame. Returns `(dx, dy, dtheta)` in rover body frame.
    ///
    /// On the first call, stores points and returns zero displacement.
    pub fn update(&mut self, classified: &ClassifiedPoints) -> (f64, f64, f64) {
        self.frame_count += 1;

        // Project to 2D ground plane (use all points for geometry)
        let points_2d: Vec<[f64; 2]> = classified
            .ground
            .iter()
            .chain(classified.obstacles.iter())
            .map(|p| [p.x, p.y])
            .collect();

        if points_2d.len() < self.config.min_points {
            warn!(
                count = points_2d.len(),
                min = self.config.min_points,
                "vo: too few points, skipping"
            );
            self.prev_points = Some(points_2d);
            return (0.0, 0.0, 0.0);
        }

        let result = if let Some(ref prev) = self.prev_points {
            if prev.len() >= self.config.min_points {
                match icp_2d(prev, &points_2d, &self.config.icp) {
                    Some(tf) => {
                        // Sanity check: reject implausible estimates
                        if tf.translation.norm() > self.config.max_displacement
                            || tf.rotation.abs() > self.config.max_rotation
                        {
                            warn!(
                                dx = tf.translation.x,
                                dy = tf.translation.y,
                                dtheta = tf.rotation,
                                "vo: implausible estimate, clamping to zero"
                            );
                            (0.0, 0.0, 0.0)
                        } else {
                            self.total_distance += tf.translation.norm();
                            (tf.translation.x, tf.translation.y, tf.rotation)
                        }
                    }
                    None => {
                        debug!("vo: ICP failed to converge");
                        (0.0, 0.0, 0.0)
                    }
                }
            } else {
                (0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0)
        };

        self.prev_points = Some(points_2d);

        trace!(
            dx = result.0,
            dy = result.1,
            dtheta = result.2,
            frame = self.frame_count,
            "vo.update"
        );

        result
    }

    /// Get total distance traveled (meters).
    pub fn total_distance(&self) -> f64 {
        self.total_distance
    }

    /// Reset state (e.g., after relocalization).
    pub fn reset(&mut self) {
        self.prev_points = None;
        self.total_distance = 0.0;
    }
}

/// Run 2D ICP: find the rigid transform that aligns `source` to `reference`.
///
/// Returns the transform that maps source points to reference points,
/// i.e., the motion of the rover between frames (reference = previous, source = current).
fn icp_2d(reference: &[[f64; 2]], source: &[[f64; 2]], config: &IcpConfig) -> Option<Transform2D> {
    // Build KD-tree from reference points
    let mut tree: KdTree<f64, 2> = KdTree::new();
    for (i, pt) in reference.iter().enumerate() {
        tree.add(pt, i as u64);
    }

    let max_dist_sq = config.max_correspondence_dist * config.max_correspondence_dist;
    let min_inliers = (source.len() as f64 * config.min_inlier_ratio) as usize;

    // Working copy of source points (transformed each iteration)
    let mut transformed: Vec<[f64; 2]> = source.to_vec();

    // Accumulated transform
    let mut total_cos = 1.0f64;
    let mut total_sin = 0.0f64;
    let mut total_tx = 0.0f64;
    let mut total_ty = 0.0f64;

    for iter in 0..config.max_iterations {
        // Find correspondences
        let mut ref_matched = Vec::with_capacity(source.len());
        let mut src_matched = Vec::with_capacity(source.len());

        for pt in &transformed {
            let nearest = tree.nearest_one::<SquaredEuclidean>(pt);
            if nearest.distance <= max_dist_sq {
                let ref_pt = reference[nearest.item as usize];
                ref_matched.push(Vector2::new(ref_pt[0], ref_pt[1]));
                src_matched.push(Vector2::new(pt[0], pt[1]));
            }
        }

        if ref_matched.len() < min_inliers {
            debug!(
                inliers = ref_matched.len(),
                min = min_inliers,
                iter,
                "icp: insufficient inliers"
            );
            return None;
        }

        // Compute centroids
        let n = ref_matched.len() as f64;
        let ref_centroid: Vector2<f64> =
            ref_matched.iter().sum::<Vector2<f64>>() / n;
        let src_centroid: Vector2<f64> =
            src_matched.iter().sum::<Vector2<f64>>() / n;

        // Compute cross-covariance matrix H
        let mut h = Matrix2::zeros();
        for (r, s) in ref_matched.iter().zip(src_matched.iter()) {
            let dr = r - ref_centroid;
            let ds = s - src_centroid;
            h += ds * dr.transpose();
        }

        // SVD of H → rotation
        let svd = h.svd(true, true);
        let u = svd.u?;
        let vt = svd.v_t?;

        // R = V * U^T, ensuring proper rotation (det = +1)
        let v = vt.transpose();
        let ut = u.transpose();
        let mut rot = v * ut;
        if rot.determinant() < 0.0 {
            // Flip sign of last column of V
            let mut v_fixed = v;
            v_fixed[(0, 1)] = -v_fixed[(0, 1)];
            v_fixed[(1, 1)] = -v_fixed[(1, 1)];
            rot = v_fixed * ut;
        }

        // Translation: t = ref_centroid - R * src_centroid
        let t = ref_centroid - rot * src_centroid;

        let step_cos = rot[(0, 0)];
        let step_sin = rot[(1, 0)];
        let step_tx = t.x;
        let step_ty = t.y;

        // Apply step to transformed points
        for pt in &mut transformed {
            let x = pt[0];
            let y = pt[1];
            pt[0] = step_cos * x - step_sin * y + step_tx;
            pt[1] = step_sin * x + step_cos * y + step_ty;
        }

        // Accumulate transform
        let new_cos = step_cos * total_cos - step_sin * total_sin;
        let new_sin = step_sin * total_cos + step_cos * total_sin;
        let new_tx = step_cos * total_tx - step_sin * total_ty + step_tx;
        let new_ty = step_sin * total_tx + step_cos * total_ty + step_ty;
        total_cos = new_cos;
        total_sin = new_sin;
        total_tx = new_tx;
        total_ty = new_ty;

        // Check convergence
        let step_angle = step_sin.atan2(step_cos).abs();
        let step_trans = (step_tx * step_tx + step_ty * step_ty).sqrt();
        if step_angle + step_trans < config.convergence_threshold {
            trace!(iter, inliers = ref_matched.len(), "icp: converged");
            break;
        }
    }

    let angle = total_sin.atan2(total_cos);

    Some(Transform2D {
        translation: Vector2::new(total_tx, total_ty),
        rotation: angle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    /// Generate a distinctive point cloud (L-shape + scattered objects)
    /// that avoids ICP's aperture problem on straight edges.
    fn make_scene_points() -> Vec<[f64; 2]> {
        let mut pts = Vec::new();

        // L-shaped wall (2 edges at 90°)
        for i in 0..30 {
            let t = i as f64 * 0.1;
            pts.push([1.0 + t, -1.0]); // horizontal wall
            pts.push([1.0, -1.0 + t]); // vertical wall
        }

        // Circular obstacle at (3, 1)
        for i in 0..20 {
            let angle = i as f64 * std::f64::consts::PI * 2.0 / 20.0;
            pts.push([3.0 + 0.3 * angle.cos(), 1.0 + 0.3 * angle.sin()]);
        }

        // Box at (2, -2)
        for i in 0..10 {
            let t = i as f64 * 0.05;
            pts.push([1.8 + t, -2.0]);
            pts.push([1.8 + t, -1.7]);
            pts.push([1.8, -2.0 + t * 6.0]);
            pts.push([2.3, -2.0 + t * 6.0]);
        }

        pts
    }

    fn transform_points(pts: &[[f64; 2]], dx: f64, dy: f64, dtheta: f64) -> Vec<[f64; 2]> {
        let cos_t = dtheta.cos();
        let sin_t = dtheta.sin();
        pts.iter()
            .map(|p| {
                [
                    cos_t * p[0] - sin_t * p[1] + dx,
                    sin_t * p[0] + cos_t * p[1] + dy,
                ]
            })
            .collect()
    }

    #[test]
    fn test_icp_identity() {
        let pts = make_scene_points();
        let config = IcpConfig::default();

        let result = icp_2d(&pts, &pts, &config).unwrap();

        assert!(
            result.translation.norm() < 0.01,
            "translation should be ~0, got {:?}",
            result.translation
        );
        assert!(
            result.rotation.abs() < 0.01,
            "rotation should be ~0, got {}",
            result.rotation
        );
    }

    #[test]
    fn test_icp_pure_translation() {
        let reference = make_scene_points();
        let source = transform_points(&reference, 0.1, 0.05, 0.0);

        let config = IcpConfig {
            max_correspondence_dist: 1.0,
            ..Default::default()
        };

        let result = icp_2d(&reference, &source, &config).unwrap();

        assert!(
            (result.translation.x - (-0.1)).abs() < 0.02,
            "dx={:.4} expected ~-0.1",
            result.translation.x
        );
        assert!(
            (result.translation.y - (-0.05)).abs() < 0.02,
            "dy={:.4} expected ~-0.05",
            result.translation.y
        );
    }

    #[test]
    fn test_icp_pure_rotation() {
        let reference = make_scene_points();
        let dtheta = 0.05;
        let source = transform_points(&reference, 0.0, 0.0, dtheta);

        let config = IcpConfig {
            max_correspondence_dist: 1.0,
            ..Default::default()
        };

        let result = icp_2d(&reference, &source, &config).unwrap();

        assert!(
            (result.rotation - (-dtheta)).abs() < 0.02,
            "rotation={:.4} expected ~{:.4}",
            result.rotation,
            -dtheta
        );
    }

    #[test]
    fn test_icp_combined_motion() {
        let reference = make_scene_points();
        let dx = 0.08;
        let dy = -0.03;
        let dtheta = 0.04;
        let source = transform_points(&reference, dx, dy, dtheta);

        let config = IcpConfig {
            max_correspondence_dist: 1.0,
            ..Default::default()
        };

        let result = icp_2d(&reference, &source, &config).unwrap();

        assert!(
            (result.translation.x - (-dx)).abs() < 0.03,
            "dx={:.4}",
            result.translation.x
        );
        assert!(
            (result.translation.y - (-dy)).abs() < 0.03,
            "dy={:.4}",
            result.translation.y
        );
        assert!(
            (result.rotation - (-dtheta)).abs() < 0.03,
            "rotation={:.4}",
            result.rotation
        );
    }

    #[test]
    fn test_visual_odometry_first_frame() {
        let mut vo = VisualOdometry::new(VisualOdometryConfig {
            min_points: 10,
            ..Default::default()
        });

        let pts = ClassifiedPoints {
            ground: (0..50)
                .map(|i| Vector3::new(i as f64 * 0.1, 0.0, 0.0))
                .collect(),
            obstacles: (0..50)
                .map(|i| Vector3::new(2.0, i as f64 * 0.1 - 2.5, 0.5))
                .collect(),
        };

        let (dx, dy, dtheta) = vo.update(&pts);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
        assert_eq!(dtheta, 0.0);
    }

    #[test]
    fn test_visual_odometry_stationary() {
        let mut vo = VisualOdometry::new(VisualOdometryConfig {
            min_points: 10,
            ..Default::default()
        });

        let pts = ClassifiedPoints {
            ground: Vec::new(),
            obstacles: make_scene_points()
                .into_iter()
                .map(|p| Vector3::new(p[0], p[1], 0.3))
                .collect(),
        };

        // First frame
        vo.update(&pts);

        // Second frame with same points (stationary)
        let (dx, dy, dtheta) = vo.update(&pts);

        assert!(dx.abs() < 0.01, "dx={dx}");
        assert!(dy.abs() < 0.01, "dy={dy}");
        assert!(dtheta.abs() < 0.01, "dtheta={dtheta}");
    }

    #[test]
    fn test_visual_odometry_too_few_points() {
        let mut vo = VisualOdometry::new(VisualOdometryConfig {
            min_points: 100,
            ..Default::default()
        });

        let pts = ClassifiedPoints {
            ground: vec![Vector3::new(1.0, 0.0, 0.0)],
            obstacles: vec![Vector3::new(2.0, 1.0, 0.5)],
        };

        let (dx, dy, dtheta) = vo.update(&pts);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
        assert_eq!(dtheta, 0.0);
    }
}
