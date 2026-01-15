# SLAM & Localization Review

Reviews SLAM (Simultaneous Localization and Mapping) and state estimation code for the BVR rover with focus on mathematical correctness, numerical stability, coordinate frame consistency, real-time performance, and robustness. Use when reviewing localization, mapping, scan matching, pose estimation, or coordinate transform code for autonomous navigation during the F.Inc Artifact program (Week 2 milestone: "SLAM + Localization working").

**Key Technologies:** Rust, nalgebra (SE2, Isometry2, Matrix types), ndarray (for grids), 2D LiDAR (RPLidar A1), differential drive odometry, Extended Kalman Filter (EKF), correlative scan matching, pose graph optimization (Levenberg-Marquardt).

**Target Performance:** 100Hz state estimation output, real-time scan matching (<100ms per scan), stable localization for 10+ consecutive autonomous demo runs.

---

## Overview

The SLAM system provides accurate pose estimates for the autonomous rover by fusing:
1. **Wheel odometry** from VESC motor controllers (with covariance propagation)
2. **LiDAR scan matching** for drift correction (correlative scan matcher)
3. **Pose graph SLAM** for global consistency and loop closure detection
4. **Occupancy grid mapping** with Bayesian log-odds updates

**Architecture:**
```
Wheel Encoders ──┐
                 ├──► Extended Kalman Filter ──► Pose Graph SLAM ──► Map
LiDAR Scans ────┘         (100Hz)                  (on demand)

Output: SE2 pose (x, y, θ) with 3x3 covariance matrix
```

**Critical Requirements:**
- **Correctness**: One degree of rotation error = 1.7cm error per meter traveled
- **Stability**: No filter divergence, matrix inversions must be numerically stable
- **Performance**: 100Hz loop rate for control, <100ms scan matching
- **Safety**: Graceful degradation if scan matching fails (fall back to odometry)

---

## Review Checklist

### 1. Coordinate Frame Management

Proper coordinate frame usage is critical. Bugs here cause subtle drift and crashes.

**Frame Definitions:**
- `world`: Fixed global frame (map origin, never moves)
- `odom`: Continuous odometry frame (drifts over time, smooth)
- `base`: Robot base frame (center of rear axle, x=forward, y=left)
- `lidar`: LiDAR sensor frame (fixed offset from base)

**Transform Chain:** `world ← odom ← base ← lidar`
- SLAM corrects `world ← odom` discontinuously (when loop closures detected)
- All other transforms are continuous

#### Checklist:

- [ ] **Frame constants defined correctly**
  ```rust
  ✅ GOOD:
  pub const WORLD: FrameId = FrameId("world");
  pub const ODOM: FrameId = FrameId("odom");
  pub const BASE: FrameId = FrameId("base");
  pub const LIDAR: FrameId = FrameId("lidar");

  ❌ BAD:
  // Hardcoded strings scattered throughout code
  transform.from_frame("world")
  ```

- [ ] **Transforms applied in correct order**
  ```rust
  ✅ GOOD:
  // Transform point from lidar to world
  let point_base = T_base_lidar * point_lidar;
  let point_odom = T_odom_base * point_base;
  let point_world = T_world_odom * point_odom;

  ❌ BAD:
  // Wrong order (will give incorrect results)
  let point_world = T_world_odom * point_lidar;
  ```

- [ ] **LiDAR-to-base transform is static** (measured once, hardcoded)
  ```rust
  ✅ GOOD:
  const T_BASE_LIDAR: Isometry2<f64> = Isometry2::new(
      Vector2::new(0.15, 0.0),  // 15cm forward of rear axle
      0.0,                       // No rotation
  );
  ```

- [ ] **Angle normalization used consistently** (wrap to [-π, π])
  ```rust
  ✅ GOOD:
  fn normalize_angle(angle: f64) -> f64 {
      let mut a = angle % (2.0 * PI);
      if a > PI { a -= 2.0 * PI; }
      else if a < -PI { a += 2.0 * PI; }
      a
  }
  // Use after all angle arithmetic
  theta = normalize_angle(theta + dtheta);

  ❌ BAD:
  theta = theta + dtheta;  // Can exceed [-π, π] range
  ```

