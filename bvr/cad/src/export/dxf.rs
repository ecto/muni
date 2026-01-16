//! DXF export for 2D laser cutting profiles.
//!
//! Exports flat profiles as DXF R12 format for SendCutSend and other
//! laser cutting services. Supports:
//! - Cut lines (layer "0" - default)
//! - Bend lines (layer "BEND" - for forming services)

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// A 2D point for DXF export
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A 2D shape for DXF export
#[derive(Debug, Clone)]
pub enum Shape2D {
    /// Rectangular outline
    Rectangle { width: f64, height: f64, center: Point2D },
    /// Circle (for holes)
    Circle { center: Point2D, radius: f64 },
    /// Rounded rectangle (for slots)
    RoundedRectangle {
        width: f64,
        height: f64,
        center: Point2D,
        corner_radius: f64,
    },
    /// Line segment (for bend lines, etc.)
    Line { start: Point2D, end: Point2D, layer: String },
    /// Slot (stadium shape - rectangle with semicircular ends)
    Slot { width: f64, height: f64, center: Point2D },
    /// Arbitrary closed polyline (for complex profiles like knuckle tabs)
    Polyline { points: Vec<Point2D>, closed: bool },
    /// Arc (for rounded corners in complex profiles)
    Arc { center: Point2D, radius: f64, start_angle: f64, end_angle: f64 },
}

/// DXF document builder
pub struct DxfDocument {
    shapes: Vec<Shape2D>,
}

impl DxfDocument {
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn add_shape(&mut self, shape: Shape2D) {
        self.shapes.push(shape);
    }

    /// Add a rectangular outline
    pub fn add_rectangle(&mut self, width: f64, height: f64, cx: f64, cy: f64) {
        self.shapes.push(Shape2D::Rectangle {
            width,
            height,
            center: Point2D::new(cx, cy),
        });
    }

    /// Add a circle (for holes)
    pub fn add_circle(&mut self, cx: f64, cy: f64, radius: f64) {
        self.shapes.push(Shape2D::Circle {
            center: Point2D::new(cx, cy),
            radius,
        });
    }

    /// Add a rounded rectangle (for slots)
    pub fn add_rounded_rectangle(
        &mut self,
        width: f64,
        height: f64,
        cx: f64,
        cy: f64,
        corner_radius: f64,
    ) {
        self.shapes.push(Shape2D::RoundedRectangle {
            width,
            height,
            center: Point2D::new(cx, cy),
            corner_radius,
        });
    }

