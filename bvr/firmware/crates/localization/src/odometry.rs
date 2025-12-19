//! Wheel odometry from VESC tachometer readings.
//!
//! Converts tachometer counts from VESCs into robot displacement.

use control::ChassisParams;
use std::f64::consts::PI;
use tracing::trace;

/// Wheel odometry calculator.
///
/// Uses differential drive kinematics to compute robot displacement
/// from wheel encoder (tachometer) readings.
pub struct WheelOdometry {
    /// Chassis geometry
    chassis: ChassisParams,
    /// Motor pole pairs (for ERPM to mechanical conversion)
    pole_pairs: u8,
    /// Previous tachometer readings [FL, FR, RL, RR]
    last_tach: Option<[i32; 4]>,
    /// Accumulated displacement since last reset
    total_distance: f64,
}

impl WheelOdometry {
    /// Create a new wheel odometry calculator.
    pub fn new(chassis: ChassisParams, pole_pairs: u8) -> Self {
        Self {
            chassis,
            pole_pairs,
            last_tach: None,
            total_distance: 0.0,
        }
    }

    /// Update with new tachometer readings from VESCs.
    ///
    /// Returns the displacement in robot frame: (dx, dy, dtheta)
    /// - dx: forward displacement (meters)
    /// - dy: lateral displacement (meters, always 0 for differential drive)
    /// - dtheta: rotation (radians, positive = counter-clockwise)
    ///
    /// The tachometer values are cumulative ERPM counts.
    pub fn update(&mut self, tach: [i32; 4]) -> (f64, f64, f64) {
        // First reading - just store it
        let Some(last) = self.last_tach else {
            self.last_tach = Some(tach);
            return (0.0, 0.0, 0.0);
        };

        // Compute deltas
        let delta: [i32; 4] = [
            tach[0].wrapping_sub(last[0]),
            tach[1].wrapping_sub(last[1]),
            tach[2].wrapping_sub(last[2]),
            tach[3].wrapping_sub(last[3]),
        ];

        self.last_tach = Some(tach);

        // Convert tachometer counts to wheel revolutions
        // VESC tachometer counts 6 steps per electrical revolution
        // Mechanical revolutions = electrical revolutions / pole_pairs
        let tach_to_revs = 1.0 / (6.0 * self.pole_pairs as f64);

        let revs: [f64; 4] = [
            delta[0] as f64 * tach_to_revs,
            delta[1] as f64 * tach_to_revs,
            delta[2] as f64 * tach_to_revs,
            delta[3] as f64 * tach_to_revs,
        ];

        // Convert revolutions to distance traveled
        let wheel_circumference = 2.0 * PI * self.chassis.wheel_radius;
        let distances: [f64; 4] = [
            revs[0] * wheel_circumference,
            revs[1] * wheel_circumference,
            revs[2] * wheel_circumference,
            revs[3] * wheel_circumference,
        ];

        // For skid-steer, average left and right sides
        // Left side: FL (0) and RL (2)
        // Right side: FR (1) and RR (3)
        let left_dist = (distances[0] + distances[2]) / 2.0;
        let right_dist = (distances[1] + distances[3]) / 2.0;

        // Differential drive kinematics
        // Linear displacement is average of left and right
        let dx = (left_dist + right_dist) / 2.0;

        // Angular displacement from difference between sides
        // dtheta = (right - left) / track_width
        let dtheta = (right_dist - left_dist) / self.chassis.track_width;

        // No lateral motion in differential drive
        let dy = 0.0;

        // Track total distance
        self.total_distance += dx.abs();

        trace!(
            delta = ?delta,
            dx = dx,
            dtheta = dtheta,
            "Odometry update"
        );

        (dx, dy, dtheta)
    }

    /// Get total distance traveled since creation/reset.
    pub fn total_distance(&self) -> f64 {
        self.total_distance
    }

    /// Reset odometry state.
    pub fn reset(&mut self) {
        self.last_tach = None;
        self.total_distance = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chassis() -> ChassisParams {
        // 165mm wheel diameter, 550mm track width
        ChassisParams::new(0.165, 0.55, 0.55)
    }

    #[test]
    fn test_first_reading() {
        let mut odom = WheelOdometry::new(test_chassis(), 15);

        // First reading should return zero displacement
        let (dx, dy, dtheta) = odom.update([1000, 1000, 1000, 1000]);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
        assert_eq!(dtheta, 0.0);
    }

    #[test]
    fn test_forward_motion() {
        let mut odom = WheelOdometry::new(test_chassis(), 15);

        // Initialize
        odom.update([0, 0, 0, 0]);

        // All wheels move forward equally
        // 90 tach counts = 1 mechanical revolution (6 * 15 pole pairs)
        let counts_per_rev = 6 * 15;
        let (dx, dy, dtheta) = odom.update([counts_per_rev, counts_per_rev, counts_per_rev, counts_per_rev]);

        // One revolution = wheel circumference
        let expected_dist = std::f64::consts::PI * 0.165; // ~0.518m
        assert!((dx - expected_dist).abs() < 0.001);
        assert_eq!(dy, 0.0);
        assert!(dtheta.abs() < 0.001); // No rotation
    }

    #[test]
    fn test_rotation() {
        let mut odom = WheelOdometry::new(test_chassis(), 15);

        // Initialize
        odom.update([0, 0, 0, 0]);

        // Right wheels forward, left wheels backward = rotation
        let counts = 90; // 1 revolution
        let (dx, _dy, dtheta) = odom.update([-counts, counts, -counts, counts]);

        // Forward motion should be ~0 (opposite sides cancel)
        assert!(dx.abs() < 0.001);

        // Should rotate (positive = CCW)
        assert!(dtheta > 0.0);
    }

    #[test]
    fn test_total_distance() {
        let mut odom = WheelOdometry::new(test_chassis(), 15);

        odom.update([0, 0, 0, 0]);
        odom.update([90, 90, 90, 90]);
        odom.update([180, 180, 180, 180]);

        assert!(odom.total_distance() > 0.0);
    }
}



