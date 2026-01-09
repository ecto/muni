//! BVR CAD - Parametric CAD library for rover parts
//!
//! Uses manifold-rs for CSG operations and STL export.
//! Optional OpenCASCADE integration for STEP export (enable `step` feature).
//!
//! # Example
//!
//! ```rust
//! use bvr_cad::parts::{Extrusion2020, BVR1Frame};
//!
//! // Create a single extrusion
//! let rail = Extrusion2020::new(500.0).generate();
//! rail.write_stl("rail.stl").unwrap();
//!
//! // Create complete frame assembly
//! let frame = BVR1Frame::default_bvr1().generate();
//! frame.write_stl("bvr1_frame.stl").unwrap();
//! ```

use manifold_rs::{Manifold, Mesh};
use nalgebra::Vector3;
use std::f64::consts::PI;
use std::io::Write;
use thiserror::Error;

pub mod export;
pub mod parts;
pub mod step;

pub use export::{Material, Materials};

#[derive(Error, Debug)]
pub enum CadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Empty geometry")]
    EmptyGeometry,
}

/// A named part with geometry
pub struct Part {
    pub name: String,
    manifold: Manifold,
}

impl Part {
    /// Create a new part with a name
    pub fn new(name: impl Into<String>, manifold: Manifold) -> Self {
        Self {
            name: name.into(),
            manifold,
        }
    }

    /// Create an empty part
    pub fn empty(name: impl Into<String>) -> Self {
        Self::new(name, Manifold::empty())
    }

    /// Create a cube/box centered at origin
    pub fn cube(name: impl Into<String>, x: f64, y: f64, z: f64) -> Self {
        let manifold = Manifold::cube(x, y, z);
        Self::new(name, manifold)
    }

    /// Create a cylinder along Z axis, centered at origin
    pub fn cylinder(name: impl Into<String>, radius: f64, height: f64, segments: u32) -> Self {
        let manifold = Manifold::cylinder(radius, radius, height, segments);
        Self::new(name, manifold)
    }

    /// Create a cone/tapered cylinder
    pub fn cone(
        name: impl Into<String>,
        radius_bottom: f64,
        radius_top: f64,
        height: f64,
        segments: u32,
    ) -> Self {
        let manifold = Manifold::cylinder(radius_bottom, radius_top, height, segments);
        Self::new(name, manifold)
    }

    /// Create a sphere centered at origin
    pub fn sphere(name: impl Into<String>, radius: f64, segments: u32) -> Self {
        let manifold = Manifold::sphere(radius, segments);
        Self::new(name, manifold)
    }

    /// Boolean difference (self - other)
    pub fn difference(&self, other: &Part) -> Self {
        Self::new(
            format!("{}-diff", self.name),
            self.manifold.difference(&other.manifold),
        )
    }

    /// Boolean union (self + other)
    pub fn union(&self, other: &Part) -> Self {
        Self::new(
            format!("{}-union", self.name),
            self.manifold.union(&other.manifold),
        )
    }

    /// Boolean intersection
    pub fn intersection(&self, other: &Part) -> Self {
        Self::new(
            format!("{}-intersect", self.name),
            self.manifold.intersection(&other.manifold),
        )
    }

