//! Toolbar Widget
//!
//! Provides the main application toolbar with file menu, tool buttons,
//! frame selector, and playback controls.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;

    // Toolbar button with icon
    pub ToolbarButton = <View> {
        width: 36, height: 36
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance active: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                let base = vec4(0.15, 0.15, 0.15, 0.0);
                let hover_color = vec4(0.25, 0.25, 0.25, 1.0);
                let active_color = vec4(0.23, 0.51, 0.96, 0.3);
                let color = mix(mix(base, hover_color, self.hover), active_color, self.active);
                sdf.fill(color);
                return sdf.result;
            }
        }
    }

    // Toolbar divider
    pub ToolbarDivider = <View> {
        width: 1, height: 24
        margin: {left: 8, right: 8}
        show_bg: true
        draw_bg: { color: (DIVIDER) }
    }

    // Frame selector dropdown
    pub FrameSelector = {{FrameSelector}} <View> {
        width: Fit, height: Fit
        flow: Right
        spacing: 8
        align: {y: 0.5}

        <Label> {
            text: "Frame:"
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
            }
        }

        frame_dropdown = <DropDown> {
            width: 120, height: 28
        }
    }

    // Playback time display
    pub TimeDisplay = <View> {
        width: Fit, height: Fit
        flow: Right
        spacing: 4
        align: {y: 0.5}
        padding: {left: 8, right: 8}

        time_label = <Label> {
            text: "0.000s"
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_REGULAR>{ font_size: 12.0 }
            }
        }
    }

    // Play/Pause toggle button
    pub PlayPauseButton = <View> {
        width: 32, height: 32
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance playing: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                let base = vec4(0.15, 0.15, 0.15, 1.0);
                let hover_color = vec4(0.25, 0.25, 0.25, 1.0);
                let playing_color = vec4(0.06, 0.73, 0.51, 0.8);
                let bg = mix(mix(base, hover_color, self.hover), playing_color, self.playing * 0.3);
                sdf.fill(bg);

                let cx = self.rect_size.x * 0.5;
                let cy = self.rect_size.y * 0.5;

                if self.playing < 0.5 {
                    // Play triangle
                    sdf.move_to(cx - 3.0, cy - 5.0);
                    sdf.line_to(cx + 5.0, cy);
                    sdf.line_to(cx - 3.0, cy + 5.0);
                    sdf.close_path();
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                } else {
                    // Pause bars
                    sdf.box(cx - 5.0, cy - 4.0, 3.5, 8.0, 1.0);
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                    sdf.box(cx + 1.5, cy - 4.0, 3.5, 8.0, 1.0);
                    sdf.fill(vec4(1.0, 1.0, 1.0, 1.0));
                }

                return sdf.result;
            }
        }
    }

    // Step forward/backward buttons
    pub StepButton = <View> {
        width: 28, height: 28
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance forward: 1.0  // 1.0 = forward, 0.0 = backward
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = vec4(0.15, 0.15, 0.15, 0.0);
                let hover_color = vec4(0.25, 0.25, 0.25, 1.0);
                sdf.fill(mix(base, hover_color, self.hover));

                let cx = self.rect_size.x * 0.5;
                let cy = self.rect_size.y * 0.5;
                let dir = self.forward * 2.0 - 1.0;

                // Step arrow
                sdf.move_to(cx - 3.0 * dir, cy - 4.0);
                sdf.line_to(cx + 3.0 * dir, cy);
                sdf.line_to(cx - 3.0 * dir, cy + 4.0);
                sdf.stroke(vec4(0.7, 0.7, 0.7, 1.0), 1.5);

                // Bar
                sdf.move_to(cx + 4.0 * dir, cy - 4.0);
                sdf.line_to(cx + 4.0 * dir, cy + 4.0);
                sdf.stroke(vec4(0.7, 0.7, 0.7, 1.0), 1.5);

                return sdf.result;
            }
        }
    }

    // Speed selector
    pub SpeedSelector = <View> {
        width: Fit, height: Fit
        flow: Right
        spacing: 4
        align: {y: 0.5}

        <Label> {
            text: "Speed:"
            draw_text: {
                color: (TEXT_SECONDARY)
                text_style: <FONT_REGULAR>{ font_size: 10.0 }
            }
        }

        speed_dropdown = <DropDown> {
            width: 60, height: 24
        }
    }

    // Complete toolbar
    pub Toolbar = {{Toolbar}} <View> {
        width: Fill, height: 44
        flow: Right
        spacing: 8
        padding: {left: 12, right: 12, top: 4, bottom: 4}
        align: {y: 0.5}
        show_bg: true
        draw_bg: { color: (PANEL_BG) }

        // File menu button
        file_btn = <Button> {
            text: "File"
            draw_text: { color: #fff }
        }

        <ToolbarDivider> {}

        // Tool buttons (for future expansion)
        move_tool = <ToolbarButton> {}
        select_tool = <ToolbarButton> {}
        measure_tool = <ToolbarButton> {}

        <ToolbarDivider> {}

        // View controls
        reset_view_btn = <Button> {
            text: "Reset View"
            draw_text: { color: #fff }
        }

        focus_btn = <Button> {
            text: "Focus"
            draw_text: { color: #fff }
        }

        <ToolbarDivider> {}

        // Frame selector
        frame_selector = <FrameSelector> {}

        <View> { width: Fill, height: 1 }

        // Playback controls
        playback_section = <View> {
            width: Fit, height: Fit
            flow: Right
            spacing: 4
            align: {y: 0.5}

            step_back = <StepButton> { draw_bg: { forward: 0.0 } }
            play_pause = <PlayPauseButton> {}
            step_forward = <StepButton> { draw_bg: { forward: 1.0 } }

            time_display = <TimeDisplay> {}
            speed_selector = <SpeedSelector> {}
        }
    }
}

// ============================================================================
// WIDGET STRUCTS
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct FrameSelector {
    #[deref]
    view: View,
    #[rust] frames: Vec<String>,
    #[rust] selected_frame: String,
}

impl Widget for FrameSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl FrameSelector {
    /// Set available frames
    pub fn set_frames(&mut self, frames: Vec<String>) {
        self.frames = frames;
    }

    /// Get selected frame
    pub fn selected_frame(&self) -> &str {
        &self.selected_frame
    }

    /// Set selected frame
    pub fn set_selected_frame(&mut self, frame: String) {
        self.selected_frame = frame;
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Toolbar {
    #[deref]
    view: View,
    #[rust] playing: bool,
    #[rust] current_time: f64,
    #[rust] playback_speed: f64,
}

impl Widget for Toolbar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl Toolbar {
    /// Set playing state
    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    /// Is currently playing?
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Set current playback time
    pub fn set_time(&mut self, time: f64) {
        self.current_time = time;
    }

    /// Get current playback time
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Set playback speed
    pub fn set_playback_speed(&mut self, speed: f64) {
        self.playback_speed = speed;
    }

    /// Get playback speed
    pub fn playback_speed(&self) -> f64 {
        self.playback_speed
    }
}
