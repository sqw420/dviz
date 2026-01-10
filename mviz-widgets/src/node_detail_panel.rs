//! Node Detail Panel Widget
//!
//! Displays detailed information about individual dataflow nodes:
//! - Node selector dropdown
//! - Live Input/Output message activity
//! - Filtered logs for selected node

use makepad_widgets::*;
use crate::log_panel::LogDisplayEntry;
use std::collections::VecDeque;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Node Detail Panel - replaces center panel content
    pub NodeDetailPanel = {{NodeDetailPanel}} <RoundedView> {
        width: Fill, height: Fill
        flow: Down
        padding: 16
        spacing: 12
        show_bg: true
        draw_bg: { color: #1e1e1e, border_radius: 8.0 }

        // Header with node selector
        header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 12
            align: {y: 0.5}

            <Label> {
                text: "NODE:"
                draw_text: { color: #888888, text_style: { font_size: 12.0 } }
            }

            node_selector = <DropDown> {
                width: 200, height: 28
                labels: ["Select Node..."]
            }

            <View> { width: Fill, height: 1 }

            status_indicator = <RoundedView> {
                width: 12, height: 12
                show_bg: true
                draw_bg: { color: #22c55e, border_radius: 6.0 }
            }

            status_label = <Label> {
                text: "Ready"
                draw_text: { color: #22c55e, text_style: { font_size: 11.0 } }
            }
        }

        // Separator
        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: #333333 }
        }

        // Input/Output columns
        io_section = <View> {
            width: Fill, height: 160
            flow: Right
            spacing: 16

            // Inputs column
            inputs_column = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 6

                <Label> {
                    text: "INPUTS:"
                    draw_text: { color: #fbbf24, text_style: { font_size: 11.0 } }
                }

                inputs_content = <Label> {
                    width: Fill, height: Fill
                    text: "  (select a node)"
                    draw_text: {
                        color: #888888
                        text_style: { font_size: 10.0 }
                        wrap: Word
                    }
                }
            }

            // Vertical separator
            <View> {
                width: 1, height: Fill
                show_bg: true
                draw_bg: { color: #333333 }
            }

            // Outputs column
            outputs_column = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 6

                <Label> {
                    text: "OUTPUTS:"
                    draw_text: { color: #60a5fa, text_style: { font_size: 11.0 } }
                }

                outputs_content = <Label> {
                    width: Fill, height: Fill
                    text: "  (select a node)"
                    draw_text: {
                        color: #888888
                        text_style: { font_size: 10.0 }
                        wrap: Word
                    }
                }
            }
        }

        // Separator
        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: #333333 }
        }

        // Logs section header
        logs_header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            align: {y: 0.5}

            <Label> {
                text: "NODE LOGS:"
                draw_text: { color: #a0a0a0, text_style: { font_size: 11.0 } }
            }

            log_count = <Label> {
                text: "0 entries"
                draw_text: { color: #606060, text_style: { font_size: 10.0 } }
            }

            <View> { width: Fill, height: 1 }

            clear_logs_btn = <Button> {
                width: Fit, height: 24
                padding: {left: 8, right: 8}
                text: "Clear"
                draw_text: { color: #888 }
            }
        }

        // Scrollable logs area
        logs_scroll = <ScrollYView> {
            width: Fill, height: Fill

            logs_content = <Label> {
                width: Fill, height: Fit
                text: "Select a node to view its logs"
                draw_text: {
                    color: #707070
                    text_style: { font_size: 10.0 }
                    wrap: Word
                }
            }
        }
    }
}

// ============================================================================
// WIDGET IMPLEMENTATION
// ============================================================================

/// Actions emitted by NodeDetailPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum NodeDetailPanelAction {
    None,
    NodeSelected(String),
    ClearLogsClicked,
}

/// Input port definition
#[derive(Clone, Debug)]
pub struct NodeInput {
    pub name: String,
    pub source: String,
}

/// Output port definition
#[derive(Clone, Debug)]
pub struct NodeOutput {
    pub name: String,
    pub destinations: Vec<String>,
}

/// I/O activity entry - a live message on a port
#[derive(Clone, Debug)]
pub struct IoActivityEntry {
    pub timestamp: f64,
    pub port_name: String,
    pub data_summary: String,
}

/// Node display state
#[derive(Clone, Debug)]
pub struct NodeDisplayState {
    pub id: String,
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutput>,
    /// Recent input activity messages
    pub input_activity: VecDeque<IoActivityEntry>,
    /// Recent output activity messages
    pub output_activity: VecDeque<IoActivityEntry>,
}

#[derive(Live, LiveHook, Widget)]
pub struct NodeDetailPanel {
    #[deref]
    view: View,

    /// All known nodes
    #[rust] nodes: Vec<NodeDisplayState>,

    /// Currently selected node ID
    #[rust] selected_node: Option<String>,

    /// Logs for selected node
    #[rust] node_logs: Vec<LogDisplayEntry>,

    /// All logs received (for filtering when node changes)
    #[rust] all_logs: Vec<LogDisplayEntry>,

    /// Maximum logs to keep
    #[rust] max_logs: usize,

    /// Discovered node IDs (from log messages)
    #[rust] discovered_nodes: Vec<String>,
}

impl Widget for NodeDetailPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Capture actions from child widgets
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Handle node selector dropdown
        if let Some(index) = self.drop_down(id!(node_selector)).changed(&actions) {
            if index == 0 {
                // "Select Node..." - deselect
                self.selected_node = None;
                self.node_logs.clear();
            } else {
                // Get node ID from discovered nodes (clone to avoid borrow issues)
                if let Some(node_id) = self.discovered_nodes.get(index - 1).cloned() {
                    self.selected_node = Some(node_id.clone());
                    // Filter logs for this node
                    self.filter_logs_for_node(&node_id);
                    cx.widget_action(self.widget_uid(), &scope.path,
                        NodeDetailPanelAction::NodeSelected(node_id));
                }
            }

            self.update_io_display(cx);
            self.update_logs_display(cx);
            self.update_status_display(cx);
            self.redraw(cx);
        }

        // Handle clear logs button
        if self.button(id!(clear_logs_btn)).clicked(&actions) {
            self.node_logs.clear();
            self.update_logs_display(cx);
            cx.widget_action(self.widget_uid(), &scope.path,
                NodeDetailPanelAction::ClearLogsClicked);
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Initialize if needed
        if self.max_logs == 0 {
            self.init();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl NodeDetailPanel {
    /// Initialize with default values
    pub fn init(&mut self) {
        self.max_logs = 500;
        self.nodes = Vec::new();
        self.selected_node = None;
        self.node_logs = Vec::new();
        self.all_logs = Vec::with_capacity(self.max_logs);
        self.discovered_nodes = Vec::new();
    }

    /// Add a discovered node
    pub fn add_discovered_node(&mut self, cx: &mut Cx, node_id: String) {
        if self.max_logs == 0 {
            self.init();
        }

        if !self.discovered_nodes.contains(&node_id) {
            self.discovered_nodes.push(node_id);
            self.discovered_nodes.sort();
            self.update_node_dropdown(cx);
        }
    }

    /// Set discovered nodes (bulk update)
    pub fn set_discovered_nodes(&mut self, cx: &mut Cx, nodes: Vec<String>) {
        if self.max_logs == 0 {
            self.init();
        }

        self.discovered_nodes = nodes;
        self.discovered_nodes.sort();
        self.update_node_dropdown(cx);
    }

    /// Add a log entry
    pub fn add_log(&mut self, cx: &mut Cx, entry: LogDisplayEntry) {
        if self.max_logs == 0 {
            self.init();
        }

        // Track node from log
        if !self.discovered_nodes.contains(&entry.node_id) {
            self.discovered_nodes.push(entry.node_id.clone());
            self.discovered_nodes.sort();
            self.update_node_dropdown(cx);
        }

        // Store in all_logs
        self.all_logs.push(entry.clone());
        if self.all_logs.len() > self.max_logs {
            self.all_logs.remove(0);
        }

        // If this log is for the selected node, add to display
        if let Some(ref selected) = self.selected_node {
            if entry.node_id == *selected {
                self.node_logs.push(entry);
                if self.node_logs.len() > self.max_logs {
                    self.node_logs.remove(0);
                }
                self.update_logs_display(cx);
            }
        }
    }

    /// Clear logs for current node
    pub fn clear_logs(&mut self, cx: &mut Cx) {
        self.node_logs.clear();
        self.update_logs_display(cx);
    }

    /// Set node definition (input/output ports)
    pub fn set_node_definition(&mut self, cx: &mut Cx, node_id: &str, inputs: Vec<NodeInput>, outputs: Vec<NodeOutput>) {
        if self.max_logs == 0 {
            self.init();
        }

        // Find or create node state
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.inputs = inputs;
            node.outputs = outputs;
        } else {
            self.nodes.push(NodeDisplayState {
                id: node_id.to_string(),
                inputs,
                outputs,
                input_activity: VecDeque::with_capacity(20),
                output_activity: VecDeque::with_capacity(20),
            });
        }

        // Update display if this is the selected node
        if self.selected_node.as_deref() == Some(node_id) {
            self.update_io_display(cx);
        }
    }

    /// Add I/O activity for a node (live message data)
    pub fn add_io_activity(&mut self, cx: &mut Cx, node_id: &str, port_name: &str, port_type: &str, timestamp: f64, data_summary: &str) {
        if self.max_logs == 0 {
            self.init();
        }

        // Ensure node exists
        if !self.nodes.iter().any(|n| n.id == node_id) {
            self.nodes.push(NodeDisplayState {
                id: node_id.to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                input_activity: VecDeque::with_capacity(20),
                output_activity: VecDeque::with_capacity(20),
            });
        }

        // Find the node and add activity
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            let entry = IoActivityEntry {
                timestamp,
                port_name: port_name.to_string(),
                data_summary: data_summary.to_string(),
            };

            let activity = if port_type == "input" {
                &mut node.input_activity
            } else {
                &mut node.output_activity
            };

            activity.push_back(entry);
            // Keep only last 15 messages
            while activity.len() > 15 {
                activity.pop_front();
            }
        }

        // Update display if this is the selected node
        if self.selected_node.as_deref() == Some(node_id) {
            self.update_io_display(cx);
            self.redraw(cx);
        }
    }

    fn filter_logs_for_node(&mut self, node_id: &str) {
        self.node_logs = self.all_logs.iter()
            .filter(|e| e.node_id == node_id)
            .cloned()
            .collect();
    }

    fn update_node_dropdown(&mut self, cx: &mut Cx) {
        let mut labels = vec!["Select Node...".to_string()];
        labels.extend(self.discovered_nodes.clone());
        self.drop_down(id!(node_selector)).set_labels(cx, labels);
        self.redraw(cx);
    }

    fn update_io_display(&mut self, cx: &mut Cx) {
        let Some(ref node_id) = self.selected_node else {
            self.label(id!(inputs_content)).set_text(cx, "  (select a node)");
            self.label(id!(outputs_content)).set_text(cx, "  (select a node)");
            return;
        };

        // Find node state
        let node = self.nodes.iter().find(|n| n.id == *node_id);

        // Build inputs text from live activity
        let inputs_text = if let Some(node) = node {
            if node.input_activity.is_empty() {
                "  (waiting for data...)".to_string()
            } else {
                node.input_activity.iter()
                    .rev()
                    .take(10)
                    .map(|entry| format!("[{:.2}] {}: {}", entry.timestamp, entry.port_name, entry.data_summary))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else {
            "  (waiting for data...)".to_string()
        };

        // Build outputs text from live activity
        let outputs_text = if let Some(node) = node {
            if node.output_activity.is_empty() {
                "  (waiting for data...)".to_string()
            } else {
                node.output_activity.iter()
                    .rev()
                    .take(10)
                    .map(|entry| format!("[{:.2}] {}: {}", entry.timestamp, entry.port_name, entry.data_summary))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else {
            "  (waiting for data...)".to_string()
        };

        self.label(id!(inputs_content)).set_text(cx, &inputs_text);
        self.label(id!(outputs_content)).set_text(cx, &outputs_text);
    }

    fn update_status_display(&mut self, cx: &mut Cx) {
        if self.selected_node.is_some() {
            self.label(id!(status_label)).set_text(cx, "Active");
        } else {
            self.label(id!(status_label)).set_text(cx, "Ready");
        }
    }

    fn update_logs_display(&mut self, cx: &mut Cx) {
        if self.selected_node.is_none() {
            self.label(id!(logs_content)).set_text(cx, "Select a node to view its logs");
            self.label(id!(log_count)).set_text(cx, "0 entries");
            return;
        }

        if self.node_logs.is_empty() {
            self.label(id!(logs_content)).set_text(cx, "No logs for this node yet");
            self.label(id!(log_count)).set_text(cx, "0 entries");
        } else {
            let logs_text = self.node_logs.iter()
                .rev()
                .take(100)
                .map(|entry| {
                    let level_prefix = match entry.level {
                        0 => "   ",  // Debug
                        1 => "   ",  // Info
                        2 => " ! ",  // Warn
                        3 => " X ",  // Error
                        _ => "   ",
                    };
                    format!("[{:.3}]{} {}", entry.timestamp, level_prefix, entry.message)
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.label(id!(logs_content)).set_text(cx, &logs_text);
            self.label(id!(log_count)).set_text(cx, &format!("{} entries", self.node_logs.len()));
        }
    }

    /// Get selected node ID
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_deref()
    }
}

// Widget reference implementation for external access
impl NodeDetailPanelRef {
    pub fn add_discovered_node(&self, cx: &mut Cx, node_id: String) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_discovered_node(cx, node_id);
        }
    }

    pub fn set_discovered_nodes(&self, cx: &mut Cx, nodes: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_discovered_nodes(cx, nodes);
        }
    }

    pub fn add_log(&self, cx: &mut Cx, entry: LogDisplayEntry) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_log(cx, entry);
        }
    }

    pub fn clear_logs(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear_logs(cx);
        }
    }

    pub fn set_node_definition(&self, cx: &mut Cx, node_id: &str, inputs: Vec<NodeInput>, outputs: Vec<NodeOutput>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_node_definition(cx, node_id, inputs, outputs);
        }
    }

    pub fn add_io_activity(&self, cx: &mut Cx, node_id: &str, port_name: &str, port_type: &str, timestamp: f64, data_summary: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_io_activity(cx, node_id, port_name, port_type, timestamp, data_summary);
        }
    }

    pub fn selected_node(&self) -> Option<String> {
        if let Some(inner) = self.borrow() {
            inner.selected_node().map(|s| s.to_string())
        } else {
            None
        }
    }
}
