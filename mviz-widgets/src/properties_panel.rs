//! Properties Panel Widget
//!
//! Provides property editors for the selected display.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Property row - label + value
    pub PropertyRow = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        prop_value = <Label> {
            width: Fill
            text: "-"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }
    }

    // Bool property with checkbox
    pub BoolProperty = {{BoolProperty}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        prop_check = <CheckBox> {
            width: Fit, height: Fit
        }
    }

    // Float property with slider
    pub FloatProperty = {{FloatProperty}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        prop_slider = <Slider> {
            width: Fill, height: 20
            min: 0.0
            max: 1.0
            step: 0.01
        }

        prop_value = <Label> {
            width: 40
            text: "0.00"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 10.0 }
            }
        }
    }

    // String property with text input
    pub StringProperty = {{StringProperty}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        prop_input = <TextInput> {
            width: Fill, height: 24
            draw_bg: {
                color: #333333
            }
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }
    }

    // Color property with color preview and inputs
    pub ColorProperty = {{ColorProperty}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        // Color swatch preview
        color_swatch = <View> {
            width: 24, height: 24
            show_bg: true
            draw_bg: {
                instance swatch_color: vec4(1.0, 1.0, 1.0, 1.0)
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                    sdf.fill(self.swatch_color);
                    sdf.stroke(vec4(0.4, 0.4, 0.4, 1.0), 1.0);
                    return sdf.result;
                }
            }
        }

        // RGB values display
        rgb_label = <Label> {
            width: Fill
            text: "255, 255, 255"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 10.0 }
            }
        }
    }

    // Vec3 property (X, Y, Z)
    pub Vec3Property = {{Vec3Property}} <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 4
        padding: {left: 12, right: 12, top: 6, bottom: 6}

        prop_header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {y: 0.5}

            prop_label = <Label> {
                width: 80
                draw_text: {
                    color: (TEXT_SECONDARY)
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                }
            }
        }

        xyz_row = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 4
            padding: {left: 80}

            <Label> { text: "X", width: 12, draw_text: { color: #ff4444, text_style: <FONT_REGULAR>{ font_size: 10.0 } } }
            x_input = <TextInput> { width: 50, height: 22, draw_bg: { color: #333333 }, draw_text: { color: (TEXT_PRIMARY), text_style: <FONT_REGULAR>{ font_size: 10.0 } } }

            <Label> { text: "Y", width: 12, draw_text: { color: #44ff44, text_style: <FONT_REGULAR>{ font_size: 10.0 } } }
            y_input = <TextInput> { width: 50, height: 22, draw_bg: { color: #333333 }, draw_text: { color: (TEXT_PRIMARY), text_style: <FONT_REGULAR>{ font_size: 10.0 } } }

            <Label> { text: "Z", width: 12, draw_text: { color: #4444ff, text_style: <FONT_REGULAR>{ font_size: 10.0 } } }
            z_input = <TextInput> { width: 50, height: 22, draw_bg: { color: #333333 }, draw_text: { color: (TEXT_PRIMARY), text_style: <FONT_REGULAR>{ font_size: 10.0 } } }
        }
    }

    // Enum/dropdown property
    pub EnumProperty = {{EnumProperty}} <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}
        align: {y: 0.5}

        prop_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        prop_dropdown = <DropDown> {
            width: Fill, height: 24
        }
    }

    // Property section header
    pub PropertySection = <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 4

        section_header = <View> {
            width: Fill, height: Fit
            flow: Right
            padding: {left: 12, right: 12, top: 8, bottom: 4}

            section_label = <Label> {
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 11.0 }
                }
            }
        }
    }

    // Complete properties panel
    pub PropertiesPanel = {{PropertiesPanel}} <RoundedView> {
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
            flow: Down
            padding: 12
            spacing: 4

            display_name = <Label> {
                text: "No Selection"
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                }
            }

            display_type = <Label> {
                text: "Select a display to view properties"
                draw_text: {
                    color: (TEXT_MUTED)
                    text_style: <FONT_REGULAR>{ font_size: 10.0 }
                }
            }
        }

        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        // Scrollable properties area
        properties_scroll = <ScrollYView> {
            width: Fill, height: Fill

            properties_content = <View> {
                width: Fill, height: Fit
                flow: Down
                spacing: 2
                padding: {top: 8, bottom: 8}

                // Common properties section
                common_section = <PropertySection> {
                    section_header = { section_label = { text: "Common" } }
                }

                enabled_prop = <BoolProperty> {
                    prop_label = { text: "Enabled" }
                }

                // Display-specific properties will be added dynamically
                specific_section = <PropertySection> {
                    section_header = { section_label = { text: "Display Options" } }
                }

                // Placeholder properties for demonstration
                alpha_prop = <FloatProperty> {
                    prop_label = { text: "Alpha" }
                }

                color_prop = <ColorProperty> {
                    prop_label = { text: "Color" }
                }

                frame_prop = <StringProperty> {
                    prop_label = { text: "Frame" }
                }
            }
        }

        // Status section at bottom
        status_section = <View> {
            width: Fill, height: Fit
            flow: Down
            padding: 12
            spacing: 4
            show_bg: true
            draw_bg: { color: #1e1e1e }

            status_label = <Label> {
                text: "Status: OK"
                draw_text: {
                    color: (STATUS_OK)
                    text_style: <FONT_REGULAR>{ font_size: 10.0 }
                }
            }

            status_message = <Label> {
                text: ""
                draw_text: {
                    color: (TEXT_MUTED)
                    text_style: <FONT_REGULAR>{ font_size: 10.0 }
                    wrap: Word
                }
            }
        }
    }
}

// ============================================================================
// PROPERTY VALUE TYPES
// ============================================================================

/// Property value type
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Bool(bool),
    Float(f64),
    String(String),
    Color { r: u8, g: u8, b: u8, a: u8 },
    Vec3 { x: f32, y: f32, z: f32 },
    Enum { value: usize, options: Vec<String> },
}

/// Property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub label: String,
    pub value: PropertyValue,
    pub read_only: bool,
}

// ============================================================================
// WIDGET STRUCTS
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct BoolProperty {
    #[deref]
    view: View,
}

impl Widget for BoolProperty {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct FloatProperty {
    #[deref]
    view: View,
}

impl Widget for FloatProperty {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct StringProperty {
    #[deref]
    view: View,
}

impl Widget for StringProperty {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ColorProperty {
    #[deref]
    view: View,
}

impl Widget for ColorProperty {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Vec3Property {
    #[deref]
    view: View,
}

impl Widget for Vec3Property {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct EnumProperty {
    #[deref]
    view: View,
}

impl Widget for EnumProperty {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PropertiesPanel {
    #[deref]
    view: View,
    #[rust] current_display_id: Option<u64>,
    #[rust] properties: Vec<Property>,
}

impl Widget for PropertiesPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl PropertiesPanel {
    /// Set the display to show properties for (with UI update)
    pub fn set_display(&mut self, cx: &mut Cx, display_id: Option<u64>, name: &str, display_type: &str) {
        self.current_display_id = display_id;

        // Update header labels
        self.label(id!(header.display_name)).set_text(cx, name);
        self.label(id!(header.display_type)).set_text(cx, display_type);

        // Update status based on selection
        if display_id.is_some() {
            self.label(id!(status_label)).set_text(cx, "Status: OK");
        } else {
            self.label(id!(status_label)).set_text(cx, "Status: -");
        }

        self.redraw(cx);
    }

    /// Clear selection (show "No Selection")
    pub fn clear_selection(&mut self, cx: &mut Cx) {
        self.current_display_id = None;
        self.label(id!(header.display_name)).set_text(cx, "No Selection");
        self.label(id!(header.display_type)).set_text(cx, "Select a display to view properties");
        self.label(id!(status_label)).set_text(cx, "Status: -");
        self.redraw(cx);
    }

    /// Set properties to display
    pub fn set_properties(&mut self, properties: Vec<Property>) {
        self.properties = properties;
    }

    /// Get current display ID
    pub fn current_display_id(&self) -> Option<u64> {
        self.current_display_id
    }
}

// ============================================================================
// WIDGET REF EXTENSIONS
// ============================================================================

impl PropertiesPanelRef {
    /// Set the display to show properties for
    pub fn set_display(&self, cx: &mut Cx, display_id: Option<u64>, name: &str, display_type: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_display(cx, display_id, name, display_type);
        }
    }

    /// Clear selection
    pub fn clear_selection(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear_selection(cx);
        }
    }
}
