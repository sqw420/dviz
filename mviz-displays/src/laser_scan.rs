//! Laser Scan Display
//!
//! Visualizes 2D laser scanner data as point clouds.

use crate::base::{BaseDisplay, DisplayUpdateContext};
use crate::point_cloud::PointCloudDisplay;
use glam::Vec3;
use mviz_core::display::{DisplayError, DisplayInfo, PropertyMeta, PropertyValue};
use mviz_core::types::{Axis, Color, ColorMode, Colormap, PointCloud, FrameId, Timestamp};

/// Laser scan data
#[derive(Debug, Clone)]
pub struct LaserScanData {
    /// Minimum angle (radians)
    pub angle_min: f32,
    /// Maximum angle (radians)
    pub angle_max: f32,
    /// Angle increment between measurements (radians)
    pub angle_increment: f32,
    /// Minimum valid range (meters)
    pub range_min: f32,
    /// Maximum valid range (meters)
    pub range_max: f32,
    /// Range measurements (meters)
    pub ranges: Vec<f32>,
    /// Intensity measurements (optional)
    pub intensities: Option<Vec<f32>>,
    /// Frame ID
    pub frame_id: FrameId,
    /// Timestamp
    pub timestamp: Timestamp,
}

impl LaserScanData {
    /// Create new laser scan data
    pub fn new(
        angle_min: f32,
        angle_max: f32,
        angle_increment: f32,
        ranges: Vec<f32>,
    ) -> Self {
        Self {
            angle_min,
            angle_max,
            angle_increment,
            range_min: 0.0,
            range_max: f32::MAX,
            ranges,
            intensities: None,
            frame_id: FrameId::new("laser"),
            timestamp: Timestamp::now(),
        }
    }

    /// Set range limits
    pub fn with_range_limits(mut self, min: f32, max: f32) -> Self {
        self.range_min = min;
        self.range_max = max;
        self
    }

    /// Set intensities
    pub fn with_intensities(mut self, intensities: Vec<f32>) -> Self {
        self.intensities = Some(intensities);
        self
    }

    /// Set frame ID
    pub fn with_frame(mut self, frame_id: FrameId) -> Self {
        self.frame_id = frame_id;
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Number of measurements
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Convert polar coordinates to Cartesian point cloud
    pub fn to_point_cloud(&self) -> PointCloud {
        let mut positions = Vec::with_capacity(self.ranges.len());
        let mut intensities = Vec::with_capacity(self.ranges.len());

        for (i, &range) in self.ranges.iter().enumerate() {
            // Skip invalid ranges
            if !range.is_finite() || range < self.range_min || range > self.range_max {
                continue;
            }

            // Calculate angle for this ray
            let angle = self.angle_min + (i as f32) * self.angle_increment;

            // Convert polar to Cartesian (laser scanner is typically in XY plane)
            let x = range * angle.cos();
            let y = range * angle.sin();
            let z = 0.0;

            positions.push(Vec3::new(x, y, z));
            intensities.push(
                self.intensities
                    .as_ref()
                    .and_then(|int| int.get(i).copied())
                    .unwrap_or(1.0)
            );
        }

        let mut cloud = PointCloud::from_positions(positions, self.frame_id.clone());
        cloud.timestamp = self.timestamp;
        cloud.set_intensities(intensities);
        cloud
    }
}

/// Laser scan display properties
#[derive(Debug, Clone)]
pub struct LaserScanProperties {
    /// Color mode for points
    pub color_mode: LaserScanColorMode,
    /// Point size
    pub point_size: f32,
    /// Alpha transparency (0-1)
    pub alpha: f32,
    /// Decay time (0 = no decay, keeps only latest scan)
    pub decay_time_secs: f32,
}

impl Default for LaserScanProperties {
    fn default() -> Self {
        Self {
            color_mode: LaserScanColorMode::Range,
            point_size: 0.02,
            alpha: 1.0,
            decay_time_secs: 0.0,
        }
    }
}

/// Color modes for laser scan visualization
#[derive(Debug, Clone)]
pub enum LaserScanColorMode {
    /// Flat color
    Flat(Color),
    /// Color by range (distance)
    Range,
    /// Color by intensity
    Intensity,
    /// Color by angle
    Angle,
}

impl LaserScanColorMode {
    /// Convert to point cloud color mode
    fn to_point_cloud_mode(&self, scan: &LaserScanData) -> ColorMode {
        match self {
            LaserScanColorMode::Flat(color) => ColorMode::FlatColor(*color),
            LaserScanColorMode::Range => {
                // Color by distance from origin (X axis for radial distance)
                ColorMode::axis(Axis::X, scan.range_min, scan.range_max, Colormap::Jet)
            }
            LaserScanColorMode::Intensity => {
                ColorMode::intensity(Colormap::Turbo)
            }
            LaserScanColorMode::Angle => {
                // Use Y axis for angle coloring (Y varies with angle)
                ColorMode::axis(Axis::Y, -scan.range_max, scan.range_max, Colormap::Rainbow)
            }
        }
    }
}

/// Laser scan display
pub struct LaserScanDisplay {
    base: BaseDisplay,
    properties: LaserScanProperties,
    /// Underlying point cloud display for rendering
    point_cloud_display: PointCloudDisplay,
    /// Most recent scan
    current_scan: Option<LaserScanData>,
}

impl LaserScanDisplay {
    /// Create a new laser scan display
    pub fn new(name: &str) -> Self {
        let mut base = BaseDisplay::new(name, "laser_scan");
        base.entity_path = "laser_scan".to_string();

        // Add properties
        base.add_property("point_size", PropertyValue::Float(0.02));
        base.add_property("alpha", PropertyValue::Float(1.0));
        base.add_property("decay_time", PropertyValue::Float(0.0));

        let mut point_cloud_display = PointCloudDisplay::new(&format!("{}_points", name));
        point_cloud_display.set_point_size(0.02);

        Self {
            base,
            properties: LaserScanProperties::default(),
            point_cloud_display,
            current_scan: None,
        }
    }

