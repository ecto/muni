//! Common test utilities for autonomy stack integration tests

use nalgebra::{Isometry2, Vector2};

/// Generate a synthetic LiDAR scan for a box-shaped room
///
/// Returns scan ranges and angles that represent a robot at the center
/// of a rectangular room.
pub fn generate_box_room_scan(
    width: f64,
    height: f64,
    num_points: usize,
) -> Vec<(f64, f64)> {
    let mut scan = Vec::new();
    let angle_increment = 2.0 * std::f64::consts::PI / num_points as f64;

    for i in 0..num_points {
        let angle = i as f64 * angle_increment;

        // Compute intersection with box walls
        let range = compute_box_intersection(width, height, angle);

        scan.push((range, angle));
    }

    scan
}

/// Compute ray-box intersection distance
fn compute_box_intersection(width: f64, height: f64, angle: f64) -> f64 {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Check intersection with each wall, return minimum
    let mut min_dist = f64::INFINITY;

    // Right wall (x = half_width)
    if cos_a > 1e-6 {
        let t = half_width / cos_a;
        let y = t * sin_a;
        if y.abs() <= half_height && t > 0.0 {
            min_dist = min_dist.min(t);
        }
    }

    // Left wall (x = -half_width)
    if cos_a < -1e-6 {
        let t = -half_width / cos_a;
        let y = t * sin_a;
        if y.abs() <= half_height && t > 0.0 {
            min_dist = min_dist.min(t);
        }
    }

    // Top wall (y = half_height)
    if sin_a > 1e-6 {
        let t = half_height / sin_a;
        let x = t * cos_a;
        if x.abs() <= half_width && t > 0.0 {
            min_dist = min_dist.min(t);
        }
    }

    // Bottom wall (y = -half_height)
    if sin_a < -1e-6 {
        let t = -half_height / sin_a;
        let x = t * cos_a;
        if x.abs() <= half_width && t > 0.0 {
            min_dist = min_dist.min(t);
        }
    }

    min_dist
}

/// Simulate differential drive kinematics for one timestep
///
/// Returns the new pose after applying velocities for duration dt.
pub fn simulate_differential_drive(
    current_pose: Isometry2<f64>,
    left_vel: f64,
    right_vel: f64,
    wheel_radius: f64,
    track_width: f64,
    dt: f64,
) -> Isometry2<f64> {
    // Compute body velocities
    let v = (left_vel + right_vel) / 2.0 * wheel_radius;
    let omega = (right_vel - left_vel) / track_width * wheel_radius;

    // Compute motion in robot frame
    let delta = if omega.abs() < 1e-6 {
        // Straight line
        Vector2::new(v * dt, 0.0)
    } else {
        // Arc
        let r = v / omega;
        let dtheta = omega * dt;
        Vector2::new(r * dtheta.sin(), r * (1.0 - dtheta.cos()))
    };

    let dtheta = omega * dt;

    // Apply to current pose
    let delta_world = current_pose.rotation * delta;
    Isometry2::new(
        current_pose.translation.vector + delta_world,
        current_pose.rotation.angle() + dtheta,
    )
}

/// Add Gaussian noise to a pose
pub fn add_pose_noise(
    pose: Isometry2<f64>,
    xy_stddev: f64,
    theta_stddev: f64,
) -> Isometry2<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let noisy_x = pose.translation.x + rng.gen::<f64>() * xy_stddev;
    let noisy_y = pose.translation.y + rng.gen::<f64>() * xy_stddev;
    let noisy_theta = pose.rotation.angle() + rng.gen::<f64>() * theta_stddev;

    Isometry2::new(Vector2::new(noisy_x, noisy_y), noisy_theta)
}

/// Assert two poses are approximately equal
#[macro_export]
macro_rules! assert_pose_approx_eq {
    ($pose1:expr, $pose2:expr, $tolerance:expr) => {
        let p1 = $pose1;
        let p2 = $pose2;
        let dx = (p1.translation.x - p2.translation.x).abs();
        let dy = (p1.translation.y - p2.translation.y).abs();
        let dtheta = (p1.rotation.angle() - p2.rotation.angle()).abs();

        assert!(
            dx < $tolerance,
            "X position error: {} vs {}, diff = {}",
            p1.translation.x,
            p2.translation.x,
            dx
        );
        assert!(
            dy < $tolerance,
            "Y position error: {} vs {}, diff = {}",
            p1.translation.y,
            p2.translation.y,
            dy
        );
        assert!(
            dtheta < $tolerance,
            "Heading error: {} vs {}, diff = {}",
            p1.rotation.angle(),
            p2.rotation.angle(),
            dtheta
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_room_scan_generation() {
        let scan = generate_box_room_scan(10.0, 10.0, 360);
        assert_eq!(scan.len(), 360);

        // Check that point directly ahead hits front wall at 5m
        let front_point = scan[0];
        assert!((front_point.0 - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_differential_drive_simulation() {
        let pose = Isometry2::identity();

        // Drive straight
        let new_pose = simulate_differential_drive(
            pose,
            10.0, // left wheel
            10.0, // right wheel
            0.1,  // wheel radius
            0.5,  // track width
            0.1,  // dt
        );

        // Should move 0.1m forward (v = 10*0.1 = 1 m/s, distance = 1*0.1 = 0.1m)
        assert!((new_pose.translation.x - 0.1).abs() < 0.01);
        assert!(new_pose.translation.y.abs() < 0.01);
        assert!(new_pose.rotation.angle().abs() < 0.01);
    }
}
