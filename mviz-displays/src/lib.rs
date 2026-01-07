//! MViz Display Plugins
//!
//! This crate provides display plugins for the MViz robotics visualizer.
//!
//! ## Available Displays
//!
//! - [`GridDisplay`] - Reference grid on XY/XZ/YZ plane
//! - [`AxesDisplay`] - RGB coordinate frame axes
//! - [`TfDisplay`] - Transform tree visualization
//! - [`PointCloudDisplay`] - Point cloud with decay and queue support
//! - [`MarkerDisplay`] - Visualization markers with lifetime management
//!
//! ## Usage
//!
//! ```ignore
//! use mviz_displays::{GridDisplay, AxesDisplay, PointCloudDisplay};
//!
//! // Create displays
//! let mut grid = GridDisplay::new("Ground Grid");
//! let mut axes = AxesDisplay::new("World Axes");
//! let mut lidar = PointCloudDisplay::new("LiDAR");
//!
//! // Configure
//! grid.set_cell_count(20);
//! axes.set_scale(0.5);
//! lidar.set_color_mode(ColorMode::z_height(0.0, 5.0));
//!
//! // Update in render loop
//! grid.update(&ctx)?;
//! axes.update(&ctx)?;
//! lidar.update(&ctx)?;
//! ```

pub mod base;
pub mod grid;
pub mod axes;
pub mod tf;
pub mod point_cloud;
pub mod marker;

// Re-exports
pub use base::{BaseDisplay, DisplayUpdateContext};
pub use grid::{GridDisplay, GridPlane, GridProperties};
pub use axes::{AxesDisplay, AxesProperties};
pub use tf::{TfDisplay, TfProperties};
pub use point_cloud::{PointCloudDisplay, PointCloudProperties};
pub use marker::{MarkerDisplay, MarkerKey, MarkerProperties};

/// Display type identifiers
pub mod display_types {
    pub const GRID: &str = "grid";
    pub const AXES: &str = "axes";
    pub const TF: &str = "tf";
    pub const POINT_CLOUD: &str = "point_cloud";
    pub const MARKER: &str = "marker";
}
