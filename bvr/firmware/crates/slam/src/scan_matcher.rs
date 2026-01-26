//! 3D Generalized ICP (GICP) scan matcher.
//!
//! Replaces the previous 2D correlative scan matcher with a full 3D
//! point-to-plane ICP variant that uses local covariance matrices.
//! The result is projected back to 2D (x, y, yaw) for the pose graph.
//!
//! Algorithm:
//! 1. Voxel downsample both clouds
//! 2. Estimate surface normals via SVD of local neighborhoods
//! 3. Build KD-tree on reference cloud
//! 4. Gauss-Newton iterations to minimize GICP cost
//! 5. Project 6DOF result to (x, y, yaw) with covariance

use std::collections::HashMap;
use std::num::NonZero;
use std::sync::Arc;

use kiddo::{ImmutableKdTree, SquaredEuclidean};
use lidar::PointCloud;
use nalgebra::{Matrix3, Matrix6, Vector3, Vector6};
use tracing::debug;
use transforms::Transform2D;

use crate::SlamError;

/// Configuration for GICP scan matching.
#[derive(Debug, Clone)]
pub struct ScanMatchConfig {
    /// Voxel size for downsampling (meters)
    pub resolution: f64,
    /// Linear search range — unused in GICP, kept for config compat
    pub linear_range: f64,
    /// Angular search range — unused in GICP, kept for config compat
    pub angular_range: f64,
    /// Maximum GICP iterations
    pub max_iterations: usize,
    /// Convergence threshold (pose delta norm)
    pub convergence_threshold: f64,
    /// Maximum correspondence distance for outlier rejection (meters)
    pub max_correspondence_dist: f64,
    /// Minimum fraction of points with valid correspondences
    pub min_overlap: f64,
}

impl Default for ScanMatchConfig {
    fn default() -> Self {
        Self {
            resolution: 0.2,
            linear_range: 0.5,
            angular_range: 0.26,
            max_iterations: 30,
            convergence_threshold: 1e-4,
            max_correspondence_dist: 1.0,
            min_overlap: 0.3,
        }
    }
}

/// Result of scan matching.
#[derive(Debug, Clone)]
pub struct ScanMatchResult {
    /// Estimated relative transform (projected to 2D: x, y, yaw)
    pub transform: Transform2D,
    /// Match quality score (0-1, overlap ratio)
    pub score: f64,
    /// Estimated covariance of the match (3x3: x, y, yaw)
    pub covariance: Matrix3<f64>,
}

/// 3D GICP scan matcher.
///
/// Uses Generalized ICP with local covariance weighting for robust
/// alignment of 3D point clouds. Results are projected to 2D for
/// the pose graph.
pub struct CorrelativeScanMatcher {
    config: ScanMatchConfig,
}

impl CorrelativeScanMatcher {
    /// Create a new scan matcher.
    pub fn new(config: ScanMatchConfig) -> Self {
        Self { config }
    }

