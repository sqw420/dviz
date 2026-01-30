//! # Theme System for MViz
//!
//! Centralized color palette, fonts, and dark mode support.
//! Based on mofa-studio theme with Tailwind CSS color system.
//!
//! ## Usage
//!
//! Import the theme in your `live_design!` macro:
//!
//! ```rust,ignore
//! live_design! {
//!     use dviz_widgets::theme::*;
//!
//!     MyWidget = <View> {
//!         draw_bg: { color: (PANEL_BG) }
//!         label = <Label> {
//!             draw_text: {
//!                 color: (TEXT_PRIMARY)
//!                 text_style: <FONT_REGULAR> { font_size: 12.0 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Color Categories
//!
//! ### Semantic Colors (Recommended)
//! Use these for consistent theming:
//! - `DARK_BG` - Main app background
//! - `PANEL_BG` / `PANEL_BG_DARK` - Card/panel backgrounds
//! - `TEXT_PRIMARY` / `TEXT_PRIMARY_DARK` - Main text
//! - `TEXT_SECONDARY` / `TEXT_SECONDARY_DARK` - Muted text
//! - `ACCENT_BLUE`, `ACCENT_GREEN`, `ACCENT_RED` - Action colors
//! - `BORDER` / `BORDER_DARK` - Borders and dividers
//! - `HOVER_BG` / `HOVER_BG_DARK` - Hover states
//!
//! ### Color Palettes
//! Full Tailwind-style palettes (50-900 shades):
//! - `SLATE_*` - Cool gray for backgrounds
//! - `GRAY_*` - Neutral gray for text
//! - `BLUE_*`, `INDIGO_*` - Primary colors
//! - `GREEN_*`, `RED_*`, `AMBER_*` - Status colors
//!
//! ## Dark Mode
//!
//! Widgets support dark mode via shader instance variables:
//!
//! ```rust,ignore
//! draw_bg: {
//!     instance dark_mode: 0.0  // 0.0 = light, 1.0 = dark
//!     fn pixel(self) -> vec4 {
//!         return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
//!     }
//! }
//! ```
//!
//! Update at runtime via `apply_over`:
//! ```rust,ignore
//! widget.apply_over(cx, live!{ draw_bg: { dark_mode: 1.0 } });
//! ```
//!
//! ## Important Notes
//!
//! - **Hex colors in shaders**: Theme constants like `(ACCENT_BLUE)` work in
//!   `live_design!{}` properties but NOT inside shader `fn pixel()` functions.
//!   Use `vec4()` literals for shader code.
//!
//! - **Lexer issues**: Some hex values are adjusted to avoid Rust lexer conflicts
//!   (e.g., `#1e293b` -> `#1f293b` because `1e` looks like scientific notation).

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // ========================================================================
    // COLOR PALETTE
    // Based on Tailwind CSS color system for consistency
    // ========================================================================

    // --- Semantic Colors (Light Theme - use these first) ---
    pub DARK_BG = #f0f4f8          // Main background (cool gray with blue tint)
    pub PANEL_BG = #f8fafc         // Card/panel background (very light slate)
    pub CARD_BG = #f1f5f9          // Card background (slate-100)
    pub TOOLBAR_BG = #e8f0f7       // Toolbar background (light blue-gray)
    pub HEADER_BG = #e0e7ef        // Section headers (light blue-gray)
    pub INPUT_BG = #eef2f7         // Text input background
    pub HOVER_BG = #e2e8f0         // Hover background (slate-200)
    pub ACCENT_BLUE = #3b82f6      // Primary action color
    pub ACCENT_GREEN = #10b981     // Success/positive
    pub ACCENT_RED = #ef4444       // Error/danger
    pub ACCENT_YELLOW = #f59f0b    // Warning (adjusted from #f59e0b)
    pub ACCENT_PURPLE = #8b5cf6    // Purple accent
    pub ACCENT_TEAL = #0d9488      // Teal accent
    pub ACCENT_CYAN = #06b6d4      // Cyan accent
    pub ACCENT_INDIGO = #6366f1    // Secondary accent
    pub TEXT_PRIMARY = #1e293b     // Main text (slate-800)
    pub TEXT_SECONDARY = #475569   // Secondary text (slate-600)
    pub TEXT_MUTED = #94a3b8       // Muted/disabled text (slate-400)
    pub DIVIDER = #cbd5e1          // Divider lines (slate-300)
    pub BORDER = #94a3b8           // Border color (slate-400)
    pub BORDER_LIGHT = #e2e8f0     // Light border (slate-200)

    // Status colors
    pub STATUS_OK = #10b981
    pub STATUS_WARN = #f59f0b      // Adjusted from #f59e0b (lexer issue)
    pub STATUS_ERROR = #ef4444

    // --- White ---
    pub WHITE = #ffffff

    // --- Slate (cool gray, used for backgrounds) ---
    pub SLATE_50 = #f8fafc
    pub SLATE_100 = #f1f5f9
    pub SLATE_200 = #e2e8f0
    pub SLATE_300 = #cbd5e1
    pub SLATE_400 = #94a3b8
    pub SLATE_500 = #64748b
    pub SLATE_600 = #475569
    pub SLATE_700 = #334155
    pub SLATE_800 = #1f293b        // Adjusted from #1e293b (lexer issue with 1e)
    pub SLATE_900 = #0f172a
    pub SLATE_950 = #0d1117        // Extra dark (waveform background)

    // --- Gray (neutral gray, used for text/icons) ---
    pub GRAY_50 = #f9fafb
    pub GRAY_100 = #f3f4f6
    pub GRAY_200 = #e5e7eb
    pub GRAY_300 = #d1d5db
    pub GRAY_400 = #9ca3af
    pub GRAY_500 = #6b7280
    pub GRAY_600 = #4b5563
    pub GRAY_700 = #374151
    pub GRAY_800 = #1f2937
    pub GRAY_900 = #111827

    // --- Blue (primary actions) ---
    pub BLUE_50 = #eff6ff
    pub BLUE_100 = #dbeafe
    pub BLUE_200 = #bfdbfe
    pub BLUE_300 = #93c5fd
    pub BLUE_400 = #60a5fa
    pub BLUE_500 = #3b82f6
    pub BLUE_600 = #2565fb      // Adjusted to avoid digit+e pattern
    pub BLUE_700 = #1d4fd8      // Adjusted from #1d4ed8 (lexer issue with 4e)
    pub BLUE_800 = #1f40af      // Adjusted from #1e40af (lexer issue with 1e)
    pub BLUE_900 = #1f3a8a      // Adjusted from #1e3a8a (lexer issue with 1e)

    // --- Indigo (secondary accent) ---
    pub INDIGO_50 = #eef2ff
    pub INDIGO_100 = #e1e7ff      // Adjusted from #e0e7ff (lexer issue with 0e)
    pub INDIGO_200 = #c7d2ff      // Adjusted from #c7d2fe (lexer issue with fe)
    pub INDIGO_300 = #a5b4fc
    pub INDIGO_400 = #818cf8
    pub INDIGO_500 = #6366f1
    pub INDIGO_600 = #4f47e5      // Adjusted from #4f46e5 (lexer issue with 6e)
    pub INDIGO_700 = #4338ca
    pub INDIGO_800 = #3730a3
    pub INDIGO_900 = #312f81      // Adjusted from #312e81 (lexer issue with 2e)

    // --- Green (success) ---
    pub GREEN_50 = #f0fdf4
    pub GREEN_100 = #dcfcf7      // Adjusted from #dcfce7 (lexer issue with ce)
    pub GREEN_200 = #bbf7d0
    pub GREEN_300 = #88ffac      // Adjusted to avoid digit+e pattern
    pub GREEN_400 = #4adf80      // Adjusted from #4ade80 (lexer issue with de)
    pub GREEN_500 = #22c55f      // Adjusted from #22c55e (lexer issue with 5e)
    pub GREEN_600 = #16a34a
    pub GREEN_700 = #15803d
    pub GREEN_800 = #166534
    pub GREEN_900 = #14532d

    // --- Emerald (alternate green) ---
    pub EMERALD_500 = #10b981
    pub EMERALD_600 = #059669
    pub EMERALD_700 = #047857

    // --- Red (error/danger) ---
    pub RED_50 = #fff2f2        // Adjusted from #fef2f2 (lexer issue with ef)
    pub RED_100 = #fff2f2       // Adjusted from #fee2e2 (lexer issue with ee)
    pub RED_200 = #ffcaca       // Adjusted from #fecaca (lexer issue with ec)
    pub RED_300 = #fca5a5
    pub RED_400 = #f87171
    pub RED_500 = #ef4444
    pub RED_600 = #dc2626
    pub RED_700 = #b91c1c
    pub RED_800 = #991b1b
    pub RED_900 = #7f1d1d

    // --- Yellow/Amber (warning) ---
    pub YELLOW_500 = #eab308
    pub AMBER_500 = #f59f0b        // Adjusted from #f59e0b (lexer issue with 9e)

    // --- Orange ---
    pub ORANGE_500 = #f97316

    // --- Transparent ---
    pub TRANSPARENT = #00000000

    // ========================================================================
    // DARK THEME VARIANTS
    // Use with mix(LIGHT_COLOR, DARK_COLOR, dark_mode) in shaders
    // ========================================================================

    // --- Dark Theme Semantic Colors ---
    pub DARK_BG_DARK = #0f172a         // Main background (dark)
    pub PANEL_BG_DARK = #1f293b        // Card/panel background (dark) - adjusted from #1e293b
    pub CARD_BG_DARK = #1f293b         // Card background (dark)
    pub TOOLBAR_BG_DARK = #0f172a      // Toolbar background (dark)
    pub HEADER_BG_DARK = #1f293b       // Section headers (dark)
    pub INPUT_BG_DARK = #334155        // Text input background (dark)
    pub TEXT_PRIMARY_DARK = #f1f5f9    // Main text (dark)
    pub TEXT_SECONDARY_DARK = #94a3b8  // Secondary text (dark)
    pub TEXT_MUTED_DARK = #64748b      // Muted text (dark)
    pub DIVIDER_DARK = #475569         // Divider lines (dark)
    pub BORDER_DARK = #334155          // Border color (dark)
    pub HOVER_BG_DARK = #334155        // Hover background (dark)
    pub ACCENT_BLUE_DARK = #60a5fa     // Primary action (brighter for dark mode)

    // ========================================================================
    // FONT STYLES
    // ========================================================================

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

    // ========================================================================
    // THEMEABLE WIDGET BASES
    // Base widgets with dark_mode instance for theme switching
    // ========================================================================

    pub ThemeableView = <View> {
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0

            fn get_bg_color(self) -> vec4 {
                // Light: PANEL_BG (#f8fafc), Dark: PANEL_BG_DARK (#1f293b)
                let light = vec4(0.973, 0.980, 0.988, 1.0);
                let dark = vec4(0.122, 0.161, 0.231, 1.0);
                return mix(light, dark, self.dark_mode);
            }

            fn pixel(self) -> vec4 {
                return self.get_bg_color();
            }
        }
    }

    pub ThemeableRoundedView = <RoundedView> {
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            border_radius: 8.0

            fn get_bg_color(self) -> vec4 {
                let light = vec4(0.973, 0.980, 0.988, 1.0);
                let dark = vec4(0.122, 0.161, 0.231, 1.0);
                return mix(light, dark, self.dark_mode);
            }
        }
    }

    // ========================================================================
    // COMMON WIDGETS
    // ========================================================================

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

    // Primary button style - blue accent
    pub PrimaryButton = <View> {
        width: Fit, height: Fit
        padding: {left: 16, right: 16, top: 8, bottom: 8}
        cursor: Hand
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                // Light mode: Blue (#3b82f6), Dark mode: Brighter blue (#60a5fa)
                let base_light = vec4(0.23, 0.51, 0.96, 1.0);
                let base_dark = vec4(0.376, 0.647, 0.98, 1.0);
                let hover_light = vec4(0.15, 0.40, 0.85, 1.0);
                let hover_dark = vec4(0.23, 0.51, 0.96, 1.0);
                let base = mix(base_light, base_dark, self.dark_mode);
                let hover_color = mix(hover_light, hover_dark, self.dark_mode);
                sdf.fill(mix(base, hover_color, self.hover));
                return sdf.result;
            }
        }
    }

    // Icon button style - themeable
    pub IconButton = <View> {
        width: 36, height: 36
        cursor: Hand
        align: {x: 0.5, y: 0.5}
        show_bg: true
        draw_bg: {
            instance hover: 0.0
            instance active: 0.0
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 6.0);
                // Light mode colors
                let base_light = vec4(0.88, 0.91, 0.94, 1.0);        // Slate-200
                let hover_light = vec4(0.80, 0.84, 0.88, 1.0);       // Slate-300
                // Dark mode colors
                let base_dark = vec4(0.20, 0.25, 0.33, 1.0);         // Slate-700
                let hover_dark = vec4(0.278, 0.337, 0.412, 1.0);     // Slate-600
                // Active color (same for both)
                let active_color = vec4(0.23, 0.51, 0.96, 1.0);      // Blue

                let base = mix(base_light, base_dark, self.dark_mode);
                let hover_color = mix(hover_light, hover_dark, self.dark_mode);
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