    /// Add a line segment on the default layer
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.shapes.push(Shape2D::Line {
            start: Point2D::new(x1, y1),
            end: Point2D::new(x2, y2),
            layer: "0".to_string(),
        });
    }

    /// Add a bend line (on BEND layer for forming services)
    pub fn add_bend_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.shapes.push(Shape2D::Line {
            start: Point2D::new(x1, y1),
            end: Point2D::new(x2, y2),
            layer: "BEND".to_string(),
        });
    }

    /// Add a slot (stadium shape - rectangle with semicircular ends)
    /// Used for louver vents and elongated cutouts
    pub fn add_slot(&mut self, width: f64, height: f64, cx: f64, cy: f64) {
        self.shapes.push(Shape2D::Slot {
            width,
            height,
            center: Point2D::new(cx, cy),
        });
    }

    /// Add an arbitrary closed polyline (for complex profiles)
    pub fn add_polyline(&mut self, points: Vec<(f64, f64)>, closed: bool) {
        self.shapes.push(Shape2D::Polyline {
            points: points.into_iter().map(|(x, y)| Point2D::new(x, y)).collect(),
            closed,
        });
    }

    /// Add an arc (for rounded corners)
    pub fn add_arc(&mut self, cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64) {
        self.shapes.push(Shape2D::Arc {
            center: Point2D::new(cx, cy),
            radius,
            start_angle,
            end_angle,
        });
    }

    /// Generate knuckle hinge tab profile points along an edge
    /// Returns points for a profile with tabs extending in the +Y direction
    /// `edge_y` - Y coordinate of the edge
    /// `edge_x_start` - X coordinate of edge start
    /// `edge_x_end` - X coordinate of edge end
    /// `tab_width` - Width of each tab
    /// `tab_height` - Height of each tab (how far they extend)
    /// `num_tabs` - Number of tabs
    /// `offset` - If true, offset tabs (for mating piece)
    pub fn knuckle_tab_edge_points(
        edge_y: f64,
        edge_x_start: f64,
        edge_x_end: f64,
        tab_width: f64,
        tab_height: f64,
        num_tabs: usize,
        offset: bool,
    ) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        let edge_length = edge_x_end - edge_x_start;
        let total_tabs = num_tabs * 2 - 1; // tabs + gaps
        let segment_width = edge_length / total_tabs as f64;

        // Start from left edge
        let mut x = edge_x_start;
        let start_with_tab = !offset;

        for i in 0..total_tabs {
            let is_tab = if start_with_tab { i % 2 == 0 } else { i % 2 == 1 };

            if is_tab {
                // Tab: go up, across, down
                points.push((x, edge_y));
                points.push((x, edge_y + tab_height));
                points.push((x + segment_width, edge_y + tab_height));
                points.push((x + segment_width, edge_y));
            } else {
                // Gap: just the edge (will connect to next segment)
                points.push((x, edge_y));
                points.push((x + segment_width, edge_y));
            }
            x += segment_width;
        }

        // Remove duplicate points where segments meet
        let mut deduped: Vec<(f64, f64)> = Vec::new();
        for p in points {
            if deduped.is_empty() || {
                let last = deduped.last().unwrap();
                (last.0 - p.0).abs() > 0.001 || (last.1 - p.1).abs() > 0.001
            } {
                deduped.push(p);
            }
        }

        deduped
    }

    /// Export to DXF file
    pub fn export(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // DXF Header
        writeln!(writer, "0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "2")?;
        writeln!(writer, "HEADER")?;
        writeln!(writer, "9")?;
        writeln!(writer, "$ACADVER")?;
        writeln!(writer, "1")?;
        writeln!(writer, "AC1009")?; // DXF R12
        writeln!(writer, "9")?;
        writeln!(writer, "$INSUNITS")?;
        writeln!(writer, "70")?;
        writeln!(writer, "4")?; // Millimeters
        writeln!(writer, "0")?;
        writeln!(writer, "ENDSEC")?;

        // Tables section (minimal)
        writeln!(writer, "0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "2")?;
        writeln!(writer, "TABLES")?;
        writeln!(writer, "0")?;
        writeln!(writer, "ENDSEC")?;

        // Entities section
        writeln!(writer, "0")?;
        writeln!(writer, "SECTION")?;
        writeln!(writer, "2")?;
        writeln!(writer, "ENTITIES")?;

        for shape in &self.shapes {
            match shape {
                Shape2D::Rectangle { width, height, center } => {
                    self.write_rectangle(&mut writer, *width, *height, center)?;
                }
                Shape2D::Circle { center, radius } => {
                    self.write_circle(&mut writer, center, *radius)?;
                }
                Shape2D::RoundedRectangle {
                    width,
                    height,
                    center,
                    corner_radius,
                } => {
                    self.write_rounded_rectangle(
                        &mut writer,
                        *width,
                        *height,
                        center,
                        *corner_radius,
                    )?;
                }
                Shape2D::Line { start, end, layer } => {
                    self.write_line(&mut writer, start, end, layer)?;
                }
                Shape2D::Slot { width, height, center } => {
                    self.write_slot(&mut writer, *width, *height, center)?;
                }
                Shape2D::Polyline { points, closed } => {
                    self.write_polyline(&mut writer, points, *closed)?;
                }
                Shape2D::Arc { center, radius, start_angle, end_angle } => {
                    self.write_arc(&mut writer, center, *radius, *start_angle, *end_angle)?;
                }
            }
        }

        writeln!(writer, "0")?;
        writeln!(writer, "ENDSEC")?;

        // End of file
        writeln!(writer, "0")?;
        writeln!(writer, "EOF")?;

        Ok(())
    }

    fn write_rectangle(
        &self,
        writer: &mut impl Write,
        width: f64,
        height: f64,
        center: &Point2D,
    ) -> std::io::Result<()> {
        let x1 = center.x - width / 2.0;
        let y1 = center.y - height / 2.0;
        let x2 = center.x + width / 2.0;
        let y2 = center.y + height / 2.0;

        // LWPOLYLINE (lightweight polyline)
        writeln!(writer, "0")?;
        writeln!(writer, "LWPOLYLINE")?;
        writeln!(writer, "8")?;
        writeln!(writer, "0")?; // Layer 0
        writeln!(writer, "90")?;
        writeln!(writer, "4")?; // 4 vertices
        writeln!(writer, "70")?;
        writeln!(writer, "1")?; // Closed polyline

        // Vertex 1 (bottom-left)
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1)?;

        // Vertex 2 (bottom-right)
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1)?;

        // Vertex 3 (top-right)
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2)?;

        // Vertex 4 (top-left)
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2)?;

        Ok(())
    }

    fn write_circle(
        &self,
        writer: &mut impl Write,
        center: &Point2D,
        radius: f64,
    ) -> std::io::Result<()> {
        writeln!(writer, "0")?;
        writeln!(writer, "CIRCLE")?;
        writeln!(writer, "8")?;
        writeln!(writer, "0")?; // Layer 0
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", center.x)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", center.y)?;
        writeln!(writer, "40")?;
        writeln!(writer, "{:.6}", radius)?;

        Ok(())
    }

    fn write_rounded_rectangle(
        &self,
        writer: &mut impl Write,
        width: f64,
        height: f64,
        center: &Point2D,
        corner_radius: f64,
    ) -> std::io::Result<()> {
        // For rounded rectangles, we approximate with a polyline
        // For simplicity, use arcs at corners

        let r = corner_radius.min(width / 2.0).min(height / 2.0);
        let x1 = center.x - width / 2.0;
        let y1 = center.y - height / 2.0;
        let x2 = center.x + width / 2.0;
        let y2 = center.y + height / 2.0;

        // Use a polyline with bulge values for rounded corners
        writeln!(writer, "0")?;
        writeln!(writer, "LWPOLYLINE")?;
        writeln!(writer, "8")?;
        writeln!(writer, "0")?;
        writeln!(writer, "90")?;
        writeln!(writer, "8")?; // 8 vertices (2 per corner)
        writeln!(writer, "70")?;
        writeln!(writer, "1")?; // Closed

        // Bulge value for 90-degree arc: tan(45°/2) = 0.414
        let bulge = 0.414213562; // tan(π/8)

        // Bottom edge
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1 + r)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1)?;

        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2 - r)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1)?;
        writeln!(writer, "42")?;
        writeln!(writer, "{:.6}", bulge)?; // Bulge for corner arc

        // Right edge
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1 + r)?;

        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2 - r)?;
        writeln!(writer, "42")?;
        writeln!(writer, "{:.6}", bulge)?;

        // Top edge
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x2 - r)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2)?;

        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1 + r)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2)?;
        writeln!(writer, "42")?;
        writeln!(writer, "{:.6}", bulge)?;

        // Left edge
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y2 - r)?;

        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", x1)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", y1 + r)?;
        writeln!(writer, "42")?;
        writeln!(writer, "{:.6}", bulge)?;

        Ok(())
    }

    fn write_line(
        &self,
        writer: &mut impl Write,
        start: &Point2D,
        end: &Point2D,
        layer: &str,
    ) -> std::io::Result<()> {
        writeln!(writer, "0")?;
        writeln!(writer, "LINE")?;
        writeln!(writer, "8")?;
        writeln!(writer, "{}", layer)?; // Layer name
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", start.x)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", start.y)?;
        writeln!(writer, "11")?;
        writeln!(writer, "{:.6}", end.x)?;
        writeln!(writer, "21")?;
        writeln!(writer, "{:.6}", end.y)?;

        Ok(())
    }

    fn write_slot(
        &self,
        writer: &mut impl Write,
        width: f64,
        height: f64,
        center: &Point2D,
    ) -> std::io::Result<()> {
        // Stadium shape: rectangle with semicircular ends
        // The semicircles are on the shorter dimension
        let (long, short) = if width >= height {
            (width, height)
        } else {
            (height, width)
        };

        let r = short / 2.0;
        let straight = long - short; // Length of straight portion

        if width >= height {
            // Horizontal slot
            let x1 = center.x - straight / 2.0;
            let x2 = center.x + straight / 2.0;

            // LWPOLYLINE with 4 vertices and bulges for semicircles
            writeln!(writer, "0")?;
            writeln!(writer, "LWPOLYLINE")?;
            writeln!(writer, "8")?;
            writeln!(writer, "0")?;
            writeln!(writer, "90")?;
            writeln!(writer, "4")?; // 4 vertices
            writeln!(writer, "70")?;
            writeln!(writer, "1")?; // Closed

            // Bottom-left (start of left semicircle)
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", x1)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", center.y - r)?;
            writeln!(writer, "42")?;
            writeln!(writer, "1.0")?; // Bulge for 180° arc (semicircle)

            // Top-left (end of left semicircle)
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", x1)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", center.y + r)?;

            // Top-right (start of right semicircle)
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", x2)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", center.y + r)?;
            writeln!(writer, "42")?;
            writeln!(writer, "1.0")?; // Bulge for 180° arc

            // Bottom-right (end of right semicircle)
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", x2)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", center.y - r)?;
        } else {
            // Vertical slot
            let y1 = center.y - straight / 2.0;
            let y2 = center.y + straight / 2.0;

            writeln!(writer, "0")?;
            writeln!(writer, "LWPOLYLINE")?;
            writeln!(writer, "8")?;
            writeln!(writer, "0")?;
            writeln!(writer, "90")?;
            writeln!(writer, "4")?;
            writeln!(writer, "70")?;
            writeln!(writer, "1")?;

            // Left-bottom
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", center.x - r)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", y1)?;
            writeln!(writer, "42")?;
            writeln!(writer, "1.0")?;

            // Right-bottom
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", center.x + r)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", y1)?;

            // Right-top
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", center.x + r)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", y2)?;
            writeln!(writer, "42")?;
            writeln!(writer, "1.0")?;

            // Left-top
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", center.x - r)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", y2)?;
        }

        Ok(())
    }

    fn write_polyline(
        &self,
        writer: &mut impl Write,
        points: &[Point2D],
        closed: bool,
    ) -> std::io::Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        writeln!(writer, "0")?;
        writeln!(writer, "LWPOLYLINE")?;
        writeln!(writer, "8")?;
        writeln!(writer, "0")?; // Layer 0
        writeln!(writer, "90")?;
        writeln!(writer, "{}", points.len())?;
        writeln!(writer, "70")?;
        writeln!(writer, "{}", if closed { 1 } else { 0 })?;

        for point in points {
            writeln!(writer, "10")?;
            writeln!(writer, "{:.6}", point.x)?;
            writeln!(writer, "20")?;
            writeln!(writer, "{:.6}", point.y)?;
        }

        Ok(())
    }

    fn write_arc(
        &self,
        writer: &mut impl Write,
        center: &Point2D,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> std::io::Result<()> {
        writeln!(writer, "0")?;
        writeln!(writer, "ARC")?;
        writeln!(writer, "8")?;
        writeln!(writer, "0")?; // Layer 0
        writeln!(writer, "10")?;
        writeln!(writer, "{:.6}", center.x)?;
        writeln!(writer, "20")?;
        writeln!(writer, "{:.6}", center.y)?;
        writeln!(writer, "40")?;
        writeln!(writer, "{:.6}", radius)?;
        writeln!(writer, "50")?;
        writeln!(writer, "{:.6}", start_angle)?;
        writeln!(writer, "51")?;
        writeln!(writer, "{:.6}", end_angle)?;

        Ok(())
    }
}

impl Default for DxfDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_dxf_rectangle() {
        let mut doc = DxfDocument::new();
        doc.add_rectangle(100.0, 50.0, 0.0, 0.0);

        let path = "/tmp/test_rect.dxf";
        doc.export(path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LWPOLYLINE"));
        assert!(content.contains("EOF"));
    }

    #[test]
    fn test_dxf_circle() {
        let mut doc = DxfDocument::new();
        doc.add_circle(10.0, 20.0, 5.0);

        let path = "/tmp/test_circle.dxf";
        doc.export(path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("CIRCLE"));
    }

    #[test]
    fn test_dxf_rounded_rectangle() {
        let mut doc = DxfDocument::new();
        doc.add_rounded_rectangle(100.0, 50.0, 0.0, 0.0, 10.0);

        let path = "/tmp/test_rounded.dxf";
        doc.export(path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LWPOLYLINE"));
        assert!(content.contains("42")); // Bulge code
    }
}
