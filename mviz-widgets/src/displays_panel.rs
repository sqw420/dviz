//! Displays Panel Widget
//!
//! Provides a list of visualization displays with enable/disable toggles
//! and status indicators.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Single display list item
    pub DisplayListItem = {{DisplayListItem}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 10, bottom: 10}
        cursor: Hand
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance selected: 0.0
            fn pixel(self) -> vec4 {
                let base = vec4(0.15, 0.15, 0.15, 0.0);
                let hover_color = vec4(0.2, 0.2, 0.2, 1.0);
                let selected_color = vec4(0.23, 0.51, 0.96, 0.2);
                return mix(mix(base, hover_color, self.hover), selected_color, self.selected);
            }
        }

        // Enable checkbox
        enabled_check = <CheckBox> {
            width: Fit, height: Fit
        }

        // Display icon based on type
        display_icon = <View> {
            width: 18, height: 18
            show_bg: true
            draw_bg: {
                instance icon_type: 0.0  // 0=grid, 1=axes, 2=pointcloud, 3=marker, 4=tf
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let cx = self.rect_size.x * 0.5;
                    let cy = self.rect_size.y * 0.5;

                    // Grid icon (default)
                    if self.icon_type < 0.5 {
                        // Draw grid pattern
                        sdf.move_to(2.0, 2.0);
                        sdf.line_to(16.0, 2.0);
                        sdf.line_to(16.0, 16.0);
                        sdf.line_to(2.0, 16.0);
                        sdf.close_path();
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 1.0);
                        sdf.move_to(9.0, 2.0);
                        sdf.line_to(9.0, 16.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 0.5);
                        sdf.move_to(2.0, 9.0);
                        sdf.line_to(16.0, 9.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 0.5);
                    } else if self.icon_type < 1.5 {
                        // Axes icon - RGB arrows
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(4.0, 4.0);
                        sdf.stroke(vec4(0.0, 0.8, 0.0, 1.0), 1.5);
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(14.0, 14.0);
                        sdf.stroke(vec4(0.8, 0.0, 0.0, 1.0), 1.5);
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(10.0, 8.0);
                        sdf.stroke(vec4(0.0, 0.0, 0.8, 1.0), 1.5);
                    } else if self.icon_type < 2.5 {
                        // Point cloud icon - dots
                        sdf.circle(4.0, 5.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                        sdf.circle(9.0, 3.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                        sdf.circle(14.0, 6.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                        sdf.circle(6.0, 10.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                        sdf.circle(12.0, 12.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                        sdf.circle(8.0, 14.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, 1.0));
                    } else if self.icon_type < 3.5 {
                        // Marker icon - cube
                        sdf.box(3.0, 3.0, 12.0, 12.0, 2.0);
                        sdf.fill(vec4(0.9, 0.6, 0.2, 0.8));
                    } else {
                        // TF icon - tree structure
                        sdf.circle(9.0, 4.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, 1.0));
                        sdf.move_to(9.0, 6.0);
                        sdf.line_to(9.0, 10.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 1.0);
                        sdf.move_to(9.0, 10.0);
                        sdf.line_to(4.0, 14.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 1.0);
                        sdf.move_to(9.0, 10.0);
                        sdf.line_to(14.0, 14.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 1.0);
                        sdf.circle(4.0, 14.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, 1.0));
                        sdf.circle(14.0, 14.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, 1.0));
                    }

                    return sdf.result;
                }
            }
        }

        // Display name
        display_name = <Label> {
            width: Fill
            text: "Display"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        // Status indicator
        status_indicator = <View> {
            width: 8, height: 8
            show_bg: true
            draw_bg: {
                instance status: 0.0  // 0=ok, 1=warning, 2=error
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, self.rect_size.x * 0.5);
                    let ok = vec4(0.06, 0.73, 0.51, 1.0);
                    let warning = vec4(0.96, 0.62, 0.04, 1.0);
                    let error = vec4(0.94, 0.27, 0.27, 1.0);
                    let color = mix(mix(ok, warning, clamp(self.status, 0.0, 1.0)), error, clamp(self.status - 1.0, 0.0, 1.0));
                    sdf.fill(color);
                    return sdf.result;
                }
            }
        }
    }

    // Add display button
    pub AddDisplayButton = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 8, bottom: 8}
        cursor: Hand
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = vec4(0.18, 0.18, 0.18, 1.0);
                let hover_color = vec4(0.25, 0.25, 0.25, 1.0);
                sdf.fill(mix(base, hover_color, self.hover));
                return sdf.result;
            }
        }

        // Plus icon
        <View> {
            width: 18, height: 18
            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let cx = self.rect_size.x * 0.5;
                    let cy = self.rect_size.y * 0.5;
                    sdf.move_to(cx, 4.0);
                    sdf.line_to(cx, 14.0);
                    sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 2.0);
                    sdf.move_to(4.0, cy);
                    sdf.line_to(14.0, cy);
                    sdf.stroke(vec4(0.5, 0.5, 0.5, 1.0), 2.0);
                    return sdf.result;
                }
            }
        }

        <Label> {
            text: "Add Display"
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }
    }

    // Complete displays panel
    pub DisplaysPanel = {{DisplaysPanel}} <RoundedView> {
        width: Fill, height: Fit
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
            spacing: 8
            padding: 12
            align: {y: 0.5}

            <Label> {
                text: "Displays"
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                }
            }

            <View> { width: Fill, height: 1 }

            display_count = <Label> {
                text: "0"
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

        // Display list
        display_list = <PortalList> {
            width: Fill, height: Fit

            GridItem = <DisplayListItem> {
                display_icon = { draw_bg: { icon_type: 0.0 } }
                display_name = { text: "Grid" }
            }

            AxesItem = <DisplayListItem> {
                display_icon = { draw_bg: { icon_type: 1.0 } }
                display_name = { text: "Axes" }
            }

            PointCloudItem = <DisplayListItem> {
                display_icon = { draw_bg: { icon_type: 2.0 } }
                display_name = { text: "Point Cloud" }
            }

            MarkerItem = <DisplayListItem> {
                display_icon = { draw_bg: { icon_type: 3.0 } }
                display_name = { text: "Markers" }
            }

            TfItem = <DisplayListItem> {
                display_icon = { draw_bg: { icon_type: 4.0 } }
                display_name = { text: "TF" }
            }
        }

        // Add button
        add_section = <View> {
            width: Fill, height: Fit
            padding: 8

            add_btn = <AddDisplayButton> {}
        }
    }
}