**Why This Matters:**
- Mixing frames causes 10-100cm position errors instantly
- Unnormalized angles break scan matching (expects [-π, π])
- Wrong transform order accumulates error quadratically

---

### 2. Differential Drive Odometry

Integrates wheel velocities with proper uncertainty propagation.

#### Checklist:

- [ ] **Exact arc integration** (not Euler approximation)
  ```rust
  ✅ GOOD:
  let delta = if omega.abs() < 1e-6 {
      // Straight line
      Vector3::new(v * dt, 0.0, 0.0)
  } else {
      // Arc motion (exact)
      let r = v / omega;
      Vector3::new(
          r * omega.sin() * dt,
          r * (1.0 - (omega * dt).cos()),
          omega * dt,
      )
  };

  ❌ BAD:
  // Euler integration (accumulates error)
  let dx = v * dt * theta.cos();
  let dy = v * dt * theta.sin();
  ```

- [ ] **Velocity computed correctly from wheel speeds**
  ```rust
  ✅ GOOD:
  let v = (left_vel + right_vel) / 2.0 * wheel_radius;
  let omega = (right_vel - left_vel) / track_width * wheel_radius;

  ❌ BAD:
  let v = left_vel * wheel_radius;  // Ignores right wheel
  ```

- [ ] **Delta transformed to world frame before applying**
  ```rust
  ✅ GOOD:
  let delta_world = self.pose.rotation * delta.xy();
  self.pose.translation.vector += delta_world;

  ❌ BAD:
  self.pose.translation.vector += delta.xy();  // Wrong frame
  ```

- [ ] **Covariance propagated with motion model Jacobian**
  ```rust
  ✅ GOOD:
  let g = Matrix3::new(
      1.0, 0.0, -v * dt * theta.sin(),
      0.0, 1.0,  v * dt * theta.cos(),
      0.0, 0.0,  1.0,
  );
  self.covariance = g * self.covariance * g.transpose() + q;

  ❌ BAD:
  self.covariance = self.covariance + q;  // Missing Jacobian
  ```

- [ ] **Motion noise is velocity-dependent**
  ```rust
  ✅ GOOD:
  let v_var = alpha[0] * v.abs() + alpha[1] * omega.abs();
  let omega_var = alpha[2] * v.abs() + alpha[3] * omega.abs();

  ❌ BAD:
  let v_var = 0.01;  // Constant noise (unrealistic)
  ```

**Why This Matters:**
- Euler integration drifts 10cm per 10m traveled
- Wrong frame transforms cause immediate 10-50cm errors
- Missing covariance propagation → EKF doesn't trust odometry correctly

---

### 3. Extended Kalman Filter (EKF)

Fuses odometry with scan matching corrections.

#### Checklist:

- [ ] **Prediction step uses odometry delta** (not absolute pose)
  ```rust
  ✅ GOOD:
  pub fn predict(&mut self, odom_delta: &Isometry2<f64>, odom_cov: &Matrix3<f64>) {
      let dx = odom_delta.translation.x;
      let dy = odom_delta.translation.y;
      let dtheta = odom_delta.rotation.angle();
      // Apply relative motion
  }

  ❌ BAD:
  pub fn predict(&mut self, odom_pose: &Isometry2<f64>) {
      self.state = odom_pose;  // Replaces EKF state entirely
  }
  ```

- [ ] **Innovation (y) normalizes angle difference**
  ```rust
  ✅ GOOD:
  let y = measured_pose - self.state;
  let y = Vector3::new(y.x, y.y, normalize_angle(y.z));

  ❌ BAD:
  let y = measured_pose - self.state;  // Angle can be > π
  ```

- [ ] **Kalman gain uses safe matrix inversion**
  ```rust
  ✅ GOOD:
  let s = self.covariance + measurement_cov;
  let k = self.covariance * s.try_inverse()
      .unwrap_or(Matrix3::identity());  // Fallback

  ❌ BAD:
  let k = self.covariance * (self.covariance + measurement_cov).inverse();
  // Panics if singular
  ```

