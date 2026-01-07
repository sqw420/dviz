//! Control Bar Widget
//!
//! Provides playback controls and connection status display.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Play/Pause button with toggle state
    pub PlayButton = <View> {
        width: 40, height: 40
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance playing: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.0);
                let base = vec4(0.2, 0.2, 0.2, 1.0);
                let hover_color = vec4(0.3, 0.3, 0.3, 1.0);
                let playing_color = vec4(0.06, 0.73, 0.51, 1.0);
                let bg_color = mix(mix(base, hover_color, self.hover), playing_color, self.playing);
                sdf.fill(bg_color);

                // Draw play/pause icon
                let cx = self.rect_size.x * 0.5;
                let cy = self.rect_size.y * 0.5;
                let icon_size = 10.0;

                if self.playing < 0.5 {
                    // Play triangle
                    sdf.move_to(cx - 4.0, cy - 6.0);
                    sdf.line_to(cx + 6.0, cy);
                    sdf.line_to(cx - 4.0, cy + 6.0);
                    sdf.close_path();
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                } else {
                    // Pause bars
                    sdf.box(cx - 6.0, cy - 5.0, 4.0, 10.0, 1.0);
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                    sdf.box(cx + 2.0, cy - 5.0, 4.0, 10.0, 1.0);
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                }

                return sdf.result;
            }
        }
    }

    // Record button
    pub RecordButton = <View> {
        width: 40, height: 40
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance recording: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.0);
                let base = vec4(0.2, 0.2, 0.2, 1.0);
                let hover_color = vec4(0.3, 0.3, 0.3, 1.0);
                sdf.fill(mix(base, hover_color, self.hover));

                // Record dot
                let cx = self.rect_size.x * 0.5;
                let cy = self.rect_size.y * 0.5;
                sdf.circle(cx, cy, 6.0);
                let dot_color = mix(vec4(0.8, 0.2, 0.2, 1.0), vec4(1.0, 0.3, 0.3, 1.0), self.recording);
                sdf.fill(dot_color);

                return sdf.result;
            }
        }
    }

    // Connection status indicator
    pub ConnectionStatus = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        align: {y: 0.5}

        connection_dot = <View> {
            width: 8, height: 8
            show_bg: true
            draw_bg: {
                instance connected: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, self.rect_size.x * 0.5);
                    let disconnected = vec4(0.94, 0.27, 0.27, 1.0);
                    let connected_color = vec4(0.06, 0.73, 0.51, 1.0);
                    sdf.fill(mix(disconnected, connected_color, self.connected));
                    return sdf.result;
                }
            }
        }

        connection_label = <Label> {
            text: "Disconnected"
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }
    }

    // Rerun status section
    pub RerunStatus = <View> {
        width: Fill, height: Fit
        flow: Right
        spacing: 8
        align: {y: 0.5}
        padding: {left: 8, right: 8, top: 4, bottom: 4}

        rerun_label = <Label> {
            text: "Rerun"
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        <View> { width: Fill, height: 1 }

        rerun_status = <View> {
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

    // Complete control bar
    pub ControlBar = {{ControlBar}} <RoundedView> {
        width: Fill, height: Fit
        flow: Down
        padding: 12
        spacing: 12
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }

        // Playback controls
        controls_section = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 8
            align: {y: 0.5}

            play_btn = <PlayButton> {}
            record_btn = <RecordButton> {}

            <View> { width: Fill, height: 1 }
        }

        <View> {
            width: Fill, height: 1
            show_bg: true
            draw_bg: { color: (DIVIDER) }
        }

        // Rerun status
        rerun_section = <RerunStatus> {}

        // Connection status
        connection_section = <ConnectionStatus> {}
    }
}

// ============================================================================
// WIDGET STRUCT
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct ControlBar {
    #[deref]
    view: View,
}

impl Widget for ControlBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

