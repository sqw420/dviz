//! Dataflow Graph Widget
//!
//! Displays the Dora dataflow graph with visual node boxes and edge lines.
//! Nodes are discovered dynamically from the bridge via Zenoh.

use makepad_widgets::*;
use std::collections::HashMap;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Node box shader
    pub DrawNodeBox = {{DrawNodeBox}} {
        fn pixel(self) -> vec4 {
            let sdf = Sdf2d::viewport(self.pos * self.rect_size);

            // Rounded rectangle for node
            sdf.box(
                1.0,
                1.0,
                self.rect_size.x - 2.0,
                self.rect_size.y - 2.0,
                4.0
            );

            // Fill color based on active state - light theme
            let fill_color = mix(
                vec4(0.96, 0.98, 0.99, 1.0),  // Idle: light gray (#f5f7fc)
                vec4(0.85, 0.97, 0.92, 1.0),  // Active: light green tint
                self.is_active
            );

            // Border color based on selection
            let border_color = mix(
                vec4(0.80, 0.83, 0.86, 1.0),  // Normal border (#ccd4dc)
                vec4(0.23, 0.51, 0.96, 1.0),  // Selected: bright blue (#3b82f6)
                self.is_selected
            );

            sdf.fill_keep(fill_color);
            sdf.stroke(border_color, 1.5);

            return sdf.result;
        }
    }

    // Edge line shader
    pub DrawEdgeLine = {{DrawEdgeLine}} {
        fn pixel(self) -> vec4 {
            let sdf = Sdf2d::viewport(self.pos * self.rect_size);

            // Simple horizontal line - light theme
            let mid_y = self.rect_size.y * 0.5;
            sdf.rect(0.0, mid_y - 0.5, self.rect_size.x, 1.0);
            sdf.fill(vec4(0.65, 0.70, 0.75, 0.8));  // Light gray edge

            return sdf.result;
        }
    }

    // Dataflow Graph Widget
    pub DataflowGraphWidget = {{DataflowGraphWidget}} <RoundedView> {
        width: Fill, height: Fill
        flow: Down
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }

        // Header
        header = <View> {
            width: Fill, height: Fit
            flow: Right
            padding: 12
            spacing: 8
            align: {y: 0.5}

            <Label> {
                text: "Dataflow Graph"
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                }
            }

            <View> { width: Fill, height: 1 }

            node_count = <Label> {
                text: "0 nodes"
                draw_text: {
                    color: (TEXT_MUTED)
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                }
            }
        }

        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        // Graph canvas area - scrollable for large graphs
        graph_canvas = <View> {
            width: Fill, height: Fill
            show_bg: true
            draw_bg: {
                // Grid background - modern tinted theme
                fn pixel(self) -> vec4 {
                    let grid_size = 20.0;
                    let pos = self.pos * self.rect_size;
                    let grid_x = mod(pos.x, grid_size);
                    let grid_y = mod(pos.y, grid_size);

                    let base_color = vec4(0.95, 0.97, 0.99, 1.0);   // Tinted slate (#f2f6fc)
                    let grid_color = vec4(0.88, 0.91, 0.95, 1.0);   // Subtle grid (#e0e8f2)

                    if grid_x < 1.0 || grid_y < 1.0 {
                        return grid_color;
                    }
                    return base_color;
                }
            }
            scroll_bars: <ScrollBars> {}

            // Graph content rendered as styled text list
            graph_content = <Label> {
                width: Fit, height: Fit
                margin: 12
                text: "Waiting for graph data..."
                draw_text: {
                    color: (TEXT_SECONDARY)
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                    wrap: Line
                }
            }
        }
    }
}

// ============================================================================
// DRAW SHADERS
// ============================================================================

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawNodeBox {
    #[deref] draw_super: DrawQuad,
    #[live] is_active: f32,
    #[live] is_selected: f32,
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawEdgeLine {
    #[deref] draw_super: DrawQuad,
}

// ============================================================================
// GRAPH DATA STRUCTURES
// ============================================================================

/// A node in the graph display
#[derive(Clone, Debug)]
pub struct GraphDisplayNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_active: bool,
}

/// An edge in the graph display
#[derive(Clone, Debug)]
pub struct GraphDisplayEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

