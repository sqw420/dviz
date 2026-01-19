//! Displays Panel Widget
//!
//! Provides a list of visualization displays with enable/disable toggles
//! and status indicators. Similar to RViz's Displays panel.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Single display list item - modern tinted theme
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
                let base = vec4(0.97, 0.98, 0.99, 1.0);        // Slightly tinted (#f8fafc)
                let hover_color = vec4(0.88, 0.91, 0.94, 1.0); // Slate-200 hover (#e2e8f0)
                let selected_color = vec4(0.85, 0.92, 0.98, 1.0); // Light blue selected
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
                instance icon_type: 0.0  // 0=grid, 1=axes, 2=pointcloud, 3=marker, 4=tf, 5=laser, 6=path, 7=pose
                instance enabled: 1.0    // 0=disabled (grayed out), 1=enabled
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let cx = self.rect_size.x * 0.5;
                    let cy = self.rect_size.y * 0.5;
                    let alpha = mix(0.3, 1.0, self.enabled);

                    // Grid icon (default)
                    if self.icon_type < 0.5 {
                        // Draw grid pattern
                        sdf.move_to(2.0, 2.0);
                        sdf.line_to(16.0, 2.0);
                        sdf.line_to(16.0, 16.0);
                        sdf.line_to(2.0, 16.0);
                        sdf.close_path();
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 1.0);
                        sdf.move_to(9.0, 2.0);
                        sdf.line_to(9.0, 16.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 0.5);
                        sdf.move_to(2.0, 9.0);
                        sdf.line_to(16.0, 9.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 0.5);
                    } else if self.icon_type < 1.5 {
                        // Axes icon - RGB arrows
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(4.0, 4.0);
                        sdf.stroke(vec4(0.0, 0.8, 0.0, alpha), 1.5);
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(14.0, 14.0);
                        sdf.stroke(vec4(0.8, 0.0, 0.0, alpha), 1.5);
                        sdf.move_to(4.0, 14.0);
                        sdf.line_to(10.0, 8.0);
                        sdf.stroke(vec4(0.0, 0.0, 0.8, alpha), 1.5);
                    } else if self.icon_type < 2.5 {
                        // Point cloud icon - dots
                        sdf.circle(4.0, 5.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                        sdf.circle(9.0, 3.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                        sdf.circle(14.0, 6.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                        sdf.circle(6.0, 10.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                        sdf.circle(12.0, 12.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                        sdf.circle(8.0, 14.0, 1.5);
                        sdf.fill(vec4(0.3, 0.7, 0.9, alpha));
                    } else if self.icon_type < 3.5 {
                        // Marker icon - cube
                        sdf.box(3.0, 3.0, 12.0, 12.0, 2.0);
                        sdf.fill(vec4(0.9, 0.6, 0.2, alpha * 0.8));
                    } else if self.icon_type < 4.5 {
                        // TF icon - tree structure
                        sdf.circle(9.0, 4.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, alpha));
                        sdf.move_to(9.0, 6.0);
                        sdf.line_to(9.0, 10.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 1.0);
                        sdf.move_to(9.0, 10.0);
                        sdf.line_to(4.0, 14.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 1.0);
                        sdf.move_to(9.0, 10.0);
                        sdf.line_to(14.0, 14.0);
                        sdf.stroke(vec4(0.5, 0.5, 0.5, alpha), 1.0);
                        sdf.circle(4.0, 14.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, alpha));
                        sdf.circle(14.0, 14.0, 2.0);
                        sdf.fill(vec4(0.7, 0.7, 0.7, alpha));
                    } else if self.icon_type < 5.5 {
                        // LaserScan icon - fan pattern
                        sdf.move_to(9.0, 14.0);
                        sdf.line_to(2.0, 4.0);
                        sdf.stroke(vec4(0.0, 1.0, 0.5, alpha), 1.0);
                        sdf.move_to(9.0, 14.0);
                        sdf.line_to(9.0, 2.0);
                        sdf.stroke(vec4(0.0, 1.0, 0.5, alpha), 1.0);
                        sdf.move_to(9.0, 14.0);
                        sdf.line_to(16.0, 4.0);
                        sdf.stroke(vec4(0.0, 1.0, 0.5, alpha), 1.0);
                    } else if self.icon_type < 6.5 {
                        // Path icon - curved line
                        sdf.move_to(2.0, 14.0);
                        sdf.line_to(6.0, 8.0);
                        sdf.stroke(vec4(0.4, 0.8, 1.0, alpha), 1.5);
                        sdf.move_to(6.0, 8.0);
                        sdf.line_to(12.0, 10.0);
                        sdf.stroke(vec4(0.4, 0.8, 1.0, alpha), 1.5);
                        sdf.move_to(12.0, 10.0);
                        sdf.line_to(16.0, 4.0);
                        sdf.stroke(vec4(0.4, 0.8, 1.0, alpha), 1.5);
                    } else {
                        // Pose icon - arrow
                        sdf.move_to(4.0, 12.0);
                        sdf.line_to(14.0, 6.0);
                        sdf.stroke(vec4(1.0, 0.4, 0.4, alpha), 2.0);
                        sdf.move_to(14.0, 6.0);
                        sdf.line_to(10.0, 6.0);
                        sdf.stroke(vec4(1.0, 0.4, 0.4, alpha), 1.5);
                        sdf.move_to(14.0, 6.0);
                        sdf.line_to(14.0, 10.0);
                        sdf.stroke(vec4(1.0, 0.4, 0.4, alpha), 1.5);
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

    // Add display button - modern tinted theme with blue accent
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
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                let base = vec4(0.88, 0.92, 0.96, 1.0);      // Light blue-gray (#e0eaf5)
                let hover_color = vec4(0.80, 0.87, 0.93, 1.0); // Darker blue on hover
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
                    sdf.stroke(vec4(0.42, 0.44, 0.48, 1.0), 2.0); // Gray (#6b7280)
                    sdf.move_to(4.0, cy);
                    sdf.line_to(14.0, cy);
                    sdf.stroke(vec4(0.42, 0.44, 0.48, 1.0), 2.0);
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

        // Display list content - rendered as text (simpler than PortalList)
        display_list_scroll = <ScrollYView> {
            width: Fill, height: 180

            display_list_content = <Label> {
                width: Fill, height: Fit
                text: ""
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                }
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
// DISPLAY TYPES
// ============================================================================

/// Display type enumeration with icon mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayType {
    #[default]
    Grid,
    Axes,
    PointCloud,
    Markers,
    TF,
    LaserScan,
    Path,
    Pose,
}

impl DisplayType {
    /// Get icon type for shader (maps to icon_type instance)
    pub fn icon_type(&self) -> f32 {
        match self {
            Self::Grid => 0.0,
            Self::Axes => 1.0,
            Self::PointCloud => 2.0,
            Self::Markers => 3.0,
            Self::TF => 4.0,
            Self::LaserScan => 5.0,
            Self::Path => 6.0,
            Self::Pose => 7.0,
        }
    }

    /// Get display type from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "grid" => Self::Grid,
            "axes" => Self::Axes,
            "pointcloud" | "point_cloud" | "points" => Self::PointCloud,
            "markers" | "marker" => Self::Markers,
            "tf" | "transform" => Self::TF,
            "laserscan" | "laser_scan" | "laser" => Self::LaserScan,
            "path" | "trajectory" => Self::Path,
            "pose" | "odometry" => Self::Pose,
            _ => Self::Markers,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Grid => "Grid",
            Self::Axes => "Axes",
            Self::PointCloud => "Point Cloud",
            Self::Markers => "Markers",
            Self::TF => "TF",
            Self::LaserScan => "LaserScan",
            Self::Path => "Path",
            Self::Pose => "Pose",
        }
    }

    /// Get all display types for the "Add Display" menu
    pub fn all() -> &'static [DisplayType] {
        &[
            Self::Grid,
            Self::Axes,
            Self::PointCloud,
            Self::Markers,
            Self::TF,
            Self::LaserScan,
            Self::Path,
            Self::Pose,
        ]
    }
}

impl std::fmt::Display for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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
    /// Display name (user-editable)
    pub name: String,
    /// Display type
    pub display_type: DisplayType,
    /// Whether enabled (visible in Rerun)
    pub enabled: bool,
    /// Status: 0=ok, 1=warning, 2=error
    pub status: u8,
    /// Status message
    pub status_message: Option<String>,
    /// Topic/entity path this display subscribes to
    pub topic: Option<String>,
    /// Color for rendering [r, g, b, a]
    pub color: [u8; 4],
    /// Alpha transparency (0.0 - 1.0)
    pub alpha: f32,
}