- [ ] **Covariance stays positive semi-definite**
  ```rust
  ✅ GOOD:
  // Joseph form (numerically stable)
  let i_kh = Matrix3::identity() - &k * &h;
  self.covariance = &i_kh * &self.covariance * i_kh.transpose()
                    + &k * &r * k.transpose();

  ⚠️ ACCEPTABLE (less stable):
  self.covariance = (Matrix3::identity() - k) * self.covariance;

  ❌ BAD:
  self.covariance = self.covariance - k;  // Not positive definite
  ```

- [ ] **State updated after covariance** (correct EKF order)
  ```rust
  ✅ GOOD:
  let k = ...;
  self.covariance = (Matrix3::identity() - k) * self.covariance;
  self.state += k * y;  // Update state last
  ```

**Why This Matters:**
- Non-normalized angles cause 180° flip errors
- Singular matrix inversions crash the system
- Negative eigenvalues in covariance → filter diverges

---

### 4. Scan Matching (Correlative Scan Matcher)

Aligns LiDAR scan to map/previous scan. More robust than ICP for 2D LiDAR.

#### Checklist:

- [ ] **Search window is reasonable** (not too large or small)
  ```rust
  ✅ GOOD:
  linear_search_window: 0.5,    // ±50cm
  angular_search_window: 0.3,   // ±17°
  linear_resolution: 0.05,      // 5cm steps
  angular_resolution: 0.05,     // ~3° steps

  ❌ BAD:
  linear_search_window: 5.0,    // ±5m (too large, slow)
  linear_resolution: 0.01,      // 1cm (overkill, 25x slower)
  ```

- [ ] **Coarse search followed by fine refinement**
  ```rust
  ✅ GOOD:
  let coarse_result = self.coarse_search(scan, lookup, initial);
  let fine_result = self.fine_search(scan, reference, coarse_result.pose);

  ❌ BAD:
  // Only coarse search (poor accuracy)
  let result = self.coarse_search(scan, lookup, initial);
  ```

- [ ] **Score normalized by scan point count**
  ```rust
  ✅ GOOD:
  score / scan.points.len() as f64

  ❌ BAD:
  score  // Biased toward longer scans
  ```

- [ ] **Covariance estimated from score surface curvature**
  ```rust
  ✅ GOOD:
  // Compute Hessian (second derivatives)
  let hessian = self.compute_hessian(scan, reference, pose);
  // Invert to get covariance
  hessian.try_inverse().unwrap_or(Matrix3::identity() * 0.1)

  ❌ BAD:
  Matrix3::identity() * 0.01  // Constant covariance (wrong)
  ```

- [ ] **Lookup table precomputed for speed**
  ```rust
  ✅ GOOD:
  let lookup = self.build_lookup_table(reference);  // Once
  for each pose {
      score = self.score_pose(scan, &lookup, pose);  // Fast
  }

  ❌ BAD:
  for each pose {
      score = self.score_directly(scan, reference, pose);  // Slow
  }
  ```

- [ ] **Invalid scans rejected** (out of range, NaN)
  ```rust
  ✅ GOOD:
  for (range, angle) in scan.iter() {
      if !range.is_finite() || *range < min_range || *range > max_range {
          continue;
      }
      // Process point
  }

  ❌ BAD:
  for (range, angle) in scan.iter() {
      // Process all points (including NaN)
  }
  ```

**Why This Matters:**
- Large search windows: 100ms → 10 seconds (100x slower)
- Missing fine search: 5-10cm accuracy loss
- No lookup table: 10-50x slower
- Bad covariance: EKF trusts scan matching incorrectly

**Performance Target:** <100ms per scan match for real-time operation.

---

### 5. Pose Graph SLAM

Maintains global consistency via loop closure and optimization.

#### Checklist:

- [ ] **Node spacing prevents redundant nodes**
  ```rust
  ✅ GOOD:
  if dist < self.node_spacing {  // e.g., 0.5m
      return None;  // Too close to last node
  }

  ❌ BAD:
  // Add node every scan (1000s of nodes)
  self.nodes.push(node);
  ```

- [ ] **Loop closure skips recent nodes** (avoid false positives)
  ```rust
  ✅ GOOD:
  if current_id.saturating_sub(node.id) < 10 {
      continue;  // Skip recent nodes (odometry already connects them)
  }

  ❌ BAD:
  // Check all nodes (detects false loop closures with adjacent scans)
  ```