    /// Set the current scan
    pub fn set_scan(&mut self, scan: LaserScanData) {
        // Convert to point cloud
        let point_cloud = scan.to_point_cloud();

        // Set color mode based on our settings
        let color_mode = self.properties.color_mode.to_point_cloud_mode(&scan);
        self.point_cloud_display.set_color_mode(color_mode);

        // Add to point cloud display
        self.point_cloud_display.add_cloud(point_cloud);

        self.current_scan = Some(scan);
    }

    /// Set color mode
    pub fn set_color_mode(&mut self, mode: LaserScanColorMode) {
        self.properties.color_mode = mode;
    }

    /// Set point size
    pub fn set_point_size(&mut self, size: f32) {
        self.properties.point_size = size;
        self.point_cloud_display.set_point_size(size);
        self.base.properties.set("point_size", PropertyValue::Float(size as f64));
    }

    /// Set alpha transparency
    pub fn set_alpha(&mut self, alpha: f32) {
        self.properties.alpha = alpha.clamp(0.0, 1.0);
        self.point_cloud_display.props.alpha = self.properties.alpha;
        self.base.properties.set("alpha", PropertyValue::Float(self.properties.alpha as f64));
    }

    /// Set decay time in seconds
    pub fn set_decay_time(&mut self, seconds: f32) {
        self.properties.decay_time_secs = seconds;
        self.point_cloud_display.set_decay_time(std::time::Duration::from_secs_f32(seconds));
        self.base.properties.set("decay_time", PropertyValue::Float(seconds as f64));
    }

    /// Get current scan
    pub fn current_scan(&self) -> Option<&LaserScanData> {
        self.current_scan.as_ref()
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), DisplayError> {
        if !self.base.enabled {
            return Ok(());
        }

        self.point_cloud_display.update(ctx)
            .map_err(|e| DisplayError::RenderError(e.to_string()))
    }

    /// Get display info
    pub fn info(&self) -> DisplayInfo {
        DisplayInfo::new(
            "laser_scan",
            "Laser Scan",
            "Visualizes 2D laser scanner data",
            "mviz",
        )
    }

