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
        }
    }
}

/// Output from the collision monitor.
#[derive(Debug, Clone)]
pub struct MonitorOutput {
    /// Filtered twist command (linear may be scaled down).
    pub twist: Twist,
    /// Distance to nearest obstacle along the arc (f64::MAX if none).
    pub nearest_obstacle_dist: f64,
    /// Whether linear velocity was limited.
    pub is_limited: bool,
}

/// Unified collision monitor that filters twist commands based on costmap obstacles.
///
/// Applied to ALL modes (teleop, autonomous, recovery) as a final safety layer
/// before the diff-drive mixer. Angular velocity is never modified — the
/// operator/controller can always steer away.
pub struct CollisionMonitor {
    config: CollisionMonitorConfig,
}

impl CollisionMonitor {
    pub fn new(config: CollisionMonitorConfig) -> Self {
        Self { config }
    }

    /// Filter a twist command based on obstacle proximity along the projected arc.
    ///
    /// Scales linear velocity down when obstacles are detected. Angular velocity
    /// is never modified. Safety zones scale with velocity: faster = bigger zone.
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
            };
        }

        let v = twist.linear;
        let w = twist.angular;
        let dt = self.config.lookahead_time / self.config.num_samples as f64;
        let cos_th = robot_theta.cos();
        let sin_th = robot_theta.sin();

        // Velocity-dependent zone scaling
        let speed_ratio = (v.abs() / self.config.max_linear_velocity).min(1.0);
        let zone_scale =
            self.config.min_zone_scale + (1.0 - self.config.min_zone_scale) * speed_ratio;
        let effective_stop = self.config.stop_distance * zone_scale;
        let effective_full_speed = self.config.full_speed_distance * zone_scale;

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
                    debug!(
                        arc_dist = format!("{:.2}", arc_dist),
                        "Collision monitor: full stop (range <= 0)"
                    );
                    return MonitorOutput {
                        twist: Twist { linear: 0.0, ..twist },
                        nearest_obstacle_dist: arc_dist,
                        is_limited: true,
                    };
                }
                let scale = ((arc_dist - effective_stop) / range).clamp(0.0, 1.0);
                debug!(
                    arc_dist = format!("{:.2}", arc_dist),
                    scale = format!("{:.2}", scale),
                    zone_scale = format!("{:.2}", zone_scale),
                    "Collision monitor: scaling velocity"
                );
                return MonitorOutput {
                    twist: Twist {
                        linear: v * scale,
                        ..twist
                    },
                    nearest_obstacle_dist: arc_dist,
                    is_limited: scale < 1.0,
                };
            }
        }

        MonitorOutput {
            twist,
            nearest_obstacle_dist: f64::MAX,
            is_limited: false,
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
    fn test_angular_never_modified() {
        let m = default_monitor();
        let t = Twist {
            linear: 1.0,
            angular: 0.5,
            boost: false,
        };
        let out = m.filter(t, 0.0, 0.0, 0.0, |_, _| 253);
        assert!((out.twist.angular - 0.5).abs() < 1e-6, "angular must not be modified");
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
}
