//! MViz Widgets - Reusable Makepad UI components
//!
//! This crate provides custom widgets for the MViz control panel:
//! - Theme definitions (colors, fonts, styles)
//! - SensorPanel for displaying sensor data
//! - ControlBar for playback and connection controls
//! - DisplaysPanel for managing visualization displays
//! - PropertiesPanel for editing display properties
//! - Toolbar for main application toolbar

use makepad_widgets::*;

pub mod theme;
pub mod sensor_panel;
pub mod control_bar;
pub mod displays_panel;
pub mod properties_panel;
pub mod toolbar;

// Re-exports
pub use displays_panel::{DisplayInfo, DisplayListItem, DisplaysPanel};
pub use properties_panel::{Property, PropertyValue, PropertiesPanel};
pub use toolbar::{FrameSelector, Toolbar};

pub fn live_design(cx: &mut Cx) {
    theme::live_design(cx);
    sensor_panel::live_design(cx);
    control_bar::live_design(cx);
    displays_panel::live_design(cx);
    properties_panel::live_design(cx);
    toolbar::live_design(cx);
}
