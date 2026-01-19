//! System Log Panel Widget
//!
//! Displays system logs from robot nodes with filtering capabilities.
//! Features:
//! - Collapsible panel
//! - Filter by log level (Debug, Info, Warn, Error)
//! - Filter by node (dynamically populated)
//! - Text search
//! - Color-coded log entries
//! - Copy to clipboard

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Log entry item - modern tinted theme
    pub LogEntryItem = <View> {
        width: Fill, height: Fit
        flow: Down
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        show_bg: true
        draw_bg: {
            instance level: 0.0  // 0=debug, 1=info, 2=warn, 3=error
            fn pixel(self) -> vec4 {
                // Tinted backgrounds with more color
                let debug = vec4(0.95, 0.96, 0.97, 1.0);   // Light slate
                let info = vec4(0.90, 0.95, 1.0, 1.0);     // Light blue tint
                let warn = vec4(1.0, 0.96, 0.89, 1.0);     // Light amber tint
                let error = vec4(1.0, 0.92, 0.92, 1.0);    // Light red tint
                let color = mix(mix(debug, info, clamp(self.level, 0.0, 1.0)),
                               mix(warn, error, clamp(self.level - 2.0, 0.0, 1.0)),
                               clamp(self.level - 1.0, 0.0, 1.0) * 0.5 + clamp(self.level - 2.0, 0.0, 1.0) * 0.5);
                return color;
            }
        }

        // Header row: timestamp, level, node
        log_header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            align: {y: 0.5}

            // Timestamp - light theme
            timestamp = <Label> {
                width: 60
                text: "0.00s"
                draw_text: {
                    color: #9ca3af
                    text_style: { font_size: 9.0 }
                }
            }

            // Level badge - light theme
            level_badge = <View> {
                width: Fit, height: Fit
                padding: {left: 6, right: 6, top: 2, bottom: 2}
                show_bg: true
                draw_bg: {
                    instance level: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 3.0);
                        let debug = vec4(0.6, 0.6, 0.6, 0.2);
                        let info = vec4(0.23, 0.51, 0.96, 0.2);
                        let warn = vec4(0.96, 0.62, 0.04, 0.3);
                        let error = vec4(0.94, 0.27, 0.27, 0.3);
                        let color = mix(mix(debug, info, clamp(self.level, 0.0, 1.0)),
                                       mix(warn, error, clamp(self.level - 2.0, 0.0, 1.0)),
                                       clamp(self.level - 1.0, 0.0, 1.0) * 0.5 + clamp(self.level - 2.0, 0.0, 1.0) * 0.5);
                        sdf.fill(color);
                        return sdf.result;
                    }
                }

                level_text = <Label> {
                    text: "INFO"
                    draw_text: {
                        color: #374151
                        text_style: { font_size: 8.0 }
                    }
                }
            }

            // Node name - light theme
            node_name = <Label> {
                width: Fit
                text: "[node]"
                draw_text: {
                    color: #2564fb
                    text_style: { font_size: 9.0 }
                }
            }
        }

        // Message text - light theme
        message = <Label> {
            width: Fill
            text: "Log message"
            draw_text: {
                color: #374151
                text_style: { font_size: 10.0 }
                wrap: Word
            }
        }
    }

    // Complete log panel
    pub LogPanel = {{LogPanel}} <RoundedView> {
        width: Fill, height: 300
        flow: Down
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }

        // Header with toggle
        header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            padding: 12
            align: {y: 0.5}
            cursor: Hand

            // Collapse arrow - light theme
            collapse_icon = <View> {
                width: 16, height: 16
                show_bg: true
                draw_bg: {
                    instance collapsed: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        let cx = self.rect_size.x * 0.5;
                        let cy = self.rect_size.y * 0.5;
                        // Triangle pointing down (expanded) or right (collapsed)
                        if self.collapsed < 0.5 {
                            // Down arrow
                            sdf.move_to(4.0, 6.0);
                            sdf.line_to(12.0, 6.0);
                            sdf.line_to(8.0, 12.0);
                            sdf.close_path();
                        } else {
                            // Right arrow
                            sdf.move_to(6.0, 4.0);
                            sdf.line_to(12.0, 8.0);
                            sdf.line_to(6.0, 12.0);
                            sdf.close_path();
                        }
                        sdf.fill(vec4(0.42, 0.44, 0.48, 1.0)); // Gray (#6b7280)
                        return sdf.result;
                    }
                }
            }

            <Label> {
                text: "System Log"
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                }
            }

            <View> { width: Fill, height: 1 }

            // Entry count
            entry_count = <Label> {
                text: "0 entries"
                draw_text: {
                    color: (TEXT_MUTED)
                    text_style: { font_size: 10.0 }
                }
            }

            // Copy button - light theme
            copy_btn = <Button> {
                width: Fit, height: Fit
                text: "Copy"
                draw_text: { color: #6b7280 }
            }

            // Clear button - light theme
            clear_btn = <Button> {
                width: Fit, height: Fit
                text: "Clear"
                draw_text: { color: #6b7280 }
            }
        }

        // Filter row - light theme
        filter_row = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            padding: {left: 12, right: 12, bottom: 8}
            align: {y: 0.5}

            // Level filter
            <Label> {
                text: "Level:"
                draw_text: { color: #6b7280, text_style: { font_size: 10.0 } }
            }
            level_filter = <DropDown> {
                width: 80, height: 26
                labels: ["All", "Debug", "Info", "Warn", "Error"]
                values: [ALL, DEBUG, INFO, WARN, ERROR]
            }

            <View> { width: 12, height: 1 }

            // Node filter - light theme
            <Label> {
                text: "Node:"
                draw_text: { color: #6b7280, text_style: { font_size: 10.0 } }
            }
            node_filter = <DropDown> {
                width: 120, height: 26
                labels: ["All Nodes"]
                values: [ALL]
            }

            <View> { width: 12, height: 1 }

            // Search input - light theme
            <Label> {
                text: "Search:"
                draw_text: { color: #6b7280, text_style: { font_size: 10.0 } }
            }
            search_input = <TextInput> {
                width: Fill, height: 26
            }
        }

        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        // Log content area (scrollable) - light theme
        log_scroll = <ScrollYView> {
            width: Fill, height: Fill

            log_content = <Label> {
                width: Fill, height: Fit
                text: ""
                draw_text: {
                    color: #374151
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

/// Actions emitted by LogPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum LogPanelAction {
    None,
    CopyClicked,
    ClearClicked,
    ToggleCollapsed,
    LevelFilterChanged(usize),
    NodeFilterChanged(String),
    SearchChanged(String),
}

/// State for a log entry in the display
#[derive(Clone, Debug)]
pub struct LogDisplayEntry {
    pub timestamp: f64,
    pub level: u8,  // 0=debug, 1=info, 2=warn, 3=error
    pub level_str: String,
    pub node_id: String,
    pub message: String,
}

#[derive(Live, LiveHook, Widget)]
pub struct LogPanel {
    #[deref]
    view: View,
    #[rust] collapsed: bool,
    #[rust] entries: Vec<LogDisplayEntry>,
    #[rust] filtered_entries: Vec<usize>,  // Indices into entries
    #[rust] level_filter: usize,  // 0=all, 1=debug, 2=info, 3=warn, 4=error
    #[rust] node_filter: String,  // "" = all nodes
    #[rust] search_text: String,
    #[rust] discovered_nodes: Vec<String>,
    #[rust] max_entries: usize,
}

impl Widget for LogPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Capture actions from child widgets during event handling
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Handle level filter dropdown changes
        if let Some(index) = self.drop_down(id!(level_filter)).changed(&actions) {
            self.level_filter = index;
            self.apply_filters();
            self.update_log_content(cx);
            self.update_entry_count(cx);
            cx.widget_action(self.widget_uid(), &scope.path, LogPanelAction::LevelFilterChanged(index));
            self.redraw(cx);
        }

        // Handle node filter dropdown changes
        if let Some(index) = self.drop_down(id!(node_filter)).changed(&actions) {
            // index 0 = "All Nodes", index 1+ = specific node
            self.node_filter = if index == 0 {
                String::new()
            } else {
                self.discovered_nodes.get(index - 1).cloned().unwrap_or_default()
            };
            self.apply_filters();
            self.update_log_content(cx);
            self.update_entry_count(cx);
            cx.widget_action(self.widget_uid(), &scope.path, LogPanelAction::NodeFilterChanged(self.node_filter.clone()));
            self.redraw(cx);
        }

        // Handle header click for collapse/expand
        let header = self.view(id!(header));
        match event.hits(cx, header.area()) {
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    self.collapsed = !self.collapsed;
                    self.update_collapsed_state(cx);
                    cx.widget_action(self.widget_uid(), &scope.path, LogPanelAction::ToggleCollapsed);
                }
            }
            _ => {}
        }

        // Handle copy button
        let copy_btn = self.view(id!(header.copy_btn));
        match event.hits(cx, copy_btn.area()) {
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(self.widget_uid(), &scope.path, LogPanelAction::CopyClicked);
                }
            }
            _ => {}
        }

        // Handle clear button
        let clear_btn = self.view(id!(header.clear_btn));
        match event.hits(cx, clear_btn.area()) {
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(self.widget_uid(), &scope.path, LogPanelAction::ClearClicked);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Make sure entries are initialized
        if self.max_entries == 0 {
            self.init();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl LogPanel {
    /// Initialize with default values
    pub fn init(&mut self) {
        self.max_entries = 1000;
        self.entries = Vec::with_capacity(self.max_entries);
        self.filtered_entries = Vec::new();
        self.discovered_nodes = Vec::new();
        self.level_filter = 0;
        self.node_filter = String::new();
        self.search_text = String::new();
        self.collapsed = false;
    }

    /// Add a log entry
    pub fn add_entry(&mut self, cx: &mut Cx, entry: LogDisplayEntry) {
        // Make sure we're initialized
        if self.max_entries == 0 {
            self.init();
        }

        // Track discovered nodes
        if !self.discovered_nodes.contains(&entry.node_id) {
            self.discovered_nodes.push(entry.node_id.clone());
            self.discovered_nodes.sort();
            self.update_node_filter_dropdown(cx);
        }

        // Add entry
        self.entries.push(entry);

        // Trim old entries if over limit
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        // Reapply filters
        self.apply_filters();

        // Update UI - the log content and entry count
        self.update_log_content(cx);
        self.update_entry_count(cx);
        self.redraw(cx);
    }

    /// Clear all entries
    pub fn clear(&mut self, cx: &mut Cx) {
        self.entries.clear();
        self.filtered_entries.clear();
        self.update_log_content(cx);
        self.update_entry_count(cx);
        self.redraw(cx);
    }

    /// Set discovered nodes (from external source like ZenohReceiver)
    pub fn set_discovered_nodes(&mut self, cx: &mut Cx, nodes: Vec<String>) {
        self.discovered_nodes = nodes;
        self.discovered_nodes.sort();
        self.update_node_filter_dropdown(cx);
    }

    /// Get all entries as text (for copying)
    pub fn get_filtered_text(&self) -> String {
        self.filtered_entries
            .iter()
            .map(|&idx| {
                let e = &self.entries[idx];
                format!("[{:.2}s] [{}] [{}] {}", e.timestamp, e.level_str, e.node_id, e.message)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get filtered entry count
    pub fn filtered_count(&self) -> usize {
        self.filtered_entries.len()
    }

    fn apply_filters(&mut self) {
        self.filtered_entries.clear();

        let search_lower = self.search_text.to_lowercase();

        for (idx, entry) in self.entries.iter().enumerate() {
            // Level filter: 0=all, 1=debug, 2=info, 3=warn, 4=error
            let level_match = match self.level_filter {
                0 => true,
                1 => entry.level == 0,  // Debug
                2 => entry.level == 1,  // Info
                3 => entry.level == 2,  // Warn
                4 => entry.level == 3,  // Error
                _ => true,
            };

            // Node filter
            let node_match = self.node_filter.is_empty() || entry.node_id == self.node_filter;

            // Search filter
            let search_match = search_lower.is_empty()
                || entry.message.to_lowercase().contains(&search_lower)
                || entry.node_id.to_lowercase().contains(&search_lower);

            if level_match && node_match && search_match {
                self.filtered_entries.push(idx);
            }
        }
    }

    /// Update the log content label with all filtered entries
    fn update_log_content(&mut self, cx: &mut Cx) {
        // Build text from filtered entries (show newest first, limit to last 100)
        let text: String = self.filtered_entries.iter()
            .rev()
            .take(100)
            .map(|&idx| {
                let e = &self.entries[idx];
                format!("[{:.2}s] [{}] [{}] {}", e.timestamp, e.level_str, e.node_id, e.message)
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Update the label
        self.label(id!(log_content)).set_text(cx, &text);
    }

    fn update_collapsed_state(&mut self, cx: &mut Cx) {
        let collapsed_val = if self.collapsed { 1.0 } else { 0.0 };
        self.view(id!(header.collapse_icon)).apply_over(cx, live! {
            draw_bg: { collapsed: (collapsed_val) }
        });

        // Show/hide content
        if self.collapsed {
            self.view(id!(filter_row)).apply_over(cx, live! { visible: false });
            self.view(id!(log_scroll)).apply_over(cx, live! { visible: false });
        } else {
            self.view(id!(filter_row)).apply_over(cx, live! { visible: true });
            self.view(id!(log_scroll)).apply_over(cx, live! { visible: true });
        }

        self.redraw(cx);
    }

    fn update_entry_count(&mut self, cx: &mut Cx) {
        let count_text = if self.filtered_entries.len() == self.entries.len() {
            format!("{} entries", self.entries.len())
        } else {
            format!("{}/{} entries", self.filtered_entries.len(), self.entries.len())
        };
        self.label(id!(header.entry_count)).set_text(cx, &count_text);
    }

    fn update_node_filter_dropdown(&mut self, cx: &mut Cx) {
        // Build labels: "All Nodes" + discovered nodes
        let mut labels = vec!["All Nodes".to_string()];
        labels.extend(self.discovered_nodes.clone());

        // Update dropdown labels dynamically using Makepad's DropDown API
        self.drop_down(id!(node_filter)).set_labels(cx, labels);
        self.redraw(cx);
    }

    /// Check if collapsed
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Set collapsed state
    pub fn set_collapsed(&mut self, cx: &mut Cx, collapsed: bool) {
        self.collapsed = collapsed;
        self.update_collapsed_state(cx);
    }
}

impl LogPanelRef {
    /// Add a log entry
    pub fn add_entry(&self, cx: &mut Cx, entry: LogDisplayEntry) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_entry(cx, entry);
        }
    }

    /// Clear all entries
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }

    /// Get filtered text for copying
    pub fn get_filtered_text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.get_filtered_text()
        } else {
            String::new()
        }
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        if let Some(inner) = self.borrow() {
            inner.entry_count()
        } else {
            0
        }
    }

    /// Set discovered nodes
    pub fn set_discovered_nodes(&self, cx: &mut Cx, nodes: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_discovered_nodes(cx, nodes);
        }
    }
}
