//! MViz Widgets - Reusable Makepad UI components
//!
//! This crate provides custom widgets for the MViz control panel:
//! - Theme definitions (colors, fonts, styles)
//! - SensorPanel for displaying sensor data
//! - ControlBar for playback and connection controls

use makepad_widgets::*;

pub mod theme;
pub mod sensor_panel;
pub mod control_bar;

pub fn live_design(cx: &mut Cx) {
    theme::live_design(cx);
    sensor_panel::live_design(cx);
    control_bar::live_design(cx);
}
