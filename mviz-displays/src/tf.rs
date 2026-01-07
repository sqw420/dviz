//! TF Display
//!
//! Displays the transform tree with frame axes, labels, and connections.

use std::collections::HashMap;
use std::time::Duration;
use glam::Vec3;
use mviz_core::{DisplayStatus, FrameId, Transform, Timestamp};
use crate::base::{BaseDisplay, DisplayUpdateContext};

/// TF display properties
#[derive(Debug, Clone)]
pub struct TfProperties {
    /// Show frame names as labels
    pub show_names: bool,
    /// Show arrows connecting parent-child frames
    pub show_arrows: bool,
    /// Show coordinate axes at each frame
    pub show_axes: bool,
    /// Scale of axes
    pub axis_scale: f32,
    /// Timeout for marking frames as stale
    pub frame_timeout: Duration,
    /// Update rate limit
    pub update_interval: Duration,
    /// Show all frames (vs only specific frames)
    pub show_all: bool,
}

impl Default for TfProperties {
    fn default() -> Self {
        Self {
            show_names: true,
            show_arrows: true,
            show_axes: true,
            axis_scale: 0.3,
            frame_timeout: Duration::from_secs(5),
            update_interval: Duration::from_millis(100),
            show_all: true,
        }
    }
}

/// State for a single frame in the display
#[derive(Debug, Clone)]
struct FrameState {
    /// Whether this frame is visible
    visible: bool,
    /// Last update time
    last_seen: Timestamp,
    /// Is frame considered stale
    stale: bool,
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            visible: true,
            last_seen: Timestamp::now(),
            stale: false,
        }
    }
}

/// TF display for transform tree visualization
pub struct TfDisplay {
    /// Base display state
    pub base: BaseDisplay,
    /// TF properties
    pub props: TfProperties,
    /// Per-frame state
    frame_states: HashMap<FrameId, FrameState>,
    /// Time of last update
    last_update: std::time::Instant,
}

// Implement common display methods
crate::impl_display_base!(TfDisplay);

impl TfDisplay {
    /// Create a new TF display
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = BaseDisplay::new(name, "tf");

        // Add properties
        base.add_property("show_names", true);
        base.add_property("show_arrows", true);
        base.add_property("show_axes", true);
        base.add_property("axis_scale", 0.3f64);

