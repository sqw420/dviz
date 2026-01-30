//! Sensor Panel Widget
//!
//! Displays sensor data in a grouped panel with status indicator.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Single sensor row with label and value
    pub SensorRow = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 6, bottom: 6}

        sensor_label = <Label> {
            width: 80
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        sensor_value = <Label> {
            text: "0.000"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_MEDIUM>{ font_size: 12.0 }
            }
        }
    }

    // Sensor group header with title and status indicator
    pub SensorGroupHeader = <View> {
        width: Fill, height: Fit
        flow: Right
        align: {y: 0.5}
        padding: {bottom: 8}

        group_title = <Label> {
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
            }
        }

        <View> { width: Fill, height: 1 }

        status_indicator = <View> {
            width: 8, height: 8
            show_bg: true
            draw_bg: {
                instance active: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, self.rect_size.x * 0.5);
                    let inactive = vec4(0.4, 0.4, 0.4, 1.0);
                    let active_color = vec4(0.06, 0.73, 0.51, 1.0);
                    sdf.fill(mix(inactive, active_color, self.active));
                    return sdf.result;
                }
            }
        }
    }

    // Complete sensor group panel
    pub SensorGroup = {{SensorGroup}} <RoundedView> {
        width: Fill, height: Fit
        flow: Down
        padding: 12
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }

        group_header = <SensorGroupHeader> {
            group_title = { text: "Sensor Group" }
        }

        <View> {
            width: Fill, height: 1
            margin: {bottom: 8}
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        sensors_container = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 2

            // IMU Accelerometer
            accel_x = <SensorRow> {
                sensor_label = { text: "Accel X" }
            }
            accel_y = <SensorRow> {
                sensor_label = { text: "Accel Y" }
            }
            accel_z = <SensorRow> {
                sensor_label = { text: "Accel Z" }
            }

            <View> { width: Fill, height: 4 }

            // IMU Gyroscope
            gyro_x = <SensorRow> {
                sensor_label = { text: "Gyro X" }
            }
            gyro_y = <SensorRow> {
                sensor_label = { text: "Gyro Y" }
            }
            gyro_z = <SensorRow> {
                sensor_label = { text: "Gyro Z" }
            }
        }
    }

    // Position display panel
    pub PositionPanel = {{PositionPanel}} <RoundedView> {
        width: Fill, height: Fit
        flow: Down
        padding: 12
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }

        pos_header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {y: 0.5}
            padding: {bottom: 8}

            <Label> {
                text: "Position"
                draw_text: {
                    color: (TEXT_PRIMARY)
                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                }
            }
        }

        <View> {
            width: Fill, height: 1
            margin: {bottom: 8}
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        pos_values = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 4

            pos_x = <SensorRow> {
                sensor_label = { text: "X" }
            }
            pos_y = <SensorRow> {
                sensor_label = { text: "Y" }
            }
            pos_z = <SensorRow> {
                sensor_label = { text: "Z" }
            }
        }
    }
}

// ============================================================================
// WIDGET STRUCTS
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct SensorGroup {
    #[deref]
    view: View,
}

impl Widget for SensorGroup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PositionPanel {
    #[deref]
    view: View,
}

impl Widget for PositionPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

