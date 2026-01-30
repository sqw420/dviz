//! Point cloud types for 3D visualization
//!
//! Types for representing point cloud data with various coloring modes.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use super::transform::{FrameId, Timestamp};

// ============================================================================
// COLOR
// ============================================================================

/// RGBA color with u8 components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    // Common colors
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0, a: 255 };
    pub const CYAN: Self = Self { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Self = Self { r: 255, g: 0, b: 255, a: 255 };
    pub const ORANGE: Self = Self { r: 255, g: 165, b: 0, a: 255 };
    pub const GRAY: Self = Self { r: 128, g: 128, b: 128, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    /// Create from RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create from RGBA values
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create from normalized float RGB [0.0, 1.0]
    pub fn from_rgb_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: 255,
        }
    }

    /// Create from normalized float RGBA [0.0, 1.0]
    pub fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    /// Create from HSV (hue: 0-360, saturation: 0-1, value: 0-1)
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let (r, g, b) = hsv_to_rgb(h, s, v);
        Self::from_rgb_f32(r, g, b)
    }

    /// Convert to normalized float array [r, g, b, a]
    pub fn to_rgba_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Convert to u8 array [r, g, b, a]
    pub fn to_rgba(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert to u8 array [r, g, b]
    pub fn to_rgb(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// Apply alpha
    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Blend with another color
    pub fn blend(&self, other: &Color, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;
        Self {
            r: (self.r as f32 * inv_t + other.r as f32 * t) as u8,
            g: (self.g as f32 * inv_t + other.g as f32 * t) as u8,
            b: (self.b as f32 * inv_t + other.b as f32 * t) as u8,
            a: (self.a as f32 * inv_t + other.a as f32 * t) as u8,
        }
    }
}

impl From<[u8; 3]> for Color {
    fn from(rgb: [u8; 3]) -> Self {
        Self::rgb(rgb[0], rgb[1], rgb[2])
    }
}

impl From<[u8; 4]> for Color {
    fn from(rgba: [u8; 4]) -> Self {
        Self::rgba(rgba[0], rgba[1], rgba[2], rgba[3])
    }
}

/// Convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    if s <= 0.0 {
        return (v, v, v);
    }

    let h = h % 360.0;
    let h = h / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

// ============================================================================
// COLORMAP
// ============================================================================

/// Predefined colormaps for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Colormap {
    #[default]
    Jet,
    Rainbow,
    Turbo,
    Viridis,
    Grayscale,
    Hot,
    Cool,
    Plasma,
}

impl Colormap {
    /// Sample the colormap at position t (0.0 to 1.0)
    pub fn sample(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        match self {
            Colormap::Jet => jet_colormap(t),
            Colormap::Rainbow => rainbow_colormap(t),
            Colormap::Turbo => turbo_colormap(t),
            Colormap::Viridis => viridis_colormap(t),
            Colormap::Grayscale => grayscale_colormap(t),
            Colormap::Hot => hot_colormap(t),
            Colormap::Cool => cool_colormap(t),
            Colormap::Plasma => plasma_colormap(t),
        }
    }
}

fn jet_colormap(t: f32) -> Color {
    let r = (1.5 - (4.0 * t - 3.0).abs()).clamp(0.0, 1.0);
    let g = (1.5 - (4.0 * t - 2.0).abs()).clamp(0.0, 1.0);
    let b = (1.5 - (4.0 * t - 1.0).abs()).clamp(0.0, 1.0);
    Color::from_rgb_f32(r, g, b)
}

fn rainbow_colormap(t: f32) -> Color {
    Color::from_hsv(t * 300.0, 1.0, 1.0)
}

