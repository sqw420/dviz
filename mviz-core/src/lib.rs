//! MViz Core - Core types and traits for robotics visualization
//!
//! This crate provides the foundational types used throughout MViz:
//!
//! - **Transform types**: `FrameId`, `Timestamp`, `Transform`, `StampedTransform`, `Pose`
//! - **Point cloud types**: `PointCloud`, `Color`, `ColorMode`, `Colormap`
//! - **Marker types**: `Marker`, `MarkerType`, `MarkerArray`
//! - **Display traits**: `Display`, `DisplayContext`, `DisplayFactory`
//! - **Configuration**: `AppConfig`, `DisplayConfig`

pub mod types;
pub mod display;
pub mod config;

// Re-export commonly used types at crate root
pub use types::*;
pub use display::*;
pub use config::*;