    /// Get property metadata
    pub fn property_meta(&self) -> Vec<PropertyMeta> {
        vec![
            PropertyMeta::new("point_size", "Point Size", "Size of points in meters", 0.02f64),
            PropertyMeta::new("alpha", "Alpha", "Transparency (0-1)", 1.0f64),
            PropertyMeta::new("decay_time", "Decay Time", "Time before old scans fade (0 = no decay)", 0.0f64),
        ]
    }
}

/// Create a simulated laser scan for testing
pub fn simulate_laser_scan(num_rays: usize, max_range: f32) -> LaserScanData {
    use std::f32::consts::PI;

    let angle_min = -PI;
    let angle_max = PI;
    let angle_increment = (angle_max - angle_min) / (num_rays - 1) as f32;

    let mut ranges = Vec::with_capacity(num_rays);
    let mut intensities = Vec::with_capacity(num_rays);

    for i in 0..num_rays {
        let angle = angle_min + (i as f32) * angle_increment;

        // Simulate a circular room with some noise
        let base_range = max_range * 0.8;
        let noise = (angle * 5.0).sin() * 0.2 + (angle * 3.0).cos() * 0.1;
        let range = (base_range + noise * base_range).clamp(0.1, max_range);

        ranges.push(range);
        intensities.push((range / max_range).clamp(0.0, 1.0));
    }

    LaserScanData::new(angle_min, angle_max, angle_increment, ranges)
        .with_range_limits(0.1, max_range)
        .with_intensities(intensities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laser_scan_data_creation() {
        let ranges = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let scan = LaserScanData::new(-1.57, 1.57, 0.785, ranges.clone());

        assert_eq!(scan.len(), 5);
        assert!(!scan.is_empty());
        assert_eq!(scan.angle_min, -1.57);
        assert_eq!(scan.angle_max, 1.57);
    }

    #[test]
    fn test_to_point_cloud() {
        let ranges = vec![1.0, 1.0, 1.0, 1.0];
        let scan = LaserScanData::new(0.0, std::f32::consts::PI, std::f32::consts::FRAC_PI_2, ranges)
            .with_range_limits(0.0, 10.0);

        let cloud = scan.to_point_cloud();

        assert_eq!(cloud.len(), 4);

        // First point should be at (1, 0, 0) - angle 0
        let p0 = cloud.positions[0];
        assert!((p0.x - 1.0).abs() < 0.01);
        assert!(p0.y.abs() < 0.01);

        // Second point should be at (0, 1, 0) - angle PI/2
        let p1 = cloud.positions[1];
        assert!(p1.x.abs() < 0.01);
        assert!((p1.y - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_invalid_ranges_filtered() {
        let ranges = vec![1.0, f32::NAN, f32::INFINITY, -1.0, 2.0];
        let scan = LaserScanData::new(0.0, 1.0, 0.25, ranges)
            .with_range_limits(0.0, 10.0);

        let cloud = scan.to_point_cloud();

        // Only valid ranges (1.0 and 2.0) should be included
        assert_eq!(cloud.len(), 2);
    }

    #[test]
    fn test_laser_scan_display_creation() {
        let display = LaserScanDisplay::new("Test Laser");
        assert!(display.current_scan().is_none());
    }

    #[test]
    fn test_set_scan() {
        let mut display = LaserScanDisplay::new("Test Laser");

        let ranges = vec![1.0, 2.0, 3.0];
        let scan = LaserScanData::new(-1.0, 1.0, 1.0, ranges);

        display.set_scan(scan);
        assert!(display.current_scan().is_some());
    }

    #[test]
    fn test_properties() {
        let mut display = LaserScanDisplay::new("Test");

        display.set_point_size(0.05);
        assert!((display.properties.point_size - 0.05).abs() < 0.001);

        display.set_alpha(0.5);
        assert!((display.properties.alpha - 0.5).abs() < 0.001);

        display.set_decay_time(2.0);
        assert!((display.properties.decay_time_secs - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_simulate_laser_scan() {
        let scan = simulate_laser_scan(360, 10.0);

        assert_eq!(scan.len(), 360);
        assert!(scan.intensities.is_some());

        // All ranges should be valid
        for &range in &scan.ranges {
            assert!(range.is_finite());
            assert!(range >= scan.range_min);
            assert!(range <= scan.range_max);
        }
    }

    #[test]
    fn test_color_modes() {
        let scan = LaserScanData::new(-1.0, 1.0, 0.5, vec![1.0, 2.0, 3.0])
            .with_range_limits(0.0, 5.0);

        // Test each color mode converts properly
        let flat = LaserScanColorMode::Flat(Color::RED);
        let _mode = flat.to_point_cloud_mode(&scan);

        let range = LaserScanColorMode::Range;
        let _mode = range.to_point_cloud_mode(&scan);

        let intensity = LaserScanColorMode::Intensity;
        let _mode = intensity.to_point_cloud_mode(&scan);

        let angle = LaserScanColorMode::Angle;
        let _mode = angle.to_point_cloud_mode(&scan);
    }
}