fn turbo_colormap(t: f32) -> Color {
    // Simplified turbo approximation
    let r = (0.13572138 + t * (4.61539260 + t * (-42.66032258 + t * (132.13108234 + t * (-152.94239396 + t * 59.28637943)))));
    let g = (0.09140261 + t * (2.19418839 + t * (4.84296658 + t * (-14.18503333 + t * (4.27729857 + t * 2.82956604)))));
    let b = (0.10667330 + t * (12.64194608 + t * (-60.58204836 + t * (110.36276771 + t * (-89.90310912 + t * 27.34824973)))));
    Color::from_rgb_f32(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

fn viridis_colormap(t: f32) -> Color {
    // Simplified viridis approximation
    let r = (0.267004 + t * (0.282327 + t * (0.348907 - t * 0.048310)));
    let g = (0.004874 + t * (0.873465 + t * (-0.333590 + t * 0.447890)));
    let b = (0.329415 + t * (-0.015357 + t * (-0.548790 + t * 0.780290)));
    Color::from_rgb_f32(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

fn grayscale_colormap(t: f32) -> Color {
    Color::from_rgb_f32(t, t, t)
}

fn hot_colormap(t: f32) -> Color {
    let r = (t * 3.0).clamp(0.0, 1.0);
    let g = ((t - 0.333) * 3.0).clamp(0.0, 1.0);
    let b = ((t - 0.666) * 3.0).clamp(0.0, 1.0);
    Color::from_rgb_f32(r, g, b)
}

fn cool_colormap(t: f32) -> Color {
    Color::from_rgb_f32(t, 1.0 - t, 1.0)
}

fn plasma_colormap(t: f32) -> Color {
    // Simplified plasma approximation
    let r = (0.05 + t * (1.5 - t * 0.5)).clamp(0.0, 1.0);
    let g = (t * t * 0.8).clamp(0.0, 1.0);
    let b = (0.53 + t * (-0.58 + t * 0.55)).clamp(0.0, 1.0);
    Color::from_rgb_f32(r, g, b)
}

// ============================================================================
// COLOR MODE
// ============================================================================

/// Axis for color mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Axis {
    X,
    Y,
    #[default]
    Z,
}

impl Axis {
    /// Get axis component from a Vec3
    pub fn get(&self, v: Vec3) -> f32 {
        match self {
            Axis::X => v.x,
            Axis::Y => v.y,
            Axis::Z => v.z,
        }
    }
}

/// How to color point cloud points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorMode {
    /// Single flat color for all points
    FlatColor(Color),
    /// Use RGB color from point data
    RGB,
    /// Color by intensity value
    Intensity {
        min: f32,
        max: f32,
        colormap: Colormap,
    },
    /// Color by axis position
    AxisColor {
        axis: Axis,
        min: f32,
        max: f32,
        colormap: Colormap,
    },
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::FlatColor(Color::WHITE)
    }
}

impl ColorMode {
    /// Create intensity color mode with auto range
    pub fn intensity(colormap: Colormap) -> Self {
        ColorMode::Intensity {
            min: 0.0,
            max: 1.0,
            colormap,
        }
    }

    /// Create axis color mode (typically Z for height coloring)
    pub fn axis(axis: Axis, min: f32, max: f32, colormap: Colormap) -> Self {
        ColorMode::AxisColor { axis, min, max, colormap }
    }

    /// Create Z-height color mode
    pub fn z_height(min: f32, max: f32) -> Self {
        Self::axis(Axis::Z, min, max, Colormap::Jet)
    }
}

// ============================================================================
// POINT CLOUD STYLE
// ============================================================================

/// Visual style for point cloud rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PointCloudStyle {
    #[default]
    Points,
    Squares,
    Circles,
    Spheres,
    Boxes,
}

// ============================================================================
// POINT CLOUD
// ============================================================================

/// 3D point cloud data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    /// Point positions
    pub positions: Vec<Vec3>,
    /// Optional per-point colors
    pub colors: Option<Vec<Color>>,
    /// Optional per-point intensities
    pub intensities: Option<Vec<f32>>,
    /// Optional per-point normals
    pub normals: Option<Vec<Vec3>>,
    /// Reference frame
    pub frame_id: FrameId,
    /// Timestamp
    pub timestamp: Timestamp,
}

impl Default for PointCloud {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            colors: None,
            intensities: None,
            normals: None,
            frame_id: FrameId::default(),
            timestamp: Timestamp::default(),
        }
    }
}