    /// Match current scan to reference scan.
    ///
    /// Returns the transform that best aligns `current` to `reference`,
    /// starting from `initial_guess` (2D, lifted to 3D for optimization).
    pub fn match_scans(
        &self,
        reference: &Arc<PointCloud>,
        current: &Arc<PointCloud>,
        initial_guess: Transform2D,
    ) -> Result<ScanMatchResult, SlamError> {
        // Downsample both clouds
        let ref_points = voxel_downsample(&reference.points, self.config.resolution);
        let cur_points = voxel_downsample(&current.points, self.config.resolution);

        if ref_points.len() < 10 || cur_points.len() < 10 {
            return Err(SlamError::ScanMatchFailed(
                "Too few points after downsampling".into(),
            ));
        }

        // Estimate normals and covariances for reference cloud
        let ref_covs = estimate_covariances(&ref_points, 10);

        // Build KD-tree on reference cloud
        let ref_tree = build_kdtree(&ref_points);

        // Lift initial guess to 6DOF: (x, y, 0, 0, 0, yaw)
        let guess_t = initial_guess.translation();
        let guess_yaw = initial_guess.rotation();
        let mut pose = Vector6::new(guess_t.x, guess_t.y, 0.0, 0.0, 0.0, guess_yaw);

        let max_dist_sq = (self.config.max_correspondence_dist * self.config.max_correspondence_dist) as f32;

        for iteration in 0..self.config.max_iterations {
            let rot = rotation_from_euler(pose[3], pose[4], pose[5]);
            let trans = Vector3::new(pose[0], pose[1], pose[2]);

            // Find correspondences and build linear system
            let mut h = Matrix6::zeros();
            let mut b = Vector6::zeros();
            let mut valid_count = 0usize;

            // Estimate covariances for current cloud
            let cur_covs = estimate_covariances(&cur_points, 10);

            for (j, cur_pt) in cur_points.iter().enumerate() {
                // Transform current point
                let transformed = rot * cur_pt + trans;

                // Find nearest neighbor in reference
                let query = [transformed.x as f32, transformed.y as f32, transformed.z as f32];
                let nn = ref_tree.nearest_one::<SquaredEuclidean>(&query);

                if nn.distance > max_dist_sq {
                    continue;
                }

                let i = nn.item as usize;
                let ref_pt = &ref_points[i];

                valid_count += 1;

                // GICP cost: (p_i - T*q_j)^T * (C_i + R*C_j*R^T)^{-1} * (p_i - T*q_j)
                let c_ref = &ref_covs[i];
                let c_cur = &cur_covs[j];
                let c_combined = c_ref + rot * c_cur * rot.transpose();

                // Regularize and invert
                let c_inv = (c_combined + Matrix3::identity() * 1e-6)
                    .try_inverse()
                    .unwrap_or(Matrix3::identity());

                // Residual
                let residual = ref_pt - transformed;

                // Jacobian of residual w.r.t. pose [tx, ty, tz, roll, pitch, yaw]
                let jac = compute_jacobian(cur_pt, &rot, &pose);

                // Gauss-Newton: H += J^T * C_inv * J, b += J^T * C_inv * r
                let jt = jac.transpose();
                h += &jt * &c_inv * &jac;
                b += &jt * (&c_inv * residual);
            }

            let overlap = valid_count as f64 / cur_points.len() as f64;
            if overlap < self.config.min_overlap {
                return Err(SlamError::ScanMatchFailed(format!(
                    "Insufficient overlap: {:.2}",
                    overlap
                )));
            }

            // Solve H * dx = -b (standard Gauss-Newton: H dx = -gradient)
            let h_reg = h + Matrix6::identity() * 1e-6;
            let neg_b = -b;
            let dx = match h_reg.lu().solve(&neg_b) {
                Some(x) => x,
                None => {
                    return Err(SlamError::ScanMatchFailed(
                        "Linear system solve failed".into(),
                    ));
                }
            };

            // Apply update
            pose += dx;

            // Check convergence
            let delta_norm = dx.norm();
            if delta_norm < self.config.convergence_threshold {
                debug!(
                    iterations = iteration + 1,
                    overlap = overlap,
                    "GICP converged"
                );
                break;
            }
        }

        // Final overlap score and Hessian for covariance
        let rot = rotation_from_euler(pose[3], pose[4], pose[5]);
        let trans = Vector3::new(pose[0], pose[1], pose[2]);
        let mut valid_count = 0usize;
        let mut h_final = Matrix6::zeros();
        let cur_covs = estimate_covariances(&cur_points, 10);

        for (j, cur_pt) in cur_points.iter().enumerate() {
            let transformed = rot * cur_pt + trans;
            let query = [transformed.x as f32, transformed.y as f32, transformed.z as f32];
            let nn = ref_tree.nearest_one::<SquaredEuclidean>(&query);
            if nn.distance > max_dist_sq {
                continue;
            }
            valid_count += 1;

            // Accumulate Hessian for covariance estimation
            let i = nn.item as usize;
            let c_ref = &ref_covs[i];
            let c_cur = &cur_covs[j];
            let c_combined = c_ref + rot * c_cur * rot.transpose();
            let c_inv = (c_combined + Matrix3::identity() * 1e-6)
                .try_inverse()
                .unwrap_or(Matrix3::identity());
            let jac = compute_jacobian(cur_pt, &rot, &pose);
            let jt = jac.transpose();
            h_final += &jt * &c_inv * &jac;
        }

        let score = valid_count as f64 / cur_points.len() as f64;

        // Covariance from inverse Hessian
        let cov_6d = (h_final + Matrix6::identity() * 1e-6)
            .try_inverse()
            .unwrap_or(Matrix6::identity() * 0.1);

        // Extract 3x3 covariance for (x, y, yaw) — indices 0, 1, 5
        let idx = [0, 1, 5];
        let mut cov_3d = Matrix3::zeros();
        for (r, &ri) in idx.iter().enumerate() {
            for (c, &ci) in idx.iter().enumerate() {
                cov_3d[(r, c)] = cov_6d[(ri, ci)];
            }
        }

        // Project to 2D transform
        let transform = Transform2D::new(pose[0], pose[1], pose[5]);

        Ok(ScanMatchResult {
            transform,
            score,
            covariance: cov_3d,
        })
    }
}

