//! Unified collision monitor for all operating modes.
//!
//! Replaces the teleop-only CollisionGuard with a velocity filter applied to
//! ALL twist commands before the diff-drive mixer. Features velocity-dependent
//! safety zones: faster speeds require larger clearance.

use tracing::debug;
use types::Twist;

/// Collision monitor configuration.
#[derive(Debug, Clone)]
pub struct CollisionMonitorConfig {
    /// Whether the monitor is active.
    pub enabled: bool,
    /// How far ahead to project the arc (seconds).
    pub lookahead_time: f64,
    /// Number of sample points along the projected arc.
    pub num_samples: usize,
    /// Distance beyond which no scaling is applied (meters).
    pub full_speed_distance: f64,
    /// Distance below which linear velocity is zeroed (meters).
    pub stop_distance: f64,
    /// Costmap cost that counts as blocked (253 = INSCRIBED).
    pub cost_threshold: u8,
    /// Minimum zone scale at zero velocity (0-1). At max speed the zone
    /// equals stop_distance * 1.0; at zero speed it equals stop_distance * min_zone_scale.
    pub min_zone_scale: f64,
    /// Maximum linear velocity for zone scaling reference (m/s).
    pub max_linear_velocity: f64,
    /// Number of angular velocity candidates to sample (1 = legacy linear-only mode).
    pub num_angular_samples: usize,
    /// Angular velocity spread around operator command (rad/s).
    pub angular_spread: f64,
    /// Cost weight for deviating from operator's angular velocity (0-1).
    pub angular_cost_weight: f64,
}

impl Default for CollisionMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lookahead_time: 1.0,
            num_samples: 10,
            full_speed_distance: 1.0,
            stop_distance: 0.15,
            cost_threshold: 253,
            min_zone_scale: 0.5,
            max_linear_velocity: 1.0,
            num_angular_samples: 5,
            angular_spread: 0.5,
            angular_cost_weight: 0.3,
        }
    }
}

/// Output from the collision monitor.
#[derive(Debug, Clone)]
pub struct MonitorOutput {
    /// Filtered twist command (linear and/or angular may be adjusted).
    pub twist: Twist,
    /// Distance to nearest obstacle along the arc (f64::MAX if none).
    pub nearest_obstacle_dist: f64,
    /// Whether the command was modified (linear scaled or angular adjusted).
    pub is_limited: bool,
    /// World-frame (x, y) points along the selected trajectory arc.
    pub predicted_arc: Vec<(f64, f64)>,
}

/// Result of evaluating a single angular velocity candidate.
struct CandidateResult {
    /// Speed fraction (0.0 = full stop, 1.0 = full speed).
    speed_fraction: f64,
    /// Distance to nearest obstacle along the arc.
    nearest_dist: f64,
    /// World-frame (x, y) points along the arc.
    arc_points: Vec<(f64, f64)>,
}

/// Unified collision monitor that filters twist commands based on costmap obstacles.
///
/// Applied to ALL modes (teleop, autonomous, recovery) as a final safety layer
/// before the diff-drive mixer. When `num_angular_samples > 1`, samples multiple
/// angular velocity candidates to find collision-free trajectories closest to
/// operator intent. With `num_angular_samples = 1`, behaves identically to the
/// legacy linear-only scaling mode.
pub struct CollisionMonitor {
    config: CollisionMonitorConfig,
}

impl CollisionMonitor {
    pub fn new(config: CollisionMonitorConfig) -> Self {
        Self { config }
    }

