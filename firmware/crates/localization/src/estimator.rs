//! Pose estimator that fuses wheel odometry with GPS.
//!
//! Uses a simple complementary filter approach:
//! - Odometry provides high-frequency, low-drift short-term updates
//! - GPS provides low-frequency, absolute position corrections

use std::f64::consts::PI;
use tracing::{debug, trace};
use types::{GpsCoord, Pose};

/// Simple pose estimator with odometry + GPS fusion.
pub struct PoseEstimator {
    /// Current estimated pose in local frame
    pose: Pose,
    /// GPS origin (first valid GPS reading becomes the origin)
    gps_origin: Option<GpsCoord>,
    /// Weight for GPS corrections (0.0 = ignore GPS, 1.0 = snap to GPS)
    gps_weight: f64,
    /// Minimum GPS accuracy (meters) to accept a reading
    min_gps_accuracy: f32,
}

impl PoseEstimator {
    /// Create a new pose estimator.
    pub fn new() -> Self {
        Self {
            pose: Pose::default(),
            gps_origin: None,
            gps_weight: 0.1, // Blend 10% GPS, 90% odometry per update
            min_gps_accuracy: 10.0, // Accept readings with <10m accuracy
        }
    }

    /// Create with custom GPS weight.
    ///
    /// - `gps_weight`: How much to trust GPS vs odometry (0.0-1.0)
    ///   - 0.0: Pure odometry (GPS ignored)
    ///   - 0.1: Light GPS correction (default, good for most cases)
    ///   - 0.5: Equal weighting
    ///   - 1.0: Snap to GPS (ignores odometry)
    pub fn with_gps_weight(mut self, weight: f64) -> Self {
        self.gps_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set minimum GPS accuracy threshold.
    pub fn with_min_accuracy(mut self, accuracy: f32) -> Self {
        self.min_gps_accuracy = accuracy;
        self
    }

    /// Update with odometry displacement.
    ///
    /// This should be called at high frequency (e.g., 100Hz control loop).
    ///
    /// Arguments:
    /// - `dx`: Forward displacement in robot frame (meters)
    /// - `dy`: Lateral displacement in robot frame (meters)
    /// - `dtheta`: Rotation (radians, positive = CCW)
    pub fn update_odometry(&mut self, dx: f64, dy: f64, dtheta: f64) {
        // Transform displacement from robot frame to world frame
        let cos_theta = self.pose.theta.cos();
        let sin_theta = self.pose.theta.sin();

        // Rotate displacement vector by current heading
        let world_dx = dx * cos_theta - dy * sin_theta;
        let world_dy = dx * sin_theta + dy * cos_theta;

        // Update pose
        self.pose.x += world_dx;
        self.pose.y += world_dy;
        self.pose.theta += dtheta;

        // Normalize theta to [-PI, PI]
        self.pose.theta = normalize_angle(self.pose.theta);

        trace!(
            x = self.pose.x,
            y = self.pose.y,
            theta = self.pose.theta,
            "Odometry update"
        );
    }

    /// Update with a GPS reading.
    ///
    /// This is called when a new GPS fix is available (typically 1-10Hz).
    /// The estimator uses a complementary filter to blend GPS with odometry.
    pub fn update_gps(&mut self, coord: &GpsCoord) {
        // Reject low-accuracy readings
        if coord.accuracy > self.min_gps_accuracy || coord.accuracy <= 0.0 {
            trace!(accuracy = coord.accuracy, "GPS reading rejected (low accuracy)");
            return;
        }

        // Set origin on first valid reading
        if self.gps_origin.is_none() {
            self.gps_origin = Some(*coord);
            debug!(
                lat = coord.lat,
                lon = coord.lon,
                "GPS origin set"
            );
            return;
        }

        let origin = self.gps_origin.as_ref().unwrap();

        // Convert GPS to local coordinates (meters from origin)
        let (gps_x, gps_y) = gps_to_local(coord, origin);

        // Compute error between GPS and current estimate
        let error_x = gps_x - self.pose.x;
        let error_y = gps_y - self.pose.y;

        // Scale weight by accuracy (better accuracy = higher weight)
        let accuracy_factor = (self.min_gps_accuracy / coord.accuracy).min(2.0) as f64;
        let effective_weight = self.gps_weight * accuracy_factor;

        // Apply correction (complementary filter)
        self.pose.x += error_x * effective_weight;
        self.pose.y += error_y * effective_weight;

        debug!(
            gps_x = gps_x,
            gps_y = gps_y,
            error_x = error_x,
            error_y = error_y,
            weight = effective_weight,
            "GPS correction applied"
        );
    }

    /// Get the current estimated pose.
    pub fn pose(&self) -> Pose {
        self.pose
    }

    /// Get the GPS origin coordinates (if set).
    pub fn gps_origin(&self) -> Option<&GpsCoord> {
        self.gps_origin.as_ref()
    }

    /// Reset the estimator to origin.
    pub fn reset(&mut self) {
        self.pose = Pose::default();
        self.gps_origin = None;
    }

    /// Set the current pose (e.g., for manual correction).
    pub fn set_pose(&mut self, pose: Pose) {
        self.pose = pose;
    }

    /// Set the GPS origin manually.
    pub fn set_gps_origin(&mut self, coord: GpsCoord) {
        self.gps_origin = Some(coord);
    }
}

impl Default for PoseEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert GPS coordinates to local X/Y (meters from origin).
///
/// Uses equirectangular projection (accurate for small distances).
fn gps_to_local(coord: &GpsCoord, origin: &GpsCoord) -> (f64, f64) {
    // Earth radius in meters
    const EARTH_RADIUS: f64 = 6_371_000.0;

    let origin_lat_rad = origin.lat.to_radians();

    // Difference in coordinates
    let dlat = coord.lat - origin.lat;
    let dlon = coord.lon - origin.lon;

    // Convert to meters using equirectangular approximation
    // X is east-west (longitude), Y is north-south (latitude)
    let x = dlon.to_radians() * EARTH_RADIUS * origin_lat_rad.cos();
    let y = dlat.to_radians() * EARTH_RADIUS;

    // Note: Our coordinate system has X = forward, Y = left
    // GPS has X = east, Y = north
    // For now, assume robot starts facing north, so GPS Y -> robot X, GPS X -> robot -Y
    // This should be configurable in a real system
    (y, -x)
}

/// Normalize angle to [-PI, PI].
fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle;
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odometry_forward() {
        let mut est = PoseEstimator::new();

        // Move forward 1 meter (robot facing +X)
        est.update_odometry(1.0, 0.0, 0.0);

        let pose = est.pose();
        assert!((pose.x - 1.0).abs() < 0.001);
        assert!(pose.y.abs() < 0.001);
        assert!(pose.theta.abs() < 0.001);
    }