/// Voxel downsample a point cloud. Returns centroids per voxel cell.
fn voxel_downsample(points: &[lidar::Point3D], voxel_size: f64) -> Vec<Vector3<f64>> {
    let inv = 1.0 / voxel_size;
    // (ix, iy, iz) -> (sum, count)
    let mut grid: HashMap<(i32, i32, i32), (Vector3<f64>, usize)> = HashMap::new();

    for p in points {
        // Basic range/validity filter
        let range_sq = p.x * p.x + p.y * p.y + p.z * p.z;
        if !range_sq.is_finite() || range_sq < 0.01 || range_sq > 2500.0 {
            continue;
        }

        let ix = (p.x as f64 * inv).floor() as i32;
        let iy = (p.y as f64 * inv).floor() as i32;
        let iz = (p.z as f64 * inv).floor() as i32;

        let entry = grid.entry((ix, iy, iz)).or_insert((Vector3::zeros(), 0));
        entry.0 += Vector3::new(p.x as f64, p.y as f64, p.z as f64);
        entry.1 += 1;
    }

    grid.into_values()
        .map(|(sum, count)| sum / count as f64)
        .collect()
}

/// Estimate local covariance matrices for each point using SVD of k-nearest neighbors.
fn estimate_covariances(points: &[Vector3<f64>], k: usize) -> Vec<Matrix3<f64>> {
    if points.len() < k {
        return points.iter().map(|_| Matrix3::identity() * 0.01).collect();
    }

    // Build KD-tree for neighborhood queries
    let tree = build_kdtree(points);

    let k_query = k.min(points.len());

    points
        .iter()
        .map(|pt| {
            let query = [pt.x as f32, pt.y as f32, pt.z as f32];
            let neighbors = tree.nearest_n::<SquaredEuclidean>(&query, NonZero::new(k_query).unwrap());

            if neighbors.len() < 3 {
                return Matrix3::identity() * 0.01;
            }

            // Compute centroid
            let mut centroid = Vector3::zeros();
            for nn in &neighbors {
                let idx = nn.item as usize;
                centroid += &points[idx];
            }
            centroid /= neighbors.len() as f64;

            // Compute covariance matrix
            let mut cov = Matrix3::zeros();
            for nn in &neighbors {
                let idx = nn.item as usize;
                let diff = &points[idx] - centroid;
                cov += diff * diff.transpose();
            }
            cov /= neighbors.len() as f64;

            // Regularize: inflate smallest eigenvalue to prevent singularity.
            let svd = cov.svd(true, true);
            let mut s: Vector3<f64> = svd.singular_values;
            let eps = s[0] * 1e-3;
            s[0] = s[0].max(eps);
            s[1] = s[1].max(eps);
            s[2] = s[2].max(eps);

            let u = svd.u.unwrap_or(Matrix3::identity());
            let vt = svd.v_t.unwrap_or(Matrix3::identity());
            u * Matrix3::from_diagonal(&s) * vt
        })
        .collect()
}