impl PointCloud {
    /// Create empty point cloud
    pub fn new(frame_id: impl Into<FrameId>) -> Self {
        Self {
            frame_id: frame_id.into(),
            timestamp: Timestamp::now(),
            ..Default::default()
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize, frame_id: impl Into<FrameId>) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            colors: None,
            intensities: None,
            normals: None,
            frame_id: frame_id.into(),
            timestamp: Timestamp::now(),
        }
    }

    /// Create from positions
    pub fn from_positions(positions: Vec<Vec3>, frame_id: impl Into<FrameId>) -> Self {
        Self {
            positions,
            frame_id: frame_id.into(),
            timestamp: Timestamp::now(),
            ..Default::default()
        }
    }

    /// Number of points
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Add a point
    pub fn push(&mut self, position: Vec3) {
        self.positions.push(position);
    }

    /// Add a point with color
    pub fn push_with_color(&mut self, position: Vec3, color: Color) {
        self.positions.push(position);
        if self.colors.is_none() {
            self.colors = Some(Vec::with_capacity(self.positions.capacity()));
        }
        self.colors.as_mut().unwrap().push(color);
    }

    /// Add a point with intensity
    pub fn push_with_intensity(&mut self, position: Vec3, intensity: f32) {
        self.positions.push(position);
        if self.intensities.is_none() {
            self.intensities = Some(Vec::with_capacity(self.positions.capacity()));
        }
        self.intensities.as_mut().unwrap().push(intensity);
    }

    /// Set colors for all points
    pub fn set_colors(&mut self, colors: Vec<Color>) {
        assert_eq!(colors.len(), self.positions.len(), "Color count must match point count");
        self.colors = Some(colors);
    }

    /// Set intensities for all points
    pub fn set_intensities(&mut self, intensities: Vec<f32>) {
        assert_eq!(intensities.len(), self.positions.len(), "Intensity count must match point count");
        self.intensities = Some(intensities);
    }

    /// Compute axis-aligned bounding box
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        if self.is_empty() {
            return None;
        }

        let mut min = self.positions[0];
        let mut max = self.positions[0];

        for p in &self.positions {
            min = min.min(*p);
            max = max.max(*p);
        }

        Some((min, max))
    }

    /// Compute colors based on ColorMode
    pub fn compute_colors(&self, mode: &ColorMode) -> Vec<Color> {
        match mode {
            ColorMode::FlatColor(color) => vec![*color; self.len()],
            ColorMode::RGB => {
                self.colors.clone().unwrap_or_else(|| vec![Color::WHITE; self.len()])
            }
            ColorMode::Intensity { min, max, colormap } => {
                if let Some(intensities) = &self.intensities {
                    intensities.iter().map(|&i| {
                        let t = ((i - min) / (max - min)).clamp(0.0, 1.0);
                        colormap.sample(t)
                    }).collect()
                } else {
                    vec![Color::WHITE; self.len()]
                }
            }
            ColorMode::AxisColor { axis, min, max, colormap } => {
                self.positions.iter().map(|&p| {
                    let v = axis.get(p);
                    let t = ((v - min) / (max - min)).clamp(0.0, 1.0);
                    colormap.sample(t)
                }).collect()
            }
        }
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.positions.clear();
        if let Some(ref mut colors) = self.colors {
            colors.clear();
        }
        if let Some(ref mut intensities) = self.intensities {
            intensities.clear();
        }
        if let Some(ref mut normals) = self.normals {
            normals.clear();
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::RED.to_rgb(), [255, 0, 0]);
        assert_eq!(Color::GREEN.to_rgb(), [0, 255, 0]);
        assert_eq!(Color::BLUE.to_rgb(), [0, 0, 255]);
    }

    #[test]
    fn test_color_from_hsv() {
        let red = Color::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(red.r, 255);
        assert!(red.g < 5);
        assert!(red.b < 5);

        let green = Color::from_hsv(120.0, 1.0, 1.0);
        assert!(green.r < 5);
        assert_eq!(green.g, 255);
        assert!(green.b < 5);
    }

    #[test]
    fn test_color_blend() {
        let c1 = Color::BLACK;
        let c2 = Color::WHITE;
        let mid = c1.blend(&c2, 0.5);
        assert!((mid.r as i32 - 127).abs() <= 1);
        assert!((mid.g as i32 - 127).abs() <= 1);
        assert!((mid.b as i32 - 127).abs() <= 1);
    }

    #[test]
    fn test_colormap_jet() {
        let low = Colormap::Jet.sample(0.0);
        let mid = Colormap::Jet.sample(0.5);
        let high = Colormap::Jet.sample(1.0);

        // Jet goes blue -> cyan -> green -> yellow -> red
        assert!(low.b > low.r); // Blue at low
        assert!(mid.g > mid.r && mid.g > mid.b); // Green in middle
        assert!(high.r > high.b); // Red at high
    }

    #[test]
    fn test_point_cloud_basic() {
        let mut cloud = PointCloud::new("base_link");
        cloud.push(Vec3::new(1.0, 2.0, 3.0));
        cloud.push(Vec3::new(4.0, 5.0, 6.0));

        assert_eq!(cloud.len(), 2);
        assert!(!cloud.is_empty());
    }

    #[test]
    fn test_point_cloud_bounds() {
        let cloud = PointCloud::from_positions(
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::new(-1.0, -2.0, -3.0),
            ],
            "test",
        );

        let (min, max) = cloud.bounds().unwrap();
        assert_eq!(min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_point_cloud_color_mode() {
        let cloud = PointCloud::from_positions(
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 5.0),
                Vec3::new(0.0, 0.0, 10.0),
            ],
            "test",
        );

        let colors = cloud.compute_colors(&ColorMode::z_height(0.0, 10.0));
        assert_eq!(colors.len(), 3);
        // First point should be different from last point
        assert_ne!(colors[0], colors[2]);
    }

    #[test]
    fn test_axis() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(Axis::X.get(v), 1.0);
        assert_eq!(Axis::Y.get(v), 2.0);
        assert_eq!(Axis::Z.get(v), 3.0);
    }
}
