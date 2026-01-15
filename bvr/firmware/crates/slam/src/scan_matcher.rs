//! Correlative scan matching for 2D LiDAR.
//!
//! Uses a grid-based correlation approach that is more robust to
//! initialization errors than ICP-based methods.

use lidar::LaserScan;
use nalgebra::{Matrix3, Vector2};
use std::sync::Arc;
use transforms::Transform2D;

use crate::SlamError;

/// Configuration for scan matching.
#[derive(Debug, Clone)]
pub struct ScanMatchConfig {
    /// Grid resolution for correlation (meters)
    pub resolution: f64,
    /// Linear search range (meters)
    pub linear_range: f64,
    /// Angular search range (radians)
    pub angular_range: f64,
    /// Angular resolution (radians)
    pub angular_resolution: f64,
}

impl Default for ScanMatchConfig {
    fn default() -> Self {
        Self {
            resolution: 0.05,
            linear_range: 0.5,
            angular_range: 0.26, // ~15 degrees
            angular_resolution: 0.02, // ~1 degree
        }
    }
}

/// Result of scan matching.
#[derive(Debug, Clone)]
pub struct ScanMatchResult {
    /// Estimated relative transform
    pub transform: Transform2D,
    /// Match quality score (0-1, higher is better)
    pub score: f64,
    /// Estimated covariance of the match
    pub covariance: Matrix3<f64>,
}

