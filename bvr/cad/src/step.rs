//! STEP file export
//!
//! STEP export requires the `step` feature and OpenCASCADE (OCCT) installed.
//!
//! ```bash
//! # macOS
//! brew install cmake opencascade
//!
//! # Ubuntu/Debian
//! sudo apt install cmake libocct-dev
//!
//! # Build with STEP support
//! cargo build --features step
//! ```
//!
//! For now, use STL export (always available) or export STL and convert
//! using FreeCAD, Blender, or online converters.

/// Check if STEP export is available
pub fn is_available() -> bool {
    cfg!(feature = "step")
}

/// Error returned when STEP export is not available
#[derive(Debug)]
pub struct StepNotAvailable;

impl std::fmt::Display for StepNotAvailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STEP export requires the 'step' feature. Build with: cargo build --features step")
    }
}

impl std::error::Error for StepNotAvailable {}

#[cfg(feature = "step")]
mod occt_impl {
    use opencascade::primitives::Shape;
    use std::path::Path;

    /// Export a shape to STEP format
    pub fn write_step(shape: &Shape, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        shape.write_step(path.as_ref())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

#[cfg(feature = "step")]
pub use occt_impl::write_step;
