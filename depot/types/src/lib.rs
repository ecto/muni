//! Shared types for Depot services.
//!
//! This crate provides common type definitions used across depot services
//! (mapper, map-api, discovery, dispatch) to ensure consistency and reduce
//! duplication.

pub mod geo;
pub mod map;
pub mod session;

// Re-export commonly used types at crate root
pub use geo::{GpsBounds, GpsCoord};
pub use map::{MapAssets, MapIndex, MapIndexEntry, MapManifest, MapSessionRef, MapStats};
pub use session::{Session, SessionMetadata, SessionStatus};