- [ ] **Loop closure uses match score threshold**
  ```rust
  ✅ GOOD:
  if result.score > 0.8 {  // High confidence
      self.edges.push(PoseEdge { ... });
  }

  ❌ BAD:
  self.edges.push(PoseEdge { ... });  // Adds bad matches
  ```

- [ ] **Information matrix is inverse covariance**
  ```rust
  ✅ GOOD:
  information: result.covariance.try_inverse()
      .unwrap_or(Matrix3::identity() * 10.0)

  ❌ BAD:
  information: result.covariance  // Wrong (should be inverse)
  ```

- [ ] **Optimization uses Levenberg-Marquardt damping**
  ```rust
  ✅ GOOD:
  let h_damped = h + Matrix::identity(n, n) * lambda;
  let dx = h_damped.lu().solve(&b)?;
  // Adjust lambda based on error reduction

  ❌ BAD:
  let dx = h.lu().solve(&b)?;  // Gauss-Newton (less stable)
  ```

- [ ] **First node fixed (gauge freedom)**
  ```rust
  ✅ GOOD:
  for k in 0..3 {
      h[(k, k)] += 1e10;  // Pin first pose
  }

  ❌ BAD:
  // No anchor (system is under-constrained, no unique solution)
  ```

- [ ] **Convergence checked** (iteration limit + tolerance)
  ```rust
  ✅ GOOD:
  const MAX_ITERATIONS: usize = 100;
  const TOLERANCE: f64 = 1e-6;
  for i in 0..MAX_ITERATIONS {
      let dx = solve_system();
      if dx.norm() < TOLERANCE { break; }
  }

  ❌ BAD:
  loop {
      let dx = solve_system();  // Infinite loop if no convergence
  }
  ```

**Why This Matters:**
- Too many nodes: optimization takes 10+ seconds
- Wrong information matrix: optimizer weights edges incorrectly
- No damping: optimization diverges or oscillates
- No anchor: system has infinite solutions (will drift)

---

### 6. Occupancy Grid Mapping

Builds 2D map using log-odds Bayesian updates.

#### Checklist:

- [ ] **Log-odds for numerical stability** (not raw probabilities)
  ```rust
  ✅ GOOD:
  data: Array2<f32>,  // Log-odds values
  log_odds_hit: 0.85_f32.ln() - 0.15_f32.ln(),

  fn update_cell(&mut self, occupied: bool) {
      self.data[(y, x)] += if occupied {
          self.log_odds_hit
      } else {
          self.log_odds_miss
      };
  }

  ❌ BAD:
  data: Array2<f32>,  // Probabilities [0, 1]
  self.data[(y, x)] *= if occupied { 0.85 } else { 0.4 };
  // Multiplying probabilities → underflow
  ```

- [ ] **Log-odds clamped to prevent certainty**
  ```rust
  ✅ GOOD:
  let updated = (current + log_odds_update)
      .clamp(log_odds_min, log_odds_max);

  ❌ BAD:
  let updated = current + log_odds_update;  // Unbounded
  ```

- [ ] **Ray tracing uses Bresenham's algorithm**
  ```rust
  ✅ GOOD:
  fn trace_ray(&mut self, start: Vector2<f64>, end: Vector2<f64>) {
      // Bresenham's line algorithm
      let mut x = start_cell.0 as i32;
      let mut y = start_cell.1 as i32;
      // ... (efficient integer arithmetic)
  }

  ❌ BAD:
  // DDA (slower, less accurate)
  for t in 0..steps {
      let point = start + (end - start) * t / steps;
      // ...
  }
  ```

- [ ] **Free cells marked along ray, obstacle at endpoint**
  ```rust
  ✅ GOOD:
  for cell in ray_cells[..ray_cells.len()-1] {
      self.update_cell(cell.x, cell.y, false);  // Free
  }
  if hit_obstacle {
      self.update_cell(end_cell.x, end_cell.y, true);  // Occupied
  }

  ❌ BAD:
  // Only mark endpoint (no free space information)
  self.update_cell(end_cell.x, end_cell.y, true);
  ```

