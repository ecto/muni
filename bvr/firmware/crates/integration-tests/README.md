# Autonomy Stack Integration Tests

This directory contains integration tests for the BVR firmware autonomy stack.

## Overview

While unit tests live alongside the code in each crate (`#[cfg(test)]` modules), integration tests here validate the full pipeline from odometry through localization to planning and control.

**Test Categories:**
1. **Odometry Tests** - Differential drive kinematics, arc motion, covariance propagation
2. **Localization Tests** - Coordinate frames, EKF sensor fusion, scan matching
3. **SLAM Tests** - Loop closure detection, pose graph optimization, map building
4. **Planning Tests** - Path planning, obstacle avoidance, trajectory generation
5. **Full Pipeline Tests** - End-to-end autonomy loop with simulated environment

## Running Tests

### All integration tests
```bash
cd bvr/firmware
cargo test --tests
```

### Specific test file
```bash
cargo test --test autonomy_integration
```

### Specific test function
```bash
cargo test --test autonomy_integration test_odometry_straight_line
```

### Include ignored tests (slow or incomplete)
```bash
cargo test --tests -- --ignored
```

### Run with logging
```bash
RUST_LOG=debug cargo test --tests -- --nocapture
```

## Test Structure

```
tests/
├── README.md                    # This file
├── autonomy_integration.rs      # Main integration tests
├── slam_scenarios.rs            # SLAM-specific tests (TODO)
├── planning_scenarios.rs        # Path planning tests (TODO)
└── common/
    ├── mod.rs                   # Shared test utilities
    ├── synthetic_scans.rs       # LiDAR scan generators (TODO)
    └── test_environments.rs     # Simulated worlds (TODO)
```

## Test Utilities (common/)

The `common/` module provides reusable utilities for tests:

### Synthetic LiDAR Scans
```rust
use common::generate_box_room_scan;

let scan = generate_box_room_scan(10.0, 10.0, 360);
// Returns 360 (range, angle) pairs for a 10m x 10m room
```

### Differential Drive Simulation
```rust
use common::simulate_differential_drive;

let new_pose = simulate_differential_drive(
    current_pose,
    left_vel,
    right_vel,
    wheel_radius,
    track_width,
    dt,
);
```

### Pose Assertions
```rust
use assert_pose_approx_eq;

assert_pose_approx_eq!(actual_pose, expected_pose, 0.01);
// Asserts poses are within 1cm and 0.01 radians
```

## Writing New Tests

### Integration Test Template
```rust
#[test]
fn test_my_feature() {
    // 1. Setup (create environment, initial state)
    let initial_pose = Isometry2::identity();

    // 2. Execute (run the algorithm/pipeline)
    let result = my_algorithm(initial_pose);

    // 3. Verify (check correctness)
    assert!(result.is_ok());
    assert_pose_approx_eq!(result.unwrap(), expected_pose, 0.01);
}
```

### Property-Based Test Template
```rust
#[test]
fn test_invariant_holds() {
    for _ in 0..1000 {
        let random_input = generate_random_input();
        let result = my_algorithm(random_input);

        // Check invariant (e.g., covariance stays positive definite)
        assert!(check_invariant(result));
    }
}
```

### Simulation Test Template
```rust
#[test]
fn test_scenario() {
    let mut state = initial_state();

    // Simulate multiple timesteps
    for t in 0..1000 {
        let sensor_data = simulate_sensors(&state);
        state = update_state(state, sensor_data);

        // Check invariants at each step
        assert!(state.is_valid());
    }

    // Check final result
    assert_eq!(state.position, expected_position);
}
```

## Test Dependencies

Integration tests have access to all workspace crates plus:
- `nalgebra` - Matrix math, SE(2) transforms
- `rand` - Random number generation for property tests
- `approx` - Floating-point comparisons (TODO: add to Cargo.toml)

Add test-specific dependencies to the workspace `Cargo.toml`:
```toml
[dev-dependencies]
approx = "0.5"
proptest = "1.0"  # For property-based testing
```

## Testing Strategy

### Week 1: Foundation
- [x] Odometry tests (straight line, arc motion)
- [x] Coordinate frame tests
- [x] Covariance propagation tests
- [ ] LiDAR driver tests (with mock data)

### Week 2: SLAM
- [ ] Scan matching tests (correlative scan matcher)
- [ ] EKF tests (predict/update cycle)
- [ ] Pose graph tests (loop closure detection)
- [ ] Occupancy grid tests (ray tracing, log-odds)

### Week 3: Planning
- [ ] Costmap tests (layering, inflation)
- [ ] Global planner tests (Hybrid A*)
- [ ] Local planner tests (DWA)
- [ ] Trajectory tracking tests

### Week 4: Integration
- [ ] Full autonomy pipeline (odometry → localization → planning → control)
- [ ] 10+ consecutive runs without crashes
- [ ] Performance benchmarks (100Hz state estimation, 10Hz planning)

## Performance Testing

Track these metrics during Week 4:

```bash
# Run with timing instrumentation
cargo test --release --tests -- --nocapture | grep -E "time|ms"

# Profile a specific test
cargo flamegraph --test autonomy_integration -- test_full_autonomy_pipeline
```

**Target Performance:**
- Odometry update: <1ms (100Hz capable)
- EKF update: <1ms (100Hz capable)
- Scan matching: <100ms (10Hz capable)
- Path planning: <500ms (acceptable for goal changes)

## Continuous Integration

These tests should run on every commit during the Artifact program:

```yaml
# .github/workflows/autonomy-tests.yml
name: Autonomy Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --tests
```

## Visual Debugging with Rerun

For complex scenarios, generate Rerun recordings:

```rust
#[test]
fn test_with_recording() {
    use rerun as rr;

    let rec = rr::RecordingStreamBuilder::new("test")
        .save("test_output.rrd")
        .unwrap();

    // Log test data
    rec.log("world/robot/pose", &rr::Transform3D::from_translation([x, y, 0.0])).unwrap();

    // Run test
    let result = my_algorithm();

    // View with: rerun test_output.rrd
}
```

## Troubleshooting

### Tests fail with "matrix not invertible"
- Check that covariances are positive definite
- Add regularization: `matrix + Matrix3::identity() * 1e-6`
- Use `try_inverse()` instead of `inverse()`

### Tests fail with angle wrapping issues
- Ensure all angles normalized to [-π, π]
- Use `normalize_angle()` after arithmetic

### Tests are slow
- Use `--release` for performance testing: `cargo test --release --tests`
- Profile with `cargo flamegraph`
- Consider using `#[ignore]` for slow tests

### Flaky tests (pass/fail randomly)
- Likely numerical precision issue
- Increase tolerances or use more iterations
- Check for race conditions in async code

## References

- [artifact-plan.md](../../../docs/artifact-plan.md) - Implementation specifications
- [CLAUDE.md](../../../CLAUDE.md) - Codebase overview
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
