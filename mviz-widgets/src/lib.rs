//! MViz Widgets - Reusable Makepad UI components
//!
//! This crate provides custom widgets for the MViz control panel:
//! - Theme definitions (colors, fonts, styles)
//! - SensorPanel for displaying sensor data
//! - ControlBar for playback and connection controls
//! - DisplaysPanel for managing visualization displays
//! - PropertiesPanel for editing display properties
//! - Toolbar for main application toolbar
//! - LogPanel for system log display with filtering
//! - NodeDetailPanel for node inspection with I/O and logs
//! - DataflowGraphWidget for visualizing the Dora dataflow graph

use makepad_widgets::*;

pub mod theme;
pub mod sensor_panel;
pub mod control_bar;
pub mod displays_panel;
pub mod properties_panel;
pub mod toolbar;
pub mod log_panel;
pub mod node_detail_panel;
pub mod dataflow_graph;

// Re-exports
pub use displays_panel::{DisplayInfo, DisplayType, DisplayListItem, DisplaysPanel, DisplaysPanelAction, DisplaysPanelDisplayOps, DisplaysPanelWidgetRefExt};
pub use properties_panel::{Property, PropertyValue, PropertiesPanel, PropertiesPanelRef, PropertiesPanelWidgetRefExt};
pub use toolbar::{FrameSelector, Toolbar};
pub use log_panel::{LogPanel, LogPanelAction, LogDisplayEntry, LogPanelRef, LogPanelWidgetRefExt};
pub use node_detail_panel::{NodeDetailPanel, NodeDetailPanelAction, NodeDetailPanelRef, NodeDetailPanelWidgetRefExt, NodeInput, NodeOutput};
pub use dataflow_graph::{DataflowGraphWidget, DataflowGraphAction, DataflowGraphWidgetRef};

pub fn live_design(cx: &mut Cx) {
    theme::live_design(cx);
    sensor_panel::live_design(cx);
    control_bar::live_design(cx);
    displays_panel::live_design(cx);
    properties_panel::live_design(cx);
    toolbar::live_design(cx);
    log_panel::live_design(cx);
    node_detail_panel::live_design(cx);
    dataflow_graph::live_design(cx);
}
