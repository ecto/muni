//! BVR CAD - Parametric CAD library for rover parts
//!
//! Built on [`vcad`] for CSG modeling and multi-format export.
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

// Re-export core types from vcad
pub use vcad::{
    bolt_pattern, centered_cube, centered_cylinder, counterbore_hole,
    CadError, Part, Scene, SceneNode,
};

// Re-export export subsystem
pub use vcad::export;
pub use vcad::step;

// Re-export commonly used export types
pub use vcad::export::{Material, Materials};

// BVR-specific parts
pub mod parts;
