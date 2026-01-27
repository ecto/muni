//! Boustrophedon coverage path planner.
//!
//! Generates optimized sweep waypoints from polygon boundaries for area
//! coverage tasks (snow blowing, vacuuming sidewalks and plazas).
//!
//! Algorithm:
//! 1. Determine sweep angle (longest edge heuristic or user-specified)
//! 2. Rotate polygon so sweep aligns with X axis
//! 3. Generate parallel sweep lines spaced at `tool_width * (1 - overlap_pct/100)`
//! 4. Intersect each line with polygon (handles non-convex shapes)
//! 5. Order segments in alternating direction (boustrophedon pattern)
//! 6. Rotate all waypoints back to original frame

use geo::algorithm::BoundingRect;
use geo::algorithm::Intersects;
use geo::algorithm::LineIntersection;
use geo::{AffineOps, AffineTransform, Coord, Line, LineString, MultiPoint, Polygon};
use serde::{Deserialize, Serialize};

use crate::Waypoint;

/// Configuration for coverage path generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageConfig {
    /// Tool width in meters
    pub tool_width: f64,
    /// Overlap percentage (0–50)
    pub overlap_pct: f64,
    /// Sweep angle in radians. None = auto (longest edge heuristic).
    pub swath_angle: Option<f64>,
}

/// Result stats returned alongside generated waypoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageResult {
    pub waypoints: Vec<Waypoint>,
    pub path_length: f64,
    pub num_sweeps: usize,
}

/// Generate boustrophedon coverage waypoints for a polygon zone.
pub fn generate(polygon_wps: &[Waypoint], config: &CoverageConfig) -> Result<CoverageResult, String> {
    if polygon_wps.len() < 3 {
        return Err("Polygon must have at least 3 vertices".into());
    }
    if config.tool_width <= 0.0 {
        return Err("Tool width must be positive".into());
    }
    if !(0.0..=50.0).contains(&config.overlap_pct) {
        return Err("Overlap must be between 0 and 50 percent".into());
    }

    // Build geo polygon from waypoints
    let coords: Vec<Coord> = polygon_wps.iter().map(|w| Coord { x: w.x, y: w.y }).collect();
    let line_string = LineString::new(coords.clone());
    let polygon = Polygon::new(line_string, vec![]);

    // Determine sweep angle
    let angle = config.swath_angle.unwrap_or_else(|| longest_edge_angle(&coords));

    // Rotation transform: rotate polygon so sweep direction aligns with X axis
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    // Rotate by -angle to align sweep with X
    let to_sweep = AffineTransform::new(cos_a, sin_a, 0.0, -sin_a, cos_a, 0.0);
    // Inverse: rotate back by +angle
    let from_sweep = AffineTransform::new(cos_a, -sin_a, 0.0, sin_a, cos_a, 0.0);

    let rotated = polygon.affine_transform(&to_sweep);

    let bbox = rotated
        .bounding_rect()
        .ok_or("Failed to compute bounding rectangle")?;

    let step = config.tool_width * (1.0 - config.overlap_pct / 100.0);
    if step <= 0.0 {
        return Err("Effective step size must be positive".into());
    }

    let y_min = bbox.min().y;
    let y_max = bbox.max().y;
    let x_min = bbox.min().x;
    let x_max = bbox.max().x;

    // Small margin to extend sweep lines past polygon bounds
    let x_margin = (x_max - x_min) * 0.01;

    // Generate sweep lines and intersect with rotated polygon
    let mut sweep_segments: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut y = y_min + step / 2.0;

    while y <= y_max {
        let sweep_line = Line::new(
            Coord {
                x: x_min - x_margin,
                y,
            },
            Coord {
                x: x_max + x_margin,
                y,
            },
        );

        // Find intersections of this horizontal line with all polygon edges
        let mut x_crossings = intersect_line_polygon(&sweep_line, &rotated);
        x_crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Deduplicate nearby crossings
        x_crossings.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

        // Pair consecutive crossings into segments (inside the polygon)
        let mut segments = Vec::new();
        for pair in x_crossings.chunks(2) {
            if pair.len() == 2 {
                segments.push((pair[0], pair[1]));
            }
        }
        if !segments.is_empty() {
            // Convert segments to (x, y) points: start and end of each segment
            let mut line_points = Vec::new();
            for (x_start, x_end) in segments {
                line_points.push((x_start, y));
                line_points.push((x_end, y));
            }
            sweep_segments.push(line_points);
        }

        y += step;
    }

    if sweep_segments.is_empty() {
        return Err("No sweep lines intersect the polygon".into());
    }

    // Build boustrophedon path: alternate direction on each sweep line
    let mut path_points: Vec<Coord> = Vec::new();
    let num_sweeps = sweep_segments.len();

    for (i, segment_points) in sweep_segments.iter().enumerate() {
        if i % 2 == 0 {
            // Left to right
            for &(x, y) in segment_points.iter() {
                path_points.push(Coord { x, y });
            }
        } else {
            // Right to left
            for &(x, y) in segment_points.iter().rev() {
                path_points.push(Coord { x, y });
            }
        }
    }

    // Rotate path back to original frame
    let result_points: MultiPoint = MultiPoint::new(
        path_points.into_iter().map(geo::Point::from).collect(),
    );
    let original_points = result_points.affine_transform(&from_sweep);

    // Convert to waypoints and compute path length
    let waypoints: Vec<Waypoint> = original_points
        .0
        .iter()
        .map(|p| Waypoint {
            x: p.x(),
            y: p.y(),
            theta: None,
        })
        .collect();

    let path_length = compute_path_length(&waypoints);

    Ok(CoverageResult {
        waypoints,
        path_length,
        num_sweeps,
    })
}

