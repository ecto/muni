//! Integration tests for the autonomy stack
//!
//! Tests the full pipeline: odometry → EKF → scan matching → pose graph → planning
//!
//! Run with: cargo test --test autonomy_integration

use nalgebra::{Isometry2, Vector2};

mod common;

/// Test basic odometry integration over a straight line
#[test]
fn test_odometry_straight_line() {
    // Simulate 10 meters forward at 1 m/s
    let dt = 0.01; // 10ms
    let velocity = 1.0; // m/s
    let duration = 10.0; // seconds

    let mut pose = Isometry2::identity();
    let mut traveled = 0.0;

    for _ in 0..(duration / dt) as usize {
        // Simulate differential drive (equal wheel speeds = straight)
        let delta = Isometry2::new(Vector2::new(velocity * dt, 0.0), 0.0);
        pose = pose * delta;
        traveled += velocity * dt;
    }

    // Should be at (10, 0) with small numerical error
    assert!(
        (pose.translation.x - 10.0_f64).abs() < 0.01,
        "X position error: expected ~10.0, got {}",
        pose.translation.x
    );
    assert!(
        (pose.translation.y as f64).abs() < 0.01,
        "Y position error: expected ~0.0, got {}",
        pose.translation.y
    );
    assert!(
        (pose.rotation.angle() as f64).abs() < 0.01,
        "Heading error: expected ~0.0, got {}",
        pose.rotation.angle()
    );
}

/// Test arc motion (differential drive with different wheel speeds)
#[test]
fn test_odometry_arc_motion() {
    // Simulate a 90-degree left turn with 1m radius
    let wheel_radius = 0.1; // 10cm wheels
    let track_width = 0.5; // 50cm between wheels
    let dt = 0.01;

    // Right wheel faster than left → turn left
    let left_vel = 5.0; // rad/s
    let right_vel = 15.0; // rad/s

    let v = (left_vel + right_vel) / 2.0 * wheel_radius;
    let omega = (right_vel - left_vel) / track_width * wheel_radius;

    let mut pose = Isometry2::identity();

    // Turn for π/2 radians (90 degrees)
    let turn_duration = (std::f64::consts::FRAC_PI_2 / omega).abs();
    let steps = (turn_duration / dt) as usize;

    for _ in 0..steps {
        // Arc motion
        let r = v / omega;
        let dtheta = omega * dt;
        let dx = r * dtheta.sin();
        let dy = r * (1.0 - dtheta.cos());

        let delta = Isometry2::new(Vector2::new(dx, dy), dtheta);
        pose = pose * delta;
    }

    // After 90° left turn with 1m radius, should be at (1, 1) facing left (π/2)
    let expected_x = 1.0;
    let expected_y = 1.0;
    let expected_theta = std::f64::consts::FRAC_PI_2;

    assert!(
        (pose.translation.x - expected_x).abs() < 0.05,
        "X position error: expected ~{}, got {}",
        expected_x,
        pose.translation.x
    );
    assert!(
        (pose.translation.y - expected_y).abs() < 0.05,
        "Y position error: expected ~{}, got {}",
        expected_y,
        pose.translation.y
    );
    assert!(
        (pose.rotation.angle() - expected_theta).abs() < 0.05,
        "Heading error: expected ~{}, got {}",
        expected_theta,
        pose.rotation.angle()
    );
}

/// Test angle normalization edge cases
#[test]
fn test_angle_normalization() {
    use std::f64::consts::PI;

    fn normalize_angle(angle: f64) -> f64 {
        let mut a = angle % (2.0 * PI);
        if a > PI {
            a -= 2.0 * PI;
        } else if a < -PI {
            a += 2.0 * PI;
        }
        a
    }

    assert!((normalize_angle(0.0) - 0.0).abs() < 1e-10);
    assert!((normalize_angle(PI) - PI).abs() < 1e-10);
    assert!((normalize_angle(-PI) - (-PI)).abs() < 1e-10);
    assert!((normalize_angle(3.0 * PI) - PI).abs() < 1e-10);
    assert!((normalize_angle(-3.0 * PI) - (-PI)).abs() < 1e-10);
    assert!((normalize_angle(2.0 * PI) - 0.0).abs() < 1e-10);
    assert!((normalize_angle(5.0 * PI) - PI).abs() < 1e-10);
}