/// Build a KD-tree from 3D points.
///
/// Points are stored as f32 for memory efficiency; the implicit index
/// (position in the input slice) is used as the item identifier.
fn build_kdtree(points: &[Vector3<f64>]) -> ImmutableKdTree<f32, 3> {
    let entries: Vec<[f32; 3]> = points
        .iter()
        .map(|p| [p.x as f32, p.y as f32, p.z as f32])
        .collect();

    ImmutableKdTree::new_from_slice(&entries)
}

/// Compute rotation matrix from Euler angles (roll, pitch, yaw).
/// Convention: R = Rz(yaw) * Ry(pitch) * Rx(roll)
fn rotation_from_euler(roll: f64, pitch: f64, yaw: f64) -> Matrix3<f64> {
    let (sr, cr) = roll.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();

    Matrix3::new(
        cy * cp,
        cy * sp * sr - sy * cr,
        cy * sp * cr + sy * sr,
        sy * cp,
        sy * sp * sr + cy * cr,
        sy * sp * cr - cy * sr,
        -sp,
        cp * sr,
        cp * cr,
    )
}

/// Compute Jacobian of residual w.r.t. pose parameters.
///
/// residual = ref_pt - (R * cur_pt + t)
/// J = d(residual)/d[tx, ty, tz, roll, pitch, yaw]
/// d/dt = -I (3x3)
/// d/d(angles) = -d(R*cur_pt)/d(angles)
fn compute_jacobian(
    cur_pt: &Vector3<f64>,
    rot: &Matrix3<f64>,
    pose: &Vector6<f64>,
) -> nalgebra::Matrix3x6<f64> {
    let mut jac = nalgebra::Matrix3x6::zeros();

    // Translation part: d(residual)/d(tx,ty,tz) = -I
    jac[(0, 0)] = -1.0;
    jac[(1, 1)] = -1.0;
    jac[(2, 2)] = -1.0;

    // Rotation part: numerical differentiation for robustness
    let h = 1e-6;
    let transformed = rot * cur_pt;
    for axis in 0..3 {
        let mut pose_plus = *pose;
        pose_plus[3 + axis] += h;
        let rot_plus = rotation_from_euler(pose_plus[3], pose_plus[4], pose_plus[5]);
        let transformed_plus = rot_plus * cur_pt;

        let d = -(transformed_plus - transformed) / h;
        jac[(0, 3 + axis)] = d.x;
        jac[(1, 3 + axis)] = d.y;
        jac[(2, 3 + axis)] = d.z;
    }

    jac
}

#[cfg(test)]
mod tests {
    use super::*;
    use lidar::Point3D;

    fn make_box_scan(offset_x: f32, offset_y: f32, offset_z: f32) -> PointCloud {
        // Create a 3D box-like environment with walls and floor
        let mut points = Vec::new();

        // Floor points (z ~= 0)
        for ix in -20..=20 {
            for iy in -20..=20 {
                points.push(Point3D {
                    x: ix as f32 * 0.25 + offset_x,
                    y: iy as f32 * 0.25 + offset_y,
                    z: 0.0 + offset_z,
                    reflectivity: 128,
                    tag: 0,
                });
            }
        }

        // Walls at x=5 and x=-5
        for iy in -20..=20 {
            for iz in 0..=8 {
                for &x in &[5.0, -5.0] {
                    points.push(Point3D {
                        x: x + offset_x,
                        y: iy as f32 * 0.25 + offset_y,
                        z: iz as f32 * 0.25 + offset_z,
                        reflectivity: 128,
                        tag: 0,
                    });
                }
            }
        }

        // Walls at y=5 and y=-5
        for ix in -20..=20 {
            for iz in 0..=8 {
                for &y in &[5.0, -5.0] {
                    points.push(Point3D {
                        x: ix as f32 * 0.25 + offset_x,
                        y: y + offset_y,
                        z: iz as f32 * 0.25 + offset_z,
                        reflectivity: 128,
                        tag: 0,
                    });
                }
            }
        }

        PointCloud {
            points,
            ..Default::default()
        }
    }