- [ ] **Out-of-bounds checks**
  ```rust
  ✅ GOOD:
  if x >= self.data.ncols() || y >= self.data.nrows() {
      return;
  }

  ❌ BAD:
  self.data[(y, x)] = ...;  // Panics if out of bounds
  ```

- [ ] **Probability conversion for visualization**
  ```rust
  ✅ GOOD:
  pub fn probability(&self, x: usize, y: usize) -> f64 {
      let log_odds = self.data[(y, x)] as f64;
      1.0 / (1.0 + (-log_odds).exp())
  }

  ❌ BAD:
  pub fn probability(&self, x: usize, y: usize) -> f64 {
      self.data[(y, x)] as f64  // Returns log-odds, not probability
  }
  ```

**Why This Matters:**
- Raw probabilities underflow after 50 scans (becomes 0.0)
- No clamping: map becomes 100% certain (can't update)
- Wrong ray tracing: 10x slower, jagged lines
- Missing free space: map is all obstacles (unusable)

---

### 7. Performance & Real-Time Constraints

SLAM must run in real-time for autonomous navigation.

#### Checklist:

- [ ] **State estimation outputs at 100Hz** (10ms loop time)
  ```rust
  ✅ GOOD:
  loop {
      let start = Instant::now();

      // Update odometry, EKF (fast operations)
      let pose = self.update_state_estimate(dt);

      let elapsed = start.elapsed();
      if elapsed > Duration::from_millis(10) {
          warn!("State estimation loop slow: {:?}", elapsed);
      }
  }
  ```

- [ ] **Scan matching runs asynchronously** (not in 100Hz loop)
  ```rust
  ✅ GOOD:
  // Spawn scan matching in separate task
  tokio::spawn(async move {
      let result = scan_matcher.match_scan(scan, map, pose);
      send_correction(result).await;
  });

  ❌ BAD:
  // Blocks 100Hz loop for 50-100ms
  let result = scan_matcher.match_scan(scan, map, pose);
  ```

- [ ] **Pose graph optimization runs on-demand** (not every scan)
  ```rust
  ✅ GOOD:
  if self.has_recent_loop_closure() {
      tokio::spawn(async move {
          self.optimize();  // Can take 1-10 seconds
      });
  }

  ❌ BAD:
  self.optimize();  // Every scan (too slow)
  ```

- [ ] **Profiling/timing instrumentation**
  ```rust
  ✅ GOOD:
  use tracing::instrument;

  #[instrument(skip(self, scan))]
  fn match_scan(&self, scan: &Scan) -> Result {
      let start = Instant::now();
      // ... algorithm ...
      let elapsed = start.elapsed();
      tracing::info!("scan_match_time_ms", elapsed.as_millis());
  }
  ```

**Performance Targets:**
- Odometry update: <1ms (100Hz capable)
- EKF predict/update: <1ms (100Hz capable)
- Scan matching: <100ms (10Hz capable)
- Pose graph optimization: <5 seconds (acceptable for loop closures)

---

### 8. Numerical Stability & Error Handling

SLAM algorithms have many failure modes. Handle them gracefully.

#### Checklist:

- [ ] **Matrix inversions use try_inverse()** with fallback
  ```rust
  ✅ GOOD:
  covariance.try_inverse()
      .unwrap_or(Matrix3::identity() * 0.1)

  ❌ BAD:
  covariance.inverse()  // Panics if singular
  ```

- [ ] **Small denominators checked**
  ```rust
  ✅ GOOD:
  let delta = if omega.abs() < 1e-6 {
      // Straight line case (omega ≈ 0)
      Vector3::new(v * dt, 0.0, 0.0)
  } else {
      // Arc motion
      let r = v / omega;
      // ...
  }

  ❌ BAD:
  let r = v / omega;  // Division by zero if omega = 0
  ```

- [ ] **Angle arithmetic uses normalize_angle()**
  ```rust
  ✅ GOOD:
  self.state.z = normalize_angle(self.state.z + dtheta);

  ❌ BAD:
  self.state.z = self.state.z + dtheta;  // Can exceed [-π, π]
  ```

- [ ] **NaN/Inf checks for sensor data**
  ```rust
  ✅ GOOD:
  if !range.is_finite() {
      continue;  // Skip invalid measurement
  }

  ❌ BAD:
  let point = compute_point(range, angle);  // NaN propagates
  ```

- [ ] **Covariance stays positive definite**
  ```rust
  ✅ GOOD:
  // Add small regularization to prevent negative eigenvalues
  let cov = (hessian + Matrix3::identity() * 1e-6)
      .try_inverse()
      .unwrap_or(Matrix3::identity() * 0.1);

  ❌ BAD:
  let cov = hessian.inverse();  // Can have negative eigenvalues
  ```

- [ ] **Graceful degradation if scan matching fails**
  ```rust
  ✅ GOOD:
  match scan_matcher.match_scan(scan, map, pose) {
      Ok(result) if result.score > 0.5 => {
          ekf.update(&result.pose, &result.covariance);
      }
      _ => {
          // Fall back to odometry-only
          warn!("Scan matching failed, using odometry only");
      }
  }

  ❌ BAD:
  let result = scan_matcher.match_scan(scan, map, pose).unwrap();
  ekf.update(&result.pose, &result.covariance);
  ```

**Why This Matters:**
- One panic during demo = fail the entire demo (no recovery)
- NaN in covariance → entire filter corrupted
- Bad scan match → 1m localization error instantly

---

### 9. Testing & Validation

SLAM is hard to test (no ground truth outdoors). Use these strategies:

#### Checklist:

- [ ] **Unit tests for coordinate transforms**
  ```rust
  #[test]
  fn test_transform_chain() {
      let T_world_odom = Isometry2::new(Vector2::new(1.0, 0.0), 0.0);
      let T_odom_base = Isometry2::new(Vector2::new(0.5, 0.0), 0.0);
      let T_base_lidar = Isometry2::new(Vector2::new(0.15, 0.0), 0.0);

      let point_lidar = Vector2::new(1.0, 0.0);
      let point_world = T_world_odom * T_odom_base * T_base_lidar * point_lidar;

      assert_approx_eq!(point_world.x, 2.65);
      assert_approx_eq!(point_world.y, 0.0);
  }
  ```

- [ ] **Tests for angle normalization edge cases**
  ```rust
  #[test]
  fn test_normalize_angle() {
      assert_approx_eq!(normalize_angle(0.0), 0.0);
      assert_approx_eq!(normalize_angle(PI), PI);
      assert_approx_eq!(normalize_angle(-PI), -PI);
      assert_approx_eq!(normalize_angle(3.0 * PI), PI);
      assert_approx_eq!(normalize_angle(-3.0 * PI), -PI);
  }
  ```

- [ ] **Property tests for covariance propagation**
  ```rust
  #[test]
  fn test_covariance_stays_positive_definite() {
      let mut odom = DifferentialOdometry::new(/* ... */);

      for _ in 0..1000 {
          odom.update(rand_vel(), rand_vel(), 0.01);

          // Check eigenvalues are all positive
          let eigenvalues = odom.covariance.symmetric_eigenvalues();
          assert!(eigenvalues.iter().all(|&e| e > 0.0));
      }
  }
  ```

- [ ] **Synthetic dataset tests** (simulate rover + LiDAR)
  ```rust
  #[test]
  fn test_slam_with_square_loop() {
      let mut slam = PoseGraph::new();

      // Simulate rover driving a 10m x 10m square
      let scans = generate_square_loop_scans();

      for scan in scans {
          slam.add_scan(scan, odom_pose);
      }

      // Check loop closure detected
      assert!(slam.has_loop_closure());

      // Check final pose error < 10cm
      let error = (slam.current_pose() - slam.initial_pose()).norm();
      assert!(error < 0.1);
  }
  ```

- [ ] **Rerun recordings for visual debugging**
  ```rust
  use rerun as rr;

  fn log_slam_state(&self, rec: &rr::RecordingStream) {
      rec.log("world/robot/pose", &rr::Transform3D::from_pose(&self.pose))?;
      rec.log("world/map/occupancy", &rr::Image::from_array2d(&self.map.data))?;
      rec.log("world/lidar/scan", &rr::Points2D::from_scan(&self.scan))?;
  }
  ```

**Testing Strategy:**
1. Unit tests: Math, transforms, covariance
2. Property tests: Stability over 1000s of iterations
3. Synthetic tests: Known ground truth (square loops, straight lines)
4. Integration tests: Real LiDAR data with manual ground truth
5. Visual validation: Rerun recordings for human inspection

---

## Common Pitfalls

### Pitfall 1: Frame Confusion
**Symptom:** Robot thinks it's 5 meters away from actual position
**Cause:** Mixed up `odom` and `world` frames, or forgot to transform LiDAR to base
**Fix:** Always check frame consistency, use typed frames (`FrameId`)

### Pitfall 2: Angle Wrapping
**Symptom:** Scan matching fails 50% of the time, rover spins randomly
**Cause:** Angles outside [-π, π] range, diff of 350° interpreted as -10°
**Fix:** Call `normalize_angle()` after every angle arithmetic operation

### Pitfall 3: Covariance Explosion
**Symptom:** EKF covariance grows to 1e10, filter stops correcting
**Cause:** Missing covariance clamping, or measurement noise too small (over-trusts sensors)
**Fix:** Clamp eigenvalues to [1e-6, 1e6], validate sensor noise parameters

### Pitfall 4: Singular Matrix
**Symptom:** Panic in `.inverse()` call during pose graph optimization
**Cause:** Under-constrained system (no anchor node) or duplicate edges
**Fix:** Use `.try_inverse()` with fallback, fix first node in optimization

### Pitfall 5: Slow Scan Matching
**Symptom:** Real-time drops to 1Hz during scan matching (should be 10Hz)
**Cause:** Search window too large, no lookup table precomputation
**Fix:** Reduce search window to ±50cm/±20°, precompute lookup table

### Pitfall 6: False Loop Closures
**Symptom:** Map suddenly warps, rover thinks it's somewhere else
**Cause:** Low score threshold (accepts bad matches), or no recent-node skip
**Fix:** Increase threshold to 0.8+, skip nodes within 10 indices

---

## Quick Commands

### Build & Test
```bash
cd bvr/firmware

# Build SLAM crates
cargo build -p transforms -p odometry -p slam -p costmap

# Run unit tests
cargo test -p slam

# Run specific test
cargo test -p slam test_normalize_angle

# Type check
cargo check -p slam
```

### Run with Mock Data
```bash
# Record test data
cargo run --bin bvrd -- --record test_slam.rrd

# Replay and visualize
rerun test_slam.rrd
```

### Profile Performance
```bash
# Profile scan matching
cargo build --release -p slam
perf record -g ./target/release/test_scan_matcher
perf report
```

### Visual Debugging
```bash
# Generate Rerun recording with SLAM state
RUST_LOG=slam=debug cargo run --bin bvrd -- --record slam_debug.rrd

# Open in Rerun viewer
rerun slam_debug.rrd
# Check:
# - world/robot/pose (should be smooth)
# - world/map/occupancy (should show walls)
# - world/lidar/scan (should align with map)
```

---

## References

- [artifact-plan.md](../../../docs/artifact-plan.md) - Detailed SLAM implementation specs
- [CLAUDE.md](../../../CLAUDE.md) - Codebase overview
- [bvr/firmware/README.md](../../../bvr/firmware/README.md) - Build and deployment
- [nalgebra docs](https://docs.rs/nalgebra) - Matrix math library
- [Probabilistic Robotics](http://www.probabilistic-robotics.org/) - SLAM theory

## Example Review Session

**Scenario:** Reviewing a scan matching implementation for Week 2 milestone.

**Steps:**
1. Read `bvr/firmware/crates/slam/src/scan_matcher.rs`
2. Check coordinate frame usage (Section 1)
3. Verify coarse + fine search (Section 4)
4. Check covariance estimation (Section 4)
5. Validate numerical stability (Section 8)
6. Check for performance issues (Section 7)
7. Run tests: `cargo test -p slam`
8. Generate Rerun recording for visual validation

**Report findings:**
- ✅ Coordinate frames correct
- ❌ Missing angle normalization in line 142
- ⚠️ Search window too large (±2m, should be ±0.5m)
- ✅ Covariance estimation looks good
- ❌ No `.try_inverse()` fallback in line 89

**Recommendation:** Fix angle normalization (critical), reduce search window (performance), add safe inversion (stability).