impl Default for DisplayInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Display".to_string(),
            display_type: DisplayType::Markers,
            enabled: true,
            status: 0,
            status_message: None,
            topic: None,
            color: [255, 255, 255, 255],
            alpha: 1.0,
        }
    }
}

impl DisplayInfo {
    /// Create a new display with the given type
    pub fn new(id: u64, display_type: DisplayType) -> Self {
        Self {
            id,
            name: display_type.name().to_string(),
            display_type,
            enabled: true,
            status: 0,
            status_message: None,
            topic: None,
            color: [255, 255, 255, 255],
            alpha: 1.0,
        }
    }

    /// Create with custom name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Create with topic
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Create with color
    pub fn with_color(mut self, color: [u8; 4]) -> Self {
        self.color = color;
        self
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
    #[rust] pub display_type: DisplayType,
    #[rust] pub enabled: bool,
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
    pub fn set_display_info(&mut self, cx: &mut Cx, info: &DisplayInfo) {
        self.display_id = info.id;
        self.display_type = info.display_type;
        self.enabled = info.enabled;

        // Update checkbox state - Makepad CheckBox uses selected property
        let checkbox = self.view.check_box(id!(enabled_check));
        if info.enabled {
            checkbox.apply_over(cx, live!{ animator: { selected: { default: on } } });
        } else {
            checkbox.apply_over(cx, live!{ animator: { selected: { default: off } } });
        }

        // Update icon type and enabled state
        let icon_type = info.display_type.icon_type();
        let enabled_val = if info.enabled { 1.0 } else { 0.0 };
        self.view(id!(display_icon)).apply_over(cx, live!{
            draw_bg: { icon_type: (icon_type), enabled: (enabled_val) }
        });

        // Update name
        self.view.label(id!(display_name)).set_text(cx, &info.name);

        // Update status indicator
        let status = info.status as f32;
        self.view(id!(status_indicator)).apply_over(cx, live!{
            draw_bg: { status: (status) }
        });

        // Update selection visual
        let selected_val = if self.selected { 1.0 } else { 0.0 };
        self.view.apply_over(cx, live!{
            draw_bg: { selected: (selected_val) }
        });
    }

    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        self.selected = selected;
        let selected_val = if selected { 1.0 } else { 0.0 };
        self.view.apply_over(cx, live!{
            draw_bg: { selected: (selected_val) }
        });
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DisplaysPanel {
    #[deref]
    view: View,
    #[rust] displays: Vec<DisplayInfo>,
    #[rust] selected_index: Option<usize>,
    #[rust] next_id: u64,
    #[rust] initialized: bool,
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

        // Handle display list item clicks
        let display_list = self.view.label(id!(display_list_content));
        match event.hits(cx, display_list.area()) {
            Hit::FingerUp(fe) => {
                if fe.is_over && !self.displays.is_empty() {
                    // Calculate which display was clicked based on Y position
                    // Line height is approximately 18px for font_size 11.0
                    let line_height = 18.0;
                    let clicked_index = (fe.abs.y - display_list.area().rect(cx).pos.y) / line_height;
                    let index = clicked_index as usize;

                    if index < self.displays.len() {
                        self.selected_index = Some(index);
                        let display_id = self.displays[index].id;
                        cx.widget_action(self.widget_uid(), &scope.path, DisplaysPanelAction::DisplaySelected(display_id));
                        self.redraw(cx);
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Initialize with default displays if not yet done
        if !self.initialized {
            self.initialized = true;
            self.init_default_displays();
        }

        // Update display count label
        self.view.label(id!(display_count)).set_text(cx, &self.displays.len().to_string());

        // Render display list as formatted text (similar to LogPanel approach)
        let display_text = self.format_display_list();
        self.view.label(id!(display_list_content)).set_text(cx, &display_text);

        self.view.draw_walk(cx, scope, walk)
    }
}

// Action emitted by DisplaysPanel
#[derive(Clone, Debug, DefaultNone)]
pub enum DisplaysPanelAction {
    None,
    /// User clicked "Add Display" button
    AddDisplayClicked,
    /// User selected a display (id)
    DisplaySelected(u64),
    /// User toggled a display's enabled state (id, enabled)
    DisplayToggled(u64, bool),
    /// User deleted a display (id)
    DisplayDeleted(u64),
}

impl DisplaysPanel {
    /// Format display list as text for rendering
    fn format_display_list(&self) -> String {
        self.displays.iter().enumerate().map(|(idx, d)| {
            let checkbox = if d.enabled { "[x]" } else { "[ ]" };
            let selected = if self.selected_index == Some(idx) { " >" } else { "  " };
            let status = match d.status {
                0 => "OK",
                1 => "WARN",
                _ => "ERR",
            };
            format!("{}{} {} - {} [{}]", selected, checkbox, d.name, d.display_type.name(), status)
        }).collect::<Vec<_>>().join("\n")
    }

    /// Initialize with default displays (Grid and Axes)
    fn init_default_displays(&mut self) {
        self.displays = vec![
            DisplayInfo::new(self.next_id(), DisplayType::Grid),
            DisplayInfo::new(self.next_id(), DisplayType::Axes),
        ];
    }

    /// Generate next unique ID
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a new display
    pub fn add_display(&mut self, cx: &mut Cx, display_type: DisplayType) -> u64 {
        let id = self.next_id();
        let display = DisplayInfo::new(id, display_type);
        self.displays.push(display);
        self.redraw(cx);
        id
    }

    /// Add a display with full info
    pub fn add_display_info(&mut self, cx: &mut Cx, mut info: DisplayInfo) -> u64 {
        if info.id == 0 {
            info.id = self.next_id();
        }
        let id = info.id;
        self.displays.push(info);
        self.redraw(cx);
        id
    }

    /// Remove a display by ID
    pub fn remove_display(&mut self, cx: &mut Cx, id: u64) -> bool {
        if let Some(pos) = self.displays.iter().position(|d| d.id == id) {
            self.displays.remove(pos);
            // Clear selection if the removed item was selected
            if self.selected_index == Some(pos) {
                self.selected_index = None;
            } else if let Some(sel) = self.selected_index {
                if sel > pos {
                    self.selected_index = Some(sel - 1);
                }
            }
            self.redraw(cx);
            true
        } else {
            false
        }
    }

    /// Set the enabled state of a display
    pub fn set_display_enabled(&mut self, cx: &mut Cx, id: u64, enabled: bool) {
        if let Some(display) = self.displays.iter_mut().find(|d| d.id == id) {
            display.enabled = enabled;
            self.redraw(cx);
        }
    }

    /// Set the status of a display
    pub fn set_display_status(&mut self, cx: &mut Cx, id: u64, status: u8, message: Option<String>) {
        if let Some(display) = self.displays.iter_mut().find(|d| d.id == id) {
            display.status = status;
            display.status_message = message;
            self.redraw(cx);
        }
    }

    /// Set the list of displays to show
    pub fn set_displays(&mut self, cx: &mut Cx, displays: Vec<DisplayInfo>) {
        self.displays = displays;
        // Update next_id to be higher than any existing ID
        if let Some(max_id) = self.displays.iter().map(|d| d.id).max() {
            self.next_id = max_id + 1;
        }
        self.redraw(cx);
    }

    /// Get the currently selected display ID
    pub fn selected_display_id(&self) -> Option<u64> {
        self.selected_index.and_then(|i| self.displays.get(i).map(|d| d.id))
    }

    /// Get the currently selected display info
    pub fn selected_display(&self) -> Option<&DisplayInfo> {
        self.selected_index.and_then(|i| self.displays.get(i))
    }

    /// Select a display by ID
    pub fn select(&mut self, cx: &mut Cx, id: u64) {
        if let Some(pos) = self.displays.iter().position(|d| d.id == id) {
            self.select_index(cx, Some(pos));
        }
    }

    /// Select a display by index
    fn select_index(&mut self, cx: &mut Cx, index: Option<usize>) {
        self.selected_index = index;
        self.redraw(cx);
    }

    /// Deselect current selection
    pub fn deselect(&mut self, cx: &mut Cx) {
        self.selected_index = None;
        self.redraw(cx);
    }

    /// Get the number of displays
    pub fn display_count(&self) -> usize {
        self.displays.len()
    }

    /// Get all displays
    pub fn displays(&self) -> &[DisplayInfo] {
        &self.displays
    }

    /// Get a display by ID
    pub fn get_display(&self, id: u64) -> Option<&DisplayInfo> {
        self.displays.iter().find(|d| d.id == id)
    }

    /// Get a mutable display by ID
    pub fn get_display_mut(&mut self, id: u64) -> Option<&mut DisplayInfo> {
        self.displays.iter_mut().find(|d| d.id == id)
    }
}

// ============================================================================
// WIDGET REF EXTENSIONS
// ============================================================================

/// Extension trait for DisplaysPanel widget references (display management)
pub trait DisplaysPanelDisplayOps {
    fn add_new_display(&self, cx: &mut Cx, display_type: DisplayType) -> u64;
    fn remove_display_by_id(&self, cx: &mut Cx, id: u64) -> bool;
    fn set_display_enabled_state(&self, cx: &mut Cx, id: u64, enabled: bool);
    fn set_display_status_info(&self, cx: &mut Cx, id: u64, status: u8, message: Option<String>);
    fn get_selected_display_id(&self) -> Option<u64>;
}

impl DisplaysPanelDisplayOps for DisplaysPanelRef {
    fn add_new_display(&self, cx: &mut Cx, display_type: DisplayType) -> u64 {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_display(cx, display_type)
        } else {
            0
        }
    }

    fn remove_display_by_id(&self, cx: &mut Cx, id: u64) -> bool {
        if let Some(mut inner) = self.borrow_mut() {
            inner.remove_display(cx, id)
        } else {
            false
        }
    }

    fn set_display_enabled_state(&self, cx: &mut Cx, id: u64, enabled: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_display_enabled(cx, id, enabled);
        }
    }

    fn set_display_status_info(&self, cx: &mut Cx, id: u64, status: u8, message: Option<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_display_status(cx, id, status, message);
        }
    }

    fn get_selected_display_id(&self) -> Option<u64> {
        if let Some(inner) = self.borrow() {
            inner.selected_display_id()
        } else {
            None
        }
    }
}