/// Correlative scan matcher.
///
/// Searches a grid of possible transforms to find the best alignment
/// between two laser scans using a lookup table approach.
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
    /// starting from `initial_guess`.
    pub fn match_scans(
        &self,
        reference: &Arc<LaserScan>,
        current: &Arc<LaserScan>,
        initial_guess: Transform2D,
    ) -> Result<ScanMatchResult, SlamError> {
        // Build lookup table from reference scan
        let lookup = self.build_lookup_table(reference);

        // Coarse search: grid search over pose space
        let coarse_result = self.coarse_search(current, &lookup, &initial_guess);

        // Fine search: gradient descent refinement
        let fine_result = self.fine_search(current, &lookup, &coarse_result.0);

        // Estimate covariance from Hessian
        let covariance = self.estimate_covariance(current, &lookup, &fine_result.0);

        Ok(ScanMatchResult {
            transform: fine_result.0,
            score: fine_result.1,
            covariance,
        })
    }

    /// Build a lookup table (precomputed map) from a laser scan.
    fn build_lookup_table(&self, scan: &LaserScan) -> LookupTable {
        // Convert scan to points
        let points: Vec<Vector2<f64>> = self.scan_to_points(scan, &Transform2D::identity());

        // Determine bounds
        let (min_x, max_x, min_y, max_y) = self.compute_bounds(&points);

        // Add margin for search
        let margin = self.config.linear_range + 2.0;
        let min_x = min_x - margin;
        let max_x = max_x + margin;
        let min_y = min_y - margin;
        let max_y = max_y + margin;

        let width = ((max_x - min_x) / self.config.resolution).ceil() as usize + 1;
        let height = ((max_y - min_y) / self.config.resolution).ceil() as usize + 1;

        let mut table = LookupTable {
            data: vec![0.0; width * height],
            width,
            height,
            resolution: self.config.resolution,
            origin_x: min_x,
            origin_y: min_y,
        };

        // Mark occupied cells with Gaussian blur for robustness
        let sigma = self.config.resolution * 2.0;
        let kernel_size = (sigma / self.config.resolution * 3.0).ceil() as i32;

        for point in &points {
            let gx = ((point.x - min_x) / self.config.resolution) as i32;
            let gy = ((point.y - min_y) / self.config.resolution) as i32;

            // Apply Gaussian kernel
            for dy in -kernel_size..=kernel_size {
                for dx in -kernel_size..=kernel_size {
                    let x = gx + dx;
                    let y = gy + dy;

                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        let dist_sq = (dx as f64 * self.config.resolution).powi(2)
                            + (dy as f64 * self.config.resolution).powi(2);
                        let value = (-dist_sq / (2.0 * sigma * sigma)).exp();
                        let idx = y as usize * width + x as usize;
                        table.data[idx] = table.data[idx].max(value);
                    }
                }
            }
        }

        table
    }

    /// Coarse grid search over pose space.
    fn coarse_search(
        &self,
        scan: &LaserScan,
        lookup: &LookupTable,
        initial: &Transform2D,
    ) -> (Transform2D, f64) {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_pose = *initial;

        let x_steps = (self.config.linear_range / self.config.resolution).ceil() as i32;
        let y_steps = x_steps;
        let theta_steps = (self.config.angular_range / self.config.angular_resolution).ceil() as i32;

        let init_x = initial.translation().x;
        let init_y = initial.translation().y;
        let init_theta = initial.rotation();

        for xi in -x_steps..=x_steps {
            for yi in -y_steps..=y_steps {
                for ti in -theta_steps..=theta_steps {
                    let dx = xi as f64 * self.config.resolution;
                    let dy = yi as f64 * self.config.resolution;
                    let dtheta = ti as f64 * self.config.angular_resolution;

                    let test_pose = Transform2D::new(
                        init_x + dx,
                        init_y + dy,
                        init_theta + dtheta,
                    );

                    let score = self.score_pose(scan, lookup, &test_pose);
                    if score > best_score {
                        best_score = score;
                        best_pose = test_pose;
                    }
                }
            }
        }

        (best_pose, best_score)
    }

    /// Fine search using gradient descent.
    fn fine_search(
        &self,
        scan: &LaserScan,
        lookup: &LookupTable,
        initial: &Transform2D,
    ) -> (Transform2D, f64) {
        let mut current = *initial;
        let step_size = self.config.resolution / 4.0;
        let angular_step = self.config.angular_resolution / 4.0;

        for _ in 0..10 {
            let mut best_score = self.score_pose(scan, lookup, &current);
            let mut best_pose = current;

            // Try small perturbations
            for (dx, dy, dtheta) in [
                (step_size, 0.0, 0.0),
                (-step_size, 0.0, 0.0),
                (0.0, step_size, 0.0),
                (0.0, -step_size, 0.0),
                (0.0, 0.0, angular_step),
                (0.0, 0.0, -angular_step),
            ] {
                let test = Transform2D::new(
                    current.translation().x + dx,
                    current.translation().y + dy,
                    current.rotation() + dtheta,
                );
                let score = self.score_pose(scan, lookup, &test);
                if score > best_score {
                    best_score = score;
                    best_pose = test;
                }
            }

            if best_pose.translation() == current.translation()
                && (best_pose.rotation() - current.rotation()).abs() < 1e-10
            {
                // Converged
                break;
            }
            current = best_pose;
        }

        let final_score = self.score_pose(scan, lookup, &current);
        (current, final_score)
    }

    /// Score a pose by summing lookup table values at scan point locations.
    fn score_pose(&self, scan: &LaserScan, lookup: &LookupTable, pose: &Transform2D) -> f64 {
        let points = self.scan_to_points(scan, pose);
        if points.is_empty() {
            return 0.0;
        }

        let mut total = 0.0;
        let mut count = 0;

        for point in &points {
            if let Some(value) = lookup.get(point.x, point.y) {
                total += value;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        total / count as f64
    }

    /// Convert a laser scan to 2D points in a given frame.
    fn scan_to_points(&self, scan: &LaserScan, pose: &Transform2D) -> Vec<Vector2<f64>> {
        let mut points = Vec::with_capacity(scan.ranges.len());

        for (i, &range) in scan.ranges.iter().enumerate() {
            if !range.is_finite() || range < scan.range_min || range > scan.range_max {
                continue;
            }

            let angle = i as f32 * scan.angle_increment;
            let local_x = range * angle.cos();
            let local_y = range * angle.sin();

            let world_point = pose.transform_point(Vector2::new(local_x as f64, local_y as f64));
            points.push(world_point);
        }

        points
    }

    /// Compute bounds of a point set.
    fn compute_bounds(&self, points: &[Vector2<f64>]) -> (f64, f64, f64, f64) {
        if points.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for p in points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        (min_x, max_x, min_y, max_y)
    }

    /// Estimate covariance from the Hessian of the score function.
    fn estimate_covariance(
        &self,
        scan: &LaserScan,
        lookup: &LookupTable,
        pose: &Transform2D,
    ) -> Matrix3<f64> {
        let h = 1e-3;
        let _score_center = self.score_pose(scan, lookup, pose);

        let mut hessian = Matrix3::zeros();

        // Numerical second derivatives
        let perturbations = [
            (h, 0.0, 0.0),
            (0.0, h, 0.0),
            (0.0, 0.0, h),
        ];

        for (i, &(dx_i, dy_i, dt_i)) in perturbations.iter().enumerate() {
            for (j, &(dx_j, dy_j, dt_j)) in perturbations.iter().enumerate().skip(i) {
                let pp = Transform2D::new(
                    pose.translation().x + dx_i + dx_j,
                    pose.translation().y + dy_i + dy_j,
                    pose.rotation() + dt_i + dt_j,
                );
                let pm = Transform2D::new(
                    pose.translation().x + dx_i - dx_j,
                    pose.translation().y + dy_i - dy_j,
                    pose.rotation() + dt_i - dt_j,
                );
                let mp = Transform2D::new(
                    pose.translation().x - dx_i + dx_j,
                    pose.translation().y - dy_i + dy_j,
                    pose.rotation() - dt_i + dt_j,
                );
                let mm = Transform2D::new(
                    pose.translation().x - dx_i - dx_j,
                    pose.translation().y - dy_i - dy_j,
                    pose.rotation() - dt_i - dt_j,
                );

                let s_pp = self.score_pose(scan, lookup, &pp);
                let s_pm = self.score_pose(scan, lookup, &pm);
                let s_mp = self.score_pose(scan, lookup, &mp);
                let s_mm = self.score_pose(scan, lookup, &mm);

                let d2 = (s_pp - s_pm - s_mp + s_mm) / (4.0 * h * h);
                hessian[(i, j)] = -d2;
                hessian[(j, i)] = -d2;
            }
        }

        // Invert Hessian to get covariance, with regularization
        let regularized = hessian + Matrix3::identity() * 1e-6;
        regularized.try_inverse().unwrap_or(Matrix3::identity() * 0.1)
    }
}

/// Lookup table for fast scan correlation.
struct LookupTable {
    data: Vec<f64>,
    width: usize,
    height: usize,
    resolution: f64,
    origin_x: f64,
    origin_y: f64,
}

impl LookupTable {
    /// Get value at world coordinates.
    fn get(&self, x: f64, y: f64) -> Option<f64> {
        let gx = ((x - self.origin_x) / self.resolution) as i32;
        let gy = ((y - self.origin_y) / self.resolution) as i32;

        if gx >= 0 && gx < self.width as i32 && gy >= 0 && gy < self.height as i32 {
            Some(self.data[gy as usize * self.width + gx as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_scan(offset_x: f32, offset_y: f32) -> LaserScan {
        let mut scan = LaserScan::default();
        scan.angle_increment = std::f32::consts::PI * 2.0 / 360.0;
        scan.ranges = (0..360)
            .map(|i| {
                let angle = i as f32 * scan.angle_increment;
                // Simple circular wall at 5m radius
                5.0 + offset_x * angle.cos() + offset_y * angle.sin()
            })
            .collect();
        scan.intensities = vec![128; 360];
        scan
    }

    #[test]
    fn test_scan_to_points() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        let scan = make_test_scan(0.0, 0.0);
        let points = matcher.scan_to_points(&scan, &Transform2D::identity());

        assert!(!points.is_empty());
        // All points should be roughly 5m from origin
        for p in &points {
            let dist = p.norm();
            assert!((dist - 5.0).abs() < 0.5);
        }
    }

    #[test]
    fn test_identity_match() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        let scan = Arc::new(make_test_scan(0.0, 0.0));
        let result = matcher.match_scans(&scan, &scan, Transform2D::identity()).unwrap();

        // Matching identical scans should give identity transform
        assert!(result.transform.translation().norm() < 0.1);
        assert!(result.transform.rotation().abs() < 0.1);
        assert!(result.score > 0.8); // Should be a good match
    }

    #[test]
    fn test_small_translation_match() {
        let config = ScanMatchConfig::default();
        let matcher = CorrelativeScanMatcher::new(config);

        let reference = Arc::new(make_test_scan(0.0, 0.0));
        let current = Arc::new(make_test_scan(0.2, 0.0)); // Small offset in scan

        let result = matcher
            .match_scans(&reference, &current, Transform2D::identity())
            .unwrap();

        // Should find a small transform
        assert!(result.score > 0.5);
    }
}
