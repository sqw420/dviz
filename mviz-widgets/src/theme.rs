//! Theme definitions for MViz UI
//!
//! Provides colors, fonts, and common styles for the application.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // ============================================================================
    // COLORS
    // ============================================================================

    // Background colors
    pub DARK_BG = #1a1a1a
    pub PANEL_BG = #252525
    pub CARD_BG = #2a2a2a
    pub HOVER_BG = #333333

    // Accent colors
    pub ACCENT_BLUE = #3b82f6
    pub ACCENT_GREEN = #10b981
    pub ACCENT_RED = #ef4444
    pub ACCENT_YELLOW = #f59e0b
    pub ACCENT_PURPLE = #8b5cf6

    // Text colors
    pub TEXT_PRIMARY = #ffffff
    pub TEXT_SECONDARY = #a0a0a0
    pub TEXT_MUTED = #606060

    // Status colors
    pub STATUS_OK = #10b981
    pub STATUS_WARN = #f59e0b
    pub STATUS_ERROR = #ef4444

    // Other
    pub DIVIDER = #3a3a3a
    pub WHITE = #ffffff

    // ============================================================================
    // FONT STYLES
    // Use THEME_FONT_REGULAR / THEME_FONT_BOLD from Makepad's theme
    // ============================================================================

    pub FONT_REGULAR = <THEME_FONT_REGULAR> {
        font_size: 11.0
    }

    pub FONT_MEDIUM = <THEME_FONT_REGULAR> {
        font_size: 11.0
    }

    pub FONT_SEMIBOLD = <THEME_FONT_BOLD> {
        font_size: 12.0
    }

    pub FONT_BOLD = <THEME_FONT_BOLD> {
        font_size: 13.0
    }

    // ============================================================================
    // COMMON WIDGETS
    // ============================================================================

    // Rounded panel container
    pub RoundedPanel = <RoundedView> {
        width: Fill, height: Fit
        padding: 12
        show_bg: true
        draw_bg: {
            color: (PANEL_BG)
            border_radius: 8.0
        }
    }

    // Status indicator dot
    pub StatusDot = <View> {
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

    // Primary button style
    pub PrimaryButton = <View> {
        width: Fit, height: Fit
        padding: {left: 16, right: 16, top: 8, bottom: 8}
        cursor: Hand
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                let base = vec4(0.23, 0.51, 0.96, 1.0);
                let hover_color = vec4(0.37, 0.62, 0.98, 1.0);
                sdf.fill(mix(base, hover_color, self.hover));
                return sdf.result;
            }
        }
    }

    // Icon button style
    pub IconButton = <View> {
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
                let base = vec4(0.2, 0.2, 0.2, 1.0);
                let hover_color = vec4(0.3, 0.3, 0.3, 1.0);
                let active_color = vec4(0.23, 0.51, 0.96, 1.0);
                let color = mix(mix(base, hover_color, self.hover), active_color, self.active);
                sdf.fill(color);
                return sdf.result;
            }
        }
    }

    // Section header
    pub SectionHeader = <View> {
        width: Fill, height: Fit
        flow: Right
        align: {y: 0.5}
        padding: {bottom: 8}

        title = <Label> {
            draw_text: {
                color: (TEXT_PRIMARY)
                text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
            }
        }
    }

    // Divider line
    pub Divider = <View> {
        width: Fill, height: 1
        margin: {top: 8, bottom: 8}
        show_bg: true
        draw_bg: { color: (DIVIDER) }
    }
}