/// Test coordinate frame transforms
#[test]
fn test_coordinate_frame_chain() {
    // world ← odom ← base ← lidar

    let T_world_odom = Isometry2::new(Vector2::new(1.0, 2.0), 0.0); // Odom is at (1, 2) in world
    let T_odom_base = Isometry2::new(Vector2::new(0.5, 0.0), std::f64::consts::FRAC_PI_2); // Robot facing left
    let T_base_lidar = Isometry2::new(Vector2::new(0.15, 0.0), 0.0); // LiDAR 15cm ahead

    // Point 1m ahead of LiDAR
    let point_lidar = Vector2::new(1.0, 0.0);

    // Transform through chain
    let point_base = T_base_lidar * point_lidar;
    let point_odom = T_odom_base * point_base;
    let point_world = T_world_odom * point_odom;

    // LiDAR at (0.15, 0) in base, point at (1.15, 0) in base
    // Base rotated 90° left in odom, so (1.15, 0) → (0, 1.15) in odom
    // Odom offset by (1, 2) in world → (1, 3.15) in world

    assert!(
        (point_world.x - 1.0).abs() < 0.01,
        "X error: expected ~1.0, got {}",
        point_world.x
    );
    assert!(
        (point_world.y - 3.15).abs() < 0.01,
        "Y error: expected ~3.15, got {}",
        point_world.y
    );
}

/// Test that covariance stays positive definite after many updates
#[test]
fn test_covariance_stays_positive_definite() {
    use nalgebra::Matrix3;

    let mut covariance = Matrix3::identity() * 0.01;

    // Simulate 1000 odometry updates
    for _ in 0..1000 {
        // Simulate motion noise
        let v = 1.0; // m/s
        let _omega = 0.1; // rad/s
        let dt = 0.01; // 10ms

        // Jacobian (simplified)
        let g = Matrix3::new(1.0, 0.0, -v * dt * 0.0, 0.0, 1.0, v * dt * 1.0, 0.0, 0.0, 1.0);

        // Process noise
        let q = Matrix3::new(0.01, 0.0, 0.0, 0.0, 0.01, 0.0, 0.0, 0.0, 0.001);

        // Propagate covariance
        covariance = g * covariance * g.transpose() + q;

        // Check eigenvalues are positive
        let eigenvalues = covariance.symmetric_eigenvalues();
        for e in eigenvalues.iter() {
            assert!(
                *e > 0.0,
                "Negative eigenvalue detected: {}. Covariance:\n{}",
                e,
                covariance
            );
        }
    }

    // After 1000 updates, covariance should be larger but still reasonable
    let eigenvalues = covariance.symmetric_eigenvalues();
    let max_eigenvalue = eigenvalues.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max_eigenvalue < 100.0,
        "Covariance exploded: max eigenvalue = {}",
        max_eigenvalue
    );
}

/// Test simulated square loop for SLAM
///
/// Rover drives a 10m x 10m square. After completing the loop,
/// SLAM should detect loop closure and correct accumulated drift.
#[test]
#[ignore] // Requires SLAM crate to be implemented
fn test_slam_square_loop() {
    // TODO: Implement when slam crate is ready
    //
    // 1. Create synthetic LiDAR scans for 4 walls
    // 2. Simulate rover driving square:
    //    - 10m forward
    //    - Turn left 90°
    //    - 10m forward
    //    - Turn left 90°
    //    - 10m forward
    //    - Turn left 90°
    //    - 10m forward (back to start)
    // 3. Add 1% odometry drift per meter
    // 4. Run SLAM
    // 5. Verify loop closure detected
    // 6. Verify final pose error < 10cm

    todo!("Implement SLAM square loop test");
}

/// Test path planning with simple obstacle
#[test]
#[ignore] // Requires planner crate to be implemented
fn test_path_planning_around_obstacle() {
    // TODO: Implement when planner crate is ready
    //
    // 1. Create costmap with obstacle at (5, 0)
    // 2. Plan path from (0, 0) to (10, 0)
    // 3. Verify path goes around obstacle (not through it)
    // 4. Verify path is kinematically feasible (respects turn radius)

    todo!("Implement path planning test");
}

/// Test full autonomy pipeline (odometry → localization → planning → control)
#[test]
#[ignore] // Requires full autonomy stack to be implemented
fn test_full_autonomy_pipeline() {
    // TODO: Implement when autonomy orchestrator is ready
    //
    // 1. Create simulated environment (walls, obstacles)
    // 2. Set goal waypoint
    // 3. Run autonomy loop for 100 iterations
    // 4. Verify:
    //    - Localization converges
    //    - Path is generated
    //    - Trajectory tracking is stable
    //    - Goal is reached

    todo!("Implement full autonomy pipeline test");
}