        Self {
            base,
            props: TfProperties::default(),
            frame_states: HashMap::new(),
            last_update: std::time::Instant::now(),
        }
    }

    /// Set frame visibility
    pub fn set_frame_visible(&mut self, frame: &FrameId, visible: bool) {
        self.frame_states
            .entry(frame.clone())
            .or_default()
            .visible = visible;
    }

    /// Get frame visibility
    pub fn is_frame_visible(&self, frame: &FrameId) -> bool {
        self.frame_states
            .get(frame)
            .map(|s| s.visible)
            .unwrap_or(true)
    }

    /// Log axes for a single frame
    fn log_frame_axes(
        &self,
        ctx: &DisplayUpdateContext,
        frame_id: &FrameId,
        transform: &Transform,
        stale: bool,
    ) -> Result<(), mviz_rerun_bridge::RerunError> {
        let origin = transform.translation;
        let scale = self.props.axis_scale;

        // Use dimmer colors if stale
        let color_mult = if stale { 0.5 } else { 1.0 };

        // Get rotated basis vectors
        let x_axis = transform.transform_vector(Vec3::X) * scale;
        let y_axis = transform.transform_vector(Vec3::Y) * scale;
        let z_axis = transform.transform_vector(Vec3::Z) * scale;

        let frame_path = format!("{}/frames/{}", self.entity_path(), frame_id.as_str());

        // Log X axis (red)
        let x_path = format!("{}/x", frame_path);
        ctx.stream
            .log(
                x_path.as_str(),
                &rerun::Arrows3D::from_vectors([[x_axis.x, x_axis.y, x_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(
                        (255.0 * color_mult) as u8, 0, 0,
                    )]),
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        // Log Y axis (green)
        let y_path = format!("{}/y", frame_path);
        ctx.stream
            .log(
                y_path.as_str(),
                &rerun::Arrows3D::from_vectors([[y_axis.x, y_axis.y, y_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(
                        0, (255.0 * color_mult) as u8, 0,
                    )]),
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        // Log Z axis (blue)
        let z_path = format!("{}/z", frame_path);
        ctx.stream
            .log(
                z_path.as_str(),
                &rerun::Arrows3D::from_vectors([[z_axis.x, z_axis.y, z_axis.z]])
                    .with_origins([[origin.x, origin.y, origin.z]])
                    .with_colors([rerun::Color::from_rgb(
                        0, 0, (255.0 * color_mult) as u8,
                    )]),
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Log frame name label
    fn log_frame_label(
        &self,
        ctx: &DisplayUpdateContext,
        frame_id: &FrameId,
        transform: &Transform,
    ) -> Result<(), mviz_rerun_bridge::RerunError> {
        let label_path = format!("{}/labels/{}", self.entity_path(), frame_id.as_str());
        let pos = transform.translation;

        ctx.stream
            .log(
                label_path.as_str(),
                &rerun::Points3D::new([[pos.x, pos.y, pos.z + self.props.axis_scale * 0.1]])
                    .with_labels([frame_id.as_str()])
                    .with_radii([0.005]),
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Log parent-child connection arrow
    fn log_connection(
        &self,
        ctx: &DisplayUpdateContext,
        parent: &Transform,
        child: &Transform,
        parent_id: &FrameId,
        child_id: &FrameId,
    ) -> Result<(), mviz_rerun_bridge::RerunError> {
        let path = format!(
            "{}/connections/{}->{}",
            self.entity_path(),
            parent_id.as_str(),
            child_id.as_str()
        );

        let start = parent.translation;
        let end = child.translation;

        ctx.stream
            .log(
                path.as_str(),
                &rerun::LineStrips3D::new([vec![
                    [start.x, start.y, start.z],
                    [end.x, end.y, end.z],
                ]])
                .with_colors([rerun::Color::from_unmultiplied_rgba(255, 255, 0, 128)]),
            )
            .map_err(|e| mviz_rerun_bridge::RerunError::LogError(e.to_string()))?;

        Ok(())
    }

    /// Update the display
    pub fn update(&mut self, ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Rate limit updates
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update) < self.props.update_interval {
            return Ok(());
        }
        self.last_update = now;

        // Get all frames from the transform buffer
        let all_frames = ctx.transform_buffer.all_frames();

        for frame_id in &all_frames {
            // Check visibility
            if !self.props.show_all && !self.is_frame_visible(frame_id) {
                continue;
            }

            // Look up transform to fixed frame
            let transform = match ctx.transform_buffer.lookup_transform(
                ctx.fixed_frame,
                frame_id,
                ctx.current_time,
            ) {
                Ok(tf) => tf,
                Err(_) => continue, // Skip frames we can't transform
            };

            // Update frame state
            let state = self.frame_states.entry(frame_id.clone()).or_default();
            state.last_seen = *ctx.current_time;
            let stale = state.stale;

            // Log axes if enabled
            if self.props.show_axes {
                self.log_frame_axes(ctx, frame_id, &transform, stale)?;
            }

            // Log labels if enabled
            if self.props.show_names {
                self.log_frame_label(ctx, frame_id, &transform)?;
            }

            // Log connections if enabled
            if self.props.show_arrows {
                if let Some(parent_id) = ctx.transform_buffer.frame_tree().parent(frame_id) {
                    if let Ok(parent_tf) = ctx.transform_buffer.lookup_transform(
                        ctx.fixed_frame,
                        &parent_id,
                        ctx.current_time,
                    ) {
                        self.log_connection(ctx, &parent_tf, &transform, &parent_id, frame_id)?;
                    }
                }
            }
        }

        // Check for stale frames
        let timeout_ns = self.props.frame_timeout.as_nanos() as i64;
        for (_frame_id, state) in &mut self.frame_states {
            let age = ctx.current_time.as_nanos() - state.last_seen.as_nanos();
            state.stale = age > timeout_ns;
        }

        // Update status
        let total_frames = all_frames.len();
        let stale_count = self.frame_states.values().filter(|s| s.stale).count();

        if stale_count > 0 {
            self.set_status(
                DisplayStatus::Warning,
                Some(format!("{} stale frames", stale_count)),
            );
        } else {
            self.set_status(
                DisplayStatus::Ok,
                Some(format!("{} frames", total_frames)),
            );
        }

        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self, _ctx: &DisplayUpdateContext) -> Result<(), mviz_rerun_bridge::RerunError> {
        self.frame_states.clear();
        Ok(())
    }

    /// Reset the display
    pub fn reset(&mut self) {
        self.props = TfProperties::default();
        self.frame_states.clear();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tf_properties_default() {
        let props = TfProperties::default();
        assert!(props.show_names);
        assert!(props.show_arrows);
        assert!(props.show_axes);
        assert_eq!(props.axis_scale, 0.3);
    }

    #[test]
    fn test_tf_display_creation() {
        let tf = TfDisplay::new("TF Tree");
        assert_eq!(tf.name(), "TF Tree");
        assert_eq!(tf.type_name(), "tf");
        assert!(tf.is_enabled());
    }

    #[test]
    fn test_tf_frame_visibility() {
        let mut tf = TfDisplay::new("TF");
        let frame = FrameId::new("test_frame");

        assert!(tf.is_frame_visible(&frame));

        tf.set_frame_visible(&frame, false);
        assert!(!tf.is_frame_visible(&frame));

        tf.set_frame_visible(&frame, true);
        assert!(tf.is_frame_visible(&frame));
    }
}