// ============================================================================
// WIDGET IMPLEMENTATION
// ============================================================================

/// Actions emitted by DataflowGraphWidget
#[derive(Clone, Debug, DefaultNone)]
pub enum DataflowGraphAction {
    None,
    /// User clicked on a node
    NodeClicked(String),
}

#[derive(Live, LiveHook, Widget)]
pub struct DataflowGraphWidget {
    #[deref]
    view: View,

    /// Nodes in the graph
    #[rust] nodes: Vec<GraphDisplayNode>,

    /// Edges in the graph
    #[rust] edges: Vec<GraphDisplayEdge>,

    /// Selected node ID
    #[rust] selected_node: Option<String>,

    /// Last update timestamp
    #[rust] last_update: f64,
}

impl Widget for DataflowGraphWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle clicks on graph canvas for node selection
        let canvas = self.view(id!(graph_canvas));
        if let Hit::FingerUp(fe) = event.hits(cx, canvas.area()) {
            if fe.is_over {
                // Find if a node was clicked based on position
                let click_x = fe.abs.x - canvas.area().rect(cx).pos.x;
                let click_y = fe.abs.y - canvas.area().rect(cx).pos.y;

                for node in &self.nodes {
                    if click_x >= node.x as f64 && click_x <= (node.x + node.width) as f64 &&
                       click_y >= node.y as f64 && click_y <= (node.y + node.height) as f64 {
                        self.selected_node = Some(node.id.clone());
                        cx.widget_action(self.widget_uid(), &scope.path,
                            DataflowGraphAction::NodeClicked(node.id.clone()));
                        self.redraw(cx);
                        break;
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update node count label
        self.view.label(id!(node_count)).set_text(cx,
            &format!("{} nodes, {} edges", self.nodes.len(), self.edges.len()));

        // Render graph as visual ASCII art style
        let graph_text = self.format_visual_graph();
        self.view.label(id!(graph_content)).set_text(cx, &graph_text);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl DataflowGraphWidget {
    /// Format graph as visual ASCII-style diagram
    fn format_visual_graph(&self) -> String {
        if self.nodes.is_empty() {
            return "Waiting for dataflow graph...\n\nConnect to a running Dora dataflow to see the node graph.".to_string();
        }

        let mut text = String::new();

        // Build node position map for edge rendering
        let mut node_positions: HashMap<String, usize> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            node_positions.insert(node.id.clone(), i);
        }

        // Group edges by target node for better visualization
        let mut incoming_edges: HashMap<String, Vec<(&str, &str)>> = HashMap::new();
        for edge in &self.edges {
            incoming_edges
                .entry(edge.to_node.clone())
                .or_default()
                .push((&edge.from_node, &edge.from_port));
        }

        // Draw each node with ASCII box
        for node in &self.nodes {
            // Status indicator - use colored emoji for visibility
            let status_icon = if node.is_active { "🟢" } else { "⚪" };
            let status_text = if node.is_active { "RUN" } else { "---" };

            // Selection marker
            let selected = if self.selected_node.as_deref() == Some(&node.id) {
                " ◀"
            } else {
                ""
            };

            // Draw incoming edges as arrows
            if let Some(sources) = incoming_edges.get(&node.id) {
                for (from_node, from_port) in sources {
                    text.push_str(&format!("     {} / {}\n", from_node, from_port));
                    text.push_str("           ↓\n");
                }
            }

            // Draw node box
            let name_len = node.id.len().max(8);
            let box_width = name_len + 8;
            let border: String = "─".repeat(box_width);

            text.push_str(&format!("  ┌{}┐\n", border));
            text.push_str(&format!("  │ {} {:<width$} [{}]{} │\n",
                status_icon,
                node.id,
                status_text,
                selected,
                width = name_len));
            text.push_str(&format!("  └{}┘\n", border));

            // Show outgoing connections count
            let outgoing: Vec<_> = self.edges.iter()
                .filter(|e| e.from_node == node.id)
                .collect();
            if !outgoing.is_empty() {
                text.push_str(&format!("     ↓ {} output(s)\n", outgoing.len()));
            }

            text.push_str("\n");
        }

        // Stats footer
        text.push_str(&format!("━━━ {} nodes, {} edges ━━━", self.nodes.len(), self.edges.len()));
        if self.last_update > 0.0 {
            text.push_str(&format!("\nUpdated: {:.1}s ago", self.last_update));
        }

        text
    }

    /// Compute layout for nodes (hierarchical based on edges)
    fn compute_layout(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // Build dependency graph to determine levels
        let mut levels: HashMap<String, usize> = HashMap::new();
        let mut node_set: std::collections::HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();

        // Find source nodes (no incoming edges)
        let targets: std::collections::HashSet<_> = self.edges.iter().map(|e| e.to_node.clone()).collect();
        let sources: Vec<_> = node_set.iter()
            .filter(|n| !targets.contains(*n))
            .cloned()
            .collect();

        // BFS to assign levels
        let mut queue: Vec<(String, usize)> = sources.iter().map(|s| (s.clone(), 0)).collect();
        while let Some((node_id, level)) = queue.pop() {
            let current_level = levels.entry(node_id.clone()).or_insert(level);
            if level > *current_level {
                *current_level = level;
            }

            // Find successors
            for edge in &self.edges {
                if edge.from_node == node_id {
                    queue.push((edge.to_node.clone(), level + 1));
                }
            }
            node_set.remove(&node_id);
        }

        // Assign remaining nodes (disconnected) to level 0
        for node in &node_set {
            levels.insert(node.clone(), 0);
        }

        // Group nodes by level
        let mut level_nodes: HashMap<usize, Vec<String>> = HashMap::new();
        for (node_id, level) in &levels {
            level_nodes.entry(*level).or_default().push(node_id.clone());
        }

        // Compute positions
        let node_width = 140.0_f32;
        let node_height = 36.0_f32;
        let h_spacing = 180.0_f32;
        let v_spacing = 60.0_f32;
        let padding = 20.0_f32;

        let max_level = levels.values().max().copied().unwrap_or(0);
        for node in &mut self.nodes {
            if let Some(&level) = levels.get(&node.id) {
                let nodes_at_level = level_nodes.get(&level).map(|v| v.len()).unwrap_or(1);
                let idx = level_nodes.get(&level)
                    .and_then(|v| v.iter().position(|n| n == &node.id))
                    .unwrap_or(0);

                node.width = node_width;
                node.height = node_height;
                node.x = padding + (level as f32) * h_spacing;
                node.y = padding + (idx as f32) * v_spacing;
            }
        }
    }

    /// Update graph from a GraphUpdate message
    pub fn update_from_graph_update(&mut self, cx: &mut Cx, nodes: Vec<(String, bool)>, edges: Vec<(String, String, String, String)>, timestamp: f64) {
        // Create nodes
        self.nodes = nodes.iter().map(|(id, is_active)| {
            GraphDisplayNode {
                id: id.clone(),
                x: 0.0,
                y: 0.0,
                width: 140.0,
                height: 36.0,
                is_active: *is_active,
            }
        }).collect();

        self.edges = edges.iter().map(|(from_node, from_port, to_node, to_port)| {
            GraphDisplayEdge {
                from_node: from_node.clone(),
                from_port: from_port.clone(),
                to_node: to_node.clone(),
                to_port: to_port.clone(),
            }
        }).collect();

        // Compute visual layout
        self.compute_layout();

        self.last_update = timestamp;
        self.redraw(cx);
    }

    /// Get the selected node ID
    pub fn selected_node(&self) -> Option<&str> {
        self.selected_node.as_deref()
    }

    /// Clear the graph
    pub fn clear(&mut self, cx: &mut Cx) {
        self.nodes.clear();
        self.edges.clear();
        self.selected_node = None;
        self.last_update = 0.0;
        self.redraw(cx);
    }
}

// ============================================================================
// WIDGET REF EXTENSIONS
// ============================================================================

impl DataflowGraphWidgetRef {
    /// Update graph from a GraphUpdate message
    pub fn update_from_graph_update(&self, cx: &mut Cx, nodes: Vec<(String, bool)>, edges: Vec<(String, String, String, String)>, timestamp: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_from_graph_update(cx, nodes, edges, timestamp);
        }
    }

    /// Get the selected node ID
    pub fn selected_node(&self) -> Option<String> {
        if let Some(inner) = self.borrow() {
            inner.selected_node().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Clear the graph
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }
}