    #[test]
    fn test_odometry_rotation() {
        let mut est = PoseEstimator::new();

        // Rotate 90 degrees CCW
        est.update_odometry(0.0, 0.0, PI / 2.0);

        let pose = est.pose();
        assert!(pose.x.abs() < 0.001);
        assert!(pose.y.abs() < 0.001);
        assert!((pose.theta - PI / 2.0).abs() < 0.001);

        // Now move "forward" - should go in +Y direction
        est.update_odometry(1.0, 0.0, 0.0);

        let pose = est.pose();
        assert!(pose.x.abs() < 0.001);
        assert!((pose.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_angle_normalization() {
        assert!((normalize_angle(0.0) - 0.0).abs() < 0.001);
        assert!((normalize_angle(PI) - PI).abs() < 0.001);
        assert!((normalize_angle(-PI) - (-PI)).abs() < 0.001);
        assert!((normalize_angle(3.0 * PI) - PI).abs() < 0.001);
        assert!((normalize_angle(-3.0 * PI) - (-PI)).abs() < 0.001);
    }

    #[test]
    fn test_gps_origin_set() {
        let mut est = PoseEstimator::new();

        assert!(est.gps_origin().is_none());

        let coord = GpsCoord {
            lat: 37.7749,
            lon: -122.4194,
            alt: 10.0,
            accuracy: 5.0,
        };

        est.update_gps(&coord);

        assert!(est.gps_origin().is_some());
        let origin = est.gps_origin().unwrap();
        assert!((origin.lat - 37.7749).abs() < 0.0001);
    }

    #[test]
    fn test_gps_to_local() {
        let origin = GpsCoord {
            lat: 37.0,
            lon: -122.0,
            alt: 0.0,
            accuracy: 1.0,
        };

        // Same location should be (0, 0)
        let (x, y) = gps_to_local(&origin, &origin);
        assert!(x.abs() < 0.01);
        assert!(y.abs() < 0.01);

        // 1 degree north should be ~111km in Y
        let north = GpsCoord {
            lat: 38.0,
            lon: -122.0,
            alt: 0.0,
            accuracy: 1.0,
        };
        let (x, y) = gps_to_local(&north, &origin);
        assert!(x > 100_000.0); // Should be ~111km
        assert!(y.abs() < 1000.0); // Small east-west component
    }
}