// ============================================================================
// DISPLAY INFO STRUCT
// ============================================================================

/// Information about a display for the UI
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    /// Unique identifier
    pub id: u64,
    /// Display name
    pub name: String,
    /// Display type (grid, axes, point_cloud, marker, tf)
    pub display_type: String,
    /// Whether enabled
    pub enabled: bool,
    /// Status: 0=ok, 1=warning, 2=error
    pub status: u8,
    /// Status message
    pub status_message: Option<String>,
}

impl Default for DisplayInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Display".to_string(),
            display_type: "unknown".to_string(),
            enabled: true,
            status: 0,
            status_message: None,
        }
    }
}

// ============================================================================
// WIDGET STRUCTS
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct DisplayListItem {
    #[deref]
    view: View,
    #[rust] pub display_id: u64,
    #[rust] pub selected: bool,
}

impl Widget for DisplayListItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl DisplayListItem {
    pub fn set_display_info(&mut self, _cx: &mut Cx, info: &DisplayInfo) {
        self.display_id = info.id;
        // In a real implementation, we would update the UI elements here
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DisplaysPanel {
    #[deref]
    view: View,
    #[rust] displays: Vec<DisplayInfo>,
    #[rust] selected_index: Option<usize>,
}

impl Widget for DisplaysPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle Add Display button click
        let add_btn = self.view(id!(add_section.add_btn));
        match event.hits(cx, add_btn.area()) {
            Hit::FingerHoverIn(_) => {
                add_btn.apply_over(cx, live!{ draw_bg: { hover: 1.0 } });
                self.redraw(cx);
            }
            Hit::FingerHoverOut(_) => {
                add_btn.apply_over(cx, live!{ draw_bg: { hover: 0.0 } });
                self.redraw(cx);
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    // Emit action when clicked
                    cx.widget_action(self.widget_uid(), &scope.path, DisplaysPanelAction::AddDisplayClicked);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

// Action emitted by DisplaysPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum DisplaysPanelAction {
    None,
    AddDisplayClicked,
    DisplaySelected(u64),
}

impl DisplaysPanel {
    /// Set the list of displays to show
    pub fn set_displays(&mut self, displays: Vec<DisplayInfo>) {
        self.displays = displays;
    }

    /// Get the currently selected display ID
    pub fn selected_display_id(&self) -> Option<u64> {
        self.selected_index.map(|i| self.displays[i].id)
    }

    /// Select a display by index
    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// Get the number of displays
    pub fn display_count(&self) -> usize {
        self.displays.len()
    }
}