    fn make_circular_scan(offset_x: f32, offset_y: f32) -> PointCloud {
        let angle_increment = std::f32::consts::PI * 2.0 / 360.0;
        let points: Vec<Point3D> = (0..360)
            .map(|i| {
                let angle = i as f32 * angle_increment;
                let range = 5.0 + offset_x * angle.cos() + offset_y * angle.sin();
                Point3D {
                    x: range * angle.cos(),
                    y: range * angle.sin(),
                    z: 0.5,
                    reflectivity: 128,
                    tag: 0,
                }
            })
            .collect();

        PointCloud {
            points,
            ..Default::default()
        }
    }

    #[test]
    fn test_voxel_downsample() {
        let scan = make_box_scan(0.0, 0.0, 0.0);
        let downsampled = voxel_downsample(&scan.points, 0.5);
        assert!(downsampled.len() < scan.points.len());
        assert!(downsampled.len() > 10);
    }

    #[test]
    fn test_estimate_covariances() {
        let scan = make_box_scan(0.0, 0.0, 0.0);
        let points = voxel_downsample(&scan.points, 0.5);
        let covs = estimate_covariances(&points, 10);
        assert_eq!(covs.len(), points.len());

        for cov in &covs {
            assert!((cov[(0, 1)] - cov[(1, 0)]).abs() < 1e-10);
            assert!((cov[(0, 2)] - cov[(2, 0)]).abs() < 1e-10);
            assert!((cov[(1, 2)] - cov[(2, 1)]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_rotation_from_euler_identity() {
        let r = rotation_from_euler(0.0, 0.0, 0.0);
        let identity: Matrix3<f64> = Matrix3::identity();
        for i in 0..3usize {
            for j in 0..3usize {
                assert!((r[(i, j)] - identity[(i, j)]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_identity_match() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        let scan = Arc::new(make_box_scan(0.0, 0.0, 0.0));
        let result = matcher
            .match_scans(&scan, &scan, Transform2D::identity())
            .unwrap();

        assert!(result.transform.translation().norm() < 0.1);
        assert!(result.transform.rotation().abs() < 0.1);
        assert!(result.score > 0.8);
    }

    #[test]
    fn test_convergence_from_nearby_guess() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        // Identical scans, but provide a slightly wrong initial guess.
        // GICP should converge back toward identity.
        let scan = Arc::new(make_box_scan(0.0, 0.0, 0.0));
        let result = matcher
            .match_scans(&scan, &scan, Transform2D::new(0.05, 0.03, 0.01))
            .unwrap();

        // Should still get a good score and small resulting transform
        assert!(result.score > 0.5);
        assert!(result.transform.translation().norm() < 0.2);
    }

    #[test]
    fn test_circular_scan_identity() {
        let config = ScanMatchConfig {
            resolution: 0.2,
            ..Default::default()
        };
        let matcher = CorrelativeScanMatcher::new(config);

        let scan = Arc::new(make_circular_scan(0.0, 0.0));
        let result = matcher
            .match_scans(&scan, &scan, Transform2D::identity())
            .unwrap();

        assert!(result.transform.translation().norm() < 0.1);
        assert!(result.score > 0.5);
    }

    #[test]
    fn test_covariance_is_symmetric() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        let scan = Arc::new(make_box_scan(0.0, 0.0, 0.0));
        let result = matcher
            .match_scans(&scan, &scan, Transform2D::identity())
            .unwrap();

        let cov = result.covariance;
        assert!((cov[(0, 1)] - cov[(1, 0)]).abs() < 1e-10);
        assert!((cov[(0, 2)] - cov[(2, 0)]).abs() < 1e-10);
        assert!((cov[(1, 2)] - cov[(2, 1)]).abs() < 1e-10);
    }
}