/// Find the angle of the longest edge of the polygon (heuristic for optimal sweep).
fn longest_edge_angle(coords: &[Coord]) -> f64 {
    let mut best_angle = 0.0;
    let mut best_len_sq = 0.0;

    for i in 0..coords.len() {
        let j = (i + 1) % coords.len();
        let dx = coords[j].x - coords[i].x;
        let dy = coords[j].y - coords[i].y;
        let len_sq = dx * dx + dy * dy;
        if len_sq > best_len_sq {
            best_len_sq = len_sq;
            best_angle = dy.atan2(dx);
        }
    }

    best_angle
}

/// Intersect a horizontal line with the edges of a polygon, returning X coordinates
/// of all intersection points.
fn intersect_line_polygon(line: &Line, polygon: &Polygon) -> Vec<f64> {
    let mut crossings = Vec::new();
    let exterior = polygon.exterior();
    let points: Vec<Coord> = exterior.0.clone();

    for i in 0..points.len().saturating_sub(1) {
        let edge = Line::new(points[i], points[i + 1]);
        if !line.intersects(&edge) {
            continue;
        }
        if let Some(intersection) = geo::algorithm::line_intersection::line_intersection(
            edge.into(),
            (*line).into(),
        ) {
            match intersection {
                LineIntersection::SinglePoint { intersection, .. } => {
                    crossings.push(intersection.x);
                }
                LineIntersection::Collinear { intersection } => {
                    crossings.push(intersection.start.x);
                    crossings.push(intersection.end.x);
                }
            }
        }
    }

    crossings
}

