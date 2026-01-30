//! Axes Display
//!
//! Displays RGB coordinate frame axes (red=X, green=Y, blue=Z).

use glam::Vec3;
use dviz_core::{FrameId, Transform};
use crate::base::{BaseDisplay, DisplayUpdateContext};

/// Axes display properties
#[derive(Debug, Clone)]
pub struct AxesProperties {
    /// Reference frame to display axes for
    pub frame: FrameId,
    /// Scale of axes in meters
    pub scale: f32,
    /// Line width
    pub line_width: f32,
    /// Show axis labels
    pub show_labels: bool,
}

impl Default for AxesProperties {
    fn default() -> Self {
        Self {
            frame: FrameId::default(),
            scale: 1.0,
            line_width: 2.0,
            show_labels: false,
        }
    }
}

/// Axes display for coordinate frame visualization
pub struct AxesDisplay {
    /// Base display state
    pub base: BaseDisplay,
    /// Axes properties
    pub props: AxesProperties,
}

// Implement common display methods
crate::impl_display_base!(AxesDisplay);

impl AxesDisplay {
    /// Create a new axes display
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = BaseDisplay::new(name, "axes");

        // Add properties
        base.add_property("frame", "");
        base.add_property("scale", 1.0f64);
        base.add_property("show_labels", false);

        Self {
            base,
            props: AxesProperties::default(),
        }
    }

    /// Set the reference frame
    pub fn set_frame(&mut self, frame: impl Into<FrameId>) {
        self.props.frame = frame.into();
    }

    /// Set the axes scale
    pub fn set_scale(&mut self, scale: f32) {
        self.props.scale = scale;
    }

    /// Set whether to show labels
    pub fn set_show_labels(&mut self, show: bool) {
        self.props.show_labels = show;
    }

    /// Log axes for a transform
    pub fn log_axes(
        &self,
        ctx: &DisplayUpdateContext,
        transform: &Transform,
    ) -> Result<(), dviz_rerun_bridge::RerunError> {
        let origin = transform.translation;
        let scale = self.props.scale;

        // Get rotated basis vectors
        let x_axis = transform.transform_vector(Vec3::X) * scale;
        let y_axis = transform.transform_vector(Vec3::Y) * scale;
        let z_axis = transform.transform_vector(Vec3::Z) * scale;

        // Log X axis (red)
        let x_path = format!("{}/x", self.entity_path());
        ctx.stream
            .log(
                x_path.as_str(),
                &rerun::Arrows3D::from_vectors([[x_axis.x, x_axis.y, x_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(255, 0, 0)]),
            )
            .map_err(|e| dviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        // Log Y axis (green)
        let y_path = format!("{}/y", self.entity_path());
        ctx.stream
            .log(
                y_path.as_str(),
                &rerun::Arrows3D::from_vectors([[y_axis.x, y_axis.y, y_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(0, 255, 0)]),
            )
            .map_err(|e| dviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        // Log Z axis (blue)
        let z_path = format!("{}/z", self.entity_path());
        ctx.stream
            .log(
                z_path.as_str(),
                &rerun::Arrows3D::from_vectors([[z_axis.x, z_axis.y, z_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(0, 0, 255)]),
            )
            .map_err(|e| dviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        // Log labels if enabled
        if self.props.show_labels {
            let label_path = format!("{}/labels", self.entity_path());
            let label_positions = vec![
                [origin.x + x_axis.x, origin.y + x_axis.y, origin.z + x_axis.z],
                [origin.x + y_axis.x, origin.y + y_axis.y, origin.z + y_axis.z],
                [origin.x + z_axis.x, origin.y + z_axis.y, origin.z + z_axis.z],
            ];
            ctx.stream
                .log(
                    label_path.as_str(),
                    &rerun::Points3D::new(label_positions)
                        .with_labels(["X", "Y", "Z"])
                        .with_radii([0.01]),
                )
                .map_err(|e| dviz_rerun_bridge::RerunError::LogError(e.to_string()))?;
        }

        Ok(())
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), dviz_rerun_bridge::RerunError> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Look up transform from frame to fixed frame
        let transform = if self.props.frame.as_str().is_empty() || self.props.frame == *ctx.fixed_frame {
            Transform::IDENTITY
        } else {
            match ctx.transform_buffer.lookup_transform(
                ctx.fixed_frame,
                &self.props.frame,
                ctx.current_time,
            ) {
                Ok(tf) => tf,
                Err(e) => {
                    self.set_status(
                        dviz_core::DisplayStatus::Warning,
                        Some(format!("Transform error: {}", e)),
                    );
                    return Ok(());
                }
            }
        };

        self.log_axes(ctx, &transform)?;
        self.clear_status();

        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self, _ctx: &DisplayUpdateContext) -> Result<(), dviz_rerun_bridge::RerunError> {
        Ok(())
    }

    /// Reset the display
    pub fn reset(&mut self) {
        self.props = AxesProperties::default();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axes_properties_default() {
        let props = AxesProperties::default();
        assert_eq!(props.scale, 1.0);
        assert!(!props.show_labels);
    }

    #[test]
    fn test_axes_display_creation() {
        let axes = AxesDisplay::new("Frame Axes");
        assert_eq!(axes.name(), "Frame Axes");
        assert_eq!(axes.type_name(), "axes");
        assert!(axes.is_enabled());
    }

    #[test]
    fn test_axes_display_set_frame() {
        let mut axes = AxesDisplay::new("Test");
        axes.set_frame("base_link");
        assert_eq!(axes.props.frame.as_str(), "base_link");
    }

    #[test]
    fn test_axes_display_set_scale() {
        let mut axes = AxesDisplay::new("Test");
        axes.set_scale(0.5);
        assert_eq!(axes.props.scale, 0.5);
    }
}