    /// Evaluate a single (v, ω) candidate along the projected arc.
    ///
    /// Returns the speed fraction (how much linear velocity can be preserved),
    /// the distance to the nearest obstacle, and the world-frame arc points.
    fn evaluate_candidate(
        &self,
        v: f64,
        w: f64,
        robot_x: f64,
        robot_y: f64,
        cos_th: f64,
        sin_th: f64,
        effective_stop: f64,
        effective_full_speed: f64,
        cost_at: &impl Fn(f64, f64) -> u8,
    ) -> CandidateResult {
        let dt = self.config.lookahead_time / self.config.num_samples as f64;
        let mut arc_points = Vec::with_capacity(self.config.num_samples);

        for i in 1..=self.config.num_samples {
            let t = dt * i as f64;

            // Body-frame position along the arc
            let (bx, by) = if w.abs() < 1e-3 {
                (v * t, 0.0)
            } else {
                let r = v / w;
                (r * (w * t).sin(), r * (1.0 - (w * t).cos()))
            };

            // Transform to world frame
            let wx = robot_x + cos_th * bx - sin_th * by;
            let wy = robot_y + sin_th * bx + cos_th * by;
            arc_points.push((wx, wy));

            // Arc distance from robot center
            let arc_dist = if w.abs() < 1e-3 {
                (v * t).abs()
            } else {
                let r = (v / w).abs();
                r * (w * t).abs()
            };

            if cost_at(wx, wy) >= self.config.cost_threshold {
                let range = effective_full_speed - effective_stop;
                if range <= 0.0 {
                    return CandidateResult {
                        speed_fraction: 0.0,
                        nearest_dist: arc_dist,
                        arc_points,
                    };
                }
                let scale = ((arc_dist - effective_stop) / range).clamp(0.0, 1.0);
                return CandidateResult {
                    speed_fraction: scale,
                    nearest_dist: arc_dist,
                    arc_points,
                };
            }
        }

        CandidateResult {
            speed_fraction: 1.0,
            nearest_dist: f64::MAX,
            arc_points,
        }
    }

