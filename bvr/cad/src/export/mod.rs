//! Export modules for multi-format CAD output.
//!
//! Supports:
//! - STL: Binary mesh for manufacturing
//! - glTF/GLB: PBR materials for visualization
//! - USD: Articulated robots for Isaac Sim
//! - DXF: 2D profiles for laser cutting

pub mod dxf;
pub mod materials;
pub mod stl;
pub mod usd;

#[cfg(feature = "gltf")]
#[path = "gltf.rs"]
pub mod gltf_export;

pub use dxf::DxfDocument;
pub use materials::{Material, Materials};
pub use stl::export_stl;
pub use usd::{export_usd, export_robot_usd, WheelConfig};

#[cfg(feature = "gltf")]
pub use gltf_export::export_glb;
