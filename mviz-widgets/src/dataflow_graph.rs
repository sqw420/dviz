//! Dataflow Graph Widget
//!
//! Displays the Dora dataflow graph with nodes and connections.
//! Nodes are discovered dynamically from the bridge via Zenoh.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

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

        // Graph canvas area
        graph_canvas = <View> {
            width: Fill, height: Fill
            show_bg: true
            draw_bg: {
                // Grid background
                fn pixel(self) -> vec4 {
                    let grid_size = 20.0;
                    let pos = self.pos * self.rect_size;
                    let grid_x = mod(pos.x, grid_size);
                    let grid_y = mod(pos.y, grid_size);
                    let line_width = 1.0;

                    let base_color = vec4(0.12, 0.12, 0.12, 1.0);
                    let grid_color = vec4(0.16, 0.16, 0.16, 1.0);

                    if grid_x < line_width || grid_y < line_width {
                        return grid_color;
                    }
                    return base_color;
                }
            }

            // Graph content rendered as text (simple approach like LogPanel)
            graph_content = <Label> {
                width: Fill, height: Fit
                margin: 12
                text: "Waiting for graph data..."
                draw_text: {
                    color: (TEXT_SECONDARY)
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                    wrap: Word
                }
            }
        }
    }
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
            &format!("{} nodes", self.nodes.len()));

        // Render graph as text (simple approach)
        let graph_text = self.format_graph_text();
        self.view.label(id!(graph_content)).set_text(cx, &graph_text);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl DataflowGraphWidget {
    /// Format graph as text for display
    fn format_graph_text(&self) -> String {
        if self.nodes.is_empty() {
            return "Waiting for dataflow graph...\n\nConnect to a running Dora dataflow to see the node graph.".to_string();
        }

        let mut text = String::new();

        // Section: Nodes
        text.push_str("=== NODES ===\n\n");
        for node in &self.nodes {
            let status = if node.is_active { "[ACTIVE]" } else { "[idle]" };
            let selected = if self.selected_node.as_deref() == Some(&node.id) { " <<" } else { "" };
            text.push_str(&format!("  {} {}{}\n", status, node.id, selected));
        }

        // Section: Connections
        if !self.edges.is_empty() {
            text.push_str("\n=== CONNECTIONS ===\n\n");
            for edge in &self.edges {
                text.push_str(&format!("  {} / {} --> {} / {}\n",
                    edge.from_node, edge.from_port,
                    edge.to_node, edge.to_port));
            }
        }

        // Stats
        text.push_str(&format!("\n--- {} nodes, {} edges ---", self.nodes.len(), self.edges.len()));
        if self.last_update > 0.0 {
            text.push_str(&format!("\nLast update: {:.1}s", self.last_update));
        }

        text
    }

    /// Update graph from a GraphUpdate message
    pub fn update_from_graph_update(&mut self, cx: &mut Cx, nodes: Vec<(String, bool)>, edges: Vec<(String, String, String, String)>, timestamp: f64) {
        // Simple layout: arrange nodes in a column
        let node_width = 120.0;
        let node_height = 30.0;
        let padding = 20.0;
        let spacing = 50.0;

        self.nodes = nodes.iter().enumerate().map(|(i, (id, is_active))| {
            GraphDisplayNode {
                id: id.clone(),
                x: padding,
                y: padding + (i as f32) * (node_height + spacing),
                width: node_width,
                height: node_height,
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
