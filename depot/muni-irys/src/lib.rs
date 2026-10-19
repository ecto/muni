//! muni-irys: the in-process GPU sim camera.
//!
//! Renders the Booster K1's head camera (metric depth `f32` and sRGB8 RGB)
//! with irys, for the sim service and for RL. A batch is N environments:
//! N copies of the street, each with its own K1, all in one irys scene
//! (bottom-level meshes shared, one top-level instance per placement), and
//! one camera per environment.
//!
//! - [`k1`]: the K1 from its URDF and STLs, placed link by link from forward
//!   kinematics; joint angles come from a [`trace`] or from the caller.
//! - [`street`]: the snowy sidewalk world.
//! - [`camera`]: the head camera's real intrinsics and extrinsics.
//! - [`render`]: [`render::SimCamera`], the batched depth/RGB renderer.
//!
//! See `docs/irys-camera.md` for the design and the numbers.

pub mod camera;
pub mod k1;
pub mod render;
pub mod stl;
pub mod street;
pub mod trace;
pub mod urdf;
pub mod xf;

use std::path::PathBuf;

/// Everything that can go wrong building or rendering.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read.
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    /// A URDF did not parse.
    #[error("urdf: {0}")]
    Urdf(String),
    /// An STL did not parse.
    #[error("stl {0}: {1}")]
    Stl(PathBuf, String),
    /// A trace did not parse.
    #[error("trace: {0}")]
    Trace(String),
    /// The GPU could not be used.
    #[error("gpu: {0}")]
    Gpu(String),
}

pub use camera::{Eye, HeadCamera};
pub use k1::{K1Model, K1Pose};
pub use render::{DepthImage, RgbImage, SimCamera};
pub use street::Street;