    /// Filter a twist command based on obstacle proximity along projected arcs.
    ///
    /// Samples angular velocity candidates around the operator's command and
    /// selects the best trajectory that maximizes speed while staying close to
    /// operator intent. Safety zones scale with velocity: faster = bigger zone.
    pub fn filter(
        &self,
        twist: Twist,
        robot_x: f64,
        robot_y: f64,
        robot_theta: f64,
        cost_at: impl Fn(f64, f64) -> u8,
    ) -> MonitorOutput {
        if !self.config.enabled || twist.linear.abs() < 1e-4 {
            return MonitorOutput {
                twist,
                nearest_obstacle_dist: f64::MAX,
                is_limited: false,
                predicted_arc: Vec::new(),
            };
        }

        let v = twist.linear;
        let w = twist.angular;
        let cos_th = robot_theta.cos();
        let sin_th = robot_theta.sin();

        // Velocity-dependent zone scaling
        let speed_ratio = (v.abs() / self.config.max_linear_velocity).min(1.0);
        let zone_scale =
            self.config.min_zone_scale + (1.0 - self.config.min_zone_scale) * speed_ratio;
        let effective_stop = self.config.stop_distance * zone_scale;
        let effective_full_speed = self.config.full_speed_distance * zone_scale;

        let n = self.config.num_angular_samples.max(1);
        let spread = self.config.angular_spread;
        let cost_weight = self.config.angular_cost_weight;

        let mut best_score = f64::NEG_INFINITY;
        let mut best_v = 0.0;
        let mut best_w = w;
        let mut best_dist = 0.0;
        let mut best_arc = Vec::new();

        for i in 0..n {
            // Generate angular candidate via linspace centered on operator's ω
            let candidate_w = if n == 1 {
                w
            } else {
                let frac = i as f64 / (n - 1) as f64; // 0.0 to 1.0
                w + spread * (2.0 * frac - 1.0) // w - spread to w + spread
            };

            let result = self.evaluate_candidate(
                v,
                candidate_w,
                robot_x,
                robot_y,
                cos_th,
                sin_th,
                effective_stop,
                effective_full_speed,
                &cost_at,
            );

            // Score: speed fraction minus angular deviation penalty
            let dw = if spread > 0.0 {
                (candidate_w - w) / spread
            } else {
                0.0
            };
            let score = result.speed_fraction - cost_weight * dw * dw;

            if score > best_score {
                best_score = score;
                best_v = v * result.speed_fraction;
                best_w = candidate_w;
                best_dist = result.nearest_dist;
                best_arc = result.arc_points;
            }
        }

        let is_limited = (best_v - v).abs() > 1e-6 || (best_w - w).abs() > 1e-6;

        if is_limited {
            debug!(
                nearest_dist = format!("{:.2}", best_dist),
                linear = format!("{:.2}", best_v),
                angular = format!("{:.2}", best_w),
                "Collision monitor: adjusted command"
            );
        }

        MonitorOutput {
            twist: Twist {
                linear: best_v,
                angular: best_w,
                boost: twist.boost,
            },
            nearest_obstacle_dist: best_dist,
            is_limited,
            predicted_arc: best_arc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_monitor() -> CollisionMonitor {
        CollisionMonitor::new(CollisionMonitorConfig::default())
    }

    fn fwd(v: f64) -> Twist {
        Twist {
            linear: v,
            angular: 0.0,
            boost: false,
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = CollisionMonitorConfig::default();
        assert!(config.enabled);
        assert!((config.lookahead_time - 1.0).abs() < 1e-6);
        assert_eq!(config.num_samples, 10);
        assert!((config.min_zone_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_no_obstacle() {
        let m = default_monitor();
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |_, _| 0);
        assert!((out.twist.linear - 1.0).abs() < 1e-6);
        assert!(!out.is_limited);
        assert_eq!(out.nearest_obstacle_dist, f64::MAX);
    }

    #[test]
    fn test_obstacle_full_stop() {
        let m = default_monitor();
        // Everything is blocked
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |_, _| 253);
        // First sample at t=0.1s, dist=0.1m < effective_stop
        assert!(out.twist.linear.abs() < 1e-3, "expected near-zero, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    #[test]
    fn test_disabled() {
        let config = CollisionMonitorConfig {
            enabled: false,
            ..Default::default()
        };
        let m = CollisionMonitor::new(config);
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.twist.linear - 1.0).abs() < 1e-6);
        assert!(!out.is_limited);
    }

    #[test]
    fn test_near_zero_linear_passthrough() {
        let m = default_monitor();
        let t = Twist {
            linear: 0.00005,
            angular: 1.0,
            boost: false,
        };
        let out = m.filter(t, 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.twist.linear - 0.00005).abs() < 1e-8);
    }

    #[test]
    fn test_angular_preserved_when_all_blocked() {
        let m = default_monitor();
        let t = Twist {
            linear: 1.0,
            angular: 0.5,
            boost: false,
        };
        // When all candidates are fully blocked, the center candidate (original ω) wins
        // because it has zero angular deviation penalty.
        let out = m.filter(t, 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.twist.angular - 0.5).abs() < 1e-6, "angular should stay at operator intent when all blocked");
    }

    #[test]
    fn test_velocity_dependent_zones() {
        // At low speed, the effective stop distance is smaller (min_zone_scale * stop_distance)
        // At high speed, the effective stop distance is larger
        let config = CollisionMonitorConfig {
            stop_distance: 0.2,
            full_speed_distance: 1.0,
            min_zone_scale: 0.5,
            max_linear_velocity: 1.0,
            ..Default::default()
        };
        let m = CollisionMonitor::new(config);

        // Obstacle at ~0.5m
        let cost_at = |x: f64, _y: f64| -> u8 {
            if x >= 0.45 { 253 } else { 0 }
        };

        // At high speed (1.0 m/s): zone_scale = 1.0, effective_stop = 0.2
        // Arc covers 0..1.0m in 10 samples, hits obstacle at ~0.5m
        let out_fast = m.filter(fwd(1.0), 0.0, 0.0, 0.0, &cost_at);
        assert!(out_fast.is_limited);

        // At low speed (0.2 m/s): arc only covers 0..0.2m in 1s, doesn't reach obstacle
        // So the slow robot won't even detect the obstacle at 0.45m
        let out_slow = m.filter(fwd(0.2), 0.0, 0.0, 0.0, &cost_at);
        assert!(!out_slow.is_limited, "slow robot shouldn't reach obstacle at 0.45m");

        // With obstacle closer (0.15m), both should see it
        let close_cost = |x: f64, _y: f64| -> u8 {
            if x >= 0.12 { 253 } else { 0 }
        };
        let out_fast2 = m.filter(fwd(1.0), 0.0, 0.0, 0.0, &close_cost);
        let out_slow2 = m.filter(fwd(0.2), 0.0, 0.0, 0.0, &close_cost);
        assert!(out_fast2.is_limited);
        assert!(out_slow2.is_limited);

        // Fast robot gets more aggressive scaling (larger effective zone)
        // so with same obstacle, fast should have lower scaling factor
        assert!(out_fast2.twist.linear.abs() <= out_slow2.twist.linear.abs() + 1e-6,
            "fast should be scaled more aggressively: fast={}, slow={}",
            out_fast2.twist.linear, out_slow2.twist.linear);
    }

    #[test]
    fn test_reverse() {
        let m = default_monitor();
        let out = m.filter(fwd(-1.0), 0.0, 0.0, 0.0, |x, _| {
            if x <= -0.45 { 253 } else { 0 }
        });
        assert!(out.twist.linear > -1.0 && out.twist.linear < 0.0,
            "expected scaled reverse, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    #[test]
    fn test_obstacle_midpoint_scaling() {
        let m = default_monitor();
        // Block at ~0.5m
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |x, _| {
            if x >= 0.45 { 253 } else { 0 }
        });
        // Should get partial scaling, not full stop
        assert!(out.twist.linear > 0.0 && out.twist.linear < 1.0,
            "expected partial scaling, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    // --- MPC sampling tests ---

    fn mpc_monitor() -> CollisionMonitor {
        CollisionMonitor::new(CollisionMonitorConfig {
            num_angular_samples: 5,
            angular_spread: 0.5,
            angular_cost_weight: 0.3,
            ..Default::default()
        })
    }

    #[test]
    fn test_multi_sample_steers_around() {
        let m = mpc_monitor();
        // Wall blocking the straight-ahead path (x >= 0.3), but clear to the left (y > 0.3)
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |x, y| {
            if x >= 0.3 && y.abs() < 0.3 { 253 } else { 0 }
        });
        // MPC should pick a nonzero angular velocity to steer around
        assert!(out.twist.angular.abs() > 1e-3,
            "expected nonzero angular to steer around wall, got {}", out.twist.angular);
        // And should preserve more speed than a linear-only filter would
        assert!(out.twist.linear.abs() > 0.1,
            "expected positive speed on steered path, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    #[test]
    fn test_multi_sample_prefers_operator_intent() {
        let m = mpc_monitor();
        // No obstacles — filter should return original command unchanged
        let t = Twist { linear: 0.8, angular: 0.2, boost: false };
        let out = m.filter(t, 0.0, 0.0, 0.0, |_, _| 0);
        assert!((out.twist.linear - 0.8).abs() < 1e-6,
            "expected original linear, got {}", out.twist.linear);
        assert!((out.twist.angular - 0.2).abs() < 1e-6,
            "expected original angular, got {}", out.twist.angular);
        assert!(!out.is_limited);
    }

    #[test]
    fn test_multi_sample_all_blocked() {
        let m = mpc_monitor();
        // Everything blocked at every angle
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |_, _| 253);
        assert!(out.twist.linear.abs() < 1e-3,
            "expected near-zero linear when all blocked, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    #[test]
    fn test_backward_compat_single_sample() {
        // With num_angular_samples=1, behavior should match legacy: only linear is scaled,
        // angular is exactly the operator's command.
        let config = CollisionMonitorConfig {
            num_angular_samples: 1,
            ..Default::default()
        };
        let m = CollisionMonitor::new(config);
        let t = Twist { linear: 1.0, angular: 0.3, boost: false };
        let out = m.filter(t, 0.0, 0.0, 0.0, |x, _| {
            if x >= 0.45 { 253 } else { 0 }
        });
        // Angular must be exactly preserved (single sample = operator's ω)
        assert!((out.twist.angular - 0.3).abs() < 1e-6,
            "single sample must preserve angular, got {}", out.twist.angular);
        // Linear should be scaled
        assert!(out.twist.linear > 0.0 && out.twist.linear < 1.0,
            "expected scaled linear, got {}", out.twist.linear);
        assert!(out.is_limited);
    }

    #[test]
    fn test_predicted_arc_length() {
        let m = mpc_monitor();
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |_, _| 0);
        // Arc should have num_samples points
        assert_eq!(out.predicted_arc.len(), 10,
            "expected 10 arc points, got {}", out.predicted_arc.len());
    }

    #[test]
    fn test_scoring_prefers_speed_over_deviation() {
        // Verify the cost function: a candidate with full speed but moderate angular
        // deviation should beat a candidate with zero speed and zero deviation.
        // score = speed_fraction - weight * (Δω / spread)²
        // Full speed, half-spread deviation: 1.0 - 0.3 * (0.25/0.5)² = 1.0 - 0.075 = 0.925
        // Zero speed, zero deviation: 0.0 - 0.0 = 0.0
        let config = CollisionMonitorConfig {
            num_angular_samples: 5,
            angular_spread: 0.5,
            angular_cost_weight: 0.3,
            ..Default::default()
        };
        let m = CollisionMonitor::new(config);

        // Wall blocks x >= 0.3 for y near 0, but candidate with positive ω curves left
        // and avoids the wall, getting a higher speed fraction.
        let out = m.filter(fwd(1.0), 0.0, 0.0, 0.0, |x, y| {
            if x >= 0.3 && y.abs() < 0.2 { 253 } else { 0 }
        });
        // The selected candidate should have higher speed than the straight-ahead one
        // (which would be heavily scaled due to the wall)
        assert!(out.twist.linear > 0.2,
            "expected speed > 0.2 via steering, got {}", out.twist.linear);
    }
}