    /// Translate the part
    pub fn translate(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.name.clone(), self.manifold.translate(x, y, z))
    }

    /// Translate by vector
    pub fn translate_vec(&self, v: Vector3<f64>) -> Self {
        self.translate(v.x, v.y, v.z)
    }

    /// Rotate the part (angles in degrees)
    pub fn rotate(&self, x_deg: f64, y_deg: f64, z_deg: f64) -> Self {
        Self::new(self.name.clone(), self.manifold.rotate(x_deg, y_deg, z_deg))
    }

    /// Scale the part
    pub fn scale(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.name.clone(), self.manifold.scale(x, y, z))
    }

    /// Uniform scale
    pub fn scale_uniform(&self, s: f64) -> Self {
        self.scale(s, s, s)
    }

    /// Check if geometry is empty
    pub fn is_empty(&self) -> bool {
        self.manifold.is_empty()
    }

    /// Get the mesh representation
    pub fn to_mesh(&self) -> Mesh {
        self.manifold.to_mesh()
    }

    /// Export to binary STL format
    pub fn to_stl(&self) -> Result<Vec<u8>, CadError> {
        let mesh = self.to_mesh();
        let vertices = mesh.vertices();
        let indices = mesh.indices();

        if vertices.is_empty() || indices.is_empty() {
            return Err(CadError::EmptyGeometry);
        }

        let num_triangles = indices.len() / 3;
        let mut buffer = Vec::with_capacity(84 + num_triangles * 50);

        // STL header (80 bytes)
        let header = format!("{:<80}", self.name);
        buffer.extend_from_slice(&header.as_bytes()[..80.min(header.len())]);
        buffer.resize(80, 0);

        // Number of triangles (4 bytes, little endian)
        buffer.extend_from_slice(&(num_triangles as u32).to_le_bytes());

        // Each triangle: normal (12 bytes) + 3 vertices (36 bytes) + attribute (2 bytes)
        for tri in indices.chunks(3) {
            let i0 = tri[0] as usize * 3;
            let i1 = tri[1] as usize * 3;
            let i2 = tri[2] as usize * 3;

            let v0 = Vector3::new(
                vertices[i0] as f64,
                vertices[i0 + 1] as f64,
                vertices[i0 + 2] as f64,
            );
            let v1 = Vector3::new(
                vertices[i1] as f64,
                vertices[i1 + 1] as f64,
                vertices[i1 + 2] as f64,
            );
            let v2 = Vector3::new(
                vertices[i2] as f64,
                vertices[i2 + 1] as f64,
                vertices[i2 + 2] as f64,
            );

            // Calculate normal
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2).normalize();

            // Write normal
            buffer.extend_from_slice(&(normal.x as f32).to_le_bytes());
            buffer.extend_from_slice(&(normal.y as f32).to_le_bytes());
            buffer.extend_from_slice(&(normal.z as f32).to_le_bytes());

            // Write vertices
            for v in [v0, v1, v2] {
                buffer.extend_from_slice(&(v.x as f32).to_le_bytes());
                buffer.extend_from_slice(&(v.y as f32).to_le_bytes());
                buffer.extend_from_slice(&(v.z as f32).to_le_bytes());
            }

            // Attribute byte count (0)
            buffer.extend_from_slice(&0u16.to_le_bytes());
        }

        Ok(buffer)
    }

    /// Write STL to file
    pub fn write_stl(&self, path: impl AsRef<std::path::Path>) -> Result<(), CadError> {
        let stl_data = self.to_stl()?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(&stl_data)?;
        Ok(())
    }
}

/// Helper to create a centered cube (manifold cubes are corner-aligned by default)
pub fn centered_cube(name: impl Into<String>, x: f64, y: f64, z: f64) -> Part {
    Part::cube(name, x, y, z).translate(-x / 2.0, -y / 2.0, -z / 2.0)
}

/// Helper to create a centered cylinder
pub fn centered_cylinder(name: impl Into<String>, radius: f64, height: f64, segments: u32) -> Part {
    Part::cylinder(name, radius, height, segments).translate(0.0, 0.0, -height / 2.0)
}

/// Create a counterbore hole (through hole + larger shallow hole for bolt head)
pub fn counterbore_hole(
    through_diameter: f64,
    counterbore_diameter: f64,
    counterbore_depth: f64,
    total_depth: f64,
    segments: u32,
) -> Part {
    let through = Part::cylinder("through", through_diameter / 2.0, total_depth, segments);
    let counterbore = Part::cylinder("counterbore", counterbore_diameter / 2.0, counterbore_depth, segments)
        .translate(0.0, 0.0, total_depth - counterbore_depth);
    through.union(&counterbore)
}

/// Create a bolt pattern (circle of holes)
pub fn bolt_pattern(
    num_holes: usize,
    bolt_circle_diameter: f64,
    hole_diameter: f64,
    depth: f64,
    segments: u32,
) -> Part {
    let radius = bolt_circle_diameter / 2.0;
    let mut result = Part::empty("bolt_pattern");

    for i in 0..num_holes {
        let angle = 2.0 * PI * (i as f64) / (num_holes as f64);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let hole = Part::cylinder("hole", hole_diameter / 2.0, depth, segments).translate(x, y, 0.0);
        result = result.union(&hole);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_creation() {
        let cube = Part::cube("test", 10.0, 10.0, 10.0);
        assert!(!cube.is_empty());
    }

    #[test]
    fn test_cylinder_creation() {
        let cyl = Part::cylinder("test", 5.0, 10.0, 32);
        assert!(!cyl.is_empty());
    }

    #[test]
    fn test_difference() {
        let cube = Part::cube("cube", 10.0, 10.0, 10.0);
        let hole = Part::cylinder("hole", 3.0, 15.0, 32).translate(5.0, 5.0, -1.0);
        let result = cube.difference(&hole);
        assert!(!result.is_empty());
    }
}