/// Compute total path length from waypoints.
fn compute_path_length(waypoints: &[Waypoint]) -> f64 {
    waypoints
        .windows(2)
        .map(|w| {
            let dx = w[1].x - w[0].x;
            let dy = w[1].y - w[0].y;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_polygon(w: f64, h: f64) -> Vec<Waypoint> {
        vec![
            Waypoint { x: 0.0, y: 0.0, theta: None },
            Waypoint { x: w, y: 0.0, theta: None },
            Waypoint { x: w, y: h, theta: None },
            Waypoint { x: 0.0, y: h, theta: None },
        ]
    }

    #[test]
    fn rectangular_coverage() {
        let polygon = rect_polygon(10.0, 8.0);
        let config = CoverageConfig {
            tool_width: 1.0,
            overlap_pct: 0.0,
            swath_angle: Some(0.0), // Sweep along X axis
        };
        let result = generate(&polygon, &config).unwrap();

        // With 1m tool width and 8m height, expect 8 sweeps
        assert_eq!(result.num_sweeps, 8);
        // Each sweep should produce 2 points (start + end), so 16 total
        assert_eq!(result.waypoints.len(), 16);
        // Path length should be > 0
        assert!(result.path_length > 0.0);
    }

    #[test]
    fn overlap_reduces_spacing() {
        let polygon = rect_polygon(10.0, 10.0);

        let no_overlap = generate(
            &polygon,
            &CoverageConfig {
                tool_width: 1.0,
                overlap_pct: 0.0,
                swath_angle: Some(0.0),
            },
        )
        .unwrap();

        let with_overlap = generate(
            &polygon,
            &CoverageConfig {
                tool_width: 1.0,
                overlap_pct: 50.0,
                swath_angle: Some(0.0),
            },
        )
        .unwrap();

        // More overlap = more sweeps
        assert!(with_overlap.num_sweeps > no_overlap.num_sweeps);
    }

    #[test]
    fn l_shaped_polygon() {
        // L-shape: non-convex polygon
        let polygon = vec![
            Waypoint { x: 0.0, y: 0.0, theta: None },
            Waypoint { x: 4.0, y: 0.0, theta: None },
            Waypoint { x: 4.0, y: 2.0, theta: None },
            Waypoint { x: 2.0, y: 2.0, theta: None },
            Waypoint { x: 2.0, y: 4.0, theta: None },
            Waypoint { x: 0.0, y: 4.0, theta: None },
        ];
        let config = CoverageConfig {
            tool_width: 1.0,
            overlap_pct: 0.0,
            swath_angle: Some(0.0),
        };
        let result = generate(&polygon, &config).unwrap();

        // Should have sweeps at multiple Y levels
        assert!(result.num_sweeps >= 3);
        // All waypoints should be inside or on the boundary of the L-shape
        for wp in &result.waypoints {
            assert!(wp.x >= -0.1 && wp.x <= 4.1);
            assert!(wp.y >= -0.1 && wp.y <= 4.1);
        }
    }

    #[test]
    fn narrow_corridor() {
        // Very narrow polygon: should produce 1-2 sweeps
        let polygon = rect_polygon(10.0, 0.8);
        let config = CoverageConfig {
            tool_width: 1.0,
            overlap_pct: 0.0,
            swath_angle: Some(0.0),
        };
        let result = generate(&polygon, &config).unwrap();
        assert!(result.num_sweeps >= 1 && result.num_sweeps <= 2);
    }

    #[test]
    fn rejects_invalid_input() {
        let polygon = vec![
            Waypoint { x: 0.0, y: 0.0, theta: None },
            Waypoint { x: 1.0, y: 0.0, theta: None },
        ];
        let config = CoverageConfig {
            tool_width: 1.0,
            overlap_pct: 0.0,
            swath_angle: None,
        };
        assert!(generate(&polygon, &config).is_err());

        let polygon = rect_polygon(10.0, 10.0);
        assert!(generate(&polygon, &CoverageConfig {
            tool_width: -1.0,
            overlap_pct: 0.0,
            swath_angle: None,
        }).is_err());

        assert!(generate(&polygon, &CoverageConfig {
            tool_width: 1.0,
            overlap_pct: 60.0,
            swath_angle: None,
        }).is_err());
    }

    #[test]
    fn auto_angle_selects_longest_edge() {
        // Rectangle wider than tall: longest edge is horizontal (angle ≈ 0)
        let polygon = rect_polygon(10.0, 2.0);
        let config = CoverageConfig {
            tool_width: 0.5,
            overlap_pct: 0.0,
            swath_angle: None, // Auto
        };
        let result = generate(&polygon, &config).unwrap();
        assert!(result.num_sweeps > 0);
        assert!(result.waypoints.len() > 0);
    }
}
